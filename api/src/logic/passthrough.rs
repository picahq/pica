use super::get_connection;
use crate::{domain::metrics::Metric, server::AppState};
use axum::{
    extract::{Query, State},
    response::IntoResponse,
    routing::get,
    Extension, Router,
};
use bson::doc;
use cache::local::LocalCacheExt;
use chrono::Utc;
use http::{header::CONTENT_LENGTH, HeaderMap, HeaderName, Method, Uri};
use hyper::body::Bytes;
use mongodb::options::FindOneOptions;
use osentities::connection_model_definition::SparseCMD;
use osentities::{
    constant::PICA_PASSTHROUGH_HEADER,
    destination::{Action, Destination},
    encrypted_access_key::EncryptedAccessKey,
    event_access::EventAccess,
    prefix::IdPrefix,
    AccessKey, ApplicationError, Event, Id, InternalError, Store, META, PASSWORD_LENGTH,
    QUERY_BY_ID_PASSTHROUGH,
};
use rand::distributions::{Alphanumeric, DistString};
use serde_json::json;
use std::{collections::HashMap, sync::Arc};
use tracing::{error, info};
use unified::domain::UnifiedMetadataBuilder;

pub fn get_router() -> Router<Arc<AppState>> {
    Router::new().route(
        "/*key",
        get(passthrough_request)
            .post(passthrough_request)
            .patch(passthrough_request)
            .put(passthrough_request)
            .delete(passthrough_request),
    )
}

pub async fn passthrough_request(
    Extension(user_event_access): Extension<Arc<EventAccess>>,
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    query_params: Option<Query<HashMap<String, String>>>,
    uri: Uri,
    method: Method,
    body: Bytes,
) -> impl IntoResponse {
    let Some(connection_key_header) = headers.get(&state.config.headers.connection_header) else {
        return Err(ApplicationError::bad_request(
            "Connection key header not found",
            None,
        ));
    };

    let Some(connection_secret_header) = headers.get(&state.config.headers.auth_header) else {
        return Err(ApplicationError::bad_request("Auth header not found", None));
    };

    let host = headers
        .get("host")
        .and_then(|h| h.to_str().map(|s| s.to_string()).ok());

    let connection = get_connection(
        user_event_access.as_ref(),
        connection_key_header,
        &state.app_stores,
        &state.app_caches.connections_cache,
    )
    .await?;

    let id_str = headers
        .get(QUERY_BY_ID_PASSTHROUGH)
        .and_then(|h| h.to_str().ok())
        .map(|s| s.to_string());

    info!("Executing {} request on {}", method, uri.path());

    let destination = Destination {
        platform: connection.platform.clone(),
        action: Action::Passthrough {
            path: uri.path().into(),
            method: method.clone(),
            id: id_str.clone().map(Into::into),
        },
        connection_key: connection.key.clone(),
    };

    let Query(query_params) = query_params.unwrap_or_default();

    let mut forwarded_headers = headers.clone();
    forwarded_headers.remove(&state.config.headers.auth_header);
    forwarded_headers.remove(&state.config.headers.connection_header);

    let model_execution_result = state
        .extractor_caller
        .dispatch_destination_request(
            Some(connection.clone()),
            &destination,
            forwarded_headers.clone(),
            query_params,
            Some(body.to_vec()),
            state.app_caches.connection_model_definition.clone(),
        )
        .await
        .map_err(|e| {
            error!("Failed to execute connection model definition in passthrough endpoint. ID: {}, Error: {}", connection.id, e);
            e
        })?;

    let mut response_headers = HeaderMap::new();
    model_execution_result
        .headers()
        .into_iter()
        .for_each(|(key, value)| match key {
            &CONTENT_LENGTH => {
                response_headers.insert(CONTENT_LENGTH, value.clone());
            }
            _ => {
                if let Ok(header_name) =
                    HeaderName::try_from(format!("{PICA_PASSTHROUGH_HEADER}-{key}"))
                {
                    response_headers.insert(header_name, value.clone());
                }
            }
        });

    // Spawn event logging
    let connection_platform = connection.platform.to_string();
    let connection_platform_version = connection.platform_version.to_string();
    let connection_key = connection.key.to_string();
    let request_status_code = model_execution_result.status();
    let database_c = state.app_stores.db.clone();
    let event_access_pass_c = state.config.event_access_password.clone();
    let event_tx_c = state.event_tx.clone();
    let cache = state
        .app_caches
        .connection_model_definition_string_key
        .clone();
    let connection_secret_str = connection_secret_header.to_str().map(|a| a.to_owned()).ok();
    let uri_path = uri.path().to_string();
    let id_str_clone = id_str.clone();
    let request_headers = forwarded_headers.clone();

    tokio::spawn(async move {
        let options = FindOneOptions::builder()
            .projection(doc! {
                "connectionPlatform": 1,
                "connectionDefinitionId": 1,
                "platformVersion": 1,
                "key": 1,
                "title": 1,
                "name": 1,
                "path": 1,
                "action": 1,
                "actionName": 1
            })
            .build();

        let cache_key = format!(
            "{}",
            id_str_clone.unwrap_or_else(|| format!(
                "{}::{}",
                connection_platform,
                Alphanumeric.sample_string(&mut rand::thread_rng(), 16)
            ))
        );

        let query = if let Some(id) = id_str {
            doc! {
                "_id": id,
            }
        } else {
            doc! {
                "connectionPlatform": connection_platform.clone(),
                "path": uri_path.clone(),
                "action": method.to_string().to_uppercase()
            }
        };

        let cmd = cache
            .get_or_insert_with_fn(&cache_key, || async {
                Ok(database_c
                    .collection::<SparseCMD>(&Store::ConnectionModelDefinitions.to_string())
                    .find_one(query)
                    .with_options(options.clone())
                    .await
                    .ok()
                    .flatten())
            })
            .await
            .ok()
            .flatten();

        if let (Some(cmd), Some(encrypted_access_key_str)) = (cmd, connection_secret_str) {
            if let Ok(encrypted_access_key) = EncryptedAccessKey::parse(&encrypted_access_key_str) {
                let path = uri_path.trim_end_matches('/');

                let metadata = UnifiedMetadataBuilder::default()
                    .timestamp(Utc::now().timestamp_millis())
                    .platform_rate_limit_remaining(0)
                    .rate_limit_remaining(0)
                    .transaction_key(Id::now(IdPrefix::Transaction))
                    .platform(connection_platform.clone())
                    .platform_version(connection_platform_version.clone())
                    .common_model_version("v1")
                    .connection_key(connection_key.clone())
                    .action(cmd.title)
                    .host(host)
                    .path(path.to_string())
                    .status_code(request_status_code)
                    .build()
                    .ok()
                    .map(|m| m.as_value());

                let password: Option<[u8; PASSWORD_LENGTH]> =
                    event_access_pass_c.as_bytes().try_into().ok();

                if let Some(password) = password {
                    if let Ok(access_key) = AccessKey::parse(&encrypted_access_key, &password) {
                        let event_name = format!(
                            "{}::{}::{}::{}",
                            connection_platform,
                            connection_platform_version,
                            cmd.name,
                            cmd.action_name
                        );

                        let name = if request_status_code.is_success() {
                            format!("{event_name}::request-succeeded")
                        } else {
                            format!("{event_name}::request-failed")
                        };

                        let body = serde_json::to_string(&json!({
                            META: metadata,
                        }))
                        .unwrap_or_default();

                        let event = Event::new(
                            &access_key,
                            &encrypted_access_key,
                            &name,
                            request_headers,
                            body,
                        );

                        if let Err(e) = event_tx_c.send(event).await {
                            error!("Could not send event to receiver: {e}");
                        }
                    } else {
                        error!("Error generating event for passthrough");
                    }
                } else {
                    error!("Error generating event for passthrough");
                }
            }
        }
    });

    let metric = Metric::passthrough(connection);
    if let Err(e) = state.metric_tx.send(metric).await {
        error!("Could not send metric to receiver: {e}");
    }

    let bytes = model_execution_result.bytes().await.map_err(|e| {
        error!(
            "Error retrieving bytes from response in passthrough endpoint: {:?}",
            e
        );

        InternalError::script_error("Error retrieving bytes from response", None)
    })?;

    Ok((request_status_code, response_headers, bytes))
}

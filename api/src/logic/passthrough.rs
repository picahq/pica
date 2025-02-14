use super::get_connection;
use crate::{domain::metrics::Metric, server::AppState};
use axum::{
    extract::{Query, State},
    response::IntoResponse,
    routing::get,
    Extension, Router,
};
use bson::doc;
use chrono::Utc;
use entities::{
    constant::PICA_PASSTHROUGH_HEADER,
    destination::{Action, Destination},
    encrypted_access_key::EncryptedAccessKey,
    event_access::EventAccess,
    prefix::IdPrefix,
    AccessKey, ApplicationError, Event, Id, InternalError, META, PASSWORD_LENGTH,
    QUERY_BY_ID_PASSTHROUGH,
};
use http::{header::CONTENT_LENGTH, HeaderMap, HeaderName, Method, Uri};
use hyper::body::Bytes;
use serde_json::json;
use std::{collections::HashMap, sync::Arc};
use tracing::error;
use unified::domain::UnifiedMetadataBuilder;

pub fn get_router() -> Router<Arc<AppState>> {
    Router::new().route(
        "/*key",
        get(passthrough_request)
            .post(passthrough_request)
            .patch(passthrough_request)
            .delete(passthrough_request),
    )
}

pub async fn passthrough_request(
    Extension(user_event_access): Extension<Arc<EventAccess>>,
    State(state): State<Arc<AppState>>,
    mut headers: HeaderMap,
    query_params: Option<Query<HashMap<String, String>>>,
    uri: Uri,
    method: Method,
    body: Bytes,
) -> impl IntoResponse {
    let Some(connection_key_header) = headers.get(&state.config.headers.connection_header) else {
        return Err(ApplicationError::bad_request(
            "Connection header not found",
            None,
        ));
    };

    let Some(connection_secret_header) = headers.get(&state.config.headers.auth_header) else {
        return Err(ApplicationError::bad_request(
            "Connection header not found",
            None,
        ));
    };

    let connection_secret_header = connection_secret_header.clone();

    let connection = get_connection(
        user_event_access.as_ref(),
        connection_key_header,
        &state.app_stores,
        &state.connections_cache,
    )
    .await?;

    let id = headers
        .get(QUERY_BY_ID_PASSTHROUGH)
        .and_then(|h| h.to_str().ok());

    tracing::info!("Executing {} request on {}", method, uri.path());

    let destination = Destination {
        platform: connection.platform.clone(),
        action: Action::Passthrough {
            path: uri.path().into(),
            method: method.clone(),
            id: id.map(|i| i.into()),
        },
        connection_key: connection.key.clone(),
    };

    let Query(query_params) = query_params.unwrap_or_default();

    headers.remove(&state.config.headers.auth_header);
    headers.remove(&state.config.headers.connection_header);

    let model_execution_result = state
        .extractor_caller
        .dispatch_destination_request(
            Some(connection.clone()),
            &destination,
            headers.clone(),
            query_params,
            Some(body.to_vec()),
        )
        .await
        .map_err(|e| {
            error!("Failed to execute connection model definition in passthrough endpoint. ID: {}, Error: {}", connection.id, e);

            e
        })?;

    let mut headers = HeaderMap::new();

    model_execution_result
        .headers()
        .into_iter()
        .for_each(|(key, value)| match key {
            &CONTENT_LENGTH => {
                headers.insert(CONTENT_LENGTH, value.clone());
            }
            _ => {
                if let Ok(header_name) =
                    HeaderName::try_from(format!("{PICA_PASSTHROUGH_HEADER}-{key}"))
                {
                    headers.insert(header_name, value.clone());
                };
            }
        });

    let cloned_state = state.clone();

    let connection_platform = connection.platform.to_string();
    let connection_platform_version = connection.platform_version.to_string();
    let connection_key = connection.key.to_string();
    let request_headers = headers.clone();
    let request_status_code = model_execution_result.status();

    tokio::spawn(async move {
        let connection_secret_header: Option<String> =
            connection_secret_header.to_str().map(|a| a.to_owned()).ok();

        // TODO: Make a projection of this to avoid sending the whole document across the wire
        if let (Some(Some(cmd)), Some(encrypted_access_key)) = (
            cloned_state
                .app_stores
                .model_config
                .collection
                .find_one(doc! {
                    "connectionPlatform": connection_platform.clone(),
                    "path": uri.path().to_string(),
                    "action": method.to_string().to_uppercase()
                })
                .await
                .ok(),
            connection_secret_header,
        ) {
            // TODO: Pass the host as well
            if let Ok(encrypted_access_key) = EncryptedAccessKey::parse(&encrypted_access_key) {
                let metadata = UnifiedMetadataBuilder::default()
                    .timestamp(Utc::now().timestamp_millis())
                    .platform_rate_limit_remaining(0)
                    .rate_limit_remaining(0)
                    .transaction_key(Id::now(IdPrefix::Transaction))
                    .platform(connection_platform.clone())
                    .platform_version(connection_platform_version.clone())
                    .common_model_version("v1")
                    .connection_key(connection_key)
                    .action(cmd.title)
                    .status_code(request_status_code.to_string())
                    .build()
                    .ok()
                    .map(|m| m.as_value());

                let password: Option<[u8; PASSWORD_LENGTH]> = cloned_state
                    .config
                    .event_access_password
                    .as_bytes()
                    .try_into()
                    .ok();

                match password {
                    Some(password) => {
                        let access_key = AccessKey::parse(&encrypted_access_key, &password).ok();

                        let event_name = format!(
                            "{}::{}::{}::{}",
                            connection_platform,
                            connection_platform_version,
                            cmd.name,
                            cmd.action_name
                        );

                        let name = if request_status_code.is_success() {
                            format!("{event_name}::request-succeeded",)
                        } else {
                            format!("{event_name}::request-failed",)
                        };

                        let body = serde_json::to_string(&json!({
                            META: metadata,
                        }))
                        .unwrap_or_default();

                        if let Some(access_key) = access_key {
                            let event = Event::new(
                                &access_key,
                                &encrypted_access_key,
                                &name,
                                request_headers.clone(),
                                body,
                            );

                            if let Err(e) = cloned_state.event_tx.send(event).await {
                                error!("Could not send event to receiver: {e}");
                            }
                        } else {
                            tracing::error!("Error generating event for passthrough")
                        }
                    }
                    None => tracing::error!("Error generating event for passthrough"),
                };
            }
        };
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

    Ok((request_status_code, headers, bytes))
}

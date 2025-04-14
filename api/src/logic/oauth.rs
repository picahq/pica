use super::event_access::CreateEventAccessPayloadWithOwnership;
use crate::{logic::event_access::get_client_throughput, server::AppState};
use axum::{
    extract::{Path, State},
    routing::post,
    Extension, Json, Router,
};
use chrono::{Duration, Utc};
use fake::Dummy;
use mongodb::bson::doc;
use osentities::{
    algebra::{MongoStore, TemplateExt},
    connection_definition::ConnectionDefinition,
    connection_oauth_definition::{
        ConnectionOAuthDefinition, OAuthResponse, PlatformSecret, Settings,
    },
    event_access::EventAccess,
    id::{prefix::IdPrefix, Id},
    oauth_secret::OAuthSecret,
    ownership::Ownership,
    ApplicationError, Connection, ConnectionIdentityType, ErrorMeta, InternalError, OAuth,
    PicaError, SanitizedConnection, Throughput, DEFAULT_NAMESPACE,
};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;
use tracing::{debug, error};
use uuid::Uuid;

pub fn get_router() -> Router<Arc<AppState>> {
    Router::new().route("/:platform", post(oauth_handler))
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, Dummy)]
#[serde(rename_all = "camelCase")]
struct OAuthRequest {
    #[serde(rename = "__isEngineeringAccount__", default)]
    is_engineering_account: bool,
    connection_definition_id: Id,
    client_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    payload: Option<Value>,
    name: Option<String>,
    group: Option<String>,
    identity: Option<String>,
    identity_type: Option<ConnectionIdentityType>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, Dummy)]
#[serde(rename_all = "camelCase")]
struct OAuthPayload {
    client_id: String,
    client_secret: String,
    metadata: Value,
}

impl OAuthPayload {
    fn as_json(&self) -> Option<Value> {
        serde_json::to_value(self).ok()
    }
}

async fn oauth_handler(
    state: State<Arc<AppState>>,
    Extension(user_event_access): Extension<Arc<EventAccess>>,
    Path(platform): Path<String>,
    Json(payload): Json<OAuthRequest>,
) -> Result<Json<SanitizedConnection>, PicaError> {
    let conn_oauth_definition = get_conn_oauth_definition(&state, &platform).await?;
    let setting = get_user_settings(
        &state,
        &user_event_access.ownership,
        payload.is_engineering_account,
    )
    .await
    .map_err(|e| {
        error!("Failed to get user settings: {:?}", e);
        e
    })?;

    let environment = user_event_access.environment;

    let secret: PlatformSecret = get_secret::<PlatformSecret>(
        &state,
        setting
            .platform_secret(&payload.connection_definition_id, environment)
            .ok_or_else(|| {
                error!("Settings does not have a secret service id for the connection platform");
                InternalError::invalid_argument(
                    "Provided connection definition does not have a secret entry",
                    None,
                )
            })?,
        if payload.is_engineering_account {
            tracing::info!("Using engineering account id for secret");
            state.config.engineering_account_id.clone()
        } else {
            tracing::info!("Using user event access id for secret");
            user_event_access.clone().ownership.id.to_string()
        },
    )
    .await
    .map_err(|e| {
        error!("Failed to get platform secret for connection: {:?}", e);
        e
    })?;

    let mut oauth_payload = OAuthPayload {
        metadata: payload.clone().payload.clone().unwrap_or(Value::Null),
        client_id: payload.clone().client_id,
        client_secret: secret.clone().client_secret,
    };

    if let Some(metadata) = oauth_payload.metadata.as_object_mut() {
        metadata.insert(
            "environment".to_string(),
            Value::String(environment.to_string()),
        );
    }

    let conn_oauth_definition = if conn_oauth_definition.is_full_template_enabled {
        state
            .template
            .render_as(&conn_oauth_definition, oauth_payload.as_json().as_ref())
            .map_err(|e| {
                error!("Failed to render oauth definition: {:?}", e);
                e
            })?
    } else {
        conn_oauth_definition
    };

    let req_payload = serde_json::json!({
        "connectionOAuthDefinition": conn_oauth_definition,
        "payload": oauth_payload.clone(),
        "secret": {
            "clientId": secret.clone().client_id,
            "clientSecret": secret.clone().client_secret
        }
    });

    let oauth_url = state.config.oauth_url.clone();
    let oauth_url = format!("{}/oauth/dynamic-dispatch/init", oauth_url);
    let response = state
        .http_client
        .post(oauth_url)
        .header("Content-Type", "application/json")
        .json(&req_payload)
        .send()
        .await
        .map_err(|e| {
            error!("Failed to execute oauth request: {}", e);
            InternalError::script_error(&e.to_string(), None)
        })?
        .json::<Value>()
        .await
        .map_err(|e| {
            error!("Failed to decode third party oauth response: {:?}", e);
            InternalError::deserialize_error(&e.to_string(), None)
        })?;

    let decoded: OAuthResponse = serde_json::from_value(response.clone()).map_err(|e| {
        error!("Failed to decode oauth response: {:?}", e);
        InternalError::script_error(&e.to_string(), None)
    })?;

    debug!("Response: {:?}", response);

    let oauth_secret = OAuthSecret::from_init(
        decoded,
        oauth_payload.client_id,
        oauth_payload.client_secret,
        response,
        payload.payload,
    );

    let secret = state
        .secrets_client
        .create(
            &oauth_secret.as_json(),
            user_event_access.clone().ownership.id.as_ref(),
        )
        .await
        .map_err(|e| {
            error!("Failed to create oauth secret: {}", e);
            InternalError::encryption_error(e.message().as_ref(), None)
        })?;

    let conn_definition = get_conn_definition(&state, &payload.connection_definition_id).await?;

    let uuid = Uuid::new_v4().to_string().replace('-', "");
    let group = payload.group.unwrap_or_else(|| uuid.clone());
    let identity = payload.identity.unwrap_or_else(|| group.clone());

    let key_suffix = if identity == uuid {
        uuid.clone()
    } else {
        format!("{}|{}", uuid, identity.replace(&[' ', ':'][..], "-"))
    };

    let key = format!(
        "{}::{}::{}::{}",
        user_event_access.environment, conn_definition.platform, DEFAULT_NAMESPACE, key_suffix
    );

    let throughput = get_client_throughput(&user_event_access.ownership.id, &state).await?;

    let event_access = CreateEventAccessPayloadWithOwnership {
        name: format!("{} {}", user_event_access.environment, conn_definition.name),
        platform: conn_definition.platform.clone(),
        namespace: None,
        connection_type: conn_definition.r#type.clone(),
        environment: user_event_access.environment,
        paths: conn_definition.paths.clone(),
        ownership: user_event_access.ownership.clone(),
        throughput: Some(throughput),
    }
    .as_event_access(&state.config)
    .map_err(|e| {
        error!("Error creating event access for oauth connection: {:?}", e);
        ApplicationError::service_unavailable("Failed to create event access", None)
    })?;

    state
        .app_stores
        .event_access
        .create_one(&event_access)
        .await
        .map_err(|e| {
            error!("Error saving event access for oauth connection: {:?}", e);
            e
        })?;

    let connection = Connection {
        id: Id::new(IdPrefix::Connection, Utc::now()),
        platform_version: conn_definition.clone().platform_version,
        connection_definition_id: conn_definition.id,
        r#type: conn_definition.to_connection_type(),
        group,
        name: payload.name,
        key: key.clone().into(),
        environment: user_event_access.environment,
        platform: platform.into(),
        secrets_service_id: secret.id(),
        event_access_id: event_access.id,
        access_key: event_access.access_key,
        identity: Some(identity),
        identity_type: payload.identity_type,
        settings: conn_definition.settings,
        has_error: false,
        error: None,
        throughput: Throughput {
            key,
            limit: throughput,
        },
        ownership: user_event_access.ownership.clone(),
        oauth: Some(OAuth::Enabled {
            connection_oauth_definition_id: conn_oauth_definition.id,
            expires_in: Some(oauth_secret.expires_in),
            expires_at: Some(
                chrono::Utc::now()
                    .checked_add_signed(Duration::seconds(oauth_secret.expires_in as i64))
                    .unwrap_or_else(chrono::Utc::now)
                    .checked_sub_signed(Duration::seconds(120))
                    .unwrap_or_else(chrono::Utc::now)
                    .timestamp(),
            ),
        }),
        record_metadata: Default::default(),
    };

    state
        .app_stores
        .connection
        .create_one(&connection)
        .await
        .map_err(|e| {
            error!("Failed to create connection: {}", e);
            ApplicationError::service_unavailable("Failed to create connection", None)
        })?;

    Ok(Json(connection.into()))
}

async fn get_conn_definition(
    state: &State<Arc<AppState>>,
    conn_definition_id: &Id,
) -> Result<ConnectionDefinition, PicaError> {
    let conn_definition_store: &MongoStore<ConnectionDefinition> =
        &state.app_stores.connection_config;

    let conn_definition: ConnectionDefinition = conn_definition_store
        .get_one(doc! {"_id": &conn_definition_id.to_string()})
        .await?
        .ok_or_else(|| ApplicationError::not_found("Connection definition", None))?;

    Ok(conn_definition)
}

async fn get_conn_oauth_definition(
    state: &State<Arc<AppState>>,
    platform: &str,
) -> Result<ConnectionOAuthDefinition, PicaError> {
    let oauth_definition_store: &MongoStore<ConnectionOAuthDefinition> =
        &state.app_stores.oauth_config;

    let conn_oauth_definition: ConnectionOAuthDefinition = oauth_definition_store
        .get_one(doc! {"connectionPlatform": &platform})
        .await?
        .ok_or_else(|| ApplicationError::not_found("Connection OAuth definition", None))?;

    Ok(conn_oauth_definition)
}

pub async fn get_user_settings(
    state: &State<Arc<AppState>>,
    ownership: &Ownership,
    is_engineering_account: bool,
) -> Result<Settings, PicaError> {
    let settings_store: &MongoStore<Settings> = &state.app_stores.settings;

    let ownership_id = if is_engineering_account {
        state.config.engineering_account_id.clone()
    } else {
        ownership.id.to_string()
    };

    let setting: Settings = settings_store
        .get_one(doc! {"ownership.buildableId": &ownership_id})
        .await?
        .ok_or_else(|| ApplicationError::not_found("Settings", None))?;

    Ok(setting)
}

async fn get_secret<S: DeserializeOwned>(
    state: &State<Arc<AppState>>,
    id: String,
    buildable_id: String,
) -> Result<S, PicaError> {
    let secrets_client = &state.secrets_client;

    let encoded_secret = secrets_client.get(&id, &buildable_id).await?;

    encoded_secret.decode::<S>()
}

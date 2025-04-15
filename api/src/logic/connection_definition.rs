use super::{create, delete, read, update, HookExt, PublicExt, ReadResponse, RequestExt};
use crate::{
    helper::shape_mongo_filter,
    router::ServerResponse,
    server::{AppState, AppStores},
};
use axum::http::HeaderMap;
use axum::{extract::Query, Extension};
use axum::{
    extract::{Path, State},
    routing::{patch, post},
    Json, Router,
};
use fake::Dummy;
use mongodb::bson::doc;
use osentities::{
    algebra::MongoStore,
    api_model_config::AuthMethod,
    connection_definition::{
        AuthSecret, ConnectionDefinition, ConnectionDefinitionType, ConnectionForm,
        ConnectionStatus, Filter, FormDataItem, Frontend, Paths, PublicConnectionDetails, Spec,
    },
    connection_model_definition::{ConnectionModelDefinition, CrudAction},
    event_access::EventAccess,
    id::{prefix::IdPrefix, Id},
    record_metadata::RecordMetadata,
    settings::Settings,
    ApplicationError, PicaError,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::sync::Arc;
use tokio::try_join;
use tracing::error;

pub fn get_router() -> Router<Arc<AppState>> {
    Router::new()
        .route(
            "/",
            post(create::<CreateRequest, ConnectionDefinition>)
                .get(read::<CreateRequest, ConnectionDefinition>),
        )
        .route(
            "/:id",
            patch(update::<CreateRequest, ConnectionDefinition>)
                .delete(delete::<CreateRequest, ConnectionDefinition>),
        )
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, Dummy)]
#[serde(rename_all = "camelCase")]
pub struct CreateRequest {
    #[serde(rename = "_id")]
    pub id: Option<Id>,
    pub platform: String,
    pub platform_version: String,
    #[serde(default)]
    pub status: ConnectionStatus,
    pub r#type: ConnectionDefinitionType,
    pub name: String,
    pub description: String,
    pub category: String,
    pub image: String,
    pub tags: Vec<String>,
    pub helper_link: Option<String>,
    pub authentication: Vec<AuthenticationItem>,
    pub auth_method: Option<AuthMethod>,
    #[serde(default)]
    pub multi_env: bool,
    pub settings: Settings,
    pub paths: Paths,
    pub test_connection: Option<Id>,
    pub test_delay_in_millis: Option<i16>,
    pub active: bool,
    #[serde(default)]
    pub markdown: Option<String>,
}

impl HookExt<ConnectionDefinition> for CreateRequest {}
impl PublicExt<ConnectionDefinition> for CreateRequest {}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, Dummy)]
pub struct AuthenticationItem {
    pub name: String,
    pub label: String,
    pub r#type: String,
    pub placeholder: String,
    pub options: Option<Vec<String>>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UpdateFields {
    pub active: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PublicGetConnectionDetailsResponse {
    pub platform: String,
    pub status: ConnectionStatus,
    pub supported_actions: Vec<CrudAction>,
    pub oauth: PublicConnectionDataOauth,
    pub pagination: bool,
    pub filtration: bool,
    pub sorting: bool,
    pub caveats: Vec<PublicConnectionDataCaveat>,
    pub supported_filters: Option<Vec<Filter>>,
    pub supported_sort_keys: Option<Vec<String>>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PublicConnectionDataCaveat {
    pub name: String,
    pub action: Option<CrudAction>,
    pub comments: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PublicConnectionDataOauth {
    pub enabled: bool,
    pub scopes: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GetPublicConnectionDetailsRequest;

impl HookExt<PublicConnectionDetails> for GetPublicConnectionDetailsRequest {}
impl PublicExt<PublicConnectionDetails> for GetPublicConnectionDetailsRequest {}
impl RequestExt for GetPublicConnectionDetailsRequest {
    type Output = PublicConnectionDetails;

    fn get_store(stores: AppStores) -> MongoStore<Self::Output> {
        stores.public_connection_details
    }
}

pub async fn public_get_connection_details(
    Path((common_model, platform_name)): Path<(String, String)>,
    State(state): State<Arc<AppState>>,
) -> Result<Json<ServerResponse<PublicGetConnectionDetailsResponse>>, PicaError> {
    let Some(connection_definition) = state
        .app_stores
        .connection_config
        .get_one(doc! {
            "platform": &platform_name,
        })
        .await
        .map_err(|e| {
            error!("Error reading from connection definitions: {e}");

            e
        })?
    else {
        return Err(ApplicationError::not_found(
            &format!("Connection definition for platform {}", &platform_name),
            None,
        ));
    };

    let connection_model_definitions = state
        .app_stores
        .model_config
        .get_many(
            Some(doc! {
                "connectionPlatform": {
                    "$regex": format!("^{}$", &platform_name),
                    "$options": "i"
                },
                "mapping.commonModelName": {
                    "$regex": format!("^{}$", &common_model),
                    "$options": "i"
                },
                "actionName": {
                    "$in": [
                        "create",
                        "update",
                        "getMany",
                        "getOne",
                        "getCount",
                        "delete"
                    ]
                }
            }),
            None,
            None,
            None,
            None,
        )
        .await
        .map_err(|e| {
            error!("Error reading from connection model definitions: {e}");
            e
        })?;

    let supported_actions = connection_model_definitions
        .clone()
        .into_iter()
        .map(|definition| definition.action_name)
        .collect::<Vec<CrudAction>>();

    let oauth_enabled = matches!(connection_definition.auth_method, Some(AuthMethod::OAuth));

    let scopes = if oauth_enabled {
        let connection_oauth_definition = state
            .app_stores
            .oauth_config
            .get_one(doc! {
                "connectionPlatform": &platform_name,
            })
            .await
            .map_err(|e| {
                error!("Error reading from connection definitions: {e}");
                e
            })?
            .ok_or_else(|| {
                ApplicationError::not_found(
                    &format!("OAuth Config for platform {}", &platform_name),
                    None,
                )
            })?;

        connection_oauth_definition.frontend.scopes
    } else {
        String::new()
    };

    let public_connection_details_record = state
        .app_stores
        .public_connection_details
        .get_one(doc! {
            "platform": &platform_name,
        })
        .await
        .map_err(|e| {
            error!("Error reading from public connection details: {e}");
            e
        })?
        .ok_or_else(|| {
            ApplicationError::not_found(
                &format!("Public connection details for platform {}", &platform_name),
                None,
            )
        })?;

    let model_features = public_connection_details_record
        .models
        .iter()
        .find(|model| model.name.to_lowercase() == common_model.to_lowercase())
        .ok_or_else(|| {
            ApplicationError::not_found(
                &format!("Model features for model {}", &common_model),
                None,
            )
        })?;

    let caveats =
        public_connection_details_record
            .caveats
            .into_iter()
            .fold(vec![], |mut v, caveat| {
                match caveat.connection_model_definition_id {
                    Some(cmd_id) => {
                        let connection_model_definition = connection_model_definitions.iter().find(
                            |definition: &&ConnectionModelDefinition| {
                                definition.id.to_string() == cmd_id
                            },
                        );

                        if let Some(definition) = connection_model_definition {
                            v.push(PublicConnectionDataCaveat {
                                name: definition.title.clone(),
                                action: Some(definition.action_name.clone()),
                                comments: caveat.comments,
                            });
                        }
                    }
                    None => {
                        v.push(PublicConnectionDataCaveat {
                            name: "General".to_string(),
                            action: None,
                            comments: caveat.comments,
                        });
                    }
                }
                v
            });

    Ok(Json(ServerResponse::new(
        "connection_definition",
        PublicGetConnectionDetailsResponse {
            platform: connection_definition.platform,
            status: connection_definition.status,
            oauth: PublicConnectionDataOauth {
                enabled: oauth_enabled,
                scopes,
            },
            supported_actions,
            pagination: model_features.pagination,
            filtration: model_features.filtration.supported,
            sorting: model_features.sorting.supported,
            caveats,
            supported_filters: model_features.filtration.filters.clone(),
            supported_sort_keys: model_features.sorting.keys.clone(),
        },
    )))
}

impl RequestExt for CreateRequest {
    type Output = ConnectionDefinition;

    fn from(&self) -> Option<Self::Output> {
        let auth_secrets: Vec<AuthSecret> = self
            .authentication
            .iter()
            .map(|item| AuthSecret {
                name: item.name.to_string(),
            })
            .collect();

        let connection_form_items: Vec<FormDataItem> = self
            .authentication
            .iter()
            .map(|item| FormDataItem {
                name: item.name.clone(),
                r#type: item.r#type.clone(),
                label: item.label.clone(),
                placeholder: item.placeholder.clone(),
                options: if item.options.is_some() {
                    item.options.clone()
                } else {
                    None
                },
            })
            .collect();

        let connection_form = ConnectionForm {
            name: "Connect".to_string(),
            description: "Securely connect your account".to_string(),
            form_data: connection_form_items,
        };

        let key = format!("api::{}::{}", self.platform, self.platform_version);

        let mut record = Self::Output {
            id: self
                .id
                .unwrap_or_else(|| Id::now(IdPrefix::ConnectionDefinition)),
            platform_version: self.platform_version.clone(),
            platform: self.platform.clone(),
            status: self.status.clone(),
            r#type: self.r#type.clone(),
            name: self.name.clone(),
            key,
            frontend: Frontend {
                spec: Spec {
                    title: self.name.clone(),
                    description: self.description.clone(),
                    platform: self.platform.clone(),
                    category: self.category.clone(),
                    image: self.image.clone(),
                    tags: self.tags.clone(),
                    helper_link: self.helper_link.clone(),
                    markdown: self.markdown.clone(),
                },
                connection_form,
            },
            test_connection: self.test_connection,
            auth_secrets,
            auth_method: self.auth_method.clone(),
            multi_env: self.multi_env,
            paths: self.paths.clone(),
            settings: self.settings.clone(),
            hidden: false,
            test_delay_in_millis: self.test_delay_in_millis,
            record_metadata: RecordMetadata::default(),
        };

        record.record_metadata.active = self.active;
        Some(record)
    }

    fn update(&self, mut record: Self::Output) -> Self::Output {
        record.name.clone_from(&self.name);
        record
            .frontend
            .spec
            .description
            .clone_from(&self.description);
        record.frontend.spec.category.clone_from(&self.category);
        record.frontend.spec.image.clone_from(&self.image);
        record.frontend.spec.tags.clone_from(&self.tags);
        record.test_connection = self.test_connection;
        record.platform.clone_from(&self.platform);
        record.multi_env = self.multi_env;
        record.record_metadata.active = self.active;
        record
    }

    fn get_store(stores: AppStores) -> MongoStore<Self::Output> {
        stores.connection_config
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AvailableConnectorsResponse {
    pub name: String,
    pub key: String,
    pub platform: String,
    pub platform_version: String,
    pub description: String,
    pub category: String,
    pub image: String,
    pub tags: Vec<String>,
    pub oauth: bool,
    #[serde(flatten)]
    pub record_metadata: RecordMetadata,
}

pub async fn get_available_connectors(
    Extension(user_event_access): Extension<Arc<EventAccess>>,
    headers: HeaderMap,
    query: Option<Query<BTreeMap<String, String>>>,
    State(state): State<Arc<AppState>>,
) -> Result<Json<ServerResponse<ReadResponse<AvailableConnectorsResponse>>>, PicaError> {
    const AUTHKIT_ENABLED_QUERY_PARAM: &str = "authkit";

    let mut query_map = query.clone().map(|q| q.0).unwrap_or_default();
    let authkit_enabled = query_map
        .remove(AUTHKIT_ENABLED_QUERY_PARAM)
        .map(|val| val == "true")
        .unwrap_or(false);

    let cleaned_query = if !query_map.is_empty() {
        Some(Query(query_map))
    } else {
        None
    };

    let query = shape_mongo_filter(cleaned_query, None, Some(headers));
    let store = state.app_stores.connection_config.clone();
    let mut filter = query.filter.clone();

    if authkit_enabled {
        let settings_store = state.app_stores.settings.clone();
        let settings = settings_store
            .get_one(doc! {
                "ownership.buildableId": user_event_access.ownership.id.to_string(),
            })
            .await
            .unwrap();

        if let Some(settings) = settings {
            // Get platforms that are active and match the user's environment
            let filtered_platforms: Vec<String> = settings
                .connected_platforms
                .iter()
                .filter(|platform| {
                    platform.active.unwrap_or(false)
                        && platform.environment == user_event_access.environment
                })
                .map(|platform| platform.r#type.clone())
                .collect();

            if !filtered_platforms.is_empty() {
                filter.insert("platform", doc! { "$in": filtered_platforms });
            }
        }
    }

    let count = store.count(filter.clone(), None);
    let find = store.get_many(
        Some(filter),
        None,
        None,
        Some(query.limit),
        Some(query.skip),
    );

    let res = match try_join!(count, find) {
        Ok((total, rows)) => {
            let available_connectors = rows
                .into_iter()
                .map(|conn_def| AvailableConnectorsResponse {
                    name: conn_def.name,
                    key: conn_def.key,
                    platform: conn_def.frontend.spec.platform,
                    platform_version: conn_def.platform_version,
                    description: conn_def.frontend.spec.description,
                    category: conn_def.frontend.spec.category,
                    image: conn_def.frontend.spec.image,
                    tags: conn_def.frontend.spec.tags,
                    oauth: matches!(conn_def.auth_method, Some(AuthMethod::OAuth)),
                    record_metadata: conn_def.record_metadata,
                })
                .collect();

            ReadResponse {
                rows: available_connectors,
                skip: query.skip,
                limit: query.limit,
                total,
            }
        }
        Err(e) => {
            error!("Error reading from store: {e}");
            return Err(e);
        }
    };

    Ok(Json(ServerResponse::new("Available Connectors", res)))
}

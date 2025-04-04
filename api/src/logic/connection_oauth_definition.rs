use super::{create, delete, read, update, HookExt, PublicExt, RequestExt};
use crate::server::{AppState, AppStores};
use axum::{
    routing::{patch, post},
    Router,
};
use chrono::Utc;
use mongodb::bson::doc;
use osentities::{
    algebra::MongoStore,
    api_model_config::{ApiModelConfig, Compute, Function, Lang},
    connection_oauth_definition::{
        ComputeRequest, ConnectionOAuthDefinition, Frontend, OAuthApiConfig, OAuthCompute,
    },
    id::{prefix::IdPrefix, Id},
    record_metadata::RecordMetadata,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

pub fn get_router() -> Router<Arc<AppState>> {
    Router::new()
        .route(
            "/",
            post(create::<CreateRequest, ConnectionOAuthDefinition>)
                .get(read::<CreateRequest, ConnectionOAuthDefinition>),
        )
        .route(
            "/:id",
            patch(update::<CreateRequest, ConnectionOAuthDefinition>)
                .delete(delete::<CreateRequest, ConnectionOAuthDefinition>),
        )
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateRequest {
    #[serde(rename = "_id")]
    pub id: Option<Id>,
    pub connection_platform: String,
    pub platform_redirect_uri: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sandbox_platform_redirect_uri: Option<String>,
    pub ios_redirect_uri: String,
    pub scopes: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub separator: Option<String>,
    pub init: RequestParams,
    pub refresh: RequestParams,
    pub is_full_template_enabled: bool,
}

impl HookExt<ConnectionOAuthDefinition> for CreateRequest {}
impl PublicExt<ConnectionOAuthDefinition> for CreateRequest {}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RequestParams {
    pub configuration: ApiModelConfig,
    #[serde(skip_serializing_if = "Option::is_none", default = "default_separator")]
    pub compute: Option<String>,
    pub response_compute: String,
}

fn default_separator() -> Option<String> {
    Some(" ".to_string())
}

impl RequestExt for CreateRequest {
    type Output = ConnectionOAuthDefinition;

    fn from(&self) -> Option<Self::Output> {
        Some(Self::Output {
            id: self
                .id
                .unwrap_or_else(|| Id::now(IdPrefix::ConnectionOAuthDefinition)),
            connection_platform: self.connection_platform.clone(),
            configuration: OAuthApiConfig {
                init: self.init.configuration.clone(),
                refresh: self.refresh.configuration.clone(),
            },
            is_full_template_enabled: self.is_full_template_enabled,
            compute: OAuthCompute {
                init: ComputeRequest {
                    response: Function(Compute {
                        entry: "compute".to_string(),
                        function: self.init.response_compute.clone(),
                        language: Lang::JavaScript,
                    }),
                    computation: self
                        .init
                        .compute
                        .iter()
                        .map(|compute| {
                            Function(Compute {
                                entry: "compute".to_string(),
                                function: compute.clone(),
                                language: Lang::JavaScript,
                            })
                        })
                        .next(),
                },
                refresh: ComputeRequest {
                    computation: self
                        .refresh
                        .compute
                        .iter()
                        .map(|compute| {
                            Function(Compute {
                                entry: "compute".to_string(),
                                function: compute.clone(),
                                language: Lang::JavaScript,
                            })
                        })
                        .next(),
                    response: Function(Compute {
                        entry: "compute".to_string(),
                        function: self.refresh.response_compute.clone(),
                        language: Lang::JavaScript,
                    }),
                },
            },
            frontend: Frontend {
                platform_redirect_uri: self.platform_redirect_uri.clone(),
                sandbox_platform_redirect_uri: self.sandbox_platform_redirect_uri.clone(),
                ios_redirect_uri: self.ios_redirect_uri.clone(),
                scopes: self.scopes.clone(),
                separator: self.separator.clone(),
            },
            record_metadata: Default::default(),
        })
    }

    fn update(&self, mut record: Self::Output) -> Self::Output {
        record
            .connection_platform
            .clone_from(&self.connection_platform);
        record.configuration = OAuthApiConfig {
            init: self.init.configuration.clone(),
            refresh: self.refresh.configuration.clone(),
        };
        record.is_full_template_enabled = self.is_full_template_enabled;
        record.compute = OAuthCompute {
            init: ComputeRequest {
                computation: self
                    .init
                    .compute
                    .iter()
                    .map(|compute| {
                        Function(Compute {
                            entry: "compute".to_string(),
                            function: compute.clone(),
                            language: Lang::JavaScript,
                        })
                    })
                    .next(),
                response: Function(Compute {
                    entry: "compute".to_string(),
                    function: self.init.response_compute.clone(),
                    language: Lang::JavaScript,
                }),
            },
            refresh: ComputeRequest {
                response: Function(Compute {
                    entry: "compute".to_string(),
                    function: self.refresh.response_compute.clone(),
                    language: Lang::JavaScript,
                }),
                computation: self
                    .refresh
                    .compute
                    .iter()
                    .map(|compute| {
                        Function(Compute {
                            entry: "compute".to_string(),
                            function: compute.clone(),
                            language: Lang::JavaScript,
                        })
                    })
                    .next(),
            },
        };
        record.frontend = Frontend {
            platform_redirect_uri: self.platform_redirect_uri.clone(),
            sandbox_platform_redirect_uri: self.sandbox_platform_redirect_uri.clone(),
            ios_redirect_uri: self.ios_redirect_uri.clone(),
            scopes: self.scopes.clone(),
            separator: self.separator.clone(),
        };
        record.record_metadata.updated_at = Utc::now().timestamp_millis();
        record.record_metadata.updated = true;

        record
    }

    fn get_store(stores: AppStores) -> MongoStore<Self::Output> {
        stores.oauth_config.clone()
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FrontendOauthConnectionDefinition {
    #[serde(rename = "_id")]
    pub id: String,
    pub connection_platform: String,
    pub frontend: Frontend,
    #[serde(flatten)]
    pub record_metadata: RecordMetadata,
}

impl PublicExt<FrontendOauthConnectionDefinition> for FrontendOauthConnectionDefinition {}

impl RequestExt for FrontendOauthConnectionDefinition {
    type Output = FrontendOauthConnectionDefinition;

    fn get_store(stores: AppStores) -> MongoStore<Self::Output> {
        stores.frontend_oauth_config.clone()
    }
}

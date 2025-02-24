use crate::{
    connection_oauth_definition::ConnectedPlatform, environment::Environment, ownership::Ownership,
    record_metadata::RecordMetadata, Connection, Id,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::connection_oauth_definition::ConnectedPlatformSlim;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EmbedToken {
    #[serde(rename = "_id")]
    pub id: Id,
    pub link_settings: EmbedLinkedToken,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub features: Vec<Feature>,
    pub label: String,
    pub group: String,
    pub environment: Environment,
    pub form_data: Option<Value>,
    pub response: Option<EventTokenResponse>,
    pub session_id: String,
    pub expires_at: Option<i64>,
    #[serde(flatten, default)]
    pub metadata: RecordMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EmbedTokenSlim {
    #[serde(rename = "_id")]
    pub id: Id,
    pub link_settings: EmbedLinkedTokenSlim,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EmbedLinkedTokenSlim {
    pub connected_platforms: Vec<ConnectedPlatformSlim>,
    pub event_inc_token: String
}


#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EmbedLinkedToken {
    pub connected_platforms: Vec<ConnectedPlatform>,
    pub event_inc_token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EventTokenResponse {
    pub is_connected: bool,
    pub message: Option<String>,
    pub connection: Option<Connection>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Feature {
    pub key: String,
    pub value: FeatureState,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FeatureState {
    Enabled,
    Disabled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EventLink {
    #[serde(rename = "_id")]
    pub id: Id,
    #[serde(rename = "_type")]
    pub r#type: String,
    pub version: Option<String>,
    pub ownership: Ownership,
    pub label: String,
    pub group: String,
    #[serde(rename = "token")]
    pub link_token_id: Id,
    #[serde(flatten, default)]
    pub metadata: RecordMetadata,
    pub environment: Environment,
    pub usage_source: String,
    pub expires_at: i64,
}

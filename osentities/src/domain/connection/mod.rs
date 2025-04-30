pub mod api_model_config;
pub mod connection_definition;
pub mod connection_model_definition;
pub mod connection_model_schema;
pub mod connection_oauth_definition;

use super::{
    configuration::environment::Environment,
    shared::{ownership::Ownership, record_metadata::RecordMetadata, settings::Settings},
};
use crate::id::Id;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{hash::Hash, sync::Arc};
use strum::{AsRefStr, Display, EnumString};
use tabled::Tabled;

fn key_default() -> Arc<str> {
    String::new().into()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Connection {
    #[serde(rename = "_id")]
    pub id: Id,
    pub platform_version: String,
    pub connection_definition_id: Id,
    pub r#type: ConnectionType,
    #[serde(default = "key_default")]
    pub key: Arc<str>,
    pub group: String,
    pub name: Option<String>,
    pub environment: Environment,
    pub platform: Arc<str>,
    pub secrets_service_id: String,
    pub event_access_id: Option<Id>,
    pub access_key: Option<String>,
    pub identity: Option<String>,
    pub identity_type: Option<ConnectionIdentityType>,
    pub settings: Settings,
    pub throughput: Throughput,
    pub ownership: Ownership,
    #[serde(default)]
    pub oauth: Option<OAuth>,
    #[serde(default)]
    pub has_error: bool,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub error: Option<String>,
    #[serde(flatten, default)]
    pub record_metadata: RecordMetadata,
}

impl Connection {
    pub fn mark_error(&mut self, error: &str) {
        self.has_error = true;
        self.error = Some(error.to_string());
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "dummy", derive(fake::Dummy))]
pub enum ConnectionIdentityType {
    Organization,
    User,
    Team,
    Project,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, Tabled)]
#[serde(rename_all = "camelCase")]
#[tabled(rename_all = "PascalCase")]
pub struct PublicConnection {
    #[serde(rename = "_id")]
    pub id: Id,
    pub platform_version: String,
    #[tabled(rename = "type")]
    pub r#type: ConnectionType,
    pub key: Arc<str>,
    pub environment: Environment,
    pub platform: Arc<str>,
    #[tabled(skip)]
    pub identity: Option<String>,
    #[tabled(skip)]
    pub identity_type: Option<ConnectionIdentityType>,
    #[serde(flatten, default)]
    #[tabled(skip)]
    pub record_metadata: RecordMetadata,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SanitizedConnection {
    #[serde(rename = "_id")]
    pub id: Id,
    pub platform_version: String,
    pub connection_definition_id: Id,
    pub r#type: ConnectionType,
    pub key: Arc<str>,
    pub group: String,
    pub name: Option<String>,
    pub environment: Environment,
    pub platform: Arc<str>,
    pub secrets_service_id: String,
    pub event_access_id: Option<Id>,
    pub identity: Option<String>,
    pub identity_type: Option<ConnectionIdentityType>,
    pub settings: Settings,
    pub throughput: Throughput,
    pub ownership: Ownership,
    #[serde(default)]
    pub oauth: Option<OAuth>,
    #[serde(default)]
    pub has_error: bool,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub error: Option<String>,
    #[serde(flatten, default)]
    pub record_metadata: RecordMetadata,
}

impl From<Connection> for SanitizedConnection {
    fn from(conn: Connection) -> Self {
        Self {
            id: conn.id,
            platform_version: conn.platform_version,
            connection_definition_id: conn.connection_definition_id,
            r#type: conn.r#type,
            key: conn.key,
            group: conn.group,
            name: conn.name,
            environment: conn.environment,
            platform: conn.platform,
            secrets_service_id: conn.secrets_service_id,
            event_access_id: conn.event_access_id,
            identity: conn.identity,
            identity_type: conn.identity_type,
            settings: conn.settings,
            throughput: conn.throughput,
            ownership: conn.ownership,
            oauth: conn.oauth,
            has_error: conn.has_error,
            error: conn.error,
            record_metadata: conn.record_metadata,
        }
    }
}

impl SanitizedConnection {
    pub fn to_value(&self) -> Value {
        serde_json::to_value(self).unwrap_or(Value::Null)
    }
}

impl Hash for Connection {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl PartialEq for Connection {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for Connection {}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, AsRefStr, Default)]
#[serde(rename_all = "camelCase")]
#[strum(serialize_all = "camelCase")]
pub enum OAuth {
    Enabled {
        connection_oauth_definition_id: Id,
        expires_in: Option<i32>,
        #[serde(default)]
        expires_at: Option<i64>,
    },
    #[default]
    Disabled,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Display, AsRefStr)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum ConnectionType {
    Api {},
    DatabaseSql {},
    DatabaseNoSql,
    FileSystem,
    Stream,
    Custom,
}

#[derive(
    Debug, Clone, Copy, Serialize, PartialEq, Eq, Deserialize, Display, AsRefStr, EnumString,
)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum Platform {
    RabbitMq,
    Xero,
    PostgreSql,
    MySql,
    MariaDb,
    MsSql,
    Stripe,
    Sage,
    Shopify,
    Snowflake,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Throughput {
    pub key: String,
    pub limit: u64,
}

use crate::{record_metadata::RecordMetadata, Id};
use http::StatusCode;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Task {
    #[serde(rename = "_id")]
    pub id: Id,
    pub start_time: i64,
    pub end_time: Option<i64>,
    pub payload: Value,
    pub endpoint: String,
    #[serde(with = "http_serde_ext_ios::status_code::option")]
    pub status: Option<StatusCode>,
    #[serde(flatten)]
    pub metadata: RecordMetadata,
}

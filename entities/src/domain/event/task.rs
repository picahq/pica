use crate::{record_metadata::RecordMetadata, Id};
use bytes::Bytes;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Task {
    #[serde(rename = "_id")]
    pub id: Id,
    pub worker_id: i64,
    pub start_time: i64,
    pub end_time: Option<i64>,
    pub payload: Value,
    pub endpoint: String,
    pub status: Option<String>,
    pub r#await: bool,
    pub log_trail: Vec<Bytes>,
    #[serde(flatten)]
    pub metadata: RecordMetadata,
}

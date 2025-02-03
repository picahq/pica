use entities::{record_metadata::RecordMetadata, Id};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Task {
    #[serde(rename = "_id")]
    pub id: Id,
    pub start_time: i64,
    pub end_time: i64,
    pub payload: Value,
    #[serde(flatten)]
    pub metadata: RecordMetadata,
}

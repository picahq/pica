use super::EventMetadata;
use chrono::{DateTime, Utc};
use osentities::{prefix::IdPrefix, Id};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Dumped {
    #[serde(rename = "_id")]
    id: Id,
    reference: Id,
    dumped_at: DateTime<Utc>,
    start_time: i64,
    end_time: i64,
}

impl Dumped {
    pub fn new(id: Id, start_time: DateTime<Utc>, end_time: DateTime<Utc>) -> Self {
        Self {
            id: Id::now(IdPrefix::Archive),
            reference: id,
            dumped_at: Utc::now(),
            start_time: start_time.timestamp_millis(),
            end_time: end_time.timestamp_millis(),
        }
    }

    pub fn start_time(&self) -> i64 {
        self.start_time
    }

    pub fn end_time(&self) -> i64 {
        self.end_time
    }

    pub fn reference(&self) -> Id {
        self.reference
    }
}

impl EventMetadata for Dumped {
    fn reference(&self) -> Id {
        self.reference
    }
}

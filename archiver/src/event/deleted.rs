use osentities::Id;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Deleted {
    reference: Id,
    start_time: i64,
    end_time: i64,
    deleted_count: i64,
}

impl Deleted {
    pub fn new(reference: Id, start_time: i64, end_time: i64, deleted_count: i64) -> Self {
        Self {
            reference,
            start_time,
            end_time,
            deleted_count,
        }
    }

    pub fn reference(&self) -> Id {
        self.reference
    }
}

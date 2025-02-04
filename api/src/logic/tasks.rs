use super::{create, delete, read, update, HookExt, PublicExt, RequestExt};
use crate::server::{AppState, AppStores};
use axum::{
    routing::{patch, post},
    Router,
};
use chrono::Utc;
use entities::{
    event_access::EventAccess, prefix::IdPrefix, record_metadata::RecordMetadata, task::Task, Id,
};
use fake::Dummy;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;

pub fn get_router() -> Router<Arc<AppState>> {
    Router::new()
        .route(
            "/",
            post(create::<CreateRequest, Task>).get(read::<CreateRequest, Task>),
        )
        .route(
            "/:id",
            patch(update::<CreateRequest, Task>).delete(delete::<CreateRequest, Task>),
        )
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, Dummy)]
#[serde(rename_all = "camelCase")]
pub struct CreateRequest {
    pub start_time: i64,
    pub endpoint: String,
    pub payload: Value,
}

impl RequestExt for CreateRequest {
    type Output = Task;

    fn from(&self) -> Option<Task> {
        Some(Task {
            id: Id::now(IdPrefix::Task),
            start_time: Utc::now().timestamp_millis(),
            end_time: None,
            payload: self.payload.clone(),
            endpoint: self.endpoint.clone(),
            status: None,
            metadata: RecordMetadata::default(),
        })
    }

    fn get_store(stores: AppStores) -> entities::MongoStore<Self::Output> {
        stores.tasks
    }

    fn access(&self, _: Arc<EventAccess>) -> Option<Self::Output> {
        self.from()
    }
}
impl HookExt<Task> for CreateRequest {}
impl PublicExt<Task> for CreateRequest {}

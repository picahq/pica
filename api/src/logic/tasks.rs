use super::{create, delete, read, update, HookExt, PublicExt, RequestExt};
use crate::server::{AppState, AppStores};
use axum::{
    routing::{patch, post},
    Router,
};
use chrono::Utc;
use entities::{prefix::IdPrefix, record_metadata::RecordMetadata, Id};
use fake::Dummy;
use http::{Method, StatusCode};
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
pub struct CreateRequest {
    pub start_time: i64,
    pub endpoint: String,
    #[serde(with = "http_serde_ext_ios::method")]
    pub method: Method,
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
            retries: 0,
            endpoint: self.endpoint.clone(),
            method: self.method.clone(),
            status: None,
            metadata: RecordMetadata::default(),
        })
    }

    fn get_store(stores: AppStores) -> entities::MongoStore<Self::Output> {
        stores.tasks
    }
}
impl HookExt<Task> for CreateRequest {}
impl PublicExt<Task> for CreateRequest {}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Task {
    #[serde(rename = "_id")]
    pub id: Id,
    pub start_time: i64,
    pub end_time: Option<i64>,
    pub payload: Value,
    pub retries: u8,
    pub endpoint: String,
    #[serde(with = "http_serde_ext_ios::status_code::option")]
    pub status: Option<StatusCode>,
    #[serde(with = "http_serde_ext_ios::method")]
    pub method: Method,
    #[serde(flatten)]
    pub metadata: RecordMetadata,
}

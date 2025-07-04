use super::{read_without_key, HookExt, PublicExt, RequestExt};
use crate::server::{AppState, AppStores};
use axum::{routing::get, Router};
use bson::doc;
use fake::Dummy;
use osentities::{record_metadata::RecordMetadata, Id, MongoStore};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

pub fn get_router() -> Router<Arc<AppState>> {
    Router::new().route("/", get(read_without_key::<ReadRequest, Knowledge>))
}

struct ReadRequest;

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, Dummy)]
#[serde(rename_all = "camelCase")]
pub struct Knowledge {
    #[serde(rename = "_id")]
    pub id: Id,
    pub connection_platform: String,
    pub title: String,
    pub path: String,
    pub knowledge: Option<String>,
    pub base_url: String,
    #[serde(with = "http_serde_ext_ios::method", alias = "action")]
    pub method: http::Method,
    #[serde(flatten)]
    pub metadata: RecordMetadata,
}

impl HookExt<Knowledge> for ReadRequest {}
impl PublicExt<Knowledge> for ReadRequest {}
impl RequestExt for ReadRequest {
    type Output = Knowledge;

    fn get_store(stores: AppStores) -> MongoStore<Self::Output> {
        stores.knowledge
    }
}

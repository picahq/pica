use super::{read_without_key, HookExt, PublicExt, ReadResponse, RequestExt};
use crate::{
    router::ServerResponse,
    server::{AppState, AppStores},
};
use axum::{
    extract::{Query, State},
    routing::get,
    Json, Router,
};
use bson::doc;
use entities::{record_metadata::RecordMetadata, Id, MongoStore, PicaError};
use fake::Dummy;
use http::HeaderMap;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{collections::BTreeMap, sync::Arc};

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

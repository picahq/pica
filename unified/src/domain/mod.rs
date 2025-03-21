use bson::doc;
use derive_builder::Builder;
use http::StatusCode;
use http::{HeaderMap, HeaderName, HeaderValue};
use osentities::Id;
use osentities::PicaError;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Builder)]
#[builder(setter(into))]
#[serde(rename_all = "camelCase")]
pub struct RequestCrud {
    #[serde(default)]
    query_params: HashMap<String, String>,
    #[serde(with = "http_serde_ext_ios::header_map", default)]
    headers: HeaderMap,
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    body: Option<Value>,
    #[builder(default)]
    path_params: Option<HashMap<String, String>>,
}

impl RequestCrud {
    pub fn get_header(&self, key: &str) -> Option<String> {
        self.headers
            .get(key)
            .map(|v| v.to_str())
            .and_then(|s| s.ok())
            .map(|s| s.to_string())
    }

    pub fn get_body(&self) -> Option<&Value> {
        self.body.as_ref()
    }

    pub fn get_path_params(&self) -> Option<&HashMap<String, String>> {
        self.path_params.as_ref()
    }

    pub fn get_query_params(&self) -> &HashMap<String, String> {
        &self.query_params
    }

    pub fn get_headers(&self) -> &HeaderMap {
        &self.headers
    }

    pub fn remove_query_params(mut self, key: &str) -> (Self, Option<String>) {
        let removed = self.query_params.remove(key);

        (self, removed)
    }

    pub fn extend_query_params(mut self, other: HashMap<String, String>) -> Self {
        self.query_params.extend(other);
        self
    }

    pub fn remove_header(mut self, key: &str) -> (Self, Option<HeaderValue>) {
        let removed = self.headers.remove(key);

        (self, removed)
    }

    pub fn extend_header(mut self, other: HashMap<HeaderName, HeaderValue>) -> Self {
        self.headers.extend(other);
        self
    }

    pub fn as_request_for_id<'a>(&'a self, id: Option<&'a str>) -> RequestForId<'a> {
        RequestForId {
            query_params: &self.query_params,
            headers: &self.headers,
            path_params: id,
        }
    }

    pub fn extend_body(mut self, other: Option<Value>) -> Self {
        match (&mut self.body, other) {
            (Some(Value::Object(a)), Some(Value::Object(b))) => {
                a.extend(b);
            }
            (body @ None, Some(mapped_body)) => {
                body.replace(mapped_body);
            }
            _ => {}
        }
        self
    }

    pub fn add_path_param(mut self, key: String, value: Option<String>) -> Self {
        if let Some(value) = value {
            match self.path_params {
                Some(path_params) => {
                    let mut path_params = path_params;
                    path_params.insert(key, value);
                    self.path_params = Some(path_params);
                }
                None => self.path_params = Some(HashMap::from([(key, value)])),
            }
        }
        self
    }

    pub fn set_body(mut self, body: Option<Value>) -> Self {
        self.body = body;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RequestForId<'a> {
    query_params: &'a HashMap<String, String>,
    #[serde(with = "http_serde_ext_ios::header_map", default)]
    headers: &'a HeaderMap,
    path_params: Option<&'a str>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Builder)]
#[serde(rename_all = "camelCase")]
#[builder(setter(into), build_fn(error = "PicaError"))]
pub struct ResponseCrudToMap<'a> {
    #[serde(with = "http_serde_ext_ios::header_map")]
    headers: &'a HeaderMap,
    #[builder(default)]
    pagination: Option<Value>,
    request: ResponseCrudToMapRequest<'a>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResponseCrudToMapRequest<'a> {
    query_params: &'a HashMap<String, String>,
}

impl<'a> ResponseCrudToMapRequest<'a> {
    pub fn new(query_params: &'a HashMap<String, String>) -> Self {
        Self { query_params }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResponseCrud {
    pagination: Option<Value>,
}

impl ResponseCrud {
    pub fn get_pagination(&self) -> Option<&Value> {
        self.pagination.as_ref()
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Builder)]
#[builder(setter(into), build_fn(error = "PicaError"))]
#[serde(rename_all = "camelCase")]
pub struct UnifiedMetadata {
    timestamp: i64,
    platform_rate_limit_remaining: i32,
    rate_limit_remaining: i32,
    #[builder(default)]
    host: Option<String>,
    #[builder(setter(strip_option), default)]
    cache: Option<UnifiedCache>,
    transaction_key: Id,
    platform: String,
    platform_version: String,
    #[builder(default)]
    action: Option<String>,
    #[builder(default)]
    common_model: Option<String>,
    common_model_version: String,
    #[builder(default)]
    #[serde(with = "http_serde_ext_ios::status_code::option")]
    status_code: Option<StatusCode>,
    #[builder(default)]
    path: Option<String>,
    connection_key: String,
    #[builder(setter(strip_option), default)]
    latency: Option<i32>,
    #[builder(setter(strip_option), default)]
    hash: Option<String>,
}

impl UnifiedMetadata {
    pub fn as_value(&self) -> Value {
        serde_json::to_value(self).unwrap_or_default()
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct UnifiedCache {
    hit: bool,
    ttl: u64,
    key: String,
}

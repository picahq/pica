use super::Metric;
use axum::async_trait;
use osentities::{InternalError, PicaError, Unit};
use posthog_rs::{ClientOptionsBuilder, Event};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

#[async_trait]
pub trait Track<E>: Send + Sync {
    async fn track_metric(&self, metric: &Metric) -> Result<Unit, PicaError>;

    async fn track_many_metrics(&self, metrics: &[Metric]) -> Result<Unit, PicaError>;

    async fn track_event(&self, event: E) -> Result<Unit, PicaError>;

    async fn track_many_events(&self, events: &[E]) -> Result<Unit, PicaError>;
}

pub struct LoggerTracker;

#[async_trait]
impl Track<TrackedMetric> for LoggerTracker {
    async fn track_metric(&self, metric: &Metric) -> Result<Unit, PicaError> {
        let track = metric.track()?;
        tracing::info!("Tracking event: {track:?}");

        Ok(())
    }

    async fn track_many_metrics(&self, metric: &[Metric]) -> Result<Unit, PicaError> {
        metric.iter().for_each(|m| {
            tracing::info!("Tracking event: {m:?}");
        });

        Ok(())
    }

    async fn track_event(&self, _: TrackedMetric) -> Result<Unit, PicaError> {
        Ok(())
    }

    async fn track_many_events(&self, _: &[TrackedMetric]) -> Result<Unit, PicaError> {
        Ok(())
    }
}

pub struct PosthogTracker {
    client: posthog_rs::Client,
}

impl PosthogTracker {
    pub async fn new(key: String, endpoint: String) -> Self {
        let options = ClientOptionsBuilder::default()
            .api_key(key)
            .api_endpoint(endpoint)
            .build()
            .expect("Unable to build client options");

        let client = posthog_rs::client(options).await;
        Self { client }
    }
}

#[async_trait]
impl Track<TrackedMetric> for PosthogTracker {
    async fn track_metric(&self, metric: &Metric) -> Result<Unit, PicaError> {
        let event = metric.track()?;

        self.client.capture(event).await.map_err(|e| {
            tracing::error!("Could not track event: {e}");
            InternalError::io_err("Could not track event", None)
        })?;

        Ok(())
    }

    async fn track_many_metrics(&self, metric: &[Metric]) -> Result<Unit, PicaError> {
        let events = metric
            .iter()
            .map(|m| m.track())
            .collect::<Result<Vec<_>, _>>()?;

        self.client.capture_batch(events).await.map_err(|e| {
            tracing::error!("Could not track event: {e}");
            InternalError::io_err("Could not track event", None)
        })?;

        Ok(())
    }

    async fn track_event(&self, event: TrackedMetric) -> Result<Unit, PicaError> {
        self.client.capture(event.track()?).await.map_err(|e| {
            tracing::error!("Could not track event: {e}");
            InternalError::io_err("Could not track event", None)
        })?;

        Ok(())
    }

    async fn track_many_events(&self, events: &[TrackedMetric]) -> Result<Unit, PicaError> {
        let events = events
            .iter()
            .map(|m| m.track())
            .collect::<Result<Vec<_>, _>>()?;

        self.client.capture_batch(events).await.map_err(|e| {
            tracing::error!("Could not track event: {e}");
            InternalError::io_err("Could not track event", None)
        })?;

        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IdentifyData {
    user_id: String,
    traits: HashMap<String, Value>,
    context: Context,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TrackData {
    event: String,
    user_id: String,
    #[serde(default)]
    properties: HashMap<String, Value>,
    context: Context,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", tag = "path")]
enum TrackType {
    #[serde(rename = "i")]
    Identify { data: IdentifyData },
    #[serde(rename = "t")]
    Track { data: TrackData },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Context {
    locale: String,
    page: Page,
    user_agent: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Page {
    path: String,
    search: String,
    title: String,
    url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TrackedMetric {
    #[serde(flatten)]
    data: TrackType,
}

impl TrackedMetric {
    pub fn event(&self) -> &str {
        match &self.data {
            TrackType::Track {
                data: TrackData { event, .. },
            } => event,
            TrackType::Identify { .. } => "identify",
        }
    }

    pub fn user_id(&self) -> &str {
        match &self.data {
            TrackType::Track {
                data: TrackData { user_id, .. },
            } => user_id.as_str(),
            TrackType::Identify {
                data: IdentifyData { user_id, .. },
            } => user_id.as_str(),
        }
    }

    pub fn track(&self) -> Result<Event, PicaError> {
        match &self.data {
            TrackType::Track {
                data:
                    TrackData {
                        properties,
                        user_id,
                        context,
                        event,
                    },
            } => {
                let mut hashmap = properties.clone();

                hashmap.insert("locale".into(), Value::String(context.locale.clone()));
                hashmap.insert(
                    "user_agent".into(),
                    Value::String(context.user_agent.clone()),
                );
                hashmap.insert("path".into(), Value::String(context.page.path.to_string()));
                hashmap.insert("search".into(), Value::String(context.page.search.clone()));
                hashmap.insert("title".into(), Value::String(context.page.title.clone()));
                hashmap.insert("url".into(), Value::String(context.page.url.clone()));

                let mut event = Event::new(event.clone(), user_id.clone());

                for (key, value) in hashmap {
                    event.insert_prop(key, value)?;
                }

                Ok(event)
            }
            TrackType::Identify {
                data:
                    IdentifyData {
                        user_id,
                        traits,
                        context,
                    },
            } => {
                let mut i_hashmap = traits.clone();
                i_hashmap.insert(
                    "version".into(),
                    traits.get("version").cloned().unwrap_or(Value::Null),
                );
                i_hashmap.insert("locale".into(), Value::String(context.locale.clone()));
                i_hashmap.insert(
                    "user_agent".into(),
                    Value::String(context.user_agent.clone()),
                );
                i_hashmap.insert("user_id".into(), Value::String(user_id.clone()));
                i_hashmap.insert("path".into(), Value::String(context.page.path.to_string()));
                i_hashmap.insert("search".into(), Value::String(context.page.search.clone()));
                i_hashmap.insert("title".into(), Value::String(context.page.title.clone()));
                i_hashmap.insert("url".into(), Value::String(context.page.url.clone()));

                let mut f_hashmap = HashMap::<String, Value>::new();
                f_hashmap.insert(
                    "$set".into(),
                    serde_json::to_value(i_hashmap).unwrap_or_default(),
                );

                let mut event = Event::new("$set", user_id);

                for (key, value) in f_hashmap {
                    event.insert_prop(key, value).inspect_err(|e| {
                        tracing::error!("Could not insert prop: {e}");
                    })?
                }

                Ok(event)
            }
        }
    }
}

use super::Metric;
use axum::async_trait;
use entities::{InternalError, PicaError, Unit};
use posthog_rs::ClientOptionsBuilder;

#[async_trait]
pub trait Track<E>: Send + Sync {
    async fn track(&self, metric: &Metric, event: Option<E>) -> Result<Unit, PicaError>;

    async fn track_many(&self, metrics: &[Metric], events: &[E]) -> Result<Unit, PicaError>;
}

pub struct LoggerTracker;

#[async_trait]
impl Track<posthog_rs::Event> for LoggerTracker {
    async fn track(
        &self,
        metric: &Metric,
        _: Option<posthog_rs::Event>,
    ) -> Result<Unit, PicaError> {
        let track = metric.track()?;
        tracing::info!("Tracking event: {track:?}");

        Ok(())
    }

    async fn track_many(
        &self,
        metric: &[Metric],
        _: &[posthog_rs::Event],
    ) -> Result<Unit, PicaError> {
        metric.iter().for_each(|m| {
            tracing::info!("Tracking event: {m:?}");
        });

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
impl Track<posthog_rs::Event> for PosthogTracker {
    async fn track(
        &self,
        metric: &Metric,
        _: Option<posthog_rs::Event>,
    ) -> Result<Unit, PicaError> {
        let event = metric.track()?;

        self.client.capture(event).await.map_err(|e| {
            tracing::error!("Could not track event: {e}");
            InternalError::io_err("Could not track event", None)
        })?;

        Ok(())
    }

    async fn track_many(
        &self,
        metric: &[Metric],
        _: &[posthog_rs::Event],
    ) -> Result<Unit, PicaError> {
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
}

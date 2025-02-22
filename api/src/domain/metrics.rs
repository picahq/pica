use chrono::{DateTime, Datelike, Utc};
use entities::{
    constant::{CREATED_AT_KEY, DAILY_KEY, MONTHLY_KEY, PLATFORMS_KEY, TOTAL_KEY},
    destination::Action,
    event_access::EventAccess,
    ownership::Ownership,
    Connection, PicaError,
};
use http::HeaderValue;
use posthog_rs::Event;
use serde::Deserialize;
use std::sync::Arc;

#[derive(Debug, Clone, strum::Display, Deserialize)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum MetricType {
    Passthrough(Arc<Connection>),
    Unified(Arc<Connection>),
    RateLimited(
        Arc<EventAccess>,
        #[serde(with = "http_serde_ext_ios::header_value::option")] Option<HeaderValue>,
    ),
}

impl MetricType {
    pub fn event_name(&self) -> &'static str {
        use MetricType::*;
        match self {
            Passthrough(_) => "Called Passthrough API",
            Unified(_) => "Called Unified API",
            RateLimited(_, _) => "Reached Rate Limit",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Metric {
    pub metric_type: MetricType,
    pub date: DateTime<Utc>,
    pub action: Option<Action>,
}

impl Metric {
    pub fn passthrough(connection: Arc<Connection>) -> Self {
        Self {
            metric_type: MetricType::Passthrough(connection),
            date: Utc::now(),
            action: None,
        }
    }

    pub fn unified(connection: Arc<Connection>, action: Action) -> Self {
        Self {
            metric_type: MetricType::Unified(connection),
            date: Utc::now(),
            action: Some(action),
        }
    }

    pub fn rate_limited(event_access: Arc<EventAccess>, key: Option<HeaderValue>) -> Self {
        Self {
            metric_type: MetricType::RateLimited(event_access, key),
            date: Utc::now(),
            action: None,
        }
    }

    pub fn ownership(&self) -> &Ownership {
        use MetricType::*;
        match &self.metric_type {
            Passthrough(c) => &c.ownership,
            Unified(c) => &c.ownership,
            RateLimited(e, _) => &e.ownership,
        }
    }

    fn platform(&self) -> &str {
        use MetricType::*;
        match &self.metric_type {
            Passthrough(c) => c.platform.as_ref(),
            Unified(c) => c.platform.as_ref(),
            RateLimited(e, _) => &e.platform,
        }
    }

    pub fn update_doc(&self) -> bson::Document {
        let platform = self.platform();
        let metric_type = &self.metric_type;
        let day = self.date.day();
        let month = self.date.month();
        let year = self.date.year();
        let daily_key = format!("{year}-{month:02}-{day:02}");
        let monthly_key = format!("{year}-{month:02}");
        bson::doc! {
            "$inc": {
                format!("{metric_type}.{TOTAL_KEY}"): 1,
                format!("{metric_type}.{PLATFORMS_KEY}.{platform}.{TOTAL_KEY}"): 1,
                format!("{metric_type}.{DAILY_KEY}.{daily_key}"): 1,
                format!("{metric_type}.{PLATFORMS_KEY}.{platform}.{DAILY_KEY}.{daily_key}"): 1,
                format!("{metric_type}.{MONTHLY_KEY}.{monthly_key}"): 1,
                format!("{metric_type}.{PLATFORMS_KEY}.{platform}.{MONTHLY_KEY}.{monthly_key}"): 1,
            },
            "$setOnInsert": {
                CREATED_AT_KEY: self.date.timestamp_millis()
            }
        }
    }

    pub fn track(&self) -> Result<Event, PicaError> {
        use MetricType::*;

        match &self.metric_type {
            Unified(conn) => {
                let event_name = self.metric_type.event_name();
                let distinct_id = self
                    .ownership()
                    .clone()
                    .user_id
                    .unwrap_or(self.ownership().id.to_string());
                let mut event = Event::new(event_name, &distinct_id);
                event.insert_prop("connectionDefinitionId", conn.id.to_string())?;
                event.insert_prop("environment", conn.environment)?;
                event.insert_prop("key", &conn.key)?;
                event.insert_prop("platform", self.platform())?;
                event.insert_prop("platformVersion", &conn.platform_version)?;
                event.insert_prop("clientId", self.ownership().client_id.clone())?;
                event.insert_prop("version", &conn.record_metadata.version)?;
                event.insert_prop("commonModel", self.action.as_ref().map(|a| a.name()))?;
                event.insert_prop("action", self.action.as_ref().map(|a| a.action()))?;

                Ok(event)
            }
            Passthrough(conn) => {
                let event_name = self.metric_type.event_name();
                let distinct_id = self
                    .ownership()
                    .clone()
                    .user_id
                    .unwrap_or(self.ownership().id.to_string());

                let mut event = Event::new(event_name, &distinct_id);
                event.insert_prop("connectionDefinitionId", conn.id.to_string())?;
                event.insert_prop("environment", conn.environment)?;
                event.insert_prop("key", &conn.key)?;
                event.insert_prop("platform", self.platform())?;
                event.insert_prop("platformVersion", &conn.platform_version)?;
                event.insert_prop("clientId", self.ownership().client_id.clone())?;
                event.insert_prop("version", &conn.record_metadata.version)?;

                Ok(event)
            }
            RateLimited(event_access, key) => {
                let event_name = self.metric_type.event_name();
                let distinct_id = self
                    .ownership()
                    .clone()
                    .user_id
                    .unwrap_or(self.ownership().id.to_string());

                let mut event = Event::new(event_name, &distinct_id);
                event.insert_prop("environment", event_access.environment)?;
                event.insert_prop(
                    "key",
                    key.as_ref()
                        .map(|k| k.to_str().unwrap_or_default().to_string()),
                )?;
                event.insert_prop("platform", self.platform())?;
                event.insert_prop("clientId", self.ownership().client_id.clone())?;
                event.insert_prop("version", &event_access.record_metadata.version)?;

                Ok(event)
            }
        }
    }
}

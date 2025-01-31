use crate::config::WatchdogConfig;
use cache::remote::RedisCache;
use chrono::Utc;
use entities::{cache::CacheConfig, database::DatabaseConfig, InternalError, PicaError};
use redis::{AsyncCommands, RedisResult};
use std::fmt::Display;
use std::time::Duration;
use tracing::{error, info};

pub struct WatchdogClient {
    watchdog: WatchdogConfig,
    cache: CacheConfig,
    database: DatabaseConfig,
}

impl Display for WatchdogClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let cache = format!("{}", self.cache);
        let database = format!("{}", self.database);
        let watchdog = format!("{}", self.watchdog);

        write!(
            f,
            "WatchdogClient {{ watchdog: {watchdog}, cache: {cache}, database: {database} }}",
        )
    }
}

impl WatchdogClient {
    pub fn new(watchdog: WatchdogConfig, cache: CacheConfig, database: DatabaseConfig) -> Self {
        Self {
            watchdog,
            cache,
            database,
        }
    }

    pub async fn start(self) -> Result<(), PicaError> {
        self.run().await
    }

    pub async fn run(self) -> Result<(), PicaError> {
        info!("Starting watchdog");
        let cache = RedisCache::new(&self.cache).await.map_err(|e| {
            error!("Could not connect to cache: {e}");
            InternalError::io_err(e.to_string().as_str(), None)
        })?;
        let key = self.cache.event_throughput_key.clone();

        info!("Initializing connection to cache");

        let mut redis_clone = cache.inner.clone();
        tokio::spawn(async move {
            loop {
                let _: RedisResult<String> = async { redis_clone.del(key.clone()).await }.await;
                tokio::time::sleep(Duration::from_secs(1)).await;
            }
        });

        let key = self.cache.api_throughput_key.clone();
        let mut redis_clone = cache.inner.clone();

        tracing::info!("Rate limiter enabled. Connecting to initialized cache");

        loop {
            let _: RedisResult<String> = async { redis_clone.del(key.clone()).await }.await;
            tracing::info!("Rate limiter cleared for {key} at {}", Utc::now());
            tokio::time::sleep(Duration::from_secs(
                self.watchdog.rate_limiter_refresh_interval,
            ))
            .await;
        }
    }
}

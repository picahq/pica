use crate::config::{self, WatchdogConfig};
use bson::doc;
use cache::remote::RedisCache;
use chrono::Utc;
use entities::{
    cache::CacheConfig, database::DatabaseConfig, task::Task, Id, InternalError, MongoStore,
    PicaError, Store, Unit,
};
use futures::{stream::FuturesUnordered, StreamExt};
use redis::{AsyncCommands, RedisResult};
use std::fmt::Display;
use std::time::Duration;
use tracing::{error, info};

pub struct WatchdogClient {
    watchdog: WatchdogConfig,
    cache: CacheConfig,
    database: DatabaseConfig,
    client: reqwest::Client,
    tasks: MongoStore<Task>,
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
    pub async fn new(
        watchdog: WatchdogConfig,
        cache: CacheConfig,
        database: DatabaseConfig,
    ) -> Result<Self, PicaError> {
        let http_client = reqwest::ClientBuilder::new().build()?;
        let client = mongodb::Client::with_uri_str(&database.event_db_url).await?;
        let db = client.database(&database.event_db_name);

        let tasks: MongoStore<Task> = MongoStore::new(&db, &Store::Tasks).await?;

        Ok(Self {
            watchdog,
            cache,
            database,
            client: http_client,
            tasks,
        })
    }

    pub async fn start(self) -> Result<Unit, PicaError> {
        self.run().await
    }

    async fn run(self) -> Result<Unit, PicaError> {
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

            let tasks: Vec<Task> = self
                .tasks
                .get_many(
                    Some(doc! {
                    "active": true,
                    "startTime": {
                        "$lte": Utc::now().timestamp_millis()
                    }}),
                    None,
                    None,
                    Some(self.watchdog.max_amount_of_tasks_to_process),
                    None,
                )
                .await?;

            let client = self.client.clone();
            let tasks_store = self.tasks.clone();
            let timeout = self.watchdog.http_client_timeout_secs;

            tokio::spawn(async move {
                let mut tasks = tasks
                    .into_iter()
                    .map(|task| execute(task, client.clone(), tasks_store.clone(), timeout))
                    .collect::<FuturesUnordered<_>>();

                while let Some(result) = tasks.next().await {
                    match result {
                        Ok(id) => tracing::info!("Task {id} executed successfully"),
                        Err(e) => {
                            tracing::error!("Error executing task: {e}");
                        }
                    }
                }
            });

            tracing::info!("Executing next batch of tasks");

            tokio::time::sleep(Duration::from_secs(
                self.watchdog.rate_limiter_refresh_interval,
            ))
            .await;
        }
    }
}

async fn execute(
    task: Task,
    http_client: reqwest::Client,
    tasks_store: MongoStore<Task>,
    timeout: u64,
) -> Result<Id, PicaError> {
    let timeout = if task.r#await {
        Duration::from_secs(300)
    } else {
        Duration::from_secs(timeout)
    };

    let response = http_client
        .post(task.endpoint)
        .timeout(timeout)
        .json(&task.payload)
        .send()
        .await?;

    let status = response.status();
    let mut stream = response.bytes_stream();

    while let Some(item) = stream.next().await {
        tracing::debug!("Response from API {:?}", item);
        tracing::info!("Response length from API {:?}", item.map(|b| b.len()));
    }

    tasks_store
        .collection
        .find_one_and_update(
            doc! {
                "_id": task.id.to_string() // Filter by task ID
            },
            doc! {
                "$set": {
                    "status": status.to_string(),
                    "endTime": Utc::now().timestamp_millis(),
                    "active": false
                }
            },
        )
        .await?;

    Ok(task.id)
}

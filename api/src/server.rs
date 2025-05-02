use crate::{
    domain::{
        track::{LoggerTracker, PosthogTracker, Track, TrackedMetric},
        ConnectionsConfig, K8sMode, Metric,
    },
    helper::{K8sDriver, K8sDriverImpl, K8sDriverLogger},
    logic::{
        connection_oauth_definition::FrontendOauthConnectionDefinition, knowledge::Knowledge,
        openapi::OpenAPIData,
    },
    router,
};
use anyhow::{anyhow, Context, Result};
use axum::Router;
use bson::doc;
use cache::local::{
    ConnectionDefinitionCache, ConnectionHeaderCache, ConnectionModelDefinitionCacheIdKey,
    ConnectionModelDefinitionCacheStringKey, ConnectionOAuthDefinitionCache, EventAccessCache,
};
use mongodb::{options::UpdateOptions, Client, Database};
use osentities::{
    algebra::{DefaultTemplate, MongoStore},
    common_model::{CommonEnum, CommonModel},
    connection_definition::{ConnectionDefinition, PublicConnectionDetails},
    connection_model_definition::ConnectionModelDefinition,
    connection_model_schema::{ConnectionModelSchema, PublicConnectionModelSchema},
    connection_oauth_definition::{ConnectionOAuthDefinition, Settings},
    event_access::EventAccess,
    page::PlatformPage,
    secret::Secret,
    secrets::SecretServiceProvider,
    task::Task,
    user::UserClient,
    Connection, Event, GoogleKms, IOSKms, PlatformData, PublicConnection, SecretExt, Store,
    MAX_BUFFER_SIZE,
    NUM_FLUSH_WORKERS,
};
use std::{sync::Arc, time::Duration};
use tokio::sync::Semaphore;
use tokio::{net::TcpListener, sync::mpsc::Sender, time::timeout, try_join};
use tracing::{error, info, trace, warn};
use unified::unified::{UnifiedCacheTTLs, UnifiedDestination};

#[derive(Clone)]
pub struct AppStores {
    pub clients: MongoStore<UserClient>,
    pub common_enum: MongoStore<CommonEnum>,
    pub common_model: MongoStore<CommonModel>,
    pub connection: MongoStore<Connection>,
    pub connection_config: MongoStore<ConnectionDefinition>,
    pub db: Database,
    pub event: MongoStore<Event>,
    pub event_access: MongoStore<EventAccess>,
    pub frontend_oauth_config: MongoStore<FrontendOauthConnectionDefinition>,
    pub model_config: MongoStore<ConnectionModelDefinition>,
    pub model_schema: MongoStore<ConnectionModelSchema>,
    pub oauth_config: MongoStore<ConnectionOAuthDefinition>,
    pub platform: MongoStore<PlatformData>,
    pub platform_page: MongoStore<PlatformPage>,
    pub public_connection: MongoStore<PublicConnection>,
    pub public_connection_details: MongoStore<PublicConnectionDetails>,
    pub public_model_schema: MongoStore<PublicConnectionModelSchema>,
    pub knowledge: MongoStore<Knowledge>,
    pub secrets: MongoStore<Secret>,
    pub settings: MongoStore<Settings>,
    pub tasks: MongoStore<Task>,
}

#[derive(Clone)]
pub struct AppCaches {
    pub connection_definitions_cache: ConnectionDefinitionCache,
    pub connection_oauth_definitions_cache: ConnectionOAuthDefinitionCache,
    pub connections_cache: ConnectionHeaderCache,
    pub event_access_cache: EventAccessCache,
    pub connection_model_definition: ConnectionModelDefinitionCacheIdKey,
    pub connection_model_definition_string_key: ConnectionModelDefinitionCacheStringKey,
}

#[derive(Clone)]
pub struct AppState {
    pub app_stores: AppStores,
    pub app_caches: AppCaches,
    pub config: ConnectionsConfig,
    pub event_tx: Sender<Event>,
    pub extractor_caller: UnifiedDestination,
    pub http_client: reqwest::Client,
    pub k8s_client: Arc<dyn K8sDriver>,
    pub metric_tx: Sender<Metric>,
    pub openapi_data: OpenAPIData,
    pub secrets_client: Arc<dyn SecretExt>,
    pub tracker_client: Arc<dyn Track<TrackedMetric>>,
    pub template: DefaultTemplate,
}

#[derive(Clone)]
pub struct Server {
    state: Arc<AppState>,
}

impl Server {
    pub async fn init(config: ConnectionsConfig) -> Result<Self> {
        let client = Client::with_uri_str(&config.db_config.event_db_url).await?;
        let db = client.database(&config.db_config.event_db_name);

        let http_client = reqwest::ClientBuilder::new()
            .timeout(Duration::from_secs(config.http_client_timeout_secs))
            .connect_timeout(Duration::from_secs(30))
            .pool_idle_timeout(Duration::from_secs(30))
            .build()?;
        let model_config = MongoStore::new(&db, &Store::ConnectionModelDefinitions).await?;
        let oauth_config = MongoStore::new(&db, &Store::ConnectionOAuthDefinitions).await?;
        let frontend_oauth_config =
            MongoStore::new(&db, &Store::ConnectionOAuthDefinitions).await?;
        let model_schema = MongoStore::new(&db, &Store::ConnectionModelSchemas).await?;
        let public_model_schema =
            MongoStore::new(&db, &Store::PublicConnectionModelSchemas).await?;
        let common_model = MongoStore::new(&db, &Store::CommonModels).await?;
        let common_enum = MongoStore::new(&db, &Store::CommonEnums).await?;
        let secrets = MongoStore::new(&db, &Store::Secrets).await?;
        let connection = MongoStore::new(&db, &Store::Connections).await?;
        let public_connection = MongoStore::new(&db, &Store::Connections).await?;
        let platform = MongoStore::new(&db, &Store::Platforms).await?;
        let platform_page = MongoStore::new(&db, &Store::PlatformPages).await?;
        let public_connection_details =
            MongoStore::new(&db, &Store::PublicConnectionDetails).await?;
        let settings = MongoStore::new(&db, &Store::Settings).await?;
        let connection_config = MongoStore::new(&db, &Store::ConnectionDefinitions).await?;
        let event_access = MongoStore::new(&db, &Store::EventAccess).await?;
        let event = MongoStore::new(&db, &Store::Events).await?;
        let knowledge = MongoStore::new(&db, &Store::ConnectionModelDefinitions).await?;
        let clients = MongoStore::new(&db, &Store::Clients).await?;
        let secrets_store = MongoStore::<Secret>::new(&db, &Store::Secrets).await?;
        let tasks = MongoStore::new(&db, &Store::Tasks).await?;

        let secrets_client: Arc<dyn SecretExt + Sync + Send> = match config.secrets_config.provider
        {
            SecretServiceProvider::GoogleKms => {
                Arc::new(GoogleKms::new(&config.secrets_config, secrets_store).await?)
            }
            SecretServiceProvider::IosKms => {
                Arc::new(IOSKms::new(&config.secrets_config, secrets_store).await?)
            }
        };

        let tracker_client: Arc<dyn Track<TrackedMetric>> = match (
            config.posthog_write_key.as_ref(),
            config.posthog_endpoint.as_ref(),
        ) {
            (Some(key), Some(endpoint)) => {
                Arc::new(PosthogTracker::new(key.to_string(), endpoint.to_string()).await)
            }
            _ => Arc::new(LoggerTracker),
        };

        let extractor_caller = UnifiedDestination::new(
            config.db_config.clone(),
            config.cache_size,
            secrets_client.clone(),
            UnifiedCacheTTLs {
                connection_cache_ttl_secs: config.connection_cache_ttl_secs,
                connection_model_schema_cache_ttl_secs: config
                    .connection_model_schema_cache_ttl_secs,
                connection_model_definition_cache_ttl_secs: config
                    .connection_model_definition_cache_ttl_secs,
                secret_cache_ttl_secs: config.secret_cache_ttl_secs,
            },
        )
        .await
        .with_context(|| "Could not initialize extractor caller")?;

        let app_stores = AppStores {
            db: db.clone(),
            model_config,
            oauth_config,
            platform_page,
            frontend_oauth_config,
            secrets,
            model_schema,
            public_model_schema,
            platform,
            settings,
            common_model,
            common_enum,
            connection,
            public_connection,
            public_connection_details,
            connection_config,
            event_access,
            knowledge,
            event,
            clients,
            tasks,
        };

        let event_access_cache =
            EventAccessCache::new(config.cache_size, config.access_key_cache_ttl_secs);
        let connections_cache =
            ConnectionHeaderCache::new(config.cache_size, config.connection_cache_ttl_secs);
        let connection_definitions_cache = ConnectionDefinitionCache::new(
            config.cache_size,
            config.connection_definition_cache_ttl_secs,
        );
        let connection_oauth_definitions_cache = ConnectionOAuthDefinitionCache::new(
            config.cache_size,
            config.connection_oauth_definition_cache_ttl_secs,
        );
        let connection_model_definition = ConnectionModelDefinitionCacheIdKey::new(
            config.cache_size,
            config.connection_model_definition_cache_ttl_secs,
        );
        let connection_model_definition_string_key = ConnectionModelDefinitionCacheStringKey::new(
            config.cache_size,
            config.connection_model_definition_cache_ttl_secs,
        );

        let openapi_data = OpenAPIData::default();
        openapi_data.spawn_openapi_generation(
            app_stores.common_model.clone(),
            app_stores.common_enum.clone(),
        );

        let k8s_client: Arc<dyn K8sDriver> = match config.k8s_mode {
            K8sMode::Real => Arc::new(K8sDriverImpl::new().await?),
            K8sMode::Logger => Arc::new(K8sDriverLogger),
        };

        // Create Event buffer in separate thread and batch saves
        let events = db.collection::<Event>(&Store::Events.to_string());
        let (event_tx, receiver) =
            tokio::sync::mpsc::channel::<Event>(config.event_save_buffer_size);

        start_event_collector(db.clone(), config.clone(), event_tx.clone(), receiver);

        let (metric_tx, receiver) =
            tokio::sync::mpsc::channel::<Metric>(config.metric_save_channel_size);

        start_metric_collector(
            db.clone(),
            config.clone(),
            tracker_client.clone(),
            metric_tx.clone(),
            receiver,
        );

        let app_caches = AppCaches {
            connection_definitions_cache,
            connection_oauth_definitions_cache,
            connections_cache,
            event_access_cache,
            connection_model_definition,
            connection_model_definition_string_key,
        };

        Ok(Self {
            state: Arc::new(AppState {
                app_stores,
                app_caches,
                config,
                event_tx,
                extractor_caller,
                http_client,
                k8s_client,
                metric_tx,
                openapi_data,
                secrets_client,
                tracker_client,
                template: DefaultTemplate::default(),
            }),
        })
    }

    pub async fn run(&self) -> Result<()> {
        let app = router::get_router(&self.state).await;

        let app: Router<()> = app.with_state(self.state.clone());

        info!("Api server listening on {}", self.state.config.address);

        let tcp_listener = TcpListener::bind(&self.state.config.address).await?;

        axum::serve(tcp_listener, app.into_make_service())
            .await
            .map_err(|e| anyhow!("Server error: {}", e))
    }
}

pub fn start_metric_collector(
    db: mongodb::Database,
    config: ConnectionsConfig,
    tracker_client: Arc<dyn Track<TrackedMetric>>,
    metric_tx: Sender<Metric>,
    mut receiver: tokio::sync::mpsc::Receiver<Metric>,
) -> Sender<Metric> {
    let metrics = db.collection::<Metric>(&Store::Metrics.to_string());
    let metric_system_id = config.metric_system_id.clone();
    let cloned_tracker_client = tracker_client.clone();

    tokio::spawn(async move {
        let options = UpdateOptions::builder().upsert(true).build();
        let mut event_buffer = Vec::new();

        loop {
            let res = timeout(
                Duration::from_secs(config.event_save_timeout_secs),
                receiver.recv(),
            )
            .await;

            if let Ok(Some(metric)) = res {
                let doc = metric.update_doc();
                let client = metrics
                    .update_one(
                        doc! { "clientId": &metric.ownership().client_id },
                        doc.clone(),
                    )
                    .with_options(options.clone());
                let system = metrics
                    .update_one(doc! { "clientId": metric_system_id.as_str() }, doc)
                    .with_options(options.clone());

                if let Err(e) = try_join!(client, system) {
                    error!("Could not upsert metric: {e}");
                } else {
                    trace!("Metric upserted successfully");
                }

                if metric.is_passthrough() {
                    continue;
                }

                event_buffer.push(metric);

                if event_buffer.len() >= MAX_BUFFER_SIZE {
                    flush_buffer(&cloned_tracker_client, &mut event_buffer).await;
                }
            } else if let Ok(None) = res {
                break;
            } else {
                trace!("Event receiver timed out waiting for new event");
                flush_buffer(&cloned_tracker_client, &mut event_buffer).await;
            }
        }

        flush_buffer(&cloned_tracker_client, &mut event_buffer).await;
    });

    metric_tx
}

async fn flush_buffer(
    tracker_client: &Arc<dyn Track<TrackedMetric>>,
    event_buffer: &mut Vec<Metric>,
) {
    if event_buffer.is_empty() {
        return;
    }

    match tracker_client.track_many_metrics(event_buffer).await {
        Ok(_) => {
            trace!("Tracked {} metrics", event_buffer.len());
        }
        Err(e) => {
            warn!("Could not track metrics: {e}");
        }
    }
    event_buffer.clear();
}


pub fn start_event_collector(
    db: mongodb::Database,
    config: ConnectionsConfig,
    event_tx: Sender<Event>,
    mut receiver: tokio::sync::mpsc::Receiver<Event>,
) -> tokio::sync::mpsc::Sender<Event> {
    let events = db.collection::<Event>(&Store::Events.to_string());

    let (flush_tx, mut flush_rx) = tokio::sync::mpsc::channel::<Vec<Event>>(NUM_FLUSH_WORKERS * 2);

    // Semaphore to limit concurrency
    let sem = Arc::new(Semaphore::new(NUM_FLUSH_WORKERS));
    let events_clone = events.clone();
    let sem_clone = sem.clone();

    tokio::spawn(async move {
        while let Some(batch) = flush_rx.recv().await {
            let permit = match sem_clone.clone().acquire_owned().await {
                Ok(p) => p,
                Err(_) => {
                    error!("Semaphore closed unexpectedly");
                    break;
                }
            };

            let events = events_clone.clone();
            tokio::spawn(async move {
                trace!("Inserting {} events", batch.len());
                if let Err(e) = events.insert_many(batch).await {
                    error!("Failed to insert events: {e}");
                }
                drop(permit); // release the permit
            });
        }
    });

    tokio::spawn({
        let flush_sender = flush_tx.clone();
        async move {
            let mut buffer = Vec::with_capacity(config.event_save_buffer_size);

            loop {
                let res = timeout(
                    Duration::from_secs(config.event_save_timeout_secs),
                    receiver.recv(),
                )
                .await;

                let is_timeout = if let Ok(Some(event)) = res {
                    buffer.push(event);
                    false
                } else if let Ok(None) = res {
                    break;
                } else {
                    trace!("Event receiver timed out");
                    true
                };

                if buffer.len() == config.event_save_buffer_size
                    || (is_timeout && !buffer.is_empty())
                {
                    trace!("Flushing {} events", buffer.len());
                    let to_send = std::mem::take(&mut buffer);
                    if let Err(e) = flush_sender.send(to_send).await {
                        error!("Failed to send buffer: {e}");
                        break;
                    }
                }
            }

            if !buffer.is_empty() {
                trace!("Final flush of {} events", buffer.len());
                if let Err(e) = flush_sender.send(buffer).await {
                    error!("Failed to send final buffer: {e}");
                }
            }
        }
    });

    event_tx
}

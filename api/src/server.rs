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
use cache::local::{
    ConnectionDefinitionCache, ConnectionHeaderCache, ConnectionModelDefinitionCacheIdKey,
    ConnectionModelDefinitionCacheStringKey, ConnectionOAuthDefinitionCache, EventAccessCache,
};
use mongodb::options::ClientOptions;
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
};
use std::{sync::Arc, time::Duration};
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
        let (event_tx, mut receiver) =
            tokio::sync::mpsc::channel::<Event>(config.event_save_buffer_size);
        tokio::spawn(async move {
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
                    trace!("Event receiver timed out waiting for new event");
                    true
                };
                // Save when buffer is full or timeout elapsed
                if buffer.len() == config.event_save_buffer_size
                    || (is_timeout && !buffer.is_empty())
                {
                    trace!("Saving {} events", buffer.len());
                    let to_save = std::mem::replace(
                        &mut buffer,
                        Vec::with_capacity(config.event_save_buffer_size),
                    );
                    let events = events.clone();
                    tokio::spawn(async move {
                        if let Err(e) = events.insert_many(to_save).await {
                            error!("Could not save buffer of events: {e}");
                        }
                    });
                }
            }
        });

        // Update metrics in separate thread
        let template = DefaultTemplate::default();

        let metrics = db.collection::<Metric>(&Store::Metrics.to_string());
        let (metric_tx, mut receiver) =
            tokio::sync::mpsc::channel::<Metric>(config.metric_save_channel_size);
        let metric_system_id = config.metric_system_id.clone();
        let cloned_tracker_client = tracker_client.clone();
        tokio::spawn(async move {
            let options = UpdateOptions::builder().upsert(true).build();
            let mut event_buffer = vec![];

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
                            bson::doc! {
                                "clientId": &metric.ownership().client_id,
                            },
                            doc.clone(),
                        )
                        .with_options(options.clone());
                    let system = metrics
                        .update_one(
                            bson::doc! {
                                "clientId": metric_system_id.as_str(),
                            },
                            doc,
                        )
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
                } else if let Ok(None) = res {
                    break;
                } else {
                    trace!("Event receiver timed out waiting for new event");
                    if let Err(e) = cloned_tracker_client
                        .track_many_metrics(&event_buffer)
                        .await
                    {
                        warn!("Could not track metrics: {e}");
                    } else {
                        trace!("Tracked {} metrics", event_buffer.len());
                    }
                    event_buffer.clear();
                }
            }
        });

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
                template,
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

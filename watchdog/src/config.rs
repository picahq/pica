use entities::{cache::CacheConfig, database::DatabaseConfig};
use envconfig::Envconfig;
use std::fmt::{Display, Formatter};

#[derive(Envconfig, Clone)] // Intentionally no Debug so secret is not printed
pub struct WatchdogConfig {
    #[envconfig(from = "RATE_LIMITER_REFRESH_INTERVAL", default = "60")]
    pub rate_limiter_refresh_interval: u64,
    #[envconfig(from = "HTTP_CLIENT_TIMEOUT_SECS", default = "10")]
    pub http_client_timeout_secs: u64,
    #[envconfig(from = "MAX_AMOUNT_OF_TASKS_TO_PROCESS", default = "100")]
    pub max_amount_of_tasks_to_process: u64,
    #[envconfig(nested = true)]
    pub redis: CacheConfig,
    #[envconfig(nested = true)]
    pub db: DatabaseConfig,
}

impl Display for WatchdogConfig {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "RATE_LIMITER_REFRESH_INTERVAL: {}",
            self.rate_limiter_refresh_interval
        )?;
        writeln!(
            f,
            "HTTP_CLIENT_TIMEOUT_SECS: {}",
            self.http_client_timeout_secs
        )?;
        writeln!(f, "{}", self.redis)?;
        writeln!(f, "{}", self.db)
    }
}

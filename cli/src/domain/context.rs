use crate::{domain::config::CliConfig, service::Printer};
use reqwest::{Client, ClientBuilder};
use std::time::Duration;

pub struct AppContext {
    config: CliConfig,
    printer: Printer,
    http: Client,
}

impl AppContext {
    pub fn new(config: CliConfig) -> AppContext {
        let timeout = Duration::from_millis(config.http().timeout().unwrap_or_default());
        let client = ClientBuilder::new()
            .timeout(timeout)
            .build()
            .expect("Could not build client");

        AppContext {
            config,
            printer: Printer,
            http: client,
        }
    }

    pub fn printer(&self) -> &Printer {
        &self.printer
    }

    pub fn config(&self) -> &CliConfig {
        &self.config
    }

    pub fn http(&self) -> &Client {
        &self.http
    }
}

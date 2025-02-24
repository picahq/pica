use crate::{
    domain::constant::{
        CONFIG_FILE_NAME, CONFIG_FILE_PATH, CONFIG_NOT_FOUND_MESSAGE_ERR,
        RUN_PICA_CONFIGURATION_SUG,
    },
    service::{Pica, Printer},
};
use clap::error::ErrorKind;
use config::Config;
use directories::BaseDirs;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Default, Deserialize, PartialEq, Eq, Serialize)]
pub struct CliConfig {
    server: Server,
    keys: Keys,
    http: Http,
}

#[derive(Debug, Default, Deserialize, PartialEq, Eq, Serialize)]
pub struct Server {
    api: String,
    base: String,
}

impl Server {
    pub fn new(api: String, base: String) -> Self {
        Self { api, base }
    }

    pub fn api(&self) -> &str {
        &self.api
    }

    pub fn base(&self) -> &str {
        &self.base
    }
}

#[derive(Debug, Default, Deserialize, PartialEq, Eq, Serialize)]
pub struct Keys {
    sandbox: String,
    production: String,
}

impl Keys {
    pub fn new(sandbox: String, production: String) -> Self {
        Self {
            sandbox,
            production,
        }
    }

    pub fn sandbox(&self) -> &str {
        &self.sandbox
    }

    pub fn production(&self) -> &str {
        &self.production
    }
}

#[derive(Debug, Default, Deserialize, PartialEq, Eq, Serialize)]
pub struct Http {
    timeout: Option<u64>,
}

impl Http {
    pub fn new(timeout: Option<u64>) -> Self {
        Self { timeout }
    }

    pub fn timeout(&self) -> Option<u64> {
        self.timeout
    }
}

impl CliConfig {
    pub fn new(server: Server, keys: Keys, http: Http) -> Self {
        Self { keys, http, server }
    }

    pub fn load() -> CliConfig {
        let (base_path, _, path_str) = CliConfig::path();

        if !base_path.exists() {
            Printer.stderr::<Pica>(
                CONFIG_NOT_FOUND_MESSAGE_ERR,
                ErrorKind::InvalidValue,
                RUN_PICA_CONFIGURATION_SUG,
                true,
            );

            std::process::exit(0);
        } else {
            let configuration = Config::builder()
                .add_source(config::File::with_name(&path_str))
                .add_source(config::Environment::with_prefix("Pica"))
                .build()
                .expect("Could not build configuration");

            configuration
                .try_deserialize()
                .expect("Could not deserialize configuration")
        }
    }

    /// Returns the paths for the configuration directory, configuration file, and the configuration file path as a string.
    ///
    /// This method constructs the paths for the configuration directory and the configuration file
    /// based on the user's home directory and a predefined configuration path. It also returns the
    /// configuration file path as a string.
    ///
    /// It first checks if the user's home directory can be determined and joins it with a predefined
    /// configuration directory. Then it constructs the full path for the configuration file and
    /// provides both the path objects and a string representation of the configuration file path.
    ///
    /// # Returns
    /// This function returns a tuple of:
    /// - The base path for the configuration directory (`PathBuf`).
    /// - The full path to the configuration file (`PathBuf`).
    /// - The configuration file path as a `String`.
    ///
    /// # Panics
    /// This function will panic if:
    /// - The home directory cannot be found.
    /// - The path cannot be converted to a string.
    pub fn path() -> (PathBuf, PathBuf, String) {
        let base_path = BaseDirs::new()
            .map(|d| d.home_dir().join(CONFIG_FILE_PATH))
            .expect("Could not find configuration directory");
        let config_path = base_path.join(CONFIG_FILE_NAME);
        let path_str = config_path
            .to_str()
            .expect("Could not convert path to string");

        (
            base_path.to_owned(),
            config_path.to_owned(),
            path_str.to_string(),
        )
    }

    pub fn http(&self) -> &Http {
        &self.http
    }

    pub fn server(&self) -> &Server {
        &self.server
    }

    pub fn keys(&self) -> &Keys {
        &self.keys
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Default)]
pub struct ReadResponse<T> {
    rows: Vec<T>,
    total: u64,
    skip: u64,
    limit: u64,
}

impl<T> ReadResponse<T> {
    pub fn rows(&self) -> &[T] {
        &self.rows
    }
}

use crate::{
    domain::Server as ConfigServer,
    domain::{
        CHECK_PORT_FOR_SERVER_SUG, CliConfig, DEFAULT_API, DEFAULT_BASE, GO_TO_URL, Http, Keys,
        RUN_PICA_CONNECTION_LIST_SUG,
    },
    service::{Pica, Printer},
};
use axum::{Router, extract::Query, routing::get};
use clap::error::ErrorKind;
use entities::{InternalError, PicaError, Unit};
use reqwest::Client;
use serde::Deserialize;
use serde_json::json;
use std::{
    fs::{File, create_dir_all},
    io::Write,
    sync::Arc,
};
use tokio::sync::Mutex;
use tokio::{net::TcpListener, sync::oneshot};

#[derive(Debug, Deserialize, Default)]
pub struct QueryParams {
    code: String,
}

#[derive(Debug, Deserialize, Default)]
pub struct OnboardingResponse {
    #[serde(rename = "liveKey")]
    production: String,
    #[serde(rename = "testKey")]
    sandbox: String,
}

pub struct Server;

impl Server {
    /// Starts the server, listens for incoming HTTP requests, and handles shutdown.
    ///
    /// This method sets up the Axum web server to listen on port `8080` and defines a route
    /// for handling the `/callback` path. The server will await a shutdown signal and
    /// will gracefully stop once the shutdown signal is received.
    ///
    /// # Workflow
    /// 1. The server is initialized and begins listening for incoming connections.
    /// 2. When a request is received on `/callback`, the `handle_callback` function is triggered.
    /// 3. After handling the callback request, the server will shut down by sending a signal through
    ///    the `shutdown_tx` channel.
    ///
    /// # Parameters
    /// This method does not take any parameters.
    ///
    /// # Returns
    /// This function returns a `Result<Unit, PicaError>`. It will return `Ok(())` if the server
    /// is started successfully, or an error if any part of the server setup or operation fails.
    pub async fn start(
        base_url: Option<String>,
        api_url: Option<String>,
    ) -> Result<Unit, PicaError> {
        let printer = Printer;

        let (shutdown_tx, shutdown_rx) = oneshot::channel::<Unit>();
        let shutdown_tx = Arc::new(Mutex::new(Some(shutdown_tx)));

        let url = format!(
            "{}/public/v3/users/oauth/provider/github",
            api_url.clone().unwrap_or(DEFAULT_BASE.to_string())
        );

        let router = Router::new().route(
            "/callback",
            get({
                let shutdown_tx = shutdown_tx.clone();
                move |query| Server::handle_callback(query, shutdown_tx, printer, base_url, api_url)
            }),
        );

        let listener = TcpListener::bind("0.0.0.0:8082").await.map_err(|e| {
            printer.stderr::<Pica>(
                &format!("{e}"),
                ErrorKind::Io,
                CHECK_PORT_FOR_SERVER_SUG,
                true,
            );
            InternalError::io_err(&format!("{e}"), None)
        })?;

        printer.stdout(&(GO_TO_URL.to_string() + &url));

        let server = axum::serve(listener, router);
        let server_handle = tokio::spawn(async move {
            server.await.map_err(|e| {
                printer.stderr::<Pica>(&format!("{e}"), ErrorKind::Io, None, true);
                InternalError::io_err(&format!("{e}"), None)
            })?;

            Ok::<Unit, PicaError>(())
        });

        let aborter = server_handle.abort_handle();

        tokio::select! {
            _ = shutdown_rx => {
                aborter.abort();
            }
            _ = server_handle => {}
        }

        Ok(())
    }

    /// Handles the callback request after a successful authorization.
    ///
    /// This method is triggered when a request is made to the `/callback` route. It extracts the
    /// query parameters, writes the values to a configuration file (`config.toml`), and triggers
    /// the shutdown of the server by sending a signal through the `shutdown_tx` channel.
    ///
    /// The `sandbox` and `production` values from the query parameters are written to the config file
    /// in the following format:
    ///
    /// ```toml
    /// [keys]
    /// sandbox = "<sandbox_value>"
    /// production = "<production_value>"
    /// ```
    ///
    /// # Parameters
    /// - `Query(params)`: The query parameters from the URL, including `sandbox` and `production` values.
    /// - `shutdown_tx`: A shared `Arc<Mutex<Option<oneshot::Sender<Unit>>>>` that is used to send a shutdown signal to the server once the callback is processed.
    /// - `printer`: A `Printer` instance used for logging error and status messages.
    ///
    /// # Returns
    /// This function returns a `Result<Unit, PicaError>`. It will return `Ok(())` if the callback is processed successfully
    /// and the shutdown signal is sent, or an error if any issue occurs during processing.
    async fn handle_callback(
        Query(params): Query<QueryParams>,
        shutdown_tx: Arc<Mutex<Option<oneshot::Sender<Unit>>>>,
        printer: Printer,
        base_url: Option<String>,
        api_url: Option<String>,
    ) -> Result<Unit, PicaError> {
        let url = format!(
            "{}/auth/github",
            api_url.clone().unwrap_or(DEFAULT_API.to_string())
        );

        let response: OnboardingResponse = Client::new()
            .post(url)
            .json(&json!({
                "code": params.code,
                "isTerminal": true
            }))
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;

        let (_, path, _) = CliConfig::path();

        if let Some(parent) = path.parent() {
            create_dir_all(parent).map_err(|e| {
                printer.stderr::<Pica>(&format!("{e}"), ErrorKind::Io, None, true);
                InternalError::io_err(&format!("{e}"), None)
            })?;
        }
        let mut file = File::create(path).map_err(|e| {
            printer.stderr::<Pica>(&format!("{e}"), ErrorKind::Io, None, true);
            InternalError::io_err(&format!("{e}"), None)
        })?;

        let config = {
            let http = Http::new(Some(30000));
            let keys = Keys::new(response.sandbox, response.production);
            let server = ConfigServer::new(
                api_url.clone().unwrap_or(DEFAULT_API.to_string()),
                base_url.clone().unwrap_or(DEFAULT_BASE.to_string()),
            );

            toml::to_string(&CliConfig::new(server, keys, http)).map_err(|e| {
                printer.stderr::<Pica>(&format!("{e}"), ErrorKind::Io, None, true);
                InternalError::io_err(&format!("{e}"), None)
            })?
        };

        file.write_all(config.as_bytes()).map_err(|e| {
            printer.stderr::<Pica>(&format!("{e}"), ErrorKind::Io, None, true);
            InternalError::io_err(&format!("{e}"), None)
        })?;

        let mut shutdown_tx = shutdown_tx.lock().await; // Lock to access the shutdown channel
        if let Some(tx) = shutdown_tx.take() {
            let _ = tx.send(());
        }

        printer.stdout(RUN_PICA_CONNECTION_LIST_SUG);

        Ok(())
    }
}

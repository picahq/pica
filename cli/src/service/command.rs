use super::{Server, handle_response, open_browser, readline};
use crate::{
    algebra::Handler,
    domain::{
        ABOUT, AppContext, CHECK_AVAILABLE_CONN_DEFS_SUG, CHECK_LIMIT_SUG,
        CONN_DEF_NOT_FOUND_MESSAGE_ERR, CONNECTION_NOT_FOUND_MESSAGE_ERR, CliConfig, DEFAULT_LIMIT,
        FORM_VALIDATION_FAILED, HEADER_SECRET_KEY, LIMIT_GREATER_THAN_EXPECTED,
        RUN_LIST_COMMANDS_SUG, ReadResponse, Step, URL_PROVIDED_IS_INVALID,
    },
};
use clap::{Args, Parser, Subcommand, ValueEnum, error::ErrorKind};
use osentities::{
    EmbedTokenSlim, Event, InternalError, PicaError, PublicConnection, Unit,
    connection_definition::ConnectionDefinition,
};
use serde::Deserialize;
use serde_json::{Value, json};
use std::{
    collections::{HashMap, HashSet},
    fmt::{Display, Formatter},
    hash::RandomState,
};
use tabled::{Table, settings::Style};
use url::Url;

/// Build performant, high-converting native integrations with a few lines of code. By unlocking more integrations, you can onboard more customers and expand app usage, overnight.
#[derive(Debug, Parser)]
#[command(name = "pica")]
#[command(
    long_about = ABOUT
)]
pub struct Pica {
    #[command(subcommand)]
    command: Command,
}

impl Pica {
    pub fn command(&self) -> &Command {
        &self.command
    }
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Manage connections via the CLI.
    Connection(Connection),
    /// Configures the CLI. It truncates the configuration file and creates a new one.
    Login {
        /// Base url of the API
        #[arg(short, long)]
        base: Option<String>,
        /// API url of the API
        #[arg(short, long)]
        api: Option<String>,
    },
}

#[derive(Debug, Args)]
#[command(args_conflicts_with_subcommands = true)]
#[command(flatten_help = true)]
pub struct Connection {
    #[command(subcommand)]
    command: ConnectionCommand,
}

#[derive(Debug, Subcommand)]
enum ConnectionCommand {
    /// Create a new connection
    Create {
        /// Platform to create a connection for. Run the command without this argument to see the available platforms
        #[arg(short, long)]
        platform: Option<String>,
        /// Whether to create a connection via the web interface or the CLI
        #[arg(short, long, default_value_t = false, default_missing_value = "true")]
        web: bool,
        #[arg(short,
            long,
            default_value_t = Environment::Sandbox,
            default_missing_value = "sandbox",
            value_enum,
            require_equals = true,
            num_args = 0..=1
        )]
        env: Environment,
    },
    /// Delete a connection
    Delete {
        /// Key of the connection to delete
        #[arg(short, long)]
        key: String,
        /// Environment to delete connection from
        #[arg(short,
            long,
            default_value_t = Environment::Sandbox,
            default_missing_value = "sandbox",
            value_enum,
            require_equals = true,
            num_args = 0..=1
        )]
        env: Environment,
    },
    /// List connections
    List {
        /// Limit of amount of connections to list
        #[arg(short, long)]
        limit: Option<u32>,
        /// Filter by connection key
        #[arg(short, long)]
        key: Option<String>,
        /// Environment to list connections connections from
        #[arg(short,
            long,
            default_value_t = Environment::Sandbox,
            default_missing_value = "sandbox",
            value_enum,
            require_equals = true,
            num_args = 0..=1
        )]
        env: Environment,
    },
}

#[derive(Debug, Deserialize, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum Environment {
    Sandbox,
    Production,
}

impl Display for Environment {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.to_possible_value()
            .expect("Could not convert environment to possible value")
            .get_name()
            .fmt(f)
    }
}

impl Handler<AppContext, Command, Event> for Pica {
    async fn load(&self) -> Result<AppContext, PicaError> {
        match self.command() {
            Command::Login { base, api } => {
                Server::start(base.clone(), api.clone()).await?;

                Ok(AppContext::new(CliConfig::load()))
            }
            _ => Ok(AppContext::new(CliConfig::load())),
        }
    }

    async fn validate(&self, ctx: &AppContext) -> Result<Unit, PicaError> {
        match &self.command {
            Command::Login { base, api } => {
                if let Some(base) = base {
                    Url::parse(base).map_err(|e| {
                        ctx.printer().stderr::<Pica>(
                            URL_PROVIDED_IS_INVALID,
                            ErrorKind::InvalidValue,
                            None,
                            true,
                        );
                        InternalError::invalid_argument(&format!("{e}"), None)
                    })?;
                }

                if let Some(api) = api {
                    Url::parse(api).map_err(|e| {
                        ctx.printer().stderr::<Pica>(
                            URL_PROVIDED_IS_INVALID,
                            ErrorKind::InvalidValue,
                            None,
                            true,
                        );
                        InternalError::invalid_argument(&format!("{e}"), None)
                    })?;
                }

                Ok(())
            }
            Command::Connection(Connection { command }) => match command {
                ConnectionCommand::Create { .. } => Ok(()),
                ConnectionCommand::Delete { .. } => Ok(()),
                ConnectionCommand::List { limit, .. } => {
                    if limit.map(|l| l > 100).is_some() {
                        ctx.printer().stderr::<Pica>(
                            LIMIT_GREATER_THAN_EXPECTED,
                            ErrorKind::InvalidValue,
                            CHECK_LIMIT_SUG,
                            true,
                        );
                    }
                    Ok(())
                }
            },
        }
    }

    async fn run(&self, ctx: &AppContext) -> Result<Unit, PicaError> {
        match &self.command {
            Command::Login { .. } => Ok(()),
            Command::Connection(Connection { command }) => match command {
                ConnectionCommand::Create { platform, web, env } => {
                    let secret = match env {
                        Environment::Sandbox => ctx.config().keys().sandbox(),
                        Environment::Production => ctx.config().keys().production(),
                    };
                    let url = format!(
                        "{}/public/v1/event-links/create-embed-token",
                        ctx.config().server().api()
                    );

                    let embed_defs = handle_response::<EmbedTokenSlim>(
                        ctx.http()
                            .post(url)
                            .json(&json!({}))
                            .header(HEADER_SECRET_KEY, secret)
                            .send()
                            .await,
                        ctx.printer(),
                    )
                    .await?;

                    let platforms: HashSet<String, RandomState> = HashSet::from_iter(
                        embed_defs
                            .link_settings
                            .connected_platforms
                            .iter()
                            .map(|c| c.r#type.clone()),
                    );

                    match platform {
                        Some(platform) if platforms.contains(platform) => {
                            let url = format!(
                                "{}/v1/public/connection-definitions?platform={}",
                                ctx.config().server().api(),
                                platform
                            );

                            let conn_def = handle_response::<ReadResponse<ConnectionDefinition>>(
                                ctx.http().get(url).send().await,
                                ctx.printer(),
                            )
                            .await?;
                            let conn_def = conn_def.rows().first();

                            if *web || conn_def.is_some_and(|cd| cd.is_oauth()) {
                                let url = format!(
                                    "{}/connections#open={}",
                                    ctx.config().server().base(),
                                    platform
                                );

                                open_browser(ctx.printer(), url.to_string());
                            } else {
                                match conn_def {
                                    Some(conn_def) => {
                                        let steps = Step::from(conn_def);

                                        let form = steps.iter().try_fold(
                                            HashMap::new(),
                                            |mut form, s| {
                                                ctx.printer().write(s.question());
                                                let line = readline()?;
                                                let line = line.trim();

                                                if line.is_empty() {
                                                    ctx.printer().stderr::<Pica>(
                                                        FORM_VALIDATION_FAILED,
                                                        ErrorKind::InvalidValue,
                                                        None,
                                                        true,
                                                    );

                                                    Err(InternalError::invalid_argument(
                                                        FORM_VALIDATION_FAILED,
                                                        None,
                                                    ))
                                                } else {
                                                    form.insert(
                                                        s.key().to_string(),
                                                        line.to_string(),
                                                    );

                                                    Ok(form)
                                                }
                                            },
                                        )?;

                                        let link_token = embed_defs.link_settings.event_inc_token;

                                        let url = format!(
                                            "{}/public/v1/event-links/create-embed-connection",
                                            ctx.config().server().api()
                                        );

                                        let payload = &json!({
                                            "linkToken": link_token,
                                            "authFormData": serde_json::to_value(&form).unwrap_or_default(),
                                            "type": conn_def.platform,
                                            "connectionDefinitionId": conn_def.id.to_string()
                                        });

                                        let connection = handle_response::<PublicConnection>(
                                            ctx.http()
                                                .post(url)
                                                .json(payload)
                                                .header(HEADER_SECRET_KEY, secret)
                                                .send()
                                                .await,
                                            ctx.printer(),
                                        )
                                        .await?;

                                        ctx.printer()
                                            .stdout("The following connection was created:");

                                        ctx.printer().stdout(
                                            &Table::new(vec![connection])
                                                .with(Style::modern_rounded())
                                                .to_string(),
                                        );
                                    }
                                    None => ctx.printer().stderr::<Pica>(
                                        &format!("{}{}", CONN_DEF_NOT_FOUND_MESSAGE_ERR, platform),
                                        ErrorKind::InvalidValue,
                                        CHECK_AVAILABLE_CONN_DEFS_SUG,
                                        true,
                                    ),
                                }
                            }
                        }
                        _ => {
                            ctx.printer().stdout(CHECK_AVAILABLE_CONN_DEFS_SUG);

                            ctx.printer().stdout(
                                &Table::new(embed_defs.link_settings.connected_platforms)
                                    .with(Style::modern_rounded())
                                    .to_string(),
                            );
                        }
                    }

                    Ok(())
                }
                ConnectionCommand::Delete { key, env } => {
                    let url = format!("{}/v1/connections?key={key}", ctx.config().server().api());

                    let secret = match env {
                        Environment::Sandbox => ctx.config().keys().sandbox(),
                        Environment::Production => ctx.config().keys().production(),
                    };

                    let connection = handle_response::<ReadResponse<PublicConnection>>(
                        ctx.http()
                            .get(url)
                            .header(HEADER_SECRET_KEY, secret)
                            .send()
                            .await,
                        ctx.printer(),
                    )
                    .await?;

                    let id = connection.rows().first().map(|c| c.id);

                    match id {
                        None => {
                            ctx.printer().stderr::<Pica>(
                                CONNECTION_NOT_FOUND_MESSAGE_ERR,
                                ErrorKind::InvalidValue,
                                RUN_LIST_COMMANDS_SUG,
                                true,
                            );
                        }
                        Some(id) => {
                            let url =
                                format!("{}/v1/connections/{id}", ctx.config().server().api());

                            handle_response::<Value>(
                                ctx.http()
                                    .delete(url)
                                    .header(HEADER_SECRET_KEY, secret)
                                    .send()
                                    .await,
                                ctx.printer(),
                            )
                            .await?;

                            ctx.printer()
                                .stdout("The following connection was deleted:");

                            ctx.printer().stdout(
                                &Table::new(connection.rows())
                                    .with(Style::modern_rounded())
                                    .to_string(),
                            );
                        }
                    };

                    Ok(())
                }
                ConnectionCommand::List { limit, key, env } => {
                    let secret = match env {
                        Environment::Sandbox => ctx.config().keys().sandbox(),
                        Environment::Production => ctx.config().keys().production(),
                    };

                    let url = match key {
                        Some(key) => {
                            format!("{}/v1/connections?key={key}", ctx.config().server().api())
                        }
                        None => {
                            format!(
                                "{}/v1/connections?limit={}",
                                ctx.config().server().api(),
                                limit.unwrap_or(DEFAULT_LIMIT)
                            )
                        }
                    };

                    let connections = handle_response::<ReadResponse<PublicConnection>>(
                        ctx.http()
                            .get(url)
                            .header(HEADER_SECRET_KEY, secret)
                            .send()
                            .await,
                        ctx.printer(),
                    )
                    .await?;

                    ctx.printer().stdout(
                        &Table::new(connections.rows())
                            .with(Style::modern_rounded())
                            .to_string(),
                    );

                    Ok(())
                }
            },
        }
    }
}

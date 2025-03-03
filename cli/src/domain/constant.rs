// Configuration
pub const CONFIG_FILE_NAME: &str = "credentials.toml";
pub const CONFIG_FILE_PATH: &str = ".pica";
pub const DEFAULT_LIMIT: u32 = 10;
pub const HEADER_SECRET_KEY: &str = "x-pica-secret";
pub const DEFAULT_API: &str = "https://api.picaos.com";
pub const DEFAULT_BASE: &str = "https://picaos.com";
pub const DEFAULT_PORT: u64 = 30000;

// Error messages
pub const CONFIG_NOT_FOUND_MESSAGE_ERR: &str = "You don't seem to have a configuration file.";
pub const CONNECTION_NOT_FOUND_MESSAGE_ERR: &str = "Connection with key not found: ";
pub const CONN_DEF_NOT_FOUND_MESSAGE_ERR: &str = "Connection definition not found: ";
pub const LIMIT_GREATER_THAN_EXPECTED: &str = "Limit must be less than 100";
pub const CALLBACK_SERVER_NOT_RUNNING: &str = "Callback server is not running";
pub const URL_PROVIDED_IS_INVALID: &str = "URL provided is invalid";
pub const FORM_VALIDATION_FAILED: &str = "Form validation failed";

// Suggestions
pub const RUN_PICA_CONFIGURATION_SUG: &str = "Run `pica login` to create a configuration file.";
pub const CHECK_INTERNET_CONNECTION_SUG: &str = "Check your internet connection and try again";
pub const CHECK_PARAMETERS_SUG: &str = "Check the parameters and try again";
pub const CHECK_LIMIT_SUG: &str = "Try with a smaller limit";
pub const RUN_LIST_COMMANDS_SUG: &str = "Run list command to check your available connections";
pub const CHECK_PORT_FOR_SERVER_SUG: &str =
    "Check the port that the server is running on and try again";
pub const RUN_PICA_CONNECTION_LIST_SUG: &str = "The configuration file was successfully created. Try listing your existing connections with `pica connection list`";
pub const CHECK_AVAILABLE_CONN_DEFS_SUG: &str =
    "The following connections are available [use `platform` to create a connection]:";

// Instructions
pub const GO_TO_URL: &str = "Go to the following url: ";
pub const GO_TO_TERMINAL: &str = "Please continue in the terminal. Happy hacking!";

// Metadata
pub const ABOUT: &str = "Build performant, high-converting native integrations with a few lines of code. By unlocking more integrations, you can onboard more customers and expand app usage, overnight.";

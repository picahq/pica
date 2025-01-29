use percent_encoding::{AsciiSet, NON_ALPHANUMERIC};

// Metric constants
pub const TOTAL_KEY: &str = "total";
pub const DAILY_KEY: &str = "daily";
pub const MONTHLY_KEY: &str = "monthly";
pub const PLATFORMS_KEY: &str = "platforms";
pub const CREATED_AT_KEY: &str = "createdAt";

// Mongo filter constants
pub const DELETED_FILTER: &str = "deleted";
pub const OWNERSHIP_FILTER: &str = "ownership.buildableId";
pub const ENVIRONMENT_FILTER: &str = "environment";
pub const DUAL_ENVIRONMENT_HEADER: &str = "x-pica-show-all-environments";
pub const LIMIT_FILTER: &str = "limit";
pub const SKIP_FILTER: &str = "skip";
pub const QUERY_BY_ID_PASSTHROUGH: &str = "x-pica-action-id";

// JWT constants
pub const BEARER_PREFIX: &str = "Bearer ";
pub const DEFAULT_AUDIENCE: &str = "pica-users";
pub const DEFAULT_ISSUER: &str = "pica";
pub const FALLBACK_AUDIENCE: &str = "integrationos-users";
pub const FALLBACK_ISSUER: &str = "integrationos";

// Event Access constants
pub const DEFAULT_NAMESPACE: &str = "default";

// Connection constants
pub const APP_LABEL: &str = "app";
pub const DATABASE_TYPE_LABEL: &str = "database-type";
pub const JWT_SECRET_REF_KEY: &str = "jwt-secret";
pub const JWT_SECRET_REF_NAME: &str = "event-secrets";

// Header constants
pub const PICA_PASSTHROUGH_HEADER: &str = "x-pica-passthrough";

// Encryption constants
pub const HASH_LENGTH: usize = 32;
pub const IV_LENGTH: usize = 16;
pub const PASSWORD_LENGTH: usize = 32;
pub const HASH_PREFIX: &str = "\x19Event Signed Message:\n";
pub const EVENT_VERSION: &str = "v1";

// OpenAPI constants
pub const URI: &str = "https://api.picaos.com/v1/unified";
pub const OPENAPI_VERSION: &str = "3.0.3";
pub const SPEC_VERSION: &str = "1.0.0";
pub const TITLE: &str = "Common Models";
pub const X_PICA_SECRET: &str = "X-PICA-SECRET";
pub const X_PICA_CONNECTION_KEY: &str = "X-PICA-CONNECTION-KEY";
pub const X_PICA_ENABLE_PASSTHROUGH: &str = "X-PICA-ENABLE-PASSTHROUGH";
pub const X_PICA_PASSTHROUGH_FORWARD: &str = "X-PICA-PASSTHROUGH-FORWARD";
pub const META: &str = "meta";
pub const STATUS: &str = "status";
pub const STATUS_CODE: &str = "statusCode";
pub const UNIFIED: &str = "unified";
pub const PASSTHROUGH: &str = "passthrough";
pub const CONNECTION_DEFINITION_KEY: &str = "connectionDefinitionKey";
pub const CONNECTION_KEY: &str = "connectionKey";
pub const TRANSACTION_KEY: &str = "transactionKey";
pub const TXN: &str = "txn";
pub const PLATFORM: &str = "platform";
pub const PLATFORM_VERSION: &str = "platformVersion";
pub const ACTION: &str = "action";
pub const COMMON_MODEL: &str = "commonModel";
pub const COMMON_MODEL_VERSION: &str = "commonModelVersion";
pub const HASH: &str = "hash";
pub const MODIFY_TOKEN: &str = "modifyToken";
pub const HEARTBEATS: &str = "heartbeats";
pub const TOTAL_TRANSACTIONS: &str = "totalTransactions";
pub const CACHE: &str = "cache";
pub const HIT: &str = "hit";
pub const TTL: &str = "ttl";
pub const KEY: &str = "key";
pub const RATE_LIMIT_REMAINING: &str = "rateLimitRemaining";
pub const PLATFORM_RATE_LIMIT_REMAINING: &str = "platformRateLimitRemaining";
pub const LATENCY: &str = "latency";
pub const TIMESTAMP: &str = "timestamp";
pub const COUNT: &str = "count";
pub const PAGINATION: &str = "pagination";
pub const CURSOR: &str = "cursor";
pub const NEXT_CURSOR: &str = "nextCursor";
pub const PREV_CURSOR: &str = "previousCursor";
pub const LIMIT: &str = "limit";
pub const CREATED_AFTER: &str = "createdAfter";
pub const CREATED_BEFORE: &str = "createdBefore";
pub const UPDATED_AFTER: &str = "updatedAfter";
pub const UPDATED_BEFORE: &str = "updatedBefore";

// Unified constants
pub const ID_KEY: &str = "id";
pub const BODY_KEY: &str = "body";
pub const MODIFY_TOKEN_KEY: &str = "modifyToken";
pub const PASSTHROUGH_PARAMS: &str = "passthroughForward";
pub const PASSTHROUGH_HEADERS: &str = "x-pica-passthrough-forward";
pub const UNIFIED_KEY: &str = "unified";
pub const COUNT_KEY: &str = "count";
pub const PASSTHROUGH_KEY: &str = "passthrough";
pub const LIMIT_KEY: &str = "limit";
pub const PAGE_SIZE_KEY: &str = "pageSize";
pub const PAGINATION_KEY: &str = "pagination";
pub const STATUS_HEADER_KEY: &str = "response-status";
pub const META_KEY: &str = "meta";
pub const ACTION_KEY: &str = "action";

// Database constants
pub const MAX_LIMIT: usize = 100;

// OAuth constants

pub const NONCE_LEN: usize = 12;
pub const OAUTH_CALLBACK: &str = "oauth_callback";
pub const OAUTH_VERIFIER: &str = "oauth_verifier";
pub const OAUTH_CONSUMER_KEY: &str = "oauth_consumer_key";
pub const OAUTH_NONCE: &str = "oauth_nonce";
pub const OAUTH_SIGNATURE: &str = "oauth_signature";
pub const OAUTH_SIGNATURE_METHOD: &str = "oauth_signature_method";
pub const OAUTH_TIMESTAMP: &str = "oauth_timestamp";
pub const OAUTH_TOKEN: &str = "oauth_token";
pub const OAUTH_VERSION: &str = "oauth_version";
pub const HMAC_LENGTH_ERROR: &str = "HMAC has no key length restrictions";
pub const EXCLUDE: &AsciiSet = &NON_ALPHANUMERIC
    .remove(b'-')
    .remove(b'.')
    .remove(b'_')
    .remove(b'~');

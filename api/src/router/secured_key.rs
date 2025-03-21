use crate::{
    logic::{
        connection, connection_definition,
        connection_model_definition::{get_available_actions, test_connection_model_definition},
        connection_model_schema::{
            public_get_connection_model_schema, PublicGetConnectionModelSchema,
        },
        event_access, events, knowledge, metrics, oauth, passthrough, secrets, tasks, unified,
        vault_connection,
    },
    middleware::{
        header_auth,
        header_blocker::{handle_blocked_error, BlockInvalidHeaders},
        header_passthrough,
        rate_limiter::{rate_limit_middleware, RateLimiter},
    },
    server::AppState,
};
use axum::{
    error_handling::HandleErrorLayer,
    middleware::{from_fn, from_fn_with_state},
    routing::{get, post},
    Router,
};
use http::HeaderName;
use osentities::{
    connection_model_schema::PublicConnectionModelSchema, telemetry::log_request_middleware,
};
use std::{iter::once, sync::Arc};
use tower::{filter::FilterLayer, ServiceBuilder};
use tower_http::{sensitive_headers::SetSensitiveRequestHeadersLayer, trace::TraceLayer};
use tracing::warn;

pub async fn get_router(state: &Arc<AppState>) -> Router<Arc<AppState>> {
    let routes = Router::new()
        .layer(TraceLayer::new_for_http())
        .nest("/connections", connection::get_router())
        .nest("/event-access", event_access::get_router())
        .nest("/events", events::get_router())
        .nest("/knowledge", knowledge::get_router())
        .nest("/tasks", tasks::get_router())
        .nest("/metrics", metrics::get_router())
        .nest("/oauth", oauth::get_router())
        .nest("/passthrough", passthrough::get_router())
        .nest("/secrets", secrets::get_router())
        .nest("/unified", unified::get_router())
        .nest("/vault/connections", vault_connection::get_router())
        .route(
            "/connection-model-definitions/test/:id",
            post(test_connection_model_definition),
        )
        .route(
            "/connection-model-schema",
            get(public_get_connection_model_schema::<
                PublicGetConnectionModelSchema,
                PublicConnectionModelSchema,
            >),
        )
        .route(
            "/available-connectors",
            get(connection_definition::get_available_connectors),
        )
        .route("/available-actions/:platform", get(get_available_actions));

    let routes = match RateLimiter::from_state(state.clone()).await {
        Ok(rate_limiter) => routes.layer(axum::middleware::from_fn_with_state(
            Arc::new(rate_limiter),
            rate_limit_middleware,
        )),
        Err(e) => {
            warn!("Could not connect to redis: {e}");
            routes
        }
    };

    routes
        .layer(from_fn_with_state(
            state.clone(),
            header_auth::header_auth_middleware,
        ))
        .layer(from_fn_with_state(
            state.clone(),
            header_passthrough::header_passthrough_middleware,
        ))
        .layer(from_fn(log_request_middleware))
        .layer(TraceLayer::new_for_http())
        .layer(SetSensitiveRequestHeadersLayer::new(once(
            HeaderName::from_lowercase(state.config.headers.auth_header.as_bytes()).unwrap(),
        )))
        .layer(
            ServiceBuilder::new()
                .layer(HandleErrorLayer::new(handle_blocked_error))
                .layer(FilterLayer::new(
                    BlockInvalidHeaders::from_state(state.clone()).await,
                )),
        )
}

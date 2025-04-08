use super::connection::{
    create_connection, delete_connection, get_vault_connections, update_connection,
};
use crate::server::AppState;
use axum::{
    routing::{delete, get, patch, post},
    Router,
};

use std::sync::Arc;

pub fn get_router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", post(create_connection))
        .route("/:id", patch(update_connection))
        .route("/:id", delete(delete_connection))
        .route("/", get(get_vault_connections))
}

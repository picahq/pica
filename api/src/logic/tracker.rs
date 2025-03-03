use crate::{domain::track::TrackedMetric, server::AppState};
use axum::{extract::State, routing::post, Json, Router};
use osentities::{PicaError, Unit};
use std::sync::Arc;

pub fn get_router() -> Router<Arc<AppState>> {
    Router::new().route("/", post(create_tracking))
}

pub async fn create_tracking(
    state: State<Arc<AppState>>,
    Json(payload): Json<TrackedMetric>,
) -> Result<Unit, PicaError> {
    state.tracker_client.track_event(payload).await?;

    Ok(())
}

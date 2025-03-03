pub mod k8s_driver;
pub mod shape_mongo_filter;

pub use k8s_driver::*;
pub use shape_mongo_filter::*;

use axum::{extract::Path, Json};
use http::StatusCode;
use osentities::{prefix::IdPrefix, Id, PicaError};
use serde::Serialize;

#[derive(Serialize)]
pub struct GenerateIdResponse {
    pub id: String,
}

pub async fn generate_id(
    Path(id_prefix): Path<String>,
) -> Result<(StatusCode, Json<GenerateIdResponse>), PicaError> {
    let id_prefix_str = id_prefix.as_str();

    let id_prefix = IdPrefix::try_from(id_prefix_str)?;

    let id = Id::now(id_prefix);

    Ok((
        StatusCode::OK,
        Json(GenerateIdResponse { id: id.to_string() }),
    ))
}

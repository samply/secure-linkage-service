use axum::{
    Router,
    routing::{get, post},
};

use super::handlers::{get_linkage_status, handle_linkage_request};
use crate::rest_api::AppState;

pub fn linkage_routes() -> Router<AppState> {
    Router::new()
        .route(
            "/linkageResult/{source_party}/{destination_party}",
            post(handle_linkage_request),
        )
        .route("/linkage/{linkage_id}/status", get(get_linkage_status))
}

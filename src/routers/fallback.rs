use axum::{http::StatusCode, response::IntoResponse, Router};

use crate::AppState;

/// Builds the fallback router.
pub fn build() -> Router<AppState> {
    Router::new().fallback(fallback_handler)
}

/// The fallback handler that returns a 404 JSON response.
#[axum::debug_handler]
#[tracing::instrument]
async fn fallback_handler() -> impl IntoResponse {
    (StatusCode::NOT_FOUND, "not found")
}

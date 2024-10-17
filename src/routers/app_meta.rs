use axum::{http::StatusCode, response::IntoResponse, routing::get, Json, Router};
use serde_json::json;

use crate::AppState;

/// Builds the fallback router.
pub fn build() -> Router<AppState> {
    Router::new()
        .route("/v1/test", get(test_handler))
        .route("/v1/health", get(health_handler))
        .route("/v1/version", get(version_handler))
}

/// `/test` handler that always returns a 200
#[axum::debug_handler]
#[tracing::instrument]
async fn test_handler() -> impl IntoResponse {
    StatusCode::OK
}

/// `/health` handler that returns a 200 if everything is good, or a 500
/// otherwise.
#[axum::debug_handler]
#[tracing::instrument]
async fn health_handler() -> impl IntoResponse {
    StatusCode::OK
}

/// `/version` handler that resturns version and commit info as JSON
#[axum::debug_handler]
#[tracing::instrument]
async fn version_handler() -> impl IntoResponse {
    Json(json!({
        "git": env!("GIT_COMMIT_SHORT"),
        "dirty": env!("GIT_DIRTY"),
        "version": env!("CARGO_PKG_VERSION"),
    }))
}

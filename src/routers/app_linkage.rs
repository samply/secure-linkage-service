use std::collections::HashMap;

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::Deserialize;
use serde_json::Value;

use crate::AppState;

/// Builds the fallback router.
pub fn build() -> Router<AppState> {
    Router::new()
        .route("/v1/testConnection/:remote_id", get(testconnection_handler))
        .route("/v1/freshIDs/:remote_id", get(idgen_handler))
        .route(
            "/v1/linkageResult/:local_id/:remote_id",
            post(linkage_handler),
        )
}

/// `/test` handler that always returns a 200
#[axum::debug_handler]
#[tracing::instrument]
async fn testconnection_handler(
    State(state): State<AppState>,
    Path(remote_id): Path<String>,
) -> impl IntoResponse {
    if remote_id == "Test" {
        (StatusCode::NO_CONTENT, "")
    } else {
        (StatusCode::FORBIDDEN, "Host not configured")
    }
}

#[derive(Deserialize, Debug, Clone, Copy)]
struct IDGenParams {
    count: usize,
}
impl Default for IDGenParams {
    fn default() -> Self {
        Self { count: 1 }
    }
}

/// `/health` handler that returns a 200 if everything is good, or a 500
/// otherwise.
#[axum::debug_handler]
#[tracing::instrument]
async fn idgen_handler(
    State(state): State<AppState>,
    Path(remote_id): Path<String>,
    count: Option<Query<IDGenParams>>,
) -> impl IntoResponse {
    let count = count.unwrap_or_default().count;
    let ids: Vec<String> = vec![
        "dfc2fd55-37ff-4f41-b3e3-65f5135134c6".into(),
        "11e928a5-c065-45d4-90fa-e2a058d9b4f9u".into(),
        "36912758-d382-4e6f-8f54-dee10507dacd".into(),
        count.to_string(),
    ];
    Json(ids)
}

/// `/version` handler that resturns version and commit info as JSON
#[axum::debug_handler]
#[tracing::instrument]
async fn linkage_handler(
    State(state): State<AppState>,
    Path((local_id, remote_id)): Path<(String, String)>,
    Json(data): Json<Value>,
) -> impl IntoResponse {
    StatusCode::NO_CONTENT
}

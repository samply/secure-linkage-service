use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::Deserialize;
use serde_json::Value;
use tracing::debug;

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
    let mut result: Vec<String> = Vec::with_capacity(count);
    {
        let mut parties = state.parties.write().expect("CANNOT WRITE LOCK PARTIES");
        if let Some(party) = parties.get_mut(remote_id.as_str()) {
            result = party.generate_ids(count);
        } else {
            return (StatusCode::NOT_FOUND, "Remote ID not registered").into_response();
        };
    }
    debug!("Result: {:?}", result);
    if result.len() == 1 {
        Json(result.get(0).unwrap()).into_response()
    } else {
        Json(result).into_response()
    }
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

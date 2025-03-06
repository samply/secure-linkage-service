use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use reqwest::header;
use serde::Deserialize;
use serde_json::Value;
use tracing::debug;

use crate::{types::Computation, AppState};

/// Builds the fallback router.
pub fn build() -> Router<AppState> {
    Router::new()
        .route("/v1/testConnection/:remote_id", get(testconnection_handler))
        .route("/v1/freshIDs/:remote_id", get(idgen_handler))
        .route(
            "/v1/linkageResult/:local_id/:remote_id",
            post(linkage_handler),
        )
        .route("/v1/result/:local_id/:remote_id", get(result_handler))
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
    {
        let parties = state
            .parties
            .read()
            .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Cannot access Parties"))?;
        if !(parties.contains_key(local_id.as_str()) && parties.contains_key(remote_id.as_str())) {
            return Err((StatusCode::BAD_REQUEST, "Invalid party ids"));
        }
    }
    let mut comp = state.computations.write().map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Cannot access computations",
        )
    })?;
    if let Some(cmp) = comp.get_mut(&(local_id.clone(), remote_id.clone())) {
        cmp.add_result(
            serde_json::from_value(data)
                .map_err(|e| (StatusCode::BAD_REQUEST, "Cannot deserialize data"))?,
        );
        if cmp.is_result_ready() {
            let mut results = state
                .results
                .write()
                .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Cannot write to results"))?;
            results.insert(
                (local_id.clone(), remote_id.clone()),
                cmp.reconstruct_result().map_err(|_| {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "Cannot reconstruct result",
                    )
                })?,
            );
            cmp.reset();
            return Ok((StatusCode::OK, [(header::LOCATION, format!("/v1/result/{local_id}/{remote_id}"))], "Result saved in queue"));
        }
    } else {
        let mut cmp = Computation::new();
        cmp.add_result(
            serde_json::from_value(data)
                .map_err(|e| (StatusCode::BAD_REQUEST, "Cannot deserialize data"))?,
        );
        comp.insert((local_id, remote_id), cmp);
    }
    Ok((StatusCode::OK, [(header::LOCATION, String::new())], "Share set. Not all shares set, yet"))
}
#[axum::debug_handler]
#[tracing::instrument]
async fn result_handler(
    State(state): State<AppState>,
    Path((local_id, remote_id)): Path<(String, String)>,
) -> impl IntoResponse {
    {
        let parties = state
            .parties
            .read()
            .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Cannot access Parties".to_string()))?;
        if !(parties.contains_key(local_id.as_str()) && parties.contains_key(remote_id.as_str())) {
            return Err((StatusCode::BAD_REQUEST, "Invalid party ids".to_string()));
        }
    }
    let results = state.results.read().map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Cannot access computations".to_string(),
        )
    })?;
    if let Some(result) = results.get(&(local_id, remote_id)) {
        Ok((StatusCode::OK, serde_json::to_string(&result).unwrap()))
    } else {
        Err((StatusCode::NOT_FOUND, "Result not ready (yet)".to_string()))
    }
}

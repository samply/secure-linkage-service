use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tracing::{error, info};

use crate::rest_api::AppState;
use crate::services::fresh_record_ids_service::FreshRecordIdsServiceError;

#[derive(Debug, Deserialize)]
pub struct FreshRecordIdsQuery {
    pub n: Option<u32>,
    pub count: Option<u32>,
}

#[derive(Debug, Serialize)]
pub struct FreshRecordIdsResponse {
    pub status: String,
    pub party: String,
    pub count: u32,
    pub fresh_record_ids: Vec<String>,
    pub message: Option<String>,
}

pub async fn generate_fresh_record_ids(
    Path(party): Path<String>,
    Query(query): Query<FreshRecordIdsQuery>,
    State(state): State<AppState>,
) -> Result<Json<FreshRecordIdsResponse>, (StatusCode, Json<FreshRecordIdsResponse>)> {
    let count = query.n.or(query.count).unwrap_or(1);

    info!("Generating {} fresh IDs for party: {}", count, party);

    // Get party from the party service
    let party_obj = match state.party_service.get_party(&party).await {
        Some(p) => p,
        None => {
            return Err((
                StatusCode::NOT_FOUND,
                Json(FreshRecordIdsResponse {
                    status: "error".to_string(),
                    party,
                    count,
                    fresh_record_ids: vec![],
                    message: Some("Party not found".to_string()),
                }),
            ));
        }
    };

    // Generate fresh IDs using the service layer
    match state
        .fresh_record_ids_service
        .generate_fresh_ids(&party_obj, count)
        .await
    {
        Ok(ids) => {
            let fresh_record_ids: Vec<String> =
                ids.into_iter().map(|id| id.0.0.to_string()).collect();

            Ok(Json(FreshRecordIdsResponse {
                status: "success".to_string(),
                party,
                count,
                fresh_record_ids,
                message: None,
            }))
        }
        Err(err) => {
            let (status_code, message) = map_fresh_ids_service_error(&err);
            if matches!(status_code, StatusCode::INTERNAL_SERVER_ERROR) {
                error!("Fresh IDs service error: {:?}", err);
            }

            Err((
                status_code,
                Json(FreshRecordIdsResponse {
                    status: "error".to_string(),
                    party,
                    count,
                    fresh_record_ids: vec![],
                    message: Some(message),
                }),
            ))
        }
    }
}

pub async fn generate_legacy_fresh_ids(
    Path(party): Path<String>,
    Query(query): Query<FreshRecordIdsQuery>,
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    match generate_fresh_record_ids(Path(party.clone()), Query(query), State(state)).await {
        Ok(Json(response)) => Ok(Json(json!({ "linkageIds": response.fresh_record_ids }))),
        Err((status, Json(response))) => Err((
            status,
            Json(json!({
                "error": response.message.unwrap_or_else(|| "fresh ID generation failed".to_string())
            })),
        )),
    }
}

fn map_fresh_ids_service_error(err: &FreshRecordIdsServiceError) -> (StatusCode, String) {
    match err {
        FreshRecordIdsServiceError::PartyNotFound(msg) => {
            (StatusCode::NOT_FOUND, format!("Party not found: {}", msg))
        }
        FreshRecordIdsServiceError::InvalidCount(msg) => (StatusCode::BAD_REQUEST, msg.clone()),
        FreshRecordIdsServiceError::GenerationFailed(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Fresh ID generation failed: {}", e),
        ),
        FreshRecordIdsServiceError::InternalError(msg) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Internal error: {}", msg),
        ),
    }
}

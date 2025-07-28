use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize, Serialize};
use tracing::{error, info};

use crate::rest_api::AppState;
use crate::services::fresh_ids_service::FreshIdsServiceError;

#[derive(Debug, Deserialize)]
pub struct FreshIdsQuery {
    pub n: Option<u32>,
}

#[derive(Debug, Serialize)]
pub struct FreshIdsResponse {
    pub status: String,
    pub party: String,
    pub count: u32,
    pub fresh_ids: Vec<String>,
    pub message: Option<String>,
}

pub async fn generate_fresh_ids(
    Path(party): Path<String>,
    Query(query): Query<FreshIdsQuery>,
    State(state): State<AppState>,
) -> Result<Json<FreshIdsResponse>, (StatusCode, Json<FreshIdsResponse>)> {
    let count = query.n.unwrap_or(1);

    info!("Generating {} fresh IDs for party: {}", count, party);

    // Get party from the party service
    let party_obj = match state.party_service.get_party(&party).await {
        Some(p) => p,
        None => {
            return Err((
                StatusCode::NOT_FOUND,
                Json(FreshIdsResponse {
                    status: "error".to_string(),
                    party,
                    count,
                    fresh_ids: vec![],
                    message: Some("Party not found".to_string()),
                }),
            ));
        }
    };

    // Generate fresh IDs using the service layer
    match state
        .fresh_ids_service
        .generate_fresh_ids(&party_obj, count)
        .await
    {
        Ok(ids) => {
            let fresh_ids: Vec<String> = ids.into_iter().map(|id| id.0.0.to_string()).collect();

            Ok(Json(FreshIdsResponse {
                status: "success".to_string(),
                party,
                count,
                fresh_ids,
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
                Json(FreshIdsResponse {
                    status: "error".to_string(),
                    party,
                    count,
                    fresh_ids: vec![],
                    message: Some(message),
                }),
            ))
        }
    }
}

fn map_fresh_ids_service_error(err: &FreshIdsServiceError) -> (StatusCode, String) {
    match err {
        FreshIdsServiceError::PartyNotFound(msg) => {
            (StatusCode::NOT_FOUND, format!("Party not found: {}", msg))
        }
        FreshIdsServiceError::InvalidCount(msg) => (StatusCode::BAD_REQUEST, msg.clone()),
        FreshIdsServiceError::GenerationFailed(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Fresh ID generation failed: {}", e),
        ),
        FreshIdsServiceError::InternalError(msg) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Internal error: {}", msg),
        ),
    }
}

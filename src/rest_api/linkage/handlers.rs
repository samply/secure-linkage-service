use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use tracing::{error, info};

use super::responses::{LinkageResult, map_linkage_service_error};
use crate::rest_api::AppState;
use crate::services::linkage_service::LinkageRequest;

pub async fn handle_linkage_request(
    Path((source_party, destination_party)): Path<(String, String)>,
    State(state): State<AppState>,
    Json(request): Json<LinkageRequest>,
) -> LinkageResult {
    info!(
        "Received linkage request from {} to {}",
        source_party, destination_party
    );

    // Get parties from the party service
    let source_party_obj = state
        .party_service
        .get_party(&source_party)
        .await
        .ok_or(StatusCode::BAD_REQUEST)?;

    let dest_party_obj = state
        .party_service
        .get_party(&destination_party)
        .await
        .ok_or(StatusCode::BAD_REQUEST)?;

    // Process the linkage request using the linkage service
    match state
        .linkage_service
        .process_linkage_request(
            source_party,
            destination_party,
            &source_party_obj,
            &dest_party_obj,
            request,
        )
        .await
    {
        Ok(response) => Ok((StatusCode::OK, Json(response))),
        Err(err) => {
            let (status_code, response) = map_linkage_service_error(err);
            if matches!(status_code, StatusCode::INTERNAL_SERVER_ERROR) {
                error!(
                    "Linkage service error: {}",
                    response.message.as_deref().unwrap_or("Unknown")
                );
            }
            Ok((status_code, Json(response)))
        }
    }
}

pub async fn get_linkage_status(
    Path(linkage_id): Path<String>,
    State(_state): State<AppState>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // TODO: Implement linkage status lookup
    Ok(Json(serde_json::json!({
        "linkage_id": linkage_id,
        "status": "pending",
        "message": "Status lookup not yet implemented"
    })))
}

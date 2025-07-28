use axum::{
    Router,
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    routing::post,
};
use std::sync::Arc;
use tokio::time::Duration;
use tracing::{error, info};

use crate::services::LinkageService;
use crate::services::PartyService;
use crate::services::linkage_service::{LinkageRequest, LinkageResponse, LinkageServiceError};

#[derive(Debug, Clone)]
pub struct AppState {
    pub linkage_service: Arc<LinkageService>,
    pub party_service: Arc<PartyService>,
}

impl AppState {
    pub fn new() -> Self {
        Self::with_timeout(Duration::from_secs(20))
    }

    pub fn with_timeout(timeout_duration: Duration) -> Self {
        Self {
            linkage_service: Arc::new(LinkageService::with_timeout(timeout_duration)),
            party_service: Arc::new(PartyService::new()),
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

pub async fn start_server(address: &str) -> Result<(), Box<dyn std::error::Error>> {
    let state = AppState::new();

    let app = Router::new()
        .route(
            "/linkageResult/{source_party}/{destination_party}",
            post(handle_linkage_request),
        )
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(address).await?;
    info!("Server listening on {}", address);

    axum::serve(listener, app).await?;
    Ok(())
}

pub async fn handle_linkage_request(
    Path((source_party, destination_party)): Path<(String, String)>,
    State(state): State<AppState>,
    Json(request): Json<LinkageRequest>,
) -> Result<(StatusCode, Json<LinkageResponse>), StatusCode> {
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
            let (status_code, response) = match err {
                LinkageServiceError::InvalidRequestPairing(msg) => (
                    StatusCode::BAD_REQUEST,
                    LinkageResponse {
                        status: "error".to_string(),
                        linkage_ids: None,
                        message: Some(format!("Invalid request pairing: {}", msg)),
                    },
                ),
                LinkageServiceError::LinkageComputationFailed(e) => {
                    error!("Linkage computation failed: {}", e);
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        LinkageResponse {
                            status: "error".to_string(),
                            linkage_ids: None,
                            message: Some(format!("Linkage computation failed: {}", e)),
                        },
                    )
                }
                LinkageServiceError::InternalError(msg) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    LinkageResponse {
                        status: "error".to_string(),
                        linkage_ids: None,
                        message: Some(format!("Internal error: {}", msg)),
                    },
                ),
                LinkageServiceError::Timeout => (
                    StatusCode::REQUEST_TIMEOUT,
                    LinkageResponse {
                        status: "timeout".to_string(),
                        linkage_ids: None,
                        message: Some("No matching request received within timeout".to_string()),
                    },
                ),
            };
            Ok((status_code, Json(response)))
        }
    }
}

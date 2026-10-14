use axum::http::StatusCode;
use axum::response::Json;

use crate::services::linkage_service::{LinkageResponse, LinkageServiceError};

pub type LinkageResult = Result<(StatusCode, Json<LinkageResponse>), StatusCode>;

pub fn map_linkage_service_error(err: LinkageServiceError) -> (StatusCode, LinkageResponse) {
    match err {
        LinkageServiceError::InvalidRequestPairing(msg) => (
            StatusCode::BAD_REQUEST,
            LinkageResponse {
                status: "error".to_string(),
                linkage_ids: None,
                result: None,
                message: Some(format!("Invalid request pairing: {}", msg)),
            },
        ),
        LinkageServiceError::LinkageComputationFailed(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            LinkageResponse {
                status: "error".to_string(),
                linkage_ids: None,
                result: None,
                message: Some(format!("Linkage computation failed: {}", e)),
            },
        ),
        LinkageServiceError::InternalError(msg) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            LinkageResponse {
                status: "error".to_string(),
                linkage_ids: None,
                result: None,
                message: Some(format!("Internal error: {}", msg)),
            },
        ),
        LinkageServiceError::Timeout => (
            StatusCode::REQUEST_TIMEOUT,
            LinkageResponse {
                status: "timeout".to_string(),
                linkage_ids: None,
                result: None,
                message: Some("No matching request received within timeout".to_string()),
            },
        ),
    }
}

use axum::http::StatusCode;
use axum::response::Json;
use serde::{Deserialize, Serialize};

pub type PartyResult<T> = Result<(StatusCode, Json<T>), StatusCode>;

#[derive(Debug, Serialize, Deserialize)]
pub struct PartyInfo {
    pub name: String,
    pub hostname: String,
    pub status: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PartiesListResponse {
    pub parties: Vec<PartyInfo>,
    pub count: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PartyRegistrationRequest {
    pub hostname: String,
    // Note: In a real implementation, you'd handle key material securely
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PartyRegistrationResponse {
    pub status: String,
    pub party_name: String,
    pub message: Option<String>,
}

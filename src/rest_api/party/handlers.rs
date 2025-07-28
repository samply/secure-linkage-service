use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use tracing::info;

use super::responses::{
    PartiesListResponse, PartyInfo, PartyRegistrationRequest, PartyRegistrationResponse,
    PartyResult,
};
use crate::rest_api::AppState;

pub async fn get_party_info(
    Path(party_name): Path<String>,
    State(state): State<AppState>,
) -> PartyResult<PartyInfo> {
    match state.party_service.get_party(&party_name).await {
        Some(party) => {
            let party_info = PartyInfo {
                name: party_name,
                hostname: party.hostname.to_string(),
                status: "active".to_string(),
            };
            Ok((StatusCode::OK, Json(party_info)))
        }
        None => Err(StatusCode::NOT_FOUND),
    }
}

pub async fn list_parties(State(state): State<AppState>) -> PartyResult<PartiesListResponse> {
    // TODO: Add method to party service to list all parties
    // For now, return the known test parties
    let test_parties = vec!["dkfz", "tuda", "test_client", "test_server"];

    let mut parties = Vec::new();
    for party_name in test_parties {
        if let Some(party) = state.party_service.get_party(party_name).await {
            parties.push(PartyInfo {
                name: party_name.to_string(),
                hostname: party.hostname.to_string(),
                status: "active".to_string(),
            });
        }
    }

    let response = PartiesListResponse {
        count: parties.len(),
        parties,
    };

    Ok((StatusCode::OK, Json(response)))
}

pub async fn register_party(
    Path(party_name): Path<String>,
    State(_state): State<AppState>,
    Json(_request): Json<PartyRegistrationRequest>,
) -> PartyResult<PartyRegistrationResponse> {
    info!("Party registration request for: {}", party_name);

    // TODO: Implement actual party registration
    let response = PartyRegistrationResponse {
        status: "success".to_string(),
        party_name,
        message: Some("Party registration not yet implemented".to_string()),
    };

    Ok((StatusCode::OK, Json(response)))
}

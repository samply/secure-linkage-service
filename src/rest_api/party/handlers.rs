use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};

use super::responses::{PartiesListResponse, PartyInfo, PartyResult};
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
    let parties = state
        .party_service
        .list_parties()
        .await
        .into_iter()
        .map(|(name, party)| PartyInfo {
            name,
            hostname: party.hostname.to_string(),
            status: "active".to_string(),
        })
        .collect::<Vec<_>>();

    let response = PartiesListResponse {
        count: parties.len(),
        parties,
    };

    Ok((StatusCode::OK, Json(response)))
}

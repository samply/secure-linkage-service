use axum::{Router, http::StatusCode, routing::get};

use super::handlers::{get_party_info, list_parties};
use crate::rest_api::AppState;

pub fn party_routes() -> Router<AppState> {
    Router::new()
        .route("/parties", get(list_parties))
        .route("/parties/{party_name}", get(get_party_info))
        .route("/testConnection/{party_name}", get(test_connection))
}

async fn test_connection(
    axum::extract::Path(party_name): axum::extract::Path<String>,
    axum::extract::State(state): axum::extract::State<AppState>,
) -> StatusCode {
    if state.party_service.party_exists(&party_name).await {
        StatusCode::NO_CONTENT
    } else {
        StatusCode::FORBIDDEN
    }
}

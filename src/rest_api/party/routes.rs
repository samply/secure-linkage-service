use axum::{
    Router,
    routing::{get, post},
};

use super::handlers::{get_party_info, list_parties, register_party};
use crate::rest_api::AppState;

pub fn party_routes() -> Router<AppState> {
    Router::new()
        .route("/parties", get(list_parties))
        .route("/parties/{party_name}", get(get_party_info))
        .route("/parties/{party_name}/register", post(register_party))
}

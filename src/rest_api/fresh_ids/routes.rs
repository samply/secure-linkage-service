use axum::{Router, routing::post};

use super::handlers::generate_fresh_ids;
use crate::rest_api::AppState;

pub fn fresh_ids_routes() -> Router<AppState> {
    Router::new().route("/freshIDs/{party}", post(generate_fresh_ids))
}

use axum::{Router, routing::post};

use super::handlers::generate_fresh_record_ids;
use crate::rest_api::AppState;

pub fn fresh_record_ids_routes() -> Router<AppState> {
    Router::new().route("/freshRecordIDs/{party}", post(generate_fresh_record_ids))
}

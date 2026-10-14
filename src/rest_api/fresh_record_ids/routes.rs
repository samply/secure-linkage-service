use axum::{
    Router,
    routing::{get, post},
};

use super::handlers::{generate_fresh_record_ids, generate_legacy_fresh_ids};
use crate::rest_api::AppState;

pub fn fresh_record_ids_routes() -> Router<AppState> {
    Router::new()
        .route("/freshRecordIDs/{party}", post(generate_fresh_record_ids))
        .route(
            "/freshIDs/{party}",
            get(generate_legacy_fresh_ids).post(generate_legacy_fresh_ids),
        )
}

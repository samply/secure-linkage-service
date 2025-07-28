use axum::{Router, routing::get};

use super::handlers::health_check;
use crate::rest_api::AppState;

pub fn health_routes() -> Router<AppState> {
    Router::new().route("/health", get(health_check))
}

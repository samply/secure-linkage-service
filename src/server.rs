use axum::Router;
use tracing::info;

use crate::rest_api::{
    AppState, fresh_ids::fresh_ids_routes, health::health_routes, linkage::linkage_routes,
    party::party_routes,
};

fn create_router(state: AppState) -> Router {
    Router::new()
        .merge(health_routes())
        .merge(fresh_ids_routes())
        .merge(linkage_routes())
        .merge(party_routes())
        .with_state(state)
}

pub async fn start_server(address: &str) -> Result<(), Box<dyn std::error::Error>> {
    let state = AppState::new();
    let app = create_router(state);

    let listener = tokio::net::TcpListener::bind(address).await?;
    info!("Server listening on {}", address);

    axum::serve(listener, app).await?;
    Ok(())
}

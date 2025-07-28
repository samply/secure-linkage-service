use std::sync::Arc;
use tokio::time::Duration;

use crate::services::{LinkageService, PartyService};

#[derive(Debug, Clone)]
pub struct AppState {
    pub linkage_service: Arc<LinkageService>,
    pub party_service: Arc<PartyService>,
}

impl AppState {
    pub fn new() -> Self {
        Self::with_timeout(Duration::from_secs(20))
    }

    pub fn with_timeout(timeout_duration: Duration) -> Self {
        Self {
            linkage_service: Arc::new(LinkageService::with_timeout(timeout_duration)),
            party_service: Arc::new(PartyService::new()),
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

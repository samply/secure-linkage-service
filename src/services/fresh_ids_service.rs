use std::collections::HashMap;
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::RwLock;

use crate::core::compute_random_linkage_ids;
use crate::domain::*;

#[derive(Debug, Error)]
pub enum FreshIdsServiceError {
    #[error("Party not found: {0}")]
    PartyNotFound(String),
    #[error("Invalid count: {0}")]
    InvalidCount(String),
    #[error("Fresh ID generation failed: {0}")]
    GenerationFailed(#[from] LinkageError),
    #[error("Internal error: {0}")]
    InternalError(String),
}

#[derive(Debug, Clone)]
pub struct FreshIdsService {
    // Future: Could store generation statistics, rate limiting, etc.
    _internal_state: Arc<RwLock<HashMap<String, u64>>>,
}

impl FreshIdsService {
    pub fn new() -> Self {
        Self {
            _internal_state: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn generate_fresh_ids(
        &self,
        party: &LinkageParty,
        count: u32,
    ) -> Result<Vec<LinkageID>, FreshIdsServiceError> {
        // Validate count
        if count == 0 {
            return Err(FreshIdsServiceError::InvalidCount(
                "Count must be greater than 0".to_string(),
            ));
        }

        if count > 1000 {
            return Err(FreshIdsServiceError::InvalidCount(
                "Count must not exceed 1000".to_string(),
            ));
        }

        // Call the core layer to generate fresh IDs
        let fresh_ids = compute_random_linkage_ids(party, count.try_into().unwrap())
            .map_err(FreshIdsServiceError::GenerationFailed)?;

        // Future: Could add logging, metrics, caching, etc. here
        self.update_generation_stats(&party.hostname.0, count as u64)
            .await;

        Ok(fresh_ids)
    }

    pub async fn get_generation_stats(&self, party_name: &str) -> Option<u64> {
        let stats = self._internal_state.read().await;
        stats.get(party_name).copied()
    }

    async fn update_generation_stats(&self, party_name: &str, count: u64) {
        let mut stats = self._internal_state.write().await;
        let entry = stats.entry(party_name.to_string()).or_insert(0);
        *entry += count;
    }
}

impl Default for FreshIdsService {
    fn default() -> Self {
        Self::new()
    }
}

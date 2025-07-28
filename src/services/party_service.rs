use crate::core::crypto::{generate_key, generate_random_key_material};
use crate::domain::{HostName, LinkageParty};
use ring::rand::SystemRandom;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone)]
pub struct PartyService {
    parties: Arc<RwLock<HashMap<String, LinkageParty>>>,
}

impl PartyService {
    pub fn new() -> Self {
        let mut parties = HashMap::new();
        let rand = SystemRandom::new();

        // Pre-register some test parties
        let test_parties = vec!["dkfz", "tuda", "test_client", "test_server"];

        for party_name in test_parties {
            if let Ok(key_material) = generate_random_key_material(&rand) {
                if let Ok(key) = generate_key(key_material) {
                    let party = LinkageParty::new(HostName(party_name.to_string()), key);
                    parties.insert(party_name.to_string(), party);
                }
            }
        }

        Self {
            parties: Arc::new(RwLock::new(parties)),
        }
    }

    pub async fn get_party(&self, party_name: &str) -> Option<LinkageParty> {
        self.parties.read().await.get(party_name).cloned()
    }

    pub async fn register_party(&self, party_name: String, party: LinkageParty) {
        self.parties.write().await.insert(party_name, party);
    }

    pub async fn party_exists(&self, party_name: &str) -> bool {
        self.parties.read().await.contains_key(party_name)
    }
}

impl Default for PartyService {
    fn default() -> Self {
        Self::new()
    }
}

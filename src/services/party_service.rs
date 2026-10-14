use crate::core::crypto::{generate_key, generate_random_key_material};
use crate::domain::{HostName, LinkageParty};
use ring::rand::SystemRandom;
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::Arc;
use std::{env, fs};
use tokio::sync::RwLock;
use tracing::warn;

#[derive(Debug, Deserialize)]
struct PartiesConfig {
    parties: Vec<PartyConfig>,
}

#[derive(Debug, Deserialize)]
struct PartyConfig {
    name: String,
    hostname: String,
}

#[derive(Debug, Clone)]
pub struct PartyService {
    parties: Arc<RwLock<HashMap<String, LinkageParty>>>,
}

impl PartyService {
    pub fn new() -> Self {
        let mut parties = HashMap::new();
        let rand = SystemRandom::new();

        if let Ok(config_path) = env::var("PARTIES_CONFIG_FILE") {
            match fs::read_to_string(&config_path)
                .ok()
                .and_then(|content| toml::from_str::<PartiesConfig>(&content).ok())
            {
                Some(config) => {
                    for party in config.parties {
                        insert_party_with_generated_key(
                            &mut parties,
                            &rand,
                            party.name,
                            party.hostname,
                        );
                    }
                }
                None => warn!(%config_path, "could not load parties config file"),
            }
        } else {
            // Keep test parties only for local development and existing tests when no
            // explicit party configuration is provided.
            let test_parties = vec!["dkfz", "tuda", "test_client", "test_server"];

            for party_name in test_parties {
                insert_party_with_generated_key(
                    &mut parties,
                    &rand,
                    party_name.to_string(),
                    party_name.to_string(),
                );
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

    pub async fn list_parties(&self) -> Vec<(String, LinkageParty)> {
        self.parties
            .read()
            .await
            .iter()
            .map(|(name, party)| (name.clone(), party.clone()))
            .collect()
    }

    pub async fn party_exists(&self, party_name: &str) -> bool {
        self.parties.read().await.contains_key(party_name)
    }
}

fn insert_party_with_generated_key(
    parties: &mut HashMap<String, LinkageParty>,
    rand: &SystemRandom,
    name: String,
    hostname: String,
) {
    if let Ok(key_material) = generate_random_key_material(rand) {
        if let Ok(key) = generate_key(key_material) {
            parties.insert(name, LinkageParty::new(HostName(hostname), key));
        }
    }
}

impl Default for PartyService {
    fn default() -> Self {
        Self::new()
    }
}

use ring::rand::SystemRandom;
use secure_linkage_service::{
    domain::*,
    transformations::crypto::{encrypt_data_to_base64, generate_key, generate_random_key_material},
};

pub struct TestSetup {
    pub rand: SystemRandom,
    pub client_party: LinkageParty,
    pub server_party: LinkageParty,
}

impl TestSetup {
    pub fn new() -> Self {
        let rand = SystemRandom::new();
        let client_key = generate_key(generate_random_key_material(&rand).unwrap()).unwrap();
        let server_key = generate_key(generate_random_key_material(&rand).unwrap()).unwrap();

        let client_party =
            LinkageParty::new(HostName::new("client.example.com".to_string()), client_key);
        let server_party =
            LinkageParty::new(HostName::new("server.example.com".to_string()), server_key);

        Self {
            rand,
            client_party,
            server_party,
        }
    }

    pub fn create_encrypted_ids(&self, plaintexts: &[&[u8]]) -> Vec<RecordId> {
        plaintexts
            .iter()
            .map(|plaintext| {
                let encrypted =
                    encrypt_data_to_base64(&self.rand, &self.server_party.secret_key, plaintext)
                        .expect("Encryption should succeed");
                RecordId(encrypted)
            })
            .collect()
    }

    pub fn create_test_data(
        &self,
        confirmed: &[MatchBit],
        tentative: &[MatchBit],
        indices: &[u64],
    ) -> Vec<ResultData> {
        confirmed
            .iter()
            .zip(tentative.iter())
            .zip(indices.iter())
            .map(|((c, t), i)| {
                ResultData::new(
                    ConfirmedMatchBit(*c),
                    TentativeMatchBit(*t),
                    IndexOfBestMatch(*i),
                )
            })
            .collect()
    }
}

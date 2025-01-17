use base64::prelude::*;
use rand::Rng;
use ring::aead::*;
use ring::rand::{SecureRandom, SystemRandom};

#[derive(Debug)]
pub struct Party {
    pub name: String,
    secret_key: LessSafeKey,
    id_counter: usize,
    rand: SystemRandom,
}

impl Party {
    pub fn new(name: &str) -> Self {
        let mut key = [0 as u8; 32];
        let rand = SystemRandom::new();
        rand.fill(&mut key).expect("CANNOT CREATE RANDOM KEY");
        Party {
            name: name.to_string(),
            secret_key: LessSafeKey::new(
                UnboundKey::new(&CHACHA20_POLY1305, &key)
                    .expect("FAILING CANNOT CREATE SECRET KEY"),
            ),
            id_counter: 0,
            rand: rand,
        }
    }
    pub fn generate_ids(&mut self, n: usize) -> Vec<String> {
        (0..n)
            .map(|_| {
                let mut nonce_bytes = [0u8; NONCE_LEN];
                self.rand
                    .fill(&mut nonce_bytes)
                    .expect("CANNOT CREATE NONCE");
                let nonce = Nonce::assume_unique_for_key(nonce_bytes);
                let mut payload = Vec::from(self.id_counter.to_le_bytes());
                let aad = Aad::from(nonce_bytes);
                self.id_counter += 1;
                self.secret_key
                    .seal_in_place_append_tag(nonce, aad, &mut payload)
                    .expect("CANNOT ENCRYPT ID");
                // Prepend the nonce (i.e. aad)
                let mut combined: Vec<u8> = nonce_bytes.to_vec();
                combined.extend(&payload);
                BASE64_STANDARD.encode(&combined)
            })
            .collect()
    }
    pub fn verify_id(&self, id: &str) -> Result<(), ()> {
        let mut decrypted = BASE64_STANDARD.decode(id).expect("CANNOT DECODE ID BASE64");
        // Nonce is 12 bytes long
        let (nonce_bytes, payload) = decrypted.split_at_mut(12);
        let nonce_bytes: [u8; 12] = nonce_bytes
            .try_into()
            .expect("WHOA THERE PARTNER! THE NONCE HAS A WRONG SIZE!");
        let nonce = Nonce::assume_unique_for_key(nonce_bytes);
        let aad = Aad::from(nonce_bytes);
        self.secret_key
            .open_in_place(nonce, aad, payload)
            .map_err(|_| ())
            .map(|_| ())
    }
}

#[tokio::test]
async fn test_id_generation_specific_count() {
    let mut party = Party::new("Test");
    let count: u16 = rand::thread_rng().gen();
    let result = party.generate_ids(count as usize);
    assert_eq!(count as usize, result.len());
}

#[tokio::test]
async fn test_ok_id_verification() {
    let mut party = Party::new("Test");
    let count: u16 = rand::thread_rng().gen();
    let ids = party.generate_ids(count as usize);
    for id in ids {
        let result = party.verify_id(&id);
        assert_eq!(Ok(()), result);
    }
}

#[tokio::test]
async fn test_err_id_verification() {
    let mut party = Party::new("Test");
    let party2 = Party::new("Test2");
    let count: u16 = rand::thread_rng().gen();
    let ids = party.generate_ids(count as usize);
    for id in ids {
        let result = party2.verify_id(&id);
        assert_eq!(Err(()), result);
    }
}

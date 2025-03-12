use crate::domain::*;
use base64::prelude::*;
use ring::aead::*;
use ring::rand::{SecureRandom, SystemRandom};

/// Result type for compute_linkage function

/// Computes linkage between client and server data using an immutable approach
/// with secure AES-GCM encryption. Always returns a vector of results.
// 2. Process Results - Create appropriate LinkageIDs based on XOR results

/// Process all XOR results to generate LinkageIDs
/// Returns a vector of Option<LinkageID> for each result
pub fn generate_linkage_ids(
    client_rand: &SystemRandom,
    client_key: &UnboundKey,
    server_key: &UnboundKey,
    server_ids: &[RecordId],
    xor_results: &[ResultData],
) -> Vec<Option<LinkageID>> {
    if xor_results.is_empty() {
        return vec![Some(LinkageID::new(&format!(
            "ERROR: XOR Results is empty."
        )))];
    }

    // Invoking generate_linkage_id as if in:
    // `(map generate-linkage-id (range) results (repeat client-key) (repeat server-key) server-ids)`
    // with `(range)` acting as the counter sequence

    xor_results
        .iter()
        .enumerate() // adds the counter, starting from 0.
        .map(|(counter, result)| {
            Some(generate_linkage_id(
                &(counter as usize),
                client_rand,
                server_key,
                client_key,
                server_ids,
                result,
            ))
        })
        .collect()
}

fn has_a_match(rd: &ResultData) -> bool {
    rd.confirmed_match.0 != 0 || rd.tentative_match.0 != 0
}

fn generate_random_key(rand: &SystemRandom) -> [u8; 32] {
    let mut key = [0 as u8; 32];
    rand.fill(&mut key).expect("CANNOT CREATE RANDOM KEY");
    key
}

fn generate_secret_key(random_key: [u8; 32]) -> LessSafeKey {
    LessSafeKey::new(
        UnboundKey::new(&CHACHA20_POLY1305, &random_key).expect("FAILING CANNOT CREATE SECRET KEY"),
    )
}

pub fn generate_linkage_id(
    id_counter: &usize,
    client_rand: &SystemRandom,
    client_key: &UnboundKey,
    server_key: &UnboundKey,
    server_ids: &[RecordId],
    xor_result: &ResultData,
) -> LinkageID {
    if has_a_match(xor_result) {
        return create_match_linkage_id(
            id_counter,
            client_key,
            client_rand,
            server_key,
            server_ids,
            xor_result,
        );
    } else {
        return generate_random_linkage_id(
            client_rand,
            generate_secret_key(generate_random_key(client_rand)),
            id_counter,
        );
    }
}

/// Create a linkage ID for a matching record
/// Returns None if the record ID couldn't be found or processed
fn create_match_linkage_id(
    id_counter: &usize,
    client_key: &UnboundKey,
    client_rand: &SystemRandom,
    server_key: &UnboundKey,
    server_ids: &[RecordId],
    xor_result: &ResultData,
) -> LinkageID {
    // TODO: Implement this!
}

/// Generate a cryptographically secure random linkage ID
fn generate_random_linkage_id(
    client_rand: &SystemRandom,
    secret_key: LessSafeKey,
    id_counter: &usize,
) -> LinkageID {
    let mut nonce_bytes = [0u8; NONCE_LEN];
    client_rand
        .fill(&mut nonce_bytes)
        .expect("CANNOT CREATE NONCE");
    let nonce = Nonce::assume_unique_for_key(nonce_bytes);
    let mut payload = Vec::from(id_counter.to_le_bytes());
    let aad = Aad::from(nonce_bytes);
    secret_key
        .seal_in_place_append_tag(nonce, aad, &mut payload)
        .expect("CANNOT ENCRYPT ID");
    // Prepend the nonce (i.e. aad)
    let mut combined: Vec<u8> = nonce_bytes.to_vec();
    combined.extend(&payload);
    LinkageID::new(&BASE64_STANDARD.encode(&combined))
}

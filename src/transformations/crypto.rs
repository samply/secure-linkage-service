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
            .and_then(|r| r.ok())
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
) -> Result<LinkageID, ring::error::Unspecified> {
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
    client_key: UnboundKey,
    client_rand: &SystemRandom,
    server_key: UnboundKey,
    server_ids: &[RecordId],
    xor_result: &ResultData,
) -> Result<LinkageID, ring::error::Unspecified> {
    // TODO: Implement this!
}

/// Generates a random nonce for use with AEAD encryption
///
/// Returns both the raw nonce bytes and the Ring library Nonce object
fn generate_nonce(
    random_source: &SystemRandom,
) -> Result<([u8; NONCE_LEN], Nonce), ring::error::Unspecified> {
    let mut nonce_bytes = [0u8; NONCE_LEN];
    random_source.fill(&mut nonce_bytes)?;
    let nonce = Nonce::assume_unique_for_key(nonce_bytes);
    Ok((nonce_bytes, nonce))
}

/// Converts a counter value into a byte representation suitable for encryption
fn prepare_payload(counter: &usize) -> Vec<u8> {
    Vec::from(counter.to_le_bytes())
}

/// Encrypts a payload using ChaCha20-Poly1305 with the provided key, nonce, and AAD
///
/// Returns the encrypted payload with authentication tag appended
fn encrypt_payload(
    key: &LessSafeKey,
    nonce: Nonce,
    aad: Aad<[u8; NONCE_LEN]>,
    mut payload: Vec<u8>,
) -> Result<Vec<u8>, ring::error::Unspecified> {
    key.seal_in_place_append_tag(nonce, aad, &mut payload)?;
    Ok(payload)
}

fn combine_nonce_and_payload(nonce_bytes: &[u8], encrypted_payload: &[u8]) -> Vec<u8> {
    let mut combined = Vec::with_capacity(nonce_bytes.len() + encrypted_payload.len());
    combined.extend_from_slice(nonce_bytes);
    combined.extend_from_slice(encrypted_payload);
    combined
}

/// Generate a cryptographically secure random linkage ID
fn generate_random_linkage_id(
    client_rand: &SystemRandom,
    secret_key: LessSafeKey,
    id_counter: &usize,
) -> Result<LinkageID, ring::error::Unspecified> {
    let (nonce_bytes, nonce) = generate_nonce(client_rand)?;

    let payload = prepare_payload(id_counter);

    let aad = Aad::from(nonce_bytes);
    let encrypted_payload = encrypt_payload(&secret_key, nonce, aad, payload)?;

    let combined = combine_nonce_and_payload(&nonce_bytes, &encrypted_payload);

    let encoded = BASE64_STANDARD.encode(&combined);

    Ok(LinkageID::new(&encoded))
}

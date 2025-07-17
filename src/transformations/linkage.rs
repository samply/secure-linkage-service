use crate::domain::*;
use crate::transformations::crypto::*;
use base64::prelude::*;
use ring::aead::*;
use ring::rand::SystemRandom;

/// Generate a cryptographically secure random linkage ID
///
/// This function should be mapped to a public endpoint
pub fn generate_random_linkage_id(
    client_rand: &SystemRandom,
    secret_key: &LessSafeKey,
    id_counter: usize,
) -> Result<LinkageID, LinkageError> {
    let (random_nonce_bytes, nonce) = generate_random_nonce(client_rand)?;
    let payload = generate_payload_from(id_counter);
    let aad = Aad::from(random_nonce_bytes);
    let encrypted_payload = encrypt_payload(&secret_key, nonce, aad, payload)?;
    let combined = combine_nonce_and_payload(&random_nonce_bytes, &encrypted_payload);
    let encoded = BASE64_STANDARD.encode(&combined);
    Ok(LinkageID::new(&encoded).unwrap())
}

/// Result type for compute_linkage function

/// Computes linkage between client and server data using an immutable approach
/// with XChaCha20-Poly1305 encryption. Always returns a vector of results.
///
/// Creates appropriate LinkageIDs based on XOR results.

/// Process all XOR results to generate LinkageIDs.
/// Returns a vector of Option<LinkageID> for each result.
///
/// This function should be mapped to a public endpoint.
pub fn generate_linkage_ids(
    client_rand: &SystemRandom,
    client_key: &LessSafeKey,
    server_key: &LessSafeKey,
    server_ids: &[RecordId],
    xor_results: &[ResultData],
) -> Result<Vec<Option<LinkageID>>, LinkageError> {
    if xor_results.is_empty() {
        return Err(LinkageError::EmptyInput);
    }

    // Invoking generate_linkage_id as if in:
    // `(map generate-linkage-id (range) results (repeat client-key) (repeat server-key) server-ids)`
    // with `(range)` acting as the counter sequence

    xor_results
        .iter()
        .enumerate() // adds the counter, starting from 0.
        .map(|(counter, xor_result)| {
            generate_linkage_id(
                counter,
                client_rand,
                client_key,
                server_key,
                server_ids,
                xor_result,
            )
            .map(Some)
        })
        .collect()
}

pub fn generate_linkage_id(
    id_counter: usize,
    client_rand: &SystemRandom,
    client_key: &LessSafeKey,
    server_key: &LessSafeKey,
    server_ids: &[RecordId],
    xor_result: &ResultData,
) -> Result<LinkageID, LinkageError> {
    if xor_result.has_a_match() {
        create_match_linkage_id(client_rand, client_key, server_key, server_ids, xor_result)
    } else {
        let key_material = generate_random_key_material(client_rand)?;
        let random_key = generate_key(key_material)?;
        generate_random_linkage_id(client_rand, &random_key, id_counter)
    }
}

/// Create a linkage ID for a matching record
fn create_match_linkage_id(
    client_rand: &SystemRandom,
    client_key: &LessSafeKey,
    server_key: &LessSafeKey,
    server_ids: &[RecordId],
    xor_result: &ResultData,
) -> Result<LinkageID, LinkageError> {
    let best_index = xor_result.index_of_best_match.0 as usize;
    let encrypted_id_from_server = server_ids
        .get(best_index)
        .ok_or(LinkageError::ServerIDNotFoundForBestMatchIndex)?;

    let decrypted_id = decrypt_data_from_base64(
        server_key,
        &encrypted_id_from_server.0, // Directly reference the Base64EncodedString from RecordId
    )?;

    // Construct new payload with match bit prefix
    let mut new_payload = Vec::with_capacity(1 + decrypted_id.len());
    new_payload.push(xor_result.tentative_match.0 as u8);
    new_payload.extend(decrypted_id);

    let encrypted_base64_str = encrypt_data_to_base64(client_rand, client_key, &new_payload)?;

    LinkageID::new(&encrypted_base64_str.0).map_err(|_| LinkageError::InvalidLinkageId)
}

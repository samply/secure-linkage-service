use crate::domain::*;
use crate::transformations::crypto::*;
use ring::aead::*;
use ring::rand::SystemRandom;

/// Generate a cryptographically secure random linkage ID
///
pub fn generate_random_linkage_id(
    rand: &SystemRandom,
    secret_key: &LessSafeKey,
    id_counter: usize,
) -> Result<LinkageID, LinkageError> {
    let payload = generate_payload_from(id_counter);
    let encrypted_base64_str = encrypt_data_to_base64(rand, secret_key, &payload)?;
    LinkageID::new(&encrypted_base64_str.0).map_err(|_| LinkageError::InvalidLinkageId)
}

/// Process all XOR results to generate LinkageIDs.
/// Returns a vector of Option<LinkageID> for each result.
pub fn generate_computed_linkage_ids(
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
    // `(map xor-result-to-linkage-id (range) xor-results (repeat client-key) (repeat server-key) server-ids)`
    // with `(range)` acting as the counter sequence

    xor_results
        .iter()
        .enumerate() // adds the counter, starting from 0.
        .map(|(counter, xor_result)| {
            xor_result_to_linkage_id(
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

fn xor_result_to_linkage_id(
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
        .ok_or(LinkageError::ServerIDNotFoundForBestMatchIndex(best_index))?;

    let decrypted_id = decrypt_data_from_base64(server_key, &encrypted_id_from_server.0)?;

    // Construct new payload with match bit prefix
    let mut new_payload = Vec::with_capacity(1 + decrypted_id.len());
    // Convert MatchBit to u64 first, then to u8
    new_payload.push(xor_result.tentative_match.0.to_u64() as u8);
    new_payload.extend(decrypted_id);

    let encrypted_base64_str = encrypt_data_to_base64(client_rand, client_key, &new_payload)?;

    LinkageID::new(&encrypted_base64_str.0).map_err(|_| LinkageError::InvalidLinkageId)
}

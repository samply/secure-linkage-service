use crate::domain::*;
use base64::prelude::*;
use ring::aead::*;
use ring::rand::{SecureRandom, SystemRandom};

/// Result type for compute_linkage function

/// Computes linkage between client and server data using an immutable approach
/// with XChaCha20-Poly1305 encryption. Always returns a vector of results.
// 2. Process Results - Create appropriate LinkageIDs based on XOR results

/// Process all XOR results to generate LinkageIDs
/// Returns a vector of Option<LinkageID> for each result
pub fn generate_linkage_ids(
    client_rand: &SystemRandom,
    client_key: &LessSafeKey,
    server_key: &LessSafeKey,
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
        .map(|(counter, xor_result)| {
            Some(generate_linkage_id(
                &(counter as usize),
                &client_rand,
                &client_key,
                &server_key,
                &server_ids,
                &xor_result,
            ))
            .and_then(|r| r.ok())
        })
        .collect()
}

fn has_a_match(rd: &ResultData) -> bool {
    rd.confirmed_match.0 != 0 || rd.tentative_match.0 != 0
}

pub fn generate_random_key_material(rand: &SystemRandom) -> [u8; 32] {
    let mut key_material = [0 as u8; 32];
    rand.fill(&mut key_material)
        .expect("ERROR: Cannot create random key material");
    key_material
}

pub fn generate_key(key_material: [u8; 32]) -> LessSafeKey {
    LessSafeKey::new(
        UnboundKey::new(&CHACHA20_POLY1305, &key_material).expect("ERROR: Cannot create key"),
    )
}

pub fn generate_linkage_id(
    id_counter: &usize,
    client_rand: &SystemRandom,
    client_key: &LessSafeKey,
    server_key: &LessSafeKey,
    server_ids: &[RecordId],
    xor_result: &ResultData,
) -> Result<LinkageID, ring::error::Unspecified> {
    if has_a_match(xor_result) {
        create_match_linkage_id(
            &client_rand,
            &client_key,
            &server_key,
            &server_ids,
            &xor_result,
        )
    } else {
        generate_random_linkage_id(
            &client_rand,
            &generate_key(generate_random_key_material(client_rand)),
            &id_counter,
        )
    }
}
/// Generates a random nonce for use with AEAD encryption
///
/// Returns both the raw nonce bytes and the Ring library Nonce object
pub fn generate_random_nonce(
    random_source: &SystemRandom,
) -> Result<([u8; NONCE_LEN], Nonce), ring::error::Unspecified> {
    let mut nonce_bytes = [0u8; NONCE_LEN];
    random_source.fill(&mut nonce_bytes)?;
    let nonce = Nonce::assume_unique_for_key(nonce_bytes);
    Ok((nonce_bytes, nonce))
}

/// Converts a counter value into a byte representation suitable for encryption
fn generate_payload_from(counter: &usize) -> Vec<u8> {
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

/// Create a linkage ID for a matching record
/// Returns Result<LinkageID, ring::error::Unspecified>
fn create_match_linkage_id(
    client_rand: &SystemRandom,
    client_key: &LessSafeKey,
    server_key: &LessSafeKey,
    server_ids: &[RecordId],
    xor_result: &ResultData,
) -> Result<LinkageID, ring::error::Unspecified> {
    // Step 1: get the server_ID[bestIndex]
    let best_index = xor_result.index_of_best_match.0 as usize;
    let encrypted_id_from_server = match server_ids.get(best_index) {
        Some(id) => id,
        None => {
            return Ok(LinkageID::new(
                "ERROR: Server ID not found for best match index",
            ))
        }
    };

    // Step 2: Decode the base64 string to bytes
    let decoded_bytes = match BASE64_STANDARD.decode(encrypted_id_from_server.0.as_bytes()) {
        Ok(bytes) => bytes,
        Err(_) => return Ok(LinkageID::new("ERROR: Invalid base64 encoding")),
    };

    // Validate length before splitting (nonce + encrypted data)
    if decoded_bytes.len() < NONCE_LEN {
        return Ok(LinkageID::new("ERROR: Decoded ID too short"));
    }

    // Step 3: Decrypt the server ID (with the server key)

    // 3.1 Split into nonce and ciphertext
    let (nonce_bytes, ciphertext) = decoded_bytes.split_at(NONCE_LEN);

    // 3.2 Convert nonce to fixed-size array
    let nonce_bytes: [u8; NONCE_LEN] = match nonce_bytes.try_into() {
        Ok(arr) => arr,
        Err(_) => return Ok(LinkageID::new("ERROR: Invalid nonce size")),
    };

    let nonce = Nonce::assume_unique_for_key(nonce_bytes);
    let aad = Aad::from(nonce_bytes);

    // 3.3 Actually decrypt the server payload (with the server key)
    let mut payload = ciphertext.to_vec();
    let decrypted_id = match server_key.open_in_place(nonce, aad, &mut payload) {
        Ok(d) => d,
        Err(e) => {
            println!("Decryption failed: {:?}", e);
            println!("Nonce bytes: {:?}", nonce_bytes);
            println!("Ciphertext length: {}", ciphertext.len());
            return Ok(LinkageID::new(&format!(
                "ERROR: Decryption failed: {:?}",
                e
            )));
        }
    };

    // 3.4: prepend the tentative match bit to the decrypted ID
    let mut new_payload = Vec::new();
    new_payload.push(xor_result.tentative_match.0 as u8);
    new_payload.extend_from_slice(decrypted_id);

    // 3.5: actually encrypt the combined data (with the client key)
    let (second_nonce_bytes, second_nonce) = generate_random_nonce(&client_rand)?;
    let second_aad = Aad::from(second_nonce_bytes);
    let encrypted_payload = encrypt_payload(&client_key, second_nonce, second_aad, new_payload)?;

    let combined = combine_nonce_and_payload(&second_nonce_bytes, &encrypted_payload);

    // Step 4: Encode again to Base64
    let encoded = BASE64_STANDARD.encode(&combined);

    Ok(LinkageID::new(&encoded))
}

/// Generate a cryptographically secure random linkage ID
fn generate_random_linkage_id(
    client_rand: &SystemRandom,
    secret_key: &LessSafeKey,
    id_counter: &usize,
) -> Result<LinkageID, ring::error::Unspecified> {
    let (random_nonce_bytes, nonce) = generate_random_nonce(client_rand)?;

    let payload = generate_payload_from(id_counter);

    let aad = Aad::from(random_nonce_bytes);

    let encrypted_payload = encrypt_payload(&secret_key, nonce, aad, payload)?;

    let combined = combine_nonce_and_payload(&random_nonce_bytes, &encrypted_payload);

    let encoded = BASE64_STANDARD.encode(&combined);

    Ok(LinkageID::new(&encoded))
}

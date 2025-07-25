use crate::domain::*;
use base64::Engine;
use base64::prelude::BASE64_STANDARD;
use ring::aead::*;
use ring::rand::{SecureRandom, SystemRandom};
use std::sync::Arc;

pub fn encrypt_data_to_base64(
    rand: &SystemRandom,
    key: &LessSafeKey,
    plaintext: &[u8],
) -> Result<Base64EncodedString, LinkageError> {
    let (nonce_bytes, nonce) = generate_random_nonce(rand)?;
    let aad = Aad::from(nonce_bytes);
    let mut data = plaintext.to_vec();
    key.seal_in_place_append_tag(nonce, aad, &mut data)
        .map_err(|_| LinkageError::CannotEncryptPayload)?;
    let combined = combine_nonce_and_payload(&nonce_bytes, &data);
    let encoded = BASE64_STANDARD.encode(&combined);
    Ok(Base64EncodedString(Arc::from(encoded)))
}

pub fn decrypt_data_from_base64(
    key: &LessSafeKey,
    base64_str: &Base64EncodedString,
) -> Result<Vec<u8>, LinkageError> {
    let base64_str_ref: &str = base64_str.0.as_ref(); // Convert Rc<str> to &str
    let decoded_bytes = BASE64_STANDARD.decode(base64_str_ref)?; // Use the string reference
    if decoded_bytes.len() < NONCE_LEN {
        return Err(LinkageError::DecodedIdTooShort);
    }
    let (nonce_bytes, ciphertext) = decoded_bytes.split_at(NONCE_LEN);
    let nonce_bytes: [u8; NONCE_LEN] = nonce_bytes
        .try_into()
        .map_err(|_| LinkageError::InvalidNonceSize)?;
    let nonce = Nonce::assume_unique_for_key(nonce_bytes);
    let aad = Aad::from(nonce_bytes);
    let mut data = ciphertext.to_vec();
    key.open_in_place(nonce, aad, &mut data)
        .map_err(|e| LinkageError::DecryptionFailed(e.to_string()))?;
    Ok(data.to_vec())
}

pub fn generate_random_key_material(rand: &SystemRandom) -> Result<[u8; 32], LinkageError> {
    let mut key_material = [0 as u8; 32];
    rand.fill(&mut key_material)
        .map_err(|_| LinkageError::CannotCreateRandomKeyMaterial)?;
    Ok(key_material)
}

pub fn generate_key(key_material: [u8; 32]) -> Result<LessSafeKey, LinkageError> {
    let unbound_key = UnboundKey::new(&CHACHA20_POLY1305, &key_material)
        .map_err(|_| LinkageError::CannotCreateLessSafeKey)?;
    Ok(LessSafeKey::new(unbound_key))
}

/// Generates a random nonce for use with AEAD encryption
///
/// Returns both the raw nonce bytes and the Ring library Nonce object
pub fn generate_random_nonce(
    random_source: &SystemRandom,
) -> Result<([u8; NONCE_LEN], Nonce), LinkageError> {
    let mut nonce_bytes = [0u8; NONCE_LEN];
    random_source
        .fill(&mut nonce_bytes)
        .map_err(|_| LinkageError::CannotGenerateNonce)?;
    let nonce = Nonce::assume_unique_for_key(nonce_bytes);
    Ok((nonce_bytes, nonce))
}

/// Converts a counter value into a byte representation suitable for encryption
pub fn generate_payload_from(counter: usize) -> Vec<u8> {
    Vec::from(counter.to_le_bytes())
}

/// Encrypts a payload using ChaCha20-Poly1305 with the provided key, nonce, and AAD
///
/// Returns the encrypted payload with authentication tag appended
pub fn encrypt_payload(
    key: &LessSafeKey,
    nonce: Nonce,
    aad: Aad<[u8; NONCE_LEN]>,
    mut payload: Vec<u8>,
) -> Result<Vec<u8>, LinkageError> {
    key.seal_in_place_append_tag(nonce, aad, &mut payload)
        .map_err(|_| LinkageError::CannotEncryptPayload)?;
    Ok(payload)
}

pub fn combine_nonce_and_payload(nonce_bytes: &[u8], encrypted_payload: &[u8]) -> Vec<u8> {
    let mut combined = Vec::with_capacity(nonce_bytes.len() + encrypted_payload.len());
    combined.extend_from_slice(nonce_bytes);
    combined.extend_from_slice(encrypted_payload);
    combined
}

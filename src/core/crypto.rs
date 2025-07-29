use crate::core::Base64EncodedString;
use crate::core::LinkageError;
use base64::Engine;
use base64::prelude::BASE64_STANDARD;
use ring::aead::Aad;
use ring::aead::CHACHA20_POLY1305;
use ring::aead::LessSafeKey;
use ring::aead::NONCE_LEN;
use ring::aead::Nonce;
use ring::aead::UnboundKey;
use ring::rand::{SecureRandom, SystemRandom};
use std::sync::Arc;

pub fn encrypt_data_to_base64(
    rand: &SystemRandom,
    key: &LessSafeKey,
    plaintext: &[u8],
) -> Result<Base64EncodedString, LinkageError> {
    let (nonce_bytes, nonce) = generate_random_nonce(rand)?;
    let aad = Aad::from(nonce_bytes);

    // Pre-allocate with expected size to reduce reallocations
    let mut data = Vec::with_capacity(plaintext.len() + 16); // 16 bytes for auth tag
    data.extend_from_slice(plaintext);

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
    let base64_str_ref: &str = base64_str.0.as_ref();
    let decoded_bytes = BASE64_STANDARD.decode(base64_str_ref)?;

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

    let plaintext = key
        .open_in_place(nonce, aad, &mut data)
        .map_err(|e| LinkageError::DecryptionFailed(e.to_string()))?;

    Ok(plaintext.to_vec())
}

#[inline]
pub fn generate_random_key_material(rand: &SystemRandom) -> Result<[u8; 32], LinkageError> {
    let mut key_material = [0u8; 32];
    rand.fill(&mut key_material)
        .map_err(|_| LinkageError::CannotCreateRandomKeyMaterial)?;
    Ok(key_material)
}

#[inline]
pub fn generate_key(key_material: [u8; 32]) -> Result<LessSafeKey, LinkageError> {
    let unbound_key = UnboundKey::new(&CHACHA20_POLY1305, &key_material)
        .map_err(|_| LinkageError::CannotCreateLessSafeKey)?;
    Ok(LessSafeKey::new(unbound_key))
}

#[inline]
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

#[inline]
pub fn generate_payload_from(counter: usize) -> Vec<u8> {
    counter.to_le_bytes().to_vec()
}

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

#[inline]
pub fn combine_nonce_and_payload(nonce_bytes: &[u8], encrypted_payload: &[u8]) -> Vec<u8> {
    let mut combined = Vec::with_capacity(nonce_bytes.len() + encrypted_payload.len());
    combined.extend_from_slice(nonce_bytes);
    combined.extend_from_slice(encrypted_payload);
    combined
}

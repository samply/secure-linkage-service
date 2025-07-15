use base64::prelude::*;
use ring::{aead::Aad, rand::SystemRandom};
use secure_linkage_service::{
    domain::*,
    transformations::{
        compute_linkage,
        crypto::{generate_key, generate_random_key_material, generate_random_nonce},
    },
};

#[test]
fn compute_linkage_should_handle_empty_input() {
    let rand = SystemRandom::new();
    let client_data = ClientLinkageData::empty();
    let server_data = ServerLinkageData::empty();

    let server_key = generate_key(generate_random_key_material(&rand).unwrap()).unwrap();
    let client_key = generate_key(generate_random_key_material(&rand).unwrap()).unwrap();

    let result = compute_linkage(&rand, &client_key, &server_key, &client_data, &server_data);

    assert!(matches!(result, Err(LinkageError::EmptyInput)));
}

#[test]
fn compute_linkage_should_compute_linkage_id_when_match_and_no_match() {
    let rand = SystemRandom::new();

    let server_key = generate_key(generate_random_key_material(&rand).unwrap()).unwrap();
    let client_key = generate_key(generate_random_key_material(&rand).unwrap()).unwrap();

    // Create test data with the same key
    let plaintexts = [
        b"learning_rust".to_vec(),
        b"functional_programming_rules".to_vec(),
        b"medical_informatics".to_vec(),
    ];

    let encrypted_server_ids: Vec<RecordId> = plaintexts
        .iter()
        .map(|plaintext| {
            let (nonce_bytes, nonce) = generate_random_nonce(&rand).unwrap();
            let mut data = plaintext.clone();
            server_key
                .seal_in_place_append_tag(nonce, Aad::from(nonce_bytes), &mut data)
                .expect("Encryption should succeed");
            let mut combined = Vec::new();
            combined.extend_from_slice(&nonce_bytes);
            combined.extend_from_slice(&data);
            RecordId::new(&BASE64_STANDARD.encode(&combined)).unwrap()
        })
        .collect();

    // Client data - IndexOfBestMatch values clamped to the length of server_ids
    let client_data = ClientLinkageData::new(vec![
        // This will produce a match when XORed with server data
        ResultData::new(
            ConfirmedMatchBit(MatchBit::Set),
            TentativeMatchBit(MatchBit::Unset),
            IndexOfBestMatch(0),
        ),
        // This will produce a match when XORed with server data
        ResultData::new(
            ConfirmedMatchBit(MatchBit::Unset),
            TentativeMatchBit(MatchBit::Set),
            IndexOfBestMatch(1),
        ),
        // This will NOT produce a match (both client and server have 0)
        ResultData::new(
            ConfirmedMatchBit(MatchBit::Unset),
            TentativeMatchBit(MatchBit::Unset),
            IndexOfBestMatch(2),
        ),
    ]);

    // Server data
    let server_data = ServerLinkageData::new(
        vec![
            // XOR with client will produce match (1 ^ 0 = 1)
            ResultData::new(
                ConfirmedMatchBit(MatchBit::Unset),
                TentativeMatchBit(MatchBit::Set),
                IndexOfBestMatch(0),
            ),
            // XOR with client will produce match (0 ^ 1 = 1)
            ResultData::new(
                ConfirmedMatchBit(MatchBit::Set),
                TentativeMatchBit(MatchBit::Unset),
                IndexOfBestMatch(1),
            ),
            // XOR with client will produce no match (0 ^ 0 = 0)
            ResultData::new(
                ConfirmedMatchBit(MatchBit::Unset),
                TentativeMatchBit(MatchBit::Unset),
                IndexOfBestMatch(2),
            ),
        ],
        encrypted_server_ids,
    );

    let linkage_result =
        compute_linkage(&rand, &client_key, &server_key, &client_data, &server_data)
            .expect("Should succeed");

    // Verify we have exactly 3 results
    assert_eq!(linkage_result.0.len(), 3);

    // First two should be match-based IDs
    for result in &linkage_result.0[0..2] {
        match result {
            Some(id) => {
                assert!(!id.0.0.contains("ERROR"), "Unexpected error in linkage ID");
                // Match-based IDs should be longer due to containing the server ID
                assert!(id.0.0.len() > 50, "Match ID seems too short");
            }
            None => panic!("Expected Some(LinkageID)"),
        }
    }

    // Last one should be a random ID
    match &linkage_result.0[2] {
        Some(id) => {
            assert!(
                !id.0.0.contains("ERROR"),
                "Unexpected error in random linkage ID"
            );
            // Random IDs will be shorter than match-based ones
            assert!(
                id.0.0.len() > 20 && id.0.0.len() < 50,
                "Random ID length unexpected"
            );
        }
        None => panic!("Expected Some(LinkageID) for random ID case"),
    }
}

#[test]
fn compute_linkage_should_fail_with_wrong_key() {
    let rand = SystemRandom::new();

    // Generate three different keys
    let real_server_key = generate_key(generate_random_key_material(&rand).unwrap()).unwrap();
    let wrong_server_key = generate_key(generate_random_key_material(&rand).unwrap()).unwrap();
    let client_key = generate_key(generate_random_key_material(&rand).unwrap()).unwrap();

    // Server encrypts with REAL key
    let (nonce_bytes, nonce) = generate_random_nonce(&rand).unwrap();
    let mut encrypted = b"test_data".to_vec();
    real_server_key
        .seal_in_place_append_tag(nonce, Aad::from(nonce_bytes), &mut encrypted)
        .expect("Encryption should succeed");

    // Package
    let mut combined = Vec::new();
    combined.extend_from_slice(&nonce_bytes);
    combined.extend_from_slice(&encrypted);
    let server_id = RecordId::new(&BASE64_STANDARD.encode(&combined)).unwrap();
    // Test setup - client has WRONG server key
    let linkage_result = compute_linkage(
        &rand,
        &client_key,
        &wrong_server_key, // This should cause decryption failure
        &ClientLinkageData::new(vec![ResultData::new(
            ConfirmedMatchBit(MatchBit::Set),
            TentativeMatchBit(MatchBit::Unset),
            IndexOfBestMatch(0),
        )]),
        &ServerLinkageData::new(
            vec![ResultData::new(
                ConfirmedMatchBit(MatchBit::Unset),
                TentativeMatchBit(MatchBit::Set),
                IndexOfBestMatch(0),
            )],
            vec![server_id],
        ),
    );

    assert!(matches!(
        linkage_result,
        Err(LinkageError::DecryptionFailed(_))
    ));
}

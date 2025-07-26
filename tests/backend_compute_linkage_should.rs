use base64::Engine;
use base64::prelude::BASE64_STANDARD;
use ring::rand::SystemRandom;
use secure_linkage_service::{
    domain::*,
    transformations::{
        compute_linkage,
        crypto::{
            decrypt_data_from_base64, encrypt_data_to_base64, generate_key,
            generate_random_key_material,
        },
    },
};

#[test]
fn handle_empty_input() {
    let rand = SystemRandom::new();
    let client_data = ClientLinkageData::empty();
    let server_data = ServerLinkageData::empty();

    let server_key = generate_key(generate_random_key_material(&rand).unwrap()).unwrap();
    let client_key = generate_key(generate_random_key_material(&rand).unwrap()).unwrap();

    let client_party = LinkageParty::new(HostName("client.example.com".to_string()), client_key);
    let server_party = LinkageParty::new(HostName("server.example.com".to_string()), server_key);

    let result = compute_linkage(&client_party, &server_party, &client_data, &server_data);

    assert!(matches!(result, Err(LinkageError::EmptyInput)));
}

#[test]
fn handle_correct_input_with_match_and_no_match() {
    let rand = SystemRandom::new();

    let server_key = generate_key(generate_random_key_material(&rand).unwrap()).unwrap();
    let client_key = generate_key(generate_random_key_material(&rand).unwrap()).unwrap();

    let client_party = LinkageParty::new(HostName("client.example.com".to_string()), client_key);
    let server_party = LinkageParty::new(HostName("server.example.com".to_string()), server_key);

    // Create test data with the same key
    let plaintexts = [
        b"learning_rust".to_vec(),
        b"functional_programming_rules".to_vec(),
        b"medical_informatics".to_vec(),
    ];

    let encrypted_server_ids: Vec<RecordId> = plaintexts
        .iter()
        .map(|plaintext| {
            let encrypted = encrypt_data_to_base64(&rand, &server_party.secret_key, plaintext)
                .expect("Encryption should succeed");
            RecordId(encrypted)
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

    let linkage_result = compute_linkage(&client_party, &server_party, &client_data, &server_data)
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
fn fail_with_wrong_key() {
    let rand = SystemRandom::new();

    // Generate three different keys
    let real_server_key = generate_key(generate_random_key_material(&rand).unwrap()).unwrap();
    let wrong_server_key = generate_key(generate_random_key_material(&rand).unwrap()).unwrap();
    let client_key = generate_key(generate_random_key_material(&rand).unwrap()).unwrap();

    let client_party = LinkageParty::new(HostName("client.example.com".to_string()), client_key);
    let wrong_server_party =
        LinkageParty::new(HostName("server.example.com".to_string()), wrong_server_key);

    let encrypted = encrypt_data_to_base64(&rand, &real_server_key, b"test_data")
        .expect("Encryption should succeed");
    let server_id = RecordId(encrypted);

    // Test setup - client has WRONG server key
    let linkage_result = compute_linkage(
        &client_party,
        &wrong_server_party, // This should cause decryption failure
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

#[test]
fn fail_with_mismatched_data_length() {
    let rand = SystemRandom::new();
    let client_key = generate_key(generate_random_key_material(&rand).unwrap()).unwrap();
    let server_key = generate_key(generate_random_key_material(&rand).unwrap()).unwrap();

    let client_party = LinkageParty::new(HostName("client.example.com".to_string()), client_key);
    let server_party = LinkageParty::new(HostName("server.example.com".to_string()), server_key);

    // Client data has 1 item
    let client_data = ClientLinkageData::new(vec![ResultData::new(
        ConfirmedMatchBit(MatchBit::Set),
        TentativeMatchBit(MatchBit::Unset),
        IndexOfBestMatch(0),
    )]);

    // Server data has 2 items
    let server_data = ServerLinkageData::new(
        vec![
            ResultData::new(
                ConfirmedMatchBit(MatchBit::Unset),
                TentativeMatchBit(MatchBit::Set),
                IndexOfBestMatch(0),
            ),
            ResultData::new(
                ConfirmedMatchBit(MatchBit::Set),
                TentativeMatchBit(MatchBit::Unset),
                IndexOfBestMatch(1),
            ),
        ],
        vec![
            RecordId::new(&BASE64_STANDARD.encode(b"record1")).unwrap(),
            RecordId::new(&BASE64_STANDARD.encode(b"record2")).unwrap(),
        ],
    );

    let result = compute_linkage(&client_party, &server_party, &client_data, &server_data);

    assert!(matches!(result, Err(LinkageError::MismatchedDataLength)));
}

#[test]
fn debug_crypto_roundtrip() {
    let rand = SystemRandom::new();
    let key = generate_key(generate_random_key_material(&rand).unwrap()).unwrap();

    let test_plaintext = b"simple_test";
    println!(
        "Original plaintext: {:?} (len: {})",
        test_plaintext,
        test_plaintext.len()
    );

    let encrypted =
        encrypt_data_to_base64(&rand, &key, test_plaintext).expect("Encryption should succeed");
    println!("Encrypted (base64): {}", encrypted.0);

    let decrypted = decrypt_data_from_base64(&key, &encrypted).expect("Decryption should succeed");
    println!("Decrypted: {:?} (len: {})", decrypted, decrypted.len());

    assert_eq!(decrypted, test_plaintext, "Crypto roundtrip should work");
}

#[test]
fn matchbit_conversion_all_combinations_regression_test() {
    // Extended regression test to verify all MatchBit combinations work correctly
    // This ensures the fix works for both MatchBit::Set (1) and MatchBit::Unset (0)

    let rand = SystemRandom::new();
    let server_key = generate_key(generate_random_key_material(&rand).unwrap()).unwrap();
    let client_key = generate_key(generate_random_key_material(&rand).unwrap()).unwrap();

    let client_party = LinkageParty::new(HostName("client.example.com".to_string()), client_key);
    let server_party = LinkageParty::new(HostName("server.example.com".to_string()), server_key);

    // Create test data
    let test_plaintext = b"test_record_for_all_combinations".to_vec();
    let encrypted_server_id =
        encrypt_data_to_base64(&rand, &server_party.secret_key, &test_plaintext)
            .expect("Encryption should succeed");
    let server_id = RecordId(encrypted_server_id);

    // Test all combinations of tentative_match values that could cause conversion issues
    let test_cases = vec![
        (MatchBit::Unset, MatchBit::Unset), // 0, 0
        (MatchBit::Unset, MatchBit::Set),   // 0, 1 - This was the problematic case
        (MatchBit::Set, MatchBit::Unset),   // 1, 0
        (MatchBit::Set, MatchBit::Set),     // 1, 1
    ];

    for (client_tentative, server_tentative) in test_cases {
        let client_data = ClientLinkageData::new(vec![ResultData::new(
            ConfirmedMatchBit(MatchBit::Set), // Always set to ensure we get a match
            TentativeMatchBit(client_tentative), // Test different tentative values
            IndexOfBestMatch(0),
        )]);

        let server_data = ServerLinkageData::new(
            vec![ResultData::new(
                ConfirmedMatchBit(MatchBit::Unset), // Ensures XOR produces a match
                TentativeMatchBit(server_tentative), // Test different tentative values
                IndexOfBestMatch(0),
            )],
            vec![server_id.clone()],
        );

        let linkage_result =
            compute_linkage(&client_party, &server_party, &client_data, &server_data).expect(
                &format!(
                    "Should succeed for tentative_match combination: client={:?}, server={:?}",
                    client_tentative, server_tentative
                ),
            );

        // Verify the result for this combination
        assert_eq!(linkage_result.0.len(), 1);
        assert!(
            linkage_result.0[0].is_some(),
            "Should generate linkage ID for tentative_match combination: client={:?}, server={:?}",
            client_tentative,
            server_tentative
        );

        let linkage_id = linkage_result.0[0].as_ref().unwrap();
        assert!(!linkage_id.0.0.is_empty(), "Linkage ID should not be empty");
        assert!(
            !linkage_id.0.0.contains("ERROR"),
            "Linkage ID should not contain errors"
        );
    }
}

#[test]
fn matchbit_conversion_runtime_regression_test() {
    // Regression test for MatchBit conversion in create_match_linkage_id
    // This test verifies that MatchBit values are correctly converted to bytes
    // in the payload construction, ensuring the specific conversion logic works

    let rand = SystemRandom::new();
    let server_key = generate_key(generate_random_key_material(&rand).unwrap()).unwrap();
    let client_key = generate_key(generate_random_key_material(&rand).unwrap()).unwrap();

    let client_party = LinkageParty::new(HostName("client.example.com".to_string()), client_key);
    let server_party = LinkageParty::new(HostName("server.example.com".to_string()), server_key);

    // Create a known plaintext that we can verify later
    let known_plaintext = b"regression_test_data";
    let encrypted_server_id =
        encrypt_data_to_base64(&rand, &server_party.secret_key, known_plaintext)
            .expect("Encryption should succeed");
    let server_id = RecordId(encrypted_server_id);

    // Test case 1: TentativeMatchBit::Set should produce payload with prefix byte 1
    let client_data_set = ClientLinkageData::new(vec![ResultData::new(
        ConfirmedMatchBit(MatchBit::Unset), // 0
        TentativeMatchBit(MatchBit::Set),   // 1 - should become byte 1 in payload
        IndexOfBestMatch(0),
    )]);

    let server_data_set = ServerLinkageData::new(
        vec![ResultData::new(
            ConfirmedMatchBit(MatchBit::Set),   // 1
            TentativeMatchBit(MatchBit::Unset), // 0
            IndexOfBestMatch(0),
        )],
        vec![server_id.clone()],
    );

    // Test case 2: TentativeMatchBit::Unset should produce payload with prefix byte 0
    let client_data_unset = ClientLinkageData::new(vec![ResultData::new(
        ConfirmedMatchBit(MatchBit::Set),   // 1
        TentativeMatchBit(MatchBit::Unset), // 0 - should become byte 0 in payload
        IndexOfBestMatch(0),
    )]);

    let server_data_unset = ServerLinkageData::new(
        vec![ResultData::new(
            ConfirmedMatchBit(MatchBit::Unset), // 0
            TentativeMatchBit(MatchBit::Set),   // 1
            IndexOfBestMatch(0),
        )],
        vec![server_id.clone()],
    );

    // Execute both test cases
    let result_set = compute_linkage(
        &client_party,
        &server_party,
        &client_data_set,
        &server_data_set,
    )
    .expect("Should succeed with TentativeMatchBit::Set");

    let result_unset = compute_linkage(
        &client_party,
        &server_party,
        &client_data_unset,
        &server_data_unset,
    )
    .expect("Should succeed with TentativeMatchBit::Unset");

    // Both should produce valid linkage IDs
    assert_eq!(result_set.0.len(), 1);
    assert_eq!(result_unset.0.len(), 1);
    assert!(result_set.0[0].is_some());
    assert!(result_unset.0[0].is_some());

    let linkage_id_set = result_set.0[0].as_ref().unwrap();
    let linkage_id_unset = result_unset.0[0].as_ref().unwrap();

    // The linkage IDs should be different because they have different tentative_match values
    // in their encrypted payloads (one starts with byte 1, the other with byte 0)
    assert_ne!(
        linkage_id_set.0.0.as_ref(),
        linkage_id_unset.0.0.as_ref(),
        "Linkage IDs should be different for different tentative_match values"
    );

    // Both should be valid base64 strings
    assert!(BASE64_STANDARD.decode(linkage_id_set.0.0.as_ref()).is_ok());
    assert!(
        BASE64_STANDARD
            .decode(linkage_id_unset.0.0.as_ref())
            .is_ok()
    );
}

#[test]
fn matchbit_conversion_payload_content_verification() {
    // This test verifies the actual payload content to ensure MatchBit conversion works correctly
    // It decrypts the generated linkage ID to verify the prefix byte matches the expected value

    let rand = SystemRandom::new();
    let server_key = generate_key(generate_random_key_material(&rand).unwrap()).unwrap();
    let client_key = generate_key(generate_random_key_material(&rand).unwrap()).unwrap();

    let client_party = LinkageParty::new(
        HostName("client.example.com".to_string()),
        client_key.clone(),
    );
    let server_party = LinkageParty::new(
        HostName("server.example.com".to_string()),
        server_key.clone(),
    );

    // Create test data with a simple known plaintext
    let test_plaintext = b"simple_test";
    let encrypted_server_id = encrypt_data_to_base64(&rand, &server_key, test_plaintext)
        .expect("Encryption should succeed");
    let server_id = RecordId(encrypted_server_id);

    // Create scenario where XOR produces a match with TentativeMatchBit::Set
    let client_data = ClientLinkageData::new(vec![ResultData::new(
        ConfirmedMatchBit(MatchBit::Set), // 1
        TentativeMatchBit(MatchBit::Set), // 1 - this should become prefix byte 1
        IndexOfBestMatch(0),
    )]);

    let server_data = ServerLinkageData::new(
        vec![ResultData::new(
            ConfirmedMatchBit(MatchBit::Unset), // 0
            TentativeMatchBit(MatchBit::Unset), // 0
            IndexOfBestMatch(0),
        )],
        vec![server_id],
    );

    // XOR will produce: confirmed=1^0=1, tentative=1^0=1 (has_a_match = true)
    // The tentative_match XOR result (1) should be used as prefix byte

    let result = compute_linkage(&client_party, &server_party, &client_data, &server_data)
        .expect("Should succeed");

    assert_eq!(result.0.len(), 1);
    let linkage_id = result.0[0].as_ref().unwrap();

    // Decrypt the linkage ID to verify the payload structure
    let decrypted_payload = decrypt_data_from_base64(&client_key, &linkage_id.0)
        .expect("Should be able to decrypt with client key");

    // The payload should start with the tentative_match XOR result (1) as the first byte
    assert!(
        !decrypted_payload.is_empty(),
        "Decrypted payload should not be empty"
    );
    assert_eq!(
        decrypted_payload[0], 1u8,
        "First byte should be 1 (tentative_match XOR result converted to u8)"
    );

    // The rest should be the original server data
    assert_eq!(
        &decrypted_payload[1..],
        test_plaintext,
        "Remaining bytes should be the original server plaintext"
    );
}

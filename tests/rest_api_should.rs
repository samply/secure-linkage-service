mod common;

use base64::Engine;
use base64::prelude::BASE64_STANDARD;
use common::TestSetup;
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
    let setup = TestSetup::new();
    let client_data = ClientLinkageData::empty();
    let server_data = ServerLinkageData::empty();

    let result = compute_linkage(
        &setup.client_party,
        &setup.server_party,
        &client_data,
        &server_data,
    );
    assert!(matches!(result, Err(LinkageError::EmptyInput)));
}

#[test]
fn handle_correct_input_with_matches() {
    let setup = TestSetup::new();

    let plaintexts: &[&[u8]] = &[
        b"learning_rust",
        b"functional_programming",
        b"medical_informatics",
    ];
    let encrypted_server_ids = setup.create_encrypted_ids(plaintexts);

    // Client data that will produce XOR matches with server data
    let client_data = ClientLinkageData::new(setup.create_test_data(
        &[MatchBit::Set, MatchBit::Unset, MatchBit::Unset],
        &[MatchBit::Unset, MatchBit::Set, MatchBit::Unset],
        &[0, 1, 2],
    ));

    // Server data that will XOR with client to produce matches
    let server_data = ServerLinkageData::new(
        setup.create_test_data(
            &[MatchBit::Unset, MatchBit::Set, MatchBit::Unset],
            &[MatchBit::Set, MatchBit::Unset, MatchBit::Unset],
            &[0, 1, 2],
        ),
        encrypted_server_ids,
    );

    let result = compute_linkage(
        &setup.client_party,
        &setup.server_party,
        &client_data,
        &server_data,
    )
    .expect("Should succeed");

    assert_eq!(result.0.len(), 3);

    // First two should be match-based IDs (longer)
    for i in 0..2 {
        let id = result.0[i].as_ref().expect("Should have linkage ID");
        assert!(!id.0.0.contains("ERROR"));
        assert!(id.0.0.len() > 50, "Match ID should be longer");
    }

    // Last one should be random ID (shorter)
    let random_id = result.0[2].as_ref().expect("Should have random ID");
    assert!(!random_id.0.0.contains("ERROR"));
    assert!(
        random_id.0.0.len() > 20 && random_id.0.0.len() < 50,
        "Random ID should be shorter"
    );
}

#[test]
fn fail_with_wrong_decryption_key() {
    let setup = TestSetup::new();
    let wrong_key = generate_key(generate_random_key_material(&setup.rand).unwrap()).unwrap();
    let wrong_server_party =
        LinkageParty::new(HostName("server.example.com".to_string()), wrong_key);

    let encrypted =
        encrypt_data_to_base64(&setup.rand, &setup.server_party.secret_key, b"test_data")
            .expect("Encryption should succeed");
    let server_id = RecordId(encrypted);

    let client_data = ClientLinkageData::new(vec![ResultData::new(
        ConfirmedMatchBit(MatchBit::Set),
        TentativeMatchBit(MatchBit::Unset),
        IndexOfBestMatch(0),
    )]);

    let server_data = ServerLinkageData::new(
        vec![ResultData::new(
            ConfirmedMatchBit(MatchBit::Unset),
            TentativeMatchBit(MatchBit::Set),
            IndexOfBestMatch(0),
        )],
        vec![server_id],
    );

    let result = compute_linkage(
        &setup.client_party,
        &wrong_server_party,
        &client_data,
        &server_data,
    );
    assert!(matches!(result, Err(LinkageError::DecryptionFailed(_))));
}

#[test]
fn fail_with_mismatched_data_length() {
    let setup = TestSetup::new();

    let client_data = ClientLinkageData::new(vec![ResultData::new(
        ConfirmedMatchBit(MatchBit::Set),
        TentativeMatchBit(MatchBit::Unset),
        IndexOfBestMatch(0),
    )]);

    let server_data = ServerLinkageData::new(
        setup.create_test_data(
            &[MatchBit::Unset, MatchBit::Set],
            &[MatchBit::Set, MatchBit::Unset],
            &[0, 1],
        ),
        vec![
            RecordId::new(&BASE64_STANDARD.encode(b"record1")).unwrap(),
            RecordId::new(&BASE64_STANDARD.encode(b"record2")).unwrap(),
        ],
    );

    let result = compute_linkage(
        &setup.client_party,
        &setup.server_party,
        &client_data,
        &server_data,
    );
    assert!(matches!(result, Err(LinkageError::MismatchedDataLength)));
}

#[test]
fn crypto_roundtrip_verification() {
    let setup = TestSetup::new();
    let test_plaintext = b"crypto_test_data";

    let encrypted =
        encrypt_data_to_base64(&setup.rand, &setup.server_party.secret_key, test_plaintext)
            .expect("Encryption should succeed");
    let decrypted = decrypt_data_from_base64(&setup.server_party.secret_key, &encrypted)
        .expect("Decryption should succeed");

    assert_eq!(decrypted, test_plaintext);
}

#[test]
fn matchbit_combinations_regression_test() {
    let setup = TestSetup::new();
    let test_plaintext = b"matchbit_test_data";
    let encrypted_id =
        encrypt_data_to_base64(&setup.rand, &setup.server_party.secret_key, test_plaintext)
            .expect("Encryption should succeed");
    let server_id = RecordId(encrypted_id);

    let test_combinations = [
        (MatchBit::Unset, MatchBit::Unset),
        (MatchBit::Unset, MatchBit::Set),
        (MatchBit::Set, MatchBit::Unset),
        (MatchBit::Set, MatchBit::Set),
    ];

    for (client_tentative, server_tentative) in test_combinations {
        let client_data = ClientLinkageData::new(vec![ResultData::new(
            ConfirmedMatchBit(MatchBit::Set),
            TentativeMatchBit(client_tentative),
            IndexOfBestMatch(0),
        )]);

        let server_data = ServerLinkageData::new(
            vec![ResultData::new(
                ConfirmedMatchBit(MatchBit::Unset),
                TentativeMatchBit(server_tentative),
                IndexOfBestMatch(0),
            )],
            vec![server_id.clone()],
        );

        let result = compute_linkage(
            &setup.client_party,
            &setup.server_party,
            &client_data,
            &server_data,
        )
        .expect(&format!(
            "Should succeed for {:?}, {:?}",
            client_tentative, server_tentative
        ));

        assert_eq!(result.0.len(), 1);
        let linkage_id = result.0[0].as_ref().expect("Should have linkage ID");
        assert!(!linkage_id.0.0.is_empty());
        assert!(!linkage_id.0.0.contains("ERROR"));
    }
}

#[test]
fn payload_prefix_verification() {
    let setup = TestSetup::new();
    let test_plaintext = b"prefix_test";
    let encrypted_id =
        encrypt_data_to_base64(&setup.rand, &setup.server_party.secret_key, test_plaintext)
            .expect("Encryption should succeed");
    let server_id = RecordId(encrypted_id);

    // Test case where XOR produces match with tentative_match = 1
    let client_data = ClientLinkageData::new(vec![ResultData::new(
        ConfirmedMatchBit(MatchBit::Set),
        TentativeMatchBit(MatchBit::Set),
        IndexOfBestMatch(0),
    )]);

    let server_data = ServerLinkageData::new(
        vec![ResultData::new(
            ConfirmedMatchBit(MatchBit::Unset),
            TentativeMatchBit(MatchBit::Unset),
            IndexOfBestMatch(0),
        )],
        vec![server_id],
    );

    let result = compute_linkage(
        &setup.client_party,
        &setup.server_party,
        &client_data,
        &server_data,
    )
    .expect("Should succeed");

    let linkage_id = result.0[0].as_ref().expect("Should have linkage ID");
    let decrypted_payload = decrypt_data_from_base64(&setup.client_party.secret_key, &linkage_id.0)
        .expect("Should decrypt with client key");

    assert!(!decrypted_payload.is_empty());
    assert_eq!(
        decrypted_payload[0], 1u8,
        "First byte should be XOR result of tentative_match (1)"
    );
    assert_eq!(
        &decrypted_payload[1..],
        test_plaintext,
        "Rest should be original server data"
    );
}

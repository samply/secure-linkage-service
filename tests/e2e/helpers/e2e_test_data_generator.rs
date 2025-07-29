// This tool generates properly encrypted test data for E2E tests
// Place this in tests/helpers/ or as a separate binary

use base64::Engine;
use base64::prelude::BASE64_STANDARD;
use ring::rand::SystemRandom;
use secure_linkage_service::{domain::*, transformations::crypto::*};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create the same setup as the integration tests
    let rand = SystemRandom::new();

    // Generate keys for test parties (matching what's in rest_api.rs)
    let server_key_material = generate_random_key_material(&rand)?;
    let server_key = generate_key(server_key_material)?;
    let server_party = LinkageParty::new(HostName("tuda".to_string()), server_key);

    let client_key_material = generate_random_key_material(&rand)?;
    let client_key = generate_key(client_key_material)?;
    let client_party = LinkageParty::new(HostName("dkfz".to_string()), client_key);

    // Test data similar to integration tests
    let plaintexts = [
        b"learning_rust",
        b"functional_programming",
        b"medical_informatics",
        b"secure_linkage",
        b"cryptographic_protocol",
    ];

    println!("=== E2E Test Data Generator ===");
    println!();

    // Generate encrypted server IDs
    println!("Encrypted Server IDs:");
    let mut encrypted_ids = Vec::new();
    for (i, plaintext) in plaintexts.iter().enumerate() {
        let encrypted = encrypt_data_to_base64(&rand, &server_party.secret_key, plaintext)?;
        encrypted_ids.push(encrypted.0.to_string());
        println!("  {}: \"{}\"", i, encrypted_ids[i]);
    }

    println!();
    println!("JSON format for shell script:");
    println!("ids_data='[");
    for (i, id) in encrypted_ids.iter().enumerate() {
        let comma = if i == encrypted_ids.len() - 1 {
            ""
        } else {
            ","
        };
        println!("    \"{}\"{}", id, comma);
    }
    println!("]'");

    println!();
    println!("=== Test Case: Multiple Matches (matching integration test pattern) ===");

    // Create test data that will produce known XOR results
    // Based on the pattern from handle_correct_input_with_matches test

    // Server data configuration
    let server_confirmed = [
        MatchBit::Unset,
        MatchBit::Set,
        MatchBit::Unset,
        MatchBit::Set,
        MatchBit::Unset,
    ];
    let server_tentative = [
        MatchBit::Set,
        MatchBit::Unset,
        MatchBit::Unset,
        MatchBit::Set,
        MatchBit::Set,
    ];
    let server_indices = [0, 1, 2, 3, 4];

    // Client data configuration (designed to XOR with server data for specific matches)
    let client_confirmed = [
        MatchBit::Set,
        MatchBit::Unset,
        MatchBit::Unset,
        MatchBit::Set,
        MatchBit::Unset,
    ];
    let client_tentative = [
        MatchBit::Unset,
        MatchBit::Set,
        MatchBit::Unset,
        MatchBit::Unset,
        MatchBit::Set,
    ];
    let client_indices = [0, 1, 2, 3, 4];

    println!("Server result data (for shell script):");
    println!("server_result_data='[");
    for i in 0..5 {
        let comma = if i == 4 { "" } else { "," };
        println!(
            "    {{\"match\": {}, \"tentativeMatch\": {}, \"bestIndex\": {}}}{}",
            server_confirmed[i].to_u64(),
            server_tentative[i].to_u64(),
            server_indices[i],
            comma
        );
    }
    println!("]'");

    println!();
    println!("Client result data (for shell script):");
    println!("client_result_data='[");
    for i in 0..5 {
        let comma = if i == 4 { "" } else { "," };
        println!(
            "    {{\"match\": {}, \"tentativeMatch\": {}, \"bestIndex\": {}}}{}",
            client_confirmed[i].to_u64(),
            client_tentative[i].to_u64(),
            client_indices[i],
            comma
        );
    }
    println!("]'");

    println!();
    println!("Expected XOR results:");
    for i in 0..5 {
        let xor_confirmed = client_confirmed[i].xor(server_confirmed[i]);
        let xor_tentative = client_tentative[i].xor(server_tentative[i]);
        let has_match = xor_confirmed == MatchBit::Set || xor_tentative == MatchBit::Set;

        println!(
            "  Record {}: confirmed_xor={:?}, tentative_xor={:?}, has_match={}",
            i, xor_confirmed, xor_tentative, has_match
        );
    }

    // Test the actual computation to verify it works
    println!();
    println!("=== Verification Test ===");

    let server_data = create_test_data(&server_confirmed, &server_tentative, &server_indices);
    let client_data = create_test_data(&client_confirmed, &client_tentative, &client_indices);

    let server_linkage_data = ServerLinkageData::new(
        server_data,
        encrypted_ids
            .iter()
            .map(|id| RecordId::new(id).unwrap())
            .collect(),
    );
    let client_linkage_data = ClientLinkageData::new(client_data);

    // Test the computation
    match secure_linkage_service::compute_linkage(
        &client_party,
        &server_party,
        &client_linkage_data,
        &server_linkage_data,
    ) {
        Ok(result) => {
            println!(
                "✅ Computation successful! Generated {} linkage IDs",
                result.0.len()
            );
            for (i, id_opt) in result.0.iter().enumerate() {
                match id_opt {
                    Some(id) => println!("  ID {}: {} chars", i, id.0.0.len()),
                    None => println!("  ID {}: None", i),
                }
            }
        }
        Err(e) => {
            println!("❌ Computation failed: {}", e);
        }
    }

    Ok(())
}

fn create_test_data(
    confirmed_bits: &[MatchBit],
    tentative_bits: &[MatchBit],
    indices: &[usize],
) -> Vec<ResultData> {
    confirmed_bits
        .iter()
        .zip(tentative_bits.iter())
        .zip(indices.iter())
        .map(|((confirmed, tentative), index)| {
            ResultData::new(
                ConfirmedMatchBit(*confirmed),
                TentativeMatchBit(*tentative),
                IndexOfBestMatch(*index as u64),
            )
        })
        .collect()
}

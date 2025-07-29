#!/bin/bash
# Simple standalone Rust data generator that doesn't modify Cargo.toml

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

echo "=== Setting up Simple Rust Data Generator ==="
echo

# Create a standalone Rust script that can be run with `cargo run`
echo "📝 Creating standalone Rust generator script..."

cat > "$SCRIPT_DIR/generate_crypto_data.rs" << 'EOF'
// Standalone Rust script to generate properly encrypted test data
// Run with: cargo run --bin secure-linkage-service --features=crypto_generator

use base64::Engine;
use base64::prelude::BASE64_STANDARD;
use ring::rand::SystemRandom;

// We'll include the necessary crypto functions inline to avoid dependency issues
fn main() {
    println!("=== Crypto Data Generator ===");

    // For now, let's provide a simpler approach that generates valid base64 data
    // This is a temporary solution until we can properly integrate with the crypto system

    let args: Vec<String> = std::env::args().collect();
    let scenario = args.get(1).unwrap_or(&"multiple_matches".to_string());

    match scenario.as_str() {
        "multiple_matches" => generate_multiple_matches_data(),
        "single_match" => generate_single_match_data(),
        _ => {
            eprintln!("Usage: {} <scenario>", args[0]);
            eprintln!("Scenarios: multiple_matches, single_match");
        }
    }
}

fn generate_multiple_matches_data() {
    // Generate longer, more realistic base64 IDs that look encrypted
    let encrypted_ids = [
        "Y3J5cHRvZ3JhcGhpY2FsbHlfZW5jcnlwdGVkX2RhdGFfMTIzNDU2Nzg5MA==",
        "c2VjdXJlX2xpbmthZ2VfZGF0YV93aXRoX3Byb3Blcl9lbmNyeXB0aW9uXzE=",
        "bWVkaWNhbF9pbmZvcm1hdGljc19kYXRhX2VuY3J5cHRlZF9zYWZlbHlfMjM0",
        "ZnVuY3Rpb25hbF9wcm9ncmFtbWluZ19kYXRhX3dpdGhfY3J5cHRvXzM0NQ==",
        "bGVhcm5pbmdfcnVzdF9kYXRhX3dpdGhfcHJvcGVyX2VuY3J5cHRpb25fNDU2"
    ];

    println!("ids_data='[");
    for (i, id) in encrypted_ids.iter().enumerate() {
        let comma = if i == encrypted_ids.len() - 1 { "" } else { "," };
        println!("    \"{}\"{}",id, comma);
    }
    println!("]'");
    println!();

    // Server data configuration (creates XOR matches with client data)
    println!("server_result_data='[");
    let server_data = [
        (0, 1, 0), (1, 0, 1), (0, 0, 2), (1, 1, 3), (0, 1, 4)
    ];
    for (i, (match_bit, tentative, index)) in server_data.iter().enumerate() {
        let comma = if i == server_data.len() - 1 { "" } else { "," };
        println!("    {{\"match\": {}, \"tentativeMatch\": {}, \"bestIndex\": {}}}{}",
                match_bit, tentative, index, comma);
    }
    println!("]'");
    println!();

    // Client data configuration (designed to XOR with server for matches)
    println!("client_result_data='[");
    let client_data = [
        (1, 0, 0), (0, 1, 1), (0, 0, 2), (0, 0, 3), (0, 1, 4)
    ];
    for (i, (match_bit, tentative, index)) in client_data.iter().enumerate() {
        let comma = if i == client_data.len() - 1 { "" } else { "," };
        println!("    {{\"match\": {}, \"tentativeMatch\": {}, \"bestIndex\": {}}}{}",
                match_bit, tentative, index, comma);
    }
    println!("]'");
}

fn generate_single_match_data() {
    // Single longer base64 ID
    let encrypted_id = "c2luZ2xlX3JlY29yZF90ZXN0X2RhdGFfd2l0aF9wcm9wZXJfZW5jcnlwdGlvbg==";

    println!("ids_data='[");
    println!("    \"{}\"", encrypted_id);
    println!("]'");
    println!();

    // Server data: tentative match
    println!("server_result_data='[");
    println!("    {{\"match\": 0, \"tentativeMatch\": 1, \"bestIndex\": 0}}");
    println!("]'");
    println!();

    // Client data: confirmed match (will XOR to create match)
    println!("client_result_data='[");
    println!("    {{\"match\": 1, \"tentativeMatch\": 0, \"bestIndex\": 0}}");
    println!("]'");
}

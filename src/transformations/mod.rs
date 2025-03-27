pub mod crypto;
use crate::domain::*;
use crate::transformations::crypto::generate_linkage_ids;
use ring::aead::LessSafeKey;
use ring::rand::SystemRandom;

/// Computes linkage between client and server data using an immutable approach with references
pub fn compute_linkage(
    client_rand: &SystemRandom,
    client_key: &LessSafeKey,
    server_key: &LessSafeKey,
    client_data: &ClientLinkageData,
    server_data: &ServerLinkageData,
) -> LinkageResult {
    // Check if there's any data to work with
    if client_data.share_data.is_empty() || server_data.share_data.is_empty() {
        return LinkageResult(vec![Some(LinkageID::new(
            "ERROR: No data available for linkage.",
        ))]);
    }

    let xor_results = compute_xor_results(&client_data.share_data, &server_data.share_data);

    let server_ids = server_data.ids();

    let linkage_ids = generate_linkage_ids(
        &client_rand,
        &client_key,
        &server_key,
        &server_ids,
        &xor_results,
    );

    LinkageResult(linkage_ids)
}

fn compute_xor_results(data1: &[ResultData], data2: &[ResultData]) -> Vec<ResultData> {
    data1
        .iter()
        .zip(data2.iter())
        .map(|(d1, d2)| {
            // Perform the XOR operations directly on the u64 values
            ResultData {
                confirmed_match: ConfirmedMatchBit(d1.confirmed_match.0 ^ d2.confirmed_match.0),
                tentative_match: TentativeMatchBit(d1.tentative_match.0 ^ d2.tentative_match.0),
                index_of_best_match: IndexOfBestMatch(
                    d1.index_of_best_match.0 ^ d2.index_of_best_match.0,
                ),
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use ring::aead::CHACHA20_POLY1305;
    use ring::aead::UnboundKey;

    #[test]
    fn compute_linkage_should_handle_empty_data() {
        let rand = SystemRandom::new();
        let client_data = ClientLinkageData::empty();
        let server_data = ServerLinkageData::empty();

        let server_key_material: Vec<u8> = vec![
            0x42, 0x39, 0x8f, 0xd5, 0xe1, 0xae, 0x41, 0x7c, 0x6a, 0x9a, 0xbb, 0x8e, 0x33, 0x94,
            0x9b, 0xc7, 0xe0, 0x73, 0x70, 0x7c, 0xae, 0x21, 0x0d, 0x30, 0xee, 0xea, 0x87, 0xe3,
            0x09, 0x7f, 0x1d, 0x08,
        ];
        let client_key_material: Vec<u8> = vec![
            0xd9, 0xee, 0x72, 0x53, 0x10, 0xe9, 0x83, 0x56, 0x1b, 0x04, 0x47, 0xbc, 0x0f, 0x5c,
            0xff, 0xc5, 0x71, 0x61, 0xc3, 0x3f, 0x3a, 0x64, 0x05, 0x15, 0x68, 0xe2, 0x4f, 0xc3,
            0xfc, 0x8a, 0x5d, 0x18,
        ];
        let unbound_client_key =
            UnboundKey::new(&CHACHA20_POLY1305, &client_key_material.as_slice()).unwrap();
        let client_key = LessSafeKey::new(unbound_client_key);
        let unbound_server_key =
            UnboundKey::new(&CHACHA20_POLY1305, server_key_material.as_slice()).unwrap();
        let server_key = LessSafeKey::new(unbound_server_key);

        // Call the function with references
        let linkage_id =
            compute_linkage(&rand, &client_key, &server_key, &client_data, &server_data);

        // Verify result for empty data
        assert_eq!(
            linkage_id,
            LinkageResult(vec![Some(LinkageID::new(
                "ERROR: No data available for linkage."
            ))])
        );
    }

    #[test]
    fn compute_linkage_should_generate_linkage_id_from_matching_data() {
        let rand = SystemRandom::new();
        // Exactly the same client data from the E2E test from the Python implementation
        let client_data = ClientLinkageData::new(vec![
            ResultData::new(
                ConfirmedMatchBit(1), // match
                TentativeMatchBit(0), // tentativeMatch
                IndexOfBestMatch(2),  // bestIndex
            ),
            ResultData::new(
                ConfirmedMatchBit(0), // match
                TentativeMatchBit(1), // tentativeMatch
                IndexOfBestMatch(0),  // bestIndex
            ),
            ResultData::new(
                ConfirmedMatchBit(1), // match
                TentativeMatchBit(1), // tentativeMatch
                IndexOfBestMatch(4),  // bestIndex
            ),
        ]);

        // (Almost) the same server data from the E2E test from the Python implementation
        let server_data = ServerLinkageData::new(
            vec![
                ResultData::new(
                    ConfirmedMatchBit(0), // match
                    TentativeMatchBit(1), // tentativeMatch
                    IndexOfBestMatch(2),  // bestIndex
                ),
                ResultData::new(
                    ConfirmedMatchBit(1), // match
                    TentativeMatchBit(0), // tentativeMatch
                    IndexOfBestMatch(0),  // bestIndex
                ),
                ResultData::new(
                    ConfirmedMatchBit(0), // match
                    TentativeMatchBit(0), // tentativeMatch
                    IndexOfBestMatch(4),  // bestIndex
                ),
            ],
            vec![
                RecordId::new("bGVhcm5pbmdfcnVzdAo="), // echo "learning_rust" | base64
                RecordId::new("ZnVuY3Rpb25hbF9wcm9ncmFtbWluZ19ydWxlcwo="), // echo functional_programming_rules |base64
                RecordId::new("bWVkaWNhbF9pbmZvcm1hdGljcwo="), // echo medical_informatics |base64
            ],
        );

        // Setup encryption keys
        let server_key_material: Vec<u8> = vec![
            0x42, 0x39, 0x8f, 0xd5, 0xe1, 0xae, 0x41, 0x7c, 0x6a, 0x9a, 0xbb, 0x8e, 0x33, 0x94,
            0x9b, 0xc7, 0xe0, 0x73, 0x70, 0x7c, 0xae, 0x21, 0x0d, 0x30, 0xee, 0xea, 0x87, 0xe3,
            0x09, 0x7f, 0x1d, 0x08,
        ]; //sha265("server_key")
        let client_key_material: Vec<u8> = vec![
            0xd9, 0xee, 0x72, 0x53, 0x10, 0xe9, 0x83, 0x56, 0x1b, 0x04, 0x47, 0xbc, 0x0f, 0x5c,
            0xff, 0xc5, 0x71, 0x61, 0xc3, 0x3f, 0x3a, 0x64, 0x05, 0x15, 0x68, 0xe2, 0x4f, 0xc3,
            0xfc, 0x8a, 0x5d, 0x18,
        ]; //sha265("client_key")
        let client_key = LessSafeKey::new(
            UnboundKey::new(&CHACHA20_POLY1305, &client_key_material.as_slice()).unwrap(),
        );
        let server_key = LessSafeKey::new(
            UnboundKey::new(&CHACHA20_POLY1305, server_key_material.as_slice()).unwrap(),
        );

        // Call the function
        let linkage_id =
            compute_linkage(&rand, &client_key, &server_key, &client_data, &server_data);

        // This assertion will fail, since we haven't implemented the proper logic yet.
        // The expected value should represent what a correct implementation would return
        // based on the XOR operations between client and server data.
        assert_eq!(
            linkage_id,
            LinkageResult(vec![Some(
                LinkageID::new("match-1-1-0"), // Expected format: "match-{confirmed}-{tentative}-{index}"
            )])
        );
    }
}

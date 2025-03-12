pub mod crypto;
use crate::domain::*;
use crate::transformations::crypto::generate_linkage_ids;
use ring::aead::UnboundKey;
use ring::rand::SystemRandom;

/// Computes linkage between client and server data using an immutable approach with references
pub fn compute_linkage(
    client_rand: &SystemRandom,
    client_key: &UnboundKey,
    server_key: &UnboundKey,
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

    #[test]
    fn compute_linkage_should_handle_empty_data() {
        // Create empty test data
        let client_data = ClientLinkageData::empty();
        let server_data = ServerLinkageData::empty();
        let client_key = UnboundKey::new("client-key");
        let server_key = UnboundKey::new("server-key");

        // Call the function with references
        let linkage_id = compute_linkage(&client_data, &server_data, &client_key, &server_key);

        // Verify result for empty data
        assert_eq!(
            linkage_id,
            LinkageResult(vec![Some(LinkageID::from(
                "ERROR: No data available for linkage."
            ))])
        );
    }

    #[test]
    fn compute_linkage_should_generate_linkage_id_from_matching_data() {
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
        let client_key = UnboundKey::new("client-secret-key");
        let server_key = UnboundKey::new("server-secret-key");

        // Call the function
        let linkage_id = compute_linkage(&client_data, &server_data, &client_key, &server_key);

        // This assertion will fail, since we haven't implemented the proper logic yet.
        // The expected value should represent what a correct implementation would return
        // based on the XOR operations between client and server data.
        assert_eq!(
            linkage_id,
            LinkageResult(vec![Some(
                LinkageID::from("match-1-1-0"), // Expected format: "match-{confirmed}-{tentative}-{index}"
            )])
        );
    }
}

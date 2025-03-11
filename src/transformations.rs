use crate::domain::*;

/// Computes linkage between client and server data using an immutable approach with references
pub fn compute_linkage(
    client_data: &ClientLinkageData,
    server_data: &ServerLinkageData,
    // client_encryption_key: &EncryptionKey,
    // server_encryption_key: &EncryptionKey,
    // encryption_strategy: &str,
) -> LinkageID {
    // Check if there's any data to work with
    if client_data.share_data.is_empty() || server_data.share_data.is_empty() {
        return LinkageID::from("ERROR: No data available for linkage.");
    }

    // Compute XOR between client and server data
    let xor_results: Vec<ResultData> = client_data
        .share_data
        .iter()
        .zip(server_data.share_data.iter())
        .map(|(client, server)| {
            // Perform the XOR operations directly on the u64 values
            ResultData {
                confirmed_match: ConfirmedMatchBit(
                    client.confirmed_match.0 ^ server.confirmed_match.0,
                ),
                tentative_match: TentativeMatchBit(
                    client.tentative_match.0 ^ server.tentative_match.0,
                ),
                index_of_best_match: IndexOfBestMatch(
                    client.index_of_best_match.0 ^ server.index_of_best_match.0,
                ),
            }
        })
        .collect();

    // Determine linkage ID from XOR results
    let linkage_id = xor_results
        .iter()
        .enumerate()
        .find_map(|(_i, result)| {
            // Check if there's a match
            if result.confirmed_match.0 != 0 || result.tentative_match.0 != 0 {
                // With a match, use the values to build a linkage ID
                Some(LinkageID::from(&format!(
                    "match-{}-{}-{}",
                    result.confirmed_match.0,
                    result.tentative_match.0,
                    result.index_of_best_match.0
                )))
            } else {
                None
            }
        })
        .unwrap_or_else(|| {
            LinkageID::from("ERROR: This is not the LinkageID you are looking for.")
        });

    linkage_id
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compute_linkage_should_handle_empty_data() {
        // Create empty test data
        let client_data = ClientLinkageData::empty();
        let server_data = ServerLinkageData::empty();
        let client_key = EncryptionKey::new("client-key");
        let server_key = EncryptionKey::new("server-key");
        let encryption_strategy = "aes256";

        // Call the function with references
        let linkage_id = compute_linkage(
            &client_data,
            &server_data,
            //&client_key,
            //&server_key,
            //encryption_strategy, // &str works directly
        );

        // Verify result for empty data
        assert_eq!(
            linkage_id,
            LinkageID::from("ERROR: No data available for linkage.")
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
        let client_key = EncryptionKey::new("client-secret-key");
        let server_key = EncryptionKey::new("server-secret-key");
        let encryption_strategy = "aes256";

        // Call the function
        let linkage_id = compute_linkage(
            &client_data,
            &server_data,
            // &client_key,
            // &server_key,
            // encryption_strategy,
        );

        // This assertion will fail, since we haven't implemented the proper logic yet.
        // The expected value should represent what a correct implementation would return
        // based on the XOR operations between client and server data.
        assert_eq!(
            linkage_id,
            LinkageID::from("match-1-1-0"), // Expected format: "match-{confirmed}-{tentative}-{index}"
        );
    }
}

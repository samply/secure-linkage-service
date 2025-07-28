pub mod crypto;
pub mod linkage;
use crate::core::linkage::{generate_computed_linkage_ids, generate_random_linkage_id};
use crate::domain::*;
use ring::rand::SystemRandom;

/// Compute N random linkage IDs
///
/// This function should be linked to an endpoint
pub fn compute_random_linkage_ids(
    party: &LinkageParty,
    n: usize,
) -> Result<Vec<LinkageID>, LinkageError> {
    let rand = SystemRandom::new();

    // Implementing the Clojure logic:
    // (map generate_random_linkage_id (repeat rand) (repeat party.secret_key) (range n))
    // with (range n) acting as the counter sequence from 0 to n-1
    (0..n)
        .map(|id_counter| generate_random_linkage_id(&rand, &party.secret_key, id_counter))
        .collect()
}

/// Compute linkage between client and server data
/// Returns a LinkageResult containing Option<LinkageID> for each record
///
/// This function should be linked to an endpoint
pub fn compute_linkage(
    client_party: &LinkageParty,
    server_party: &LinkageParty,
    client_data: &ClientLinkageData,
    server_data: &ServerLinkageData,
) -> Result<LinkageResult, LinkageError> {
    // Validate input data
    if client_data.share_data.is_empty() || server_data.share_data.is_empty() {
        return Err(LinkageError::EmptyInput);
    }
    if client_data.share_data.len() != server_data.share_data.len() {
        return Err(LinkageError::MismatchedDataLength);
    }

    let xor_results = xor_data_shares(&client_data.share_data, &server_data.share_data);

    let rand = SystemRandom::new();

    // Generate linkage IDs based on XOR results
    let linkage_ids = generate_computed_linkage_ids(
        &rand,
        &client_party.secret_key,
        &server_party.secret_key,
        &server_data.ids(),
        &xor_results,
    )?;

    Ok(LinkageResult(linkage_ids))
}

fn xor_data_shares(data_share_1: &[ResultData], data_share_2: &[ResultData]) -> Vec<ResultData> {
    data_share_1
        .iter()
        .zip(data_share_2.iter())
        .map(|(d1, d2)| ResultData {
            confirmed_match: ConfirmedMatchBit(d1.confirmed_match.0.xor(d2.confirmed_match.0)),
            tentative_match: TentativeMatchBit(d1.tentative_match.0.xor(d2.tentative_match.0)),
            index_of_best_match: IndexOfBestMatch(
                // Clamp index to valid range
                d1.index_of_best_match.0.min(usize::MAX as u64)
                    ^ d2.index_of_best_match.0.min(usize::MAX as u64),
            ),
        })
        .collect()
}

pub mod crypto;
pub mod linkage;
use crate::domain::*;
use crate::transformations::linkage::generate_linkage_ids;
use ring::aead::LessSafeKey;
use ring::rand::SystemRandom;

/// Computes linkage between client and server data using an immutable approach with references
pub fn compute_linkage(
    client_rand: &SystemRandom,
    client_key: &LessSafeKey,
    server_key: &LessSafeKey,
    client_data: &ClientLinkageData,
    server_data: &ServerLinkageData,
) -> Result<LinkageResult, LinkageError> {
    // Check if there's any data to work with
    if client_data.share_data.is_empty() || server_data.share_data.is_empty() {
        return Err(LinkageError::EmptyInput);
    }

    let xor_results = compute_xor_results(&client_data.share_data, &server_data.share_data);

    let server_ids = server_data.ids();

    let linkage_ids = generate_linkage_ids(
        &client_rand,
        &client_key,
        &server_key,
        &server_ids,
        &xor_results,
    )?;

    Ok(LinkageResult(linkage_ids))
}

fn compute_xor_results(data1: &[ResultData], data2: &[ResultData]) -> Vec<ResultData> {
    data1
        .iter()
        .zip(data2.iter())
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

use crate::domain::*;

pub fn compute_linkage(
    client_data: ClientLinkageData,
    server_data: ServerLinkageData,
    client_encryption_key: EncryptionKey,
    server_encryption_key: EncryptionKey,
    encryption_strategy: String,
) -> LinkageID {
    LinkageID::from("This is not the LinkageID you're looking for.")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compute_linkage_should() {
        // Create test data
        let client_data = ClientLinkageData::empty();
        let server_data = ServerLinkageData::empty();
        let client_key = EncryptionKey::new("client-key");
        let server_key = EncryptionKey::new("server-key");

        let encryption_strategy = "aes256";

        // Call the function
        let linkage_id = compute_linkage(
            client_data,
            server_data,
            client_key,
            server_key,
            encryption_strategy.to_string(),
        );

        // Verify result
        assert_eq!(
            linkage_id,
            LinkageID::from("This is certainly not the LinkageID *I* am looking for!")
        );
    }
}

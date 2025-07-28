use ring::rand::SystemRandom;
use secure_linkage_service::core::crypto::{generate_key, generate_random_key_material};
use secure_linkage_service::domain::{HostName, LinkageParty};
use secure_linkage_service::services::PartyService;

#[tokio::test]
async fn party_service_should_register_and_retrieve_parties() {
    let party_service = PartyService::new();

    // Create a test party
    let rand = SystemRandom::new();
    let key = generate_key(generate_random_key_material(&rand).unwrap()).unwrap();
    let party = LinkageParty::new(HostName("test_party".to_string()), key);

    // Register the party
    party_service
        .register_party("test_party".to_string(), party.clone())
        .await;

    // Retrieve the party
    let retrieved_party = party_service.get_party("test_party").await;
    assert!(retrieved_party.is_some());
    assert_eq!(retrieved_party.unwrap().hostname, party.hostname);

    // Check if party exists
    assert!(party_service.party_exists("test_party").await);
    assert!(!party_service.party_exists("nonexistent_party").await);
}

#[tokio::test]
async fn party_service_should_have_default_test_parties() {
    let party_service = PartyService::new();

    let test_parties = ["dkfz", "tuda", "test_client", "test_server"];

    for party_name in test_parties {
        assert!(party_service.party_exists(party_name).await);
        let party = party_service.get_party(party_name).await;
        assert!(party.is_some());
        assert_eq!(party.unwrap().hostname.0, party_name);
    }
}

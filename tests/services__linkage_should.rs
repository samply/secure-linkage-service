use ring::rand::SystemRandom;
use secure_linkage_service::core::crypto::{generate_key, generate_random_key_material};
use secure_linkage_service::{
    domain::{HostName, LinkageParty},
    services::{
        LinkageService,
        linkage_service::{
            ClientLinkageRequest, LinkageRequest, LinkageRequestData, LinkageServiceError,
        },
    },
};
use tokio::time::Duration;
#[tokio::test]
async fn linkage_service_should_handle_timeout() {
    let linkage_service = LinkageService::with_timeout(Duration::from_millis(100));

    let rand = SystemRandom::new();
    let key1 = generate_key(generate_random_key_material(&rand).unwrap()).unwrap();
    let key2 = generate_key(generate_random_key_material(&rand).unwrap()).unwrap();

    let source_party = LinkageParty::new(HostName("source".to_string()), key1);
    let dest_party = LinkageParty::new(HostName("dest".to_string()), key2);

    let request = LinkageRequest::Client(ClientLinkageRequest {
        role: "client".to_string(),
        result: vec![LinkageRequestData {
            match_bit: 1,
            tentative_match: 0,
            best_index: 0,
        }],
    });

    let result = linkage_service
        .process_linkage_request(
            "source".to_string(),
            "dest".to_string(),
            &source_party,
            &dest_party,
            request,
        )
        .await;

    assert!(matches!(result, Err(LinkageServiceError::Timeout)));
}

#[tokio::test]
async fn linkage_service_should_reject_invalid_request_pairing() {
    let linkage_service = LinkageService::new();

    let rand = SystemRandom::new();
    let key1 = generate_key(generate_random_key_material(&rand).unwrap()).unwrap();
    let key2 = generate_key(generate_random_key_material(&rand).unwrap()).unwrap();

    let source_party = LinkageParty::new(HostName("source".to_string()), key1);
    let dest_party = LinkageParty::new(HostName("dest".to_string()), key2);

    // Create two client requests (invalid pairing)
    let client_request1 = LinkageRequest::Client(ClientLinkageRequest {
        role: "client".to_string(),
        result: vec![LinkageRequestData {
            match_bit: 1,
            tentative_match: 0,
            best_index: 0,
        }],
    });

    let client_request2 = LinkageRequest::Client(ClientLinkageRequest {
        role: "client".to_string(),
        result: vec![LinkageRequestData {
            match_bit: 0,
            tentative_match: 1,
            best_index: 0,
        }],
    });

    // Simulate the first request being stored
    let _result1 = tokio::spawn({
        let service = linkage_service.clone();
        let source_party = source_party.clone();
        let dest_party = dest_party.clone();
        let request = client_request1;
        async move {
            service
                .process_linkage_request(
                    "source".to_string(),
                    "dest".to_string(),
                    &source_party,
                    &dest_party,
                    request,
                )
                .await
        }
    });

    // Give some time for the first request to be stored
    tokio::time::sleep(Duration::from_millis(10)).await;

    // Send the second request which should find the first and fail due to invalid pairing
    let result2 = linkage_service
        .process_linkage_request(
            "dest".to_string(),
            "source".to_string(),
            &dest_party,
            &source_party,
            client_request2,
        )
        .await;

    assert!(matches!(
        result2,
        Err(LinkageServiceError::InvalidRequestPairing(_))
    ));
}

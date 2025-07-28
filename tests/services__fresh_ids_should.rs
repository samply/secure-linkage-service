use ring::rand::SystemRandom;
use secure_linkage_service::core::crypto::{generate_key, generate_random_key_material};
use secure_linkage_service::domain::{HostName, LinkageParty};
use secure_linkage_service::services::fresh_ids_service::{FreshIdsService, FreshIdsServiceError};

#[tokio::test]
async fn fresh_ids_service_should_generate_single_id() {
    let service = FreshIdsService::new();

    let rand = SystemRandom::new();
    let key = generate_key(generate_random_key_material(&rand).unwrap()).unwrap();
    let party = LinkageParty::new(HostName("test_party".to_string()), key);

    let result = service.generate_fresh_ids(&party, 1).await;

    assert!(result.is_ok());
    let ids = result.unwrap();
    assert_eq!(ids.len(), 1);
    assert!(!ids[0].0.0.is_empty());
}

#[tokio::test]
async fn fresh_ids_service_should_generate_multiple_ids() {
    let service = FreshIdsService::new();

    let rand = SystemRandom::new();
    let key = generate_key(generate_random_key_material(&rand).unwrap()).unwrap();
    let party = LinkageParty::new(HostName("test_party".to_string()), key);

    let result = service.generate_fresh_ids(&party, 5).await;

    assert!(result.is_ok());
    let ids = result.unwrap();
    assert_eq!(ids.len(), 5);

    // Check that all IDs are unique
    let id_strings: Vec<String> = ids.iter().map(|id| id.0.0.to_string()).collect();
    let mut unique_ids = id_strings.clone();
    unique_ids.sort();
    unique_ids.dedup();
    assert_eq!(unique_ids.len(), id_strings.len());
}

#[tokio::test]
async fn fresh_ids_service_should_reject_zero_count() {
    let service = FreshIdsService::new();

    let rand = SystemRandom::new();
    let key = generate_key(generate_random_key_material(&rand).unwrap()).unwrap();
    let party = LinkageParty::new(HostName("test_party".to_string()), key);

    let result = service.generate_fresh_ids(&party, 0).await;

    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        FreshIdsServiceError::InvalidCount(_)
    ));
}

#[tokio::test]
async fn fresh_ids_service_should_reject_excessive_count() {
    let service = FreshIdsService::new();

    let rand = SystemRandom::new();
    let key = generate_key(generate_random_key_material(&rand).unwrap()).unwrap();
    let party = LinkageParty::new(HostName("test_party".to_string()), key);

    let result = service.generate_fresh_ids(&party, 1001).await;

    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        FreshIdsServiceError::InvalidCount(_)
    ));
}

#[tokio::test]
async fn fresh_ids_service_should_track_generation_stats() {
    let service = FreshIdsService::new();

    let rand = SystemRandom::new();
    let key = generate_key(generate_random_key_material(&rand).unwrap()).unwrap();
    let party = LinkageParty::new(HostName("test_party".to_string()), key);

    // Initially no stats
    assert_eq!(service.get_generation_stats("test_party").await, None);

    // Generate some IDs
    let _result1 = service.generate_fresh_ids(&party, 3).await.unwrap();
    assert_eq!(service.get_generation_stats("test_party").await, Some(3));

    // Generate more IDs
    let _result2 = service.generate_fresh_ids(&party, 7).await.unwrap();
    assert_eq!(service.get_generation_stats("test_party").await, Some(10));
}

#[tokio::test]
async fn fresh_ids_service_should_handle_maximum_valid_count() {
    let service = FreshIdsService::new();

    let rand = SystemRandom::new();
    let key = generate_key(generate_random_key_material(&rand).unwrap()).unwrap();
    let party = LinkageParty::new(HostName("test_party".to_string()), key);

    let result = service.generate_fresh_ids(&party, 1000).await;

    assert!(result.is_ok());
    let ids = result.unwrap();
    assert_eq!(ids.len(), 1000);

    // Verify all IDs are unique (sampling check for performance)
    let id_strings: Vec<String> = ids.iter().take(100).map(|id| id.0.0.to_string()).collect();
    let mut unique_ids = id_strings.clone();
    unique_ids.sort();
    unique_ids.dedup();
    assert_eq!(unique_ids.len(), id_strings.len());
}

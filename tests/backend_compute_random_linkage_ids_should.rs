use ring::rand::SystemRandom;
use secure_linkage_service::{
    domain::*,
    transformations::{
        compute_random_linkage_ids,
        crypto::{generate_key, generate_random_key_material},
    },
};

#[test]
fn generate_zero_linkage_ids() {
    let rand = SystemRandom::new();
    let key = generate_key(generate_random_key_material(&rand).unwrap()).unwrap();
    let party = LinkageParty::new(HostName("test.example.com".to_string()), key);

    let result = compute_random_linkage_ids(&party, 0).expect("Should succeed with n=0");

    assert_eq!(result.len(), 0, "Should return empty vector for n=0");
}

#[test]
fn generate_single_linkage_id() {
    let rand = SystemRandom::new();
    let key = generate_key(generate_random_key_material(&rand).unwrap()).unwrap();
    let party = LinkageParty::new(HostName("test.example.com".to_string()), key);

    let result = compute_random_linkage_ids(&party, 1).expect("Should succeed with n=1");

    assert_eq!(result.len(), 1, "Should return exactly one linkage ID");

    let linkage_id = &result[0];
    assert!(!linkage_id.0.0.is_empty(), "Linkage ID should not be empty");
    assert!(
        !linkage_id.0.0.contains("ERROR"),
        "Linkage ID should not contain errors"
    );

    // Random linkage IDs should be base64 encoded and reasonably sized
    assert!(
        linkage_id.0.0.len() > 10,
        "Linkage ID should be reasonably long"
    );
    assert!(
        linkage_id.0.0.len() < 200,
        "Linkage ID should not be excessively long"
    );
}

#[test]
fn generate_multiple_linkage_ids() {
    let rand = SystemRandom::new();
    let key = generate_key(generate_random_key_material(&rand).unwrap()).unwrap();
    let party = LinkageParty::new(HostName("test.example.com".to_string()), key);

    let n = 5;
    let result = compute_random_linkage_ids(&party, n).expect("Should succeed with n=5");

    assert_eq!(result.len(), n, "Should return exactly {} linkage IDs", n);

    // Verify all IDs are valid
    for (i, linkage_id) in result.iter().enumerate() {
        assert!(
            !linkage_id.0.0.is_empty(),
            "Linkage ID {} should not be empty",
            i
        );
        assert!(
            !linkage_id.0.0.contains("ERROR"),
            "Linkage ID {} should not contain errors",
            i
        );
        assert!(
            linkage_id.0.0.len() > 10,
            "Linkage ID {} should be reasonably long",
            i
        );
    }
}

#[test]
fn generate_linkage_ids_are_unique() {
    let rand = SystemRandom::new();
    let key = generate_key(generate_random_key_material(&rand).unwrap()).unwrap();
    let party = LinkageParty::new(HostName("test.example.com".to_string()), key);

    let n = 10;
    let result = compute_random_linkage_ids(&party, n).expect("Should succeed");

    let id_strings: Vec<&str> = result.iter().map(|id| id.0.0.as_ref()).collect();

    // Check that all IDs are unique
    for i in 0..id_strings.len() {
        for j in (i + 1)..id_strings.len() {
            assert_ne!(
                id_strings[i], id_strings[j],
                "Linkage IDs at positions {} and {} should be unique",
                i, j
            );
        }
    }
}

#[test]
fn generate_linkage_ids_with_different_parties_are_different() {
    let rand = SystemRandom::new();

    let key1 = generate_key(generate_random_key_material(&rand).unwrap()).unwrap();
    let key2 = generate_key(generate_random_key_material(&rand).unwrap()).unwrap();

    let party1 = LinkageParty::new(HostName("party1.example.com".to_string()), key1);
    let party2 = LinkageParty::new(HostName("party2.example.com".to_string()), key2);

    let n = 3;
    let result1 = compute_random_linkage_ids(&party1, n).expect("Should succeed for party1");
    let result2 = compute_random_linkage_ids(&party2, n).expect("Should succeed for party2");

    assert_eq!(result1.len(), n);
    assert_eq!(result2.len(), n);

    // Different parties should generate different linkage IDs
    for i in 0..n {
        assert_ne!(
            result1[i].0.0.as_ref(),
            result2[i].0.0.as_ref(),
            "Linkage IDs from different parties at position {} should be different",
            i
        );
    }
}

#[test]
fn generate_large_number_of_linkage_ids() {
    let rand = SystemRandom::new();
    let key = generate_key(generate_random_key_material(&rand).unwrap()).unwrap();
    let party = LinkageParty::new(HostName("test.example.com".to_string()), key);

    let n = 100;
    let result = compute_random_linkage_ids(&party, n).expect("Should succeed with large n");

    assert_eq!(result.len(), n, "Should return exactly {} linkage IDs", n);

    // Spot check a few random positions
    for &i in &[0, 25, 50, 75, 99] {
        let linkage_id = &result[i];
        assert!(
            !linkage_id.0.0.is_empty(),
            "Linkage ID {} should not be empty",
            i
        );
        assert!(
            !linkage_id.0.0.contains("ERROR"),
            "Linkage ID {} should not contain errors",
            i
        );
    }
}

#[test]
fn generate_linkage_ids_with_same_party_multiple_calls_are_different() {
    let rand = SystemRandom::new();
    let key = generate_key(generate_random_key_material(&rand).unwrap()).unwrap();
    let party = LinkageParty::new(HostName("test.example.com".to_string()), key);

    // Generate IDs twice with the same party
    let result1 = compute_random_linkage_ids(&party, 3).expect("Should succeed on first call");
    let result2 = compute_random_linkage_ids(&party, 3).expect("Should succeed on second call");

    assert_eq!(result1.len(), 3);
    assert_eq!(result2.len(), 3);

    // Even with the same party, different calls should generate different IDs
    // (due to randomness in the generation process)
    for i in 0..3 {
        assert_ne!(
            result1[i].0.0.as_ref(),
            result2[i].0.0.as_ref(),
            "Linkage IDs from same party but different calls at position {} should be different",
            i
        );
    }
}

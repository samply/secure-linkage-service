use axum::{
    Router,
    body::Body,
    http::{Request, StatusCode},
};
use serde_json::Value;
use tower::ServiceExt;

use secure_linkage_service::rest_api::{AppState, fresh_record_ids::fresh_record_ids_routes};

fn create_test_app() -> Router {
    let state = AppState::new();
    fresh_record_ids_routes().with_state(state)
}

#[tokio::test]
async fn should_generate_single_fresh_id_when_no_count_specified() {
    let app = create_test_app();

    let request = Request::builder()
        .method("POST")
        .uri("/freshRecordIDs/test_client")
        .header("content-type", "application/json")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["status"], "success");
    assert_eq!(json["party"], "test_client");
    assert_eq!(json["count"], 1);
    assert_eq!(json["fresh_record_ids"].as_array().unwrap().len(), 1);
    assert!(json["fresh_record_ids"][0].as_str().unwrap().len() > 0);
}

#[tokio::test]
async fn should_generate_multiple_fresh_ids_when_count_specified() {
    let app = create_test_app();

    let request = Request::builder()
        .method("POST")
        .uri("/freshRecordIDs/test_server?n=5")
        .header("content-type", "application/json")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["status"], "success");
    assert_eq!(json["party"], "test_server");
    assert_eq!(json["count"], 5);
    assert_eq!(json["fresh_record_ids"].as_array().unwrap().len(), 5);

    // Verify all IDs are unique and non-empty
    let ids: Vec<&str> = json["fresh_record_ids"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap())
        .collect();

    for id in &ids {
        assert!(!id.is_empty());
    }

    // Check for uniqueness
    let mut unique_ids = ids.clone();
    unique_ids.sort();
    unique_ids.dedup();
    assert_eq!(unique_ids.len(), ids.len());
}

#[tokio::test]
async fn should_return_bad_request_for_zero_count() {
    let app = create_test_app();

    let request = Request::builder()
        .method("POST")
        .uri("/freshRecordIDs/test_client?n=0")
        .header("content-type", "application/json")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["status"], "error");
    assert_eq!(json["count"], 0);
    assert_eq!(json["fresh_record_ids"].as_array().unwrap().len(), 0);
    assert!(json["message"].as_str().unwrap().contains("greater than 0"));
}

#[tokio::test]
async fn should_return_bad_request_for_excessive_count() {
    let app = create_test_app();

    let request = Request::builder()
        .method("POST")
        .uri("/freshRecordIDs/test_client?n=1001")
        .header("content-type", "application/json")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["status"], "error");
    assert_eq!(json["count"], 1001);
    assert_eq!(json["fresh_record_ids"].as_array().unwrap().len(), 0);
    assert!(json["message"].as_str().unwrap().contains("exceed 1000"));
}

#[tokio::test]
async fn should_return_not_found_for_unknown_party() {
    let app = create_test_app();

    let request = Request::builder()
        .method("POST")
        .uri("/freshRecordIDs/unknown_party?n=1")
        .header("content-type", "application/json")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["status"], "error");
    assert_eq!(json["party"], "unknown_party");
    assert_eq!(json["fresh_record_ids"].as_array().unwrap().len(), 0);
    assert!(json["message"].as_str().unwrap().contains("not found"));
}

#[tokio::test]
async fn should_generate_fresh_ids_for_all_default_parties() {
    let app = create_test_app();
    let test_parties = ["dkfz", "tuda", "test_client", "test_server"];

    for party in test_parties {
        let request = Request::builder()
            .method("POST")
            .uri(&format!("/freshRecordIDs/{}?n=3", party))
            .header("content-type", "application/json")
            .body(Body::empty())
            .unwrap();

        let response = app.clone().oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(json["status"], "success");
        assert_eq!(json["party"], party);
        assert_eq!(json["count"], 3);
        assert_eq!(json["fresh_record_ids"].as_array().unwrap().len(), 3);
    }
}

#[tokio::test]
async fn should_handle_large_valid_count() {
    let app = create_test_app();

    let request = Request::builder()
        .method("POST")
        .uri("/freshRecordIDs/test_client?n=100")
        .header("content-type", "application/json")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(json["status"], "success");
    assert_eq!(json["count"], 100);
    assert_eq!(json["fresh_record_ids"].as_array().unwrap().len(), 100);

    // Verify all generated IDs are unique
    let ids: Vec<&str> = json["fresh_record_ids"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap())
        .collect();

    let mut unique_ids = ids.clone();
    unique_ids.sort();
    unique_ids.dedup();
    assert_eq!(unique_ids.len(), ids.len(), "All IDs should be unique");
}

#[tokio::test]
async fn should_return_consistent_response_format() {
    let app = create_test_app();

    let request = Request::builder()
        .method("POST")
        .uri("/freshRecordIDs/test_client?n=2")
        .header("content-type", "application/json")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();

    // Verify all expected fields are present
    assert!(json.get("status").is_some());
    assert!(json.get("party").is_some());
    assert!(json.get("count").is_some());
    assert!(json.get("fresh_record_ids").is_some());
    assert!(json.get("message").is_some());

    // Verify field types
    assert!(json["status"].is_string());
    assert!(json["party"].is_string());
    assert!(json["count"].is_number());
    assert!(json["fresh_record_ids"].is_array());
    assert!(json["message"].is_null() || json["message"].is_string());
}

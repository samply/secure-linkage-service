use axum::{
    Router,
    body::Body,
    http::{Request, StatusCode},
};
use ring::rand::SystemRandom;
use serde_json::json;
use tokio::time::Duration;
use tower::ServiceExt;

use secure_linkage_service::rest_api::{AppState, LinkageResponse, handle_linkage_request};
use secure_linkage_service::transformations::crypto::encrypt_data_to_base64;

fn create_test_app() -> Router {
    let state = AppState::new();
    Router::new()
        .route(
            "/linkageResult/{source_party}/{destination_party}",
            axum::routing::post(handle_linkage_request),
        )
        .with_state(state)
}

fn create_test_app_with_timeout(timeout: Duration) -> Router {
    let state = AppState::with_timeout(timeout);
    Router::new()
        .route(
            "/linkageResult/{source_party}/{destination_party}",
            axum::routing::post(handle_linkage_request),
        )
        .with_state(state)
}

#[tokio::test]
async fn server_request_returns_200_immediately() {
    let app = create_test_app();

    let server_request = json!({
        "role": "server",
        "result": [
            {
                "match": 0,
                "tentativeMatch": 1,
                "bestIndex": 2
            },
            {
                "match": 1,
                "tentativeMatch": 0,
                "bestIndex": 0
            },
            {
                "match": 0,
                "tentativeMatch": 0,
                "bestIndex": 4
            }
        ],
        "ids": ["aGVsbG93b3JsZA==", "ZGF0YXNjaWVuY2U=", "cHl0aG9uaXNmdW4="]
    });

    let request = Request::builder()
        .method("POST")
        .uri("/linkageResult/dkfz/tuda")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&server_request).unwrap()))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let response_json: LinkageResponse = serde_json::from_slice(&body).unwrap();

    assert_eq!(response_json.status, "accepted");
    assert!(response_json.message.is_some());
    assert!(response_json.linkage_ids.is_none());
}

#[tokio::test]
async fn client_request_waits_and_times_out() {
    // Use 2 second timeout for tests
    let app = create_test_app_with_timeout(Duration::from_secs(2));

    let client_request = json!({
        "role": "client",
        "result": [
            {
                "match": 1,
                "tentativeMatch": 0,
                "bestIndex": 2
            },
            {
                "match": 0,
                "tentativeMatch": 1,
                "bestIndex": 0
            },
            {
                "match": 1,
                "tentativeMatch": 1,
                "bestIndex": 4
            }
        ]
    });

    let request = Request::builder()
        .method("POST")
        .uri("/linkageResult/tuda/dkfz")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&client_request).unwrap()))
        .unwrap();

    let start = std::time::Instant::now();
    let response = app.oneshot(request).await.unwrap();
    let duration = start.elapsed();

    // Should timeout after approximately 2 seconds
    assert!(duration.as_secs() >= 1);
    // Allow some margin for test timing
    assert!(duration.as_secs() < 4);
    assert_eq!(response.status(), StatusCode::REQUEST_TIMEOUT);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let response_json: LinkageResponse = serde_json::from_slice(&body).unwrap();

    assert_eq!(response_json.status, "timeout");
    assert!(response_json.linkage_ids.is_none());
}

#[tokio::test]
async fn successful_linkage_computation() {
    // Create shared state and get the server key that will be used
    let state = AppState::new();
    let parties = state.parties.read().await;
    let server_party = parties.get("dkfz").unwrap();
    let server_key = &server_party.secret_key;

    // Create properly encrypted IDs using the same key the system will use
    let rand = SystemRandom::new();
    let encrypted_id1 = encrypt_data_to_base64(&rand, server_key, b"helloworld").unwrap();
    let encrypted_id2 = encrypt_data_to_base64(&rand, server_key, b"datascience").unwrap();
    let encrypted_id3 = encrypt_data_to_base64(&rand, server_key, b"pythonisfun").unwrap();
    drop(parties);

    let server_request = json!({
        "role": "server",
        "result": [
            {
                "match": 0,
                "tentativeMatch": 1,
                "bestIndex": 0
            },
            {
                "match": 1,
                "tentativeMatch": 0,
                "bestIndex": 1
            },
            {
                "match": 0,
                "tentativeMatch": 0,
                "bestIndex": 2
            }
        ],
        "ids": [encrypted_id1.0.as_ref(), encrypted_id2.0.as_ref(), encrypted_id3.0.as_ref()]
    });

    let client_request = json!({
        "role": "client",
        "result": [
            {
                "match": 1,
                "tentativeMatch": 0,
                "bestIndex": 0
            },
            {
                "match": 0,
                "tentativeMatch": 1,
                "bestIndex": 1
            },
            {
                "match": 0,
                "tentativeMatch": 0,
                "bestIndex": 2
            }
        ]
    });

    // Create a shared state that both requests will use
    let app1 = Router::new()
        .route(
            "/linkageResult/{source_party}/{destination_party}",
            axum::routing::post(handle_linkage_request),
        )
        .with_state(state.clone());

    let app2 = Router::new()
        .route(
            "/linkageResult/{source_party}/{destination_party}",
            axum::routing::post(handle_linkage_request),
        )
        .with_state(state);

    let server_req = Request::builder()
        .method("POST")
        .uri("/linkageResult/dkfz/tuda")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&server_request).unwrap()))
        .unwrap();

    let client_req = Request::builder()
        .method("POST")
        .uri("/linkageResult/tuda/dkfz")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&client_request).unwrap()))
        .unwrap();

    // Send both requests concurrently with a small delay to ensure proper order
    let (server_response, client_response) = tokio::join!(app1.oneshot(server_req), async {
        // Small delay to ensure server request is processed first
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
        app2.oneshot(client_req).await
    });

    let server_resp = server_response.unwrap();
    let client_resp = client_response.unwrap();

    // Check status codes first
    let server_status = server_resp.status();
    let client_status = client_resp.status();

    assert_eq!(server_status, StatusCode::OK);

    // Debug the client response if it's not 200
    if client_status != StatusCode::OK {
        let body = axum::body::to_bytes(client_resp.into_body(), usize::MAX)
            .await
            .unwrap();
        let response_json: LinkageResponse = serde_json::from_slice(&body).unwrap();
        eprintln!("Client response status: {:?}", client_status);
        eprintln!("Client response: {:?}", response_json);
        panic!("Expected 200, got {}", client_status.as_u16());
    }

    assert_eq!(client_status, StatusCode::OK);

    // Check server response (should be "accepted")
    let server_body = axum::body::to_bytes(server_resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let server_json: LinkageResponse = serde_json::from_slice(&server_body).unwrap();
    assert_eq!(server_json.status, "accepted");

    // Check client response (should be "completed" with linkage IDs)
    let client_body = axum::body::to_bytes(client_resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let client_json: LinkageResponse = serde_json::from_slice(&client_body).unwrap();

    assert_eq!(client_json.status, "completed");
    assert!(client_json.linkage_ids.is_some());

    let linkage_ids = client_json.linkage_ids.unwrap();
    assert_eq!(linkage_ids.len(), 3);

    // First two should have linkage IDs (matches), last one should also have an ID (random)
    for id in &linkage_ids {
        assert!(id.is_some());
    }
}

#[tokio::test]
async fn invalid_party_returns_400() {
    let app = create_test_app();

    let request_data = json!({
        "role": "server",
        "result": [
            {
                "match": 0,
                "tentativeMatch": 1,
                "bestIndex": 2
            }
        ],
        "ids": ["aGVsbG93b3JsZA=="]
    });

    let request = Request::builder()
        .method("POST")
        .uri("/linkageResult/invalid_party/tuda")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&request_data).unwrap()))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn concurrent_requests_handled_correctly() {
    // Create shared state and get the server key
    let state = AppState::new();
    let parties = state.parties.read().await;
    let server_party = parties.get("dkfz").unwrap();
    let server_key = &server_party.secret_key;

    // Create properly encrypted ID using the same key the system will use
    let rand = SystemRandom::new();
    let encrypted_id = encrypt_data_to_base64(&rand, server_key, b"helloworld").unwrap();
    drop(parties);

    let server_request = json!({
        "role": "server",
        "result": [
            {
                "match": 0,
                "tentativeMatch": 1,
                "bestIndex": 0
            }
        ],
        "ids": [encrypted_id.0.as_ref()]
    });

    let client_request = json!({
        "role": "client",
        "result": [
            {
                "match": 1,
                "tentativeMatch": 0,
                "bestIndex": 0
            }
        ]
    });

    // Send both requests concurrently with proper timing
    let server_req = Request::builder()
        .method("POST")
        .uri("/linkageResult/dkfz/tuda")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&server_request).unwrap()))
        .unwrap();

    let client_req = Request::builder()
        .method("POST")
        .uri("/linkageResult/tuda/dkfz")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&client_request).unwrap()))
        .unwrap();

    // Send both requests concurrently with a small delay to ensure server goes first
    let (server_response, client_response) = tokio::join!(
        Router::new()
            .route(
                "/linkageResult/{source_party}/{destination_party}",
                axum::routing::post(handle_linkage_request),
            )
            .with_state(state.clone())
            .oneshot(server_req),
        async {
            // Small delay to ensure server request is processed first
            tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
            Router::new()
                .route(
                    "/linkageResult/{source_party}/{destination_party}",
                    axum::routing::post(handle_linkage_request),
                )
                .with_state(state)
                .oneshot(client_req)
                .await
        }
    );

    let server_resp = server_response.unwrap();
    let client_resp = client_response.unwrap();

    assert_eq!(server_resp.status(), StatusCode::OK);
    assert_eq!(client_resp.status(), StatusCode::OK);

    // Check that both got successful responses
    let server_body = axum::body::to_bytes(server_resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let client_body = axum::body::to_bytes(client_resp.into_body(), usize::MAX)
        .await
        .unwrap();

    let server_json: LinkageResponse = serde_json::from_slice(&server_body).unwrap();
    let client_json: LinkageResponse = serde_json::from_slice(&client_body).unwrap();

    // Server should return "accepted" since it's the first to arrive and waits
    assert_eq!(server_json.status, "accepted");
    assert!(server_json.linkage_ids.is_none());

    // Client should return "completed" with linkage IDs since it triggers the computation
    assert_eq!(client_json.status, "completed");
    assert!(client_json.linkage_ids.is_some());

    let linkage_ids = client_json.linkage_ids.unwrap();
    assert_eq!(linkage_ids.len(), 1);
    assert!(linkage_ids[0].is_some());
}

#[tokio::test]
async fn mismatched_data_length_returns_error() {
    // Create shared state that both requests will use
    let state = AppState::new();

    let server_request = json!({
        "role": "server",
        "result": [
            {
                "match": 0,
                "tentativeMatch": 1,
                "bestIndex": 0
            },
            {
                "match": 1,
                "tentativeMatch": 0,
                "bestIndex": 1
            }
        ],
        "ids": ["aGVsbG93b3JsZA==", "ZGF0YXNjaWVuY2U="]
    });

    let client_request = json!({
        "role": "client",
        "result": [
            {
                "match": 1,
                "tentativeMatch": 0,
                "bestIndex": 0
            }
        ]
    });

    // Create apps with shared state
    let app1 = Router::new()
        .route(
            "/linkageResult/{source_party}/{destination_party}",
            axum::routing::post(handle_linkage_request),
        )
        .with_state(state.clone());

    let app2 = Router::new()
        .route(
            "/linkageResult/{source_party}/{destination_party}",
            axum::routing::post(handle_linkage_request),
        )
        .with_state(state);

    // Send server request first
    let server_req = Request::builder()
        .method("POST")
        .uri("/linkageResult/dkfz/tuda")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&server_request).unwrap()))
        .unwrap();

    let server_response = app1.oneshot(server_req).await.unwrap();
    assert_eq!(server_response.status(), StatusCode::OK);

    // Send client request with mismatched length
    let client_req = Request::builder()
        .method("POST")
        .uri("/linkageResult/tuda/dkfz")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(&client_request).unwrap()))
        .unwrap();

    let client_response = app2.oneshot(client_req).await.unwrap();
    assert_eq!(client_response.status(), StatusCode::INTERNAL_SERVER_ERROR);

    let body = axum::body::to_bytes(client_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let response_json: LinkageResponse = serde_json::from_slice(&body).unwrap();

    assert_eq!(response_json.status, "error");
    assert!(response_json.message.is_some());

    // Check that we got an error message that indicates data length mismatch
    let error_message = response_json.message.unwrap();
    let has_length_related_error = error_message.to_lowercase().contains("length")
        || error_message.to_lowercase().contains("mismatch")
        || error_message.to_lowercase().contains("different");

    assert!(
        has_length_related_error,
        "Error message should indicate a length/mismatch issue, but got: '{}'",
        error_message
    );
}

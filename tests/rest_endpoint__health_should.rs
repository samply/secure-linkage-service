mod common;

use axum::http::StatusCode;
use secure_linkage_service::rest_api::AppState;
use serde_json::Value;
use tokio::net::TcpListener;
use tower::ServiceExt;

// Helper function to create a test router
fn create_test_router() -> axum::Router {
    use axum::Router;
    use secure_linkage_service::rest_api::health::health_routes;

    let state = AppState::new();

    Router::new().merge(health_routes()).with_state(state)
}

#[tokio::test]
async fn health_endpoint_should_return_healthy_status() {
    // Arrange
    let app = create_test_router();

    // Act
    let response = app
        .oneshot(
            axum::http::Request::builder()
                .uri("/health")
                .method("GET")
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // Assert
    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();

    let health_response: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(health_response["status"], "healthy");
    assert_eq!(health_response["service"], "linkage-service");
    assert!(health_response["timestamp"].is_string());
    assert!(health_response["version"].is_string());
}

#[tokio::test]
async fn health_endpoint_should_have_correct_response_format() {
    // Arrange
    let app = create_test_router();

    // Act
    let response = app
        .oneshot(
            axum::http::Request::builder()
                .uri("/health")
                .method("GET")
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // Assert response structure
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();

    let health_response: Value = serde_json::from_slice(&body).unwrap();

    // Verify all expected fields are present
    assert!(health_response.get("status").is_some());
    assert!(health_response.get("service").is_some());
    assert!(health_response.get("timestamp").is_some());
    assert!(health_response.get("version").is_some());

    // Verify timestamp is a valid RFC3339 format
    let timestamp = health_response["timestamp"].as_str().unwrap();
    chrono::DateTime::parse_from_rfc3339(timestamp)
        .expect("Timestamp should be valid RFC3339 format");
}

#[tokio::test]
async fn health_endpoint_should_work_with_actual_server() {
    // Arrange - Start a real server on a random port
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    let app = create_test_router();

    // Start server in background
    let server_handle = tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    // Give the server a moment to start
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Act - Make actual HTTP request
    let client = reqwest::Client::new();
    let response = client
        .get(&format!("http://{}/health", addr))
        .send()
        .await
        .unwrap();

    // Assert
    assert_eq!(response.status(), reqwest::StatusCode::OK);

    let health_response: Value = response.json().await.unwrap();
    assert_eq!(health_response["status"], "healthy");
    assert_eq!(health_response["service"], "linkage-service");

    // Clean up
    server_handle.abort();
}

#[tokio::test]
async fn nonexistent_endpoint_should_return_404() {
    // Arrange
    let app = create_test_router();

    // Act
    let response = app
        .oneshot(
            axum::http::Request::builder()
                .uri("/nonexistent")
                .method("GET")
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // Assert
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn health_endpoint_should_handle_wrong_http_method() {
    // Arrange
    let app = create_test_router();

    // Act - Try POST instead of GET
    let response = app
        .oneshot(
            axum::http::Request::builder()
                .uri("/health")
                .method("POST")
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // Assert
    assert_eq!(response.status(), StatusCode::METHOD_NOT_ALLOWED);
}

#[tokio::test]
async fn health_endpoints_should_be_accessible_concurrently() {
    // Arrange
    let app = create_test_router();

    // Act - Make multiple concurrent requests
    let tasks = (0..10).map(|_| {
        let app_clone = app.clone();
        tokio::spawn(async move {
            app_clone
                .oneshot(
                    axum::http::Request::builder()
                        .uri("/health")
                        .method("GET")
                        .body(axum::body::Body::empty())
                        .unwrap(),
                )
                .await
                .unwrap()
        })
    });

    let responses = futures::future::join_all(tasks).await;

    // Assert - All requests should succeed
    for response_result in responses {
        let response = response_result.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();

        let health_response: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(health_response["status"], "healthy");
    }
}

#[tokio::test]
async fn health_check_should_not_affect_application_state() {
    // Arrange
    let app = create_test_router();

    // Act - Make health check request
    let health_response = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .uri("/health")
                .method("GET")
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // Make another request to verify state is unchanged
    let second_response = app
        .oneshot(
            axum::http::Request::builder()
                .uri("/health")
                .method("GET")
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // Assert - Both responses should be identical in structure
    assert_eq!(health_response.status(), StatusCode::OK);
    assert_eq!(second_response.status(), StatusCode::OK);

    let first_body = axum::body::to_bytes(health_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let second_body = axum::body::to_bytes(second_response.into_body(), usize::MAX)
        .await
        .unwrap();

    let first_json: Value = serde_json::from_slice(&first_body).unwrap();
    let second_json: Value = serde_json::from_slice(&second_body).unwrap();

    // Status and service should be identical
    assert_eq!(first_json["status"], second_json["status"]);
    assert_eq!(first_json["service"], second_json["service"]);
}

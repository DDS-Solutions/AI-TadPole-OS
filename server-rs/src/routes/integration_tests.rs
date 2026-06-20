//! @docs ARCHITECTURE:Gateways
//! 
//! ### AI Assist Note
//! **! Axum Router API Integration Tests**
//! This module implements high-fidelity logic for the Sovereign Reality layer.
//! 
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: Runtime logic error, state desynchronization, or resource exhaustion.
//! - **Telemetry Link**: Search `[integration_tests]` in tracing logs.

//! Axum Router API Integration Tests
//!
//! Exercises the full Axum router (`create_router`) and the entire middleware pipeline
//! (auth token validation, compression, headers, and fallbacks) against a test state.
//!
//! @docs ARCHITECTURE:Observability
//! @docs ARCHITECTURE:Gateway

use axum::{
    body::Body,
    http::{header, Request, StatusCode},
    Router,
};
use std::sync::Arc;
use tower::ServiceExt;

use crate::{router::create_router, state::AppState};

// Helper function to build a full production router backed by AppState
async fn test_app() -> (Router, Arc<AppState>) {
    let app_state = Arc::new(
        AppState::new()
            .await
            .expect("Failed to initialize state for integration tests"),
    );

    let app = create_router(app_state.clone());

    (app, app_state)
}

#[tokio::test]
async fn test_metrics_endpoint_public() {
    let (app, _) = test_app().await;

    let request = Request::builder()
        .uri("/metrics")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    
    let content_type = response.headers().get(header::CONTENT_TYPE).unwrap();
    assert!(content_type.to_str().unwrap().contains("text/plain"));

    let bytes = axum::body::to_bytes(response.into_body(), 100000)
        .await
        .unwrap();
    let body = String::from_utf8(bytes.to_vec()).unwrap();
    
    // Verify some standard Prometheus metric format elements are present
    assert!(
        body.contains("# HELP") 
        || body.contains("process_cpu_seconds_total") 
        || body.contains("tool_latency_p50")
    );
}

#[tokio::test]
async fn test_versioned_metrics_endpoint_public() {
    let (app, _) = test_app().await;

    let request = Request::builder()
        .uri("/v1/engine/metrics")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    
    let content_type = response.headers().get(header::CONTENT_TYPE).unwrap();
    assert!(content_type.to_str().unwrap().contains("text/plain"));
}

#[tokio::test]
async fn test_health_endpoint_public() {
    let (app, _) = test_app().await;

    let request = Request::builder()
        .uri("/v1/engine/health")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    
    let content_type = response.headers().get(header::CONTENT_TYPE).unwrap();
    assert!(content_type.to_str().unwrap().contains("application/json"));

    let bytes = axum::body::to_bytes(response.into_body(), 100000)
        .await
        .unwrap();
    let body: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    
    // Check fields in the health response
    assert_eq!(body["status"].as_str().unwrap().contains("tadpole"), true);
    assert!(body["version"].is_string());
}

#[tokio::test]
async fn test_protected_agents_endpoint_denied() {
    let (app, _) = test_app().await;

    let request = Request::builder()
        .uri("/v1/agents")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    // Requests to /v1/agents without a valid Bearer token must fail auth
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_protected_agents_endpoint_allowed() {
    let (app, state) = test_app().await;
    let auth_header = format!("Bearer {}", state.security.deploy_token);

    let request = Request::builder()
        .uri("/v1/agents")
        .header(header::AUTHORIZATION, auth_header)
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_fallback_handler() {
    let (app, _) = test_app().await;

    let request = Request::builder()
        .uri("/v1/invalid-route-name-xyz")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    
    let content_type = response.headers().get(header::CONTENT_TYPE).unwrap();
    assert!(content_type.to_str().unwrap().contains("application/json"));
    
    let bytes = axum::body::to_bytes(response.into_body(), 100000)
        .await
        .unwrap();
    let body: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    
    // ProblemDetails elements
    assert_eq!(body["title"].as_str().unwrap(), "Not Found");
    assert!(body["detail"].as_str().unwrap().contains("does not exist"));
}

// Metadata: [integration_tests]

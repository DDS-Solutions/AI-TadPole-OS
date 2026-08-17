//! @docs ARCHITECTURE:Gateways
//!
//! ### AI Assist Note
//! **! Axum Router API Integration Tests**
//! This module implements high-fidelity logic for the Sovereign Reality layer.
//!
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: Runtime logic error, state desynchronization, or resource exhaustion.
//! - **Telemetry Link**: Search `[integration_tests]` in tracing logs.
//!
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
async fn test_metrics_endpoint_root_unexposed() {
    let (app, _) = test_app().await;

    let request = Request::builder()
        .uri("/metrics")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    // Root unauthenticated /metrics endpoint is removed for zero-trust compliance
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_versioned_metrics_endpoint_protected() {
    let (app, _) = test_app().await;

    let token = std::env::var("NEURAL_TOKEN").unwrap_or_else(|_| "test-token".to_string());
    let request = Request::builder()
        .uri("/v1/engine/metrics")
        .header(header::AUTHORIZATION, format!("Bearer {}", token))
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
    assert!(body["status"].as_str().unwrap().contains("tadpole"));
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
async fn test_remote_pairing_token_requires_desktop_authentication() {
    let (app, state) = test_app().await;

    let denied = Request::builder()
        .uri("/v1/remote/pairing-token")
        .header("x-forwarded-for", "10.0.0.1")
        .body(Body::empty())
        .unwrap();
    assert_eq!(
        app.clone().oneshot(denied).await.unwrap().status(),
        StatusCode::UNAUTHORIZED
    );

    let allowed = Request::builder()
        .uri("/v1/remote/pairing-token")
        .header(
            header::AUTHORIZATION,
            format!("Bearer {}", state.security.deploy_token),
        )
        .header("x-forwarded-for", "10.0.0.1")
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(allowed).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let bytes = axum::body::to_bytes(response.into_body(), 100000)
        .await
        .unwrap();
    let body: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert!(body["token"]
        .as_str()
        .is_some_and(|token| token.starts_with("TP-PAIR-")));
}

#[tokio::test]
async fn test_remote_sensitive_read_requires_signed_device_request() {
    let (app, _) = test_app().await;
    let request = Request::builder()
        .uri("/v1/remote/agents/health")
        .header("x-forwarded-for", "10.0.0.1")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
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

#[tokio::test]
async fn test_chat_completions_endpoint_denied() {
    let (app, _) = test_app().await;

    let request = Request::builder()
        .uri("/v1/agents/chat/completions")
        .method("POST")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            serde_json::json!({
                "model": "general",
                "messages": [
                    { "role": "user", "content": "who are you" }
                ]
            })
            .to_string(),
        ))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_chat_completions_endpoint_allowed() {
    let (app, state) = test_app().await;
    let auth_header = format!("Bearer {}", state.security.deploy_token);

    // Register a mock agent in registry to ensure model validation passes
    // let mut mock_agent = crate::agent::types::EngineAgent::default();
    // mock_agent.identity.id = "mock-completion-agent".to_string();
    // mock_agent.identity.name = "Mock Completion Agent".to_string();
    // state.registry.agents.insert("mock-completion-agent".to_string(), mock_agent);

    let request = Request::builder()
        .uri("/v1/agents/chat/completions")
        .method("POST")
        .header(header::AUTHORIZATION, auth_header)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            serde_json::json!({
                "model": "mock-completion-agent",
                "messages": [
                    { "role": "user", "content": "who are you" }
                ]
            })
            .to_string(),
        ))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    let status = response.status();
    let bytes = axum::body::to_bytes(response.into_body(), 100000)
        .await
        .unwrap();
    let body = String::from_utf8(bytes.to_vec()).unwrap();

    println!("STATUS: {:?}", status);
    println!("BODY: {}", body);

    // The route exists and auth is correct; it should not be UNAUTHORIZED or ROUTE_NOT_FOUND
    assert_ne!(status, StatusCode::UNAUTHORIZED);
    if status == StatusCode::NOT_FOUND {
        // If it's a NOT_FOUND, it must be because of the mock agent, not the route
        assert!(body.contains("mock-completion-agent"));
    }
}

#[tokio::test]
async fn test_a2a_mailbox_send_denied_missing_signature() {
    let (app, _) = test_app().await;

    let payload = serde_json::json!({
        "id": "msg-999",
        "mission_id": "mission-xyz",
        "source_agent_id": "agent-alice",
        "target_agent_id": "agent-bob",
        "instruction": "ping",
        "status": "pending"
    });

    let request = Request::builder()
        .uri("/v1/engine/a2a/mailbox/send")
        .method("POST")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(payload.to_string()))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_a2a_mailbox_send_allowed_with_valid_signature() {
    let (app, state) = test_app().await;

    // Run migrations to ensure mailbox tables exist
    sqlx::migrate!("./migrations")
        .run(&state.resources.pool)
        .await
        .unwrap();

    // Seed agent and mission
    sqlx::query(
        "INSERT OR IGNORE INTO agents (id, name, role, department, description, status, metadata) \
         VALUES ('agent-bob', 'Bob', 'assistant', 'engineering', 'bob desc', 'active', '{}')",
    )
    .execute(&state.resources.pool)
    .await
    .unwrap();

    sqlx::query(
        "INSERT OR IGNORE INTO mission_history (id, agent_id, title, status) \
         VALUES ('mission-abc', 'agent-bob', 'test mission', 'active')",
    )
    .execute(&state.resources.pool)
    .await
    .unwrap();

    // Setup keyring
    use crate::agent::runner::tools::capability::{
        reset_keyring_for_test, set_key, sign_a2a_envelope,
    };
    let test_key = [7u8; 32];
    reset_keyring_for_test();
    set_key("test_a2a_key", test_key);
    set_key("curr", test_key);

    let envelope = crate::agent::runner::a2a_mailbox::MailboxEnvelope {
        id: "msg-123".to_string(),
        mission_id: "mission-abc".to_string(),
        source_agent_id: "agent-alice".to_string(),
        target_agent_id: "agent-bob".to_string(),
        instruction: "ping".to_string(),
        reasoning_trace: None,
        status: "pending".to_string(),
        result: None,
        artifacts: None,
    };

    let envelope_str = serde_json::to_string(&envelope).unwrap();
    let signature = sign_a2a_envelope(&envelope_str);

    let request = Request::builder()
        .uri("/v1/engine/a2a/mailbox/send")
        .method("POST")
        .header("X-A2A-Signature", signature)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(envelope_str))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::ACCEPTED);
}

// Metadata: [integration_tests]

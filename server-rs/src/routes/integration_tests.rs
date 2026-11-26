//! @docs ARCHITECTURE:Gateways
//!
//! ### AI Assist Note
//! - Use isolated fixture storage and settings while exercising real routes and oversight services.
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / HTTP Routes / integration_tests
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Type-safe state handling and bounded execution without unhandled panics.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: none declared

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
    let app_state = Arc::new(AppState::new_minimal_mock().await);

    for sub in [
        "Database",
        "Agents",
        "MCP",
        "Heartbeat",
        "SecurityEviction",
        "PrivacyGuard",
        "BudgetFlush",
        "Network",
        "SwarmPulse",
        "CodeGraph",
        "CodeGraphDbRefresh",
    ] {
        app_state
            .resources
            .set_subsystem_status(sub, crate::types::SubsystemStatus::Ready);
    }

    let app = create_router(app_state.clone()).layer(axum::middleware::from_fn(
        |mut req: Request<Body>, next: axum::middleware::Next| async move {
            if req
                .extensions()
                .get::<axum::extract::ConnectInfo<std::net::SocketAddr>>()
                .is_none()
            {
                req.extensions_mut().insert(axum::extract::ConnectInfo(
                    std::net::SocketAddr::from(([127, 0, 0, 1], 8080)),
                ));
            }
            next.run(req).await
        },
    ));

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
    let (app, state) = test_app().await;

    // 1. Unauthenticated request must fail
    let unauth_request = Request::builder()
        .uri("/v1/engine/metrics")
        .body(Body::empty())
        .expect("build unauth request");

    let unauth_response = app.clone().oneshot(unauth_request).await.unwrap();
    assert_eq!(unauth_response.status(), StatusCode::UNAUTHORIZED);

    // 2. Authenticated request with deploy token must succeed
    let auth_request = Request::builder()
        .uri("/v1/engine/metrics")
        .header(
            header::AUTHORIZATION,
            format!("Bearer {}", state.security.deploy_token),
        )
        .body(Body::empty())
        .expect("build auth request");

    let response = app.oneshot(auth_request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let content_type = response
        .headers()
        .get(header::CONTENT_TYPE)
        .expect("content-type header present");
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

    let deploy_denied = Request::builder()
        .uri("/v1/remote/pairing-token")
        .header(
            header::AUTHORIZATION,
            format!("Bearer {}", state.security.deploy_token),
        )
        .header("x-forwarded-for", "10.0.0.1")
        .body(Body::empty())
        .unwrap();
    assert_eq!(
        app.clone().oneshot(deploy_denied).await.unwrap().status(),
        StatusCode::FORBIDDEN
    );

    let allowed = Request::builder()
        .uri("/v1/remote/pairing-token")
        .header(
            header::AUTHORIZATION,
            format!("Bearer {}", state.security.admin_token),
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
    assert!(content_type
        .to_str()
        .unwrap()
        .contains("application/problem+json"));

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
    let mut mock_agent = crate::agent::types::EngineAgent::default();
    mock_agent.identity.id = "mock-completion-agent".to_string();
    mock_agent.identity.name = "Mock Completion Agent".to_string();
    state
        .registry
        .agents
        .insert("mock-completion-agent".to_string(), mock_agent);

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

    // The route exists and auth is valid
    assert_ne!(status, StatusCode::UNAUTHORIZED);
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
        reset_keyring_for_test, set_key, sign_a2a_canonical, TEST_KEYRING_LOCK,
    };
    let _key_lock = TEST_KEYRING_LOCK.lock().await;
    let test_key = [9u8; 32];
    reset_keyring_for_test();
    set_key("curr", test_key);

    let now = chrono::Utc::now().timestamp_millis();
    let nonce = "integration-nonce-123";

    let envelope = crate::agent::runner::a2a_mailbox::MailboxEnvelope {
        id: "msg-123".to_string(),
        mission_id: "mission-abc".to_string(),
        source_agent_id: "agent-alice".to_string(),
        target_agent_id: "agent-bob".to_string(),
        recipient_agent_id: None,
        instruction: "ping".to_string(),
        reasoning_trace: None,
        status: "pending".to_string(),
        result: None,
        artifacts: None,
        timestamp: Some(now),
        nonce: Some(nonce.to_string()),
    };

    let signature = sign_a2a_canonical(
        &envelope.id,
        &envelope.mission_id,
        &envelope.source_agent_id,
        &envelope.target_agent_id,
        &envelope.instruction,
        now,
        nonce,
    );
    let envelope_str = serde_json::to_string(&envelope).unwrap();

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

#[tokio::test]
async fn test_hardened_admin_routes_reject_deploy_token_with_forbidden() {
    let (app, state) = test_app().await;
    let deploy_auth = format!("Bearer {}", state.security.deploy_token);

    let cases = [
        ("GET", "/v1/system/backup/status", String::new()),
        ("GET", "/v1/system/backup/history", String::new()),
        (
            "POST",
            "/v1/system/backup/configure",
            serde_json::json!({
                "enabled": true,
                "destination_type": "local",
                "destination_path": "backups/test"
            })
            .to_string(),
        ),
        ("POST", "/v1/system/backup/trigger", "{}".to_string()),
        (
            "POST",
            "/v1/system/backup/restore",
            serde_json::json!({
                "snapshot_path": "backups/test.tadpole-snap.enc",
                "apply": false
            })
            .to_string(),
        ),
        (
            "POST",
            "/v1/agents",
            serde_json::json!({
                "id": "rbac-test-agent",
                "name": "RBAC Agent",
                "role": "Analyst",
                "department": "QA",
                "description": "test"
            })
            .to_string(),
        ),
        (
            "PUT",
            "/v1/agents/rbac-test-agent",
            serde_json::json!({"requiresOversight": false}).to_string(),
        ),
        ("DELETE", "/v1/agents/rbac-test-agent", String::new()),
        (
            "POST",
            "/v1/continuity/jobs",
            serde_json::json!({
                "agent_id": "a1",
                "name": "cron",
                "schedule": "0 * * * *",
                "prompt": "hi"
            })
            .to_string(),
        ),
        (
            "POST",
            "/v1/continuity/workflows",
            serde_json::json!({"name": "wf"}).to_string(),
        ),
        ("POST", "/v1/engine/kill", "{}".to_string()),
        ("POST", "/v1/engine/resume", "{}".to_string()),
        ("POST", "/v1/engine/shutdown", "{}".to_string()),
        (
            "POST",
            "/v1/cas/restore",
            serde_json::json!({"file_path": "README.md", "version_num": 1}).to_string(),
        ),
        (
            "POST",
            "/v1/governance/blueprints",
            serde_json::json!({
                "id": "bp-1",
                "name": "Test BP",
                "department": "eng",
                "description": "",
                "skills": "[]",
                "workflows": "[]",
                "mcp_tools": "[]",
                "requires_oversight": false
            })
            .to_string(),
        ),
        (
            "POST",
            "/v1/skills/mcp-tools/read_file/execute",
            "{}".to_string(),
        ),
        (
            "PUT",
            "/a2a/v1/profile",
            serde_json::json!({"business_name": "Unauthorized Mutation"}).to_string(),
        ),
        (
            "POST",
            "/a2a/v1/catalog/import",
            serde_json::json!({"csv_content": "title,category,price,description\nA,B,1.0,C\n"})
                .to_string(),
        ),
    ];

    for (method, uri, body) in cases {
        let mut builder = Request::builder()
            .method(method)
            .uri(uri)
            .header(header::AUTHORIZATION, &deploy_auth);
        if !body.is_empty() {
            builder = builder.header(header::CONTENT_TYPE, "application/json");
        }
        let req = builder.body(Body::from(body)).unwrap();
        let res = app.clone().oneshot(req).await.unwrap();
        assert_eq!(
            res.status(),
            StatusCode::FORBIDDEN,
            "Expected deploy_token to receive 403 FORBIDDEN on {} {}",
            method,
            uri
        );
    }

    // Positive admin checks + protected env var denylist verification
    let admin_auth = format!("Bearer {}", state.security.admin_token);
    let status_res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/system/backup/status")
                .header(header::AUTHORIZATION, &admin_auth)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(status_res.status(), StatusCode::OK);

    let env_poison_res = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/system/environment")
                .header(header::AUTHORIZATION, &admin_auth)
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    serde_json::json!({"variables": {"PATH": "/tmp/evil"}}).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(env_poison_res.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_plane2_fail_closed_runtime_guards() {
    let (app, state) = test_app().await;
    let auth = format!("Bearer {}", state.security.deploy_token);

    let mut plane2_agent = crate::agent::types::EngineAgent::default();
    plane2_agent.identity.id = "plane2-smb-agent".to_string();
    plane2_agent.identity.name = "SMB Twin".to_string();
    plane2_agent.identity.department = crate::agent::runner::PLANE2_DEPARTMENT_MARKER.to_string();
    plane2_agent.health.status = "idle".to_string();
    plane2_agent.economics.budget_usd = 10.0;
    state
        .registry
        .agents
        .insert(plane2_agent.identity.id.clone(), plane2_agent);

    // 1. Task dispatch route fails closed with 403
    let task_res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/agents/plane2-smb-agent/tasks")
                .header(header::AUTHORIZATION, &auth)
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    serde_json::json!({"message": "Import customer invoices"}).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(task_res.status(), StatusCode::FORBIDDEN);

    // 2. Mission dispatch route fails closed with 403
    let mission_res = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/agents/plane2-smb-agent/mission")
                .header(header::AUTHORIZATION, &auth)
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    serde_json::json!({"objective": "Process customer PII"}).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(mission_res.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_business_renew_facts_requires_post_and_get_is_readonly() {
    let (app, state) = test_app().await;
    let auth = format!("Bearer {}", state.security.deploy_token);

    let profile_override = state
        .base_dir
        .join("data")
        .join("business")
        .join("workspaces")
        .join("Mock-1-Product-profile.json");
    assert!(!profile_override.exists());

    // GET with legacy ?renew_facts=true query must remain strictly read-only
    let get_req = Request::builder()
        .method("GET")
        .uri("/v1/business/companies/Mock-1-Product?renew_facts=true")
        .header(header::AUTHORIZATION, &auth)
        .body(Body::empty())
        .unwrap();
    let get_res = app.clone().oneshot(get_req).await.unwrap();
    assert_eq!(get_res.status(), StatusCode::OK);
    assert!(
        !profile_override.exists(),
        "GET must never mutate profile disk state"
    );

    // POST /renew-facts performs the mutation explicitly
    let post_req = Request::builder()
        .method("POST")
        .uri("/v1/business/companies/Mock-1-Product/renew-facts")
        .header(header::AUTHORIZATION, &auth)
        .body(Body::empty())
        .unwrap();
    let post_res = app.oneshot(post_req).await.unwrap();
    assert_eq!(post_res.status(), StatusCode::OK);
    assert!(
        profile_override.exists(),
        "POST /renew-facts should persist profile override"
    );
}

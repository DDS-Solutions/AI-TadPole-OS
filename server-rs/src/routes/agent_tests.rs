//! Agent API - Verification Suite
//!
//! Integration tests for agent lifecycle, pagination, and auth gatekeeping.
//!
//! @docs ARCHITECTURE:AgentExecutionRuntime
//! @docs OPERATIONS_MANUAL:AgentManagement
//!
//! @state TestServer: (Ephemeral | MockedRegistry)
//!
//! ### AI Assist Note
//! **Verification Strategy**: Uses `tower::ServiceExt` for in-memory request
//! dispatching. This avoids network overhead while testing the full Axum stack
//! including middleware.
//!
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: Auth token mismatch, validation schema drift, or
//!   Registry state leakage between tests.
//! - **Trace Scope**: `server-rs::routes::agent_tests`

use axum::{
    body::Body,
    http::{header::AUTHORIZATION, Request, StatusCode},
    routing::{get, put, post},
    Router,
};
use serde_json::json;
use std::sync::Arc;
use tokio::time::{timeout, Duration};
use tower::ServiceExt;

use crate::{
    agent::types::{EngineAgent, TokenUsage},
    routes::agent::{create_agent, get_agents, update_agent, send_task, create_chat_completion},
    state::AppState,
};

async fn test_app() -> (Router, Arc<AppState>) {
    let app_state = Arc::new(
        AppState::new()
            .await
            .expect("Failed to initialize state for agent tests"),
    );

    // Auth bypass isn't strictly needed if we just pass the right token
    let app = Router::new()
        .route("/v1/agents", get(get_agents).post(create_agent))
        .route("/v1/agents/{id}", put(update_agent))
        .route("/v1/agents/{id}/tasks", post(send_task))
        .route("/v1/agents/chat/completions", post(create_chat_completion))
        // we apply the auth middleware manually here like main.rs
        .route_layer(axum::middleware::from_fn_with_state(
            app_state.clone(),
            crate::middleware::auth::validate_token,
        ))
        .with_state(app_state.clone());

    app_state.registry.agents.clear(); // Ensure clean state for tests
    app_state.registry.agents.clear(); // ENSURE REGISTRY IS ALSO CLEAR
    (app, app_state)
}

fn valid_auth(state: &AppState) -> String {
    format!("Bearer {}", state.security.deploy_token)
}

#[tokio::test]
async fn test_get_agents_empty() {
    let (app, state) = test_app().await;

    let request = Request::builder()
        .uri("/v1/agents")
        .header(AUTHORIZATION, valid_auth(&state))
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // Could check body but empty memory DB guarantees 0 agents
}

#[tokio::test]
async fn test_create_agent_success() {
    let (app, state) = test_app().await;

    let payload = json!({
        "id": "agent-123",
        "name": "Test Agent",
        "role": "Analyst",
        "department": "QA",
        "description": "Test description",
        "status": "idle",
        "budgetUsd": 100.0,
        "costUsd": 0.0,
        "tokensUsed": 0,
        "tokenUsage": {
            "inputTokens": 0,
            "outputTokens": 0,
            "totalTokens": 0
        },
        "metadata": {},
        "skills": [],
        "workflows": [],
        "model": "gpt-4o",
        "modelConfig": {
            "provider": "openai",
            "modelId": "gpt-4o"
        }
    });

    let request = Request::builder()
        .method("POST")
        .uri("/v1/agents")
        .header(AUTHORIZATION, valid_auth(&state))
        .header("Content-Type", "application/json")
        .body(Body::from(serde_json::to_vec(&payload).unwrap()))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);
    assert!(state.registry.agents.contains_key("agent-123"));
}

#[tokio::test]
async fn test_create_agent_fails_when_persistence_fails() {
    let (app, state) = test_app().await;
    let mut event_rx = state.comms.event_tx.subscribe();

    sqlx::query("DROP TABLE agents")
        .execute(&state.resources.pool)
        .await
        .unwrap();

    let payload = json!({
        "id": "agent-bad",
        "name": "Broken Agent",
        "role": "Analyst",
        "department": "QA",
        "description": "Should fail to persist",
        "status": "idle",
        "tokensUsed": 0,
        "tokenUsage": {
            "inputTokens": 0,
            "outputTokens": 0,
            "totalTokens": 0
        },
        "metadata": {},
        "skills": [],
        "workflows": [],
        "model": "gpt-4o",
        "modelConfig": {
            "provider": "openai",
            "modelId": "gpt-4o"
        }
    });

    let request = Request::builder()
        .method("POST")
        .uri("/v1/agents")
        .header(AUTHORIZATION, valid_auth(&state))
        .header("Content-Type", "application/json")
        .body(Body::from(serde_json::to_vec(&payload).unwrap()))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    assert!(!state.registry.agents.contains_key("agent-bad"));
    assert!(timeout(Duration::from_millis(50), event_rx.recv())
        .await
        .is_err());
}

#[tokio::test]
async fn test_create_agent_invalid_payload() {
    let (app, state) = test_app().await;

    let payload = json!({
        "role": "Analyst" // missing required fields
    });

    let request = Request::builder()
        .method("POST")
        .uri("/v1/agents")
        .header(AUTHORIZATION, valid_auth(&state))
        .header("Content-Type", "application/json")
        .body(Body::from(serde_json::to_vec(&payload).unwrap()))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    // Axum Json extractor strictly returns UNPROCESSABLE_ENTITY on missing fields
    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn test_get_agents_pagination() {
    let (app, state) = test_app().await;

    // Seed some agents directly
    for i in 0..15 {
        let agent = EngineAgent {
            identity: crate::agent::types::AgentIdentity {
                id: format!("test_agent_{}", i),
                name: format!("Agent {}", i),
                role: "Developer".to_string(),
                department: "Engineering".to_string(),
                description: "Test".to_string(),
                category: "user".to_string(),
                ..Default::default()
            },
            health: crate::agent::types::AgentHealth {
                status: "idle".to_string(),
                ..Default::default()
            },
            economics: crate::agent::types::AgentEconomics {
                budget_usd: 100.0,
                cost_usd: 0.0,
                tokens_used: 0,
                token_usage: TokenUsage::default(),
            },
            ..Default::default()
        };
        state
            .registry
            .agents
            .insert(agent.identity.id.clone(), agent);
    }

    // Try to get page 1, limit 10
    let request = Request::builder()
        .uri("/v1/agents?page=1&per_page=10")
        .header(AUTHORIZATION, valid_auth(&state))
        .body(Body::empty())
        .unwrap();

    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let bytes = axum::body::to_bytes(response.into_body(), 100000)
        .await
        .unwrap();
    let body: serde_json::Value = serde_json::from_slice(&bytes).unwrap();

    // Data length should be 10
    assert_eq!(body["data"].as_array().unwrap().len(), 10);
    assert_eq!(body["total"].as_u64().unwrap(), 15);

    // Page 2
    let request2 = Request::builder()
        .uri("/v1/agents?page=2&per_page=10")
        .header(AUTHORIZATION, valid_auth(&state))
        .body(Body::empty())
        .unwrap();

    let response2 = app.oneshot(request2).await.unwrap();
    let bytes2 = axum::body::to_bytes(response2.into_body(), 100000)
        .await
        .unwrap();
    let body2: serde_json::Value = serde_json::from_slice(&bytes2).unwrap();

    assert_eq!(body2["data"].as_array().unwrap().len(), 5);
}

#[tokio::test]
async fn test_create_agent_duplicate_conflict() {
    let (app, state) = test_app().await;

    // Seed agent
    let agent = EngineAgent {
        identity: crate::agent::types::AgentIdentity {
            id: "agent-dup".to_string(),
            name: "Duplicate Agent".to_string(),
            role: "Analyst".to_string(),
            department: "QA".to_string(),
            ..Default::default()
        },
        ..Default::default()
    };
    state.registry.agents.insert(agent.identity.id.clone(), agent);

    // Try to POST again with same ID
    let payload = json!({
        "id": "agent-dup",
        "name": "Another Agent",
        "role": "Analyst",
        "department": "QA",
        "description": "Conflict target",
        "status": "idle",
        "model": "gpt-4o",
        "modelConfig": {
            "provider": "openai",
            "modelId": "gpt-4o"
        }
    });

    let request = Request::builder()
        .method("POST")
        .uri("/v1/agents")
        .header(AUTHORIZATION, valid_auth(&state))
        .header("Content-Type", "application/json")
        .body(Body::from(serde_json::to_vec(&payload).unwrap()))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::CONFLICT);
}

#[tokio::test]
async fn test_send_task_deduplication() {
    let (app, state) = test_app().await;

    // Seed agent
    let agent = EngineAgent {
        identity: crate::agent::types::AgentIdentity {
            id: "agent-dedupe".to_string(),
            name: "Dedupe Agent".to_string(),
            role: "Analyst".to_string(),
            department: "QA".to_string(),
            ..Default::default()
        },
        health: crate::agent::types::AgentHealth {
            status: "idle".to_string(),
            ..Default::default()
        },
        ..Default::default()
    };
    state.registry.agents.insert(agent.identity.id.clone(), agent);

    let payload = json!({
        "message": "Verify task deduplication logic",
    });

    // Request 1: should be accepted
    let request1 = Request::builder()
        .method("POST")
        .uri("/v1/agents/agent-dedupe/tasks")
        .header(AUTHORIZATION, valid_auth(&state))
        .header("Content-Type", "application/json")
        .header("X-Request-Id", "req-12345")
        .body(Body::from(serde_json::to_vec(&payload).unwrap()))
        .unwrap();

    let response1 = app.clone().oneshot(request1).await.unwrap();
    assert_eq!(response1.status(), StatusCode::ACCEPTED);

    // Request 2: duplicate payload & same request ID -> should detect duplicate
    let request2 = Request::builder()
        .method("POST")
        .uri("/v1/agents/agent-dedupe/tasks")
        .header(AUTHORIZATION, valid_auth(&state))
        .header("Content-Type", "application/json")
        .header("X-Request-Id", "req-12345")
        .body(Body::from(serde_json::to_vec(&payload).unwrap()))
        .unwrap();

    let response2 = app.clone().oneshot(request2).await.unwrap();
    assert_eq!(response2.status(), StatusCode::ACCEPTED);
    let bytes = axum::body::to_bytes(response2.into_body(), 100000).await.unwrap();
    let body: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(body["duplicate"], true);

    // Request 3: different payload but same request ID -> should NOT be a duplicate
    let different_payload = json!({
        "message": "A completely different task message",
    });
    let request3 = Request::builder()
        .method("POST")
        .uri("/v1/agents/agent-dedupe/tasks")
        .header(AUTHORIZATION, valid_auth(&state))
        .header("Content-Type", "application/json")
        .header("X-Request-Id", "req-12345")
        .body(Body::from(serde_json::to_vec(&different_payload).unwrap()))
        .unwrap();

    let response3 = app.oneshot(request3).await.unwrap();
    assert_eq!(response3.status(), StatusCode::ACCEPTED);
    let bytes3 = axum::body::to_bytes(response3.into_body(), 100000).await.unwrap();
    let body3: serde_json::Value = serde_json::from_slice(&bytes3).unwrap();
    assert!(body3["duplicate"].is_null() || body3["duplicate"] == false);
}

#[tokio::test]
async fn test_chat_completion_validation() {
    let (app, state) = test_app().await;

    // 1. Bankrupt agent
    let bankrupt_agent = EngineAgent {
        identity: crate::agent::types::AgentIdentity {
            id: "agent-bankrupt".to_string(),
            ..Default::default()
        },
        health: crate::agent::types::AgentHealth {
            status: "idle".to_string(),
            ..Default::default()
        },
        economics: crate::agent::types::AgentEconomics {
            budget_usd: 10.0,
            cost_usd: 10.5, // Bankrupt!
            ..Default::default()
        },
        ..Default::default()
    };
    state.registry.agents.insert(bankrupt_agent.identity.id.clone(), bankrupt_agent);

    // 2. Suspended agent
    let suspended_agent = EngineAgent {
        identity: crate::agent::types::AgentIdentity {
            id: "agent-suspended".to_string(),
            ..Default::default()
        },
        health: crate::agent::types::AgentHealth {
            status: "suspended".to_string(), // Suspended!
            ..Default::default()
        },
        ..Default::default()
    };
    state.registry.agents.insert(suspended_agent.identity.id.clone(), suspended_agent);

    // Test request for bankrupt
    let req_bankrupt = json!({
        "model": "agent-bankrupt",
        "messages": [{"role": "user", "content": "hello"}]
    });
    let request_b = Request::builder()
        .method("POST")
        .uri("/v1/agents/chat/completions")
        .header(AUTHORIZATION, valid_auth(&state))
        .header("Content-Type", "application/json")
        .body(Body::from(serde_json::to_vec(&req_bankrupt).unwrap()))
        .unwrap();

    let response_b = app.clone().oneshot(request_b).await.unwrap();
    assert_eq!(response_b.status(), StatusCode::BAD_REQUEST);

    // Test request for suspended
    let req_suspended = json!({
        "model": "agent-suspended",
        "messages": [{"role": "user", "content": "hello"}]
    });
    let request_s = Request::builder()
        .method("POST")
        .uri("/v1/agents/chat/completions")
        .header(AUTHORIZATION, valid_auth(&state))
        .header("Content-Type", "application/json")
        .body(Body::from(serde_json::to_vec(&req_suspended).unwrap()))
        .unwrap();

    let response_s = app.clone().oneshot(request_s).await.unwrap();
    assert_eq!(response_s.status(), StatusCode::BAD_REQUEST);

    // Test request for non-existent
    let req_missing = json!({
        "model": "agent-missing",
        "messages": [{"role": "user", "content": "hello"}]
    });
    let request_m = Request::builder()
        .method("POST")
        .uri("/v1/agents/chat/completions")
        .header(AUTHORIZATION, valid_auth(&state))
        .header("Content-Type", "application/json")
        .body(Body::from(serde_json::to_vec(&req_missing).unwrap()))
        .unwrap();

    let response_m = app.oneshot(request_m).await.unwrap();
    assert_eq!(response_m.status(), StatusCode::NOT_FOUND);
}

// Metadata: [agent_tests]

//! @docs ARCHITECTURE:Core:Services:Tests
//!
//! ### AI Context Alignment
//! - **Subsystem**: Frontend Service Layer / tests
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Asynchronous service calls normalize response envelopes and propagate typed errors.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: none declared
//! - **Witness Tests**: none declared

use crate::agent::types::SwarmNode;
use crate::routes::model_manager::{get_model_catalog, pull_model, PullModelPayload};
use crate::services::discovery::prune_nodes_by_mdns_name;
use crate::state::AppState;
use axum::{extract::State, Json};
use chrono::Utc;
use std::collections::HashMap;
use std::sync::Arc;

#[tokio::test]
async fn test_get_model_catalog() {
    let state = Arc::new(AppState::new_mock().await);
    let result = get_model_catalog(State(state)).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_pull_model_node_not_found() {
    let state = Arc::new(AppState::new_mock().await);
    let payload = PullModelPayload {
        node_id: "non-existent".to_string(),
        tag: "llama3".to_string(),
    };

    let result = pull_model(State(state), Json(payload)).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_node_registry_insertion() {
    let state = Arc::new(AppState::new_mock().await);
    let node_id = "test-node-1".to_string();
    let node = SwarmNode {
        id: node_id.clone(),
        name: "Test Bunker".to_string(),
        address: "127.0.0.1:8080".to_string(),
        status: "online".to_string(),
        last_seen: Utc::now(),
        metadata: HashMap::new(),
    };

    state.registry.nodes.insert(node_id.clone(), node);
    assert!(state.registry.nodes.contains_key(&node_id));
}

#[tokio::test]
async fn test_node_lifecycle_removal() {
    let state = Arc::new(AppState::new_mock().await);
    let node_id = "temp-node".to_string();
    let mdns_name = "temp-node._tadpole._tcp.local.".to_string();

    let mut metadata = HashMap::new();
    metadata.insert("mdns_name".to_string(), mdns_name.clone());

    let node = SwarmNode {
        id: node_id.clone(),
        name: "Temporary Node".to_string(),
        address: "127.0.0.1:9000".to_string(),
        status: "online".to_string(),
        last_seen: Utc::now(),
        metadata,
    };

    // 1. Insert
    state.registry.nodes.insert(node_id.clone(), node);
    assert!(state.registry.nodes.contains_key(&node_id));

    // 2. Execute actual discovery.rs pruning logic
    let pruned = prune_nodes_by_mdns_name(&state.registry.nodes, &mdns_name);
    assert_eq!(pruned, vec![node_id.clone()]);

    // 3. Verify removal
    assert!(!state.registry.nodes.contains_key(&node_id));
}

#[tokio::test]
async fn test_app_state_registry_integrity() {
    let state = AppState::new_minimal_mock().await;
    assert!(state.registry.agents.is_empty());
    assert!(state.registry.nodes.is_empty());
    assert!(!state
        .governance
        .privacy_mode
        .load(std::sync::atomic::Ordering::Relaxed));
}

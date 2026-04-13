//! System Service Tests — Internal logic and state verification
//!
//! @docs ARCHITECTURE:State
//!
//! ### AI Assist Note
//! **Test Suite**: Verification of the core system services and state management.
//! Focuses on model catalog retrieval, node registry synchronization, and swarm-node lifecycle validation.
//!
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: AppState mock uninitialized, or model catalog schema mismatch.
//! - **Telemetry Link**: Search for `NodeRegistry` or `AppState` in backend tracing logs.

use crate::agent::types::SwarmNode;
use crate::routes::model_manager::{get_model_catalog, pull_model, PullModelPayload};
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

    // 2. Simulate discovery.rs removal logic
    let name_to_remove = mdns_name;
    let to_remove: Vec<String> = state
        .registry
        .nodes
        .iter()
        .filter(|entry| {
            entry.value().metadata.get("mdns_name").map(|n| n == &name_to_remove).unwrap_or(false)
        })
        .map(|entry| entry.key().clone())
        .collect();

    for id in to_remove {
        state.registry.nodes.remove(&id);
    }

    // 3. Verify removal
    assert!(!state.registry.nodes.contains_key(&node_id));
}

#[tokio::test]
async fn test_app_state_registry_integrity() {
    let state = AppState::new_mock().await;
    // Ensure hubs are correctly initialized
    assert!(state.registry.agents.is_empty());
    assert!(state.registry.nodes.is_empty());
    assert!(state.governance.privacy_mode.load(std::sync::atomic::Ordering::Relaxed) == false);
}

//! System Service Tests — Internal logic and state verification
//!
//! @docs ARCHITECTURE:State

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

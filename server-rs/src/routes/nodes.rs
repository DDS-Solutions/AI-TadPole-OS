use crate::agent::types::SwarmNode;
use crate::routes::error::AppError;
use crate::state::AppState;
use axum::{extract::State, Json};
use std::sync::Arc;

/// GET /v1/infra/nodes — Returns all registered Bunker nodes.
pub async fn get_nodes(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<SwarmNode>>, AppError> {
    let nodes: Vec<SwarmNode> = state
        .registry
        .nodes
        .iter()
        .map(|kv| kv.value().clone())
        .collect();
    Ok(Json(nodes))
}

/// POST /v1/infra/nodes/discover — Triggers a network discovery scan for new Bunkers.
/// For the prototype, this will simulate discovery by "finding" a new node if none exist beyond the defaults.
pub async fn discover_nodes(
    State(state): State<Arc<AppState>>,
) -> Result<Json<serde_json::Value>, AppError> {
    tracing::info!("🔍 Discovery scan initiated...");

    // Simulate finding a new node
    let new_id = "bunker-3";
    if !state.registry.nodes.contains_key(new_id) {
        state.registry.nodes.insert(
            new_id.to_string(),
            SwarmNode {
                id: new_id.to_string(),
                name: "Swarm Bunker 3 (Edge)".to_string(),
                address: "10.0.0.1".to_string(),
                status: "online".to_string(),
                last_seen: chrono::Utc::now(),
                metadata: std::collections::HashMap::from([(
                    "tier".to_string(),
                    "edge".to_string(),
                )]),
            },
        );

        state.broadcast_sys("New Bunker node discovered: Swarm Bunker 3", "success");

        Ok(Json(serde_json::json!({
            "status": "success",
            "discovered": ["bunker-3"]
        })))
    } else {
        Ok(Json(serde_json::json!({
            "status": "success",
            "discovered": []
        })))
    }
}

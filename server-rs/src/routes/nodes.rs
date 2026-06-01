//! @docs ARCHITECTURE:Networking
//!
//! ### AI Assist Note
//! **Distributed Nodes (Peer Discovery)**: Orchestrates the
//! infrastructure for discovering and managing physical Bunker nodes
//! within the local network swarm for the Tadpole OS engine.
//! Features **Dynamic Peer Discovery**: provides a simulated
//! discovery mechanism (`discover_nodes`) to expand the swarm
//! capacity at runtime. Implements **Swarm Metadata Management**:
//! maintains real-time status, network addresses, and tier-based
//! metadata for each registered node. AI agents should use these
//! endpoints to identify available execution environments and
//! balance mission loads across the distributed infrastructure
//! (NET-03).
//!
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: Duplicate node registration causing logic
//!   collisions, network timeout during peer discovery, or
//!   inaccessible node addresses causing mission routing failures.
//! - **Telemetry Link**: Search for `🔍 Discovery scan initiated` in
//!   `tracing` logs for swarm topology events.
//! - **Trace Scope**: `server-rs::routes::nodes`

use crate::agent::types::SwarmNode;

use crate::error::AppError;
use crate::state::AppState;
use axum::extract::State;
use axum::response::IntoResponse;
use axum::Json;
use std::sync::Arc;

/// GET /v1/infra/nodes — Returns all registered Bunker nodes.
#[tracing::instrument(skip(state), name = "infra_nodes::get_all")]
pub async fn get_nodes(State(state): State<Arc<AppState>>) -> Result<impl IntoResponse, AppError> {
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
#[tracing::instrument(skip(state), name = "infra_nodes::discover")]
pub async fn discover_nodes(
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, AppError> {
    tracing::info!("🔍 Discovery scan initiated...");

    // Simulate finding a new node
    let new_id = "bunker_3";
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

        state.broadcast_sys(
            "New Bunker node discovered: Swarm Bunker 3",
            "success",
            None,
        );

        Ok(Json(serde_json::json!({
            "status": "success",
            "discovered": ["bunker_3"]
        })))
    } else {
        Ok(Json(serde_json::json!({
            "status": "success",
            "discovered": []
        })))
    }
}

// Metadata: [nodes]

// Metadata: [nodes]

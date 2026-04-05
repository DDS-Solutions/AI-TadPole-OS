//! @docs ARCHITECTURE:Networking
//! 
//! ### AI Assist Note
//! **Engine Governance (Lifecycle Manager)**: Orchestrates the 
//! emergency kill switches, graceful shutdowns, and global state 
//! persistence for the Tadpole OS engine. Features **Emergency Kill 
//! Switch**: provides a centralized mechanism to halt all active 
//! agent processing and clear the oversight queue during systemic 
//! crises. Implements **Graceful Shutdown Protocols**: ensures 
//! that all agent states and mission histories are durable 
//! persistent to SQLite before terminating the process. AI agents 
//! should monitor the `engine:kill` and `engine:shutdown` events to 
//! handle disconnection and state resumption (GOV-05).
//! 
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: Partial agent halting due to registry lock 
//!   contention, database write failures during shutdown, or zombie 
//!   processes if the delayed exit fails to trigger.
//! - **Telemetry Link**: Search for `🛑 [Kill Switch]` or `💀 [Shutdown]` 
//!   in `tracing` logs for governance audit events.
//! - **Trace Scope**: `server-rs::routes::engine_control`

use crate::routes::error::AppError;

use crate::state::AppState;
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use std::sync::Arc;

/// POST /v1/engine/kill
///
/// Emergency kill switch that halts all active swarm processing.
/// Sets every agent's status to "idle" and clears their active missions.
///
/// @docs OPERATIONS_MANUAL:EmergencyKill
pub async fn kill_agents(
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, AppError> {
    let mut halted = 0usize;

    for mut entry in state.registry.agents.iter_mut() {
        if entry.status == "active"
            || entry.status == "thinking"
            || entry.status == "coding"
            || entry.status == "speaking"
        {
            entry.status = "idle".to_string();
            entry.active_mission = None;
            halted += 1;
        }
    }

    // Abort all pending oversight entries — no point waiting for approval on halted agents
    let pending_ids: Vec<String> = state
        .comms
        .oversight_queue
        .iter()
        .map(|e| e.key().clone())
        .collect();
    for id in &pending_ids {
        state.comms.oversight_queue.remove(id);
        if let Some((_, resolver)) = state.comms.oversight_resolvers.remove(id) {
            let _ = resolver.send(false); // reject
        }
    }

    tracing::warn!(
        "🛑 [Kill Switch] Halted {} agents, cleared {} pending oversight entries.",
        halted,
        pending_ids.len()
    );

    state.emit_event(serde_json::json!({
        "type": "engine:kill",
        "haltedAgents": halted,
        "clearedOversight": pending_ids.len()
    }));

    Ok((
        StatusCode::OK,
        Json(serde_json::json!({
            "status": "ok",
            "halted": halted,
            "clearedOversight": pending_ids.len()
        })),
    ))
}

/// POST /engine/shutdown — Graceful server shutdown.
///
/// Persists all agent state to the database and then terminates the process.
/// The caller should expect the connection to drop after receiving the response.
pub async fn shutdown_engine(
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, AppError> {
    tracing::warn!("💀 [Shutdown] Engine shutdown requested by operator. Persisting state...");

    // Save all agents before shutting down
    state.save_agents().await;

    state.emit_event(serde_json::json!({
        "type": "engine:shutdown",
        "message": "Engine shutting down. Goodbye."
    }));

    // Spawn a delayed exit so the response can be sent first
    tokio::spawn(async {
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        tracing::info!("👋 Engine process exiting.");
        std::process::exit(0);
    });

    Ok((
        StatusCode::OK,
        Json(serde_json::json!({
            "status": "ok",
            "message": "Shutdown initiated. State persisted."
        })),
    ))
}

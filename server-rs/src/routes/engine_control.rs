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

use crate::state::AppState;
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use std::sync::Arc;

/// POST /v1/engine/kill
///
/// Emergency kill switch that halts all active swarm processing.
/// Sets every agent's status to "idle" and clears their active missions.
///
/// @docs OPERATIONS_MANUAL:EmergencyKill
#[tracing::instrument(skip(state), fields(trace_id = tracing::field::Empty, request_id = tracing::field::Empty, http_status = tracing::field::Empty), name = "governance::kill_swarm")]
pub async fn kill_agents(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let mut halted = 0usize;

    for mut entry in state.registry.agents.iter_mut() {
        let current_status = entry.health.status.as_str();
        if current_status == "active"
            || current_status == "thinking"
            || current_status == "coding"
            || current_status == "speaking"
        {
            entry.health.status = "idle".to_string();
            entry.state.active_mission = None;
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
            let _ = resolver.send(crate::agent::types::OversightResolution {
                approved: false,
                override_slot: None,
            }); // reject
        }
    }

    tracing::warn!(
        "🛑 [Kill Switch] Halted {} agents, cleared {} pending oversight entries.",
        halted,
        pending_ids.len()
    );

    state.emit_event(serde_json::json!({
        "type": "engine:kill",
        "halted_agents": halted,
        "cleared_oversight": pending_ids.len()
    }));

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "status": "ok",
            "halted_agents": halted,
            "cleared_oversight": pending_ids.len()
        })),
    )
}

/// POST /engine/shutdown — Graceful server shutdown.
///
/// Persists all agent state to the database and then terminates the process.
/// The caller should expect the connection to drop after receiving the response.
#[tracing::instrument(skip(state), fields(trace_id = tracing::field::Empty, request_id = tracing::field::Empty, http_status = tracing::field::Empty), name = "governance::shutdown")]
pub async fn shutdown_engine(State(state): State<Arc<AppState>>) -> impl IntoResponse {
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

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "status": "ok",
            "message": "Shutdown initiated. State persisted."
        })),
    )
}

/// GET /v1/engine/mirror/status
///
/// Returns current mirror mode status and drift alerts.
///
/// @docs OPERATIONS_MANUAL:MirrorStatus
#[tracing::instrument(skip(state), fields(trace_id = tracing::field::Empty, request_id = tracing::field::Empty, http_status = tracing::field::Empty), name = "governance::mirror_status")]
pub async fn get_mirror_status(
    State(state): State<Arc<AppState>>,
) -> (StatusCode, Json<serde_json::Value>) {
    let alerts: Vec<serde_json::Value> = state
        .drift_alerts
        .iter()
        .map(|entry| entry.value().clone())
        .collect();

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "mirror_mode": state.mirror_mode,
            "drift_alerts": alerts
        })),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::AppState;
    use axum::extract::State;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_get_mirror_status_default() {
        let state = Arc::new(AppState::new_mock().await);
        let (status, Json(body)) = get_mirror_status(State(state)).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(body["mirror_mode"], serde_json::Value::Bool(false));
        assert!(body["drift_alerts"].as_array().unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_get_mirror_status_with_alerts() {
        let state = Arc::new(AppState::new_mock().await);
        let alert = serde_json::json!({
            "id": "alert-1",
            "timestamp": "2026-07-10T17:40:00Z",
            "agent_id": "assistant",
            "alert_type": "mutating_tool_blocked",
            "detail": "Blocked write_file"
        });
        state
            .drift_alerts
            .insert("alert-1".to_string(), alert.clone());

        let (status, Json(body)) = get_mirror_status(State(state)).await;
        assert_eq!(status, StatusCode::OK);
        let alerts = body["drift_alerts"].as_array().unwrap();
        assert_eq!(alerts.len(), 1);
        assert_eq!(alerts[0]["id"], "alert-1");
    }
}

// Metadata: [engine_control]

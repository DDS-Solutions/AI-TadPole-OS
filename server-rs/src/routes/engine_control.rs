//! @docs ARCHITECTURE:Networking
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / HTTP Routes / engine_control
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Type-safe state handling and bounded execution without unhandled panics.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: `AppError::InternalServerError`
//! - **Telemetry Targets**: none declared
//! - **Witness Tests**: `engine_control::tests::test_kill_agents_halts_and_persists`

use crate::state::AppState;
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use std::sync::Arc;

/// Helper to abort in-flight agent tasks and reset their active state in memory.
pub(crate) async fn halt_active_agents(state: &AppState) -> (usize, Vec<String>) {
    let mut halted = 0usize;
    let mut interrupted_missions = Vec::new();

    for mut entry in state.registry.agents.iter_mut() {
        let current_status = entry.health.status.as_str();
        if current_status == "active"
            || current_status == "thinking"
            || current_status == "coding"
            || current_status == "speaking"
        {
            if let Some(mission) = &entry.state.active_mission {
                let m_id = if let Some(m_str) = mission.as_str() {
                    m_str.to_string()
                } else if let Some(m_obj) = mission.get("id").and_then(|v| v.as_str()) {
                    m_obj.to_string()
                } else {
                    mission.to_string()
                };
                if !m_id.is_empty() && !interrupted_missions.contains(&m_id) {
                    interrupted_missions.push(m_id);
                }
            }
            entry.health.status = "idle".to_string();
            entry.state.active_mission = None;
            entry.state.current_task = None;
            halted += 1;
        }
    }

    // Abort all in-flight AgentRunner task handles
    let runner_keys: Vec<String> = state
        .comms
        .active_runners
        .iter()
        .map(|e| e.key().clone())
        .collect();

    for key in runner_keys {
        if let Some((_, handle)) = state.comms.active_runners.remove(&key) {
            handle.abort_handle.abort();
        }
    }

    (halted, interrupted_missions)
}

/// POST /v1/engine/kill
///
/// Emergency kill switch that halts all active swarm processing.
/// Sets every agent's status to "idle", clears active missions/tasks,
/// aborts in-flight task handles, and durably persists the state.
///
/// @docs OPERATIONS_MANUAL:EmergencyKill
#[tracing::instrument(skip(state), name = "governance::kill_swarm")]
pub async fn kill_agents(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let (halted, interrupted_missions) = halt_active_agents(&state).await;

    // Abort all pending oversight entries
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
                user_answer: None,
            });
        }
    }

    let now = chrono::Utc::now();
    if let Err(error) = sqlx::query(
        "UPDATE oversight_log SET status = 'rejected', decision = 'aborted', decided_at = ?, decided_by = 'system_kill' WHERE status = 'pending'",
    )
    .bind(now)
    .execute(&state.resources.pool)
    .await
    {
        tracing::error!(
            "Failed to persist kill-switch cleanup for pending oversight entries: {}",
            error
        );
    }

    // Durably persist the idle statuses so they do not resurrect on reboot
    if let Err(e) = state.save_agents().await {
        tracing::error!("❌ [Kill Switch] Failed to persist agent statuses: {}", e);
    }

    tracing::warn!(
        "🛑 [Kill Switch] Halted {} agents across {:?} missions, cleared {} pending oversight entries.",
        halted,
        interrupted_missions,
        pending_ids.len()
    );

    state.emit_event(serde_json::json!({
        "type": "engine:kill",
        "halted_agents": halted,
        "interrupted_missions": interrupted_missions,
        "cleared_oversight": pending_ids.len(),
        "timestamp": chrono::Utc::now().to_rfc3339()
    }));

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "status": "ok",
            "halted_agents": halted,
            "interrupted_missions": interrupted_missions,
            "cleared_oversight": pending_ids.len()
        })),
    )
}

/// POST /engine/shutdown — Graceful server shutdown.
///
/// Persists all agent state to the database and then terminates the process.
/// The caller should expect the connection to drop after receiving the response.
#[tracing::instrument(skip(state), name = "governance::shutdown")]
pub async fn shutdown_engine(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    tracing::warn!("💀 [Shutdown] Engine shutdown requested by operator. Persisting state...");

    // 1. Halt running agents and abort task handles to prevent in-flight state divergence
    let (halted, interrupted_missions) = halt_active_agents(&state).await;
    tracing::info!(
        "💀 [Shutdown] Halted {} running agent tasks across missions: {:?} prior to shutdown",
        halted,
        interrupted_missions
    );

    // 2. Save all agents before shutting down
    let save_ok = state.save_agents().await.is_ok();

    // 3. Flush database WAL journal checkpoint
    if let Err(e) = sqlx::query("PRAGMA wal_checkpoint(TRUNCATE)")
        .execute(&state.resources.pool)
        .await
    {
        tracing::warn!(
            "⚠️ [Shutdown] WAL checkpoint before exit encountered error: {}",
            e
        );
    }

    state.emit_event(serde_json::json!({
        "type": "engine:shutdown",
        "message": "Engine shutting down. Goodbye.",
        "halted_agents": halted,
        "interrupted_missions": interrupted_missions,
        "timestamp": chrono::Utc::now().to_rfc3339()
    }));

    // Spawn a delayed exit so the response can be sent first
    tokio::spawn(async {
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        tracing::info!("👋 Engine process exiting.");
        std::process::exit(0);
    });

    let shutdown_message = if save_ok {
        "Shutdown initiated. State persisted."
    } else {
        "Shutdown initiated. State persistence failed; check logs."
    };

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "status": "ok",
            "message": shutdown_message,
            "halted_agents": halted,
            "interrupted_missions": interrupted_missions
        })),
    )
}

/// GET /v1/engine/mirror/status
///
/// Returns current mirror mode status and drift alerts.
///
/// @docs OPERATIONS_MANUAL:MirrorStatus
#[tracing::instrument(skip(state), name = "governance::mirror_status")]
pub async fn get_mirror_status(
    State(state): State<Arc<AppState>>,
) -> (StatusCode, Json<serde_json::Value>) {
    let mut alerts: Vec<serde_json::Value> = state
        .drift_alerts
        .iter()
        .map(|entry| entry.value().clone())
        .collect();

    // Sort by timestamp descending
    alerts.sort_by(|a, b| {
        let ts_a = a.get("timestamp").and_then(|v| v.as_str()).unwrap_or("");
        let ts_b = b.get("timestamp").and_then(|v| v.as_str()).unwrap_or("");
        ts_b.cmp(ts_a)
    });

    alerts.truncate(100);

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
        let state = Arc::new(AppState::new_minimal_mock().await);
        let (status, Json(body)) = get_mirror_status(State(state)).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(body["mirror_mode"], serde_json::Value::Bool(false));
        assert!(body["drift_alerts"].as_array().unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_kill_agents_halts_and_persists() {
        let state = Arc::new(AppState::new_minimal_mock().await);

        let mut agent = crate::agent::types::EngineAgent::default();
        agent.identity.id = "test-running-agent".to_string();
        agent.health.status = "active".to_string();
        agent.state.active_mission = Some(serde_json::json!("mission-123"));
        agent.state.current_task = Some("task-abc".to_string());
        state
            .registry
            .agents
            .insert("test-running-agent".to_string(), agent);

        let response = kill_agents(State(state.clone())).await.into_response();
        assert_eq!(response.status(), StatusCode::OK);

        let agent_after = state.registry.agents.get("test-running-agent").unwrap();
        assert_eq!(agent_after.health.status, "idle");
        assert!(agent_after.state.active_mission.is_none());
        assert!(agent_after.state.current_task.is_none());
    }
}

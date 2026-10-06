//! @docs ARCHITECTURE:Gateways
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / HTTP Routes / Agent Tasks
//! - **Primary Entrypoints**: `send_task`, `spawn_agent_runner`, `register_agent_runner`, `validate_agent_preflight`, `claim_task_request`
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: `AppError`
//! - **Telemetry Targets**: `[Gateway]`
//! - **Witness Tests**: `test_parse_traceparent_valid_v00`, `test_claim_task_request_deduplication`, `test_validate_agent_preflight_budget_scrubbing`

use super::models::{DEDUP_CACHE_PRUNE_SECS, DEDUP_WINDOW_SECS, STATUS_SUSPENDED};
use crate::{
    agent::{runner::AgentRunner, types::TaskPayload},
    error::AppError,
    state::AppState,
};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Parses and validates a W3C traceparent context header or payload string.
/// Format: {version}-{trace_id}-{parent_id}-{trace_flags}
/// Enforces W3C Trace Context v1.0 specifications:
/// - Version 2 hex chars, 'ff' is forbidden
/// - Trace ID 32 hex chars, all-zero forbidden
/// - Parent ID 16 hex chars, all-zero forbidden
/// - Flags 2 hex chars
/// - Version 00 must be exactly 55 characters
pub fn parse_traceparent(raw: &str) -> Option<String> {
    let s = raw.trim();
    let parts: Vec<&str> = s.split('-').collect();
    if parts.len() < 4 {
        return None;
    }
    let version = parts[0];
    let trace_id = parts[1];
    let parent_id = parts[2];
    let flags = parts[3];

    if version.len() != 2 || version == "ff" || !version.chars().all(|c| c.is_ascii_hexdigit()) {
        return None;
    }
    if trace_id.len() != 32
        || trace_id == "00000000000000000000000000000000"
        || !trace_id.chars().all(|c| c.is_ascii_hexdigit())
    {
        return None;
    }
    if parent_id.len() != 16
        || parent_id == "0000000000000000"
        || !parent_id.chars().all(|c| c.is_ascii_hexdigit())
    {
        return None;
    }
    if flags.len() != 2 || !flags.chars().all(|c| c.is_ascii_hexdigit()) {
        return None;
    }
    if version == "00" && (parts.len() != 4 || s.len() != 55) {
        return None;
    }

    Some(s.to_string())
}

pub fn claim_task_request(
    requests: &dashmap::DashMap<String, Instant>,
    key: &str,
    now: Instant,
) -> bool {
    use dashmap::mapref::entry::Entry;
    match requests.entry(key.to_owned()) {
        Entry::Occupied(mut entry) => {
            if now.saturating_duration_since(*entry.get()) < Duration::from_secs(DEDUP_WINDOW_SECS)
            {
                return false;
            }
            entry.insert(now);
        }
        Entry::Vacant(entry) => {
            entry.insert(now);
        }
    }
    true
}

struct ActiveRunnerCleanupGuard {
    state: Arc<AppState>,
    agent_id: String,
    task_id: String,
}

impl Drop for ActiveRunnerCleanupGuard {
    fn drop(&mut self) {
        self.state
            .comms
            .active_runners
            .remove_if(&self.agent_id, |_, stored_runner| {
                stored_runner.task_id == self.task_id
            });
    }
}

/// Canonical runner spawn. Creates a background `AgentRunner` task with
/// semaphore throttling, error masking (F-04), RFC 9457 failure events, and
/// auto-cleanup keyed by task ID (F-03).
pub fn spawn_agent_runner(
    state: &Arc<AppState>,
    agent_id: &str,
    payload: TaskPayload,
) -> (
    tokio::task::JoinHandle<()>,
    crate::state::hubs::comm::RunnerHandle,
    tokio::sync::oneshot::Sender<()>,
) {
    let state_clone = state.clone();
    let task_id = uuid::Uuid::new_v4().to_string();
    let task_id_for_cleanup = task_id.clone();
    let agent_id_for_cleanup = agent_id.to_string();
    let (start_tx, start_rx) = tokio::sync::oneshot::channel();

    let join_handle = tokio::spawn(async move {
        let _cleanup_guard = ActiveRunnerCleanupGuard {
            state: state_clone.clone(),
            agent_id: agent_id_for_cleanup.clone(),
            task_id: task_id_for_cleanup.clone(),
        };

        if start_rx.await.is_err() {
            return;
        }

        // Acquire permit before running to throttle concurrency
        let _permit = match state_clone.comms.runner_semaphore.acquire().await {
            Ok(p) => p,
            Err(_) => {
                tracing::error!("❌ [Runner] Semaphore closed");
                let is_current_runner = state_clone
                    .comms
                    .active_runners
                    .get(&agent_id_for_cleanup)
                    .map(|stored| stored.task_id == task_id_for_cleanup)
                    .unwrap_or(false);
                if is_current_runner {
                    AgentRunner::new(state_clone.clone()).update_status(
                        &agent_id_for_cleanup,
                        &task_id_for_cleanup,
                        "idle",
                        None,
                    );
                    if let Err(e) = state_clone.save_agents().await {
                        tracing::warn!("Failed to persist agent status in runner fallback: {}", e);
                    }
                }
                state_clone.emit_event(serde_json::json!({
                    "type": "agent:task_failed",
                    "agent_id": agent_id_for_cleanup,
                    "error": {
                        "type": "urn:tadpole:error:runner_unavailable",
                        "title": "RUNNER UNAVAILABLE",
                        "status": 503,
                        "detail": "Agent runner is unavailable; retry the task.",
                        "error_code": "RUNNER_UNAVAILABLE"
                    }
                }));
                return;
            }
        };
        let runner = AgentRunner::new(state_clone.clone());
        if let Err(e) = runner.run(agent_id_for_cleanup.clone(), payload).await {
            tracing::error!("❌ [Runner] Agent {} failed: {}", agent_id_for_cleanup, e);

            // Mask/redact detailed error for client security (F-04)
            let masked_detail = if e.status_code().as_u16() >= 500 {
                "Internal Server Error. Please inspect server traces.".to_string()
            } else {
                e.to_string()
            };

            // Async Failure Feedback with structured RFC 9457 support
            let error_data = serde_json::json!({
                "type": e.type_slug(),
                "title": e.type_slug().replace(['-', ':'], " ").to_uppercase(),
                "status": e.status_code().as_u16(),
                "detail": masked_detail,
                "error_code": e.type_slug().to_uppercase()
            });

            state_clone.emit_event(serde_json::json!({
                "type": "agent:task_failed",
                "agent_id": agent_id_for_cleanup.clone(),
                "error": error_data
            }));
        }

        // `_cleanup_guard` removes only this task's entry, including cancellation paths.
    });

    let runner_handle = crate::state::hubs::comm::RunnerHandle {
        abort_handle: join_handle.abort_handle(),
        task_id,
    };

    (join_handle, runner_handle, start_tx)
}

/// Atomically publishes a runner handle before allowing its task to execute.
/// Replacing an overlapping dispatch cancels and closes the previous mission first.
pub async fn register_agent_runner(
    state: &Arc<AppState>,
    agent_id: &str,
    runner_handle: crate::state::hubs::comm::RunnerHandle,
    start_tx: tokio::sync::oneshot::Sender<()>,
) {
    let task_id = runner_handle.task_id.clone();
    let previous = state
        .comms
        .active_runners
        .insert(agent_id.to_string(), runner_handle);

    if let Some(previous_runner) = previous {
        previous_runner.abort_handle.abort();
        let previous_mission_id = state.registry.agents.get(agent_id).and_then(|agent| {
            agent
                .state
                .active_mission
                .as_ref()
                .and_then(|mission| mission.get("id"))
                .and_then(serde_json::Value::as_str)
                .map(str::to_string)
        });

        if let Some(mission_id) = previous_mission_id.as_deref() {
            if let Err(error) = sqlx::query(
                "UPDATE mission_history SET status = 'failed', updated_at = CURRENT_TIMESTAMP WHERE id = ? AND agent_id = ? AND status IN ('pending', 'active')",
            )
            .bind(mission_id)
            .bind(agent_id)
            .execute(&state.resources.pool)
            .await
            {
                tracing::warn!(
                    "Failed to mark replaced mission {} for agent {}: {}",
                    mission_id,
                    agent_id,
                    error
                );
            }
        }

        let is_current_runner = state
            .comms
            .active_runners
            .get(agent_id)
            .map(|stored| stored.task_id == task_id)
            .unwrap_or(false);
        if is_current_runner {
            AgentRunner::new(state.clone()).update_status(
                agent_id,
                previous_mission_id
                    .as_deref()
                    .unwrap_or(&previous_runner.task_id),
                "idle",
                None,
            );
            if let Err(e) = state.save_agents().await {
                tracing::warn!(
                    "Failed to persist agent status in runner replacement: {}",
                    e
                );
            }
        }
    }

    let is_current_runner = state
        .comms
        .active_runners
        .get(agent_id)
        .map(|stored| stored.task_id == task_id)
        .unwrap_or(false);
    if is_current_runner {
        let _ = start_tx.send(());
    }
}

/// Helper to validate agent status/budget and extract traceparent from headers.
pub fn validate_agent_preflight(
    state: &Arc<AppState>,
    agent_id: &str,
    headers: &axum::http::HeaderMap,
    payload: &mut TaskPayload,
) -> Result<(), AppError> {
    // Auth, Existence & Budget Check (F-06)
    match state.registry.agents.get(agent_id) {
        None => {
            return Err(AppError::NotFound(format!(
                "Agent '{}' not found",
                agent_id
            )))
        }
        Some(agent) => {
            if agent.health.status == STATUS_SUSPENDED {
                return Err(AppError::BadRequest(format!(
                    "Agent '{}' is currently suspended.",
                    agent_id
                )));
            }
            if agent.is_bankrupt() {
                tracing::warn!(
                    "Agent '{}' preflight rejected: budget exhausted (cost: ${}, budget: ${})",
                    agent_id,
                    agent.economics.cost_usd,
                    agent.economics.budget_usd
                );
                return Err(AppError::BadRequest(format!(
                    "Agent '{}' has exhausted its allocated budget.",
                    agent_id
                )));
            }
        }
    }

    // Forward and validate traceparent for distributed tracing (uniform validation for header & body)
    if let Some(tp) = payload.traceparent.as_ref() {
        if let Some(validated) = parse_traceparent(tp) {
            payload.traceparent = Some(validated);
        } else {
            tracing::warn!("Blocked invalid traceparent in payload body");
            payload.traceparent = None;
        }
    } else if let Some(tp) = headers.get("traceparent").and_then(|v| v.to_str().ok()) {
        if let Some(validated) = parse_traceparent(tp) {
            payload.traceparent = Some(validated);
        } else {
            tracing::warn!("Blocked invalid traceparent header");
        }
    }

    Ok(())
}

/// POST /v1/agents/:id/tasks
///
/// Dispatches a high-level text task to a specific autonomous agent.
/// Automatically handles distributed trace propagation (via W3C `traceparent`)
/// and validates agent existence before dispatch.
/// Identical payloads for the same agent and `X-Request-Id` are accepted once
/// per 15-second window, including concurrent deliveries. Duplicates return
/// HTTP 202 with `duplicate: true` and do not start another runner.
///
/// ### 🔦 Distributed Tracing (AGNT-01)
/// If a `traceparent` header is present in the UI request, it is parsed
/// and injected into the mission payload. This ensures that the engine's
/// background `AgentRunner` spans are correctly linked to the front-end
/// session in our Jaeger/OTel traces.
///
/// @docs API_REFERENCE:SendTask
#[tracing::instrument(skip(state, headers, payload), fields(agent_id = %agent_id), name = "agent_gateway::dispatch")]
pub async fn send_task(
    Path(agent_id): Path<String>,
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Json(mut payload): Json<TaskPayload>,
) -> Result<impl IntoResponse, AppError> {
    // 1. Run agent preflight check first
    validate_agent_preflight(&state, &agent_id, &headers, &mut payload)?;

    // 2. Backend Request Deduplication using X-Request-Id header (run after validation)
    if let Some(req_id_val) = headers.get("x-request-id").and_then(|v| v.to_str().ok()) {
        let req_id = req_id_val.to_string();
        let now = Instant::now();

        // Proactively prune expired entries (>30 seconds old) using exact duration
        state
            .comms
            .recent_requests
            .retain(|_, time| time.elapsed() < Duration::from_secs(DEDUP_CACHE_PRUNE_SECS));

        // Serialize payload to hash it (TaskPayload does not implement Hash directly)
        let serialized_payload = serde_json::to_string(&payload).map_err(|e| {
            AppError::BadRequest(format!(
                "Failed to serialize task payload for deduplication: {}",
                e
            ))
        })?;
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        use std::hash::{Hash, Hasher};
        serialized_payload.hash(&mut hasher);
        let payload_hash = hasher.finish();

        let dedupe_key = format!("{}:{}:{:x}", agent_id, req_id, payload_hash);

        if !claim_task_request(&state.comms.recent_requests, &dedupe_key, now) {
            tracing::info!(
                "🛑 [Gateway] Duplicate task request detected for X-Request-Id: {} (key: {}). Skipping execution.",
                req_id, dedupe_key
            );
            return Ok((
                StatusCode::ACCEPTED,
                Json(serde_json::json!({
                    "status": "accepted",
                    "agent_id": agent_id,
                    "duplicate": true
                })),
            ));
        }
    }

    tracing::info!("📡 [Gateway] Task dispatched to Agent {}", agent_id);

    // 3. Auto-Inject Socratic Context Envelope (Mandatory Reasoning Integrity Gate)
    if payload.skip_socratic_gate == Some(true) {
        tracing::warn!(
            "⚠️ [Security] Client requested skip_socratic_gate for agent {}. Mandatory governance applied.",
            agent_id
        );
    }

    let (agent_name, agent_role, budget, active_slot, agent_cluster) =
        if let Some(agent_entry) = state.registry.agents.get(&agent_id) {
            let a = agent_entry.value();
            (
                a.identity.name.clone(),
                a.identity.role.clone(),
                Some(a.economics.budget_usd),
                a.models.active_model_slot.as_deref().map(|s| match s {
                    "2" | "execution" => 2,
                    "3" | "planning" => 3,
                    _ => 1,
                }),
                a.metadata
                    .get("cluster_id")
                    .and_then(|v| v.as_str())
                    .map(str::to_string),
            )
        } else {
            (
                agent_id.clone(),
                "General Intelligence Node".to_string(),
                None,
                None,
                None,
            )
        };

    // Fail-closed privacy: active if either agent's cluster, requested cluster, or global setting demands it
    let is_privacy = state
        .governance
        .is_privacy_mode_enabled(agent_cluster.as_deref())
        || state
            .governance
            .is_privacy_mode_enabled(payload.cluster_id.as_deref());

    // Sanitize context files / allowed files against base directory boundary (prevent path traversal)
    let raw_files = payload
        .allowed_files
        .clone()
        .or_else(|| payload.context_files.clone());
    let sanitized_files = raw_files.map(|files| {
        files
            .into_iter()
            .filter_map(|f| {
                match crate::security::path_guard::validate_path(&state.base_dir, &f) {
                    Ok(safe) => Some(safe.as_path().to_string_lossy().to_string()),
                    Err(err) => {
                        tracing::warn!(
                            "⚠️ [Security] Filtered invalid or escaping file path in task context '{}': {}",
                            f,
                            err
                        );
                        None
                    }
                }
            })
            .collect::<Vec<String>>()
    });

    let envelope = crate::agent::socratic::SocraticContextEnvelope::compile(
        &agent_id,
        &agent_name,
        &agent_role,
        payload
            .primary_goal
            .as_deref()
            .unwrap_or("Autonomous Task Execution"),
        sanitized_files,
        budget,
        active_slot,
        is_privacy,
    );
    payload.message = envelope.inject_into_prompt(&payload.message);

    // Proactive Abort-on-New Policy: Terminate any existing task for this agent
    if let Some((_, runner)) = state.comms.active_runners.remove(&agent_id) {
        tracing::info!(
            "🔄 [Gateway] Aborting existing task for agent {} to prioritize new request.",
            agent_id
        );
        runner.abort_handle.abort();

        let aborted_mission_id = state.registry.agents.get(&agent_id).and_then(|agent| {
            agent
                .state
                .active_mission
                .as_ref()
                .and_then(|mission| mission.get("id"))
                .and_then(serde_json::Value::as_str)
                .map(str::to_string)
        });

        // Finish the old mission update before the replacement task can create a new active row.
        let aborted_task_id = runner.task_id;
        if let Some(mission_id) = aborted_mission_id.as_deref() {
            if let Err(error) = sqlx::query(
                "UPDATE mission_history SET status = 'failed', updated_at = CURRENT_TIMESTAMP WHERE id = ? AND agent_id = ? AND status IN ('pending', 'active')",
            )
            .bind(mission_id)
            .bind(&agent_id)
            .execute(&state.resources.pool)
            .await
            {
                tracing::warn!(
                    "Failed to mark aborted mission {} (task {}) for agent {}: {}",
                    mission_id,
                    aborted_task_id,
                    agent_id,
                    error
                );
            }
        }

        AgentRunner::new(state.clone()).update_status(
            &agent_id,
            aborted_mission_id.as_deref().unwrap_or(&aborted_task_id),
            "idle",
            None,
        );
        if let Err(e) = state.save_agents().await {
            tracing::warn!("Failed to persist aborted agent status in dispatch: {}", e);
        }
    }

    // Spawn Runner via canonical helper (F-03, #6)
    let (join_handle, runner_handle, start_tx) = spawn_agent_runner(&state, &agent_id, payload);
    let task_id = runner_handle.task_id.clone();
    register_agent_runner(&state, &agent_id, runner_handle, start_tx).await;
    // join_handle is intentionally dropped — the task runs in background.
    drop(join_handle);

    Ok((
        StatusCode::ACCEPTED,
        Json(serde_json::json!({
            "status": "accepted",
            "agent_id": agent_id,
            "task_id": task_id
        })),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use dashmap::DashMap;

    #[test]
    fn test_parse_traceparent_valid_v00() {
        let valid = "00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01";
        assert_eq!(parse_traceparent(valid), Some(valid.to_string()));
    }

    #[test]
    fn test_parse_traceparent_rejects_version_ff() {
        let invalid = "ff-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01";
        assert_eq!(parse_traceparent(invalid), None);
    }

    #[test]
    fn test_parse_traceparent_rejects_all_zero_trace_id() {
        let invalid = "00-00000000000000000000000000000000-00f067aa0ba902b7-01";
        assert_eq!(parse_traceparent(invalid), None);
    }

    #[test]
    fn test_parse_traceparent_rejects_all_zero_parent_id() {
        let invalid = "00-4bf92f3577b34da6a3ce929d0e0e4736-0000000000000000-01";
        assert_eq!(parse_traceparent(invalid), None);
    }

    #[test]
    fn test_parse_traceparent_rejects_malformed_lengths() {
        assert_eq!(parse_traceparent("00-1234-5678-01"), None);
        assert_eq!(parse_traceparent("not-a-traceparent-header"), None);
    }

    #[test]
    fn test_parse_traceparent_forward_compatible_higher_version() {
        let v01 = "01-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01-extra";
        assert_eq!(parse_traceparent(v01), Some(v01.to_string()));
    }

    #[test]
    fn test_claim_task_request_deduplication() {
        let requests = DashMap::new();
        let now = Instant::now();
        let key = "agent-1:req-100:abc123hash";

        // First attempt succeeds
        assert!(claim_task_request(&requests, key, now));

        // Immediate duplicate attempt fails
        assert!(!claim_task_request(
            &requests,
            key,
            now + Duration::from_secs(5)
        ));

        // Attempt after window (16 seconds later) succeeds
        assert!(claim_task_request(
            &requests,
            key,
            now + Duration::from_secs(DEDUP_WINDOW_SECS + 1)
        ));
    }

    #[tokio::test]
    async fn test_validate_agent_preflight_budget_scrubbing() {
        let app_state = Arc::new(AppState::new().await.unwrap());
        let agent_id = "test-bankrupt-agent";
        let agent = crate::agent::types::EngineAgent {
            identity: crate::agent::types::AgentIdentity {
                id: agent_id.to_string(),
                name: "Bankrupt Agent".to_string(),
                role: "Analyst".to_string(),
                department: "Finance".to_string(),
                ..Default::default()
            },
            economics: crate::agent::types::AgentEconomics {
                budget_usd: 10.0,
                cost_usd: 15.0,
                ..Default::default()
            },
            ..Default::default()
        };
        app_state
            .registry
            .agents
            .insert(agent_id.to_string(), agent);

        let headers = axum::http::HeaderMap::new();
        let mut payload = TaskPayload {
            message: "Hello".to_string(),
            ..Default::default()
        };

        let result = validate_agent_preflight(&app_state, agent_id, &headers, &mut payload);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("exhausted its allocated budget"));
        assert!(!err_msg.contains("$15"));
        assert!(!err_msg.contains("$10"));
    }
}

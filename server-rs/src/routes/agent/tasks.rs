//! @docs ARCHITECTURE:Networking
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / HTTP Routes / Agent Tasks
//! - **Primary Entrypoints**: `send_task`, `spawn_agent_runner`, `claim_task_request`
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none declared
//! - **Telemetry Targets**: none declared
//! - **Witness Tests**: none declared

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

static TRACEPARENT_REGEX: once_cell::sync::Lazy<regex::Regex> = once_cell::sync::Lazy::new(|| {
    regex::Regex::new(r"^[0-9a-f]{2}-[0-9a-f]{32}-[0-9a-f]{16}-[0-9a-f]{2}$").unwrap()
});

pub fn claim_task_request(
    requests: &dashmap::DashMap<String, std::time::Instant>,
    key: &str,
    now: std::time::Instant,
) -> bool {
    use dashmap::mapref::entry::Entry;
    match requests.entry(key.to_owned()) {
        Entry::Occupied(mut entry) => {
            if now.saturating_duration_since(*entry.get()).as_secs() < DEDUP_WINDOW_SECS {
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
) {
    let agent_id_owned = agent_id.to_string();
    let state_clone = state.clone();
    let task_id = uuid::Uuid::new_v4().to_string();
    let task_id_for_cleanup = task_id.clone();
    let agent_id_for_cleanup = agent_id_owned.clone();

    let join_handle = tokio::spawn(async move {
        // Acquire permit before running to throttle concurrency
        let _permit = match state_clone.comms.runner_semaphore.acquire().await {
            Ok(p) => p,
            Err(_) => {
                tracing::error!("❌ [Runner] Semaphore closed");
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

        // Auto-cleanup handle only if it matches our task ID (F-03)
        state_clone
            .comms
            .active_runners
            .remove_if(&agent_id_for_cleanup, |_, stored_runner| {
                stored_runner.task_id == task_id_for_cleanup
            });
    });

    let runner_handle = crate::state::hubs::comm::RunnerHandle {
        abort_handle: join_handle.abort_handle(),
        task_id,
    };

    (join_handle, runner_handle)
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
                return Err(AppError::BadRequest(format!(
                    "Agent '{}' is bankrupt. Cost: ${}, Budget: ${}",
                    agent_id, agent.economics.cost_usd, agent.economics.budget_usd
                )));
            }
        }
    }

    // Forward traceparent for distributed tracing
    if payload.traceparent.is_none() {
        if let Some(tp) = headers.get("traceparent").and_then(|v| v.to_str().ok()) {
            // W3C traceparent context validation regex (F-09)
            if TRACEPARENT_REGEX.is_match(tp) {
                payload.traceparent = Some(tp.to_string());
            } else {
                tracing::warn!("Blocked invalid traceparent header");
            }
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
        let now = std::time::Instant::now();

        // Proactively prune expired entries (>30 seconds old)
        state
            .comms
            .recent_requests
            .retain(|_, time| time.elapsed().as_secs() < DEDUP_CACHE_PRUNE_SECS);

        // Serialize payload to hash it (TaskPayload does not implement Hash directly)
        let serialized_payload = serde_json::to_string(&payload).unwrap_or_default();
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

    // 3. Auto-Inject Socratic Context Envelope if not explicitly bypassed
    if !payload.skip_socratic_gate.unwrap_or(false) {
        let (agent_name, agent_role, budget, active_slot) =
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
                )
            } else {
                (
                    agent_id.clone(),
                    "General Intelligence Node".to_string(),
                    None,
                    None,
                )
            };

        let is_privacy = state
            .governance
            .is_privacy_mode_enabled(payload.cluster_id.as_deref());
        let envelope = crate::agent::socratic::SocraticContextEnvelope::compile(
            &agent_id,
            &agent_name,
            &agent_role,
            payload
                .primary_goal
                .as_deref()
                .unwrap_or("Autonomous Task Execution"),
            payload
                .allowed_files
                .clone()
                .or_else(|| payload.context_files.clone()),
            budget,
            active_slot,
            is_privacy,
        );
        payload.message = envelope.inject_into_prompt(&payload.message);
    }

    // Proactive Abort-on-New Policy: Terminate any existing task for this agent
    if let Some((_, runner)) = state.comms.active_runners.remove(&agent_id) {
        tracing::info!(
            "🔄 [Gateway] Aborting existing task for agent {} to prioritize new request.",
            agent_id
        );
        runner.abort_handle.abort();
    }

    // Spawn Runner via canonical helper (F-03, #6)
    let (join_handle, runner_handle) = spawn_agent_runner(&state, &agent_id, payload);
    state
        .comms
        .active_runners
        .insert(agent_id.clone(), runner_handle);
    // join_handle is intentionally dropped — the task runs in background.
    drop(join_handle);

    Ok((
        StatusCode::ACCEPTED,
        Json(serde_json::json!({
            "status": "accepted",
            "agent_id": agent_id
        })),
    ))
}

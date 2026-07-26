//! @docs ARCHITECTURE:Networking
//!
//! ### AI Assist Note
//! **Agent Gateway Orchestrator**: Manages the REST surface for autonomous
//! agent registration, configuration, and task dispatching. Features
//! **HATEOAS-Compliant Discovery**: responses include `_links` for
//! self-discovery and related actions. Implements **Async Task
//! Dispatch**: high-level text tasks are acknowledged with `202 ACCEPTED`
//! and spawned into background `AgentRunner` instances. Enforces **W3C
//! Traceparent Propagation** to ensure end-to-end observability from the
//! UI request to the final tool execution (AGNT-01).
//!
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: 404 on valid agent IDs due to registry cache
//!   staling, 400 on suspended agent tasks, or zombie runner tasks
//!   failing silently after process restarts.
//! - **Telemetry Link**: Search for `[Gateway]` in `tracing` logs for
//!   dispatch/sync events.
//! - **Trace Scope**: `server-rs::routes::agent`

use crate::agent::mission::get_swarm_graph;
use crate::{
    agent::{
        runner::AgentRunner,
        types::{EngineAgent, TaskPayload},
    },
    error::AppError,
    routes::pagination::{PaginatedResponse, PaginationParams},
    state::AppState,
};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use futures::StreamExt;
use serde::Serialize;
use std::sync::Arc;

static TRACEPARENT_REGEX: once_cell::sync::Lazy<regex::Regex> = once_cell::sync::Lazy::new(|| {
    regex::Regex::new(r"^[0-9a-f]{2}-[0-9a-f]{32}-[0-9a-f]{16}-[0-9a-f]{2}$").unwrap()
});

// --- Constants (#9, #8) ---
pub(crate) const STATUS_IDLE: &str = "idle";
pub(crate) const STATUS_BUSY: &str = "busy";
pub(crate) const STATUS_SUSPENDED: &str = "suspended";
pub(crate) const MAX_FAILURE_COUNT: u32 = 5;
const RECOVERY_CONCURRENCY: usize = 8;

/// #6: Canonical runner spawn. Creates a background `AgentRunner` task with
/// semaphore throttling, error masking (F-04), RFC 9457 failure events, and
/// auto-cleanup keyed by task ID (F-03).
///
/// Returns a `(JoinHandle, RunnerHandle)` pair. The caller is responsible for
/// inserting the `RunnerHandle` into `state.comms.active_runners`.
fn spawn_agent_runner(
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

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AgentResponse {
    pub id: String,
    pub name: String,
    pub role: String,
    pub department: String,
    pub status: String,
    pub model: String,
    pub provider: String,
    pub model_config: crate::agent::types::ModelConfig,
    pub planning_slot: Option<crate::agent::types::ModelConfig>,
    pub execution_slot: Option<crate::agent::types::ModelConfig>,
    pub active_model_slot: Option<String>,
    pub budget_usd: f64,
    pub cost_usd: f64,
    pub is_healthy: bool,
    pub is_bankrupt: bool,
    pub failure_count: u32,
    pub last_failure_at: Option<chrono::DateTime<chrono::Utc>>,
    pub skills: Vec<String>,
    pub workflows: Vec<String>,
    pub mcp_tools: Vec<String>,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    pub version: u32,
    pub tokens_used: u64,
    pub token_usage: crate::agent::types::TokenUsage,
}

impl From<&EngineAgent> for AgentResponse {
    fn from(agent: &EngineAgent) -> Self {
        let model_name = if agent.models.model.model_id.trim().is_empty() {
            agent
                .models
                .model_id
                .as_deref()
                .unwrap_or_default()
                .to_string()
        } else {
            agent.models.model.model_id.clone()
        };

        if model_name.trim().is_empty() {
            tracing::warn!("⚠️ Agent {} has no configured model_id!", agent.identity.id);
        }

        Self {
            id: agent.identity.id.clone(),
            name: agent.identity.name.clone(),
            role: agent.identity.role.clone(),
            department: agent.identity.department.clone(),
            status: agent.health.status.clone(),
            model: model_name,
            provider: agent.models.model.provider.to_string(),
            // SEC: Redact api_key from REST responses to prevent credential leakage.
            // ModelConfig may contain per-agent API keys loaded from the DB.
            model_config: {
                let mut mc = agent.models.model.clone();
                mc.api_key = None;
                mc
            },
            planning_slot: agent.models.planning_slot.clone().map(|mut s| {
                s.api_key = None;
                s
            }),
            execution_slot: agent.models.execution_slot.clone().map(|mut s| {
                s.api_key = None;
                s
            }),
            active_model_slot: agent.models.active_model_slot.clone(),
            budget_usd: agent.economics.budget_usd,
            cost_usd: agent.economics.cost_usd,
            is_healthy: agent.health.failure_count < MAX_FAILURE_COUNT,
            is_bankrupt: agent.economics.cost_usd >= agent.economics.budget_usd
                && agent.economics.budget_usd > 0.0,
            failure_count: agent.health.failure_count,
            last_failure_at: agent.health.last_failure_at,
            skills: agent.capabilities.skills.clone(),
            workflows: agent.capabilities.workflows.clone(),
            mcp_tools: agent.capabilities.mcp_tools.clone(),
            created_at: agent.created_at,
            version: agent.version,
            tokens_used: agent.economics.tokens_used,
            token_usage: agent.economics.token_usage.clone(),
        }
    }
}

/// Centralized persistence and broadcast helper for agent updates.
async fn update_and_persist_agent<F>(
    state: &Arc<AppState>,
    agent_id: &str,
    f: F,
) -> Result<EngineAgent, AppError>
where
    F: FnOnce(&mut EngineAgent),
{
    // 1. Retrieve current agent state from registry
    let (mut agent_clone, original_agent, current_version) = {
        let agent = state
            .registry
            .agents
            .get(agent_id)
            .ok_or_else(|| AppError::NotFound(format!("Agent {} not found", agent_id)))?;
        (agent.clone(), agent.clone(), agent.version)
    };

    // 2. Apply updates to the clone
    f(&mut agent_clone);

    // 3. Attempt to save the updated agent to the database.
    // The DB query checks if version == current_version in the WHERE clause,
    // and atomically increments it to current_version + 1 on success.
    crate::agent::persistence::save_agent_db(&state.resources.pool, &mut agent_clone).await?;

    // 4. Update memory registry only if DB write succeeded.
    // (agent_clone.version has already been incremented by save_agent_db)

    let final_agent = {
        let mut agent_entry = state
            .registry
            .agents
            .get_mut(agent_id)
            .ok_or_else(|| AppError::NotFound(format!("Agent {} not found", agent_id)))?;

        // Defensive check: save_agent_db already guards against version conflicts
        // via AppError::Conflict. If we reach here, the DB write succeeded. The only
        // way the memory version could differ is if another thread modified memory
        // between our DB write and this point — an extremely narrow race.
        if agent_entry.version != current_version {
            tracing::error!(
                "⚠️ [Gateway] Unexpected version mismatch in memory registry for agent {}. \
                 DB wrote version {}, but memory has version {}. Overwriting with DB-committed state.",
                agent_id, agent_clone.version, agent_entry.version
            );
        }

        // Preserve live runtime fields from agent_entry if they weren't mutated by the closure `f`
        if agent_clone.health.status == original_agent.health.status {
            agent_clone.health.status = agent_entry.health.status.clone();
        }
        if agent_clone.health.heartbeat_at == original_agent.health.heartbeat_at {
            agent_clone.health.heartbeat_at = agent_entry.health.heartbeat_at;
        }
        if agent_clone.health.failure_count == original_agent.health.failure_count {
            agent_clone.health.failure_count = agent_entry.health.failure_count;
        }
        if agent_clone.health.last_failure_at == original_agent.health.last_failure_at {
            agent_clone.health.last_failure_at = agent_entry.health.last_failure_at;
        }
        if agent_clone.state.current_task == original_agent.state.current_task {
            agent_clone.state.current_task = agent_entry.state.current_task.clone();
        }
        if agent_clone.state.active_mission == original_agent.state.active_mission {
            agent_clone.state.active_mission = agent_entry.state.active_mission.clone();
        }
        if agent_clone.economics.cost_usd == original_agent.economics.cost_usd {
            agent_clone.economics.cost_usd = agent_entry.economics.cost_usd;
        }
        if agent_clone.economics.tokens_used == original_agent.economics.tokens_used {
            agent_clone.economics.tokens_used = agent_entry.economics.tokens_used;
        }
        if agent_clone.economics.token_usage.input_tokens == original_agent.economics.token_usage.input_tokens {
            agent_clone.economics.token_usage.input_tokens = agent_entry.economics.token_usage.input_tokens;
        }
        if agent_clone.economics.token_usage.output_tokens == original_agent.economics.token_usage.output_tokens {
            agent_clone.economics.token_usage.output_tokens = agent_entry.economics.token_usage.output_tokens;
        }
        if agent_clone.economics.token_usage.total_tokens == original_agent.economics.token_usage.total_tokens {
            agent_clone.economics.token_usage.total_tokens = agent_entry.economics.token_usage.total_tokens;
        }

        *agent_entry = agent_clone.clone();
        agent_clone
    };

    // Broadcast update
    state.emit_event(serde_json::json!({
        "type": "agent:update",
        "agent_id": agent_id,
        "data": AgentResponse::from(&final_agent)
    }));

    // Sync registry memory to data/agents.json
    let agents: Vec<EngineAgent> = state
        .registry
        .agents
        .iter()
        .map(|kv| kv.value().clone())
        .collect();
    if let Err(e) = crate::agent::persistence::save_agents_json(&state.base_dir, agents).await {
        tracing::error!("⚠️ [Gateway] Failed to update agents.json: {:?}", e);
    }

    Ok(final_agent)
}

/// GET /v1/agents
///
/// Retrieves the list of all registered agents in the swarm. Implements
/// HATEOAS-compliant pagination to allow for efficient UI rendering and discovery.
///
/// ### 🛰️ Registry Introspection
/// This handler pulls directly from the engine's memory-mapped `AgentResponse`.
/// It maps raw back-end models into a clean, RESTful representation for
/// dashboard consumption.
///
/// @docs API_REFERENCE:GetAgents
pub async fn get_agents(
    State(state): State<Arc<AppState>>,
    Query(params): Query<PaginationParams>,
) -> Result<impl IntoResponse, AppError> {
    let agents: Vec<AgentResponse> = state
        .registry
        .agents
        .iter()
        .map(|kv| AgentResponse::from(kv.value()))
        .collect();
    Ok(Json(PaginatedResponse::from_vec(
        agents,
        &params,
        "/v1/agents",
    )))
}

/// GET /v1/agents/:id
///
/// Retrieves the detailed state of a specific agent by its unique identifier.
/// Provides O(1) discovery for high-density swarms.
///
/// @docs API_REFERENCE:GetAgent
pub async fn get_agent(
    Path(agent_id): Path<String>,
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, AppError> {
    let agent = state
        .registry
        .agents
        .get(&agent_id)
        .ok_or_else(|| AppError::NotFound(format!("Agent '{}' not found", agent_id)))?;

    Ok(Json(AgentResponse::from(agent.value())))
}

/// POST /v1/agents/:id/tasks
///
/// Dispatches a high-level text task to a specific autonomous agent.
/// Automatically handles distributed trace propagation (via W3C `traceparent`)
/// and validates agent existence before dispatch.
///
/// ### 🔦 Distributed Tracing (AGNT-01)
/// If a `traceparent` header is present in the UI request, it is parsed
/// and injected into the mission payload. This ensures that the engine's
/// background `AgentRunner` spans are correctly linked to the front-end
/// session in our Jaeger/OTel traces.
///
/// @docs API_REFERENCE:SendTask
#[tracing::instrument(skip(state, headers, payload), fields(agent_id = %agent_id), name = "agent_gateway::dispatch")]
/// Helper to validate agent status/budget and extract traceparent from headers.
fn validate_agent_preflight(
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
            if agent.economics.cost_usd >= agent.economics.budget_usd
                && agent.economics.budget_usd > 0.0
            {
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
        
        // Clean up expired entries (>30 seconds old) periodically to avoid memory leaks
        state.comms.recent_requests.retain(|_, time| time.elapsed().as_secs() < 30);

        // Serialize payload to hash it (TaskPayload does not implement Hash directly)
        let serialized_payload = serde_json::to_string(&payload).unwrap_or_default();
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        use std::hash::{Hash, Hasher};
        serialized_payload.hash(&mut hasher);
        let payload_hash = hasher.finish();

        let dedupe_key = format!("{}:{}:{:x}", agent_id, req_id, payload_hash);

        if let Some(prev_time) = state.comms.recent_requests.get(&dedupe_key) {
            if prev_time.elapsed().as_secs() < 15 {
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
                    }))
                ));
            }
        }
        state.comms.recent_requests.insert(dedupe_key, now);
    }

    tracing::info!("📡 [Gateway] Task dispatched to Agent {}", agent_id);

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

/// POST /agents
///
/// Registers a new agent in the system and triggers persistence.
#[tracing::instrument(skip(state, new_agent), fields(agent_id = %new_agent.identity.id), name = "agent_registry::create")]
pub async fn create_agent(
    State(state): State<Arc<AppState>>,
    Json(mut new_agent): Json<EngineAgent>,
) -> Result<impl IntoResponse, AppError> {
    let agent_id = new_agent.identity.id.clone();

    // Check if agent already exists in memory registry
    if state.registry.agents.contains_key(&agent_id) {
        return Err(AppError::Conflict(format!(
            "Agent '{}' already exists in memory registry",
            agent_id
        )));
    }

    // Check if agent already exists in the database
    let exists = sqlx::query_scalar::<_, i64>("SELECT 1 FROM agents WHERE id = ?1")
        .bind(&agent_id)
        .fetch_optional(&state.resources.pool)
        .await
        .map_err(AppError::Sqlx)?
        .is_some();

    if exists {
        return Err(AppError::Conflict(format!(
            "Agent '{}' already exists in database",
            agent_id
        )));
    }

    // ── Item-6 Guard: Warn on Ollama agents missing rate limits ──────────────
    // Ollama agents with tpm=None fall back to a 100k token estimate in the pruner,
    // which is also computed using cl100k_base (a GPT-4 tokenizer — inaccurate for
    // Gemma4/Llama/Mistral). Without tpm set, context pruning can miss the real limit
    // and produce runaway token usage. This warning is non-blocking.
    let is_ollama = matches!(
        new_agent.models.model.provider,
        crate::agent::types::ModelProvider::Ollama
    );
    let missing_limits = new_agent.models.model.tpm.is_none()
        || new_agent.models.model.rpm.is_none();
    let rate_limit_warning = if is_ollama && missing_limits {
        tracing::warn!(
            "⚠️ [Gateway] Ollama agent '{}' created without tpm/rpm limits. \
             Context pruner will use inaccurate fallback (100k token estimate with GPT-4 tokenizer). \
             Recommend setting tpm ≥ 32000 and rpm ≥ 10 for local models.",
            new_agent.identity.id
        );
        Some("Ollama agent created without tpm/rpm rate limits. Context pruner will use inaccurate fallback. Recommend setting tpm >= 32000 and rpm >= 10.")
    } else {
        None
    };

    crate::agent::persistence::save_agent_db(&state.resources.pool, &mut new_agent).await?;

    state
        .registry
        .agents
        .insert(agent_id.clone(), new_agent.clone());

    let agent_path = format!("/v1/agents/{}", agent_id);
    state.emit_event(serde_json::json!({
        "type": "agent:create",
        "agent_id": agent_id.clone(),
        "data": AgentResponse::from(&new_agent)
    }));

    // Sync registry memory to data/agents.json
    let agents: Vec<EngineAgent> = state
        .registry
        .agents
        .iter()
        .map(|kv| kv.value().clone())
        .collect();
    if let Err(e) = crate::agent::persistence::save_agents_json(&state.base_dir, agents).await {
        tracing::error!("⚠️ [Gateway] Failed to update agents.json: {:?}", e);
    }

    Ok((
        StatusCode::CREATED,
        [(axum::http::header::LOCATION, agent_path.clone())],
        Json(serde_json::json!({
            "status": "ok",
            "agent_id": agent_id,
            "warnings": rate_limit_warning.map(|w| vec![w]).unwrap_or_default(),
            "_links": {
                "self":    { "href": agent_path.clone(), "method": "GET" },
                "tasks":   { "href": format!("{}/tasks", agent_path), "method": "POST" },
                "collection": { "href": "/v1/agents", "method": "GET" }
            }
        })),
    ))
}

/// PUT /agents/:id
///
/// Updates an existing agent's configuration, metadata, or role.
#[tracing::instrument(skip(state, update), fields(agent_id = %agent_id), name = "agent_registry::update")]
pub async fn update_agent(
    Path(agent_id): Path<String>,
    State(state): State<Arc<AppState>>,
    Json(update): Json<crate::agent::types::AgentConfigUpdate>,
) -> Result<impl IntoResponse, AppError> {
    update_and_persist_agent(&state, &agent_id, |agent| {
        update.apply_to(agent);
    })
    .await?;

    Ok(Json(serde_json::json!({ "status": "ok" })))
}

/// POST /agents/:id/pause
#[tracing::instrument(skip(state), fields(agent_id = %agent_id, trace_id = tracing::field::Empty, request_id = tracing::field::Empty, http_status = tracing::field::Empty), name = "agent_registry::pause")]
pub async fn pause_agent(
    Path(agent_id): Path<String>,
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, AppError> {
    update_and_persist_agent(&state, &agent_id, |agent| {
        agent.pause();
    })
    .await?;

    // Zombie Task Termination
    if let Some((_, runner)) = state.comms.active_runners.remove(&agent_id) {
        tracing::info!(
            "🛑 [Gateway] Aborting active runner for suspended agent: {}",
            agent_id
        );
        runner.abort_handle.abort();
    }

    Ok(Json(serde_json::json!({ "status": "ok" })))
}

/// POST /agents/:id/resume
#[tracing::instrument(skip(state), fields(agent_id = %agent_id, trace_id = tracing::field::Empty, request_id = tracing::field::Empty, http_status = tracing::field::Empty), name = "agent_registry::resume")]
pub async fn resume_agent(
    Path(agent_id): Path<String>,
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, AppError> {
    update_and_persist_agent(&state, &agent_id, |agent| {
        agent.resume();
    })
    .await?;

    Ok(Json(serde_json::json!({ "status": "ok" })))
}

/// POST /agents/:id/reset
///
/// @docs API_REFERENCE:ResetAgent
/// Resets an agent's failure count and returns it to idle status.
/// Used to clear "Self-heal cooldowns" after configuration fixes.
#[tracing::instrument(skip(state), fields(agent_id = %agent_id, trace_id = tracing::field::Empty, request_id = tracing::field::Empty, http_status = tracing::field::Empty), name = "agent_registry::reset")]
pub async fn reset_agent(
    Path(agent_id): Path<String>,
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, AppError> {
    update_and_persist_agent(&state, &agent_id, |agent| {
        agent.reset();
    })
    .await?;

    // Zombie Task Termination on Reset
    if let Some((_, runner)) = state.comms.active_runners.remove(&agent_id) {
        tracing::info!(
            "🛑 [Gateway] Aborting specific runner for reset agent: {}",
            agent_id
        );
        runner.abort_handle.abort();
    }

    Ok(Json(
        serde_json::json!({ "status": "ok", "message": "Failure count reset and tasks terminated." }),
    ))
}

/// POST /agents/:id/mission
///
/// Synchronizes a mission objective to an agent's active mission state.
#[tracing::instrument(skip(state, mission), fields(agent_id = %id), name = "agent_registry::sync_mission")]
pub async fn sync_mission(
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
    Json(mission): Json<serde_json::Value>,
) -> Result<impl IntoResponse, AppError> {
    update_and_persist_agent(&state, &id, |agent| {
        agent.set_mission(mission);
    })
    .await?;

    Ok(Json(serde_json::json!({ "status": "ok" })))
}

/// GET /v1/agents/graph
///
/// Retrieves the complete knowledge graph of agents, missions, and their
/// relationships for real-time visualization in the dashboard.
pub async fn get_swarm_graph_handler(
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, AppError> {
    let graph = get_swarm_graph(&state.resources.pool).await?;
    Ok(Json(graph))
}

#[derive(Debug, serde::Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, serde::Deserialize)]
pub struct ChatCompletionRequest {
    pub model: String,
    pub messages: Vec<ChatMessage>,
    #[allow(dead_code)]
    pub temperature: Option<f32>,
    #[allow(dead_code)]
    pub max_tokens: Option<u32>,
}

#[derive(Debug, serde::Serialize)]
pub struct ChatCompletionChoiceMessage {
    pub role: &'static str,
    pub content: String,
}

#[derive(Debug, serde::Serialize)]
pub struct ChatCompletionChoice {
    pub index: u32,
    pub message: ChatCompletionChoiceMessage,
    pub finish_reason: &'static str,
}

#[derive(Debug, serde::Serialize)]
pub struct ChatCompletionUsage {
    pub prompt_tokens: u64,
    pub completion_tokens: u64,
    pub total_tokens: u64,
}

#[derive(Debug, serde::Serialize)]
pub struct ChatCompletionResponse {
    pub id: String,
    pub object: &'static str,
    pub created: i64,
    pub model: String,
    pub choices: Vec<ChatCompletionChoice>,
    pub usage: ChatCompletionUsage,
}

/// POST /v1/chat/completions
///
/// OpenAI-compatible completion endpoint that routes the task to a specific Swarm agent (the "model")
/// and executes the task synchronously, returning the final report/result.
/// POST /v1/chat/completions
///
/// OpenAI-compatible completion endpoint that routes the task to a specific Swarm agent (the "model")
/// and executes the task synchronously, returning the final report/result.
pub async fn create_chat_completion(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Json(req): Json<ChatCompletionRequest>,
) -> Result<impl IntoResponse, AppError> {
    let agent_id = req.model.clone();

    let last_user_message = req
        .messages
        .iter()
        .rfind(|m| m.role == "user")
        .map(|m| m.content.clone())
        .ok_or_else(|| {
            AppError::BadRequest("No user message found in completion history".to_string())
        })?;

    let mut payload = TaskPayload {
        message: last_user_message,
        active_model_slot: Some("default".to_string()),
        ..Default::default()
    };

    // Preflight validation for chat completion (Existence, Status, Budget, and Traceparent)
    validate_agent_preflight(&state, &agent_id, &headers, &mut payload)?;

    let _permit = state.comms.runner_semaphore.acquire().await.map_err(|_| {
        AppError::InternalServerError("Runner execution throttle semaphore closed".to_string())
    })?;

    tracing::info!(
        "📡 [Completions API] Dispatching synchronous task to agent {}",
        agent_id
    );
    let runner = AgentRunner::new(state.clone());
    let run_res = runner.run(agent_id.clone(), payload).await?;

    let (prompt_tokens, completion_tokens, total_tokens) =
        if let Some(agent) = state.registry.agents.get(&agent_id) {
            let usage = &agent.value().economics.token_usage;
            (usage.input_tokens, usage.output_tokens, usage.total_tokens)
        } else {
            (0, 0, 0)
        };

    let response = ChatCompletionResponse {
        id: format!("chatcmpl-{}", uuid::Uuid::new_v4()),
        object: "chat.completion",
        created: chrono::Utc::now().timestamp(),
        model: agent_id,
        choices: vec![ChatCompletionChoice {
            index: 0,
            message: ChatCompletionChoiceMessage {
                role: "assistant",
                content: run_res,
            },
            finish_reason: "stop",
        }],
        usage: ChatCompletionUsage {
            prompt_tokens,
            completion_tokens,
            total_tokens,
        },
    };

    Ok(Json(response))
}

#[derive(Debug, serde::Deserialize)]
pub struct CloneMissionRequest {
    pub primary_goal: Option<String>,
}

/// Clones an existing mission into a fresh record with a unique UUID.
pub async fn clone_mission(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(payload): Json<CloneMissionRequest>,
) -> Result<impl IntoResponse, AppError> {
    let source_mission = crate::agent::mission::get_mission_by_id(&state.resources.pool, &id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Mission {} not found", id)))?;

    let new_id = format!("msn-{}", uuid::Uuid::new_v4());
    let title = payload
        .primary_goal
        .clone()
        .unwrap_or_else(|| format!("[Copy] {}", source_mission.title));

    let created = crate::agent::mission::create_mission_with_id(
        &state.resources.pool,
        &new_id,
        &source_mission.agent_id,
        &title,
        source_mission.budget_usd,
    )
    .await?;

    tracing::info!(
        "📋 [Mission Clone] Cloned mission {} -> new mission {}",
        id,
        new_id
    );

    Ok((StatusCode::CREATED, Json(created)))
}

/// Scans the database on startup and resumes runners for agents found in an active ("busy") state.
pub async fn recover_active_agents(state: Arc<AppState>) {
    let agents: Vec<crate::agent::types::EngineAgent> = state
        .registry
        .agents
        .iter()
        .map(|kv| kv.value().clone())
        .collect();

    let busy_agents: Vec<_> = agents
        .into_iter()
        .filter(|a| {
            a.health.status == STATUS_BUSY
                || a.health.status == "working"
                || a.health.status == "thinking"
                || a.health.status == "active"
        })
        .collect();

    if busy_agents.is_empty() {
        return;
    }

    // Bounded concurrency recovery to prevent pool/connection exhaustion (F-10)
    futures::stream::iter(busy_agents)
        .for_each_concurrent(RECOVERY_CONCURRENCY, |agent| {
            let state = state.clone();
            async move {
                if let Some(task) = agent.state.current_task.clone() {
                    if !task.is_empty() {
                        let agent_id = agent.identity.id.clone();
                        let cluster_id = agent
                            .state
                            .active_mission
                            .as_ref()
                            .and_then(|m| m.get("id"))
                            .and_then(|id| id.as_str())
                            .map(|s| s.to_string());

                        tracing::info!(
                            "🔄 [State Recovery] Recovering active agent {} for task: {}",
                            agent_id,
                            task
                        );

                        let payload = crate::agent::types::TaskPayload {
                            message: task,
                            cluster_id,
                            ..Default::default()
                        };

                        let (join_handle, runner_handle) =
                            spawn_agent_runner(&state, &agent_id, payload);
                        state
                            .comms
                            .active_runners
                            .insert(agent_id.clone(), runner_handle);

                        let _ = join_handle.await;
                    } else {
                        // Reset to idle since task is empty or missing.
                        // #2+#3: Clone data, release DashMap guard, then await DB save.
                        // On success, re-acquire and write back. On failure, keep memory as-is
                        // so it stays consistent with the DB state.
                        let aid = agent.identity.id.clone();
                        let mut clone = state.registry.agents.get(&aid).map(|e| e.value().clone());
                        // DashMap Ref guard is consumed by .map(clone) above — no held lock.

                        if let Some(ref mut a) = clone {
                            a.health.status = STATUS_IDLE.to_string();
                            match crate::agent::persistence::save_agent_db(&state.resources.pool, a).await {
                                Ok(()) => {
                                    // DB succeeded — now safe to update memory
                                    if let Some(mut entry) = state.registry.agents.get_mut(&aid) {
                                        *entry = a.clone();
                                    }
                                }
                                Err(e) => {
                                    tracing::error!(
                                        "❌ [State Recovery] Failed to reset agent {} to idle: {}. Memory NOT modified.",
                                        aid, e
                                    );
                                }
                            }
                        }
                    }
                } else {
                    // No task details available, fallback to idle to prevent deadlock/busy lock.
                    // Same safe pattern as above.
                    let aid = agent.identity.id.clone();
                    let mut clone = state.registry.agents.get(&aid).map(|e| e.value().clone());

                    if let Some(ref mut a) = clone {
                        a.health.status = STATUS_IDLE.to_string();
                        match crate::agent::persistence::save_agent_db(&state.resources.pool, a).await {
                            Ok(()) => {
                                if let Some(mut entry) = state.registry.agents.get_mut(&aid) {
                                    *entry = a.clone();
                                }
                            }
                            Err(e) => {
                                tracing::error!(
                                    "❌ [State Recovery] Failed to reset agent {} to idle: {}. Memory NOT modified.",
                                    aid, e
                                );
                            }
                        }
                    }
                }
            }
        })
        .await;
}

// Metadata: [agent]

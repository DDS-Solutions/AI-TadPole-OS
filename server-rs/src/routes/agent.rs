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
use serde::Serialize;
use std::sync::Arc;

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
    pub skills: Vec<String>,
    pub workflows: Vec<String>,
    pub mcp_tools: Vec<String>,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    pub version: u32,
    pub tokens_used: u32,
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
            model_config: agent.models.model.clone(),
            planning_slot: agent.models.planning_slot.clone(),
            execution_slot: agent.models.execution_slot.clone(),
            active_model_slot: agent.models.active_model_slot.clone(),
            budget_usd: agent.economics.budget_usd,
            cost_usd: agent.economics.cost_usd,
            is_healthy: agent.health.failure_count < 5,
            is_bankrupt: agent.economics.cost_usd >= agent.economics.budget_usd
                && agent.economics.budget_usd > 0.0,
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
    let (mut agent_clone, current_version) = {
        let agent = state
            .registry
            .agents
            .get(agent_id)
            .ok_or_else(|| AppError::NotFound(format!("Agent {} not found", agent_id)))?;
        (agent.clone(), agent.version)
    };

    // 2. Apply updates to the clone
    f(&mut agent_clone);

    // 3. Attempt to save the updated agent to the database.
    // The DB query checks if version == current_version in the WHERE clause,
    // and atomically increments it to current_version + 1 on success.
    crate::agent::persistence::save_agent_db(&state.resources.pool, &agent_clone).await?;

    // 4. Update memory registry only if DB write succeeded.
    // Increment version in the cloned struct to match DB version.
    agent_clone.version = current_version + 1;

    {
        let mut agent_entry = state
            .registry
            .agents
            .get_mut(agent_id)
            .ok_or_else(|| AppError::NotFound(format!("Agent {} not found", agent_id)))?;

        // Ensure no concurrent write modified the registry in the meantime.
        if agent_entry.version == current_version {
            *agent_entry = agent_clone.clone();
        } else {
            // Concurrency conflict: another thread updated memory while we were writing to DB.
            // Reload from DB to ensure memory is fresh.
            let db_agent = crate::agent::persistence::load_agents_db(&state.resources.pool)
                .await?
                .into_iter()
                .find(|a| a.identity.id == agent_id)
                .ok_or_else(|| {
                    AppError::InternalServerError("Recovered agent not found in DB".to_string())
                })?;
            *agent_entry = db_agent;
        }
    }

    // Broadcast update
    state.emit_event(serde_json::json!({
        "type": "agent:update",
        "agent_id": agent_id,
        "data": agent_clone
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

    Ok(agent_clone)
}

/// GET /v1/agents
///
/// Retrieves the list of all registered agents in the swarm. Implements
/// HATEOAS-compliant pagination to allow for efficient UI rendering and discovery.
///
/// ### 🛰️ Registry Introspection
/// This handler pulls directly from the engine's memory-mapped `AgentRegistry`.
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
pub async fn send_task(
    Path(agent_id): Path<String>,
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Json(mut payload): Json<TaskPayload>,
) -> Result<impl IntoResponse, AppError> {
    // Forward traceparent for distributed tracing
    if payload.traceparent.is_none() {
        if let Some(tp) = headers.get("traceparent").and_then(|v| v.to_str().ok()) {
            payload.traceparent = Some(tp.to_string());
        }
    }

    // Auth & Existence Check
    match state.registry.agents.get(&agent_id) {
        None => {
            return Err(AppError::NotFound(format!(
                "Agent '{}' not found",
                agent_id
            )))
        }
        Some(agent) if agent.health.status == "suspended" => {
            return Err(AppError::BadRequest(format!(
                "Agent '{}' is currently suspended.",
                agent_id
            )));
        }
        Some(_) => {} // All systems go
    }

    tracing::info!("📡 [Gateway] Task dispatched to Agent {}", agent_id);

    // Proactive Abort-on-New Policy: Terminate any existing task for this agent
    if let Some((_, old_handle)) = state.comms.active_runners.remove(&agent_id) {
        tracing::info!(
            "🔄 [Gateway] Aborting existing task for agent {} to prioritize new request.",
            agent_id
        );
        old_handle.abort();
    }

    // Spawn Runner with AbortHandle registration
    let agent_id_for_spawn = agent_id.clone();
    let state_clone = state.clone();
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
        if let Err(e) = runner.run(agent_id_for_spawn.clone(), payload).await {
            tracing::error!("❌ [Runner] Agent {} failed: {}", agent_id_for_spawn, e);

            // Async Failure Feedback with structured RFC 9457 support
            let error_data = serde_json::json!({
                "type": e.type_slug(),
                "title": e.type_slug().replace(['-', ':'], " ").to_uppercase(),
                "status": e.status_code().as_u16(),
                "detail": e.to_string(),
                "error_code": e.type_slug().to_uppercase()
            });

            state_clone.emit_event(serde_json::json!({
                "type": "agent:task_failed",
                "agent_id": agent_id_for_spawn.clone(),
                "error": error_data
            }));
        }

        // Auto-cleanup handle
        state_clone.comms.active_runners.remove(&agent_id_for_spawn);
    });

    state
        .comms
        .active_runners
        .insert(agent_id.clone(), join_handle.abort_handle());

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
    Json(new_agent): Json<EngineAgent>,
) -> Result<impl IntoResponse, AppError> {
    crate::agent::persistence::save_agent_db(&state.resources.pool, &new_agent).await?;

    let agent_id = new_agent.identity.id.clone();
    state
        .registry
        .agents
        .insert(agent_id.clone(), new_agent.clone());

    let agent_path = format!("/v1/agents/{}", agent_id);
    state.emit_event(serde_json::json!({
        "type": "agent:create",
        "agent_id": agent_id.clone(),
        "data": new_agent.clone()
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
#[tracing::instrument(skip(state), fields(agent_id = %agent_id), name = "agent_registry::pause")]
pub async fn pause_agent(
    Path(agent_id): Path<String>,
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, AppError> {
    update_and_persist_agent(&state, &agent_id, |agent| {
        agent.pause();
    })
    .await?;

    // Zombie Task Termination
    if let Some((_, abort_handle)) = state.comms.active_runners.remove(&agent_id) {
        tracing::info!(
            "🛑 [Gateway] Aborting active runner for suspended agent: {}",
            agent_id
        );
        abort_handle.abort();
    }

    Ok(Json(serde_json::json!({ "status": "ok" })))
}

/// POST /agents/:id/resume
#[tracing::instrument(skip(state), fields(agent_id = %agent_id), name = "agent_registry::resume")]
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
#[tracing::instrument(skip(state), fields(agent_id = %agent_id), name = "agent_registry::reset")]
pub async fn reset_agent(
    Path(agent_id): Path<String>,
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, AppError> {
    update_and_persist_agent(&state, &agent_id, |agent| {
        agent.reset();
    })
    .await?;

    // Zombie Task Termination on Reset
    if let Some((_, abort_handle)) = state.comms.active_runners.remove(&agent_id) {
        tracing::info!(
            "🛑 [Gateway] Aborting specific runner for reset agent: {}",
            agent_id
        );
        abort_handle.abort();
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

/// Scans the database on startup and resumes runners for agents found in an active ("busy") state.
pub async fn recover_active_agents(state: Arc<AppState>) {
    let agents: Vec<crate::agent::types::EngineAgent> = state
        .registry
        .agents
        .iter()
        .map(|kv| kv.value().clone())
        .collect();

    for mut agent in agents {
        // If the agent was marked as busy, it was actively running a task when the server shut down or crashed.
        if agent.health.status == "busy" {
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

                    let agent_id_for_spawn = agent_id.clone();
                    let state_clone = state.clone();
                    let join_handle = tokio::spawn(async move {
                        // Acquire permit before running
                        let _permit = match state_clone.comms.runner_semaphore.acquire().await {
                            Ok(p) => p,
                            Err(_) => {
                                tracing::error!("❌ [Runner] Semaphore closed during recovery");
                                return;
                            }
                        };
                        let runner = crate::agent::runner::AgentRunner::new(state_clone.clone());
                        if let Err(e) = runner.run(agent_id_for_spawn.clone(), payload).await {
                            tracing::error!(
                                "❌ [Runner] Recovered Agent {} failed: {}",
                                agent_id_for_spawn,
                                e
                            );
                            let error_data = serde_json::json!({
                                "type": e.type_slug(),
                                "title": e.type_slug().replace(['-', ':'], " ").to_uppercase(),
                                "status": e.status_code().as_u16(),
                                "detail": e.to_string(),
                                "error_code": e.type_slug().to_uppercase()
                            });
                            state_clone.emit_event(serde_json::json!({
                                "type": "agent:task_failed",
                                "agent_id": agent_id_for_spawn.clone(),
                                "error": error_data
                            }));
                        }
                        state_clone.comms.active_runners.remove(&agent_id_for_spawn);
                    });

                    state
                        .comms
                        .active_runners
                        .insert(agent_id.clone(), join_handle.abort_handle());
                } else {
                    // Reset to idle since task is empty
                    if let Some(mut entry) = state.registry.agents.get_mut(&agent.identity.id) {
                        entry.value_mut().health.status = "idle".to_string();
                    }
                    let _ = crate::agent::persistence::save_agent_db(&state.resources.pool, &agent)
                        .await;
                }
            } else {
                // No task details available, fallback to idle to prevent deadlock/busy lock
                agent.health.status = "idle".to_string();
                if let Some(mut entry) = state.registry.agents.get_mut(&agent.identity.id) {
                    entry.value_mut().health.status = "idle".to_string();
                }
                let _ =
                    crate::agent::persistence::save_agent_db(&state.resources.pool, &agent).await;
            }
        }
    }
}

// Metadata: [agent]

// Metadata: [agent]

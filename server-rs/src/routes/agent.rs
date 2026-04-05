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
    routes::error::AppError,
    routes::pagination::{PaginatedResponse, PaginationParams},
    state::AppState,
};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use std::sync::Arc;

/// GET /v1/agents
///
/// Retrieves the list of all registered agents in the swarm. Implements
/// HATEOAS-compliant pagination to allow for efficient UI rendering and discovery.
///
/// @docs API_REFERENCE:GetAgents
pub async fn get_agents(
    State(state): State<Arc<AppState>>,
    Query(params): Query<PaginationParams>,
) -> Result<impl IntoResponse, AppError> {
    let agents: Vec<EngineAgent> = state
        .registry
        .agents
        .iter()
        .map(|kv| kv.value().clone())
        .collect();
    Ok(Json(PaginatedResponse::from_vec(
        agents,
        &params,
        "/v1/agents",
    )))
}

/// POST /v1/agents/:id/tasks
///
/// Dispatches a high-level text task to a specific autonomous agent.
/// Automatically handles distributed trace propagation (via W3C `traceparent`)
/// and validates agent existence before dispatch.
///
/// @docs API_REFERENCE:SendTask
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
    let agent_status = state
        .registry
        .agents
        .get(&agent_id)
        .map(|a| a.status.clone());
    if agent_status.is_none() {
        return Err(AppError::NotFound(format!(
            "Agent '{}' not found",
            agent_id
        )));
    }

    if agent_status.unwrap() == "suspended" {
        return Err(AppError::BadRequest(format!(
            "Agent '{}' is currently suspended.",
            agent_id
        )));
    }

    tracing::info!("📡 [Gateway] Task dispatched to Agent {}", agent_id);

    // Spawn Runner
    let agent_id_for_spawn = agent_id.clone();
    tokio::spawn(async move {
        let runner = AgentRunner::new(state.clone());
        if let Err(e) = runner.run(agent_id_for_spawn.clone(), payload).await {
            tracing::error!("❌ [Runner] Agent {} failed: {}", agent_id_for_spawn, e);
        }
    });

    Ok((
        StatusCode::ACCEPTED,
        Json(serde_json::json!({
            "status": "accepted",
            "agentId": agent_id
        })),
    ))
}

/// POST /agents
///
/// Registers a new agent in the system and triggers persistence.
pub async fn create_agent(
    State(state): State<Arc<AppState>>,
    Json(new_agent): Json<EngineAgent>,
) -> Result<impl IntoResponse, AppError> {
    crate::agent::persistence::save_agent_db(&state.resources.pool, &new_agent)
        .await
        .map_err(AppError::Anyhow)?;

    let agent_id = new_agent.id.clone();
    state
        .registry
        .agents
        .insert(agent_id.clone(), new_agent.clone());

    let agent_path = format!("/v1/agents/{}", agent_id);
    state.emit_event(serde_json::json!({
        "type": "agent:create",
        "agentId": agent_id.clone(),
        "data": new_agent.clone()
    }));
    Ok((
        StatusCode::CREATED,
        [(axum::http::header::LOCATION, agent_path.clone())],
        Json(serde_json::json!({
            "status": "ok",
            "agentId": agent_id,
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
pub async fn update_agent(
    Path(agent_id): Path<String>,
    State(state): State<Arc<AppState>>,
    Json(update): Json<crate::agent::types::AgentConfigUpdate>,
) -> Result<impl IntoResponse, AppError> {
    let mut entry = state
        .registry
        .agents
        .get_mut(&agent_id)
        .ok_or_else(|| AppError::NotFound(format!("Agent {} not found", agent_id)))?;

    if let Some(name) = update.name {
        entry.name = name;
    }
    if let Some(role) = update.role {
        entry.role = role;
    }
    if let Some(dept) = update.department {
        entry.department = dept;
    }
    if let Some(model_id) = update.model_id {
        entry.model_id = Some(model_id.clone());
        entry.model.model_id = model_id;
    }
    if let Some(provider) = update.provider {
        entry.model.provider = provider;
    }
    if let Some(temp) = update.temperature {
        entry.model.temperature = Some(temp);
    }
    if let Some(prompt) = update.system_prompt {
        entry.model.system_prompt = Some(prompt);
    }
    if let Some(api_key) = update.api_key {
        entry.model.api_key = Some(api_key);
    }
    if let Some(color) = update.theme_color {
        entry.theme_color = Some(color);
    }
    if let Some(budget) = update.budget_usd {
        entry.budget_usd = budget;
    }
    if let Some(skills) = update.skills {
        entry.skills = skills;
    }
    if let Some(workflow) = update.workflows {
        entry.workflows = workflow;
    }
    if let Some(mcp_tools) = update.mcp_tools {
        entry.mcp_tools = mcp_tools;
    }
    if let Some(m2) = update.model2 {
        entry.model_2 = Some(m2);
    }
    if let Some(m3) = update.model3 {
        entry.model_3 = Some(m3);
    }
    if let Some(active_slot) = update.active_model_slot {
        entry.active_model_slot = Some(active_slot);
    }
    if let Some(mc2) = update.model_config2 {
        entry.model_config2 = Some(mc2);
    }
    if let Some(mc3) = update.model_config3 {
        entry.model_config3 = Some(mc3);
    }
    if let Some(connector_configs) = update.connector_configs {
        entry.connector_configs = connector_configs;
    }
    if let Some(voice_id) = update.voice_id {
        entry.voice_id = Some(voice_id);
    }
    if let Some(voice_engine) = update.voice_engine {
        entry.voice_engine = Some(voice_engine);
    }
    if let Some(base_url) = update.base_url {
        entry.model.base_url = Some(base_url);
    }
    if let Some(oversight) = update.requires_oversight {
        entry.requires_oversight = oversight;
    }
    if let Some(category) = update.category {
        entry.category = category;
    }
    if let Some(created_at) = update.created_at {
        entry.created_at = Some(created_at);
    }
    if let Some(last_pulse) = update.last_pulse {
        entry.heartbeat_at = Some(last_pulse);
    }
    if let Some(current_task) = update.current_task {
        entry.current_task = Some(current_task);
    }

    let mut token_usage_changed = false;
    if let Some(it) = update.input_tokens {
        entry.token_usage.input_tokens = it;
        token_usage_changed = true;
    }
    if let Some(ot) = update.output_tokens {
        entry.token_usage.output_tokens = ot;
        token_usage_changed = true;
    }
    if let Some(tt) = update.total_tokens {
        entry.token_usage.total_tokens = tt;
        entry.tokens_used = tt;
        token_usage_changed = false;
    } else if let Some(tokens_used) = update.tokens_used {
        entry.token_usage.total_tokens = tokens_used;
        entry.tokens_used = tokens_used;
        token_usage_changed = false;
    }
    if token_usage_changed {
        let total = entry.token_usage.input_tokens + entry.token_usage.output_tokens;
        entry.token_usage.total_tokens = total;
        entry.tokens_used = total;
    }
    // Sync to DB immediately
    let agent_clone = entry.clone();
    drop(entry); // Release RefMut before async DB call

    crate::agent::persistence::save_agent_db(&state.resources.pool, &agent_clone)
        .await
        .map_err(AppError::Anyhow)?;

    // Instant UI Update
    state.emit_event(serde_json::json!({
        "type": "agent:update",
        "agentId": agent_id,
        "data": agent_clone
    }));

    Ok(Json(serde_json::json!({ "status": "ok" })))
}

/// POST /agents/:id/pause
pub async fn pause_agent(
    Path(agent_id): Path<String>,
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, AppError> {
    let mut entry = state
        .registry
        .agents
        .get_mut(&agent_id)
        .ok_or_else(|| AppError::NotFound(format!("Agent {} not found", agent_id)))?;
    entry.status = "suspended".to_string();

    // Sync to DB immediately
    let agent_clone = entry.clone();
    drop(entry); // Release RefMut before async DB call

    crate::agent::persistence::save_agent_db(&state.resources.pool, &agent_clone)
        .await
        .map_err(AppError::Anyhow)?;

    state.emit_event(serde_json::json!({
        "type": "agent:update",
        "agentId": agent_id,
        "data": agent_clone
    }));

    Ok(Json(serde_json::json!({ "status": "ok" })))
}

/// POST /agents/:id/resume
pub async fn resume_agent(
    Path(agent_id): Path<String>,
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, AppError> {
    let mut entry = state
        .registry
        .agents
        .get_mut(&agent_id)
        .ok_or_else(|| AppError::NotFound(format!("Agent {} not found", agent_id)))?;
    entry.status = "idle".to_string();

    // Sync to DB immediately
    let agent_clone = entry.clone();
    drop(entry); // Release RefMut before async DB call

    crate::agent::persistence::save_agent_db(&state.resources.pool, &agent_clone)
        .await
        .map_err(AppError::Anyhow)?;

    state.emit_event(serde_json::json!({
        "type": "agent:update",
        "agentId": agent_id,
        "data": agent_clone
    }));

    Ok(Json(serde_json::json!({ "status": "ok" })))
}

/// POST /agents/:id/reset
///
/// @docs API_REFERENCE:ResetAgent
/// Resets an agent's failure count and returns it to idle status.
/// Used to clear "Self-heal cooldowns" after configuration fixes.
pub async fn reset_agent(
    Path(agent_id): Path<String>,
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, AppError> {
    let mut entry = state
        .registry
        .agents
        .get_mut(&agent_id)
        .ok_or_else(|| AppError::NotFound(format!("Agent {} not found", agent_id)))?;

    entry.failure_count = 0;
    entry.last_failure_at = None;
    entry.status = "idle".to_string();

    // Sync to DB immediately
    let agent_clone = entry.clone();
    drop(entry); // Release RefMut before async DB call

    crate::agent::persistence::save_agent_db(&state.resources.pool, &agent_clone)
        .await
        .map_err(AppError::Anyhow)?;

    state.emit_event(serde_json::json!({
        "type": "agent:update",
        "agentId": agent_id,
        "data": agent_clone
    }));

    Ok(Json(
        serde_json::json!({ "status": "ok", "message": "Failure count reset." }),
    ))
}

/// POST /agents/:id/mission
///
/// Synchronizes a mission objective to an agent's active mission state.
pub async fn sync_mission(
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
    Json(mission): Json<serde_json::Value>,
) -> Result<impl IntoResponse, AppError> {
    let mut agent = state
        .registry
        .agents
        .get_mut(&id)
        .ok_or_else(|| AppError::NotFound(format!("Agent {} not found", id)))?;

    agent.active_mission = Some(mission);

    // Sync to DB immediately
    let agent_clone = agent.clone();
    drop(agent); // Release RefMut before async DB call

    crate::agent::persistence::save_agent_db(&state.resources.pool, &agent_clone)
        .await
        .map_err(AppError::Anyhow)?;

    state.emit_event(serde_json::json!({
        "type": "agent:update",
        "agentId": id,
        "data": agent_clone
    }));

    Ok(Json(serde_json::json!({ "status": "ok" })))
}

/// GET /v1/agents/graph
///
/// Retrieves the complete knowledge graph of agents, missions, and their
/// relationships for real-time visualization in the dashboard.
pub async fn get_swarm_graph_handler(
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, AppError> {
    let graph = get_swarm_graph(&state.resources.pool)
        .await
        .map_err(AppError::Anyhow)?;
    Ok(Json(graph))
}

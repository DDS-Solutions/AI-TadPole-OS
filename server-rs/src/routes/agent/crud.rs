//! @docs ARCHITECTURE:Networking
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / HTTP Routes / Agent CRUD & Lifecycle
//! - **Primary Entrypoints**: `get_agents`, `get_agent`, `create_agent`, `update_agent`, `delete_agent`
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none declared
//! - **Telemetry Targets**: none declared
//! - **Witness Tests**: none declared

use super::models::{AgentResponse, CreateAgentRequest};
use crate::{
    agent::types::EngineAgent,
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
use std::sync::Arc;

/// Centralized persistence and broadcast helper for agent updates.
pub async fn update_and_persist_agent<F>(
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

        // Guard against concurrent memory mutation during the DB write window
        if agent_entry.version != current_version {
            return Err(AppError::Conflict(format!(
                "Concurrent modification detected in memory registry for agent {}. Version expected: {}, found: {}",
                agent_id, current_version, agent_entry.version
            )));
        }

        // Preserve live runtime fields from agent_entry if they weren't mutated by the closure `f`
        agent_clone.apply_runtime_state(&agent_entry, &original_agent);

        *agent_entry = agent_clone.clone();
        agent_clone
    };

    // Broadcast update
    state.emit_event(serde_json::json!({
        "type": "agent:update",
        "agent_id": agent_id,
        "data": AgentResponse::from(&final_agent)
    }));

    // Async background sync of registry memory to data/agents.json (Non-blocking I/O)
    let state_ref = state.clone();
    tokio::spawn(async move {
        let agents: Vec<EngineAgent> = state_ref
            .registry
            .agents
            .iter()
            .map(|kv| kv.value().clone())
            .collect();
        if let Err(e) =
            crate::agent::persistence::save_agents_json(&state_ref.base_dir, agents).await
        {
            tracing::error!(
                "⚠️ [Gateway] Failed to update agents.json asynchronously: {:?}",
                e
            );
        }
    });

    Ok(final_agent)
}

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
    let total = state.registry.agents.len() as u32;
    let offset = params.offset();
    let (_, per_page) = params.sanitize();

    let mut all_agents: Vec<AgentResponse> = state
        .registry
        .agents
        .iter()
        .map(|kv| AgentResponse::from(kv.value()))
        .collect();

    // Sort by stable key (`id`) before slicing to ensure deterministic pagination boundaries
    all_agents.sort_by(|a, b| a.id.cmp(&b.id));

    let agents: Vec<AgentResponse> = all_agents
        .into_iter()
        .skip(offset)
        .take(per_page as usize)
        .collect();

    Ok(Json(PaginatedResponse::from_pre_sliced(
        agents,
        total,
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

/// POST /agents
///
/// Registers a new agent in the system and triggers persistence.
#[tracing::instrument(skip(state, payload), fields(agent_id = %payload.id), name = "agent_registry::create")]
pub async fn create_agent(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateAgentRequest>,
) -> Result<impl IntoResponse, AppError> {
    let mut new_agent = payload.into_engine_agent()?;
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

    // ── Guard: Warn on Ollama agents missing rate limits ──────────────
    let is_ollama = matches!(
        new_agent.models.model.provider,
        crate::agent::types::ModelProvider::Ollama
    );
    let missing_limits =
        new_agent.models.model.tpm.is_none() || new_agent.models.model.rpm.is_none();
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

    if let Err(e) =
        crate::agent::persistence::save_agent_db(&state.resources.pool, &mut new_agent).await
    {
        if matches!(&e, AppError::Conflict(_))
            || e.to_string().contains("UNIQUE constraint failed")
            || e.to_string().contains("already exists")
        {
            return Err(AppError::Conflict(format!(
                "Agent '{}' already exists in database",
                agent_id
            )));
        }
        return Err(e);
    }

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

/// POST /v1/agents/:id/reset
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

/// DELETE /v1/agents/:id — Transactionally deletes an agent and cascades all metadata.
#[tracing::instrument(skip(state), name = "agent::delete_agent")]
pub async fn delete_agent(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let clean_id = crate::utils::security::sanitize_id(&id);
    if clean_id.is_empty() {
        return Err(AppError::BadRequest("Invalid agent id".to_string()));
    }

    // 0. Proactively abort any active runner task for the deleted agent
    if let Some((_, runner)) = state.comms.active_runners.remove(&clean_id) {
        tracing::info!(
            "🛑 [Gateway] Aborting active runner for deleted agent: {}",
            clean_id
        );
        runner.abort_handle.abort();
    }

    // 1. Remove from SQLite (cascading deletes across all dependent tables)
    crate::agent::persistence::delete_agent_cascade(&state.resources.pool, &clean_id).await?;

    // 2. Remove from the in-memory registry
    state.registry.agents.remove(&clean_id);

    // 3. Remove persisted JSON file from disk if present
    let workspace_root = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
    let agent_file = workspace_root
        .join("data/swarm_config/agents")
        .join(format!("{}.json", clean_id));
    if tokio::fs::try_exists(&agent_file).await.unwrap_or(false) {
        let _ = tokio::fs::remove_file(&agent_file).await;
    }

    Ok((
        StatusCode::OK,
        Json(serde_json::json!({
            "status": "success",
            "message": format!("Agent {} and all associated data deleted successfully.", clean_id)
        })),
    ))
}

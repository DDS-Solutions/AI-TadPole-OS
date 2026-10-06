//! @docs ARCHITECTURE:Gateways
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / HTTP Routes / Agent CRUD & Lifecycle
//! - **Primary Entrypoints**: `get_agents`, `get_agent`, `create_agent`, `update_agent`, `pause_agent`, `resume_agent`, `reset_agent`, `delete_agent`, `update_and_persist_agent`
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: `AppError::NotFound`, `AppError::Conflict`, `AppError::BadRequest`, `AppError::Sqlx`
//! - **Telemetry Targets**: `[Gateway]`, `[agent:create]`, `[agent:update]`, `[agent:delete]`

use super::models::{AgentResponse, CreateAgentRequest};
use crate::{
    agent::types::EngineAgent,
    error::AppError,
    routes::pagination::{PaginatedResponse, PaginationParams},
    state::AppState,
};
use axum::{
    extract::{Path, Query, State},
    http::{header::LOCATION, HeaderValue, StatusCode},
    response::IntoResponse,
    Json,
};
use std::sync::Arc;

/// Validates that an agent ID is non-empty, <= 64 characters, and consists
/// strictly of ASCII alphanumeric characters, dashes, and underscores.
pub fn validate_agent_id(id: &str) -> Result<&str, AppError> {
    let trimmed = id.trim();
    if trimmed.is_empty() || trimmed.len() > 64 {
        return Err(AppError::BadRequest(
            "Agent ID must be non-empty and <= 64 characters".into(),
        ));
    }
    if !trimmed
        .bytes()
        .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
    {
        return Err(AppError::BadRequest(
            "Agent ID must contain only ASCII alphanumeric characters, dashes, and underscores"
                .into(),
        ));
    }
    Ok(trimmed)
}

/// Centralized persistence and broadcast helper for agent updates.
/// Implements a claim-then-confirm protocol:
/// 1. Verifies existence and reserves the next version in-memory.
/// 2. Applies the update mutation.
/// 3. Persists to SQLite with atomic OCC (`WHERE version = current_version`).
///    If DB write fails or conflicts, memory reservation is rolled back.
/// 4. Confirms memory swap and preserves live runtime telemetry.
/// 5. Broadcasts `agent:update` and triggers background snapshot persistence.
pub async fn update_and_persist_agent<F>(
    state: &Arc<AppState>,
    agent_id: &str,
    f: F,
) -> Result<EngineAgent, AppError>
where
    F: FnOnce(&mut EngineAgent),
{
    let clean_id = validate_agent_id(agent_id)?;

    // 1. Retrieve current agent state and reserve next version in memory
    let (mut agent_clone, original_agent, current_version) = {
        let mut agent_entry = state
            .registry
            .agents
            .get_mut(clean_id)
            .ok_or_else(|| AppError::NotFound(format!("Agent {} not found", clean_id)))?;
        let cur_v = agent_entry.version;
        // Temporarily claim the next version in memory to prevent concurrent writers from reading stale version
        agent_entry.version = cur_v + 1;
        (agent_entry.clone(), agent_entry.clone(), cur_v)
    };

    // 2. Apply updates to the clone
    f(&mut agent_clone);
    // Reset version to current_version for DB matching (execute_save_agent matches WHERE version = ?)
    agent_clone.version = current_version;

    // 3. Attempt to save updated agent to SQLite.
    // On failure, rollback the memory version reservation so state does not diverge.
    if let Err(e) =
        crate::agent::persistence::save_agent_db(&state.resources.pool, &mut agent_clone).await
    {
        if let Some(mut agent_entry) = state.registry.agents.get_mut(clean_id) {
            if agent_entry.version == current_version + 1 {
                agent_entry.version = current_version;
            }
        }
        return Err(e);
    }

    // 4. DB write succeeded and bumped agent_clone.version to current_version + 1.
    // Confirm the memory swap while preserving concurrent runtime telemetry if untouched by `f`.
    let final_agent = {
        let mut agent_entry = state
            .registry
            .agents
            .get_mut(clean_id)
            .ok_or_else(|| AppError::NotFound(format!("Agent {} not found", clean_id)))?;

        agent_clone.apply_runtime_state(&agent_entry, &original_agent);
        *agent_entry = agent_clone.clone();
        agent_clone
    };

    // 5. Broadcast update event
    tracing::info!(
        "[Gateway] [agent:update] Agent {} updated to version {}",
        clean_id,
        final_agent.version
    );
    state.emit_event(serde_json::json!({
        "type": "agent:update",
        "agent_id": clean_id,
        "data": AgentResponse::from(&final_agent)
    }));

    // 6. Background async sync of registry memory to data/agents.json
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
    let clean_id = validate_agent_id(&agent_id)?;
    let agent = state
        .registry
        .agents
        .get(clean_id)
        .ok_or_else(|| AppError::NotFound(format!("Agent '{}' not found", clean_id)))?;

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
    validate_agent_id(&agent_id)?;

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
        let is_conflict = matches!(&e, AppError::Conflict(_))
            || matches!(&e, AppError::Sqlx(sqlx::Error::Database(ref db_err)) if db_err.is_unique_violation());
        if is_conflict {
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
    let location_header = HeaderValue::from_str(&agent_path)
        .map_err(|_| AppError::BadRequest("Failed to construct Location header".into()))?;

    tracing::info!(
        "[Gateway] [agent:create] Agent {} created successfully",
        agent_id
    );
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
        [(LOCATION, location_header)],
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
    let clean_id = validate_agent_id(&agent_id)?;

    // Numerical and input invariant validation
    if let Some(budget) = update.budget_usd {
        if budget < 0.0 || !budget.is_finite() {
            return Err(AppError::BadRequest(
                "budget_usd must be a non-negative finite number".into(),
            ));
        }
    }
    if let Some(base_url) = &update.base_url {
        let parsed = reqwest::Url::parse(base_url)
            .map_err(|_| AppError::BadRequest("base_url must be a valid absolute URL".into()))?;
        if parsed.scheme() != "http" && parsed.scheme() != "https" {
            return Err(AppError::BadRequest(
                "base_url must use http or https scheme".into(),
            ));
        }
        if !parsed.username().is_empty() || parsed.password().is_some() {
            return Err(AppError::BadRequest(
                "base_url credentials are not permitted in URL".into(),
            ));
        }
    }

    update_and_persist_agent(&state, clean_id, |agent| {
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
    let clean_id = validate_agent_id(&agent_id)?;

    update_and_persist_agent(&state, clean_id, |agent| {
        agent.pause();
    })
    .await?;

    // Zombie Task Termination & Terminal Mission Cleanup
    if let Some((_, runner)) = state.comms.active_runners.remove(clean_id) {
        tracing::info!(
            "🛑 [Gateway] Aborting active runner for suspended agent: {}",
            clean_id
        );
        runner.abort_handle.abort();

        // Mark running mission in mission_history terminal to prevent stranded active rows
        sqlx::query(
            "UPDATE mission_history SET status = 'cancelled', updated_at = ?1 WHERE agent_id = ?2 AND status IN ('pending', 'active')"
        )
        .bind(chrono::Utc::now())
        .bind(clean_id)
        .execute(&state.resources.pool)
        .await
        .map_err(AppError::Sqlx)?;
    }

    Ok(Json(serde_json::json!({
        "status": "ok",
        "agent_id": clean_id,
        "paused": true,
        "message": "Agent paused and active tasks cancelled"
    })))
}

/// POST /agents/:id/resume
#[tracing::instrument(skip(state), fields(agent_id = %agent_id), name = "agent_registry::resume")]
pub async fn resume_agent(
    Path(agent_id): Path<String>,
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, AppError> {
    let clean_id = validate_agent_id(&agent_id)?;

    update_and_persist_agent(&state, clean_id, |agent| {
        agent.resume();
    })
    .await?;

    Ok(Json(serde_json::json!({
        "status": "ok",
        "agent_id": clean_id,
        "state": "idle",
        "reopened_mission": false,
        "message": "Agent resumed to idle state. Dispatch a new task to continue work."
    })))
}

/// POST /v1/agents/:id/reset
///
/// @docs API_REFERENCE:ResetAgent
/// Resets an agent's failure count and returns it to idle status.
/// Used to clear "Self-heal cooldowns" after configuration fixes.
#[tracing::instrument(skip(state), fields(agent_id = %agent_id), name = "agent_registry::reset")]
pub async fn reset_agent(
    Path(agent_id): Path<String>,
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, AppError> {
    let clean_id = validate_agent_id(&agent_id)?;

    update_and_persist_agent(&state, clean_id, |agent| {
        agent.reset();
    })
    .await?;

    // Zombie Task Termination & Terminal Mission Cleanup on Reset
    if let Some((_, runner)) = state.comms.active_runners.remove(clean_id) {
        tracing::info!(
            "🛑 [Gateway] Aborting specific runner for reset agent: {}",
            clean_id
        );
        runner.abort_handle.abort();

        sqlx::query(
            "UPDATE mission_history SET status = 'cancelled', updated_at = ?1 WHERE agent_id = ?2 AND status IN ('pending', 'active')"
        )
        .bind(chrono::Utc::now())
        .bind(clean_id)
        .execute(&state.resources.pool)
        .await
        .map_err(AppError::Sqlx)?;
    }

    Ok(Json(
        serde_json::json!({ "status": "ok", "agent_id": clean_id, "message": "Failure count reset and tasks terminated." }),
    ))
}

/// DELETE /v1/agents/:id — Transactionally deletes an agent and cascades all metadata.
#[tracing::instrument(skip(state), fields(agent_id = %id), name = "agent::delete_agent")]
pub async fn delete_agent(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let clean_id = validate_agent_id(&id)?;

    // 0. Verify agent existence (prevent silent success on non-existent agent)
    let exists_in_memory = state.registry.agents.contains_key(clean_id);
    let exists_in_db = sqlx::query_scalar::<_, i64>("SELECT 1 FROM agents WHERE id = ?1")
        .bind(clean_id)
        .fetch_optional(&state.resources.pool)
        .await
        .map_err(AppError::Sqlx)?
        .is_some();

    if !exists_in_memory && !exists_in_db {
        return Err(AppError::NotFound(format!(
            "Agent '{}' not found",
            clean_id
        )));
    }

    // 1. Proactively abort any active runner task for the deleted agent
    if let Some((_, runner)) = state.comms.active_runners.remove(clean_id) {
        tracing::info!(
            "🛑 [Gateway] Aborting active runner for deleted agent: {}",
            clean_id
        );
        runner.abort_handle.abort();
    }

    // 2. Mark pending/active missions as cancelled in mission_history
    let _ = sqlx::query(
        "UPDATE mission_history SET status = 'cancelled', updated_at = ?1 WHERE agent_id = ?2 AND status IN ('pending', 'active')"
    )
    .bind(chrono::Utc::now())
    .bind(clean_id)
    .execute(&state.resources.pool)
    .await;

    // 3. Remove from SQLite (cascading deletes across all dependent tables)
    crate::agent::persistence::delete_agent_cascade(&state.resources.pool, clean_id).await?;

    // 4. Remove from the in-memory registry
    state.registry.agents.remove(clean_id);

    // 5. Remove legacy persisted JSON file from disk if present
    let legacy_file = state
        .base_dir
        .join("data/swarm_config/agents")
        .join(format!("{}.json", clean_id));
    if tokio::fs::try_exists(&legacy_file).await.unwrap_or(false) {
        let _ = tokio::fs::remove_file(&legacy_file).await;
    }

    // 6. Synchronize remaining agents to data/agents.json to prevent resurrection
    let remaining_agents: Vec<EngineAgent> = state
        .registry
        .agents
        .iter()
        .map(|kv| kv.value().clone())
        .collect();
    if let Err(e) =
        crate::agent::persistence::save_agents_json(&state.base_dir, remaining_agents).await
    {
        tracing::error!(
            "⚠️ [Gateway] Failed to update agents.json after agent deletion: {:?}",
            e
        );
    }

    // 7. Emit deletion event across WebSocket subscribers
    tracing::info!(
        "[Gateway] [agent:delete] Agent {} deleted successfully",
        clean_id
    );
    state.emit_event(serde_json::json!({
        "type": "agent:delete",
        "agent_id": clean_id,
    }));

    Ok((
        StatusCode::OK,
        Json(serde_json::json!({
            "status": "success",
            "message": format!("Agent {} and all associated data deleted successfully.", clean_id)
        })),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_agent_id_valid() {
        assert_eq!(validate_agent_id("agent_1").unwrap(), "agent_1");
        assert_eq!(validate_agent_id("AGENT-X-99").unwrap(), "AGENT-X-99");
        assert_eq!(
            validate_agent_id("a".repeat(64).as_str()).unwrap(),
            "a".repeat(64).as_str()
        );
    }

    #[test]
    fn test_validate_agent_id_invalid() {
        assert!(validate_agent_id("").is_err());
        assert!(validate_agent_id(" ").is_err());
        assert!(validate_agent_id("agent 1").is_err());
        assert!(validate_agent_id("café").is_err());
        assert!(validate_agent_id("../etc/passwd").is_err());
        assert!(validate_agent_id("agent/sub").is_err());
        assert!(validate_agent_id("a".repeat(65).as_str()).is_err());
    }

    #[test]
    fn test_location_header_visible_ascii() {
        let agent_path = format!("/v1/agents/{}", "agent-123_test");
        let header_res = HeaderValue::from_str(&agent_path);
        assert!(header_res.is_ok());
    }
}

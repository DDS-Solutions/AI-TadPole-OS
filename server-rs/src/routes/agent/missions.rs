//! @docs ARCHITECTURE:Networking
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / HTTP Routes / Agent Missions & Topology
//! - **Primary Entrypoints**: `sync_mission`, `clone_mission`, `get_swarm_graph_handler`
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none declared
//! - **Telemetry Targets**: none declared
//! - **Witness Tests**: none declared

use super::crud::update_and_persist_agent;
use super::models::CloneMissionRequest;
use crate::{
    agent::{
        mission::get_swarm_graph,
        runner::{AgentIdentity, PLANE2_DEPARTMENT_MARKER},
    },
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

fn reject_plane2_mission_access(state: &AppState, agent_id: &str) -> Result<(), AppError> {
    let is_plane2 = state
        .registry
        .agents
        .get(agent_id)
        .is_some_and(|agent| AgentIdentity::is_plane2_department(&agent.identity.department));

    if is_plane2 {
        tracing::warn!(
            target: "agent_plane_boundary",
            agent_id = %agent_id,
            marker = PLANE2_DEPARTMENT_MARKER,
            reason = "company_scoped_execution_pending",
            "Plane 2 mission operation denied"
        );
        return Err(AppError::Forbidden(format!(
            "{} missions cannot be accessed until company-scoped execution is implemented",
            PLANE2_DEPARTMENT_MARKER
        )));
    }

    Ok(())
}

/// POST /agents/:id/mission
///
/// Synchronizes a mission objective to an agent's active mission state.
///
/// NOTE: This persists the mission configuration on the agent entity. It does NOT
/// enqueue or start runner execution turns. To launch execution, dispatch a task
/// via POST /v1/agents/{id}/tasks.
#[tracing::instrument(skip(state, mission), fields(agent_id = %id), name = "agent_registry::sync_mission")]
pub async fn sync_mission(
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
    Json(mission): Json<serde_json::Value>,
) -> Result<impl IntoResponse, AppError> {
    reject_plane2_mission_access(&state, &id)?;

    update_and_persist_agent(&state, &id, |agent| {
        agent.set_mission(mission);
    })
    .await?;

    Ok(Json(serde_json::json!({
        "status": "synchronized",
        "executed": false,
        "agent_id": id,
        "message": "Mission definition saved to agent state; dispatch via /tasks to start execution"
    })))
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

/// Clones an existing mission into a fresh record with a unique UUID.
///
/// The cloned mission starts in "pending" draft status with zeroed financial spend
/// and does not automatically start execution or copy prior findings/logs.
pub async fn clone_mission(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(payload): Json<CloneMissionRequest>,
) -> Result<impl IntoResponse, AppError> {
    // Read only the owner ID first; do not load the source mission body until
    // its agent has passed the Plane 2 boundary check.
    let source_agent_id: Option<String> =
        sqlx::query_scalar("SELECT agent_id FROM mission_history WHERE id = ?1")
            .bind(&id)
            .fetch_optional(&state.resources.pool)
            .await?
            .flatten();
    let source_agent_id = source_agent_id
        .filter(|agent_id| !agent_id.is_empty())
        .ok_or_else(|| AppError::NotFound(format!("Mission {} not found", id)))?;

    if !state.registry.agents.contains_key(&source_agent_id) {
        return Err(AppError::Forbidden(
            "Mission owner cannot be verified".to_string(),
        ));
    }
    reject_plane2_mission_access(&state, &source_agent_id)?;

    let source_mission = crate::agent::mission::get_mission_by_id(&state.resources.pool, &id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Mission {} not found", id)))?;

    // Recheck the persisted owner before copying in case the mission row changed
    // between the metadata lookup and full record read.
    if source_mission.agent_id != source_agent_id {
        return Err(AppError::Forbidden(
            "Mission owner changed while preparing the clone".to_string(),
        ));
    }
    reject_plane2_mission_access(&state, &source_mission.agent_id)?;

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

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
use crate::{agent::mission::get_swarm_graph, error::AppError, state::AppState};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use std::sync::Arc;

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

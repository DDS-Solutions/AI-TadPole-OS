use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde_json::json;
use std::sync::Arc;

use crate::agent::capabilities::{SkillDefinition, WorkflowDefinition};
use crate::routes::error::AppError;
use crate::state::AppState;

/// GET /v1/capabilities
/// 
/// Discovers all registered unit skills and multi-step workflows. This serves
/// as the primary inventory for the engine's cognitive abilities.
///
/// @docs API_REFERENCE:ListCapabilities
pub async fn list_capabilities(State(state): State<Arc<AppState>>) -> Result<impl IntoResponse, AppError> {
    let mut skills = Vec::new();
    for kv in state.registry.capabilities.skills.iter() {
        skills.push(kv.value().clone());
    }

    let mut workflows = Vec::new();
    for kv in state.registry.capabilities.workflows.iter() {
        workflows.push(kv.value().clone());
    }

    Ok(Json(json!({
        "skills": skills,
        "workflows": workflows
    })))
}

/// POST /v1/capabilities/skills
/// 
/// Registers or updates a skill definition. Skills define discrete tool
/// capabilities that agents can utilize during a mission.
///
/// @docs API_REFERENCE:PostSkill
pub async fn post_skill(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<SkillDefinition>,
) -> Result<impl IntoResponse, AppError> {
    match state.registry.capabilities.save_skill(payload.clone()).await {
        Ok(_) => {
            state.broadcast_sys(&format!("Updated skill: {}", payload.name), "success");
            Ok((StatusCode::OK, Json(json!({ "status": "saved" }))))
        }
        Err(e) => Err(AppError::InternalServerError(format!("Failed to save skill: {}", e))),
    }
}

/// DELETE /v1/capabilities/skills/:name
pub async fn delete_skill(
    Path(name): Path<String>,
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, AppError> {
    match state.registry.capabilities.delete_skill(&name).await {
        Ok(_) => {
            state.broadcast_sys(&format!("Deleted skill: {}", name), "warning");
            Ok((StatusCode::OK, Json(json!({ "status": "deleted" }))))
        }
        Err(e) => Err(AppError::InternalServerError(format!("Failed to delete skill: {}", e))),
    }
}

/// POST /v1/capabilities/workflows
pub async fn post_workflow(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<WorkflowDefinition>,
) -> Result<impl IntoResponse, AppError> {
    match state.registry.capabilities.save_workflow(payload.clone()).await {
        Ok(_) => {
            state.broadcast_sys(&format!("Updated workflow: {}", payload.name), "success");
            Ok((StatusCode::OK, Json(json!({ "status": "saved" }))))
        }
        Err(e) => Err(AppError::InternalServerError(format!("Failed to save workflow: {}", e))),
    }
}

/// DELETE /v1/capabilities/workflows/:name
pub async fn delete_workflow(
    Path(name): Path<String>,
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, AppError> {
    match state.registry.capabilities.delete_workflow(&name).await {
        Ok(_) => {
            state.broadcast_sys(&format!("Deleted workflow: {}", name), "warning");
            Ok((StatusCode::OK, Json(json!({ "status": "deleted" }))))
        }
        Err(e) => Err(AppError::InternalServerError(format!("Failed to delete workflow: {}", e))),
    }
}

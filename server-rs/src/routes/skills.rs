//! @docs ARCHITECTURE:Networking
//! 
//! ### AI Assist Note
//! **Capability Hub**: Unified endpoint for all agent abilities, including 
//! **Tool Manifests** (JSON schemas), **Executable Scripts**, and 
//! **Multi-step Workflows**. Features a **Multipart Capability Importer** 
//! that autonomously parses Markdown frontmatter and script extensions 
//! (ps1, sh, py) to register new agentic skills.
//! 
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: 400 Bad Request on invalid multipart boundaries, 
//!   500 Internal Server Error during script persistence to disk, or 
//!   silent failure of YAML frontmatter parsing in imported skills.
//! - **Trace Scope**: `server-rs::routes::skills`

use axum::{

    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde_json::json;
use std::sync::Arc;

use crate::agent::script_skills::{HookDefinition, SkillDefinition, WorkflowDefinition};
use crate::agent::skill_manifest::SkillManifest;
use crate::routes::error::AppError;
use crate::state::AppState;

/// GET /v1/skills
///
/// Unified endpoint for all agent abilities:
/// - Manifests: JSON schemas for LLM tool-calling.
/// - Scripts: Executable file-based skills.
/// - Workflows: Multi-step passive sequences.
#[tracing::instrument(skip(state), name = "capability_registry::get_all")]
pub async fn list_all_skills(
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, AppError> {
    // 1. Collect Manifests
    let mut manifests: Vec<SkillManifest> = state
        .registry
        .skill_registry
        .manifests
        .values()
        .cloned()
        .collect();
    manifests.sort_by(|a, b| a.name.cmp(&b.name));

    // 2. Collect Script Skills
    let mut scripts = Vec::new();
    for kv in state.registry.skills.skills.iter() {
        scripts.push(kv.value().clone());
    }

    // 3. Collect Workflows
    let mut workflows = Vec::new();
    for kv in state.registry.skills.workflows.iter() {
        workflows.push(kv.value().clone());
    }

    // 4. Collect Hooks
    let mut hooks = Vec::new();
    for kv in state.registry.skills.hooks.iter() {
        hooks.push(kv.value().clone());
    }

    Ok(Json(json!({
        "manifests": manifests,
        "scripts": scripts,
        "workflows": workflows,
        "hooks": hooks
    })))
}

/// GET /v1/skills/manifests
/// Legacy/Narrow endpoint for just manifests (compatible with old MissionApiService GET /v1/skills)
#[tracing::instrument(skip(state), name = "capability_registry::list_manifests")]
pub async fn list_manifests(
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, AppError> {
    let mut manifests: Vec<SkillManifest> = state
        .registry
        .skill_registry
        .manifests
        .values()
        .cloned()
        .collect();
    manifests.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(Json(manifests))
}

/// GET /v1/skills/manifests/:name
#[tracing::instrument(skip(state), fields(name = %name), name = "capability_registry::get_manifest")]
pub async fn get_manifest(
    State(state): State<Arc<AppState>>,
    Path(name): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    if let Some(manifest) = state.registry.skill_registry.get(&name) {
        Ok(Json(manifest.clone()))
    } else {
        Err(AppError::NotFound(format!(
            "Skill manifest '{}' not found",
            name
        )))
    }
}

/// POST /v1/skills/scripts
#[tracing::instrument(skip(state, payload), fields(name = %payload.name), name = "capability_registry::save_script")]
pub async fn post_script(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<SkillDefinition>,
) -> Result<impl IntoResponse, AppError> {
    match state.registry.skills.save_skill(payload.clone()).await {
        Ok(_) => {
            state.broadcast_sys(
                &format!("Updated skill script: {}", payload.name),
                "success",
                None,
            );
            Ok((StatusCode::OK, Json(json!({ "status": "saved" }))))
        }
        Err(e) => Err(AppError::InternalServerError(format!(
            "Failed to save script: {}",
            e
        ))),
    }
}

/// DELETE /v1/skills/scripts/:name
#[tracing::instrument(skip(state), fields(name = %name), name = "capability_registry::delete_script")]
pub async fn delete_script(
    Path(name): Path<String>,
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, AppError> {
    match state.registry.skills.delete_skill(&name).await {
        Ok(_) => {
            state.broadcast_sys(&format!("Deleted script: {}", name), "warning", None);
            Ok((StatusCode::OK, Json(json!({ "status": "deleted" }))))
        }
        Err(e) => Err(AppError::InternalServerError(format!(
            "Failed to delete script: {}",
            e
        ))),
    }
}

/// POST /v1/skills/workflows
#[tracing::instrument(skip(state, payload), fields(name = %payload.name), name = "capability_registry::save_workflow")]
pub async fn post_workflow(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<WorkflowDefinition>,
) -> Result<impl IntoResponse, AppError> {
    match state.registry.skills.save_workflow(payload.clone()).await {
        Ok(_) => {
            state.broadcast_sys(
                &format!("Updated workflow: {}", payload.name),
                "success",
                None,
            );
            Ok((StatusCode::OK, Json(json!({ "status": "saved" }))))
        }
        Err(e) => Err(AppError::InternalServerError(format!(
            "Failed to save workflow: {}",
            e
        ))),
    }
}

/// DELETE /v1/skills/workflows/:name
#[tracing::instrument(skip(state), fields(name = %name), name = "capability_registry::delete_workflow")]
pub async fn delete_workflow(
    Path(name): Path<String>,
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, AppError> {
    match state.registry.skills.delete_workflow(&name).await {
        Ok(_) => {
            state.broadcast_sys(&format!("Deleted workflow: {}", name), "warning", None);
            Ok((StatusCode::OK, Json(json!({ "status": "deleted" }))))
        }
        Err(e) => Err(AppError::InternalServerError(format!(
            "Failed to delete workflow: {}",
            e
        ))),
    }
}

/// POST /v1/skills/hooks
#[tracing::instrument(skip(state, payload), fields(name = %payload.name), name = "capability_registry::save_hook")]
pub async fn post_hook(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<HookDefinition>,
) -> Result<impl IntoResponse, AppError> {
    match state.registry.skills.save_hook(payload.clone()).await {
        Ok(_) => {
            state.broadcast_sys(
                &format!("Updated lifecycle hook: {}", payload.name),
                "success",
                None,
            );
            Ok((StatusCode::OK, Json(json!({ "status": "saved" }))))
        }
        Err(e) => Err(AppError::InternalServerError(format!(
            "Failed to save hook: {}",
            e
        ))),
    }
}

/// DELETE /v1/skills/hooks/:name
#[tracing::instrument(skip(state), fields(name = %name), name = "capability_registry::delete_hook")]
pub async fn delete_hook(
    Path(name): Path<String>,
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, AppError> {
    match state.registry.skills.delete_hook(&name).await {
        Ok(_) => {
            state.broadcast_sys(
                &format!("Deleted lifecycle hook: {}", name),
                "warning",
                None,
            );
            Ok((StatusCode::OK, Json(json!({ "status": "deleted" }))))
        }
        Err(e) => Err(AppError::InternalServerError(format!(
            "Failed to delete hook: {}",
            e
        ))),
    }
}

/// POST /v1/skills/import
/// Accepts a markdown file or script, parses it, and returns a structured preview.
#[tracing::instrument(skip(_state, multipart), name = "capability_hub::import")]
pub async fn import_capability(
    State(_state): State<Arc<AppState>>,
    mut multipart: axum::extract::Multipart,
) -> Result<impl IntoResponse, AppError> {
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| AppError::InternalServerError(format!("Multipart error: {}", e)))?
    {
        let name = field.name().unwrap_or("file").to_string();
        let file_name = field.file_name().unwrap_or("unknown.md").to_string();
        let content = field
            .text()
            .await
            .map_err(|e| AppError::InternalServerError(format!("Failed to read field: {}", e)))?;

        if name == "file" {
            if file_name.ends_with(".md") {
                // Parse as Skill or Workflow
                if let Some(skill) = crate::agent::script_skills::parse_skill_md(&content) {
                    return Ok(Json(json!({
                        "type": "skill",
                        "data": skill,
                        "preview": content
                    })));
                } else {
                    // Treat as basic Workflow
                    let workflow = WorkflowDefinition {
                        id: None,
                        name: file_name.trim_end_matches(".md").to_string(),
                        content: content.clone(),
                        doc_url: None,
                        tags: None,
                        category: "user".to_string(),
                    };
                    return Ok(Json(json!({
                        "type": "workflow",
                        "data": workflow,
                        "preview": content
                    })));
                }
            } else {
                // Treat as Hook potentially (script extensions)
                let ext = std::path::Path::new(&file_name)
                    .extension()
                    .and_then(|e| e.to_str())
                    .unwrap_or("");
                if ["ps1", "sh", "bat", "exe", "py"].contains(&ext) {
                    let hook = HookDefinition {
                        name: file_name.clone(),
                        description: format!("Imported hook from {}", file_name),
                        hook_type: "post-tool".to_string(), // Default
                        content: content.clone(),
                        active: true,
                        category: "user".to_string(),
                    };
                    return Ok(Json(json!({
                        "type": "hook",
                        "data": hook,
                        "preview": content
                    })));
                }
            }
        }
    }

    Err(AppError::BadRequest(
        "No valid file found in multipart request".to_string(),
    ))
}

#[derive(serde::Deserialize, Debug)]
pub struct RegisterPayload {
    pub r#type: String,
    pub data: serde_json::Value,
    pub category: String,
}

/// POST /v1/skills/register
/// Finalizes registration of a parsed capability.
#[tracing::instrument(skip(state, payload), fields(capability_type = %payload.r#type), name = "capability_registry::register")]
pub async fn register_capability(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<RegisterPayload>,
) -> Result<impl IntoResponse, AppError> {
    match state
        .registry
        .skills
        .register_capability(&payload.r#type, payload.data, &payload.category)
        .await
    {
        Ok(name) => {
            state.broadcast_sys(
                &format!("Registered new {}: {}", payload.r#type, name),
                "success",
                None,
            );
            Ok((
                StatusCode::OK,
                Json(json!({ "status": "registered", "name": name })),
            ))
        }
        Err(e) => Err(AppError::InternalServerError(format!(
            "Failed to register capability: {}",
            e
        ))),
    }
}

/// GET /v1/skills/proposals
///
/// Returns all pending capability proposals (skills, workflows, hooks) submitted by agents.
#[tracing::instrument(skip(state), name = "capability_registry::list_proposals")]
pub async fn list_capability_proposals(
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, AppError> {
    let rows = sqlx::query(
        "SELECT id, mission_id, agent_id, cap_type, name, description, payload, created_at, status 
         FROM capability_proposals 
         WHERE status = 'pending' 
         ORDER BY created_at DESC"
    )
    .fetch_all(&state.resources.pool)
    .await
    .map_err(|e| AppError::InternalServerError(format!("Database error: {}", e)))?;

    let proposals: Vec<serde_json::Value> = rows.into_iter().map(|row| {
        use sqlx::Row;
        serde_json::json!({
            "id": row.get::<i64, _>("id"),
            "mission_id": row.get::<Option<String>, _>("mission_id"),
            "agent_id": row.get::<String, _>("agent_id"),
            "type": row.get::<String, _>("cap_type"),
            "name": row.get::<String, _>("name"),
            "description": row.get::<String, _>("description"),
            "payload": serde_json::from_str::<serde_json::Value>(&row.get::<String, _>("payload")).unwrap_or_default(),
            "created_at": row.get::<chrono::DateTime<chrono::Utc>, _>("created_at"),
            "status": row.get::<String, _>("status"),
        })
    }).collect();

    Ok(Json(proposals))
}

#[derive(serde::Deserialize, Debug)]
pub struct ResolveProposalPayload {
    pub decision: String, // "approved" or "rejected"
}

/// POST /v1/skills/proposals/:id/resolve
///
/// Finalizes a capability proposal. If approved, it is physically written to the agent_generated directory.
#[tracing::instrument(skip(state, payload), fields(proposal_id = %proposal_id, decision = %payload.decision), name = "capability_registry::resolve_proposal")]
pub async fn resolve_capability_proposal(
    Path(proposal_id): Path<i64>,
    State(state): State<Arc<AppState>>,
    Json(payload): Json<ResolveProposalPayload>,
) -> Result<impl IntoResponse, AppError> {
    let approved = payload.decision == "approved";

    // 1. Fetch proposal
    let row: (String, String, String, String) = sqlx::query_as(
        "SELECT cap_type, name, payload, agent_id FROM capability_proposals WHERE id = ?"
    )
    .bind(proposal_id)
    .fetch_one(&state.resources.pool)
    .await
    .map_err(|_| AppError::NotFound("Proposal not found".to_string()))?;

    let (cap_type, name, payload_str, _agent_id) = row;
    let data: serde_json::Value = serde_json::from_str(&payload_str)
        .map_err(|e| AppError::InternalServerError(format!("Invalid proposal payload: {}", e)))?;

    // 2. If approved, physically register it
    if approved {
        state.registry.skills.register_capability(&cap_type, data, "agent_generated").await
            .map_err(|e| AppError::InternalServerError(format!("Failed to register capability: {}", e)))?;

        state.broadcast_sys(
            &format!("✅ Autonomous Capability Approved: {} ({})", name, cap_type),
            "success",
            None,
        );
    }

    // 3. Update status in DB
    sqlx::query("UPDATE capability_proposals SET status = ? WHERE id = ?")
        .bind(if approved { "approved" } else { "rejected" })
        .bind(proposal_id)
        .execute(&state.resources.pool)
        .await
        .map_err(|e| AppError::InternalServerError(format!("Failed to update proposal status: {}", e)))?;

    Ok(Json(json!({ "status": "ok", "decision": payload.decision })))
}

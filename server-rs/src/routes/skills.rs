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
pub async fn list_all_skills(State(state): State<Arc<AppState>>) -> Result<impl IntoResponse, AppError> {
    // 1. Collect Manifests
    let mut manifests: Vec<SkillManifest> = state.registry.skill_registry.manifests.values().cloned().collect();
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
pub async fn list_manifests(State(state): State<Arc<AppState>>) -> Result<Json<Vec<SkillManifest>>, AppError> {
    let mut manifests: Vec<SkillManifest> = state.registry.skill_registry.manifests.values().cloned().collect();
    manifests.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(Json(manifests))
}

/// GET /v1/skills/manifests/:name
pub async fn get_manifest(
    State(state): State<Arc<AppState>>,
    Path(name): Path<String>,
) -> Result<Json<SkillManifest>, AppError> {
    if let Some(manifest) = state.registry.skill_registry.get(&name) {
        Ok(Json(manifest.clone()))
    } else {
        Err(AppError::NotFound(format!("Skill manifest '{}' not found", name)))
    }
}

/// POST /v1/skills/scripts
pub async fn post_script(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<SkillDefinition>,
) -> Result<impl IntoResponse, AppError> {
    match state.registry.skills.save_skill(payload.clone()).await {
        Ok(_) => {
            state.broadcast_sys(&format!("Updated skill script: {}", payload.name), "success");
            Ok((StatusCode::OK, Json(json!({ "status": "saved" }))))
        }
        Err(e) => Err(AppError::InternalServerError(format!("Failed to save script: {}", e))),
    }
}

/// DELETE /v1/skills/scripts/:name
pub async fn delete_script(
    Path(name): Path<String>,
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, AppError> {
    match state.registry.skills.delete_skill(&name).await {
        Ok(_) => {
            state.broadcast_sys(&format!("Deleted script: {}", name), "warning");
            Ok((StatusCode::OK, Json(json!({ "status": "deleted" }))))
        }
        Err(e) => Err(AppError::InternalServerError(format!("Failed to delete script: {}", e))),
    }
}

/// POST /v1/skills/workflows
pub async fn post_workflow(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<WorkflowDefinition>,
) -> Result<impl IntoResponse, AppError> {
    match state.registry.skills.save_workflow(payload.clone()).await {
        Ok(_) => {
            state.broadcast_sys(&format!("Updated workflow: {}", payload.name), "success");
            Ok((StatusCode::OK, Json(json!({ "status": "saved" }))))
        }
        Err(e) => Err(AppError::InternalServerError(format!("Failed to save workflow: {}", e))),
    }
}

/// DELETE /v1/skills/workflows/:name
pub async fn delete_workflow(
    Path(name): Path<String>,
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, AppError> {
    match state.registry.skills.delete_workflow(&name).await {
        Ok(_) => {
            state.broadcast_sys(&format!("Deleted workflow: {}", name), "warning");
            Ok((StatusCode::OK, Json(json!({ "status": "deleted" }))))
        }
        Err(e) => Err(AppError::InternalServerError(format!("Failed to delete workflow: {}", e))),
    }
}

/// POST /v1/skills/hooks
pub async fn post_hook(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<HookDefinition>,
) -> Result<impl IntoResponse, AppError> {
    match state.registry.skills.save_hook(payload.clone()).await {
        Ok(_) => {
            state.broadcast_sys(&format!("Updated lifecycle hook: {}", payload.name), "success");
            Ok((StatusCode::OK, Json(json!({ "status": "saved" }))))
        }
        Err(e) => Err(AppError::InternalServerError(format!("Failed to save hook: {}", e))),
    }
}

/// DELETE /v1/skills/hooks/:name
pub async fn delete_hook(
    Path(name): Path<String>,
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, AppError> {
    match state.registry.skills.delete_hook(&name).await {
        Ok(_) => {
            state.broadcast_sys(&format!("Deleted lifecycle hook: {}", name), "warning");
            Ok((StatusCode::OK, Json(json!({ "status": "deleted" }))))
        }
        Err(e) => Err(AppError::InternalServerError(format!("Failed to delete hook: {}", e))),
    }
}

/// POST /v1/skills/import
/// Accepts a markdown file or script, parses it, and returns a structured preview.
pub async fn import_capability(
    State(_state): State<Arc<AppState>>,
    mut multipart: axum::extract::Multipart,
) -> Result<impl IntoResponse, AppError> {
    while let Some(field) = multipart.next_field().await.map_err(|e| AppError::InternalServerError(format!("Multipart error: {}", e)))? {
        let name = field.name().unwrap_or("file").to_string();
        let file_name = field.file_name().unwrap_or("unknown.md").to_string();
        let content = field.text().await.map_err(|e| AppError::InternalServerError(format!("Failed to read field: {}", e)))?;

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
                let ext = std::path::Path::new(&file_name).extension().and_then(|e| e.to_str()).unwrap_or("");
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

    Err(AppError::BadRequest("No valid file found in multipart request".to_string()))
}

#[derive(serde::Deserialize)]
pub struct RegisterPayload {
    pub r#type: String,
    pub data: serde_json::Value,
    pub category: String,
}

/// POST /v1/skills/register
/// Finalizes registration of a parsed capability.
pub async fn register_capability(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<RegisterPayload>,
) -> Result<impl IntoResponse, AppError> {
    match state.registry.skills.register_capability(&payload.r#type, payload.data, &payload.category).await {
        Ok(name) => {
            state.broadcast_sys(&format!("Registered new {}: {}", payload.r#type, name), "success");
            Ok((StatusCode::OK, Json(json!({ "status": "registered", "name": name }))))
        }
        Err(e) => Err(AppError::InternalServerError(format!("Failed to register capability: {}", e))),
    }
}

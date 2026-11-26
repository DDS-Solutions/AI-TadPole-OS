//! @docs ARCHITECTURE:Networking
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / HTTP Routes / skills
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Type-safe state handling and bounded execution without unhandled panics.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: none declared
//! - **Witness Tests**: none declared

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
use crate::error::AppError;
use crate::state::AppState;
use serde::Serialize;

/// Discriminator for agent capabilities.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum CapabilityType {
    Script,
    Workflow,
    Hook,
    Manifest,
    Unknown,
}

impl From<&std::path::Path> for CapabilityType {
    fn from(path: &std::path::Path) -> Self {
        match path
            .extension()
            .and_then(|s| s.to_str())
            .map(|s| s.to_ascii_lowercase())
            .as_deref()
        {
            Some("ps1") | Some("sh") | Some("py") | Some("bat") => Self::Script,
            Some("yaml") | Some("yml") => Self::Workflow,
            Some("json") => Self::Manifest,
            Some("md") => Self::Unknown, // Requires content parsing
            _ => Self::Unknown,
        }
    }
}

/// GET /v1/skills
///
/// Unified endpoint for all agent abilities:
/// - Manifests: JSON schemas for LLM tool-calling.
/// - Scripts: Executable file-based skills.
/// - Workflows: Multi-step passive sequences.
#[tracing::instrument(skip(state), fields(trace_id = tracing::field::Empty, request_id = tracing::field::Empty, http_status = tracing::field::Empty), name = "capability_registry::get_all")]
pub async fn list_all_skills(
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, AppError> {
    let mut manifests = Vec::new();
    let mut scripts = Vec::new();
    let mut workflows = Vec::new();
    let mut hooks = Vec::new();

    // 1. Collect Manifests
    for kv in state.registry.skill_registry.manifests.iter() {
        manifests.push(kv.value().clone());
    }

    // 2. Collect Script Skills
    for kv in state.registry.skills.skills.iter() {
        scripts.push(kv.value().clone());
    }

    // 3. Collect Workflows
    for kv in state.registry.skills.workflows.iter() {
        workflows.push(kv.value().clone());
    }

    // 4. Collect Hooks
    for kv in state.registry.skills.hooks.iter() {
        hooks.push(kv.value().clone());
    }

    // Sort all arrays by name to keep display deterministic
    manifests.sort_by(|a, b| a.name.cmp(&b.name));
    scripts.sort_by(|a, b| a.name.cmp(&b.name));
    workflows.sort_by(|a, b| a.name.cmp(&b.name));
    hooks.sort_by(|a, b| a.name.cmp(&b.name));

    Ok(Json(json!({
        "manifests": manifests,
        "scripts": scripts,
        "workflows": workflows,
        "hooks": hooks
    })))
}

/// GET /v1/skills/manifests
/// Legacy/Narrow endpoint for just manifests (compatible with old MissionApiService GET /v1/skills)
#[tracing::instrument(skip(state), fields(trace_id = tracing::field::Empty, request_id = tracing::field::Empty, http_status = tracing::field::Empty), name = "capability_registry::list_manifests")]
pub async fn list_manifests(
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, AppError> {
    let mut manifests: Vec<SkillManifest> = state
        .registry
        .skill_registry
        .manifests
        .iter()
        .map(|kv| kv.value().clone())
        .collect();
    manifests.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(Json(manifests))
}

/// GET /v1/skills/manifests/:name
#[tracing::instrument(skip(state), fields(name = %name, trace_id = tracing::field::Empty, request_id = tracing::field::Empty, http_status = tracing::field::Empty), name = "capability_registry::get_manifest")]
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

fn reconcile_capability_name(path_name: &str, payload_name: &mut String) -> Result<(), AppError> {
    let trimmed = path_name.trim();
    if trimmed.is_empty() {
        return Err(AppError::BadRequest(
            "Capability URL path name cannot be empty".to_string(),
        ));
    }
    if !payload_name.trim().is_empty() && payload_name.trim() != trimmed {
        return Err(AppError::BadRequest(format!(
            "Capability URL path name '{}' does not match payload name '{}'",
            trimmed, payload_name
        )));
    }
    *payload_name = trimmed.to_string();
    Ok(())
}

/// PUT /v1/skills/scripts/:name
#[tracing::instrument(skip(state, _admin, payload), fields(name = %name), name = "capability_registry::save_script")]
pub async fn post_script(
    Path(name): Path<String>,
    State(state): State<Arc<AppState>>,
    _admin: crate::middleware::auth::RequireAdmin,
    Json(mut payload): Json<SkillDefinition>,
) -> Result<impl IntoResponse, AppError> {
    reconcile_capability_name(&name, &mut payload.name)?;
    match state.registry.skills.save_skill(payload.clone()).await {
        Ok(_) => {
            state.broadcast_sys(
                &format!("Updated skill script: {}", payload.name),
                "success",
                None,
            );
            Ok((StatusCode::OK, Json(json!({ "status": "saved" }))))
        }
        Err(e) => Err(e),
    }
}

/// DELETE /v1/skills/scripts/:name
#[tracing::instrument(skip(state, _admin), fields(name = %name), name = "capability_registry::delete_script")]
pub async fn delete_script(
    Path(name): Path<String>,
    State(state): State<Arc<AppState>>,
    _admin: crate::middleware::auth::RequireAdmin,
) -> Result<impl IntoResponse, AppError> {
    match state.registry.skills.delete_skill(&name).await {
        Ok(_) => {
            state.broadcast_sys(&format!("Deleted script: {}", name), "warning", None);
            Ok((StatusCode::OK, Json(json!({ "status": "deleted" }))))
        }
        Err(e) => Err(e),
    }
}

/// PUT /v1/skills/workflows/:name
#[tracing::instrument(skip(state, _admin, payload), fields(name = %name), name = "capability_registry::save_workflow")]
pub async fn post_workflow(
    Path(name): Path<String>,
    State(state): State<Arc<AppState>>,
    _admin: crate::middleware::auth::RequireAdmin,
    Json(mut payload): Json<WorkflowDefinition>,
) -> Result<impl IntoResponse, AppError> {
    reconcile_capability_name(&name, &mut payload.name)?;
    match state.registry.skills.save_workflow(payload.clone()).await {
        Ok(_) => {
            state.broadcast_sys(
                &format!("Updated workflow: {}", payload.name),
                "success",
                None,
            );
            Ok((StatusCode::OK, Json(json!({ "status": "saved" }))))
        }
        Err(e) => Err(e),
    }
}

/// DELETE /v1/skills/workflows/:name
#[tracing::instrument(skip(state, _admin), fields(name = %name), name = "capability_registry::delete_workflow")]
pub async fn delete_workflow(
    Path(name): Path<String>,
    State(state): State<Arc<AppState>>,
    _admin: crate::middleware::auth::RequireAdmin,
) -> Result<impl IntoResponse, AppError> {
    match state.registry.skills.delete_workflow(&name).await {
        Ok(_) => {
            state.broadcast_sys(&format!("Deleted workflow: {}", name), "warning", None);
            Ok((StatusCode::OK, Json(json!({ "status": "deleted" }))))
        }
        Err(e) => Err(e),
    }
}

/// PUT /v1/skills/hooks/:name
#[tracing::instrument(skip(state, _admin, payload), fields(name = %name), name = "capability_registry::save_hook")]
pub async fn post_hook(
    Path(name): Path<String>,
    State(state): State<Arc<AppState>>,
    _admin: crate::middleware::auth::RequireAdmin,
    Json(mut payload): Json<HookDefinition>,
) -> Result<impl IntoResponse, AppError> {
    reconcile_capability_name(&name, &mut payload.name)?;
    match state.registry.skills.save_hook(payload.clone()).await {
        Ok(_) => {
            state.broadcast_sys(
                &format!("Updated lifecycle hook: {}", payload.name),
                "success",
                None,
            );
            Ok((StatusCode::OK, Json(json!({ "status": "saved" }))))
        }
        Err(e) => Err(e),
    }
}

/// DELETE /v1/skills/hooks/:name
#[tracing::instrument(skip(state, _admin), fields(name = %name), name = "capability_registry::delete_hook")]
pub async fn delete_hook(
    Path(name): Path<String>,
    State(state): State<Arc<AppState>>,
    _admin: crate::middleware::auth::RequireAdmin,
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
        Err(e) => Err(e),
    }
}

pub const MAX_CAPABILITY_IMPORT_BYTES: usize = 25 * 1024 * 1024; // 25 MB

/// POST /v1/skills/import
/// Accepts a markdown file or script, parses it, and returns a structured preview.
#[tracing::instrument(skip(_state, _admin, multipart), name = "capability_hub::import")]
pub async fn import_capability(
    State(_state): State<Arc<AppState>>,
    _admin: crate::middleware::auth::RequireAdmin,
    mut multipart: axum::extract::Multipart,
) -> Result<impl IntoResponse, AppError> {
    while let Some(mut field) = multipart
        .next_field()
        .await
        .map_err(|e| AppError::InternalServerError(format!("Multipart error: {}", e)))?
    {
        let name = field.name().unwrap_or("file").to_string();
        let file_name = field.file_name().unwrap_or("unknown.md").to_string();

        let mut body_bytes = Vec::new();
        while let Some(chunk) = field
            .chunk()
            .await
            .map_err(|e| AppError::InternalServerError(format!("Failed to read chunk: {}", e)))?
        {
            if body_bytes.len() + chunk.len() > MAX_CAPABILITY_IMPORT_BYTES {
                return Err(AppError::BadRequest(
                    "Uploaded file exceeds maximum allowed size of 25MB".to_string(),
                ));
            }
            body_bytes.extend_from_slice(&chunk);
        }

        let content = String::from_utf8(body_bytes)
            .map_err(|_| AppError::BadRequest("Uploaded content is not valid UTF-8".to_string()))?;

        if name == "file" {
            let path = std::path::Path::new(&file_name);
            let cap_type = CapabilityType::from(path);
            let lower_name = file_name.to_ascii_lowercase();
            let stem = path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("imported_capability");

            if lower_name.ends_with(".md") {
                // Parse as Skill or Workflow
                if let Some(skill) = crate::agent::script_skills::parse_skill_md(&content) {
                    return Ok(Json(json!({
                        "type": "skill",
                        "data": skill,
                        "preview": content
                    })));
                } else {
                    let workflow = WorkflowDefinition {
                        id: None,
                        name: stem.to_string(),
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
            } else if cap_type == CapabilityType::Manifest {
                let val: serde_json::Value = serde_json::from_str(&content).map_err(|e| {
                    AppError::BadRequest(format!("Invalid JSON capability blueprint: {}", e))
                })?;
                if val.get("hook_type").is_some() {
                    let hook: HookDefinition = serde_json::from_value(val).map_err(|e| {
                        AppError::BadRequest(format!("Invalid hook blueprint: {}", e))
                    })?;
                    return Ok(Json(json!({
                        "type": "hook",
                        "data": hook,
                        "preview": content
                    })));
                } else if val.get("execution_command").is_some() {
                    let skill: SkillDefinition = serde_json::from_value(val).map_err(|e| {
                        AppError::BadRequest(format!("Invalid skill blueprint: {}", e))
                    })?;
                    return Ok(Json(json!({
                        "type": "skill",
                        "data": skill,
                        "preview": content
                    })));
                } else if val.get("content").is_some() {
                    let workflow: WorkflowDefinition =
                        serde_json::from_value(val).map_err(|e| {
                            AppError::BadRequest(format!("Invalid workflow blueprint: {}", e))
                        })?;
                    return Ok(Json(json!({
                        "type": "workflow",
                        "data": workflow,
                        "preview": content
                    })));
                } else {
                    return Err(AppError::BadRequest(
                        "JSON blueprint must define skill ('execution_command'), workflow ('content'), or hook ('hook_type') fields".to_string(),
                    ));
                }
            } else if cap_type == CapabilityType::Workflow {
                let val: serde_json::Value = serde_norway::from_str(&content).map_err(|e| {
                    AppError::BadRequest(format!("Invalid YAML capability blueprint: {}", e))
                })?;
                if val.get("hook_type").is_some() {
                    let hook: HookDefinition = serde_json::from_value(val).map_err(|e| {
                        AppError::BadRequest(format!("Invalid YAML hook blueprint: {}", e))
                    })?;
                    return Ok(Json(json!({
                        "type": "hook",
                        "data": hook,
                        "preview": content
                    })));
                } else if val.get("execution_command").is_some() {
                    let skill: SkillDefinition = serde_json::from_value(val).map_err(|e| {
                        AppError::BadRequest(format!("Invalid YAML skill blueprint: {}", e))
                    })?;
                    return Ok(Json(json!({
                        "type": "skill",
                        "data": skill,
                        "preview": content
                    })));
                } else if let Ok(workflow) =
                    serde_json::from_value::<WorkflowDefinition>(val.clone())
                {
                    return Ok(Json(json!({
                        "type": "workflow",
                        "data": workflow,
                        "preview": content
                    })));
                } else {
                    let wf_name = val
                        .get("name")
                        .and_then(|v| v.as_str())
                        .unwrap_or(stem)
                        .to_string();
                    let workflow = WorkflowDefinition {
                        id: None,
                        name: wf_name,
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
            } else if cap_type == CapabilityType::Script {
                let hook = HookDefinition {
                    name: file_name.clone(),
                    description: format!("Imported script from {}", file_name),
                    hook_type: "post-tool".to_string(),
                    content: content.clone(),
                    active: true,
                    category: "user".to_string(),
                };
                return Ok(Json(json!({
                    "type": "hook",
                    "data": hook,
                    "preview": content
                })));
            } else {
                return Err(AppError::BadRequest(format!(
                    "Unsupported capability file extension for '{}'",
                    file_name
                )));
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
#[tracing::instrument(skip(state, _admin, payload), fields(capability_type = %payload.r#type), name = "capability_registry::register")]
pub async fn register_capability(
    State(state): State<Arc<AppState>>,
    _admin: crate::middleware::auth::RequireAdmin,
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
        Err(e) => Err(e),
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
        "SELECT id, mission_id, agent_id, cap_type, name, description, content, CAST(created_at AS TEXT) AS created_at, status
         FROM capability_proposals
         WHERE status = 'pending'
         ORDER BY created_at DESC",
    )
    .fetch_all(&state.resources.pool)
    .await
    .map_err(AppError::Sqlx)?;

    // Schema (20260412000100_autonomous_governance.sql): id TEXT, description/created_at nullable.
    let proposals: Vec<serde_json::Value> = rows
        .into_iter()
        .map(|row| {
            use sqlx::Row;
            let content: String = row.try_get("content").unwrap_or_default();
            serde_json::json!({
                "id": row.try_get::<String, _>("id").unwrap_or_default(),
                "mission_id": row.try_get::<Option<String>, _>("mission_id").ok().flatten(),
                "agent_id": row.try_get::<String, _>("agent_id").unwrap_or_default(),
                "type": row.try_get::<String, _>("cap_type").unwrap_or_default(),
                "name": row.try_get::<String, _>("name").unwrap_or_default(),
                "description": row.try_get::<Option<String>, _>("description").ok().flatten().unwrap_or_default(),
                "payload": serde_json::from_str::<serde_json::Value>(&content).unwrap_or_default(),
                "created_at": row.try_get::<Option<String>, _>("created_at").ok().flatten(),
                "status": row.try_get::<String, _>("status").unwrap_or_default(),
            })
        })
        .collect();

    Ok(Json(proposals))
}

#[derive(serde::Deserialize, Debug)]
pub struct ResolveProposalPayload {
    pub decision: String, // "approved" or "rejected"
}

/// POST /v1/skills/proposals/:id/resolve
///
/// Finalizes a capability proposal. If approved, it is physically written to the agent_generated directory.
#[tracing::instrument(skip(state, _admin, payload), fields(proposal_id = %proposal_id, decision = %payload.decision), name = "capability_registry::resolve_proposal")]
pub async fn resolve_capability_proposal(
    Path(proposal_id): Path<String>,
    State(state): State<Arc<AppState>>,
    _admin: crate::middleware::auth::RequireAdmin,
    Json(payload): Json<ResolveProposalPayload>,
) -> Result<impl IntoResponse, AppError> {
    let norm_decision = payload.decision.trim().to_lowercase();
    if norm_decision != "approved" && norm_decision != "rejected" {
        return Err(AppError::BadRequest(
            "Decision must be either 'approved' or 'rejected'".to_string(),
        ));
    }
    let approved = norm_decision == "approved";

    // 1. Fetch proposal
    let row: (String, String, String, String, String) = sqlx::query_as(
        "SELECT cap_type, name, content, agent_id, status FROM capability_proposals WHERE id = ?",
    )
    .bind(&proposal_id)
    .fetch_one(&state.resources.pool)
    .await
    .map_err(|e| match e {
        sqlx::Error::RowNotFound => {
            AppError::NotFound(format!("Proposal '{}' not found", proposal_id))
        }
        other => AppError::Sqlx(other),
    })?;

    let (cap_type, name, payload_str, agent_id, status) = row;
    if status != "pending" {
        return Err(AppError::BadRequest(format!(
            "Proposal '{}' is already {}",
            proposal_id, status
        )));
    }
    let data: serde_json::Value = serde_json::from_str(&payload_str)
        .map_err(|e| AppError::InternalServerError(format!("Invalid proposal payload: {}", e)))?;

    // 2a. weight_edit (SEC-09): approval is a recorded HITL decision only. It never registers a
    //     skill; the operator applies it via execution/weight_space/operators.py to a derived model.
    if cap_type == crate::agent::runner::WEIGHT_EDIT_CAP_TYPE {
        if approved {
            reject_plane2_weight_edit(&state, &agent_id)?;
            crate::agent::runner::validate_weight_edit_proposal(&data)
                .map_err(AppError::BadRequest)?;
            tracing::info!(
                "🛡️ [WEIGHTS] weight_edit proposal {} ({}) approved by admin for agent {}",
                proposal_id,
                name,
                agent_id
            );
            state.broadcast_sys(
                &format!(
                    "✅ Weight edit approved: {} — apply with execution/weight_space/operators.py to a derived model tag (SEC-09)",
                    name
                ),
                "warning",
                None,
            );
        }
    } else if approved {
        // 2b. Skills/workflows/hooks: physically register on approval
        state
            .registry
            .skills
            .register_capability(&cap_type, data, "agent_generated")
            .await
            .map_err(|e| {
                AppError::InternalServerError(format!("Failed to register capability: {}", e))
            })?;

        state.broadcast_sys(
            &format!("✅ Autonomous Capability Approved: {} ({})", name, cap_type),
            "success",
            None,
        );
    }

    // 3. Update status in DB
    sqlx::query("UPDATE capability_proposals SET status = ? WHERE id = ?")
        .bind(if approved { "approved" } else { "rejected" })
        .bind(&proposal_id)
        .execute(&state.resources.pool)
        .await
        .map_err(AppError::Sqlx)?;

    Ok(Json(json!({ "status": "ok", "decision": norm_decision })))
}

/// FR-11: a weight_edit proposal authored by a Plane 2 agent can never be approved.
fn reject_plane2_weight_edit(state: &AppState, agent_id: &str) -> Result<(), AppError> {
    let is_plane2 = state.registry.agents.get(agent_id).is_some_and(|agent| {
        crate::agent::runner::AgentIdentity::is_plane2_department(&agent.identity.department)
    });
    if is_plane2 {
        return Err(AppError::Forbidden(
            "weight_edit proposals are restricted to Plane 1 agents (FR-11)".to_string(),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn insert_proposal(state: &AppState, id: &str, cap_type: &str, content: &str) {
        sqlx::query(
            "INSERT INTO capability_proposals (id, name, description, cap_type, content, agent_id, status) VALUES (?, 'p', NULL, ?, ?, 'agent-99', 'pending')",
        )
        .bind(id)
        .bind(cap_type)
        .bind(content)
        .execute(&state.resources.pool)
        .await
        .unwrap();
    }

    async fn status_of(state: &AppState, id: &str) -> String {
        sqlx::query_scalar("SELECT status FROM capability_proposals WHERE id = ?")
            .bind(id)
            .fetch_one(&state.resources.pool)
            .await
            .unwrap()
    }

    #[tokio::test]
    async fn test_weight_edit_resolution_records_decision_without_registering() {
        let state = Arc::new(AppState::new_minimal_mock().await);
        let payload = serde_json::json!({
            "model_id": "qwen2.5:0.5b", "layers": [3], "columns": [0, 1],
            "direction_ref": "dim:v1", "witness_metric": "loop_rate"
        })
        .to_string();
        insert_proposal(&state, "we-1", "weight_edit", &payload).await;

        // Listing reads the real schema (TEXT id, nullable description).
        let listed = list_capability_proposals(State(state.clone())).await;
        assert!(listed.is_ok());

        let res = resolve_capability_proposal(
            Path("we-1".to_string()),
            State(state.clone()),
            crate::middleware::auth::RequireAdmin,
            Json(ResolveProposalPayload {
                decision: "approved".into(),
            }),
        )
        .await;
        assert!(res.is_ok());
        assert_eq!(status_of(&state, "we-1").await, "approved");

        // A resolved proposal cannot be re-decided.
        let again = resolve_capability_proposal(
            Path("we-1".to_string()),
            State(state.clone()),
            crate::middleware::auth::RequireAdmin,
            Json(ResolveProposalPayload {
                decision: "rejected".into(),
            }),
        )
        .await;
        assert!(again.is_err());
    }

    #[tokio::test]
    async fn test_weight_edit_resolution_rejects_out_of_bounds_payload() {
        let state = Arc::new(AppState::new_minimal_mock().await);
        let payload = serde_json::json!({ "model_id": "m", "layers": [], "columns": [0],
            "direction_ref": "d", "witness_metric": "w" })
        .to_string();
        insert_proposal(&state, "we-2", "weight_edit", &payload).await;
        let res = resolve_capability_proposal(
            Path("we-2".to_string()),
            State(state.clone()),
            crate::middleware::auth::RequireAdmin,
            Json(ResolveProposalPayload {
                decision: "approved".into(),
            }),
        )
        .await;
        assert!(res.is_err());
        assert_eq!(status_of(&state, "we-2").await, "pending");
    }

    #[test]
    fn test_capability_type_from_extension() {
        assert_eq!(
            CapabilityType::from(std::path::Path::new("test.py")),
            CapabilityType::Script
        );
        assert_eq!(
            CapabilityType::from(std::path::Path::new("test.yaml")),
            CapabilityType::Workflow
        );
        assert_eq!(
            CapabilityType::from(std::path::Path::new("test.txt")),
            CapabilityType::Unknown
        );
    }

    #[tokio::test]
    async fn test_put_script_enforces_url_path_name_and_json_yaml_import_parses() {
        let state = Arc::new(AppState::new_minimal_mock().await);
        let mismatched_skill = SkillDefinition {
            id: None,
            name: "payload_name".to_string(),
            description: "desc".to_string(),
            execution_command: "python execution/example.py".to_string(),
            schema: serde_json::json!({}),
            oversight_required: true,
            doc_url: None,
            tags: None,
            full_instructions: None,
            negative_constraints: None,
            verification_script: None,
            category: "user".to_string(),
            security_score: None,
            security_severity: None,
            security_report: None,
        };

        // Calling PUT /v1/skills/scripts/url_name with payload.name == "payload_name" must reject mismatch
        let app = axum::Router::new()
            .route("/scripts/{name}", axum::routing::put(post_script))
            .route("/import", axum::routing::post(import_capability))
            .with_state(state.clone());

        use tower::ServiceExt;
        let put_req = axum::http::Request::builder()
            .method("PUT")
            .uri("/scripts/url_name")
            .header("content-type", "application/json")
            .extension(crate::middleware::auth::AuthenticatedRole::Admin)
            .body(axum::body::Body::from(
                serde_json::to_vec(&mismatched_skill).unwrap(),
            ))
            .unwrap();
        let put_resp = app.clone().oneshot(put_req).await.unwrap();
        assert_eq!(
            put_resp.status(),
            StatusCode::BAD_REQUEST,
            "PUT /scripts/{{name}} must reject mismatched URL path name vs payload.name"
        );

        // Multipart import of .json capability blueprint must succeed
        let boundary = "----TadpoleCapImportBoundary";
        let json_body = format!(
            "--{b}\r\nContent-Disposition: form-data; name=\"file\"; filename=\"audit_skill.json\"\r\nContent-Type: application/json\r\n\r\n{{\"name\":\"audit_skill\",\"description\":\"Audit\",\"execution_command\":\"python execution/audit.py\",\"schema\":{{}}}}\r\n--{b}--\r\n",
            b = boundary
        );
        let import_req = axum::http::Request::builder()
            .method("POST")
            .uri("/import")
            .header(
                "content-type",
                format!("multipart/form-data; boundary={}", boundary),
            )
            .extension(crate::middleware::auth::AuthenticatedRole::Admin)
            .body(axum::body::Body::from(json_body))
            .unwrap();
        let import_resp = app.oneshot(import_req).await.unwrap();
        assert_eq!(
            import_resp.status(),
            StatusCode::OK,
            "POST /v1/skills/import must accept .json capability blueprints"
        );
    }
}

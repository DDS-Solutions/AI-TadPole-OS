//! @docs ARCHITECTURE:Networking
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / HTTP Routes / templates
//! - **Primary Entrypoints**: `get_templates_catalog`, `install_template`, `import_template`, `get_installed_templates`, `uninstall_template`
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Type-safe state handling and bounded execution without unhandled panics.
//! - `[Structural]` Strict parity across install and import: workflow risk scanning, schema validation, atomic writes.
//! - `[Structural]` Global serialization across install, import, and uninstall operations via TEMPLATE_OPS_LOCK.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: `AppError::BadRequest`, `AppError::Forbidden`, `AppError::NotFound`, `AppError::Io`
//! - **Telemetry Targets**: none declared
//! - **Witness Tests**: `routes::templates::tests::*`

pub mod catalog;
pub mod dto;
pub mod installed;
pub mod mcp_store;
pub mod naming;
pub mod source;
pub mod validate;

pub use catalog::{fetch_catalog, get_offline_catalog, CATALOG_FETCH_TIMEOUT, REMOTE_CATALOG_URL};
pub use dto::{
    ImportTemplateRequest, InstallAssetReceipt, InstallTemplateRequest, InstallTemplateResponse,
    InstallationReceipt, InstalledSwarmSummary, InstalledTemplatesResponse, ModelOverrideConfig,
    UninstallTemplateRequest, UninstallTemplateResponse,
};
pub use installed::{
    archive_and_remove_swarm, list_installed_swarms, read_installed_manifest,
    write_installed_manifest,
};
pub use mcp_store::{
    merge_incoming_mcp_config, merge_mcp_config_data, prepare_incoming_mcp_config,
    prune_mcp_servers,
};
pub use naming::{
    apply_swarm_namespace, apply_workflow_namespace, get_installed_swarm_id,
    sanitize_agent_filename, sanitize_workflow_filename,
};
pub use validate::{
    is_allowed_skill_extension, validate_agent_payload, validate_template_assets,
    validate_workflow_content, ValidatedAgent, ValidatedTemplateAssets, ALLOWED_SKILL_EXTENSIONS,
};

use crate::error::AppError;
use crate::state::AppState;
use crate::utils::fs_transaction::{InstallTransaction, TemporaryClone};
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use std::path::PathBuf;
use std::sync::Arc;

/// Global process-wide serialization lock for template installations, imports, and uninstalls.
static TEMPLATE_OPS_LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

/// Helper to register validated agents in the runtime state and database with non-fatal logging.
async fn register_validated_agents(
    state: &Arc<AppState>,
    agents: &[validate::ValidatedAgent],
) -> Vec<String> {
    let mut warnings = Vec::new();
    for agent in agents {
        let mut ea_clone = agent.agent.clone();
        if let Err(err) =
            crate::agent::persistence::save_agent_db(&state.resources.pool, &mut ea_clone).await
        {
            tracing::warn!(
                "Agent '{}' persisted to filesystem but database indexing deferred: {}",
                agent.agent.identity.id,
                err
            );
            warnings.push(format!(
                "Database indexing deferred for agent '{}'",
                agent.agent.identity.id
            ));
        }
        state
            .registry
            .agents
            .insert(agent.agent.identity.id.clone(), ea_clone);
    }
    warnings
}

/// GET /v1/engine/templates/catalog
/// Fetches available templates from the remote repository index or falls back to offline catalog.
/// @docs API_REFERENCE:GetTemplatesCatalog
#[tracing::instrument(skip(state), name = "templates::catalog")]
pub async fn get_templates_catalog(
    State(state): State<Arc<AppState>>,
) -> Result<axum::response::Response, AppError> {
    let catalog = fetch_catalog(&state.resources.http_client).await;
    Ok((StatusCode::OK, Json(catalog)).into_response())
}

/// POST /v1/engine/templates/install
/// Clones a remote template repository and installs member agents, workflows, skills, and MCP configuration.
/// @docs API_REFERENCE:InstallTemplate
#[tracing::instrument(skip(state, payload), name = "templates::install")]
pub async fn install_template(
    State(state): State<Arc<AppState>>,
    _admin: crate::middleware::auth::RequireAdmin,
    Json(payload): Json<InstallTemplateRequest>,
) -> Result<axum::response::Response, AppError> {
    let _ops_guard = TEMPLATE_OPS_LOCK.lock().await;

    let dl_id = uuid::Uuid::new_v4().to_string();
    let temp_dir = std::env::current_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join("data")
        .join(".bunker_cache")
        .join(&dl_id);

    tokio::fs::create_dir_all(&temp_dir)
        .await
        .map_err(AppError::Io)?;
    let _temporary_clone = TemporaryClone(temp_dir.clone());

    // 1. Clone repository
    source::clone_template_repository(
        &payload.repository_url,
        payload.git_ref.as_deref(),
        &temp_dir,
    )
    .await?;

    // 2. Validate template path
    let source_path = match crate::utils::security::validate_path(&temp_dir, &payload.path) {
        Ok(p) => p,
        Err(e) => {
            return Err(AppError::BadRequest(format!(
                "Invalid template path: {}",
                e
            )))
        }
    };
    if !source_path.exists() {
        return Err(AppError::NotFound(format!(
            "Template path '{}' not found in repo",
            payload.path
        )));
    }

    let workspace_root = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let source_revision = source::cloned_revision(&temp_dir).await;
    let validated = validate_template_assets(
        source_path.as_path(),
        &workspace_root,
        &payload.path,
        payload.model_override.as_ref(),
        payload.namespace.as_deref(),
        payload.overwrite,
    )
    .await?;

    let installed_id = get_installed_swarm_id(&validated.safe_name, payload.namespace.as_deref());
    let mut transaction = InstallTransaction::default();

    // 3. Stage agent file writes
    let mut replaced_agents = 0;
    let mut installed_agent_ids = Vec::new();
    for agent in &validated.agents {
        let destination = workspace_root
            .join("data/swarm_config/agents")
            .join(&agent.safe_filename);
        let replaced = transaction
            .write_new(&destination, &agent.serialized, payload.overwrite)
            .await?;
        if replaced {
            replaced_agents += 1;
        }
        installed_agent_ids.push(agent.agent.identity.id.clone());
    }

    if let Some(ref swarm_manifest) = validated.swarm_manifest {
        let destination = workspace_root
            .join("data/swarm_config/installed")
            .join(&installed_id)
            .join("swarm.json");
        transaction
            .copy_new(swarm_manifest, &destination, payload.overwrite)
            .await?;
    }

    // 4. Stage workflow file writes
    let mut replaced_workflows = 0;
    let mut installed_workflow_names = Vec::new();
    for (filename, workflow_path) in &validated.workflows {
        let destination = workspace_root.join("directives").join(filename);
        let replaced = transaction
            .copy_new(workflow_path, &destination, payload.overwrite)
            .await?;
        if replaced {
            replaced_workflows += 1;
        }
        installed_workflow_names.push(filename.clone());
    }

    // 5. Stage skill file writes
    let mut replaced_skills = 0;
    let mut installed_skill_names = Vec::new();
    for skill in &validated.skills {
        let skill_name = validate::validated_file_name(skill, "skill")?;
        let destination = workspace_root.join("execution").join(skill_name);
        let replaced = transaction
            .copy_new(skill, &destination, payload.overwrite)
            .await?;
        if replaced {
            replaced_skills += 1;
        }
        installed_skill_names.push(skill_name.to_string_lossy().to_string());
    }

    // 6. Stage MCP configuration write
    if let Some(ref merged_mcp_config) = validated.merged_mcp_config {
        let mcp_path = workspace_root.join(".agent/mcp_config.json");
        transaction
            .replace_atomically(&mcp_path, merged_mcp_config)
            .await?;
    }

    // 7. Save comprehensive installed manifest (Engine authoritative)
    let installed_summary = InstalledSwarmSummary {
        id: installed_id.clone(),
        name: payload.path.replace(['/', '\\'], " "),
        description: format!("Installed from {}", payload.path),
        industry: None,
        installed_at: Some(chrono::Utc::now().to_rfc3339()),
        template_path: payload.path.clone(),
        agents: installed_agent_ids.clone(),
        workflows: installed_workflow_names,
        skills: installed_skill_names,
        mcp_servers: validated.mcp_server_names.clone(),
    };
    let manifest_bytes = serde_json::to_vec_pretty(&installed_summary).map_err(|e| {
        AppError::InternalServerError(format!("Failed to serialize installed manifest: {}", e))
    })?;
    let manifest_dest = workspace_root
        .join("data/swarm_config/installed")
        .join(&installed_id)
        .join("installed_manifest.json");
    // Manifest is engine-owned metadata and always authoritative
    transaction
        .write_new(&manifest_dest, &manifest_bytes, true)
        .await?;

    // 8. Commit filesystem transaction (files now permanent)
    transaction.commit().await;

    // 9. Register agents in database & in-memory registry after successful file commit
    register_validated_agents(&state, &validated.agents).await;

    // 10. Construct receipt with accurate replacement metrics
    let receipt = InstallationReceipt {
        template_path: payload.path.clone(),
        source_revision,
        agents: InstallAssetReceipt::new(
            validated.agents.len(),
            validated.agents.len(),
            replaced_agents,
        ),
        workflows: InstallAssetReceipt::new(
            validated.workflows.len(),
            validated.workflows.len(),
            replaced_workflows,
        ),
        skills: InstallAssetReceipt::new(
            validated.skills.len(),
            validated.skills.len(),
            replaced_skills,
        ),
        swarm_manifest: InstallAssetReceipt::complete(usize::from(
            validated.swarm_manifest.is_some(),
        )),
        mcp_servers: InstallAssetReceipt::new(
            validated.mcp_server_count,
            validated.mcp_server_count,
            validated.mcp_replaced_count,
        ),
    };

    Ok((
        StatusCode::OK,
        Json(InstallTemplateResponse {
            status: "success".to_string(),
            message: format!(
                "Successfully installed swarm template from {}",
                payload.path
            ),
            receipt,
        }),
    )
        .into_response())
}

/// POST /v1/engine/templates/import
/// Imports a locally staged swarm bundle with validation, namespacing, and atomic configuration merging.
/// @docs API_REFERENCE:ImportTemplate
#[tracing::instrument(skip(state, payload), name = "templates::import")]
pub async fn import_template(
    State(state): State<Arc<AppState>>,
    _admin: crate::middleware::auth::RequireAdmin,
    Json(payload): Json<ImportTemplateRequest>,
) -> Result<axum::response::Response, AppError> {
    let _ops_guard = TEMPLATE_OPS_LOCK.lock().await;
    let workspace_root = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));

    // Early validation: swarm manifest must be a valid JSON object
    let swarm_obj = payload.swarm.as_object().ok_or_else(|| {
        AppError::BadRequest("Field 'swarm' must be a valid JSON object".to_string())
    })?;

    let swarm_id = swarm_obj
        .get("id")
        .and_then(|v| v.as_str())
        .or_else(|| swarm_obj.get("name").and_then(|v| v.as_str()))
        .unwrap_or("imported-swarm");
    let safe_swarm_name = crate::utils::security::sanitize_id(&swarm_id.replace(['/', '\\'], "_"));
    let installed_id = get_installed_swarm_id(&safe_swarm_name, payload.namespace.as_deref());

    // 1. Validate agents
    let mut validated_agents = Vec::new();
    let mut installed_agent_ids = Vec::new();
    for raw_agent in &payload.agents {
        let validated_agent = validate_agent_payload(
            &raw_agent.filename,
            &raw_agent.content,
            payload.model_override.as_ref(),
            payload.namespace.as_deref(),
        )?;
        installed_agent_ids.push(validated_agent.agent.identity.id.clone());
        validated_agents.push(validated_agent);
    }

    // 2. Validate workflows
    let mut validated_workflows = Vec::new();
    let mut installed_workflow_names = Vec::new();
    for raw_wf in &payload.workflows {
        let safe_base = validate_workflow_content(&raw_wf.filename, &raw_wf.content).await?;
        let namespaced_name = apply_workflow_namespace(&safe_base, payload.namespace.as_deref());
        installed_workflow_names.push(namespaced_name.clone());
        validated_workflows.push((namespaced_name, raw_wf.content.clone()));
    }

    // 3. Validate and merge MCP config using authoritative mcp_store helper
    let (merged_mcp_config, mcp_count, mcp_server_names, replaced_mcp) =
        if let Some(ref mcp_val) = payload.mcps {
            let incoming: crate::agent::mcp::McpConfig = serde_json::from_value(mcp_val.clone())
                .map_err(|error| {
                    AppError::BadRequest(format!("Invalid incoming MCP configuration: {}", error))
                })?;
            let (bytes, count, names, replaced) =
                mcp_store::merge_incoming_mcp_config(&incoming, &workspace_root, payload.overwrite)
                    .await?;
            (Some(bytes), count, names, replaced)
        } else {
            (None, 0, Vec::new(), 0)
        };

    let mut transaction = InstallTransaction::default();

    // 4. Stage agent writes
    let mut replaced_agents = 0;
    let agents_dest = workspace_root.join("data/swarm_config/agents");
    for agent in &validated_agents {
        let agent_path = agents_dest.join(&agent.safe_filename);
        let replaced = transaction
            .write_new(&agent_path, &agent.serialized, payload.overwrite)
            .await?;
        if replaced {
            replaced_agents += 1;
        }
    }

    // 5. Stage workflow writes
    let mut replaced_workflows = 0;
    let workflows_dest = workspace_root.join("directives");
    for (filename, content) in &validated_workflows {
        let wf_path = workflows_dest.join(filename);
        let replaced = transaction
            .write_new(&wf_path, content.as_bytes(), payload.overwrite)
            .await?;
        if replaced {
            replaced_workflows += 1;
        }
    }

    // 6. Stage MCP write
    if let Some(ref mcp_bytes) = merged_mcp_config {
        let mcp_config_path = workspace_root.join(".agent/mcp_config.json");
        transaction
            .replace_atomically(&mcp_config_path, mcp_bytes)
            .await?;
    }

    // 7. Stage swarm manifest write
    let swarm_bytes = serde_json::to_vec_pretty(swarm_obj).map_err(|e| {
        AppError::InternalServerError(format!("Failed to serialize swarm manifest: {}", e))
    })?;
    let swarm_dest = workspace_root
        .join("data/swarm_config/installed")
        .join(&installed_id)
        .join("swarm.json");
    transaction
        .write_new(&swarm_dest, &swarm_bytes, true)
        .await?;

    // 8. Save installed summary manifest (Engine authoritative)
    let installed_summary = InstalledSwarmSummary {
        id: installed_id.clone(),
        name: swarm_obj
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or(&safe_swarm_name)
            .to_string(),
        description: swarm_obj
            .get("description")
            .and_then(|v| v.as_str())
            .unwrap_or("Imported sovereign swarm cluster")
            .to_string(),
        industry: swarm_obj
            .get("industry")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()),
        installed_at: Some(chrono::Utc::now().to_rfc3339()),
        template_path: format!("imported/{}", safe_swarm_name),
        agents: installed_agent_ids.clone(),
        workflows: installed_workflow_names,
        skills: Vec::new(),
        mcp_servers: mcp_server_names.clone(),
    };
    let manifest_bytes = serde_json::to_vec_pretty(&installed_summary).map_err(|e| {
        AppError::InternalServerError(format!("Failed to serialize installed manifest: {}", e))
    })?;
    let manifest_dest = workspace_root
        .join("data/swarm_config/installed")
        .join(&installed_id)
        .join("installed_manifest.json");
    transaction
        .write_new(&manifest_dest, &manifest_bytes, true)
        .await?;

    // 9. Commit filesystem operations atomically
    transaction.commit().await;

    // 10. Register imported agents in database and runtime registry
    register_validated_agents(&state, &validated_agents).await;

    // 11. Produce receipt
    let receipt = InstallationReceipt {
        template_path: format!("local/{}", installed_id),
        source_revision: None,
        agents: InstallAssetReceipt::new(
            validated_agents.len(),
            validated_agents.len(),
            replaced_agents,
        ),
        workflows: InstallAssetReceipt::new(
            validated_workflows.len(),
            validated_workflows.len(),
            replaced_workflows,
        ),
        skills: InstallAssetReceipt::complete(0),
        swarm_manifest: InstallAssetReceipt::complete(1),
        mcp_servers: InstallAssetReceipt::new(mcp_count, mcp_count, replaced_mcp),
    };

    Ok((
        StatusCode::OK,
        Json(InstallTemplateResponse {
            status: "success".to_string(),
            message: format!("Successfully imported local swarm '{}'", installed_id),
            receipt,
        }),
    )
        .into_response())
}

/// GET /v1/engine/templates/installed
/// Lists all currently installed swarms along with their member agents, workflows, and skills.
/// @docs API_REFERENCE:GetInstalledTemplates
#[tracing::instrument(skip(_state), name = "templates::list_installed")]
pub async fn get_installed_templates(
    State(_state): State<Arc<AppState>>,
) -> Result<axum::response::Response, AppError> {
    let workspace_root = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let swarms = list_installed_swarms(&workspace_root).await?;
    Ok((StatusCode::OK, Json(InstalledTemplatesResponse { swarms })).into_response())
}

/// POST /v1/engine/templates/uninstall
/// Safely deactivates agents, unregisters them from DB & state, archives or deletes files, and prunes MCP config.
/// @docs API_REFERENCE:UninstallTemplate
#[tracing::instrument(skip(state, payload), name = "templates::uninstall")]
pub async fn uninstall_template(
    State(state): State<Arc<AppState>>,
    _admin: crate::middleware::auth::RequireAdmin,
    Json(payload): Json<UninstallTemplateRequest>,
) -> Result<axum::response::Response, AppError> {
    let _ops_guard = TEMPLATE_OPS_LOCK.lock().await;

    let clean_id = crate::utils::security::sanitize_id(&payload.swarm_id);
    if clean_id.is_empty() {
        return Err(AppError::BadRequest("Invalid swarm_id".to_string()));
    }

    let workspace_root = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let installed_dir = workspace_root
        .join("data/swarm_config/installed")
        .join(&clean_id);

    if !tokio::fs::try_exists(&installed_dir)
        .await
        .map_err(AppError::Io)?
    {
        return Err(AppError::NotFound(format!(
            "Installed swarm '{}' not found",
            clean_id
        )));
    }

    let summary = read_installed_manifest(&installed_dir, &clean_id).await?;

    // 1. Prune MCP servers first while swarm files are still intact
    let uninstalled_mcp_servers = prune_mcp_servers(&workspace_root, &summary.mcp_servers).await?;

    // 2. Archive and remove swarm directories, agents, workflows, and database entries
    let (uninstalled_agents, uninstalled_workflows, uninstalled_skills, archived_path) =
        archive_and_remove_swarm(&workspace_root, &summary, &state, payload.archive).await?;

    Ok((
        StatusCode::OK,
        Json(UninstallTemplateResponse {
            status: "success".to_string(),
            message: format!("Successfully uninstalled swarm '{}'", clean_id),
            uninstalled_agents,
            uninstalled_workflows,
            uninstalled_skills,
            uninstalled_mcp_servers,
            archived_path,
        }),
    )
        .into_response())
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    #[test]
    fn test_import_request_rejects_non_object_swarm() {
        let bad_json = serde_json::json!({
            "swarm": "invalid string instead of object",
            "agents": [],
            "workflows": []
        });
        let req: Result<ImportTemplateRequest, _> = serde_json::from_value(bad_json);
        assert!(req.is_ok()); // JSON parses
        let payload = req.unwrap();
        assert!(payload.swarm.as_object().is_none());
    }
}

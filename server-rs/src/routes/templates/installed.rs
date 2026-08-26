//! @docs ARCHITECTURE:Networking
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / HTTP Routes / templates / installed
//! - **Primary Entrypoints**: `list_installed_swarms`, `read_installed_manifest`, `write_installed_manifest`, `archive_and_remove_swarm`
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Single source of truth for installed_manifest.json format and lifecycle.
//! - `[Structural]` Missing or corrupt manifests fail loudly with AppError::BadRequest rather than silent no-op.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: `AppError::NotFound`, `AppError::BadRequest`, `AppError::InternalServerError`, `AppError::Io`
//! - **Telemetry Targets**: none declared
//! - **Witness Tests**: `routes::templates::installed::tests::*`

use super::dto::InstalledSwarmSummary;
use crate::error::AppError;
use std::path::Path;

pub async fn list_installed_swarms(
    workspace_root: &Path,
) -> Result<Vec<InstalledSwarmSummary>, AppError> {
    let installed_root = workspace_root.join("data/swarm_config/installed");

    if !tokio::fs::try_exists(&installed_root).await.map_err(AppError::Io)? {
        return Ok(Vec::new());
    }

    let mut swarms = Vec::new();
    let mut entries = tokio::fs::read_dir(&installed_root).await.map_err(AppError::Io)?;
    while let Some(entry) = entries.next_entry().await.map_err(AppError::Io)? {
        if !entry.file_type().await.map_err(AppError::Io)?.is_dir() {
            continue;
        }

        let dir_path = entry.path();
        let manifest_path = dir_path.join("installed_manifest.json");
        let swarm_json_path = dir_path.join("swarm.json");

        if tokio::fs::try_exists(&manifest_path).await.map_err(AppError::Io)? {
            match tokio::fs::read_to_string(&manifest_path).await {
                Ok(content) => match serde_json::from_str::<InstalledSwarmSummary>(&content) {
                    Ok(summary) => {
                        swarms.push(summary);
                        continue;
                    }
                    Err(e) => {
                        tracing::warn!(target: "Templates:Installed", "Unreadable installed_manifest.json in {:?}: {}", dir_path, e);
                    }
                },
                Err(e) => {
                    tracing::warn!(target: "Templates:Installed", "Failed to read installed_manifest.json in {:?}: {}", dir_path, e);
                }
            }
        }

        if tokio::fs::try_exists(&swarm_json_path).await.map_err(AppError::Io)? {
            if let Ok(content) = tokio::fs::read_to_string(&swarm_json_path).await {
                if let Ok(val) = serde_json::from_str::<serde_json::Value>(&content) {
                    let id = dir_path.file_name().and_then(|n| n.to_str()).unwrap_or("unknown").to_string();
                    let name = val.get("name").and_then(|v| v.as_str()).unwrap_or(&id).to_string();
                    let description = val.get("description").and_then(|v| v.as_str()).unwrap_or("").to_string();
                    let industry = val.get("industry").and_then(|v| v.as_str()).map(|s| s.to_string());
                    swarms.push(InstalledSwarmSummary {
                        id,
                        name,
                        description,
                        industry,
                        installed_at: None,
                        template_path: "unknown".to_string(),
                        agents: Vec::new(),
                        workflows: Vec::new(),
                        skills: Vec::new(),
                        mcp_servers: Vec::new(),
                    });
                }
            }
        }
    }

    swarms.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(swarms)
}

pub async fn read_installed_manifest(
    installed_dir: &Path,
    swarm_id: &str,
) -> Result<InstalledSwarmSummary, AppError> {
    let manifest_path = installed_dir.join("installed_manifest.json");

    if !tokio::fs::try_exists(&manifest_path).await.map_err(AppError::Io)? {
        return Err(AppError::BadRequest(format!(
            "Installed swarm manifest missing for '{}'; manual cleanup required",
            swarm_id
        )));
    }

    let manifest_content = tokio::fs::read_to_string(&manifest_path)
        .await
        .map_err(AppError::Io)?;

    serde_json::from_str(&manifest_content).map_err(|e| {
        AppError::BadRequest(format!(
            "Installed swarm manifest is corrupt for '{}'; manual cleanup required: {}",
            swarm_id, e
        ))
    })
}

pub async fn write_installed_manifest(
    installed_dir: &Path,
    summary: &InstalledSwarmSummary,
) -> Result<(), AppError> {
    let manifest_bytes = serde_json::to_vec_pretty(summary).map_err(|e| {
        AppError::InternalServerError(format!("Failed to serialize installed manifest: {}", e))
    })?;
    let manifest_dest = installed_dir.join("installed_manifest.json");
    if let Some(parent) = manifest_dest.parent() {
        tokio::fs::create_dir_all(parent).await.map_err(AppError::Io)?;
    }
    tokio::fs::write(&manifest_dest, manifest_bytes).await.map_err(AppError::Io)?;
    Ok(())
}

pub async fn archive_and_remove_swarm(
    workspace_root: &Path,
    summary: &InstalledSwarmSummary,
    state: &crate::state::AppState,
    should_archive: bool,
) -> Result<(Vec<String>, Vec<String>, Vec<String>, Option<String>), AppError> {
    let clean_id = crate::utils::security::sanitize_id(&summary.id);
    let installed_dir = workspace_root.join("data/swarm_config/installed").join(&clean_id);

    let archive_dir = if should_archive {
        let dir = workspace_root
            .join("data/swarm_config/archive")
            .join(&clean_id)
            .join(chrono::Utc::now().format("%Y%m%d_%H%M%S").to_string());
        tokio::fs::create_dir_all(&dir).await.map_err(AppError::Io)?;
        Some(dir)
    } else {
        None
    };

    let mut uninstalled_agents = Vec::new();
    let mut uninstalled_workflows = Vec::new();
    let mut uninstalled_skills = Vec::new();

    // 1. Unregister and delete/archive agents
    for agent_id in &summary.agents {
        let clean_agent_id = crate::utils::security::sanitize_id(agent_id);
        if clean_agent_id.is_empty() {
            continue;
        }

        let mut touched = state.registry.agents.remove(&clean_agent_id).is_some();
        let delete_res = sqlx::query("DELETE FROM agents WHERE id = ?")
            .bind(&clean_agent_id)
            .execute(&state.resources.pool)
            .await;

        if let Err(err) = delete_res {
            tracing::error!(target: "Templates:Installed", "Failed to delete agent '{}' from database during uninstall: {}", clean_agent_id, err);
            return Err(AppError::InternalServerError(format!(
                "Failed to unregister agent '{}' from database: {}",
                clean_agent_id, err
            )));
        }

        let agent_file = workspace_root
            .join("data/swarm_config/agents")
            .join(format!("{}.json", clean_agent_id));
        if tokio::fs::try_exists(&agent_file).await.unwrap_or(false) {
            if let Some(ref arch) = archive_dir {
                let dest = arch.join("agents").join(format!("{}.json", clean_agent_id));
                if let Some(parent) = dest.parent() {
                    let _ = tokio::fs::create_dir_all(parent).await;
                }
                let _ = tokio::fs::rename(&agent_file, dest).await;
            } else {
                let _ = tokio::fs::remove_file(&agent_file).await;
            }
            touched = true;
        }
        if touched {
            uninstalled_agents.push(clean_agent_id);
        }
    }

    // 2. Remove/archive workflows using authoritative naming module
    for wf in &summary.workflows {
        let clean_wf_base = match super::naming::sanitize_workflow_filename(wf) {
            Ok(base) => base,
            Err(_) => {
                let stem = wf.strip_suffix(".md").or_else(|| wf.strip_suffix(".MD")).unwrap_or(wf);
                let path_stem = Path::new(stem).file_name().and_then(|n| n.to_str()).unwrap_or(stem);
                crate::utils::security::sanitize_id(path_stem)
            }
        };
        if clean_wf_base.is_empty() {
            continue;
        }
        let clean_wf_filename = format!("{}.md", clean_wf_base);

        let mut touched = false;
        let wf_file = workspace_root.join("directives").join(&clean_wf_filename);
        if tokio::fs::try_exists(&wf_file).await.unwrap_or(false) {
            if let Some(ref arch) = archive_dir {
                let dest = arch.join("workflows").join(&clean_wf_filename);
                if let Some(parent) = dest.parent() {
                    let _ = tokio::fs::create_dir_all(parent).await;
                }
                let _ = tokio::fs::rename(&wf_file, dest).await;
            } else {
                let _ = tokio::fs::remove_file(&wf_file).await;
            }
            touched = true;
        }
        if touched {
            uninstalled_workflows.push(clean_wf_filename);
        }
    }

    // 3. Remove/archive skills (Basenames from collect_regular_files are flat and traversal-safe by construction)
    for skill in &summary.skills {
        let path_stem = Path::new(skill).file_name().and_then(|n| n.to_str()).unwrap_or(skill);
        let clean_skill_name = path_stem.to_string();

        let mut touched = false;
        let skill_file = workspace_root.join("execution").join(&clean_skill_name);
        if tokio::fs::try_exists(&skill_file).await.unwrap_or(false) {
            if let Some(ref arch) = archive_dir {
                let dest = arch.join("skills").join(&clean_skill_name);
                if let Some(parent) = dest.parent() {
                    let _ = tokio::fs::create_dir_all(parent).await;
                }
                let _ = tokio::fs::rename(&skill_file, dest).await;
            } else {
                let _ = tokio::fs::remove_file(&skill_file).await;
            }
            touched = true;
        }
        if touched {
            uninstalled_skills.push(clean_skill_name);
        }
    }

    // 4. Archive or remove the installed directory itself
    let manifest_path = installed_dir.join("installed_manifest.json");
    let swarm_json_path = installed_dir.join("swarm.json");
    if let Some(ref arch) = archive_dir {
        if tokio::fs::try_exists(&manifest_path).await.unwrap_or(false) {
            let _ = tokio::fs::copy(&manifest_path, arch.join("installed_manifest.json")).await;
        }
        if tokio::fs::try_exists(&swarm_json_path).await.unwrap_or(false) {
            let _ = tokio::fs::copy(&swarm_json_path, arch.join("swarm.json")).await;
        }
    }
    tokio::fs::remove_dir_all(&installed_dir).await.map_err(AppError::Io)?;

    tracing::info!(
        target: "Templates:Installed",
        "Uninstalled swarm '{}' (agents: {}, workflows: {}, skills: {})",
        clean_id,
        uninstalled_agents.len(),
        uninstalled_workflows.len(),
        uninstalled_skills.len()
    );

    let archived_path = archive_dir.map(|p| p.to_string_lossy().to_string());
    Ok((uninstalled_agents, uninstalled_workflows, uninstalled_skills, archived_path))
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    #[tokio::test]
    async fn test_read_manifest_missing_fails_loudly() {
        let temp = tempfile::tempdir().unwrap();
        let res = read_installed_manifest(temp.path(), "missing_swarm").await;
        assert!(matches!(res, Err(AppError::BadRequest(_))));
    }

    #[tokio::test]
    async fn test_read_manifest_corrupt_fails_loudly() {
        let temp = tempfile::tempdir().unwrap();
        let manifest_path = temp.path().join("installed_manifest.json");
        tokio::fs::write(&manifest_path, b"{ invalid json").await.unwrap();

        let res = read_installed_manifest(temp.path(), "corrupt_swarm").await;
        assert!(matches!(res, Err(AppError::BadRequest(_))));
    }

    #[tokio::test]
    async fn test_write_and_read_manifest_roundtrip() {
        let temp = tempfile::tempdir().unwrap();
        let summary = InstalledSwarmSummary {
            id: "mkt_test".to_string(),
            name: "Marketing Swarm".to_string(),
            description: "Tests roundtrip".to_string(),
            industry: Some("Marketing".to_string()),
            installed_at: Some("2026-08-25T12:00:00Z".to_string()),
            template_path: "templates/mkt".to_string(),
            agents: vec!["lead_gen".to_string()],
            workflows: vec!["daily_sync.md".to_string()],
            skills: vec!["search.py".to_string()],
            mcp_servers: vec!["hubspot".to_string()],
        };

        let write_res = write_installed_manifest(temp.path(), &summary).await;
        assert!(write_res.is_ok());

        let read_res = read_installed_manifest(temp.path(), "mkt_test").await;
        assert!(read_res.is_ok());
        let read_summary = read_res.unwrap();
        assert_eq!(read_summary.id, "mkt_test");
        assert_eq!(read_summary.agents, vec!["lead_gen"]);
        assert_eq!(read_summary.skills, vec!["search.py"]);
    }
}

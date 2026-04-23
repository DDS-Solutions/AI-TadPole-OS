//! @docs ARCHITECTURE:Networking
//!
//! ### AI Assist Note
//! **Mission Templates (Starter Kits)**: Orchestrates the
//! installation and management of multi-agent swarm blueprints for
//! the Tadpole OS engine. Features **Git-Integrated Installation**:
//! allows users to pull complete swarm configurations (agents,
//! workflows, skills, MCP configs) from remote repositories.
//! Implements **Swarm Partitioning**: automatically categorizes and
//! persists individual components (directives, execution scripts,
//! registries) to ensure a seamless "one-click" deployment
//! experience. AI agents should use this endpoint to expand their
//! operational capabilities by installing specialized skill-sets or
//! mission-specific swarm architectures (TMP-01).
//!
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: Git clone failures due to network or
//!   authentication issues, duplicate agent ID collisions during
//!   template merge, or schema mismatches in the imported `swarm.json`
//!   config.
//! - **Telemetry Link**: Search for `📥 [ModelStore]` or `✅ Successfully
//!   initiated pull` in `tracing` logs (internal Note: templates use
//!   similar pull patterns).
//! - **Trace Scope**: `server-rs::routes::templates`

use crate::error::AppError;
use crate::state::AppState;
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Debug, Deserialize)]
pub struct InstallTemplateRequest {
    pub repository_url: String,
    pub path: String,
}

#[derive(Debug, Serialize)]
pub struct InstallTemplateResponse {
    pub status: String,
    pub message: String,
}

#[axum::debug_handler]
#[tracing::instrument(skip(state, payload), name = "templates::install")]
pub async fn install_template(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<InstallTemplateRequest>,
) -> Result<impl IntoResponse, AppError> {
    let dl_id = uuid::Uuid::new_v4().to_string();
    let temp_dir = std::env::current_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join("data")
        .join(".bunker_cache")
        .join(&dl_id);

    let _ = tokio::fs::create_dir_all(&temp_dir).await;

    // 1. Clone repository
    let status = match tokio::process::Command::new("git")
        .arg("clone")
        .arg("--depth")
        .arg("1")
        .arg(&payload.repository_url)
        .arg(&temp_dir)
        .status()
        .await
    {
        Ok(s) => s,
        Err(e) => {
            return Err(AppError::InternalServerError(format!(
                "Failed to execute git: {}",
                e
            )));
        }
    };

    if !status.success() {
        let _ = tokio::fs::remove_dir_all(&temp_dir).await;
        return Err(AppError::InternalServerError(
            "Failed to clone template repository".to_string(),
        ));
    }

    // 2. Identify template source directory
    // SEC: Prevent traversal within the temp clone
    let source_path = match crate::utils::security::validate_path(&temp_dir, &payload.path) {
        Ok(p) => p,
        Err(e) => {
            let _ = tokio::fs::remove_dir_all(&temp_dir).await;
            return Err(AppError::BadRequest(format!(
                "Invalid template path: {}",
                e
            )));
        }
    };
    if !source_path.exists() {
        let _ = tokio::fs::remove_dir_all(&temp_dir).await;
        return Err(AppError::NotFound(format!(
            "Template path '{}' not found in repo",
            payload.path
        )));
    }

    // 3. Copy agents to the vault
    let agents_src = source_path.join("agents");
    let agents_dest = PathBuf::from("data/swarm_config/agents");

    if agents_src.exists() {
        let _ = tokio::fs::create_dir_all(&agents_dest).await;
        if let Ok(mut entries) = tokio::fs::read_dir(&agents_src).await {
            while let Ok(Some(entry)) = entries.next_entry().await {
                if let Ok(file_type) = entry.file_type().await {
                    if file_type.is_file() {
                        let dest_file = agents_dest.join(entry.file_name());
                        let _ = tokio::fs::copy(entry.path(), dest_file).await;
                    }
                }
            }
        }
    }

    let swarm_json_src = source_path.join("swarm.json");
    if swarm_json_src.exists() {
        let safe_name = crate::utils::security::sanitize_id(&payload.path.replace("/", "_"));
        let dest_folder = PathBuf::from("data/swarm_config/installed").join(safe_name);
        let _ = tokio::fs::create_dir_all(&dest_folder).await;
        let _ = tokio::fs::copy(&swarm_json_src, dest_folder.join("swarm.json")).await;
    }

    // 3.5 Copy workflows to directives
    let workflows_src = source_path.join("workflows");
    let workflows_dest = PathBuf::from("directives");
    if workflows_src.exists() {
        let _ = tokio::fs::create_dir_all(&workflows_dest).await;
        if let Ok(mut entries) = tokio::fs::read_dir(&workflows_src).await {
            while let Ok(Some(entry)) = entries.next_entry().await {
                if entry.path().extension().and_then(|s| s.to_str()) == Some("md") {
                    let dest_file = workflows_dest.join(entry.file_name());
                    let _ = tokio::fs::copy(entry.path(), dest_file).await;
                }
            }
        }
    }

    // 3.6 Copy skills to execution
    let skills_src = source_path.join("skills");
    let skills_dest = PathBuf::from("execution");
    if skills_src.exists() {
        let _ = tokio::fs::create_dir_all(&skills_dest).await;
        if let Ok(mut entries) = tokio::fs::read_dir(&skills_src).await {
            while let Ok(Some(entry)) = entries.next_entry().await {
                let ext = entry
                    .path()
                    .extension()
                    .and_then(|s| s.to_str())
                    .unwrap_or("")
                    .to_lowercase();
                if ext == "json" || ext == "py" || ext == "js" || ext == "ts" {
                    let dest_file = skills_dest.join(entry.file_name());
                    let _ = tokio::fs::copy(entry.path(), dest_file).await;
                }
            }
        }
    }

    // 3.7 Merge MCP configuration
    let mcps_src = source_path.join("mcps.json");
    if mcps_src.exists() {
        if let Ok(content) = tokio::fs::read_to_string(&mcps_src).await {
            if let Ok(incoming_config) =
                serde_json::from_str::<crate::agent::mcp::McpConfig>(&content)
            {
                let mcp_config_path = PathBuf::from(".agent/mcp_config.json");
                let mut current_config = if mcp_config_path.exists() {
                    tokio::fs::read_to_string(&mcp_config_path)
                        .await
                        .ok()
                        .and_then(|c| serde_json::from_str::<crate::agent::mcp::McpConfig>(&c).ok())
                        .unwrap_or_else(|| crate::agent::mcp::McpConfig {
                            mcp_servers: std::collections::HashMap::new(),
                        })
                } else {
                    crate::agent::mcp::McpConfig {
                        mcp_servers: std::collections::HashMap::new(),
                    }
                };

                // Merge servers
                for (name, config) in incoming_config.mcp_servers {
                    current_config.mcp_servers.insert(name, config);
                }

                if let Ok(merged_json) = serde_json::to_string_pretty(&current_config) {
                    let _ = tokio::fs::create_dir_all(".agent").await;
                    let _ = tokio::fs::write(mcp_config_path, merged_json).await;
                }
            }
        }
    }

    // 4. Parse installed agent JSONs directly and save to database
    let mut loaded_agents = Vec::new();
    if agents_dest.exists() {
        if let Ok(mut entries) = tokio::fs::read_dir(&agents_dest).await {
            while let Ok(Some(entry)) = entries.next_entry().await {
                let path = entry.path();
                if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("json") {
                    if let Ok(content) = tokio::fs::read_to_string(&path).await {
                        if let Ok(agent) =
                            serde_json::from_str::<crate::agent::types::EngineAgent>(&content)
                        {
                            // Persist to SQLite
                            let _ = crate::agent::persistence::save_agent_db(
                                &state.resources.pool,
                                &agent,
                            )
                            .await;
                            loaded_agents.push(agent);
                        }
                    }
                }
            }
        }
    }

    // Populate active in-memory registry
    for agent in loaded_agents {
        state.registry.agents.insert(agent.identity.id.clone(), agent);
    }

    // 5. Cleanup
    let _ = tokio::fs::remove_dir_all(&temp_dir).await;

    Ok((
        StatusCode::OK,
        Json(InstallTemplateResponse {
            status: "success".to_string(),
            message: format!(
                "Successfully installed swarm template from {}",
                payload.path
            ),
        }),
    ))
}

// Metadata: [templates]

// Metadata: [templates]

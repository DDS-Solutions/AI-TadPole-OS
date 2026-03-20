//! Rapid deployment templates for pre-configured agent clusters.
//!
//! Provides endpoints for listing, installing, and managing "Starter Kits" 
//! that allow users to deploy complex multi-agent swarms with one click.

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
pub async fn install_template(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<InstallTemplateRequest>,
) -> impl IntoResponse {
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
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(InstallTemplateResponse {
                    status: "error".to_string(),
                    message: format!("Failed to execute git: {}", e),
                }),
            )
                .into_response();
        }
    };

    if !status.success() {
        let _ = tokio::fs::remove_dir_all(&temp_dir).await;
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(InstallTemplateResponse {
                status: "error".to_string(),
                message: "Failed to clone template repository".to_string(),
            }),
        )
            .into_response();
    }

    // 2. Identify template source directory
    let source_path = temp_dir.join(&payload.path);
    if !source_path.exists() {
        let _ = tokio::fs::remove_dir_all(&temp_dir).await;
        return (
            StatusCode::NOT_FOUND,
            Json(InstallTemplateResponse {
                status: "error".to_string(),
                message: format!("Template path '{}' not found in repo", payload.path),
            }),
        )
            .into_response();
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
        let dest_folder = PathBuf::from("data/swarm_config/installed").join(payload.path.replace("/", "_"));
        let _ = tokio::fs::create_dir_all(&dest_folder).await;
        let _ = tokio::fs::copy(&swarm_json_src, dest_folder.join("swarm.json")).await;
    }

    // 4. Parse installed agent JSONs directly and save to database
    let mut loaded_agents = Vec::new();
    if agents_dest.exists() {
        if let Ok(mut entries) = tokio::fs::read_dir(&agents_dest).await {
            while let Ok(Some(entry)) = entries.next_entry().await {
                let path = entry.path();
                if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("json") {
                    if let Ok(content) = tokio::fs::read_to_string(&path).await {
                        if let Ok(agent) = serde_json::from_str::<crate::agent::types::EngineAgent>(&content) {
                            // Persist to SQLite
                            let _ = crate::agent::persistence::save_agent_db(&state.resources.pool, &agent).await;
                            loaded_agents.push(agent);
                        }
                    }
                }
            }
        }
    }

    // Populate active in-memory registry
    for agent in loaded_agents {
        state.registry.agents.insert(agent.id.clone(), agent);
    }

    // 5. Cleanup
    let _ = tokio::fs::remove_dir_all(&temp_dir).await;

    (
        StatusCode::OK,
        Json(InstallTemplateResponse {
            status: "success".to_string(),
            message: format!("Successfully installed swarm template from {}", payload.path),
        }),
    )
        .into_response()
}

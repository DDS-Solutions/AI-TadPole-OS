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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TemplateCatalogEntry {
    pub id: String,
    pub name: String,
    pub description: String,
    pub repository_url: String,
    pub path: String,
    #[serde(default)]
    pub required_models: Vec<String>,
    #[serde(default)]
    pub required_skills: Vec<String>,
}

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
#[tracing::instrument(skip(state), name = "templates::catalog")]
pub async fn get_templates_catalog(
    State(state): State<Arc<AppState>>,
) -> Result<axum::response::Response, AppError> {
    let url = "https://raw.githubusercontent.com/DDS-Solutions/AI-Tadpole-OS-Industry-Templates/main/index.json";

    tracing::info!("📥 [Templates] Fetching template catalog from {}", url);

    let response = match state.resources.http_client.get(url).send().await {
        Ok(res) => res,
        Err(e) => {
            tracing::warn!(
                "⚠️ [Templates] Failed to fetch remote template catalog: {:?}",
                e
            );
            let offline_catalog = vec![TemplateCatalogEntry {
                id: "marketing-swarm".to_string(),
                name: "Marketing Swarm (Offline Fallback)".to_string(),
                description: "Offline fallback for the marketing automation swarm. Run locally."
                    .to_string(),
                repository_url:
                    "https://github.com/DDS-Solutions/AI-Tadpole-OS-Industry-Templates.git"
                        .to_string(),
                path: "templates/marketing".to_string(),
                required_models: vec!["llama-3.3-70b-versatile".to_string()],
                required_skills: vec!["web_search".to_string(), "write_file".to_string()],
            }];
            return Ok((StatusCode::OK, Json(offline_catalog)).into_response());
        }
    };

    if !response.status().is_success() {
        return Err(AppError::InternalServerError(format!(
            "Template catalog server returned status {}",
            response.status()
        )));
    }

    let catalog = response
        .json::<Vec<TemplateCatalogEntry>>()
        .await
        .map_err(|e| {
            AppError::InternalServerError(format!("Failed to parse template catalog JSON: {}", e))
        })?;

    Ok((StatusCode::OK, Json(catalog)).into_response())
}

#[axum::debug_handler]
#[tracing::instrument(skip(state, payload), name = "templates::install")]
pub async fn install_template(
    State(state): State<Arc<AppState>>,
    _admin: crate::middleware::auth::RequireAdmin,
    Json(payload): Json<InstallTemplateRequest>,
) -> Result<axum::response::Response, AppError> {
    let validated_target =
        crate::utils::security::validate_public_http_url(&payload.repository_url).await?;

    let dl_id = uuid::Uuid::new_v4().to_string();
    let temp_dir = std::env::current_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join("data")
        .join(".bunker_cache")
        .join(&dl_id);

    let _ = tokio::fs::create_dir_all(&temp_dir).await;

    // 1. Clone repository
    let resolve_arg = format!(
        "http.curloptResolve={}:{}:{}",
        validated_target.host, validated_target.port, validated_target.ip
    );

    let mut git_cmd = tokio::process::Command::new("git");
    if cfg!(target_os = "windows") {
        git_cmd.arg("-c").arg("http.sslBackend=schannel");
    }
    let status = match git_cmd
        .arg("-c")
        .arg(&resolve_arg)
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
    let agents_src = source_path.as_path().join("agents");
    let agents_dest = PathBuf::from("data/swarm_config/agents");

    if agents_src.exists() {
        let _ = tokio::fs::create_dir_all(&agents_dest).await;
        if let Ok(mut entries) = tokio::fs::read_dir(&agents_src).await {
            while let Ok(Some(entry)) = entries.next_entry().await {
                if let Ok(file_type) = entry.file_type().await {
                    if !file_type.is_symlink() && file_type.is_file() {
                        let file_name = entry.file_name();
                        let dest_file = crate::utils::security::validate_path(
                            &agents_dest,
                            &file_name.to_string_lossy(),
                        )?
                        .to_path_buf();
                        if dest_file.exists() {
                            return Err(AppError::Forbidden(format!(
                                "Security boundary: Refusing to overwrite existing agent '{}'",
                                file_name.to_string_lossy()
                            )));
                        }
                        tokio::fs::copy(entry.path(), &dest_file).await?;
                    }
                }
            }
        }
    }

    let swarm_json_src = source_path.as_path().join("swarm.json");
    if swarm_json_src.exists() {
        if let Ok(meta) = tokio::fs::symlink_metadata(&swarm_json_src).await {
            if !meta.file_type().is_symlink() {
                let safe_name =
                    crate::utils::security::sanitize_id(&payload.path.replace("/", "_"));
                let dest_folder = PathBuf::from("data/swarm_config/installed").join(safe_name);
                let _ = tokio::fs::create_dir_all(&dest_folder).await;
                let dest_file = dest_folder.join("swarm.json");
                if dest_file.exists() {
                    return Err(AppError::Forbidden(format!(
                        "Security boundary: Refusing to overwrite existing file '{:?}'",
                        dest_file
                    )));
                }
                tokio::fs::copy(&swarm_json_src, &dest_file).await?;
            }
        }
    }

    // 3.5 Copy workflows to directives
    let workflows_src = source_path.as_path().join("workflows");
    let workflows_dest = PathBuf::from("directives");
    if workflows_src.exists() {
        let _ = tokio::fs::create_dir_all(&workflows_dest).await;
        if let Ok(mut entries) = tokio::fs::read_dir(&workflows_src).await {
            while let Ok(Some(entry)) = entries.next_entry().await {
                if let Ok(file_type) = entry.file_type().await {
                    if !file_type.is_symlink()
                        && entry.path().extension().and_then(|s| s.to_str()) == Some("md")
                    {
                        let file_name = entry.file_name();
                        let dest_file = crate::utils::security::validate_path(
                            &workflows_dest,
                            &file_name.to_string_lossy(),
                        )?
                        .to_path_buf();
                        if dest_file.exists() {
                            return Err(AppError::Forbidden(format!(
                                "Security boundary: Refusing to overwrite existing workflow '{}'",
                                file_name.to_string_lossy()
                            )));
                        }
                        tokio::fs::copy(entry.path(), &dest_file).await?;
                    }
                }
            }
        }
    }

    // 3.6 Copy skills to execution
    let skills_src = source_path.as_path().join("skills");
    let skills_dest = PathBuf::from("execution");
    if skills_src.exists() {
        // Run security scan gate on all incoming skill files
        if let Ok(mut entries) = tokio::fs::read_dir(&skills_src).await {
            while let Ok(Some(entry)) = entries.next_entry().await {
                if let Ok(file_type) = entry.file_type().await {
                    let ext = entry
                        .path()
                        .extension()
                        .and_then(|s| s.to_str())
                        .unwrap_or("")
                        .to_lowercase();
                    if !file_type.is_symlink()
                        && (ext == "json" || ext == "py" || ext == "js" || ext == "ts")
                    {
                        match crate::security::skillspector::scan_path(&entry.path()) {
                            Ok(scan_res) => {
                                tracing::info!(
                                    "🔍 [Templates Security Scan] File: {:?}, Score: {}, Severity: {}",
                                    entry.file_name(),
                                    scan_res.risk_score,
                                    scan_res.risk_severity
                                );
                                if scan_res.risk_score >= 50 {
                                    let _ = tokio::fs::remove_dir_all(&temp_dir).await;
                                    return Err(AppError::Forbidden(format!(
                                        "Security audit failed: template skill '{}' has a high-risk security score of {} (Severity: {}). Installation aborted.",
                                        entry.file_name().to_string_lossy(),
                                        scan_res.risk_score,
                                        scan_res.risk_severity
                                    )));
                                }
                            }
                            Err(e) => {
                                tracing::warn!(
                                    "⚠️ [Templates Security Scan] Failed to scan template skill file {:?}: {:?}",
                                    entry.path(),
                                    e
                                );
                            }
                        }
                    }
                }
            }
        }

        let _ = tokio::fs::create_dir_all(&skills_dest).await;
        if let Ok(mut entries) = tokio::fs::read_dir(&skills_src).await {
            while let Ok(Some(entry)) = entries.next_entry().await {
                if let Ok(file_type) = entry.file_type().await {
                    let ext = entry
                        .path()
                        .extension()
                        .and_then(|s: &std::ffi::OsStr| s.to_str())
                        .unwrap_or("")
                        .to_lowercase();
                    if !file_type.is_symlink()
                        && (ext == "json" || ext == "py" || ext == "js" || ext == "ts")
                    {
                        let file_name = entry.file_name();
                        let dest_file = crate::utils::security::validate_path(
                            &skills_dest,
                            &file_name.to_string_lossy(),
                        )?
                        .to_path_buf();
                        if dest_file.exists() {
                            return Err(AppError::Forbidden(format!(
                                "Security boundary: Refusing to overwrite existing skill '{}'",
                                file_name.to_string_lossy()
                            )));
                        }
                        tokio::fs::copy(entry.path(), &dest_file).await?;
                    }
                }
            }
        }
    }

    // 3.7 Merge MCP configuration
    let mcps_src = source_path.as_path().join("mcps.json");
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
                        if let Ok(mut agent) =
                            serde_json::from_str::<crate::agent::types::EngineAgent>(&content)
                        {
                            // Persist to SQLite
                            let _ = crate::agent::persistence::save_agent_db(
                                &state.resources.pool,
                                &mut agent,
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
        state
            .registry
            .agents
            .insert(agent.identity.id.clone(), agent);
    }

    // 4.5 Ingest Knowledge into OKF (Institutional Knowledge Store)
    #[cfg(feature = "vector-memory")]
    {
        if let Some(ks) = state.resources.get_knowledge_store().await.ok() {
            let knowledge_json_src = source_path.as_path().join("knowledge.json");
            if knowledge_json_src.exists() {
                if let Ok(content) = tokio::fs::read_to_string(&knowledge_json_src).await {
                    if let Ok(requests) = serde_json::from_str::<
                        Vec<crate::agent::knowledge_store::AddKnowledgeRequest>,
                    >(&content)
                    {
                        for req in requests {
                            let client = state.resources.http_client.as_ref().clone();
                            // Resilient import: log warning but do not fail installation
                            if let Err(e) = ks.add_entry(req, client).await {
                                tracing::warn!("⚠️ [Templates] Failed to ingest template knowledge JSON entry: {:?}", e);
                            }
                        }
                    }
                }
            }

            let knowledge_dir = source_path.as_path().join("knowledge");
            if knowledge_dir.exists() {
                if let Ok(mut entries) = tokio::fs::read_dir(&knowledge_dir).await {
                    while let Ok(Some(entry)) = entries.next_entry().await {
                        let path = entry.path();
                        if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("md")
                        {
                            if let Ok(content) = tokio::fs::read_to_string(&path).await {
                                let (fm_title, fm_uri, fm_tags, fm_desc, body_content) =
                                    parse_frontmatter(&content);

                                let title = fm_title.unwrap_or_else(|| {
                                    path.file_stem()
                                        .and_then(|s| s.to_str())
                                        .unwrap_or("Untitled Template Doc")
                                        .replace("_", " ")
                                        .replace("-", " ")
                                });

                                let req = crate::agent::knowledge_store::AddKnowledgeRequest {
                                    text: body_content,
                                    topic: "templates".to_string(),
                                    cluster_id: None,
                                    source_node_id: None,
                                    source_agent_id: None,
                                    confidence: Some(1.0),
                                    ttl_days: None,
                                    human_confirmed: Some(true),
                                    concept_type: Some("playbook".to_string()),
                                    title: Some(title),
                                    description: fm_desc.or_else(|| {
                                        Some(format!("Imported from template {}", payload.path))
                                    }),
                                    resource_uri: fm_uri,
                                    tags: fm_tags.or_else(|| Some("template,imported".to_string())),
                                };
                                let client = state.resources.http_client.as_ref().clone();
                                if let Err(e) = ks.add_entry(req, client).await {
                                    tracing::warn!("⚠️ [Templates] Failed to ingest template markdown knowledge: {:?}", e);
                                }
                            }
                        }
                    }
                }
            }
        }
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
    )
        .into_response())
}

#[cfg(feature = "vector-memory")]
fn parse_frontmatter(
    content: &str,
) -> (
    Option<String>,
    Option<String>,
    Option<String>,
    Option<String>,
    String,
) {
    let content_trimmed = content.trim_start();
    if !content_trimmed.starts_with("---") {
        return (None, None, None, None, content.to_string());
    }

    let mut parts = content_trimmed.splitn(3, "---");
    // First split is empty because of starts_with("---")
    let _ = parts.next();
    let frontmatter_text = match parts.next() {
        Some(t) => t,
        None => return (None, None, None, None, content.to_string()),
    };
    let body_text = parts.next().unwrap_or("").trim_start().to_string();

    let mut title = None;
    let mut resource_uri = None;
    let mut tags = None;
    let mut description = None;

    for line in frontmatter_text.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        if let Some((key, val)) = line.split_once(':') {
            let key = key.trim().to_lowercase();
            let val = val.trim().to_string();
            // Remove surrounding quotes if any
            let clean_val = if (val.starts_with('"') && val.ends_with('"'))
                || (val.starts_with('\'') && val.ends_with('\''))
            {
                val[1..val.len() - 1].to_string()
            } else {
                val
            };

            match key.as_str() {
                "title" => title = Some(clean_val),
                "resource_uri" | "url" => resource_uri = Some(clean_val),
                "tags" => tags = Some(clean_val),
                "description" | "desc" => description = Some(clean_val),
                _ => {}
            }
        }
    }

    (title, resource_uri, tags, description, body_text)
}

// Metadata: [templates]

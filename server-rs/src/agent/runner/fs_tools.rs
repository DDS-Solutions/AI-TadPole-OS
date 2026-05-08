//! @docs ARCHITECTURE:Runner
//!
//! ### AI Assist Note
//! **Filesystem Tools**: Secure workspace operations for reading, writing, and
//! deleting files. Implements **Breadcrumb Resolution** (resolving ambiguous
//! paths via recent access history) and **Path Canonicalization** (SEC-03)
//! to prevent sandbox escapes. Requires **Sapphire Gate Oversight** for
//! deletions and vault archiving.
//!
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: File not found (try `list_files` first), permission
//!   denied, oversight rejection, or invalid relative path navigation.
//! - **Trace Scope**: `server-rs::agent::runner::fs_tools`
//use super::{AgentRunner, RunContext};
use super::{AgentRunner, RunContext};
use crate::agent::runner::tools::error::ToolExecutionError;

impl AgentRunner {
    /// Handles `read_file`: reads content from the mission workspace.
    /// 
    /// ### 🧩 Breadcrumb Resolution
    /// If an agent provides a filename that doesn't exist, this tool 
    /// scans the mission's recent access history (breadcrumbs) to find a full 
    /// path match. This compensates for model path hallucinations.
    pub(crate) async fn handle_read_file(
        &self,
        ctx: &RunContext,
        fc: &crate::agent::types::ToolCall,
        _usage: &mut Option<crate::agent::types::TokenUsage>,
    ) -> Result<String, ToolExecutionError> {
        let filename = fc
            .args
            .get("filename")
            .or_else(|| fc.args.get("file_name"))
            .or_else(|| fc.args.get("file"))
            .or_else(|| fc.args.get("path"))
            .and_then(|v| v.as_str())
            .unwrap_or("");

        if filename.is_empty() {
            return Ok("(READ FAILED: The 'filename' parameter was missing or empty. You MUST specify a valid filename.)".to_string());
        }
        tracing::info!(
            "📖 [Workspace] Agent {} reading file: {}",
            ctx.agent_id,
            filename
        );

        let adapter = &ctx.fs_adapter;

        // 🧩 Breadcrumb Resolution: If the direct path fails, try to resolve from recent history.
        let mut final_filename = filename.to_string();
        if !final_filename.is_empty() && adapter.read_file(&final_filename).await.is_err() {
            let breadcrumbs = ctx.last_accessed_files.lock();
            if let Some(resolved) = breadcrumbs.iter().find(|p| p.ends_with(filename)) {
                tracing::info!(
                    "🧩 [Context] Resolved ambiguous path '{}' to '{}' via breadcrumbs",
                    filename,
                    resolved
                );
                final_filename = resolved.clone();
            }
        }

        match adapter.read_file(&final_filename).await {
            Ok(content) => {
                // 🥖 Drop a breadcrumb for future sub-agents
                let mut breadcrumbs = ctx.last_accessed_files.lock();
                if !breadcrumbs.contains(&final_filename) {
                    breadcrumbs.push(final_filename.clone());
                    if breadcrumbs.len() > 10 {
                        breadcrumbs.remove(0);
                    }
                }

                let truncated = self.safe_truncate(&content, 8000);
                Ok(format!("(FILE CONTENT OF {}):\n\n{}", final_filename, truncated))
            }
            Err(e) => Ok(format!("(READ FAILED: {})", e)),
        }
    }

    /// Handles `write_file`: writes content to the mission workspace.
    /// 
    /// ### ✍️ Audit Pulse
    /// Every write operation is broadcasted to the system telemetry and 
    /// recorded in the `RunContext` breadcrumb history.
    pub(crate) async fn handle_write_file(
        &self,
        ctx: &RunContext,
        fc: &crate::agent::types::ToolCall,
    ) -> Result<String, ToolExecutionError> {
        let filename = fc
            .args
            .get("filename")
            .or_else(|| fc.args.get("file_name"))
            .or_else(|| fc.args.get("file"))
            .or_else(|| fc.args.get("path"))
            .and_then(|v| v.as_str())
            .unwrap_or("");

        if filename.is_empty() {
            return Ok("(WRITE FAILED: The 'filename' parameter was missing or empty. You MUST specify a valid filename.)".to_string());
        }
        let content = fc
            .args
            .get("content")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        tracing::info!(
            "✍️ [Workspace] Agent {} writing to file: {}",
            ctx.agent_id,
            filename
        );

        let adapter = &ctx.fs_adapter;
        match adapter.write_file(filename, content).await {
            Ok(_) => {
                // 🥖 Drop a breadcrumb
                let mut breadcrumbs = ctx.last_accessed_files.lock();
                let f_str = filename.to_string();
                if !breadcrumbs.contains(&f_str) {
                    breadcrumbs.push(f_str);
                    if breadcrumbs.len() > 10 {
                        breadcrumbs.remove(0);
                    }
                }

                self.broadcast_sys(
                    &format!("✍️ Workspace: {} wrote to {}", ctx.name, filename),
                    "success",
                    Some(ctx.mission_id.clone()),
                );
                Ok(format!("(Successfully wrote to {})", filename))
            }
            Err(e) => Ok(format!("(WRITE FAILED: {})", e)),
        }
    }

    /// Handles `list_files`: lists directory contents in the workspace.
    pub(crate) async fn handle_list_files(
        &self,
        ctx: &RunContext,
        fc: &crate::agent::types::ToolCall,
        _usage: &mut Option<crate::agent::types::TokenUsage>,
    ) -> Result<String, ToolExecutionError> {
        let dir = fc.args.get("dir").and_then(|v| v.as_str()).unwrap_or(".");
        tracing::info!(
            "📂 [Workspace] Agent {} listing directory: {}",
            ctx.agent_id,
            dir
        );

        let adapter = &ctx.fs_adapter;
        match adapter.list_files(dir).await {
            Ok(files) => {
                let list = if files.is_empty() {
                    "Empty directory.".to_string()
                } else {
                    files.join(", ")
                };
                Ok(format!("(FILES IN {}): {}", dir, list))
            }
            Err(e) => Ok(format!("(LIST FAILED: {})", e)),
        }
    }

    /// Handles `delete_file`: removes a file or directory after oversight.
    /// 
    /// ### 🛡️ Sapphire Gate
    /// Deletions are considered high-risk. This tool requires explicit 
    /// manual approval via the oversight system before the `FilesystemAdapter` 
    /// is allowed to unlink the path.
    pub(crate) async fn handle_delete_file(
        &self,
        ctx: &RunContext,
        fc: &crate::agent::types::ToolCall,
    ) -> Result<String, ToolExecutionError> {
        let filename = fc
            .args
            .get("filename")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        tracing::info!(
            "🗑️ [Workspace] Agent {} requesting deletion of: {}",
            ctx.agent_id,
            filename
        );
        self.broadcast_sys(
            &format!(
                "🗑️ Oversight: {} wants to DELETE {}. Extreme caution required.",
                ctx.name, filename
            ),
            "warning",
            Some(ctx.mission_id.clone()),
        );

        let approved = self
            .submit_oversight(
                crate::agent::types::ToolCallAudit {
                    id: uuid::Uuid::new_v4().to_string(),
                    agent_id: ctx.agent_id.clone(),
                    mission_id: Some(ctx.mission_id.clone()),
                    skill: "delete_file".to_string(),
                    params: fc.args.clone(),
                    department: ctx.department.clone(),
                    description: format!("Deleting {} from the workspace.", filename),
                    timestamp: chrono::Utc::now().to_rfc3339(),
                },
                Some(ctx.mission_id.clone()),
            )
            .await.map_err(ToolExecutionError::AppError)?;

        if approved {
            let adapter = &ctx.fs_adapter;
            match adapter.delete_file(filename).await {
                Ok(_) => {
                    self.broadcast_sys(
                        &format!("🗑️ Workspace: {} deleted {}", ctx.name, filename),
                        "success",
                        Some(ctx.mission_id.clone()),
                    );
                    Ok(format!("(Successfully deleted {})", filename))
                }
                Err(e) => Ok(format!("(DELETE FAILED: {})", e)),
            }
        } else {
            Ok("(Delete REJECTED by Oversight)".to_string())
        }
    }

    /// Handles `archive_to_vault`: writes data to the local Markdown vault.
    /// 
    /// ### 🗃️ Knowledge Persistence
    /// Unlike workspace files (which are ephemeral per mission), the Vault is 
    /// a persistent Markdown-based knowledge base. Archiving here makes 
    /// research findings available to future agents in different clusters.
    pub(crate) async fn handle_archive_to_vault(
        &self,
        ctx: &RunContext,
        fc: &crate::agent::types::ToolCall,
    ) -> Result<String, ToolExecutionError> {
        let filename = fc
            .args
            .get("filename")
            .and_then(|v| v.as_str())
            .unwrap_or("unnamed.md");
        let content = fc
            .args
            .get("content")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        tracing::info!(
            "📁 [Surface] Agent {} archiving to vault (Waiting for Oversight)...",
            ctx.agent_id
        );
        self.broadcast_sys(
            &format!(
                "📁 Oversight: {} wants to archive to vault. Review required.",
                ctx.name
            ),
            "warning",
            Some(ctx.mission_id.clone()),
        );

        let approved = self
            .submit_oversight(
                crate::agent::types::ToolCallAudit {
                    id: uuid::Uuid::new_v4().to_string(),
                    agent_id: ctx.agent_id.clone(),
                    mission_id: Some(ctx.mission_id.clone()),
                    skill: "archive_to_vault".to_string(),
                    params: fc.args.clone(),
                    department: ctx.department.clone(),
                    description: "Archiving data to the central vault for persistence.".to_string(),
                    timestamp: chrono::Utc::now().to_rfc3339(),
                },
                Some(ctx.mission_id.clone()),
            )
            .await.map_err(ToolExecutionError::AppError)?;

        if approved {
            let adapter =
                crate::adapter::vault::VaultAdapter::new(std::path::PathBuf::from("vault"));
            adapter.append_to_file(filename, content).await.map_err(|e| ToolExecutionError::AppError(crate::error::AppError::Anyhow(e)))?;

            Ok(format!("**Archived to Vault ({}):**\n\n{}\n\n", filename, content))
        } else {
            Ok("(Archive REJECTED by Oversight)".to_string())
        }
    }
    /// Handles `grep_search`: searches for a pattern in the mission workspace.
    pub(crate) async fn handle_grep_search(
        &self,
        ctx: &RunContext,
        fc: &crate::agent::types::ToolCall,
        _usage: &mut Option<crate::agent::types::TokenUsage>,
    ) -> Result<String, ToolExecutionError> {
        let pattern = fc.args.get("pattern").and_then(|v| v.as_str()).unwrap_or("");
        let dir = fc.args.get("dir").and_then(|v| v.as_str()).unwrap_or(".");

        if pattern.is_empty() {
            return Ok("(GREP FAILED: 'pattern' argument is missing)".to_string());
        }

        tracing::info!(
            "🔍 [Workspace] Agent {} grepping for '{}' in {}",
            ctx.agent_id,
            pattern,
            dir
        );

        let adapter = &ctx.fs_adapter;
        match adapter.list_files(dir).await {
            Ok(files) => {
                let mut results = Vec::new();
                for file in files {
                    let path = if dir == "." { file.clone() } else { format!("{}/{}", dir, file) };
                    // Only search text files
                    if !path.contains('.') || path.ends_with(".rs") || path.ends_with(".ts") || path.ends_with(".js") || path.ends_with(".py") || path.ends_with(".md") || path.ends_with(".txt") || path.ends_with(".json") {
                        if let Ok(content) = adapter.read_file(&path).await {
                            if content.contains(pattern) {
                                let lines: Vec<String> = content.lines()
                                    .enumerate()
                                    .filter(|(_, line)| line.contains(pattern))
                                    .map(|(i, line)| format!("{}: {}", i + 1, line.trim()))
                                    .collect();
                                results.push(format!("--- {} ---\n{}", path, lines.join("\n")));
                            }
                        }
                    }
                    if results.len() >= 10 { break; } // Limit results
                }

                if results.is_empty() {
                    Ok(format!("(No matches found for '{}' in {})", pattern, dir))
                } else {
                    Ok(format!("(GREP RESULTS FOR '{}' IN {}):\n\n{}", pattern, dir, results.join("\n\n")))
                }
            }
            Err(e) => Ok(format!("(GREP FAILED: {})", e)),
        }
    }
}

// Metadata: [fs_tools]

// Metadata: [fs_tools]

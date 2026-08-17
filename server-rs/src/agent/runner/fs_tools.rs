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

pub const MAX_BREADCRUMBS: usize = 10;
pub const READ_TRUNCATE_LIMIT: usize = 8000;
pub const GREP_MAX_FILE_SIZE: u64 = 5 * 1024 * 1024;
pub const GREP_MAX_RESULTS: usize = 10;

impl AgentRunner {
    /// Helper to extract filename/path argument from tool call aliases.
    fn extract_path_arg(fc: &crate::agent::types::ToolCall) -> &str {
        ["filename", "file_name", "file", "path"]
            .iter()
            .find_map(|k| fc.args.get(*k).and_then(|v| v.as_str()))
            .unwrap_or("")
    }

    /// Helper to resolve ambiguous path argument using breadcrumb access history if direct path doesn't exist.
    fn resolve_path_arg(ctx: &RunContext, fc: &crate::agent::types::ToolCall) -> String {
        let raw = Self::extract_path_arg(fc);
        if raw.is_empty() {
            return String::new();
        }

        let abs_path = ctx.workspace_root.join(raw);
        if abs_path.exists() {
            return raw.to_string();
        }

        let breadcrumbs = ctx.last_accessed_files.lock();
        if let Some(resolved) = breadcrumbs.iter().find(|p| {
            p == &raw || p.ends_with(&format!("/{}", raw)) || p.ends_with(&format!("\\{}", raw))
        }) {
            tracing::info!(
                "🧩 [Context] Resolved ambiguous mutation path '{}' -> '{}' via breadcrumbs",
                raw,
                resolved
            );
            resolved.clone()
        } else {
            raw.to_string()
        }
    }

    /// Helper to record accessed path in mission breadcrumb context.
    fn record_fs_breadcrumb(ctx: &RunContext, path: &str) {
        let mut breadcrumbs = ctx.last_accessed_files.lock();
        if !breadcrumbs.iter().any(|p| p == path) {
            breadcrumbs.push(path.to_string());
            if breadcrumbs.len() > MAX_BREADCRUMBS {
                breadcrumbs.remove(0);
            }
        }
    }

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
        let final_filename = Self::resolve_path_arg(ctx, fc);

        if final_filename.is_empty() {
            return Ok("(READ FAILED: The 'filename' parameter was missing or empty. You MUST specify a valid filename.)".to_string());
        }
        tracing::info!(
            "📖 [Workspace] Agent {} reading file: {}",
            ctx.agent_id,
            final_filename
        );

        let adapter = &ctx.fs_adapter;

        let content = match adapter.read_file(&final_filename).await {
            Ok(c) => c,
            Err(e) => return Ok(format!("(READ FAILED: {})", e)),
        };

        Self::record_fs_breadcrumb(ctx, &final_filename);

        let start_line = fc
            .args
            .get("start_line")
            .or_else(|| fc.args.get("startLine"))
            .and_then(|v| v.as_i64())
            .unwrap_or(1) as usize;

        let end_line = fc
            .args
            .get("end_line")
            .or_else(|| fc.args.get("endLine"))
            .and_then(|v| v.as_i64())
            .unwrap_or(-1) as isize;

        // Slicing logic
        let lines: Vec<&str> = content.lines().collect();
        let total_lines = lines.len();
        let start_idx = (start_line.max(1) - 1).min(total_lines);
        let end_idx = if end_line < 0 {
            total_lines
        } else {
            (end_line as usize).min(total_lines)
        };

        let (sliced_content, header) = if start_idx < end_idx {
            (
                lines[start_idx..end_idx].join("\n"),
                format!(
                    "(FILE CONTENT OF {} - Lines {} to {} of {}):\n\n",
                    final_filename,
                    start_idx + 1,
                    end_idx,
                    total_lines
                ),
            )
        } else {
            (
                "".to_string(),
                format!("(FILE CONTENT OF {} - Empty range):\n\n", final_filename),
            )
        };

        let truncated = self.safe_truncate(&sliced_content, READ_TRUNCATE_LIMIT);
        Ok(format!("{}{}", header, truncated))
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
        let filename = Self::resolve_path_arg(ctx, fc);

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

        let target_abs = ctx.workspace_root.join(&filename);
        if let Err(e) = crate::services::cas::capture_pre_mutation(
            &self.state.resources.pool,
            &ctx.workspace_root,
            &target_abs,
            Some(&ctx.mission_id),
            Some(&ctx.agent_id),
        )
        .await
        {
            tracing::error!(
                "❌ [CAS] Pre-mutation capture failed for '{}': {}",
                filename,
                e
            );
        }

        let adapter = &ctx.fs_adapter;
        match adapter.write_file(&filename, content).await {
            Ok(_) => {
                Self::record_fs_breadcrumb(ctx, &filename);

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
        let filename = Self::resolve_path_arg(ctx, fc);

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
            .await
            .map_err(ToolExecutionError::AppError)?;

        if approved {
            let target_abs = ctx.workspace_root.join(&filename);
            if let Err(e) = crate::services::cas::capture_pre_mutation(
                &self.state.resources.pool,
                &ctx.workspace_root,
                &target_abs,
                Some(&ctx.mission_id),
                Some(&ctx.agent_id),
            )
            .await
            {
                tracing::error!(
                    "❌ [CAS] Pre-mutation capture failed for deletion of '{}': {}",
                    filename,
                    e
                );
            }

            let adapter = &ctx.fs_adapter;
            match adapter.delete_file(&filename).await {
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

    /// Handles `restore_file_version`: restores a workspace file to a target version number.
    pub(crate) async fn handle_restore_file_version(
        &self,
        ctx: &RunContext,
        fc: &crate::agent::types::ToolCall,
    ) -> Result<String, ToolExecutionError> {
        let filename = Self::resolve_path_arg(ctx, fc);
        let version_num = fc
            .args
            .get("version_num")
            .or_else(|| fc.args.get("version"))
            .and_then(|v| v.as_i64())
            .unwrap_or(0);

        if filename.is_empty() {
            return Ok("(RESTORE FAILED: 'filename' parameter is required)".to_string());
        }

        match crate::services::cas::restore_file_version(
            &self.state.resources.pool,
            &ctx.workspace_root,
            &filename,
            version_num,
        )
        .await
        {
            Ok(summary) => {
                self.broadcast_sys(
                    &format!(
                        "🔄 Workspace: {} restored {} to v{}",
                        ctx.name, filename, version_num
                    ),
                    "success",
                    Some(ctx.mission_id.clone()),
                );
                Ok(format!(
                    "(RESTORE SUCCESS: File '{}' restored to version v{} [hash: {}])",
                    filename,
                    version_num,
                    &summary.hash[..8.min(summary.hash.len())]
                ))
            }
            Err(e) => Ok(format!("(RESTORE FAILED: {})", e)),
        }
    }

    /// Handles `get_file_history`: retrieves revision history for a workspace file.
    pub(crate) async fn handle_get_file_history(
        &self,
        ctx: &RunContext,
        fc: &crate::agent::types::ToolCall,
    ) -> Result<String, ToolExecutionError> {
        let filename = Self::resolve_path_arg(ctx, fc);
        if filename.is_empty() {
            return Ok("(HISTORY FAILED: 'filename' parameter is required)".to_string());
        }

        match crate::services::cas::get_file_history(
            &self.state.resources.pool,
            &ctx.workspace_root,
            &filename,
        )
        .await
        {
            Ok(history) => {
                if history.is_empty() {
                    Ok(format!("(No revision history recorded for '{}')", filename))
                } else {
                    let lines: Vec<String> = history
                        .iter()
                        .map(|r| {
                            format!(
                                "- v{} | {} | {} bytes | agent: {}",
                                r.version_num,
                                &r.hash[..8.min(r.hash.len())],
                                r.size_bytes,
                                r.agent_id.as_deref().unwrap_or("unknown")
                            )
                        })
                        .collect();
                    Ok(format!(
                        "(REVISION HISTORY FOR {}):\n{}",
                        filename,
                        lines.join("\n")
                    ))
                }
            }
            Err(e) => Ok(format!("(HISTORY FAILED: {})", e)),
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
            .await
            .map_err(ToolExecutionError::AppError)?;

        if approved {
            let vault_dir = ctx.workspace_root.join("vault");
            let adapter = crate::adapter::vault::VaultAdapter::new(vault_dir);
            adapter
                .append_to_file(filename, content)
                .await
                .map_err(|e| ToolExecutionError::AppError(crate::error::AppError::Anyhow(e)))?;

            Ok(format!(
                "(Successfully archived {} bytes to vault file '{}')",
                content.len(),
                filename
            ))
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
        let pattern = fc
            .args
            .get("pattern")
            .and_then(|v| v.as_str())
            .unwrap_or("");
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
                    let path = if dir == "." {
                        file.clone()
                    } else {
                        format!("{}/{}", dir, file)
                    };
                    // Only search text/source code files
                    let ext = path.rsplit('.').next().unwrap_or("").to_lowercase();
                    let is_text_file = !path.contains('.')
                        || matches!(
                            ext.as_str(),
                            "rs" | "ts"
                                | "js"
                                | "tsx"
                                | "jsx"
                                | "py"
                                | "md"
                                | "txt"
                                | "json"
                                | "toml"
                                | "yaml"
                                | "yml"
                                | "sh"
                                | "ps1"
                                | "sql"
                                | "h"
                                | "hpp"
                                | "c"
                                | "cpp"
                                | "go"
                                | "css"
                                | "html"
                                | "xml"
                                | "csv"
                        );

                    if is_text_file {
                        // OOM Protection: Skip reading files larger than 5MB
                        let is_safe_size = adapter
                            .get_file_size(&path)
                            .await
                            .map(|sz| sz <= GREP_MAX_FILE_SIZE)
                            .unwrap_or(true);
                        if is_safe_size {
                            if let Ok(content) = adapter.read_file(&path).await {
                                if content.contains(pattern) {
                                    let lines: Vec<String> = content
                                        .lines()
                                        .enumerate()
                                        .filter(|(_, line)| line.contains(pattern))
                                        .map(|(i, line)| format!("{}: {}", i + 1, line.trim()))
                                        .collect();
                                    results.push(format!("--- {} ---\n{}", path, lines.join("\n")));
                                }
                            }
                        }
                    }
                    if results.len() >= GREP_MAX_RESULTS {
                        break;
                    } // Limit results
                }

                if results.is_empty() {
                    Ok(format!("(No matches found for '{}' in {})", pattern, dir))
                } else {
                    Ok(format!(
                        "(GREP RESULTS FOR '{}' IN {}):\n\n{}",
                        pattern,
                        dir,
                        results.join("\n\n")
                    ))
                }
            }
            Err(e) => Ok(format!("(GREP FAILED: {})", e)),
        }
    }
}

// Metadata: [fs_tools]

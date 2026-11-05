//! @docs ARCHITECTURE:Runner
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Agent Runner / fs_tools
//!
//! ### AI Assist Note
//! Resolve file aliases and optional breadcrumbs before authorization; handlers use the resolved target.
//!
//! ### 🔍 Debugging & Observability
//! - **Telemetry Targets**: none declared

use super::{AgentRunner, RunContext};
use crate::agent::runner::tools::error::ToolExecutionError;

pub const MAX_BREADCRUMBS: usize = 10;
pub const READ_TRUNCATE_LIMIT: usize = 8000;
pub const GREP_MAX_FILE_SIZE: u64 = 5 * 1024 * 1024;
pub const GREP_MAX_RESULTS: usize = 10;

fn resolve_file_path(
    args: &serde_json::Value,
    workspace_root: &std::path::Path,
    breadcrumbs: &[String],
) -> Result<String, ToolExecutionError> {
    let Some(mut path) = super::tools::validation::extract_path_from_args(args, workspace_root)?
    else {
        return Ok(String::new());
    };
    let root = workspace_root
        .canonicalize()
        .map_err(|e| ToolExecutionError::ExecutionFailed(e.to_string()))?;
    let relative = path
        .strip_prefix(&root)
        .map_err(|_| {
            ToolExecutionError::SecurityBlocked("File target is outside workspace".to_string())
        })?
        .to_string_lossy()
        .to_string();
    if !path.exists() {
        if let Some(candidate) = breadcrumbs.iter().find(|candidate| {
            candidate.as_str() == relative
                || candidate.ends_with(&format!("/{relative}"))
                || candidate.ends_with(&format!("\\{relative}"))
        }) {
            path = crate::utils::security::validate_path(workspace_root, candidate)
                .map_err(ToolExecutionError::AppError)?
                .to_path_buf();
        }
    }
    Ok(path
        .strip_prefix(&root)
        .map_err(|_| {
            ToolExecutionError::SecurityBlocked("File target is outside workspace".to_string())
        })?
        .to_string_lossy()
        .to_string())
}

impl AgentRunner {
    pub(crate) fn resolve_path_arg(
        ctx: &RunContext,
        fc: &crate::agent::types::ToolCall,
        use_breadcrumbs: bool,
    ) -> Result<String, ToolExecutionError> {
        let breadcrumbs = if use_breadcrumbs {
            ctx.last_accessed_files.lock().clone()
        } else {
            Vec::new()
        };
        resolve_file_path(&fc.args, &ctx.env.workspace_root, &breadcrumbs)
    }

    /// Helper to record accessed path in mission breadcrumb context.
    fn record_fs_breadcrumb(ctx: &RunContext, path: &str) {
        let path_obj = std::path::Path::new(path);
        let normalized = if path_obj.is_absolute() {
            if let Ok(rel) = path_obj.strip_prefix(&ctx.env.workspace_root) {
                rel.to_string_lossy().replace('\\', "/")
            } else {
                return; // Do not record external paths outside workspace root
            }
        } else {
            path.replace('\\', "/")
        };

        let mut breadcrumbs = ctx.last_accessed_files.lock();
        if !breadcrumbs.iter().any(|p| p == &normalized) {
            breadcrumbs.push(normalized);
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
        let final_filename = Self::resolve_path_arg(ctx, fc, false)?;

        if final_filename.is_empty() {
            return Ok("(READ FAILED: The 'filename' parameter was missing or empty. You MUST specify a valid filename.)".to_string());
        }
        tracing::info!(
            "📖 [Workspace] Agent {} reading file: {}",
            ctx.identity.agent_id,
            final_filename
        );

        let adapter = &ctx.env.fs_adapter;

        let content = match adapter.read_file(&final_filename).await {
            Ok(c) => c,
            Err(e) => {
                // 🌉 Codebase Fallback Bridge: Check if candidate exists in project repository root
                let root =
                    std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
                let is_in_repo = root.join(&final_filename).is_file()
                    || root.join(format!("server-rs/{}", final_filename)).is_file();

                if is_in_repo {
                    tracing::info!(
                        "🌉 [Bridge] Agent {} requested codebase file '{}' via read_file — auto-delegating to read_codebase_file",
                        ctx.identity.agent_id,
                        final_filename
                    );
                    let mut codebase_fc = fc.clone();
                    if let Some(obj) = codebase_fc.args.as_object_mut() {
                        obj.insert(
                            "path".to_string(),
                            serde_json::Value::String(final_filename.clone()),
                        );
                    } else {
                        codebase_fc.args = serde_json::json!({ "path": final_filename.clone() });
                    }
                    return self.handle_read_codebase_file(ctx, &codebase_fc).await;
                }

                return Ok(format!(
                    "(READ FAILED in workspace sandbox: {}. Note: If this is a project repository file, invoke 'read_codebase_file' with path '{}'.)",
                    e, final_filename
                ));
            }
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

        // Bounded line slicing without collecting all lines of the file into a Vec
        let total_lines = content.lines().count();
        let start_idx = (start_line.max(1) - 1).min(total_lines);
        let end_idx = if end_line < 0 {
            total_lines
        } else {
            (end_line as usize).min(total_lines)
        };

        let (sliced_content, header) = if start_idx < end_idx {
            let slice_len = end_idx - start_idx;
            let sliced_lines: Vec<&str> = content.lines().skip(start_idx).take(slice_len).collect();
            (
                sliced_lines.join("\n"),
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
        let filename = Self::resolve_path_arg(ctx, fc, false)?;

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
            ctx.identity.agent_id,
            filename
        );

        let adapter = &ctx.env.fs_adapter;
        let safe_target = match adapter.get_safe_path(&filename).await {
            Ok(p) => p,
            Err(e) => return Ok(format!("(WRITE FAILED: {})", e)),
        };

        // Guard concurrent file mutations across swarm agents with 30s TTL lease lock
        if let Err(e) = self
            .state
            .security
            .conflict_manager
            .acquire_lease(safe_target.clone(), ctx.identity.agent_id.clone())
        {
            tracing::warn!(
                "⚠️ [ConflictManager] Write lease denied for {}: {}",
                filename,
                e
            );
            return Ok(format!(
                "(WRITE FAILED: Concurrent file write collision: {})",
                e
            ));
        }

        if let Err(e) = crate::services::cas::capture_pre_mutation(
            &self.state.resources.pool,
            &ctx.env.workspace_root,
            &safe_target,
            Some(&ctx.mission_state.mission_id),
            Some(&ctx.identity.agent_id),
        )
        .await
        {
            tracing::error!(
                "❌ [CAS] Pre-mutation capture failed for '{}': {}",
                filename,
                e
            );
        }

        let write_result = adapter.write_file(&filename, content).await;
        self.state
            .security
            .conflict_manager
            .release_lease(&safe_target);

        match write_result {
            Ok(_) => {
                Self::record_fs_breadcrumb(ctx, &filename);

                self.broadcast_sys(
                    &format!("✍️ Workspace: {} wrote to {}", ctx.identity.name, filename),
                    "success",
                    Some(ctx.mission_state.mission_id.clone()),
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
        let dir = fc
            .args
            .get("dir")
            .or_else(|| fc.args.get("path"))
            .and_then(|v| v.as_str())
            .unwrap_or(".");
        tracing::info!(
            "📂 [Workspace] Agent {} listing directory: {}",
            ctx.identity.agent_id,
            dir
        );

        let adapter = &ctx.env.fs_adapter;
        match adapter.list_files(dir).await {
            Ok(files) => {
                let list = if files.is_empty() {
                    let root = &self.state.base_dir;
                    let clean_dir = dir
                        .trim_start_matches("./")
                        .trim_start_matches(".\\")
                        .replace('\\', "/");

                    let mut codebase_files = Vec::new();
                    if clean_dir != "." && !clean_dir.is_empty() {
                        let codebase_target = root.join(&clean_dir);
                        if codebase_target.is_dir() {
                            if let Ok(mut entries) = tokio::fs::read_dir(&codebase_target).await {
                                while let Ok(Some(entry)) = entries.next_entry().await {
                                    let name = entry.file_name().to_string_lossy().to_string();
                                    if name == "target" || name == ".git" || name == "node_modules"
                                    {
                                        continue;
                                    }
                                    let is_dir = entry
                                        .file_type()
                                        .await
                                        .map(|t| t.is_dir())
                                        .unwrap_or(false);
                                    codebase_files.push(if is_dir {
                                        format!("{}/", name)
                                    } else {
                                        name
                                    });
                                }
                                codebase_files.sort();
                            }
                        }
                    }

                    if !codebase_files.is_empty() {
                        return Ok(format!(
                            "(FILES IN {} [from Codebase Root]): {}",
                            dir,
                            codebase_files.join(", ")
                        ));
                    }

                    "Empty directory. (NOTE: You are inside the mission workspace sandbox. If you are looking for Tadpole OS source files, use 'read_codebase_file' or 'grep_search' with repository paths like 'src/' or 'server-rs/src/'.)".to_string()
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
        let filename = Self::resolve_path_arg(ctx, fc, false)?;

        tracing::info!(
            "🗑️ [Workspace] Agent {} requesting deletion of: {}",
            ctx.identity.agent_id,
            filename
        );
        self.broadcast_sys(
            &format!(
                "🗑️ Oversight: {} wants to DELETE {}. Extreme caution required.",
                ctx.identity.name, filename
            ),
            "warning",
            Some(ctx.mission_state.mission_id.clone()),
        );

        let approved = self
            .submit_oversight(
                crate::agent::types::ToolCallAudit {
                    id: uuid::Uuid::new_v4().to_string(),
                    agent_id: ctx.identity.agent_id.clone(),
                    mission_id: Some(ctx.mission_state.mission_id.clone()),
                    skill: "delete_file".to_string(),
                    params: fc.args.clone(),
                    department: ctx.identity.department.clone(),
                    description: format!("Deleting {} from the workspace.", filename),
                    timestamp: chrono::Utc::now().to_rfc3339(),
                },
                Some(ctx.mission_state.mission_id.clone()),
            )
            .await
            .map_err(ToolExecutionError::AppError)?;

        if approved {
            let target_abs = ctx.env.workspace_root.join(&filename);
            if let Err(e) = crate::services::cas::capture_pre_mutation(
                &self.state.resources.pool,
                &ctx.env.workspace_root,
                &target_abs,
                Some(&ctx.mission_state.mission_id),
                Some(&ctx.identity.agent_id),
            )
            .await
            {
                tracing::error!(
                    "❌ [CAS] Pre-mutation capture failed for deletion of '{}': {}",
                    filename,
                    e
                );
            }

            let adapter = &ctx.env.fs_adapter;
            match adapter.delete_file(&filename).await {
                Ok(_) => {
                    self.broadcast_sys(
                        &format!("🗑️ Workspace: {} deleted {}", ctx.identity.name, filename),
                        "success",
                        Some(ctx.mission_state.mission_id.clone()),
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
        let filename = Self::resolve_path_arg(ctx, fc, false)?;
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
            &ctx.env.workspace_root,
            &filename,
            version_num,
        )
        .await
        {
            Ok(summary) => {
                self.broadcast_sys(
                    &format!(
                        "🔄 Workspace: {} restored {} to v{}",
                        ctx.identity.name, filename, version_num
                    ),
                    "success",
                    Some(ctx.mission_state.mission_id.clone()),
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
        let filename = Self::resolve_path_arg(ctx, fc, false)?;
        if filename.is_empty() {
            return Ok("(HISTORY FAILED: 'filename' parameter is required)".to_string());
        }

        match crate::services::cas::get_file_history(
            &self.state.resources.pool,
            &ctx.env.workspace_root,
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
            ctx.identity.agent_id
        );
        self.broadcast_sys(
            &format!(
                "📁 Oversight: {} wants to archive to vault. Review required.",
                ctx.identity.name
            ),
            "warning",
            Some(ctx.mission_state.mission_id.clone()),
        );

        let approved = self
            .submit_oversight(
                crate::agent::types::ToolCallAudit {
                    id: uuid::Uuid::new_v4().to_string(),
                    agent_id: ctx.identity.agent_id.clone(),
                    mission_id: Some(ctx.mission_state.mission_id.clone()),
                    skill: "archive_to_vault".to_string(),
                    params: fc.args.clone(),
                    department: ctx.identity.department.clone(),
                    description: "Archiving data to the central vault for persistence.".to_string(),
                    timestamp: chrono::Utc::now().to_rfc3339(),
                },
                Some(ctx.mission_state.mission_id.clone()),
            )
            .await
            .map_err(ToolExecutionError::AppError)?;

        if approved {
            let vault_dir = ctx.env.workspace_root.join("vault");
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
            .or_else(|| fc.args.get("query"))
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let dir = fc
            .args
            .get("dir")
            .or_else(|| fc.args.get("path"))
            .and_then(|v| v.as_str())
            .unwrap_or(".");

        if pattern.is_empty() {
            return Ok("(GREP FAILED: 'query' or 'pattern' argument is required)".to_string());
        }

        tracing::info!(
            "🔍 [Workspace] Agent {} grepping for '{}' in {}",
            ctx.identity.agent_id,
            pattern,
            dir
        );

        let adapter = &ctx.env.fs_adapter;
        let mut files = adapter.list_files(dir).await.unwrap_or_default();
        let mut is_codebase = false;

        if files.is_empty() {
            let root = &self.state.base_dir;
            let clean_dir = dir
                .trim_start_matches("./")
                .trim_start_matches(".\\")
                .replace('\\', "/");
            let target = if clean_dir == "." || clean_dir.is_empty() {
                root.clone()
            } else {
                root.join(&clean_dir)
            };
            if clean_dir != "." && target.is_dir() {
                is_codebase = true;
                let mut stack = vec![target];
                while let Some(current_dir) = stack.pop() {
                    if let Ok(mut entries) = tokio::fs::read_dir(&current_dir).await {
                        while let Ok(Some(entry)) = entries.next_entry().await {
                            let name = entry.file_name().to_string_lossy().to_string();
                            if name == "target" || name == ".git" || name == "node_modules" {
                                continue;
                            }
                            if let Ok(ft) = entry.file_type().await {
                                if ft.is_dir() {
                                    if stack.len() < 5 {
                                        stack.push(entry.path());
                                    }
                                } else if let Ok(rel) = entry.path().strip_prefix(root) {
                                    files.push(rel.to_string_lossy().to_string());
                                }
                            }
                        }
                    }
                }
            }
        }

        let mut results = Vec::new();
        for file in files {
            let path = if is_codebase || dir == "." {
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
                let content_opt = if is_codebase {
                    let root = &self.state.base_dir;
                    let full_path = root.join(&path);
                    if let Ok(meta) = tokio::fs::metadata(&full_path).await {
                        if meta.len() <= GREP_MAX_FILE_SIZE {
                            tokio::fs::read_to_string(&full_path).await.ok()
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                } else {
                    let is_safe_size = adapter
                        .get_file_size(&path)
                        .await
                        .map(|sz| sz <= GREP_MAX_FILE_SIZE)
                        .unwrap_or(false);
                    if is_safe_size {
                        adapter.read_file(&path).await.ok()
                    } else {
                        None
                    }
                };

                if let Some(content) = content_opt {
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
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Ensures the hardcoded safety constants remain within sane bounds.
    /// Regression guard: prevents accidental increase of GREP_MAX_FILE_SIZE > 10MB
    /// or READ_TRUNCATE_LIMIT that would cause OOM under swarm load.
    #[test]
    fn test_fs_tool_constants_within_bounds() {
        // Read truncation must stay <= 32k chars to prevent context blowout
        const {
            assert!(
                READ_TRUNCATE_LIMIT <= 32_000,
                "READ_TRUNCATE_LIMIT is too high"
            );
        }
        // Grep file size cap must stay below 10MB
        const {
            assert!(
                GREP_MAX_FILE_SIZE <= 10 * 1024 * 1024,
                "GREP_MAX_FILE_SIZE is too high"
            );
        }
        // Grep result cap must stay <= 50 to prevent context blowout
        const {
            assert!(GREP_MAX_RESULTS <= 50, "GREP_MAX_RESULTS is too high");
        }
        // Breadcrumb cap must stay bounded
        const {
            assert!(MAX_BREADCRUMBS <= 50, "MAX_BREADCRUMBS is too high");
        }
    }

    #[test]
    fn test_extract_path_arg_alias_resolution() {
        let tmp = tempfile::tempdir().unwrap();
        for alias in ["filename", "file_name", "file", "path"] {
            assert_eq!(
                resolve_file_path(&serde_json::json!({alias: "foo.rs"}), tmp.path(), &[]).unwrap(),
                "foo.rs"
            );
        }
        assert!(resolve_file_path(
            &serde_json::json!({"filename":"foo.rs", "path":"bar.rs"}),
            tmp.path(),
            &[]
        )
        .is_err());
        assert_eq!(
            resolve_file_path(&serde_json::json!({"content":"ignored"}), tmp.path(), &[]).unwrap(),
            ""
        );
    }

    #[test]
    fn test_breadcrumb_target_is_authorized_before_execution() {
        use super::super::tools::{
            capability::{Permission, ZeroTrustGuard},
            policy::resolve_required_permission,
        };
        let tmp = tempfile::tempdir().unwrap();
        std::fs::create_dir(tmp.path().join("restricted")).unwrap();
        std::fs::write(tmp.path().join("restricted/target.txt"), "original").unwrap();
        let raw = serde_json::json!({"filename":"target.txt"});
        let resolved =
            resolve_file_path(&raw, tmp.path(), &["restricted/target.txt".to_string()]).unwrap();
        assert!(std::path::Path::new(&resolved).ends_with("restricted/target.txt"));
        let args = serde_json::json!({"filename":resolved,"path":resolved});
        let permission =
            resolve_required_permission("write_file", &args, tmp.path(), true).unwrap();
        let token = ZeroTrustGuard::mint_token(
            "1",
            "mission",
            crate::agent::types::RoleAuthorityLevel::Specialist,
            Some(&["target.txt".to_string()]),
            None,
            tmp.path(),
        );
        assert!(!token.verify(&permission));
        assert!(matches!(permission, Permission::FileWrite(_)));
        assert_eq!(
            resolve_file_path(&args, tmp.path(), &["other/target.txt".to_string()]).unwrap(),
            resolved
        );
        assert_eq!(
            std::fs::read_to_string(tmp.path().join("restricted/target.txt")).unwrap(),
            "original"
        );
    }
}

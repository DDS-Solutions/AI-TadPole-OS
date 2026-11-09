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

pub mod grep;
pub mod vault_ops;

pub use grep::{GREP_MAX_FILE_SIZE, GREP_MAX_RESULTS};

use super::{AgentRunner, RunContext};
use crate::agent::runner::tools::error::ToolExecutionError;

pub const MAX_BREADCRUMBS: usize = 10;
pub const READ_TRUNCATE_LIMIT: usize = 8000;

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

async fn scan_codebase_listing(root: &std::path::Path, dir: &str) -> Vec<String> {
    let clean_dir = dir
        .trim_start_matches("./")
        .trim_start_matches(".\\")
        .replace('\\', "/");
    if clean_dir == "." || clean_dir.is_empty() {
        return Vec::new();
    }
    let codebase_target = root.join(&clean_dir);
    if !codebase_target.is_dir() {
        return Vec::new();
    }
    let Ok(mut entries) = tokio::fs::read_dir(&codebase_target).await else {
        return Vec::new();
    };
    let mut files = Vec::new();
    while let Ok(Some(entry)) = entries.next_entry().await {
        let name = entry.file_name().to_string_lossy().to_string();
        if name == "target" || name == ".git" || name == "node_modules" {
            continue;
        }
        let is_dir = entry.file_type().await.map(|t| t.is_dir()).unwrap_or(false);
        files.push(if is_dir { format!("{}/", name) } else { name });
    }
    files.sort();
    files
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
            let Ok(rel) = path_obj.strip_prefix(&ctx.env.workspace_root) else {
                return;
            };
            rel.to_string_lossy().replace('\\', "/")
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

    async fn try_codebase_fallback(
        &self,
        ctx: &RunContext,
        fc: &crate::agent::types::ToolCall,
        filename: &str,
    ) -> Option<Result<String, ToolExecutionError>> {
        let root = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
        let is_in_repo =
            root.join(filename).is_file() || root.join(format!("server-rs/{}", filename)).is_file();

        if !is_in_repo {
            return None;
        }

        tracing::info!(
            "🌉 [Bridge] Agent {} requested codebase file '{}' via read_file — auto-delegating to read_codebase_file",
            ctx.identity.agent_id,
            filename
        );
        let mut codebase_fc = fc.clone();
        if let Some(obj) = codebase_fc.args.as_object_mut() {
            obj.insert(
                "path".to_string(),
                serde_json::Value::String(filename.to_string()),
            );
        } else {
            codebase_fc.args = serde_json::json!({ "path": filename });
        }
        Some(self.handle_read_codebase_file(ctx, &codebase_fc).await)
    }

    fn slice_read_content(
        content: &str,
        fc: &crate::agent::types::ToolCall,
        filename: &str,
    ) -> (String, String) {
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

        let total_lines = content.lines().count();
        let start_idx = (start_line.max(1) - 1).min(total_lines);
        let end_idx = if end_line < 0 {
            total_lines
        } else {
            (end_line as usize).min(total_lines)
        };

        if start_idx < end_idx {
            let slice_len = end_idx - start_idx;
            let sliced_lines: Vec<&str> = content.lines().skip(start_idx).take(slice_len).collect();
            let header = format!(
                "(FILE CONTENT OF {} - Lines {} to {} of {}):\n\n",
                filename,
                start_idx + 1,
                end_idx,
                total_lines
            );
            (sliced_lines.join("\n"), header)
        } else {
            (
                "".to_string(),
                format!("(FILE CONTENT OF {} - Empty range):\n\n", filename),
            )
        }
    }

    async fn handle_read_failure(
        &self,
        ctx: &RunContext,
        fc: &crate::agent::types::ToolCall,
        filename: &str,
        err: impl std::fmt::Display,
    ) -> Result<String, ToolExecutionError> {
        if let Some(res) = self.try_codebase_fallback(ctx, fc, filename).await {
            return res;
        }
        let note =
            "Note: If this is a project repository file, invoke 'read_codebase_file' with path";
        Ok(format!(
            "(READ FAILED in workspace sandbox: {err}. {note} '{filename}'.)"
        ))
    }

    /// Handles `read_file`: reads content from the mission workspace.
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

        let content = match ctx.env.fs_adapter.read_file(&final_filename).await {
            Ok(c) => c,
            Err(e) => return self.handle_read_failure(ctx, fc, &final_filename, e).await,
        };

        Self::record_fs_breadcrumb(ctx, &final_filename);
        let (sliced_content, header) = Self::slice_read_content(&content, fc, &final_filename);
        let truncated = self.safe_truncate(&sliced_content, READ_TRUNCATE_LIMIT);
        Ok(format!("{}{}", header, truncated))
    }

    async fn acquire_and_record_write_lease(
        &self,
        ctx: &RunContext,
        filename: &str,
        safe_target: &std::path::Path,
    ) -> Result<(), String> {
        if let Err(e) = self
            .state
            .security
            .conflict_manager
            .acquire_lease(safe_target.to_path_buf(), ctx.identity.agent_id.clone())
        {
            tracing::warn!(
                "⚠️ [ConflictManager] Write lease denied for {}: {}",
                filename,
                e
            );
            return Err(format!(
                "(WRITE FAILED: Concurrent file write collision: {})",
                e
            ));
        }

        if let Err(e) = crate::services::cas::capture_pre_mutation(
            &self.state.resources.pool,
            &ctx.env.workspace_root,
            safe_target,
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
        Ok(())
    }

    /// Handles `write_file`: writes content to the mission workspace.
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

        if let Err(msg) = self
            .acquire_and_record_write_lease(ctx, &filename, &safe_target)
            .await
        {
            return Ok(msg);
        }

        let write_result = adapter.write_file(&filename, content).await;
        self.state
            .security
            .conflict_manager
            .release_lease(&safe_target);

        match write_result {
            Ok(_) => {
                Self::record_fs_breadcrumb(ctx, &filename);
                let msg = format!("✍️ Workspace: {} wrote to {}", ctx.identity.name, filename);
                self.broadcast_sys(&msg, "success", Some(ctx.mission_state.mission_id.clone()));
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

        let files = match ctx.env.fs_adapter.list_files(dir).await {
            Ok(f) => f,
            Err(e) => return Ok(format!("(LIST FAILED: {})", e)),
        };

        if !files.is_empty() {
            return Ok(format!("(FILES IN {}): {}", dir, files.join(", ")));
        }

        let codebase_files = scan_codebase_listing(&self.state.base_dir, dir).await;
        if !codebase_files.is_empty() {
            return Ok(format!(
                "(FILES IN {} [from Codebase Root]): {}",
                dir,
                codebase_files.join(", ")
            ));
        }

        let note = "Empty directory. (NOTE: You are inside the mission workspace sandbox. If you are looking for Tadpole OS source files, use 'read_codebase_file' or 'grep_search' with repository paths like 'src/' or 'server-rs/src/'.)";
        Ok(format!("(FILES IN {}): {}", dir, note))
    }

    async fn request_delete_oversight(
        &self,
        ctx: &RunContext,
        fc: &crate::agent::types::ToolCall,
        filename: &str,
    ) -> Result<bool, ToolExecutionError> {
        self.broadcast_sys(
            &format!(
                "🗑️ Oversight: {} wants to DELETE {}. Extreme caution required.",
                ctx.identity.name, filename
            ),
            "warning",
            Some(ctx.mission_state.mission_id.clone()),
        );
        self.submit_oversight(
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
        .map_err(ToolExecutionError::AppError)
    }

    async fn execute_file_deletion(
        &self,
        ctx: &RunContext,
        filename: &str,
    ) -> Result<String, ToolExecutionError> {
        let target_abs = ctx.env.workspace_root.join(filename);
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

        match ctx.env.fs_adapter.delete_file(filename).await {
            Ok(_) => {
                let msg = format!("🗑️ Workspace: {} deleted {}", ctx.identity.name, filename);
                self.broadcast_sys(&msg, "success", Some(ctx.mission_state.mission_id.clone()));
                Ok(format!("(Successfully deleted {})", filename))
            }
            Err(e) => Ok(format!("(DELETE FAILED: {})", e)),
        }
    }

    /// Handles `delete_file`: removes a file or directory after oversight.
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

        if !self.request_delete_oversight(ctx, fc, &filename).await? {
            return Ok("(Delete REJECTED by Oversight)".to_string());
        }

        self.execute_file_deletion(ctx, &filename).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fs_tool_constants_within_bounds() {
        const {
            assert!(
                READ_TRUNCATE_LIMIT <= 32_000,
                "READ_TRUNCATE_LIMIT is too high"
            );
        }
        const {
            assert!(
                GREP_MAX_FILE_SIZE <= 10 * 1024 * 1024,
                "GREP_MAX_FILE_SIZE is too high"
            );
        }
        const {
            assert!(GREP_MAX_RESULTS <= 50, "GREP_MAX_RESULTS is too high");
        }
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

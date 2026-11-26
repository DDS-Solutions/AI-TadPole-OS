//! @docs ARCHITECTURE:Registry
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Agent Runner / codebase access
//!
//! ### AI Assist Note
//! - Keep repository path authorization separate from symbol parsing and graph queries.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: ToolExecutionError, path validation failures
//! - **Telemetry Targets**: [Context]

use super::*;

impl AgentRunner {
    /// Handles `read_codebase_file`: allows reading files from the project root.
    ///
    /// ### 🛡️ Security Filter (Sovereign)
    /// - **Oversight**: Requires manual approval to access files outside the
    ///   mission sandbox.
    /// - **Credential Filter**: Blocks any files containing "key", "token", or
    ///   ".env" to prevent data leakage.
    /// - **Breadcrumb Resolution**: If a relative path is ambiguous, uses
    ///   the `RunContext` history to resolve the absolute path.
    pub(crate) async fn handle_read_codebase_file(
        &self,
        ctx: &RunContext,
        fc: &crate::agent::types::ToolCall,
    ) -> Result<String, ToolExecutionError> {
        let path_str = require_path(ctx, &fc.args, "read_codebase_file")?;

        tracing::info!(
            "🔍 [Sovereignty] Agent {} requesting codebase read: {}",
            ctx.identity.agent_id,
            path_str
        );

        let start_line = fc
            .args
            .get("start_line")
            .or_else(|| fc.args.get("startLine"))
            .and_then(|v| v.as_i64())
            .filter(|line| *line > 0)
            .and_then(|line| usize::try_from(line).ok())
            .unwrap_or(1);

        let end_line = fc
            .args
            .get("end_line")
            .or_else(|| fc.args.get("endLine"))
            .and_then(|v| v.as_i64())
            .unwrap_or(-1) as isize;

        let final_path = match self.require_path_safety(ctx, &path_str).await {
            Ok(p) => p,
            Err(e) => return Err(ToolExecutionError::SecurityBlocked(e)),
        };

        match self
            .read_codebase_file_helper(ctx, &final_path, &path_str)
            .await
        {
            Ok(content) => {
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
                    let sliced_lines: Vec<&str> =
                        content.lines().skip(start_idx).take(slice_len).collect();
                    (
                        sliced_lines.join("\n"),
                        format!(
                            "(FILE CONTENT OF {} - Lines {} to {} of {}):\n\n",
                            path_str,
                            start_idx + 1,
                            end_idx,
                            total_lines
                        ),
                    )
                } else {
                    (
                        "".to_string(),
                        format!("(FILE CONTENT OF {} - Empty range):\n\n", path_str),
                    )
                };

                let truncated = self.safe_truncate(&sliced_content, CODEBASE_READ_MAX_CHARS);
                Ok(format!("{}{}", header, truncated))
            }
            Err(e) => Err(ToolExecutionError::ExecutionFailed(format!(
                "Codebase read failed for {path_str}: {e}"
            ))),
        }
    }

    fn codebase_path_is_sensitive(&self, ctx: &RunContext, path: &std::path::Path) -> bool {
        let normalized = path.to_string_lossy().to_lowercase().replace('\\', "/");
        if [".env", "key", "token", "credential", "secret", "private"]
            .iter()
            .any(|pattern| normalized.contains(pattern))
        {
            return true;
        }

        // Only the current canonical mission directory may use a relative check.
        // Sibling workspaces and central data/state keep the global data restriction.
        if let (Ok(base), Ok(workspace)) = (
            std::fs::canonicalize(&self.state.base_dir),
            std::fs::canonicalize(&ctx.env.workspace_root),
        ) {
            let parent = base.join("data/workspaces");
            if workspace.parent() == Some(parent.as_path()) {
                if let Ok(relative) = path.strip_prefix(&workspace) {
                    return crate::routes::system::is_sensitive_file(
                        &relative.to_string_lossy().replace('\\', "/"),
                    );
                }
            }
        }
        crate::routes::system::is_sensitive_file(&normalized)
    }

    /// Centralized safety path validator and resolver helper
    pub(crate) async fn require_path_safety(
        &self,
        ctx: &RunContext,
        path_str: &str,
    ) -> Result<crate::utils::security::SafePath, String> {
        let sensitive_patterns = [".env", "key", "token", "credential", "secret", "private"];
        if sensitive_patterns
            .iter()
            .any(|p| path_str.to_lowercase().contains(p))
        {
            return Err(format!(
                "(SECURITY BLOCKED: Access to sensitive file '{}' is prohibited.)",
                path_str
            ));
        }

        let root = &self.state.base_dir;
        let root_clean_buf = if let Some(s) = root.to_str() {
            std::path::PathBuf::from(s.strip_prefix(r"\\?\").unwrap_or(s))
        } else {
            root.clone()
        };
        let root_clean = root_clean_buf.as_path();

        let stripped_prefix = path_str.strip_prefix(r"\\?\").unwrap_or(path_str);
        let mut clean_path_str = stripped_prefix.to_string();
        let path = std::path::Path::new(stripped_prefix);
        if path.is_absolute() {
            if let Ok(relative) = path.strip_prefix(root_clean) {
                clean_path_str = relative.to_string_lossy().to_string();
            } else if let Ok(relative) = path.strip_prefix(root) {
                clean_path_str = relative.to_string_lossy().to_string();
            }
        }

        // If an agent prepended its cluster workspace sandbox path (e.g. data/workspaces/cl-chain-a/...)
        // to a codebase target, sanitize and resolve relative to repository root.
        let norm_clean = clean_path_str.replace('\\', "/");
        if let Some(ws_idx) = norm_clean.find("data/workspaces/") {
            let after_ws = &norm_clean[ws_idx + "data/workspaces/".len()..];
            if let Some(slash_idx) = after_ws.find('/') {
                let candidate_subpath = &after_ws[slash_idx + 1..];
                if !candidate_subpath.is_empty() {
                    let candidate_in_root = root_clean.join(candidate_subpath);
                    let orig_in_root = root_clean.join(&clean_path_str);
                    if !orig_in_root.exists() && candidate_in_root.exists() {
                        tracing::info!(
                            "🧩 [Context] Auto-sanitized workspace-prefixed codebase path '{}' -> '{}'",
                            clean_path_str,
                            candidate_subpath
                        );
                        clean_path_str = candidate_subpath.to_string();
                    }
                }
            }
        }

        let target_path = match crate::utils::security::validate_path(root, &clean_path_str) {
            Ok(p) => p,
            Err(e) => {
                return Err(format!("(SECURITY BLOCKED: {})", e));
            }
        };

        // Re-check canonical resolved target against sensitive file patterns to prevent symlink bypass
        if self.codebase_path_is_sensitive(ctx, target_path.as_path()) {
            return Err(format!(
                "(SECURITY BLOCKED: Access to sensitive file '{}' is prohibited.)",
                path_str
            ));
        }

        let mut final_path = target_path.clone();
        if tokio::fs::metadata(&final_path).await.is_err() {
            let candidate_opt = {
                let breadcrumbs = ctx.last_accessed_files.lock();
                breadcrumbs
                    .iter()
                    .find(|p| {
                        let p_path = std::path::Path::new(p);
                        let target = std::path::Path::new(path_str);
                        p_path.ends_with(target)
                    })
                    .cloned()
            };

            if let Some(resolved) = candidate_opt {
                let resolved_path = std::path::Path::new(&resolved);
                let rel_candidate = if resolved_path.is_absolute() {
                    match resolved_path.strip_prefix(root) {
                        Ok(rel) => rel.to_string_lossy().to_string(),
                        Err(_) => String::new(),
                    }
                } else {
                    resolved
                };

                if !rel_candidate.is_empty() {
                    if let Ok(safe_resolved) =
                        crate::utils::security::validate_path(root, &rel_candidate)
                    {
                        if tokio::fs::metadata(&safe_resolved).await.is_ok() {
                            tracing::info!(
                                "🧩 [Context] Resolved ambiguous codebase path '{}' to '{}' via breadcrumbs",
                                path_str,
                                safe_resolved.as_path().display()
                            );
                            final_path = safe_resolved;
                        }
                    }
                }
            }
        }

        if tokio::fs::metadata(&final_path).await.is_err()
            && !path_str.contains('/')
            && !path_str.contains('\\')
        {
            let common_dirs = ["src", "src/agent", "server-rs/src", "server-rs/src/agent"];
            for dir in common_dirs {
                let rel_str = format!("{}/{}", dir, path_str);
                if let Ok(safe_alt) = crate::utils::security::validate_path(root, &rel_str) {
                    if tokio::fs::metadata(&safe_alt).await.is_ok() {
                        tracing::info!("🧩 [Context] Resolved ambiguous codebase path '{}' to '{:?}' via common-dirs", path_str, safe_alt.as_path());
                        final_path = safe_alt;
                        break;
                    }
                }
            }
        }

        // Re-check canonical resolved target against sensitive file patterns to prevent symlink bypass or fallback leaks
        if self.codebase_path_is_sensitive(ctx, final_path.as_path()) {
            return Err(format!(
                "(SECURITY BLOCKED: Access to sensitive file '{}' is prohibited.)",
                path_str
            ));
        }

        Ok(final_path)
    }

    /// Read file and record breadcrumb
    pub(crate) async fn read_codebase_file_helper(
        &self,
        ctx: &RunContext,
        final_path: &std::path::Path,
        path_str: &str,
    ) -> Result<String, String> {
        if final_path.is_dir() {
            return Err(format!(
                "Path '{}' is a directory, not a file. For directories, please use directory listing or search tools instead.",
                path_str
            ));
        }
        if self.codebase_path_is_sensitive(ctx, final_path) {
            return Err(format!(
                "(SECURITY BLOCKED: Access to sensitive file '{}' is prohibited.)",
                path_str
            ));
        }
        if let Ok(meta) = tokio::fs::metadata(final_path).await {
            const CODEBASE_MAX_READ_BYTES: u64 = 10 * 1024 * 1024;
            if meta.len() > CODEBASE_MAX_READ_BYTES {
                return Err(format!(
                    "File '{}' size ({} bytes) exceeds maximum codebase read limit ({} bytes). Specify line ranges ('start_line', 'end_line') or use 'grep_search'.",
                    path_str,
                    meta.len(),
                    CODEBASE_MAX_READ_BYTES
                ));
            }
        }

        match tokio::fs::read_to_string(final_path).await {
            Ok(content) => {
                self.record_breadcrumb(ctx, final_path, path_str);
                Ok(content)
            }
            Err(e) => Err(format!("{}", e)),
        }
    }

    /// Record path access breadcrumb
    pub(crate) fn record_breadcrumb(
        &self,
        ctx: &RunContext,
        final_path: &std::path::Path,
        path_str: &str,
    ) {
        let root = std::fs::canonicalize(&self.state.base_dir)
            .unwrap_or_else(|_| self.state.base_dir.clone());
        let is_abs = final_path.is_absolute()
            || final_path.starts_with("/")
            || (final_path.to_string_lossy().len() >= 3
                && final_path
                    .to_string_lossy()
                    .chars()
                    .next()
                    .map_or(false, |c| c.is_ascii_alphabetic())
                && final_path.to_string_lossy().chars().nth(1) == Some(':'));
        let path_to_record = if is_abs {
            if let Ok(rel) = final_path.strip_prefix(&root) {
                rel.to_string_lossy().replace('\\', "/")
            } else {
                return; // Do not record external paths outside repository root
            }
        } else {
            path_str.replace('\\', "/")
        };

        let mut breadcrumbs = ctx.last_accessed_files.lock();
        if !breadcrumbs.contains(&path_to_record) {
            breadcrumbs.push(path_to_record);
            if breadcrumbs.len() > BREADCRUMB_CAP {
                breadcrumbs.remove(0);
            }
        }
    }
}

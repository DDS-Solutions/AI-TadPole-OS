//! @docs ARCHITECTURE:Runner
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Agent Runner / fs_tools / grep
//!
//! ### AI Assist Note
//! Fast pattern search across workspace and codebase files with size and count bounds.
//!
//! ### 🔍 Debugging & Observability
//! - **Telemetry Targets**: `[Workspace]`

use super::AgentRunner;
use crate::agent::runner::tools::error::ToolExecutionError;
use crate::agent::runner::RunContext;
use std::path::Path;

pub const GREP_MAX_FILE_SIZE: u64 = 5 * 1024 * 1024;
pub const GREP_MAX_RESULTS: usize = 10;

fn is_text_extension(path: &str) -> bool {
    let ext = path.rsplit('.').next().unwrap_or("").to_lowercase();
    !path.contains('.')
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
        )
}

async fn scan_dir_entries(
    root: &Path,
    dir: &Path,
    stack: &mut Vec<std::path::PathBuf>,
    files: &mut Vec<String>,
) {
    let Ok(mut entries) = tokio::fs::read_dir(dir).await else {
        return;
    };
    while let Ok(Some(entry)) = entries.next_entry().await {
        let name = entry.file_name().to_string_lossy().to_string();
        if name == "target" || name == ".git" || name == "node_modules" {
            continue;
        }
        let Ok(ft) = entry.file_type().await else {
            continue;
        };
        if ft.is_dir() {
            if stack.len() < 5 {
                stack.push(entry.path());
            }
        } else if let Ok(rel) = entry.path().strip_prefix(root) {
            files.push(rel.to_string_lossy().to_string());
        }
    }
}

async fn crawl_codebase_subdirs(root: &Path, start_dir: &Path) -> Vec<String> {
    let mut files = Vec::new();
    let mut stack = vec![start_dir.to_path_buf()];

    while let Some(current_dir) = stack.pop() {
        scan_dir_entries(root, &current_dir, &mut stack, &mut files).await;
    }
    files
}

fn search_pattern_in_text(path: &str, content: &str, pattern: &str) -> Option<String> {
    if !content.contains(pattern) {
        return None;
    }
    let lines: Vec<String> = content
        .lines()
        .enumerate()
        .filter(|(_, line)| line.contains(pattern))
        .map(|(i, line)| format!("{}: {}", i + 1, line.trim()))
        .collect();
    if lines.is_empty() {
        None
    } else {
        Some(format!("--- {} ---\n{}", path, lines.join("\n")))
    }
}

impl AgentRunner {
    async fn read_grep_candidate(
        &self,
        ctx: &RunContext,
        path: &str,
        is_codebase: bool,
    ) -> Option<String> {
        if is_codebase {
            let root = &self.state.base_dir;
            let full_path = root.join(path);
            let meta = tokio::fs::metadata(&full_path).await.ok()?;
            if meta.len() <= GREP_MAX_FILE_SIZE {
                tokio::fs::read_to_string(&full_path).await.ok()
            } else {
                None
            }
        } else {
            let sz = ctx.env.fs_adapter.get_file_size(path).await.ok()?;
            if sz <= GREP_MAX_FILE_SIZE {
                ctx.env.fs_adapter.read_file(path).await.ok()
            } else {
                None
            }
        }
    }

    async fn resolve_grep_files(&self, ctx: &RunContext, dir: &str) -> (Vec<String>, bool) {
        let files = ctx.env.fs_adapter.list_files(dir).await.unwrap_or_default();
        if !files.is_empty() {
            return (files, false);
        }
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
            (crawl_codebase_subdirs(root, &target).await, true)
        } else {
            (Vec::new(), false)
        }
    }

    async fn search_file_for_pattern(
        &self,
        ctx: &RunContext,
        path: &str,
        is_codebase: bool,
        pattern: &str,
    ) -> Option<String> {
        if !is_text_extension(path) {
            return None;
        }
        let content = self.read_grep_candidate(ctx, path, is_codebase).await?;
        search_pattern_in_text(path, &content, pattern)
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

        let (files, is_codebase) = self.resolve_grep_files(ctx, dir).await;
        let mut results = Vec::new();

        for file in files {
            let path = if is_codebase || dir == "." {
                file
            } else {
                format!("{}/{}", dir, file)
            };

            if let Some(res) = self
                .search_file_for_pattern(ctx, &path, is_codebase, pattern)
                .await
            {
                results.push(res);
            }
            if results.len() >= GREP_MAX_RESULTS {
                break;
            }
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

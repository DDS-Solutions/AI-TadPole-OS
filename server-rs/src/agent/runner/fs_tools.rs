//! Filesystem interaction tools for agent workspace management.
//!
//! Implements secure file operations (CRUD) within the agent's sandbox.
//! All pathing is handled via the canonicalizing `FilesystemAdapter` to
//! prevent directory traversal and symlink escapes.

use super::{AgentRunner, RunContext};

impl AgentRunner {
    /// Handles `read_file`: reads content from the workspace.
    pub(crate) async fn handle_read_file(
        &self,
        ctx: &RunContext,
        fc: &crate::agent::types::GeminiFunctionCall,
        output_text: &mut String,
        _usage: &mut Option<crate::agent::types::TokenUsage>,
    ) -> anyhow::Result<()> {
        let filename = fc
            .args
            .get("filename")
            .or_else(|| fc.args.get("file_name"))
            .or_else(|| fc.args.get("file"))
            .or_else(|| fc.args.get("path"))
            .and_then(|v| v.as_str())
            .unwrap_or("");

        if filename.is_empty() {
            *output_text = "(READ FAILED: The 'filename' parameter was missing or empty. You MUST specify a valid filename.)".to_string();
            return Ok(());
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
            let breadcrumbs = ctx.last_accessed_files.lock().unwrap();
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
                let mut breadcrumbs = ctx.last_accessed_files.lock().unwrap();
                if !breadcrumbs.contains(&final_filename) {
                    breadcrumbs.push(final_filename.clone());
                    if breadcrumbs.len() > 10 {
                        breadcrumbs.remove(0);
                    }
                }

                let truncated = if content.len() > 5000 {
                    format!("{}... [TRUNCATED]", &content[..5000])
                } else {
                    content
                };
                *output_text = format!("(FILE CONTENT OF {}):\n\n{}", final_filename, truncated);
            }
            Err(e) => {
                *output_text = format!("(READ FAILED: {}) {}", e, output_text);
            }
        }
        Ok(())
    }

    /// Handles `write_file`: writes content to the workspace.
    pub(crate) async fn handle_write_file(
        &self,
        ctx: &RunContext,
        fc: &crate::agent::types::GeminiFunctionCall,
        output_text: &mut String,
    ) -> anyhow::Result<()> {
        let filename = fc
            .args
            .get("filename")
            .or_else(|| fc.args.get("file_name"))
            .or_else(|| fc.args.get("file"))
            .or_else(|| fc.args.get("path"))
            .and_then(|v| v.as_str())
            .unwrap_or("");

        if filename.is_empty() {
            *output_text = "(WRITE FAILED: The 'filename' parameter was missing or empty. You MUST specify a valid filename.)".to_string();
            return Ok(());
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
                let mut breadcrumbs = ctx.last_accessed_files.lock().unwrap();
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
                );
                *output_text = format!("(Successfully wrote to {}) {}", filename, output_text);
            }
            Err(e) => {
                *output_text = format!("(WRITE FAILED: {}) {}", e, output_text);
            }
        }
        Ok(())
    }

    /// Handles `list_files`: lists directory contents in the workspace.
    pub(crate) async fn handle_list_files(
        &self,
        ctx: &RunContext,
        fc: &crate::agent::types::GeminiFunctionCall,
        output_text: &mut String,
        _usage: &mut Option<crate::agent::types::TokenUsage>,
    ) -> anyhow::Result<()> {
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
                *output_text = format!("(FILES IN {}): {}", dir, list);
            }
            Err(e) => {
                *output_text = format!("(LIST FAILED: {}) {}", e, output_text);
            }
        }
        Ok(())
    }

    /// Handles `delete_file`: removes a file or directory after oversight.
    pub(crate) async fn handle_delete_file(
        &self,
        ctx: &RunContext,
        fc: &crate::agent::types::GeminiFunctionCall,
        output_text: &mut String,
    ) -> anyhow::Result<()> {
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
        );

        let approved = self
            .submit_oversight(
                crate::agent::types::ToolCall {
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
            .await;

        if approved {
            let adapter = &ctx.fs_adapter;
            match adapter.delete_file(filename).await {
                Ok(_) => {
                    self.broadcast_sys(
                        &format!("🗑️ Workspace: {} deleted {}", ctx.name, filename),
                        "success",
                    );
                    *output_text = format!("(Successfully deleted {}) {}", filename, output_text);
                }
                Err(e) => {
                    *output_text = format!("(DELETE FAILED: {}) {}", e, output_text);
                }
            }
        } else {
            *output_text = format!("(Delete REJECTED by Oversight) {}", output_text);
        }

        Ok(())
    }

    /// Handles `archive_to_vault`: writes data to the local Markdown vault after oversight.
    pub(crate) async fn handle_archive_to_vault(
        &self,
        ctx: &RunContext,
        fc: &crate::agent::types::GeminiFunctionCall,
        output_text: &mut String,
    ) -> anyhow::Result<()> {
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
        );

        let approved = self
            .submit_oversight(
                crate::agent::types::ToolCall {
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
            .await;

        if approved {
            let adapter =
                crate::adapter::vault::VaultAdapter::new(std::path::PathBuf::from("vault"));
            adapter.append_to_file(filename, content).await?;

            let echo = format!("**Archived to Vault ({}):**\n\n{}\n\n", filename, content);
            *output_text = format!("{}{}", echo, output_text);
        } else {
            *output_text = format!("(Archive REJECTED by Oversight) {}", output_text);
        }
        Ok(())
    }
}

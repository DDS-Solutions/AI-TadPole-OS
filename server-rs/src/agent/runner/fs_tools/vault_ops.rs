//! @docs ARCHITECTURE:Runner
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Agent Runner / fs_tools / vault_ops
//!
//! ### AI Assist Note
//! Manages Markdown vault persistence and CAS versioning/restore workflows.
//!
//! ### 🔍 Debugging & Observability
//! - **Telemetry Targets**: `[Surface]`

use super::AgentRunner;
use crate::agent::runner::tools::error::ToolExecutionError;
use crate::agent::runner::RunContext;

fn format_revision_line(r: &crate::services::cas::RevisionSummary) -> String {
    let short_hash = &r.hash[..8.min(r.hash.len())];
    let agent = r.agent_id.as_deref().unwrap_or("unknown");
    format!(
        "- v{} | {} | {} bytes | agent: {}",
        r.version_num, short_hash, r.size_bytes, agent
    )
}

impl AgentRunner {
    async fn restore_and_broadcast(
        &self,
        ctx: &RunContext,
        filename: &str,
        version_num: i64,
    ) -> Result<String, String> {
        let summary = crate::services::cas::restore_file_version(
            &self.state.resources.pool,
            &ctx.env.workspace_root,
            filename,
            version_num,
        )
        .await
        .map_err(|e| e.to_string())?;

        let short_hash = &summary.hash[..8.min(summary.hash.len())];
        let msg = format!(
            "🔄 Workspace: {} restored {} to v{}",
            ctx.identity.name, filename, version_num
        );
        self.broadcast_sys(&msg, "success", Some(ctx.mission_state.mission_id.clone()));
        Ok(format!(
            "(RESTORE SUCCESS: File '{}' restored to version v{} [hash: {}])",
            filename, version_num, short_hash
        ))
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

        match self
            .restore_and_broadcast(ctx, &filename, version_num)
            .await
        {
            Ok(msg) => Ok(msg),
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

        let history = crate::services::cas::get_file_history(
            &self.state.resources.pool,
            &ctx.env.workspace_root,
            &filename,
        )
        .await
        .map_err(|e| {
            ToolExecutionError::AppError(crate::error::AppError::Anyhow(anyhow::anyhow!(e)))
        })?;

        if history.is_empty() {
            return Ok(format!("(No revision history recorded for '{}')", filename));
        }

        let lines: Vec<String> = history.iter().map(format_revision_line).collect();
        Ok(format!(
            "(REVISION HISTORY FOR {}):\n{}",
            filename,
            lines.join("\n")
        ))
    }

    async fn request_archive_oversight(
        &self,
        ctx: &RunContext,
        fc: &crate::agent::types::ToolCall,
    ) -> Result<bool, ToolExecutionError> {
        self.broadcast_sys(
            &format!(
                "📁 Oversight: {} wants to archive to vault. Review required.",
                ctx.identity.name
            ),
            "warning",
            Some(ctx.mission_state.mission_id.clone()),
        );

        self.submit_oversight(
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
        .map_err(ToolExecutionError::AppError)
    }

    /// Handles `archive_to_vault`: writes data to the local Markdown vault.
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

        if !self.request_archive_oversight(ctx, fc).await? {
            return Ok("(Archive REJECTED by Oversight)".to_string());
        }

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
    }
}

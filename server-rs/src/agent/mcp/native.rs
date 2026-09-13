//! @docs ARCHITECTURE:Registry:Mcp
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Agent Runner / Native MCP Tools
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Native tool definitions conforming to `ToolHandler` and JSON schema contracts.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: `AppError::BadRequest`, `AppError::Forbidden`, `AppError::NotFound`
//! - **Telemetry Targets**: none declared
//! - **Witness Tests**: `agent::mcp_tests::tests::*`

use super::registry::ToolHandler;
use super::types::{McpResult, McpToolHub, McpToolStats};
use crate::error::{AppError, InfrastructureErrorKind, ProviderId};
use crate::utils::parser::SymbolExtractor;
use server_rs_macros::agent_tool;
use std::sync::Arc;
use std::time::Duration;

pub const DEFAULT_INTEGRITY_CHECK_TIMEOUT: Duration = Duration::from_secs(300);

// --- Native Tool Handlers ---

#[agent_tool]
pub async fn recruit_specialist(
    agent_id: String,
    task_description: String,
    _workspace_root: std::path::PathBuf,
) -> Result<McpResult, AppError> {
    Ok(McpResult::SystemDelegate(
        "recruit_specialist".to_string(),
        serde_json::json!({
            "agent_id": agent_id,
            "task_description": task_description
        }),
    ))
}

#[agent_tool]
pub async fn list_file_symbols(
    path: String,
    workspace_root: std::path::PathBuf,
) -> Result<McpResult, AppError> {
    let full_path = crate::utils::security::validate_path(&workspace_root, &path)
        .map_err(|e| AppError::Forbidden(e.to_string()))?;
    if full_path.is_dir() {
        return Err(AppError::BadRequest(format!(
            "Path '{}' is a directory, not a file. list_file_symbols only accepts file paths.",
            path
        )));
    }
    let content = tokio::fs::read_to_string(&full_path)
        .await
        .map_err(AppError::Io)?;
    let mut extractor = SymbolExtractor::new();
    let symbols = extractor.extract_symbols(&full_path, &content);
    let outline: Vec<String> = symbols
        .iter()
        .map(|s| format!("{} {} -> {}", s.kind, s.name, s.signature))
        .collect();
    Ok(McpResult::Raw(outline.join("\n")))
}

#[agent_tool]
pub async fn get_symbol_body(
    path: String,
    symbol_name: String,
    workspace_root: std::path::PathBuf,
) -> Result<McpResult, AppError> {
    let full_path = crate::utils::security::validate_path(&workspace_root, &path)
        .map_err(|e| AppError::Forbidden(e.to_string()))?;
    if full_path.is_dir() {
        return Err(AppError::BadRequest(format!(
            "Path '{}' is a directory, not a file. get_symbol_body only accepts file paths.",
            path
        )));
    }
    let content = tokio::fs::read_to_string(&full_path)
        .await
        .map_err(AppError::Io)?;
    let mut extractor = SymbolExtractor::new();
    let symbols = extractor.extract_symbols(&full_path, &content);
    if let Some(symbol) = symbols.into_iter().find(|s| s.name == symbol_name) {
        Ok(McpResult::Raw(symbol.body))
    } else {
        Err(AppError::NotFound(format!(
            "Symbol '{}' not found",
            symbol_name
        )))
    }
}

#[agent_tool]
pub async fn run_integrity_check(
    workspace_root: std::path::PathBuf,
) -> Result<McpResult, AppError> {
    let mut cmd = tokio::process::Command::new("python");
    cmd.arg("execution/self_audit_tool.py");
    cmd.current_dir(workspace_root);

    let output = tokio::time::timeout(DEFAULT_INTEGRITY_CHECK_TIMEOUT, cmd.output())
        .await
        .map_err(|_| AppError::InfrastructureError {
            provider_id: ProviderId::System,
            kind: InfrastructureErrorKind::Timeout,
            detail: format!(
                "Integrity check timed out after {:?}",
                DEFAULT_INTEGRITY_CHECK_TIMEOUT
            ),
            help_link: None,
        })?
        .map_err(AppError::Io)?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();

    if output.status.success() {
        Ok(McpResult::Raw(stdout))
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        Err(AppError::InfrastructureError {
            provider_id: ProviderId::System,
            kind: InfrastructureErrorKind::ApiError,
            detail: format!("Integrity Audit Failed: {}\n{}", stdout, stderr),
            help_link: None,
        })
    }
}

pub struct InspectEngineHealthHandler {
    pub stats: Arc<dashmap::DashMap<String, McpToolStats>>,
}

#[async_trait::async_trait]
impl ToolHandler for InspectEngineHealthHandler {
    async fn execute(
        &self,
        _args: serde_json::Value,
        _workspace_root: std::path::PathBuf,
    ) -> Result<McpResult, AppError> {
        let stats_vec: Vec<serde_json::Value> = self
            .stats
            .iter()
            .map(|kv| {
                let name = kv.key();
                let stats = kv.value();
                serde_json::json!({
                    "tool": name,
                    "invocations": stats.invocations,
                    "success_rate": if stats.invocations > 0 { stats.success_count as f64 / stats.invocations as f64 } else { 0.0 },
                    "avg_latency_ms": stats.avg_latency_ms
                })
            })
            .collect();

        Ok(McpResult::Raw(
            serde_json::to_string_pretty(&stats_vec)
                .map_err(|e| AppError::InternalServerError(e.to_string()))?,
        ))
    }

    fn metadata(&self) -> McpToolHub {
        McpToolHub {
            name: "inspect_engine_health".to_string(),
            description: "Retrieves real-time execution statistics for all registered MCP tools."
                .to_string(),
            input_schema: serde_json::json!({}),
            source: "native".to_string(),
            stats: McpToolStats::default(),
            category: "introspection".to_string(),
        }
    }
}

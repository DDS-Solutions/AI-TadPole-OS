//! @docs ARCHITECTURE:Networking
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / HTTP Routes / mcp
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Type-safe state handling and bounded execution without unhandled panics.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: `AppError::Forbidden`, `AppError::InternalServerError`
//! - **Telemetry Targets**: `[MCP]`
//! - **Witness Tests**: `mcp_test::tests::*`

use crate::agent::mcp::McpResult;
use crate::error::AppError;
use crate::security::permissions::PermissionMode;
use crate::state::AppState;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde_json::json;
use std::sync::Arc;
use std::time::Duration;

pub const MCP_TOOL_TIMEOUT_SECS: u64 = 30;

/// GET /api/mcp/tools
/// Lists all available MCP tools across system, legacy, and external servers.
#[tracing::instrument(skip(state), name = "mcp::list_tools")]
pub async fn list_mcp_tools(
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, AppError> {
    let all_agent_skills: Vec<String> = state
        .registry
        .skills
        .skills
        .iter()
        .map(|kv| kv.key().clone())
        .collect();

    let tools = state
        .registry
        .mcp_host
        .list_tools(&all_agent_skills, &state.registry.skills.skills)
        .await;

    Ok((StatusCode::OK, Json(tools)))
}

/// POST /api/mcp/tools/:name/execute
/// Executes an MCP tool directly via the API (Governance/Debugging).
#[tracing::instrument(skip(state, arguments), name = "mcp::execute_tool")]
pub async fn execute_mcp_tool(
    Path(name): Path<String>,
    State(state): State<Arc<AppState>>,
    Json(arguments): Json<serde_json::Value>,
) -> Result<impl IntoResponse, AppError> {
    let workspace_root = state.base_dir.join("data/workspaces/api_debug");
    if !workspace_root.exists() {
        tokio::fs::create_dir_all(&workspace_root)
            .await
            .map_err(|e| {
                AppError::InternalServerError(format!(
                    "Failed to create API debug workspace: {}",
                    e
                ))
            })?;
    }

    // 1. Governance: Check Privacy Shield
    if state
        .governance
        .privacy_mode
        .load(std::sync::atomic::Ordering::SeqCst)
    {
        // High-risk: block known external or potentially leaky tools in privacy mode
        if name.contains("google") || name.contains("openai") || name.contains("anthropic") {
            state.emit_event(json!({
                "type": "mcp:tool_denied",
                "tool": name,
                "reason": "privacy_shield_active",
                "timestamp": chrono::Utc::now().to_rfc3339()
            }));

            return Err(AppError::Forbidden(format!(
                "Tool '{}' is blocked while Privacy Shield (Local-First) is active.",
                name
            )));
        }
    }

    // 2. Governance: Check Permission Policy
    let mode = state
        .security
        .permission_policy
        .get_mode(None, None, &name)
        .await;
    match mode {
        PermissionMode::Deny => {
            state.emit_event(json!({
                "type": "mcp:tool_denied",
                "tool": name,
                "reason": "policy_denied",
                "timestamp": chrono::Utc::now().to_rfc3339()
            }));

            return Err(AppError::Forbidden(format!(
                "Execution of tool '{}' is explicitly denied by governance policy.",
                name
            )));
        }
        PermissionMode::Prompt => {
            state.emit_event(json!({
                "type": "mcp:tool_denied",
                "tool": name,
                "reason": "prompt_mode_requires_allow",
                "timestamp": chrono::Utc::now().to_rfc3339()
            }));

            return Err(AppError::Forbidden(format!(
                "Tool '{}' requires explicit 'Allow' status in Governance settings for direct API execution.",
                name
            )));
        }
        PermissionMode::Allow => {}
    }

    // Execute with timeout
    let call_future = state.registry.mcp_host.call_tool(
        &name,
        arguments,
        workspace_root,
        &state.registry.skills.skills,
    );

    let execution_result =
        tokio::time::timeout(Duration::from_secs(MCP_TOOL_TIMEOUT_SECS), call_future).await;

    match execution_result {
        Ok(Ok(McpResult::Raw(output))) => {
            let redacted_output = state.security.secret_redactor.redact(&output);
            state.emit_event(json!({
                "type": "mcp:tool_executed",
                "tool": name,
                "status": "success",
                "timestamp": chrono::Utc::now().to_rfc3339()
            }));

            Ok((
                StatusCode::OK,
                Json(json!({ "status": "success", "output": redacted_output })),
            )
                .into_response())
        }
        Ok(Ok(McpResult::SystemDelegate(sys_name, _))) => {
            state.emit_event(json!({
                "type": "mcp:tool_delegated",
                "tool": name,
                "delegated_system": sys_name,
                "timestamp": chrono::Utc::now().to_rfc3339()
            }));

            Ok((
                StatusCode::ACCEPTED,
                Json(json!({
                    "status": "delegated",
                    "delegated_system": sys_name,
                    "message": format!("System tool '{}' requires an active AgentRunner context for execution.", sys_name)
                })),
            )
                .into_response())
        }
        Ok(Err(e)) => {
            let safe_err = state.security.secret_redactor.redact(&e.to_string());
            tracing::error!("❌ [MCP] Tool '{}' execution error: {}", name, safe_err);
            Err(AppError::InternalServerError(format!(
                "MCP tool execution failed: {}",
                safe_err
            )))
        }
        Err(_) => {
            tracing::error!(
                "❌ [MCP] Tool '{}' execution timed out after {}s",
                name,
                MCP_TOOL_TIMEOUT_SECS
            );
            Err(AppError::InternalServerError(format!(
                "MCP tool execution timed out after {} seconds",
                MCP_TOOL_TIMEOUT_SECS
            )))
        }
    }
}

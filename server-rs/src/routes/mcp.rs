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

/// Recursively redacts secrets from all string leaves in a structured JSON value.
fn redact_json_value(
    val: &mut serde_json::Value,
    redactor: &crate::secret_redactor::SecretRedactor,
) {
    match val {
        serde_json::Value::String(s) => {
            let redacted = redactor.redact(s);
            if redacted != *s {
                *s = redacted;
            }
        }
        serde_json::Value::Array(items) => {
            for item in items {
                redact_json_value(item, redactor);
            }
        }
        serde_json::Value::Object(map) => {
            for (_, v) in map.iter_mut() {
                redact_json_value(v, redactor);
            }
        }
        _ => {}
    }
}

/// Identifies MCP tools that interact with external cloud AI providers or outbound network egress.
fn is_external_egress_tool(name: &str) -> bool {
    let lower = name.to_ascii_lowercase();
    const BLOCKED_MARKERS: &[&str] = &[
        "google",
        "openai",
        "anthropic",
        "groq",
        "deepseek",
        "xai",
        "mistral",
        "perplexity",
        "cohere",
        "http",
        "fetch",
        "curl",
        "web_search",
        "browser",
        "slack",
        "discord",
        "github",
        "webhook",
        "email",
        "smtp",
    ];
    BLOCKED_MARKERS.iter().any(|marker| lower.contains(marker))
}

/// POST /api/mcp/tools/:name/execute
/// Executes an MCP tool directly via the API (Governance/Debugging).
#[tracing::instrument(skip(state, arguments), name = "mcp::execute_tool")]
pub async fn execute_mcp_tool(
    Path(name): Path<String>,
    State(state): State<Arc<AppState>>,
    _admin: crate::middleware::auth::RequireAdmin,
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
        // High-risk: block external cloud provider and network egress tools in privacy mode
        if is_external_egress_tool(&name) {
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
        Ok(Ok(McpResult::Structured(mut result))) => {
            redact_json_value(&mut result, &state.security.secret_redactor);
            let status = if result
                .get("isError")
                .and_then(serde_json::Value::as_bool)
                .unwrap_or(false)
            {
                "tool_error"
            } else {
                "success"
            };
            state.emit_event(json!({
                "type": "mcp:tool_executed",
                "tool": name,
                "status": status,
                "timestamp": chrono::Utc::now().to_rfc3339()
            }));
            Ok((
                StatusCode::OK,
                Json(json!({ "status": "complete", "result": result })),
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

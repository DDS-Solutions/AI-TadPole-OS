//! @docs ARCHITECTURE:Networking
//!
//! ### AI Assist Note
//! **MCP API Gateway (Tool Orchestrator)**: Orchestrates the REST
//! entry points for tool discovery and direct execution for the
//! Tadpole OS engine. Features **Dynamic Tool Discovery**: provides
//! a unified list of available tools across system, legacy, and
//! external MCP servers. Implements **Direct Tool Execution**:
//! allows for administrative or diagnostic tool calls via a
//! dedicated `api_debug` workspace. AI agents should use this
//! gateway to verify tool availability and schemas before formulating
//! complex environment interactions (MCP-04).
//!
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: Tool discovery timeouts from slow external
//!   MCP servers, 403 Forbidden for unauthorized direct execution, or
//!   workspace creation failures in the `data/workspaces` directory.
//! - **Telemetry Link**: Search for `MCP tool execution` in `tracing`
//!   logs for tool-call audit trails.
//! - **Trace Scope**: `server-rs::routes::mcp`

use crate::agent::mcp::McpResult;

use crate::routes::error::AppError;
use crate::state::AppState;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde_json::json;
use std::sync::Arc;

/// GET /api/mcp/tools
/// Lists all available MCP tools across system, legacy, and external servers.
pub async fn list_mcp_tools(
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, AppError> {
    // For the global list, we return all skills registered in the engine
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
pub async fn execute_mcp_tool(
    Path(name): Path<String>,
    State(state): State<Arc<AppState>>,
    Json(arguments): Json<serde_json::Value>,
) -> Result<impl IntoResponse, AppError> {
    // Note: Direct API execution uses a 'data/workspaces/api_debug' folder as root
    let workspace_root = std::path::PathBuf::from("data/workspaces/api_debug");
    if !workspace_root.exists() {
        let _ = std::fs::create_dir_all(&workspace_root);
    }

    match state.registry.mcp_host.call_tool(&name, arguments, workspace_root, &state.registry.skills.skills).await {
        Ok(McpResult::Raw(output)) => {
            Ok((StatusCode::OK, Json(json!({ "status": "success", "output": output }))).into_response())
        }
        Ok(McpResult::SystemDelegate(sys_name, _)) => Ok((StatusCode::ACCEPTED, Json(json!({
            "status": "delegated",
            "message": format!("System tool '{}' requires an active AgentRunner context for execution.", sys_name)
        })))
        .into_response()),
        Err(e) => Err(AppError::InternalServerError(format!("MCP tool execution failed: {}", e)))
    }
}

//! @docs ARCHITECTURE:Registry
//! 
//! ### AI Assist Note
//! **Core technical resource for the Tadpole OS Sovereign infrastructure.**
//! This module implements high-fidelity logic for the Sovereign Reality layer.
//! 
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: Runtime logic error, state desynchronization, or resource exhaustion.
//! - **Telemetry Link**: Search `[mcp_wrapper]` in tracing logs.

use crate::agent::runner::tools::trait_tool::{Tool, ToolContext};
use crate::agent::runner::tools::error::ToolExecutionError;
use super::policy::{
    is_cacheable_tool_name, is_mutating_tool_name, is_dangerous_mcp_operation, is_dangerous_native_operation,
};
use std::sync::Arc;

pub(crate) struct McpToolWrapper {
    pub(crate) name: String,
    pub(crate) mcp_host: Arc<crate::agent::mcp::McpHost>,
}

#[async_trait::async_trait]
impl Tool for McpToolWrapper {
    fn metadata(&self) -> crate::agent::runner::tools::trait_tool::ToolDefinitionData {
        // Fallback: generic schema that tells the LLM the call takes an object.
        // The real schema lives in the MCP registry (async Mutex), which can't be
        // locked synchronously here. The LLM receives full schemas via list_tools.
        crate::agent::runner::tools::trait_tool::ToolDefinitionData {
            name: self.name.clone(),
            description: format!("MCP tool: {}", self.name),
            parameters: serde_json::json!({"type": "object", "additionalProperties": true}),
        }
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn is_cacheable(&self) -> bool {
        is_cacheable_tool_name(&self.name)
    }

    fn is_mutating(&self) -> bool {
        is_mutating_tool_name(&self.name)
    }

    fn is_dangerous(&self) -> bool {
        // IMPORTANT: Composition is intentional. Every mutating tool is also dangerous.
        // Add new dangerous operations to one of three places:
        //   1. `is_mutating_tool_name` — if it modifies workspace files/state
        //   2. `is_dangerous_native_operation` — if it's a built-in with security risk
        //   3. `is_dangerous_mcp_operation` — if it's an MCP tool with data-leak risk
        let name = self.name.as_str();
        is_mutating_tool_name(name)
            || is_dangerous_native_operation(name)
            || is_dangerous_mcp_operation(name)
    }

    async fn execute(
        &self,
        ctx: &ToolContext,
        args: serde_json::Value,
        usage: &mut Option<crate::agent::types::TokenUsage>,
    ) -> Result<String, ToolExecutionError> {
        let result = self
            .mcp_host
            .call_tool(
                &self.name,
                args,
                ctx.workspace_root.clone(),
                &ctx.state.registry.skills.skills,
            )
            .await
            .map_err(ToolExecutionError::AppError)?;

        match result {
            crate::agent::mcp::McpResult::Raw(out) => Ok(out),
            crate::agent::mcp::McpResult::SystemDelegate(name, delegate_args) => {
                if name == "recruit_specialist" {
                    let mut mapped_args = serde_json::Map::new();
                    if let Some(aid) = delegate_args.get("agent_id") {
                        mapped_args.insert("agent_id".to_string(), aid.clone());
                    }
                    if let Some(msg) = delegate_args.get("task_description") {
                        mapped_args.insert("message".to_string(), msg.clone());
                    }

                    let spawn_tool = ctx
                        .state
                        .registry
                        .tool_registry
                        .get("spawn_subagent")
                        .ok_or_else(|| ToolExecutionError::ToolNotFound {
                            name: "spawn_subagent".to_string(),
                        })?;

                    spawn_tool
                        .execute(ctx, serde_json::Value::Object(mapped_args), usage)
                        .await
                } else {
                    Err(ToolExecutionError::ExecutionFailed(format!(
                        "Unhandled system delegate '{}'",
                        name
                    )))
                }
            }
        }
    }
}

// Metadata: [mcp_wrapper]

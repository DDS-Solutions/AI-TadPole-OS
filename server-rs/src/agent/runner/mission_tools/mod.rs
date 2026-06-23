//! @docs ARCHITECTURE:Runner
//!
//! ### AI Assist Note
//! **Mission Tools Entry Point**: Routes to codebase, knowledge, lifecycle, ledger, and swarm submodules.
//!
//! ### 🔍 Debugging & Observability
//! - **Trace Scope**: `server-rs::agent::runner::mission_tools`

mod codebase;
mod knowledge;
mod ledger;
mod lifecycle;
mod swarm;

use crate::agent::runner::RunContext;
use crate::agent::runner::tools::error::ToolExecutionError;
use crate::error::AppError;

pub(crate) fn require_str(
    ctx: &RunContext,
    args: &serde_json::Value,
    key: &str,
    tool_name: &str,
) -> Result<String, ToolExecutionError> {
    args.get(key)
        .ok_or_else(|| {
            ToolExecutionError::AppError(AppError::BadRequest(format!(
                "[Agent {} | Mission {}] Tool '{}' missing required argument '{}'",
                ctx.agent_id, ctx.mission_id, tool_name, key
            )))
        })?
        .as_str()
        .filter(|s| !s.trim().is_empty())
        .map(|s| s.to_string())
        .ok_or_else(|| {
            ToolExecutionError::AppError(AppError::BadRequest(format!(
                "[Agent {} | Mission {}] Tool '{}' argument '{}' must be a non-empty string",
                ctx.agent_id, ctx.mission_id, tool_name, key
            )))
        })
}

pub(crate) fn require_str_opt(
    ctx: &RunContext,
    args: &serde_json::Value,
    key: &str,
    tool_name: &str,
) -> Result<Option<String>, ToolExecutionError> {
    match args.get(key) {
        None => Ok(None),
        Some(v) => {
            if v.is_null() {
                Ok(None)
            } else {
                v.as_str()
                    .map(|s| Some(s.to_string()))
                    .ok_or_else(|| {
                        ToolExecutionError::AppError(AppError::BadRequest(format!(
                            "[Agent {} | Mission {}] Tool '{}' argument '{}' must be a valid string",
                            ctx.agent_id, ctx.mission_id, tool_name, key
                        )))
                    })
            }
        }
    }
}

pub(crate) fn require_u64(
    ctx: &RunContext,
    args: &serde_json::Value,
    key: &str,
    tool_name: &str,
) -> Result<u64, ToolExecutionError> {
    args.get(key)
        .ok_or_else(|| {
            ToolExecutionError::AppError(AppError::BadRequest(format!(
                "[Agent {} | Mission {}] Tool '{}' missing required argument '{}'",
                ctx.agent_id, ctx.mission_id, tool_name, key
            )))
        })?
        .as_u64()
        .ok_or_else(|| {
            ToolExecutionError::AppError(AppError::BadRequest(format!(
                "[Agent {} | Mission {}] Tool '{}' argument '{}' must be a valid positive integer",
                ctx.agent_id, ctx.mission_id, tool_name, key
            )))
        })
}

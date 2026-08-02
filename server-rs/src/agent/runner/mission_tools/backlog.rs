//! @docs ARCHITECTURE:Registry
//!
//! ### AI Assist Note
//! **! @docs ARCHITECTURE:Runner**
//! This module implements high-fidelity logic for the Sovereign Reality layer.
//!
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: Runtime logic error, state desynchronization, or resource exhaustion.
//! - **Telemetry Link**: Search `[backlog]` in tracing logs.

//!   @docs ARCHITECTURE:Runner
//!
//! ### AI Assist Note
//! **Mission Backlog Tools**: Implementation for task DAG management.

use super::require_str;
use crate::agent::runner::tools::error::ToolExecutionError;
use crate::agent::runner::{AgentRunner, RunContext};
use crate::error::AppError;

impl AgentRunner {
    pub(crate) async fn handle_add_mission_task(
        &self,
        ctx: &RunContext,
        fc: &crate::agent::types::ToolCall,
    ) -> Result<String, ToolExecutionError> {
        let description = require_str(ctx, &fc.args, "description", "add_mission_task")?;

        let dependencies =
            if let Some(deps) = fc.args.get("dependencies").and_then(|v| v.as_array()) {
                deps.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect()
            } else {
                Vec::new()
            };

        if let Some(backlog_arc) = &ctx.backlog {
            let mut backlog = backlog_arc.lock();
            let task_id = backlog.add_task(&description, dependencies.clone());

            tracing::info!(
                "📋 [Backlog] Agent {} added task {} to mission {}",
                ctx.agent_id,
                task_id,
                ctx.mission_id
            );

            self.broadcast_agent(
                ctx,
                &format!("📋 Added task to mission backlog: {}", description),
                "info",
            );

            Ok(format!(
                "Task added successfully. Task ID: {}\n\nCurrent Backlog:\n{}",
                task_id,
                backlog.progress_report()
            ))
        } else {
            Err(ToolExecutionError::AppError(AppError::InternalServerError(
                "Mission backlog not initialized for this run context.".to_string(),
            )))
        }
    }

    pub(crate) async fn handle_update_mission_task(
        &self,
        ctx: &RunContext,
        fc: &crate::agent::types::ToolCall,
    ) -> Result<String, ToolExecutionError> {
        let task_id = require_str(ctx, &fc.args, "task_id", "update_mission_task")?;
        let status_str = require_str(ctx, &fc.args, "status", "update_mission_task")?;

        let status = match status_str.to_lowercase().as_str() {
            "pending" => crate::agent::backlog::TaskStatus::Pending,
            "inprogress" | "in_progress" | "in-progress" => {
                crate::agent::backlog::TaskStatus::InProgress
            }
            "completed" => crate::agent::backlog::TaskStatus::Completed,
            "failed" => crate::agent::backlog::TaskStatus::Failed,
            "blocked" => crate::agent::backlog::TaskStatus::Blocked,
            _ => {
                return Err(ToolExecutionError::AppError(AppError::BadRequest(format!(
                    "Invalid status: {}",
                    status_str
                ))))
            }
        };

        let result = fc
            .args
            .get("result")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        if let Some(backlog_arc) = &ctx.backlog {
            let mut backlog = backlog_arc.lock();
            backlog.update_status(&task_id, status, result);

            tracing::info!(
                "📋 [Backlog] Agent {} updated task {} status to {}",
                ctx.agent_id,
                task_id,
                status_str
            );

            self.broadcast_agent(
                ctx,
                &format!("📋 Updated task {} status to {}", task_id, status_str),
                "info",
            );

            Ok(format!(
                "Task updated successfully.\n\nCurrent Backlog:\n{}",
                backlog.progress_report()
            ))
        } else {
            Err(ToolExecutionError::AppError(AppError::InternalServerError(
                "Mission backlog not initialized for this run context.".to_string(),
            )))
        }
    }
}

// Metadata: [backlog]

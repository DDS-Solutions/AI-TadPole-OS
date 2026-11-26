//! @docs ARCHITECTURE:Runner
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Agent Runner / script_builder
//!
//! ### AI Assist Note
//! Recursively executes a batch of tool calls provided by the LLM.
//! Collapses multiple model turns into a single execution chain.
//!
//! ### 🔍 Debugging & Observability
//! - **Telemetry Targets**: `[ScriptBuilder]`

use super::policy::{
    is_dangerous_mcp_operation, is_dangerous_native_operation, is_mutating_tool_name,
};
use crate::agent::runner::service_traits::ToolExecutor;
use crate::agent::runner::{AgentRunner, RunContext};
use crate::error::AppError;

fn validate_tool_name(name: &str, context: &str) -> Result<(), AppError> {
    let valid = !name.is_empty()
        && name.len() <= 64
        && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-');
    if !valid {
        let snippet = &name[..name.len().min(20)];
        return Err(AppError::BadRequest(format!(
            "Invalid tool name in {}: '{}'. Must be 1-64 alphanumeric/underscore/dash characters.",
            context, snippet
        )));
    }
    Ok(())
}

fn extract_batch_params(
    fc: &crate::agent::types::ToolCall,
) -> Result<(&Vec<serde_json::Value>, Vec<String>), AppError> {
    let steps = fc
        .args
        .get("steps")
        .and_then(|v| v.as_array())
        .ok_or_else(|| {
            AppError::BadRequest("'steps' must be an array in script_builder".to_string())
        })?;

    let allowed_tools: Vec<String> = fc
        .args
        .get("allowed_tools")
        .and_then(|v| v.as_array())
        .ok_or_else(|| {
            AppError::BadRequest(
                "'allowed_tools' must be an array of strings in script_builder".to_string(),
            )
        })?
        .iter()
        .filter_map(|v| v.as_str().map(|s| s.to_string()))
        .collect();

    for tool_name in &allowed_tools {
        validate_tool_name(tool_name, "allowed_tools")?;
    }

    Ok((steps, allowed_tools))
}

fn is_tool_dangerous(runner: &AgentRunner, tool_name: &str) -> bool {
    if let Some(t) = runner.state.registry.tool_registry.get(tool_name) {
        return t.is_dangerous() || t.is_mutating();
    }
    is_mutating_tool_name(tool_name)
        || is_dangerous_mcp_operation(tool_name)
        || is_dangerous_native_operation(tool_name)
}

fn inspect_batch_safety(
    runner: &AgentRunner,
    steps: &[serde_json::Value],
    allowed_tools: &[String],
) -> Result<bool, AppError> {
    let mut has_dangerous = false;
    for (i, step) in steps.iter().enumerate() {
        let tool_name = step
            .get("tool")
            .and_then(|v| v.as_str())
            .ok_or_else(|| AppError::BadRequest(format!("Step {} missing 'tool' name", i)))?;

        validate_tool_name(tool_name, &format!("step {}", i))?;

        if tool_name == "script_builder" {
            return Err(AppError::BadRequest(
                "Recursive batching via 'script_builder' is strictly forbidden.".to_string(),
            ));
        }

        if !allowed_tools.iter().any(|a| a == tool_name) {
            return Err(AppError::BadRequest(format!(
                "Tool '{}' in step {} is not in the allowed_tools list for this batch",
                tool_name, i
            )));
        }

        if !has_dangerous && is_tool_dangerous(runner, tool_name) {
            has_dangerous = true;
        }
    }
    Ok(has_dangerous)
}

struct StepState<'a> {
    output_text: &'a mut String,
    usage: &'a mut Option<crate::agent::types::TokenUsage>,
    user_message: &'a str,
}

impl AgentRunner {
    async fn request_batch_oversight(
        &self,
        ctx: &RunContext,
        fc: &crate::agent::types::ToolCall,
        allowed_tools: &[String],
    ) -> Result<bool, AppError> {
        self.broadcast_sys(
            "🔒 Security Gate: Batch execution via 'script_builder' requires explicit approval.",
            "warning",
            Some(ctx.mission_state.mission_id.clone()),
        );

        let desc_list: Vec<_> = allowed_tools.iter().take(10).cloned().collect();
        let description = format!(
            "Batch tool execution containing dangerous/mutating steps: [{}]",
            desc_list.join(", ")
        );
        let description = self.state.security.secret_redactor.redact(&description);

        self.submit_oversight(
            crate::agent::types::ToolCallAudit {
                id: uuid::Uuid::new_v4().to_string(),
                agent_id: ctx.identity.agent_id.clone(),
                mission_id: Some(ctx.mission_state.mission_id.clone()),
                skill: "script_builder".to_string(),
                params: fc.args.clone(),
                department: ctx.identity.department.clone(),
                description,
                timestamp: chrono::Utc::now().to_rfc3339(),
            },
            Some(ctx.mission_state.mission_id.clone()),
        )
        .await
    }

    async fn execute_single_step(
        &self,
        ctx: &RunContext,
        i: usize,
        step: &serde_json::Value,
        state: &mut StepState<'_>,
    ) -> Result<(), AppError> {
        let Some(tool_name) = step.get("tool").and_then(|v| v.as_str()) else {
            return Ok(());
        };
        let params = step
            .get("params")
            .cloned()
            .unwrap_or_else(|| serde_json::Value::Object(serde_json::Map::new()));

        let step_fc = crate::agent::types::ToolCall {
            name: tool_name.to_string(),
            args: params,
        };

        tracing::info!("📦 [ScriptBuilder] Executing step {}: {}", i + 1, tool_name);
        state
            .output_text
            .push_str(&format!("\n[Step {}: {}]\n", i + 1, tool_name));

        let (res_text, res_usage) = self.execute_tool(ctx, &step_fc, state.user_message).await?;
        if let Some(u) = res_usage {
            self.accumulate_usage(state.usage, Some(u));
        }
        state.output_text.push_str(&res_text);
        Ok(())
    }

    /// Recursively executes a batch of tool calls provided by the LLM.
    pub(crate) async fn handle_script_builder(
        &self,
        ctx: &RunContext,
        fc: &crate::agent::types::ToolCall,
        output_text: &mut String,
        usage: &mut Option<crate::agent::types::TokenUsage>,
        user_message: &str,
    ) -> Result<(), AppError> {
        let (steps, allowed_tools) = extract_batch_params(fc)?;
        let has_dangerous = inspect_batch_safety(self, steps, &allowed_tools)?;

        if has_dangerous {
            let approved = self
                .request_batch_oversight(ctx, fc, &allowed_tools)
                .await?;
            if !approved {
                *output_text = "(Batch tool execution REJECTED by Oversight)".to_string();
                return Ok(());
            }
        }

        output_text.push_str("\n--- BATCH EXECUTION STARTED ---\n");
        let mut state = StepState {
            output_text,
            usage,
            user_message,
        };
        for (i, step) in steps.iter().enumerate() {
            self.execute_single_step(ctx, i, step, &mut state).await?;
        }
        state
            .output_text
            .push_str("\n--- BATCH EXECUTION COMPLETED ---\n");

        Ok(())
    }
}

//! @docs ARCHITECTURE:Registry
//! 
//! ### AI Assist Note
//! **Core technical resource for the Tadpole OS Sovereign infrastructure.**
//! This module implements high-fidelity logic for the Sovereign Reality layer.
//! 
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: Runtime logic error, state desynchronization, or resource exhaustion.
//! - **Telemetry Link**: Search `[system_tools]` in tracing logs.

use crate::error::AppError;
use crate::agent::runner::{AgentRunner, RunContext};
use super::policy::{
    is_mutating_tool_name, is_dangerous_mcp_operation, is_dangerous_native_operation,
};

/// Performs a null-aware shallow merge of `patch` into `target`.
/// Keys whose value is `null` in the patch are removed from the target (RFC 7396 JSON Merge Patch).
fn merge_working_memory(target: &mut serde_json::Map<String, serde_json::Value>, patch: &serde_json::Map<String, serde_json::Value>) {
    for (k, v) in patch {
        if v.is_null() {
            target.remove(k);
        } else {
            target.insert(k.clone(), v.clone());
        }
    }
}

impl AgentRunner {
    /// Updates the agent's persistent working memory (scratchpad).
    ///
    /// ### 🧠 Cognition Side Effects
    /// This memory persists across agent spawns and engine restarts. It is the
    /// primary mechanism for an agent to maintain "Context Continuity" when
    /// executing multi-stage missions.
    pub(crate) async fn handle_update_working_memory(
        &self,
        ctx: &RunContext,
        fc: &crate::agent::types::ToolCall,
        output_text: &mut String,
    ) -> Result<(), AppError> {
        let new_memory = fc
            .args
            .get("memory")
            .cloned()
            .unwrap_or(serde_json::json!({}));

        let mut agent_clone = {
            if let Some(mut entry) = self.state.registry.agents.get_mut(&ctx.agent_id) {
                let agent = entry.value_mut();

                // If both are objects, we perform a shallow merge (and remove keys where value is null).
                if let (Some(old_obj), Some(new_obj)) = (
                    agent.state.working_memory.as_object_mut(),
                    new_memory.as_object(),
                ) {
                    merge_working_memory(old_obj, new_obj);
                } else {
                    agent.state.working_memory = new_memory.clone();
                }
                agent.clone()
            } else {
                *output_text =
                    "(ERROR: Agent not found in registry during working memory update)".to_string();
                return Ok(());
            }
        };

        // Sync to DB with optimistic concurrency conflict retry
        let pool = self.state.resources.pool.clone();
        let mut current_version = agent_clone.version;
        let mut retries = 0;
        loop {
            match crate::agent::persistence::save_agent_db(&pool, &mut agent_clone).await {
                Ok(_) => break,
                Err(AppError::Conflict(_)) if retries < 5 => {
                    retries += 1;
                    if let Some(db_agent) = crate::agent::persistence::load_agent_by_id_db(&pool, &ctx.agent_id).await? {
                        let mut updated_agent = db_agent;
                        if let (Some(old_obj), Some(new_obj)) = (
                            updated_agent.state.working_memory.as_object_mut(),
                            new_memory.as_object(),
                        ) {
                            merge_working_memory(old_obj, new_obj);
                        } else {
                            updated_agent.state.working_memory = new_memory.clone();
                        }
                        current_version = updated_agent.version;
                        agent_clone = updated_agent;
                    }
                    tokio::time::sleep(std::time::Duration::from_millis(50 * retries)).await;
                }
                Err(e) => return Err(e),
            }
        }

        // Re-align memory registry with updated version
        if let Some(mut entry) = self.state.registry.agents.get_mut(&ctx.agent_id) {
            let agent = entry.value_mut();
            if agent.version == current_version {
                *agent = agent_clone.clone();
            } else {
                // Concurrency conflict: reload from DB
                if let Some(db_agent) = crate::agent::persistence::load_agent_by_id_db(&pool, &ctx.agent_id).await? {
                    *agent = db_agent;
                }
            }

            self.state.emit_event(serde_json::json!({
                "type": "agent:update",
                "agent_id": ctx.agent_id,
                "data": agent.clone()
            }));

            *output_text = "(WORKING MEMORY UPDATED SUCCESSFULLY)".to_string();
        } else {
            *output_text =
                "(ERROR: Agent not found in registry during working memory update)".to_string();
        }

        Ok(())
    }

    pub(crate) async fn handle_request_model_switch(
        &self,
        ctx: &RunContext,
        fc: &crate::agent::types::ToolCall,
        output_text: &mut String,
    ) -> Result<(), AppError> {
        let requested_slot = fc
            .args
            .get("slot")
            .and_then(|v| v.as_str())
            .unwrap_or("default");

        if requested_slot != "planning"
            && requested_slot != "execution"
            && requested_slot != "default"
        {
            *output_text = format!("(ERROR: Invalid model slot '{}'. Valid slots are: 'planning', 'execution', 'default')", requested_slot);
            return Ok(());
        }

        let (model_id, provider) = if let Some(entry) =
            self.state.registry.agents.get(&ctx.agent_id)
        {
            let agent = entry.value();
            let config_opt = match requested_slot {
                "planning" => agent.models.planning_slot.as_ref(),
                "execution" => agent.models.execution_slot.as_ref(),
                _ => Some(&agent.models.model),
            };

            if let Some(config) = config_opt {
                (config.model_id.clone(), config.provider.to_string())
            } else {
                *output_text = format!("(ERROR: Requested model slot '{}' has no configuration defined for this agent)", requested_slot);
                return Ok(());
            }
        } else {
            *output_text = "(ERROR: Agent not found in registry during model switch)".to_string();
            return Ok(());
        };

        self.broadcast_agent(
            ctx,
            &format!(
                "💎 Oversight: wants to switch active model slot to '{}' (model: {}, provider: {}). CRITICAL REVIEW REQUIRED.",
                requested_slot, model_id, provider
            ),
            "error",
        );

        let res = self
            .submit_oversight_resolution(
                crate::agent::types::ToolCallAudit {
                    id: uuid::Uuid::new_v4().to_string(),
                    agent_id: ctx.agent_id.clone(),
                    mission_id: Some(ctx.mission_id.clone()),
                    skill: "request_model_switch".to_string(),
                    params: fc.args.clone(),
                    department: ctx.department.clone(),
                    description: format!(
                        "Switch active model slot to '{}' (model: {}, provider: {})",
                        requested_slot, model_id, provider
                    ),
                    timestamp: chrono::Utc::now().to_rfc3339(),
                },
                Some(ctx.mission_id.clone()),
            )
            .await?;

        if !res.approved {
            *output_text = "(Model slot switch REJECTED by Oversight)".to_string();
            return Ok(());
        }

        let target_slot = res
            .override_slot
            .clone()
            .unwrap_or_else(|| requested_slot.to_string());

        if target_slot != "planning" && target_slot != "execution" && target_slot != "default" {
            *output_text = format!("(ERROR: Invalid overridden model slot '{}')", target_slot);
            return Ok(());
        }

        // Verify if overridden slot exists
        if let Some(entry) = self.state.registry.agents.get(&ctx.agent_id) {
            let agent = entry.value();
            let config_opt = match target_slot.as_str() {
                "planning" => agent.models.planning_slot.as_ref(),
                "execution" => agent.models.execution_slot.as_ref(),
                _ => Some(&agent.models.model),
            };
            if config_opt.is_none() {
                *output_text = format!("(ERROR: Overridden model slot '{}' has no configuration defined for this agent)", target_slot);
                return Ok(());
            }
        }

        let mut agent_clone = {
            if let Some(mut entry) = self.state.registry.agents.get_mut(&ctx.agent_id) {
                let agent = entry.value_mut();
                agent.models.active_model_slot = Some(target_slot.clone());
                agent.clone()
            } else {
                *output_text = "(ERROR: Agent not found in registry during update)".to_string();
                return Ok(());
            }
        };

        // Sync to DB with optimistic concurrency conflict retry
        let pool = self.state.resources.pool.clone();
        let mut current_version = agent_clone.version;
        let mut retries = 0;
        loop {
            match crate::agent::persistence::save_agent_db(&pool, &mut agent_clone).await {
                Ok(_) => break,
                Err(AppError::Conflict(_)) if retries < 5 => {
                    retries += 1;
                    if let Some(db_agent) = crate::agent::persistence::load_agent_by_id_db(&pool, &ctx.agent_id).await? {
                        let mut updated_agent = db_agent;
                        updated_agent.models.active_model_slot = Some(target_slot.clone());
                        current_version = updated_agent.version;
                        agent_clone = updated_agent;
                    }
                    tokio::time::sleep(std::time::Duration::from_millis(50 * retries)).await;
                }
                Err(e) => return Err(e),
            }
        }

        // Re-align memory registry with updated version
        if let Some(mut entry) = self.state.registry.agents.get_mut(&ctx.agent_id) {
            let agent = entry.value_mut();
            if agent.version == current_version {
                *agent = agent_clone.clone();
            } else {
                // Concurrency conflict: reload from DB
                if let Some(db_agent) = crate::agent::persistence::load_agent_by_id_db(&pool, &ctx.agent_id).await? {
                    *agent = db_agent;
                }
            }

            let agents: Vec<_> = self
                .state
                .registry
                .agents
                .iter()
                .map(|kv| kv.value().clone())
                .collect();
            if let Err(err) =
                crate::agent::persistence::save_agents_json(&self.state.base_dir, agents).await
            {
                tracing::error!("Failed to sync agents.json during model switch: {}", err);
            }

            self.state.emit_event(serde_json::json!({
                "type": "agent:update",
                "agent_id": ctx.agent_id,
                "data": agent.clone()
            }));

            self.broadcast_agent(
                ctx,
                &format!(
                    "🔄 System: Model slot successfully switched to '{}'.",
                    target_slot
                ),
                "info",
            );

            *output_text = format!("(MODEL SLOT SUCCESSFULLY SWITCHED TO '{}')", target_slot);
        } else {
            *output_text = "(ERROR: Agent not found in registry during update)".to_string();
        }

        Ok(())
    }

    /// Recursively executes a batch of tool calls provided by the LLM.
    ///
    /// ### ⏩ Efficiency Engine
    /// This "collapses" multiple model turns into a single execution chain.
    /// It is used by the model when it has high confidence in a sequence of
    /// deterministic steps (e.g., "Read File -> Grep -> Write Result").
    pub(crate) async fn handle_script_builder(
        &self,
        ctx: &RunContext,
        fc: &crate::agent::types::ToolCall,
        output_text: &mut String,
        usage: &mut Option<crate::agent::types::TokenUsage>,
        user_message: &str,
    ) -> Result<(), AppError> {
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

        // Helper to validate tool name identifier format
        let validate_tool_name = |name: &str, context: &str| -> Result<(), AppError> {
            if name.is_empty()
                || name.len() > 64
                || !name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
            {
                Err(AppError::BadRequest(format!(
                    "Invalid tool name in {}: '{}'. Must be 1-64 alphanumeric/underscore/dash characters.",
                    context,
                    &name[..name.len().min(20)]
                )))
            } else {
                Ok(())
            }
        };

        // Validate allowed_tools element shape: only safe identifiers
        for tool_name in &allowed_tools {
            validate_tool_name(tool_name, "allowed_tools")?;
        }

        // Single-pass: validate all steps are in allowed_tools AND detect dangerous tools
        let mut has_dangerous = false;
        for (i, step) in steps.iter().enumerate() {
            let tool_name = step
                .get("tool")
                .and_then(|v| v.as_str())
                .ok_or_else(|| AppError::BadRequest(format!("Step {} missing 'tool' name", i)))?;

            // Validate step["tool"] charset and length to prevent name collision/injection
            validate_tool_name(tool_name, &format!("step {}", i))?;

            if !allowed_tools.iter().any(|a| a == tool_name) {
                return Err(AppError::BadRequest(format!(
                    "Tool '{}' in step {} is not in the allowed_tools list for this batch",
                    tool_name, i
                )));
            }

            if !has_dangerous {
                if let Some(t) = self.state.registry.tool_registry.get(tool_name) {
                    if t.is_dangerous() || t.is_mutating() {
                        has_dangerous = true;
                    }
                } else if is_mutating_tool_name(tool_name)
                    || is_dangerous_mcp_operation(tool_name)
                    || is_dangerous_native_operation(tool_name)
                {
                    has_dangerous = true;
                }
            }
        }

        if has_dangerous {
            // Submit the entire batch to oversight!
            self.broadcast_sys(
                "🔒 Security Gate: Batch execution via 'script_builder' requires explicit approval.",
                "warning",
                Some(ctx.mission_id.clone()),
            );

            let description = format!(
                "Batch tool execution containing dangerous/mutating steps: [{}]",
                allowed_tools
                    .iter()
                    .take(10)
                    .cloned()
                    .collect::<Vec<_>>()
                    .join(", ")
            );
            let description = self.state.security.secret_redactor.redact(&description);

            let approved = self
                .submit_oversight(
                    crate::agent::types::ToolCallAudit {
                        id: uuid::Uuid::new_v4().to_string(),
                        agent_id: ctx.agent_id.clone(),
                        mission_id: Some(ctx.mission_id.clone()),
                        skill: "script_builder".to_string(),
                        params: fc.args.clone(),
                        department: ctx.department.clone(),
                        description,
                        timestamp: chrono::Utc::now().to_rfc3339(),
                    },
                    Some(ctx.mission_id.clone()),
                )
                .await?;

            if !approved {
                *output_text = "(Batch tool execution REJECTED by Oversight)".to_string();
                return Ok(());
            }
        }

        output_text.push_str("\n--- BATCH EXECUTION STARTED ---\n");

        for (i, step) in steps.iter().enumerate() {
            let tool_name = step.get("tool").and_then(|v| v.as_str()).unwrap();
            let params = step
                .get("params")
                .cloned()
                .unwrap_or(serde_json::Value::Object(serde_json::Map::new()));

            let step_fc = crate::agent::types::ToolCall {
                name: tool_name.to_string(),
                args: params,
            };

            tracing::info!("📦 [ScriptBuilder] Executing step {}: {}", i + 1, tool_name);
            output_text.push_str(&format!("\n[Step {}: {}]\n", i + 1, tool_name));

            // Execute the individual tool
            use crate::agent::runner::service_traits::ToolExecutor;
            let (res_text, res_usage) = self.execute_tool(ctx, &step_fc, user_message).await?;
            if let Some(u) = res_usage {
                self.accumulate_usage(usage, Some(u));
            }
            let step_output = res_text;

            output_text.push_str(&step_output);
        }

        output_text.push_str("\n--- BATCH EXECUTION COMPLETED ---\n");
        Ok(())
    }

    /// Handles `execute_shell`: runs a terminal command in the workspace.
    /// 🛡️ PROTECTED: Requires Sapphire Gate (Critical Oversight) and ShellScanner.
    pub(crate) async fn handle_execute_shell(
        &self,
        ctx: &RunContext,
        fc: &crate::agent::types::ToolCall,
        output_text: &mut String,
    ) -> Result<(), AppError> {
        let command_str = fc
            .args
            .get("command")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        if command_str.is_empty() {
            *output_text = "(SHELL FAILED: 'command' argument is missing)".to_string();
            return Ok(());
        }

        tracing::info!(
            "💻 [System] Agent {} requesting shell execution: {}",
            ctx.agent_id,
            command_str
        );

        // 1. Security Scanner
        if let Err(e) = crate::utils::security::validate_shell_command(command_str) {
            tracing::warn!(
                "🛡️ [Security] Shell execution BLOCKED by basic scanner: {}",
                e
            );
            *output_text = format!("(SECURITY BLOCKED: {})", e);
            return Ok(());
        }

        match self.state.security.shell_scanner.scan(command_str) {
            crate::security::scanner::ScannerResult::Risky(reason) => {
                tracing::warn!(
                    "🛡️ [Security] Shell execution BLOCKED by advanced scanner: {}",
                    reason
                );
                *output_text = format!("(SECURITY BLOCKED: {})", reason);
                return Ok(());
            }
            crate::security::scanner::ScannerResult::Safe => {}
        }

        self.broadcast_agent(
            ctx,
            &format!(
                "💎 Oversight: wants to run terminal command: {}. CRITICAL REVIEW REQUIRED.",
                command_str
            ),
            "error", // Use error color for Sapphire Gate
        );

        // 2. Sapphire Gate Oversight
        let approved = self
            .submit_oversight(
                crate::agent::types::ToolCallAudit {
                    id: uuid::Uuid::new_v4().to_string(),
                    agent_id: ctx.agent_id.clone(),
                    mission_id: Some(ctx.mission_id.clone()),
                    skill: "execute_shell".to_string(),
                    params: fc.args.clone(),
                    department: ctx.department.clone(),
                    description: format!(
                        "Executing terminal command in workspace: {}",
                        command_str
                    ),
                    timestamp: chrono::Utc::now().to_rfc3339(),
                },
                Some(ctx.mission_id.clone()),
            )
            .await?;

        if !approved {
            *output_text = "(Shell execution REJECTED by Oversight)".to_string();
            return Ok(());
        }

        self.broadcast_agent(
            ctx,
            &format!("💻 System: running '{}'...", command_str),
            "info",
        );

        // 3. Execution
        let shell = if cfg!(windows) { "powershell" } else { "sh" };
        let flag = if cfg!(windows) { "-Command" } else { "-c" };

        let child = tokio::process::Command::new(shell)
            .arg(flag)
            .arg(command_str)
            .current_dir(&ctx.workspace_root)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn();

        match child {
            Ok(mut child) => {
                {
                    let mut cmds = ctx.commands_run.lock();
                    cmds.insert(command_str.to_string());
                }

                use tokio::io::AsyncReadExt;

                let mut child_stdout = child.stdout.take();
                let mut child_stderr = child.stderr.take();

                // Read both pipes concurrently to prevent OS pipe buffer deadlock (F2)
                let (stdout_buf, stderr_buf) = tokio::join!(
                    async {
                        let mut buf = Vec::new();
                        if let Some(pipe) = child_stdout.as_mut() {
                            let _ = pipe.take(32_768).read_to_end(&mut buf).await;
                        }
                        buf
                    },
                    async {
                        let mut buf = Vec::new();
                        if let Some(pipe) = child_stderr.as_mut() {
                            let _ = pipe.take(32_768).read_to_end(&mut buf).await;
                        }
                        buf
                    }
                );

                let _ = child.wait().await;

                let stdout_str = String::from_utf8_lossy(&stdout_buf);
                let stderr_str = String::from_utf8_lossy(&stderr_buf);

                let combined = format!("{}{}", stdout_str, stderr_str);
                let truncated = self.safe_truncate(&combined, 5000);

                *output_text = format!("(SHELL OUTPUT of '{}'):\n\n{}", command_str, truncated);
            }
            Err(e) => {
                *output_text = format!("(SHELL EXECUTION FAILED: {})", e);
            }
        }

        Ok(())
    }
}

// Metadata: [system_tools]

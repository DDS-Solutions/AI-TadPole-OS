//! @docs ARCHITECTURE:Runner
//!
//! ### AI Assist Note
//! **Tool Dispatcher**: Orchestrates the execution of both built-in and dynamic
//! script-based tools. Enforces **Zero-Trust CBS (Capability-Based Security)** and
//! **Human-in-the-Loop Oversight**. Implements **WAL (Write-Ahead Logging)**
//! to ensure all tool attempts are persisted before execution (SEC-04).
//!
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: Tool registration collision, WAL persistence failure, 
//!   or security gate rejection.
//! - **Telemetry Link**: Search `[tools]` in tracing logs.
//! - **Trace Scope**: `server-rs::agent::runner::tools`

pub mod capability;
pub mod dispatcher;
pub mod error;
pub mod manifest;
pub mod registry;
pub mod security;
pub mod trait_tool;
pub mod cache;
pub mod plugin;
#[macro_use]
pub mod macros;
use crate::error::AppError;
use security::{DefaultSecurityManager, SecurityManager};

pub use capability::{CapabilityToken, ZeroTrustGuard, Permission};
pub use trait_tool::{Tool, ToolContext};

use super::{AgentRunner, RunContext};
use error::ToolExecutionError;
use std::sync::Arc;

pub struct ToolLeaseGuard {
    conflict_manager: Arc<crate::security::conflict::ConflictManager>,
    path: std::path::PathBuf,
    active: bool,
}

impl ToolLeaseGuard {
    pub fn acquire(
        conflict_manager: Arc<crate::security::conflict::ConflictManager>,
        path: std::path::PathBuf,
        agent_id: String,
    ) -> Result<Self, ToolExecutionError> {
        conflict_manager
            .acquire_lease(path.clone(), agent_id)
            .map_err(ToolExecutionError::AppError)?;
        Ok(Self {
            conflict_manager,
            path,
            active: true,
        })
    }

    pub fn release(&mut self) {
        if self.active {
            self.conflict_manager.release_lease(&self.path);
            self.active = false;
        }
    }
}

impl Drop for ToolLeaseGuard {
    fn drop(&mut self) {
        if self.active {
            self.conflict_manager.release_lease(&self.path);
        }
    }
}

impl AgentRunner {
    /// Dispatches a function call to the appropriate tool handler.
    /// Orchestrates the Zero-Trust pipeline: WAL -> CBS -> Audit -> Execute.
    pub(crate) async fn execute_tool(
        &self,
        ctx: &RunContext,
        fc: &crate::agent::types::ToolCall,
        output_text: &mut String,
        usage: &mut Option<crate::agent::types::TokenUsage>,
        user_message: &str,
    ) -> Result<Option<String>, AppError> {
        // 1. Mint Capability Token for this specific call
        let token = ZeroTrustGuard::mint_token(
            &ctx.agent_id,
            &ctx.mission_id,
            ctx.authority_level,
            ctx.allowed_files.as_deref(),
            &ctx.workspace_root,
        );

        match self
            .run_zero_trust_pipeline(ctx, fc, usage, user_message, token)
            .await
        {
            Ok(output) => {
                let redacted = self.state.security.secret_redactor.redact(&output);
                *output_text = redacted.clone();
                Ok(Some(redacted))
            }
            Err(e) => {
                let recovery = e.recovery_strategy();
                let error_msg = format!("(TOOL FAILURE: {} | RECOVERY: {:?})", e, recovery);
                let redacted = self.state.security.secret_redactor.redact(&error_msg);
                *output_text = redacted.clone();

                // Even on error, we return Ok(Some) to surface the failure to the agent
                // unless it's a critical infrastructure failure.
                Ok(Some(redacted))
            }
        }
    }

    /// Manages the Zero-Trust sequence (WAL -> CBS -> Execute)
    async fn run_zero_trust_pipeline(
        &self,
        ctx: &RunContext,
        fc: &crate::agent::types::ToolCall,
        usage: &mut Option<crate::agent::types::TokenUsage>,
        _user_message: &str,
        _token: CapabilityToken,
    ) -> Result<String, ToolExecutionError> {
        let args_str = serde_json::to_string(&fc.args).unwrap_or_default();
        let mission_id_opt = Some(ctx.mission_id.clone());
        tracing::info!("📦 [tools] Executing tool: {}", fc.name);

        // Retrieve tool from registry (optional early lookup, fallback to MCP wrapper if not found in static registry)
        let tool = match self.state.registry.tool_registry.get(&fc.name) {
            Some(t) => Some(t),
            None => {
                let mcp_host = self.state.registry.mcp_host.clone();
                let has_mcp = {
                    let reg = mcp_host.registry.lock().await;
                    reg.get(&fc.name).is_some()
                } || fc.name.starts_with("mcp_") || self.state.registry.skills.skills.contains_key(&fc.name);

                if has_mcp {
                    Some(Arc::new(McpToolWrapper {
                        name: fc.name.clone(),
                        mcp_host: mcp_host.clone(),
                    }) as Arc<dyn Tool>)
                } else {
                    None
                }
            }
        };
        let is_cacheable = tool.as_ref().map(|t| t.is_cacheable()).unwrap_or(false);
        let is_mutating = tool.as_ref().map(|t| t.is_mutating()).unwrap_or(false);

        // 1. Write-Ahead Log (WAL)
        // We MUST record the intent before execution.
        let _log_id = uuid::Uuid::new_v4().to_string();
        self.state
            .security
            .audit_trail
            .record(
                &ctx.agent_id,
                mission_id_opt.as_deref(),
                ctx.user_id.as_deref(),
                &format!("[INTENT] {}", fc.name),
                &self.state.security.secret_redactor.redact(&args_str),
            )
            .await
            .map_err(|e| {
                ToolExecutionError::AppError(AppError::InternalServerError(format!(
                    "WAL Failure: {}",
                    e
                )))
            })?;

        // 2. Capability Check (CBS)
        // Map tool call to required Permission and verify the token.
        let required_permission = match fc.name.as_str() {
            "read_file" | "get_file_contents" | "read_codebase_file" | "list_files" | "grep_search" | "list_file_symbols" | "get_symbol_body" => {
                let path = extract_path_from_args(&fc.args, &ctx.workspace_root)?
                    .map(|p| p.to_string_lossy().to_string())
                    .unwrap_or_else(|| ".".to_string());
                Permission::FileRead(path)
            }
            "write_file" | "delete_file" => {
                let path = extract_path_from_args(&fc.args, &ctx.workspace_root)?
                    .map(|p| p.to_string_lossy().to_string())
                    .unwrap_or_else(|| ".".to_string());
                Permission::FileWrite(path)
            }
            "execute_shell" => {
                let cmd = fc.args.get("command").and_then(|v| v.as_str()).unwrap_or("");
                Permission::ShellExecute(cmd.to_string())
            }
            "spawn_subagent" | "recruit_specialist" => Permission::SpawnAgent,
            "fetch_url" => {
                let url = fc.args.get("url").and_then(|v| v.as_str()).unwrap_or("");
                Permission::NetworkFetch(url.to_string())
            }
            _ => {
                if is_mutating {
                    Permission::FileWrite(".".to_string())
                } else {
                    Permission::FileRead(".".to_string())
                }
            }
        };

        if !_token.verify(&required_permission) {
            return Err(ToolExecutionError::SecurityBlocked(format!(
                "Capability Token verification failed for tool '{}': missing required permission {:?}",
                fc.name, required_permission
            )));
        }

        // 3. Security Manager (Hierarchy & Policy)
        let sec_mgr = DefaultSecurityManager;
        let validation = sec_mgr.pre_validate(self, ctx, fc).await?;

        // 4. Oversight Check
        if validation.oversight_required {
            self.broadcast_sys(
                &format!(
                    "🔒 Security Gate: '{}' requires explicit approval.",
                    fc.name
                ),
                "warning",
                mission_id_opt.clone(),
            );

            let approved = self
                .submit_oversight(
                    crate::agent::types::ToolCallAudit {
                        id: uuid::Uuid::new_v4().to_string(),
                        agent_id: ctx.agent_id.clone(),
                        mission_id: mission_id_opt.clone(),
                        skill: fc.name.clone(),
                        params: fc.args.clone(),
                        department: ctx.department.clone(),
                        description: validation.oversight_reason,
                        timestamp: chrono::Utc::now().to_rfc3339(),
                    },
                    mission_id_opt.clone(),
                )
                .await
                .map_err(ToolExecutionError::AppError)?;

            if !approved {
                return Err(ToolExecutionError::SecurityBlocked(format!(
                    "Execution of {} REJECTED by Oversight Security Gate",
                    fc.name
                )));
            }
        }

        // 5. Execute with Isolated Context
        let tool_ctx = ToolContext {
            mission_id: ctx.mission_id.clone(),
            agent_id: ctx.agent_id.clone(),
            workspace_root: ctx.workspace_root.clone(),
            fs_adapter: ctx.fs_adapter.clone(),
            state: self.state.clone(),
        };

        let workspace_root_str = tool_ctx.workspace_root.to_string_lossy().to_string();

        if is_cacheable {
            let mut cache = self.state.resources.tool_cache.lock();
            if let Some((cached_output, cached_usage)) = cache.get(&fc.name, &args_str, &workspace_root_str) {
                tracing::info!("🎯 [Cache Hit] Bypassing execution for: {}", fc.name);
                if let Some(u) = cached_usage {
                    self.accumulate_usage(usage, Some(u));
                }
                return Ok(cached_output);
            }
        }

        let mut lease_guard = None;
        if is_mutating {
            if fc.name == "execute_shell" {
                self.state.resources.tool_cache.lock().clear();
            } else if let Some(path) = extract_path_from_args(&fc.args, &tool_ctx.workspace_root)? {
                self.state.resources.tool_cache.lock().invalidate_path(&path);
                let guard = ToolLeaseGuard::acquire(
                    self.state.resources.conflict_manager.clone(),
                    path,
                    ctx.agent_id.clone(),
                )?;
                lease_guard = Some(guard);
            }
        }

        // Execution Loop with Self-Annealing
        let mut retry_count = 0;
        let max_retries = 2;

        let tool_instance = tool.ok_or_else(|| {
            ToolExecutionError::ToolNotFound { name: fc.name.clone() }
        })?;

        loop {
            let result = tool_instance.execute(&tool_ctx, fc.args.clone(), usage).await;

            match result {
                Ok(res) => {
                    if let Some(mut guard) = lease_guard {
                        guard.release();
                    }
                    if is_mutating {
                        if let Ok(Some(path)) = extract_path_from_args(&fc.args, &tool_ctx.workspace_root) {
                            let path_str = path.to_string_lossy().to_string();
                            let mut mod_files = ctx.modified_files.lock();
                            if !mod_files.contains(&path_str) {
                                mod_files.push(path_str);
                            }
                        }
                    }
                    if is_cacheable {
                        let mut cache = self.state.resources.tool_cache.lock();
                        let file_path = extract_path_from_args(&fc.args, &tool_ctx.workspace_root)?;
                        cache.insert(
                            &fc.name,
                            &args_str,
                            &workspace_root_str,
                            res.clone(),
                            usage.as_ref().cloned(),
                            file_path,
                        );
                    }
                    // Record successful completion in audit trail
                    let _ = self
                        .state
                        .security
                        .audit_trail
                        .record(
                            &ctx.agent_id,
                            mission_id_opt.as_deref(),
                            ctx.user_id.as_deref(),
                            &format!("[SUCCESS] {}", fc.name),
                            "Execution completed successfully",
                        )
                        .await;
                    return Ok(res);
                }
                Err(e) if e.is_transient() && retry_count < max_retries => {
                    retry_count += 1;
                    tokio::time::sleep(std::time::Duration::from_millis(500 * retry_count)).await;
                    continue;
                }
                Err(e) => {
                    if let Some(mut guard) = lease_guard {
                        guard.release();
                    }
                    // Record failure in audit trail (redacted)
                    let redacted_error = self.state.security.secret_redactor.redact(&format!("Error: {}", e));
                    let _ = self
                        .state
                        .security
                        .audit_trail
                        .record(
                            &ctx.agent_id,
                            mission_id_opt.as_deref(),
                            ctx.user_id.as_deref(),
                            &format!("[FAILURE] {}", fc.name),
                            &redacted_error,
                        )
                        .await;
                    return Err(e);
                }
            }
        }
    }

    /// Handles execution of dynamic file-based skills via the MCP Host.
    ///
    /// ### 🚀 Dynamic Lifecycle
    /// - **Verification**: If the skill defines a `verification_script`, it is run
    ///   immediately after tool completion to validate the "Physical Reality"
    ///   matches the tool's intended effect.
    /// - **Sanitization**: All output is passed through the `Sanitizer` to prevent
    ///   secret leakage or terminal escape sequences.
    #[allow(dead_code)]
    async fn handle_dynamic_skill(
        &self,
        ctx: &RunContext,
        fc: &crate::agent::types::ToolCall,
        output_text: &mut String,
        skill: &crate::agent::script_skills::SkillDefinition,
        _usage: &mut Option<crate::agent::types::TokenUsage>,
    ) -> Result<(), AppError> {
        let result = self
            .state
            .registry
            .mcp_host
            .call_tool(
                &skill.name,
                fc.args.clone(),
                ctx.workspace_root.clone(),
                &self.state.registry.skills.skills,
            )
            .await;

        match result {
            Ok(crate::agent::mcp::McpResult::Raw(output)) => {
                // 🛡️ [Security] Sanitization Hook
                if let crate::agent::sanitizer::SanitizationResult::Alert(msg) =
                    crate::agent::sanitizer::Sanitizer::scan(&output)
                {
                    *output_text = format!("(TOOL EXECUTION HALTED FOR SECURITY: {})", msg);
                    return Ok(());
                }

                let mut final_output = output;
                if let Some(verify_script) = &skill.verification_script {
                    match self
                        .run_verification_script(
                            verify_script,
                            &skill.name,
                            &fc.args,
                            &final_output,
                            &ctx.workspace_root,
                        )
                        .await
                    {
                        Ok(verify_res) => {
                            final_output = format!(
                                "{}\n\n[VERIFICATION STATUS]:\n{}",
                                final_output, verify_res
                            );
                        }
                        Err(e) => {
                            final_output =
                                format!("{}\n\n[VERIFICATION CRITICAL ERROR]: {}", final_output, e);
                        }
                    }
                }

                *output_text = format!(
                    "({} EXECUTED SUCCESSFULLY):\n\n{}",
                    skill.name, final_output
                );
            }
            Ok(crate::agent::mcp::McpResult::SystemDelegate(name, args))
                if name == "recruit_specialist" =>
            {
                let mut mapped_args = serde_json::Map::new();
                if let Some(aid) = args.get("agent_id") {
                    mapped_args.insert("agent_id".to_string(), aid.clone());
                }
                if let Some(msg) = args.get("task_description") {
                    mapped_args.insert("message".to_string(), msg.clone());
                }

                let mapped_fc = crate::agent::types::ToolCall {
                    name: "spawn_subagent".to_string(),
                    args: serde_json::Value::Object(mapped_args),
                };
                let res = self
                    .handle_spawn_subagent(ctx, &mapped_fc, _usage)
                    .await
                    .map_err(|e| match e {
                        ToolExecutionError::AppError(ae) => ae,
                        _ => AppError::InternalServerError(e.to_string()),
                    })?;
                output_text.push_str(&res);
            }
            Ok(crate::agent::mcp::McpResult::SystemDelegate(_, _)) => {
                // Handle other delegates if any
            }
            Err(e) => {
                *output_text = format!("(SKILL EXEC FAILED: {})", e);
            }
        }
        Ok(())
    }

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

        if let Some(mut entry) = self.state.registry.agents.get_mut(&ctx.agent_id) {
            let agent = entry.value_mut();

            // If both are objects, we perform a shallow merge. Otherwise, full overwrite.
            if let (Some(old_obj), Some(new_obj)) = (
                agent.state.working_memory.as_object_mut(),
                new_memory.as_object(),
            ) {
                for (k, v) in new_obj {
                    old_obj.insert(k.clone(), v.clone());
                }
            } else {
                agent.state.working_memory = new_memory;
            }

            let agent_data = agent.clone();
            drop(entry); // Release DashMap lock

            // Sync to DB
            crate::agent::persistence::save_agent_db(&self.state.resources.pool, &agent_data)
                .await?;

            self.state.emit_event(serde_json::json!({
                "type": "agent:update",
                "data": agent_data
            }));

            *output_text = "(WORKING MEMORY UPDATED SUCCESSFULLY)".to_string();
        } else {
            *output_text =
                "(ERROR: Agent not found in registry during working memory update)".to_string();
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

        output_text.push_str("\n--- BATCH EXECUTION STARTED ---\n");

        for (i, step) in steps.iter().enumerate() {
            let tool_name = step
                .get("tool")
                .and_then(|v| v.as_str())
                .ok_or_else(|| AppError::BadRequest(format!("Step {} missing 'tool' name", i)))?;
            let params = step
                .get("params")
                .cloned()
                .unwrap_or(serde_json::Value::Object(serde_json::Map::new()));

            let mut step_output = String::new();
            let step_fc = crate::agent::types::ToolCall {
                name: tool_name.to_string(),
                args: params,
            };

            tracing::info!("📦 [ScriptBuilder] Executing step {}: {}", i + 1, tool_name);
            output_text.push_str(&format!("\n[Step {}: {}]\n", i + 1, tool_name));

            // Execute the individual tool
            let _ = std::pin::Pin::from(Box::new(self.execute_tool(
                ctx,
                &step_fc,
                &mut step_output,
                usage,
                user_message,
            )))
            .await?;

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
            *output_text = format!("(Shell execution REJECTED by Oversight) {}", output_text);
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
            Ok(child) => {
                {
                    let mut cmds = ctx.commands_run.lock();
                    let cmd_str = command_str.to_string();
                    if !cmds.contains(&cmd_str) {
                        cmds.push(cmd_str);
                    }
                }
                let output = child
                    .wait_with_output()
                    .await
                    .map_err(|e: std::io::Error| AppError::Io(e))?;
                let stdout = String::from_utf8_lossy(&output.stdout);
                let stderr = String::from_utf8_lossy(&output.stderr);

                let combined = format!("{}{}", stdout, stderr);
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::constants::*;
    use crate::agent::types::{EngineAgent, ToolCall};
    use crate::state::AppState;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_execute_tool_cbs_block() {
        let state = Arc::new(AppState::new_minimal_mock().await);
        let runner = AgentRunner::new(state.clone());
        let mut ctx = RunContext::default();
        ctx.agent_id = "worker-1".to_string();

        let mut agent = EngineAgent::default();
        agent.identity.id = ctx.agent_id.clone();
        agent.capabilities.skills = vec!["allowed_skill".to_string()];
        state.registry.agents.insert(ctx.agent_id.clone(), agent);

        let fc = ToolCall {
            name: "forbidden_skill".to_string(),
            args: serde_json::json!({}),
        };

        let mut output = String::new();
        let mut usage = None;
        let result = runner
            .execute_tool(&ctx, &fc, &mut output, &mut usage, "")
            .await;
        println!("DEBUG OUTPUT: {}", output);

        assert!(result.is_ok());
        assert!(
            output.contains("Security Violation: Skill 'forbidden_skill' not in agent allowlist")
        );
        assert!(output.contains("| RECOVERY: Escalate"));
    }

    #[tokio::test]
    async fn test_execute_tool_hierarchy_block() {
        let state = Arc::new(AppState::new_minimal_mock().await);
        let runner = AgentRunner::new(state.clone());
        let mut ctx = RunContext::default();
        ctx.agent_id = AGENT_CEO.to_string();
        ctx.authority_level = crate::agent::types::RoleAuthorityLevel::Executive;

        let mut agent = EngineAgent::default();
        agent.identity.id = ctx.agent_id.clone();
        state.registry.agents.insert(ctx.agent_id.clone(), agent);

        let fc = ToolCall {
            name: "spawn_subagent".to_string(),
            args: serde_json::json!({"agent_id": "worker"}),
        };

        let mut output = String::new();
        let mut usage = None;
        let result = runner
            .execute_tool(&ctx, &fc, &mut output, &mut usage, "")
            .await;

        assert!(result.is_ok());
        assert!(output.contains(
            "Hierarchy Violation: As CEO, you are prohibited from direct worker recruitment."
        ));
        assert!(output.contains("| RECOVERY: Escalate"));
    }

    #[tokio::test]
    async fn test_execute_tool_policy_deny() {
        let state = Arc::new(AppState::new_minimal_mock().await);
        let runner = AgentRunner::new(state.clone());
        let ctx = RunContext::default();

        // Set policy to Deny for a specific tool
        state
            .security
            .permission_policy
            .set_mode(
                "risky_tool",
                crate::security::permissions::PermissionMode::Deny,
            )
            .await;

        let fc = ToolCall {
            name: "risky_tool".to_string(),
            args: serde_json::json!({}),
        };

        let mut output = String::new();
        let mut usage = None;
        let result = runner
            .execute_tool(&ctx, &fc, &mut output, &mut usage, "")
            .await;

        assert!(result.is_ok());
        assert!(output.contains("Security Violation: Policy for 'risky_tool' is set to DENY"));
        assert!(output.contains("| RECOVERY: Escalate"));
    }

    #[tokio::test]
    async fn test_update_working_memory() {
        let state = Arc::new(AppState::new_minimal_mock().await);
        let runner = AgentRunner::new(state.clone());
        let mut ctx = RunContext::default();
        ctx.agent_id = "memory-agent".to_string();

        let mut agent = EngineAgent::default();
        agent.identity.id = ctx.agent_id.clone();
        state.registry.agents.insert(ctx.agent_id.clone(), agent);

        // Ensure agent exists in DB for persistence call
        crate::agent::persistence::save_agent_db(
            &state.resources.pool,
            &state.registry.agents.get(&ctx.agent_id).unwrap(),
        )
        .await
        .unwrap();

        let fc = ToolCall {
            name: "update_working_memory".to_string(),
            args: serde_json::json!({"memory": {"last_step": "initialized"}}),
        };

        let mut output = String::new();
        let result = runner
            .handle_update_working_memory(&ctx, &fc, &mut output)
            .await;

        assert!(result.is_ok());
        let agent = state.registry.agents.get(&ctx.agent_id).unwrap();
        assert_eq!(agent.state.working_memory["last_step"], "initialized");
    }
}

struct McpToolWrapper {
    name: String,
    mcp_host: Arc<crate::agent::mcp::McpHost>,
}

#[async_trait::async_trait]
impl Tool for McpToolWrapper {
    fn metadata(&self) -> crate::agent::runner::tools::trait_tool::ToolDefinitionData {
        crate::agent::runner::tools::trait_tool::ToolDefinitionData {
            name: self.name.clone(),
            description: "Dynamic MCP Tool wrapper".to_string(),
            parameters: serde_json::json!({}),
        }
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn is_cacheable(&self) -> bool {
        matches!(
            self.name.as_str(),
            "list_file_symbols" | "get_symbol_body"
        )
    }

    fn is_mutating(&self) -> bool {
        !matches!(
            self.name.as_str(),
            "list_file_symbols" | "get_symbol_body" | "inspect_engine_health"
        )
    }

    fn is_dangerous(&self) -> bool {
        self.is_mutating()
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
                    Ok(format!("System delegate '{}' executed", name))
                }
            }
        }
    }
}

fn extract_path_from_args(
    args: &serde_json::Value,
    workspace_root: &std::path::Path,
) -> Result<Option<std::path::PathBuf>, ToolExecutionError> {
    if let serde_json::Value::Object(map) = args {
        let path_val = map.get("path")
            .or_else(|| map.get("file"))
            .or_else(|| map.get("target_path"))
            .or_else(|| map.get("SearchPath"))
            .or_else(|| map.get("DirectoryPath"))
            .or_else(|| map.get("AbsolutePath"));

        if let Some(v) = path_val {
            if let Some(path_str) = v.as_str() {
                let safe_path = crate::utils::security::validate_path(workspace_root, path_str)
                    .map_err(|e| ToolExecutionError::SecurityBlocked(format!("Path validation failed: {}", e)))?;
                return Ok(Some(safe_path.to_path_buf()));
            }
        }
    }
    Ok(None)
}

// Metadata: [tools]

// Metadata: [mod]

// Metadata: [mod]

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
//!

pub mod cache;
pub mod capability;
pub mod dispatcher;
pub mod error;
pub mod manifest;
pub mod plugin;
pub mod registry;
pub mod security;
pub mod trait_tool;
#[macro_use]
pub mod macros;

// Decomposed submodules
pub mod lease;
pub mod mcp_wrapper;
pub mod policy;
pub mod system_tools;
pub mod validation;

use crate::error::AppError;
use security::{DefaultSecurityManager, SecurityManager};

pub use capability::{CapabilityToken, ZeroTrustGuard};
pub use trait_tool::{Tool, ToolContext};

use super::{AgentRunner, RunContext};
use error::ToolExecutionError;
use std::sync::Arc;

pub use lease::ToolLeaseGuard;
pub(crate) use mcp_wrapper::McpToolWrapper;
use policy::resolve_required_permission;
use validation::{extract_path_from_args, validate_json_schema};

#[async_trait::async_trait]
impl super::service_traits::ToolExecutor for AgentRunner {
    async fn execute_tool(
        &self,
        ctx: &RunContext,
        fc: &crate::agent::types::ToolCall,
        user_message: &str,
    ) -> Result<(String, Option<crate::agent::types::TokenUsage>), AppError> {
        // 1. Mint Capability Token for this specific call
        let token = ZeroTrustGuard::mint_token(
            &ctx.agent_id,
            &ctx.mission_id,
            ctx.authority_level,
            ctx.allowed_files.as_deref(),
            &ctx.workspace_root,
        );

        let mut local_usage = None;
        match self
            .run_zero_trust_pipeline(ctx, fc, &mut local_usage, user_message, token)
            .await
        {
            Ok(output) => {
                if output.len() > 1024 * 1024 {
                    // 1MB DoS Safety Check (Fix 6)
                    let error_msg = format!(
                        "(TOOL FAILURE: Output size {} bytes exceeded the 1MB safety limit)",
                        output.len()
                    );
                    return Ok((error_msg, local_usage));
                }
                let redacted = self.state.security.secret_redactor.redact(&output);
                Ok((redacted, local_usage))
            }
            Err(e) => {
                let recovery = e.recovery_strategy();
                tracing::error!(
                    "Tool '{}' failed for agent {}: {} (recovery: {:?})",
                    fc.name,
                    ctx.agent_id,
                    e,
                    recovery
                );
                // SAFETY: `user_safe_message()` is an exhaustive match on all
                // `ToolExecutionError` variants (see error.rs L101-L123). Sensitive
                // variants (`ExecutionFailed`, `CommandFailed`, generic `AppError`)
                // return hardcoded strings. User-controlled fields pass through
                // `sanitize_for_llm()` which strips non-alphanumeric chars and
                // truncates to 200 chars. The `secret_redactor` provides a second
                // defense layer. Both are tested (`test_user_safe_message_sanitized`,
                // `test_sanitize_for_llm_strips_injection`).
                let error_msg = format!(
                    "(TOOL FAILURE: {} | RECOVERY: {:?})",
                    e.user_safe_message(),
                    recovery
                );
                let redacted = self.state.security.secret_redactor.redact(&error_msg);

                // Return Ok to surface structured failure info to the agent
                // for self-annealing. Callers that need to distinguish success/failure
                // should check output_text for the "(TOOL FAILURE:" prefix.
                Ok((redacted, local_usage))
            }
        }
    }

    fn update_status(&self, agent_id: &str, mission_id: &str, status: &str, task: Option<&str>) {
        self.update_status(agent_id, mission_id, status, task);
    }

    fn handle_tool_failure_refinement(
        &self,
        ctx: &RunContext,
        fc: &crate::agent::types::ToolCall,
        local_text: &mut String,
    ) {
        self.handle_tool_failure_refinement(ctx, fc, local_text);
    }

    fn accumulate_usage(
        &self,
        accumulated: &mut Option<crate::agent::types::TokenUsage>,
        new_usage: Option<crate::agent::types::TokenUsage>,
    ) {
        self.accumulate_usage(accumulated, new_usage);
    }

    fn verify_mission_success(&self, observation_buffer: &str) -> bool {
        self.verify_mission_success(observation_buffer)
    }

    fn broadcast_agent(&self, ctx: &RunContext, msg: &str, level: &str) {
        self.broadcast_agent(ctx, msg, level);
    }

    fn handle_tool_failure_slot_swap(&self, agent_id: &str) -> Option<String> {
        if let Some(mut entry) = self.state.registry.agents.get_mut(agent_id) {
            let agent = entry.value_mut();
            let current_slot = agent
                .models
                .active_model_slot
                .as_deref()
                .unwrap_or("default");
            let next_slot = match current_slot {
                "execution" => "planning",
                "planning" => "execution",
                _ => "planning",
            };
            tracing::info!("🔄 [Builder-Debugger Swap] Switching agent '{}' active model slot from '{}' to '{}' due to tool failure", agent_id, current_slot, next_slot);
            agent.models.active_model_slot = Some(next_slot.to_string());
            Some(next_slot.to_string())
        } else {
            None
        }
    }
}

impl AgentRunner {
    /// Manages the Zero-Trust sequence (WAL -> CBS -> Execute)
    async fn run_zero_trust_pipeline(
        &self,
        ctx: &RunContext,
        fc: &crate::agent::types::ToolCall,
        usage: &mut Option<crate::agent::types::TokenUsage>,
        user_message: &str,
        token: CapabilityToken,
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
                } || fc.name.starts_with("mcp_")
                    || self.state.registry.skills.skills.contains_key(&fc.name);

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

        if self.state.mirror_mode && is_mutating {
            let alert_id = uuid::Uuid::new_v4().to_string();
            let timestamp = chrono::Utc::now().to_rfc3339();
            let alert_value = serde_json::json!({
                "id": alert_id,
                "timestamp": timestamp,
                "agent_id": ctx.agent_id,
                "mission_id": ctx.mission_id,
                "alert_type": "mutating_tool_blocked",
                "detail": format!("Blocked mutating tool call '{}' with arguments: {}", fc.name, args_str)
            });
            self.state
                .drift_alerts
                .insert(alert_id, alert_value.clone());
            self.state.emit_event(serde_json::json!({
                "type": "oversight:drift",
                "alert": alert_value
            }));
            tracing::warn!(
                "⚠️  [Mirror Mode] Blocked mutating tool '{}' execution. Drift alert registered.",
                fc.name
            );
            return Ok(format!(
                "(PASSIVE MIRROR MODE ENFORCED: Blocked mutating execution of tool '{}'. Registered drift alert.)",
                fc.name
            ));
        }

        // 1. Write-Ahead Log (WAL)
        // We MUST record the intent before execution.
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
        let required_permission =
            resolve_required_permission(&fc.name, &fc.args, &ctx.workspace_root, is_mutating)?;

        if !token.verify(&required_permission) {
            return Err(ToolExecutionError::SecurityBlocked(format!(
                "Capability Token verification failed for tool '{}': missing required permission {:?}",
                fc.name, required_permission
            )));
        }

        // 3. Security Manager & Oversight Gates
        self.validate_security_and_oversight(ctx, fc, user_message)
            .await?;

        // 4. Execute with Isolated Context
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
            if let Some((cached_output, cached_usage)) =
                cache.get(&fc.name, &args_str, &workspace_root_str)
            {
                tracing::info!("🎯 [Cache Hit] Bypassing execution for: {}", fc.name);
                if let Some(u) = cached_usage {
                    self.accumulate_usage(usage, Some(u));
                }
                return Ok(cached_output);
            }
        }

        // Compute target path once — avoids 3× JSON walks + path validation per call
        let target_path = extract_path_from_args(&fc.args, &tool_ctx.workspace_root)?;

        let mut lease_guard = None;
        if is_mutating {
            if fc.name == "execute_shell" {
                self.state.resources.tool_cache.lock().clear();
            } else if let Some(ref path) = target_path {
                self.state.resources.tool_cache.lock().invalidate_path(path);
                let guard = ToolLeaseGuard::acquire(
                    self.state.resources.conflict_manager.clone(),
                    path.clone(),
                    ctx.agent_id.clone(),
                )?;
                lease_guard = Some(guard);
            }
        }

        let tool_instance = tool.ok_or_else(|| ToolExecutionError::ToolNotFound {
            name: fc.name.clone(),
        })?;

        // Validate arguments against parameters JSON schema
        if let Err(e) = validate_json_schema(
            &tool_instance_metadata(&Some(tool_instance.clone())),
            &fc.args,
        ) {
            tracing::warn!(
                "🛡️ [Schema Validation] Argument validation failed for tool '{}': {}",
                fc.name,
                e
            );
            return Err(ToolExecutionError::SecurityBlocked(format!(
                "Argument validation failed: {}",
                e
            )));
        }

        self.execute_with_retry(
            tool_instance,
            &tool_ctx,
            fc,
            usage,
            lease_guard,
            is_mutating,
            is_cacheable,
            &target_path,
            &args_str,
            &workspace_root_str,
            &mission_id_opt,
            &ctx.agent_id,
            ctx.user_id.as_deref(),
            &ctx.modified_files,
        )
        .await
    }

    /// Executes the tool with retries on transient errors and registers caching/audit state.
    #[allow(clippy::too_many_arguments)]
    async fn execute_with_retry(
        &self,
        tool_instance: Arc<dyn Tool>,
        tool_ctx: &ToolContext,
        fc: &crate::agent::types::ToolCall,
        usage: &mut Option<crate::agent::types::TokenUsage>,
        lease_guard: Option<ToolLeaseGuard>,
        is_mutating: bool,
        is_cacheable: bool,
        target_path: &Option<std::path::PathBuf>,
        args_str: &str,
        workspace_root_str: &str,
        mission_id_opt: &Option<String>,
        agent_id: &str,
        user_id: Option<&str>,
        modified_files: &parking_lot::Mutex<Vec<String>>,
    ) -> Result<String, ToolExecutionError> {
        let mut retry_count = 0;
        // Raised to 5 to give SQLite transient errors (BUSY/LOCKED) sufficient retry budget
        // during high-concurrency swarm spawning. Backoff: 500ms * retry_count = max ~7.5s total.
        let max_retries = 5;

        loop {
            let result = tool_instance
                .execute(tool_ctx, fc.args.clone(), usage)
                .await;

            match result {
                Ok(res) => {
                    if let Some(mut guard) = lease_guard {
                        guard.release();
                    }
                    if is_mutating {
                        if let Some(ref path) = target_path {
                            let path_str = path.to_string_lossy().to_string();
                            let mut mod_files = modified_files.lock();
                            if !mod_files.contains(&path_str) {
                                mod_files.push(path_str);
                            }
                        }
                    }
                    if is_cacheable {
                        let mut cache = self.state.resources.tool_cache.lock();
                        cache.insert(
                            &fc.name,
                            args_str,
                            workspace_root_str,
                            res.clone(),
                            usage.as_ref().cloned(),
                            target_path.clone(),
                        );
                    }
                    // Record successful completion in audit trail
                    if let Err(err) = self
                        .state
                        .security
                        .audit_trail
                        .record(
                            agent_id,
                            mission_id_opt.as_deref(),
                            user_id,
                            &format!("[SUCCESS] {}", fc.name),
                            "Execution completed successfully",
                        )
                        .await
                    {
                        tracing::error!("Failed to record tool success in audit trail: {}", err);
                    }
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
                    let redacted_error = self
                        .state
                        .security
                        .secret_redactor
                        .redact(&format!("Error: {}", e));
                    if let Err(err) = self
                        .state
                        .security
                        .audit_trail
                        .record(
                            agent_id,
                            mission_id_opt.as_deref(),
                            user_id,
                            &format!("[FAILURE] {}", fc.name),
                            &redacted_error,
                        )
                        .await
                    {
                        tracing::error!("Failed to record tool failure in audit trail: {}", err);
                    }
                    return Err(e);
                }
            }
        }
    }

    /// Evaluates and executes the security policy and human-in-the-loop oversight gates.
    async fn validate_security_and_oversight(
        &self,
        ctx: &RunContext,
        fc: &crate::agent::types::ToolCall,
        user_message: &str,
    ) -> Result<(), ToolExecutionError> {
        let sec_mgr = DefaultSecurityManager;
        let validation = sec_mgr.pre_validate(self, ctx, fc).await?;
        let mission_id_opt = Some(ctx.mission_id.clone());

        if validation.oversight_required {
            self.broadcast_sys(
                &format!(
                    "🔒 Security Gate: '{}' requires explicit approval.",
                    fc.name
                ),
                "warning",
                mission_id_opt.clone(),
            );

            let description = if user_message.is_empty() {
                validation.oversight_reason
            } else {
                let raw = format!(
                    "User Context: '{}' | Reason: {}",
                    user_message, validation.oversight_reason
                );
                self.state.security.secret_redactor.redact(&raw)
            };

            let approved = self
                .submit_oversight(
                    crate::agent::types::ToolCallAudit {
                        id: uuid::Uuid::new_v4().to_string(),
                        agent_id: ctx.agent_id.clone(),
                        mission_id: mission_id_opt.clone(),
                        skill: fc.name.clone(),
                        params: fc.args.clone(),
                        department: ctx.department.clone(),
                        description,
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
        Ok(())
    }
}

// Helper to avoid borrow issues on tool_instance before its extraction
fn tool_instance_metadata(tool: &Option<Arc<dyn Tool>>) -> serde_json::Value {
    if let Some(t) = tool {
        t.metadata().parameters
    } else {
        serde_json::Value::Null
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::constants::*;
    use crate::agent::runner::tools::capability::Permission;
    use crate::agent::types::{EngineAgent, ToolCall};
    use crate::state::AppState;
    use policy::*;
    use std::sync::Arc;
    use validation::validate_json_schema;

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

        let token = ZeroTrustGuard::mint_token(
            &ctx.agent_id,
            &ctx.mission_id,
            ctx.authority_level,
            ctx.allowed_files.as_deref(),
            &ctx.workspace_root,
        );

        let mut usage = None;
        let result = runner
            .run_zero_trust_pipeline(&ctx, &fc, &mut usage, "", token)
            .await;

        assert!(matches!(
            result,
            Err(ToolExecutionError::SecurityBlocked(ref msg)) if msg.contains("not in agent allowlist")
        ));
    }

    #[tokio::test]
    async fn test_execute_tool_mirror_mode() {
        std::env::set_var("MIRROR_MODE", "true");
        let state = Arc::new(AppState::new_minimal_mock().await);
        std::env::remove_var("MIRROR_MODE");

        assert!(state.mirror_mode);

        let runner = AgentRunner::new(state.clone());
        let mut ctx = RunContext::default();
        ctx.agent_id = "worker-1".to_string();

        let mut agent = EngineAgent::default();
        agent.identity.id = ctx.agent_id.clone();
        agent.capabilities.skills = vec!["write_file".to_string()];
        state.registry.agents.insert(ctx.agent_id.clone(), agent);

        let fc = ToolCall {
            name: "write_file".to_string(),
            args: serde_json::json!({
                "path": "test_file.txt",
                "content": "hello"
            }),
        };

        let token = ZeroTrustGuard::mint_token(
            &ctx.agent_id,
            &ctx.mission_id,
            ctx.authority_level,
            ctx.allowed_files.as_deref(),
            &ctx.workspace_root,
        );

        let mut usage = None;
        let result = runner
            .run_zero_trust_pipeline(&ctx, &fc, &mut usage, "", token)
            .await;

        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("PASSIVE MIRROR MODE ENFORCED"));

        assert_eq!(state.drift_alerts.len(), 1);
        let alert = state.drift_alerts.iter().next().unwrap();
        assert_eq!(alert.value()["alert_type"], "mutating_tool_blocked");
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

        let token = ZeroTrustGuard::mint_token(
            &ctx.agent_id,
            &ctx.mission_id,
            ctx.authority_level,
            ctx.allowed_files.as_deref(),
            &ctx.workspace_root,
        );

        let mut usage = None;
        let result = runner
            .run_zero_trust_pipeline(&ctx, &fc, &mut usage, "", token)
            .await;

        assert!(matches!(
            result,
            Err(ToolExecutionError::HierarchyBlocked(ref msg)) if msg.contains("prohibited from direct worker recruitment")
        ));
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

        let token = ZeroTrustGuard::mint_token(
            &ctx.agent_id,
            &ctx.mission_id,
            ctx.authority_level,
            ctx.allowed_files.as_deref(),
            &ctx.workspace_root,
        );

        let mut usage = None;
        let result = runner
            .run_zero_trust_pipeline(&ctx, &fc, &mut usage, "", token)
            .await;

        assert!(matches!(
            result,
            Err(ToolExecutionError::SecurityBlocked(ref msg)) if msg.contains("is set to DENY")
        ));
    }

    #[tokio::test]
    async fn test_update_working_memory() {
        let state = Arc::new(AppState::new_minimal_mock().await);
        let runner = AgentRunner::new(state.clone());
        let mut ctx = RunContext::default();
        ctx.agent_id = "memory-agent".to_string();

        let mut agent = EngineAgent::default();
        agent.identity.id = ctx.agent_id.clone();

        // Ensure agent exists in DB for persistence call
        crate::agent::persistence::save_agent_db(&state.resources.pool, &mut agent)
            .await
            .unwrap();

        // Reload to sync version between DB and registry
        let agents = crate::agent::persistence::load_agents_db(&state.resources.pool)
            .await
            .unwrap();
        let agent = agents
            .into_iter()
            .find(|a| a.identity.id == ctx.agent_id)
            .unwrap();

        state.registry.agents.insert(ctx.agent_id.clone(), agent);

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

    #[tokio::test]
    async fn test_unknown_mutating_tool_default_denied() {
        let state = Arc::new(AppState::new_minimal_mock().await);
        let runner = AgentRunner::new(state.clone());
        let mut ctx = RunContext::default();
        ctx.agent_id = "test-agent".to_string();

        let mut agent = EngineAgent::default();
        agent.identity.id = ctx.agent_id.clone();
        agent.capabilities.skills = vec!["mcp_git_create_repository".to_string()];
        state.registry.agents.insert(ctx.agent_id.clone(), agent);

        let fc = ToolCall {
            name: "mcp_git_create_repository".to_string(), // Mutating tool name
            args: serde_json::json!({}),
        };

        let token = ZeroTrustGuard::mint_token(
            &ctx.agent_id,
            &ctx.mission_id,
            ctx.authority_level,
            ctx.allowed_files.as_deref(),
            &ctx.workspace_root,
        );

        let mut usage = None;
        let result = runner
            .run_zero_trust_pipeline(&ctx, &fc, &mut usage, "", token)
            .await;

        assert!(
            matches!(
                &result,
                Err(ToolExecutionError::SecurityBlocked(ref msg)) if msg.contains("Mutating permission is denied by default")
            ) || matches!(
                &result,
                Ok(ref msg) if msg.contains("PASSIVE MIRROR MODE ENFORCED")
            ),
            "Expected mutating tool to be blocked, but got: {:?}",
            result
        );
    }

    #[tokio::test]
    async fn test_unknown_non_mutating_tool_allowed_past_gates() {
        let state = Arc::new(AppState::new_minimal_mock().await);
        let runner = AgentRunner::new(state.clone());
        let mut ctx = RunContext::default();
        ctx.agent_id = "test-agent".to_string();

        let mut agent = EngineAgent::default();
        agent.identity.id = ctx.agent_id.clone();
        agent.capabilities.skills = vec!["mcp_docs_query_docs".to_string()];
        state.registry.agents.insert(ctx.agent_id.clone(), agent);

        let fc = ToolCall {
            name: "mcp_docs_query_docs".to_string(), // Non-mutating tool name
            args: serde_json::json!({}),
        };

        let token = ZeroTrustGuard::mint_token(
            &ctx.agent_id,
            &ctx.mission_id,
            ctx.authority_level,
            ctx.allowed_files.as_deref(),
            &ctx.workspace_root,
        );

        let mut usage = None;
        let result = runner
            .run_zero_trust_pipeline(&ctx, &fc, &mut usage, "", token)
            .await;

        assert!(
            matches!(
                result,
                Err(ToolExecutionError::ToolNotFound { ref name }) if name == "mcp_docs_query_docs"
            ) || matches!(result, Err(ToolExecutionError::AppError(_)))
        );
    }

    #[test]
    fn test_is_mutating_mcp_prefix_strip() {
        // Standard MCP prefixed tool: strip "mcp_{server}_" to get operation
        assert!(is_mutating_tool_name("mcp_git_create_repository"));
        assert!(is_mutating_tool_name("mcp_github_push_files"));
        assert!(is_mutating_tool_name("mcp_github_merge_pull_request"));

        // Short MCP name with no operation segment: falls back to full name, not in list
        assert!(!is_mutating_tool_name("mcp_repository"));

        // Non-MCP tools: matched directly
        assert!(is_mutating_tool_name("write_file"));
        assert!(is_mutating_tool_name("delete_file"));
        assert!(is_mutating_tool_name("execute_shell"));

        // Non-mutating tools
        assert!(!is_mutating_tool_name("read_file"));
        assert!(!is_mutating_tool_name("grep_search"));
        assert!(!is_mutating_tool_name("mcp_docs_query_docs"));
    }

    #[test]
    fn test_is_dangerous_mcp_reads_not_flagged() {
        // Benign MCP read tools should NOT be dangerous
        assert!(!is_dangerous_mcp_operation("mcp_docs_query_docs"));
        assert!(!is_dangerous_mcp_operation(
            "mcp_context7_resolve_library_id"
        ));

        // Non-MCP tools should not be flagged by this function
        assert!(!is_dangerous_mcp_operation("read_file"));
        assert!(!is_dangerous_mcp_operation("grep_search"));
    }

    #[test]
    fn test_is_dangerous_mcp_mutating_flagged() {
        // Mutating MCP tools should be dangerous
        assert!(is_dangerous_mcp_operation("mcp_git_create_repository"));
        assert!(is_dangerous_mcp_operation("mcp_github_push_files"));

        // GitHub data-access endpoints should be dangerous
        assert!(is_dangerous_mcp_operation("mcp_github_search_code"));
        assert!(is_dangerous_mcp_operation("mcp_github_get_file_contents"));
        assert!(is_dangerous_mcp_operation("mcp_github_get_issue"));
        assert!(is_dangerous_mcp_operation("mcp_github_search_repositories"));
    }

    #[test]
    fn test_resolve_permission_unknown_non_mutating() {
        let root = std::path::Path::new("/workspace");
        let args = serde_json::json!({});

        // Unknown non-mutating tool should get ToolExec, NOT FileRead(".")
        let perm = resolve_required_permission("some_unknown_tool", &args, root, false).unwrap();
        assert_eq!(perm, Permission::ToolExec("some_unknown_tool".to_string()));

        // Unknown mutating tool should be blocked
        let perm_err = resolve_required_permission("some_unknown_tool", &args, root, true);
        assert!(perm_err.is_err());
    }

    #[test]
    fn test_is_dangerous_native_operation() {
        // Native tools with security risk
        assert!(is_dangerous_native_operation("request_model_switch"));
        assert!(is_dangerous_native_operation("fetch_url"));
        assert!(is_dangerous_native_operation("run_integrity_check"));

        // Should NOT flag mutating tools (those are in is_mutating_tool_name)
        assert!(!is_dangerous_native_operation("write_file"));
        assert!(!is_dangerous_native_operation("execute_shell"));

        // Should NOT flag read tools
        assert!(!is_dangerous_native_operation("read_file"));
        assert!(!is_dangerous_native_operation("grep_search"));

        // Should NOT flag MCP tools (those are in is_dangerous_mcp_operation)
        assert!(!is_dangerous_native_operation("mcp_github_push_files"));
    }

    #[test]
    fn test_dangerous_composition_invariant() {
        // Every mutating tool must also be considered dangerous.
        // This test verifies the composition holds for representative tools.
        let mutating_samples = [
            "write_file",
            "delete_file",
            "execute_shell",
            "spawn_subagent",
            "recruit_specialist",
            "mcp_git_create_repository",
            "mcp_github_push_files",
        ];

        for tool in &mutating_samples {
            assert!(
                is_mutating_tool_name(tool),
                "'{}' should be classified as mutating",
                tool
            );
            // Dangerous is the union of mutating + native-dangerous + mcp-dangerous
            let is_dangerous = is_mutating_tool_name(tool)
                || is_dangerous_native_operation(tool)
                || is_dangerous_mcp_operation(tool);
            assert!(
                is_dangerous,
                "'{}' is mutating but not dangerous — composition broken",
                tool
            );
        }
    }

    #[test]
    fn test_validate_json_schema_basic() {
        let schema = serde_json::json!({
            "type": "object",
            "required": ["path", "content"],
            "properties": {
                "path": { "type": "string" },
                "content": { "type": "string" },
                "overwrite": { "type": "boolean" },
                "lines_count": { "type": "integer" },
                "ratio": { "type": "number" },
                "tags": { "type": "array" },
                "options": { "type": "object" }
            }
        });

        // 1. Success case
        let args = serde_json::json!({
            "path": "/workspace/file.txt",
            "content": "hello world",
            "overwrite": true,
            "lines_count": 42,
            "ratio": 0.75,
            "tags": ["a", "b"],
            "options": { "key": "value" }
        });
        assert!(validate_json_schema(&schema, &args).is_ok());

        // 2. Success case: Float integer (e.g. 42.0)
        let args_float_int = serde_json::json!({
            "path": "/workspace/file.txt",
            "content": "hello world",
            "lines_count": 42.0
        });
        assert!(validate_json_schema(&schema, &args_float_int).is_ok());

        // 3. Fail: Missing required parameter
        let args_missing = serde_json::json!({
            "path": "/workspace/file.txt"
        });
        let res = validate_json_schema(&schema, &args_missing);
        assert!(res.is_err());
        assert!(res
            .unwrap_err()
            .to_string()
            .contains("Missing required parameter"));

        // 4. Fail: Wrong type (string instead of boolean)
        let args_wrong_type = serde_json::json!({
            "path": "/workspace/file.txt",
            "content": "hello world",
            "overwrite": "yes"
        });
        let res2 = validate_json_schema(&schema, &args_wrong_type);
        assert!(res2.is_err());
        assert!(res2
            .unwrap_err()
            .to_string()
            .contains("expects type 'boolean'"));

        // 5. Fail: Wrong type (float with fractional part instead of integer)
        let args_wrong_int = serde_json::json!({
            "path": "/workspace/file.txt",
            "content": "hello world",
            "lines_count": 42.5
        });
        let res3 = validate_json_schema(&schema, &args_wrong_int);
        assert!(res3.is_err());
        assert!(res3
            .unwrap_err()
            .to_string()
            .contains("expects type 'integer'"));
    }
}

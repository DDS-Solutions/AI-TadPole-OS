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
use crate::error::AppError;
use security::{DefaultSecurityManager, SecurityManager};

pub use capability::{CapabilityToken, Permission, ZeroTrustGuard};
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
}#[async_trait::async_trait]
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
                if output.len() > 1024 * 1024 { // 1MB DoS Safety Check (Fix 6)
                    let error_msg = format!("(TOOL FAILURE: Output size {} bytes exceeded the 1MB safety limit)", output.len());
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
                let error_msg = format!("(TOOL FAILURE: {} | RECOVERY: {:?})", e.user_safe_message(), recovery);
                let redacted = self.state.security.secret_redactor.redact(&error_msg);

                // Return Ok to surface structured failure info to the agent
                // for self-annealing. Callers that need to distinguish success/failure
                // should check output_text for the "(TOOL FAILURE:" prefix.
                Ok((redacted, local_usage))
            }
        }
    }

    fn update_status(
        &self,
        agent_id: &str,
        mission_id: &str,
        status: &str,
        task: Option<&str>,
    ) {
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
        let required_permission = resolve_required_permission(
            &fc.name,
            &fc.args,
            &ctx.workspace_root,
            is_mutating,
        )?;

        if !token.verify(&required_permission) {
            return Err(ToolExecutionError::SecurityBlocked(format!(
                "Capability Token verification failed for tool '{}': missing required permission {:?}",
                fc.name, required_permission
            )));
        }

        // 3. Security Manager & Oversight Gates
        self.validate_security_and_oversight(ctx, fc, user_message).await?;

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
                self.state
                    .resources
                    .tool_cache
                    .lock()
                    .invalidate_path(path);
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
        if let Err(e) = validate_json_schema(&tool_instance.metadata().parameters, &fc.args) {
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
        let max_retries = 2;

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

                // If both are objects, we perform a shallow merge. Otherwise, full overwrite.
                if let (Some(old_obj), Some(new_obj)) = (
                    agent.state.working_memory.as_object_mut(),
                    new_memory.as_object(),
                ) {
                    for (k, v) in new_obj {
                        old_obj.insert(k.clone(), v.clone());
                    }
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
                    if let Some(db_agent) = crate::agent::persistence::load_agents_db(&pool)
                        .await?
                        .into_iter()
                        .find(|a| a.identity.id == ctx.agent_id)
                    {
                        let mut updated_agent = db_agent;
                        if let (Some(old_obj), Some(new_obj)) = (
                            updated_agent.state.working_memory.as_object_mut(),
                            new_memory.as_object(),
                        ) {
                            for (k, v) in new_obj {
                                old_obj.insert(k.clone(), v.clone());
                            }
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
                if let Some(db_agent) = crate::agent::persistence::load_agents_db(&pool)
                    .await?
                    .into_iter()
                    .find(|a| a.identity.id == ctx.agent_id)
                {
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
                    if let Some(db_agent) = crate::agent::persistence::load_agents_db(&pool)
                        .await?
                        .into_iter()
                        .find(|a| a.identity.id == ctx.agent_id)
                    {
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
                if let Some(db_agent) = crate::agent::persistence::load_agents_db(&pool)
                    .await?
                    .into_iter()
                    .find(|a| a.identity.id == ctx.agent_id)
                {
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
                AppError::BadRequest("'allowed_tools' must be an array of strings in script_builder".to_string())
            })?
            .iter()
            .filter_map(|v| v.as_str().map(|s| s.to_string()))
            .collect();

        // Validate allowed_tools element shape: only safe identifiers
        for tool_name in &allowed_tools {
            if tool_name.is_empty()
                || tool_name.len() > 64
                || !tool_name
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
            {
                return Err(AppError::BadRequest(format!(
                    "Invalid tool name in allowed_tools: '{}'. Must be 1-64 alphanumeric/underscore/dash characters.",
                    &tool_name[..tool_name.len().min(20)]
                )));
            }
        }

        // Single-pass: validate all steps are in allowed_tools AND detect dangerous tools
        let mut has_dangerous = false;
        for (i, step) in steps.iter().enumerate() {
            let tool_name = step
                .get("tool")
                .and_then(|v| v.as_str())
                .ok_or_else(|| AppError::BadRequest(format!("Step {} missing 'tool' name", i)))?;

            // Validate step["tool"] charset and length to prevent name collision/injection
            if tool_name.is_empty()
                || tool_name.len() > 64
                || !tool_name
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
            {
                return Err(AppError::BadRequest(format!(
                    "Invalid tool name in step {}: '{}'. Must be 1-64 alphanumeric/underscore/dash characters.",
                    i, &tool_name[..tool_name.len().min(20)]
                )));
            }

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
                allowed_tools.iter().take(10).cloned().collect::<Vec<_>>().join(", ")
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
            let (res_text, res_usage) = self
                .execute_tool(ctx, &step_fc, user_message)
                .await?;
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
            Ok(child) => {
                {
                    let mut cmds = ctx.commands_run.lock();
                    cmds.insert(command_str.to_string());
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
        crate::agent::persistence::save_agent_db(
            &state.resources.pool,
            &mut agent,
        )
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

        assert!(matches!(
            result,
            Err(ToolExecutionError::SecurityBlocked(ref msg)) if msg.contains("Mutating permission is denied by default")
        ));
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

        assert!(matches!(
            result,
            Err(ToolExecutionError::ToolNotFound { ref name }) if name == "mcp_docs_query_docs"
        ) || matches!(
            result,
            Err(ToolExecutionError::AppError(_))
        ));
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
        assert!(!is_dangerous_mcp_operation("mcp_context7_resolve_library_id"));

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
        assert!(res.unwrap_err().to_string().contains("Missing required parameter"));

        // 4. Fail: Wrong type (string instead of boolean)
        let args_wrong_type = serde_json::json!({
            "path": "/workspace/file.txt",
            "content": "hello world",
            "overwrite": "yes"
        });
        let res2 = validate_json_schema(&schema, &args_wrong_type);
        assert!(res2.is_err());
        assert!(res2.unwrap_err().to_string().contains("expects type 'boolean'"));

        // 5. Fail: Wrong type (float with fractional part instead of integer)
        let args_wrong_int = serde_json::json!({
            "path": "/workspace/file.txt",
            "content": "hello world",
            "lines_count": 42.5
        });
        let res3 = validate_json_schema(&schema, &args_wrong_int);
        assert!(res3.is_err());
        assert!(res3.unwrap_err().to_string().contains("expects type 'integer'"));
    }
}

struct McpToolWrapper {
    name: String,
    mcp_host: Arc<crate::agent::mcp::McpHost>,
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
                        "Unhandled system delegate '{}'", name
                    )))
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
        let path_val = map
            .get("path")
            .or_else(|| map.get("file"))
            .or_else(|| map.get("target_path"))
            .or_else(|| map.get("SearchPath"))
            .or_else(|| map.get("DirectoryPath"))
            .or_else(|| map.get("AbsolutePath"));

        if let Some(v) = path_val {
            if let Some(path_str) = v.as_str() {
                let safe_path = crate::utils::security::validate_path(workspace_root, path_str)
                    .map_err(|e| {
                        ToolExecutionError::SecurityBlocked(format!(
                            "Path validation failed: {}",
                            e
                        ))
                    })?;
                return Ok(Some(safe_path.to_path_buf()));
            }
        }
    }
    Ok(None)
}

fn resolve_required_permission(
    name: &str,
    args: &serde_json::Value,
    workspace_root: &std::path::Path,
    is_mutating: bool,
) -> Result<Permission, ToolExecutionError> {
    let get_workspace_str = || {
        let workspace_canonical = workspace_root
            .canonicalize()
            .unwrap_or_else(|_| workspace_root.to_path_buf());
        let mut ws_str = workspace_canonical
            .to_string_lossy()
            .to_string()
            .replace('\\', "/");
        if ws_str.starts_with("//?/") {
            ws_str = ws_str[4..].to_string();
        }
        ws_str
    };

    match name {
        "read_file" | "get_file_contents" | "read_codebase_file" | "list_files"
        | "grep_search" | "list_file_symbols" | "get_symbol_body" => {
            let path = extract_path_from_args(args, workspace_root)?
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(get_workspace_str);
            Ok(Permission::FileRead(path))
        }
        "write_file" | "delete_file" => {
            let path = extract_path_from_args(args, workspace_root)?
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(get_workspace_str);
            Ok(Permission::FileWrite(path))
        }
        "execute_shell" => {
            let cmd = args
                .get("command")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            Ok(Permission::ShellExecute(cmd.to_string()))
        }
        "spawn_subagent" | "recruit_specialist" | "request_model_switch" => {
            Ok(Permission::SpawnAgent)
        }
        "fetch_url" => {
            let url = args.get("url").and_then(|v| v.as_str()).unwrap_or("");
            Ok(Permission::NetworkFetch(url.to_string()))
        }
        _ => {
            if is_mutating {
                Err(ToolExecutionError::SecurityBlocked(format!(
                    "Security Violation: Mutating permission is denied by default for unknown tool '{}'",
                    name
                )))
            } else {
                // Least-privilege: unknown non-mutating tools get ToolExec only,
                // not workspace-wide FileRead. Actual security is enforced by
                // the CBS skill-gate and oversight pipeline.
                Ok(Permission::ToolExec(name.to_string()))
            }
        }
    }
}

pub fn is_cacheable_tool_name(name: &str) -> bool {
    matches!(
        name,
        "read_file"
            | "get_file_contents"
            | "read_codebase_file"
            | "list_files"
            | "grep_search"
            | "list_file_symbols"
            | "get_symbol_body"
    )
}

pub fn is_mutating_tool_name(name: &str) -> bool {
    // Strip "mcp_{server}_" prefix to get the operation name.
    // e.g. "mcp_git_create_repository" → "create_repository"
    // e.g. "mcp_github_push_files"     → "push_files"
    // Falls back to the full name for non-MCP tools or short MCP names.
    let operation = name
        .strip_prefix("mcp_")
        .and_then(|rest| rest.find('_').map(|idx| &rest[idx + 1..]))
        .unwrap_or(name);

    matches!(
        operation,
        "write_file"
            | "delete_file"
            | "execute_shell"
            | "synthesize_micro_script"
            | "refactor_synthesized_skill"
            | "spawn_subagent"
            | "recruit_specialist"
            | "create_or_update_file"
            | "push_files"
            | "create_repository"
            | "create_issue"
            | "create_pull_request"
            | "fork_repository"
            | "create_branch"
            | "update_issue"
            | "add_issue_comment"
            | "create_pull_request_review"
            | "merge_pull_request"
            | "update_pull_request_branch"
    )
}

/// Checks if an MCP-prefixed tool name maps to a dangerous operation.
/// This includes all mutating operations plus GitHub data-access endpoints
/// that could leak private repository data.
///
/// IMPORTANT: every mutating tool is also considered dangerous via the
/// `is_mutating_tool_name` check. This composition is intentional.
/// Add new dangerous operations to one of:
///   - `is_mutating_tool_name` (modifies workspace files)
///   - this function's `matches!` block (reads sensitive MCP data)
fn is_dangerous_mcp_operation(name: &str) -> bool {
    if !name.starts_with("mcp_") {
        return false;
    }

    // Already covers mutating ops
    if is_mutating_tool_name(name) {
        return true;
    }

    let operation = name
        .strip_prefix("mcp_")
        .and_then(|rest| rest.find('_').map(|idx| &rest[idx + 1..]))
        .unwrap_or(name);

    // Dangerous read operations that could leak private data or
    // perform broad reconnaissance
    matches!(
        operation,
        "search_repositories"
            | "search_code"
            | "search_users"
            | "search_issues"
            | "get_pull_request_files"
            | "get_file_contents"
            | "get_pull_request"
            | "get_pull_request_comments"
            | "get_pull_request_reviews"
            | "get_pull_request_status"
            | "get_issue"
    )
}

/// Checks if a native (non-MCP) tool name is a dangerous operation.
/// These tools don't modify workspace files (so they're not in `is_mutating_tool_name`)
/// but they carry security risk from prompt injection, data exfiltration,
/// or unauthorized state changes.
fn is_dangerous_native_operation(name: &str) -> bool {
    matches!(
        name,
        "request_model_switch" | "fetch_url" | "run_integrity_check"
    )
}

/// Validates dynamic or core tool arguments against their parameters JSON schema (properties and required fields).
fn validate_json_schema(schema: &serde_json::Value, args: &serde_json::Value) -> Result<(), crate::error::AppError> {
    if schema.is_null() || !schema.is_object() {
        return Ok(());
    }

    let schema_obj = schema.as_object().unwrap();
    let args_obj = args.as_object().ok_or_else(|| {
        crate::error::AppError::BadRequest("Arguments must be a JSON object".to_string())
    })?;

    // 1. Check required parameters
    if let Some(required) = schema_obj.get("required").and_then(|r| r.as_array()) {
        for req_val in required {
            if let Some(req_name) = req_val.as_str() {
                if !args_obj.contains_key(req_name) || args_obj.get(req_name).unwrap().is_null() {
                    return Err(crate::error::AppError::BadRequest(format!(
                        "Missing required parameter: '{}'",
                        req_name
                    )));
                }
            }
        }
    }

    // 2. Check properties and types
    if let Some(properties) = schema_obj.get("properties").and_then(|p| p.as_object()) {
        for (prop_name, prop_def) in properties {
            if let Some(val) = args_obj.get(prop_name) {
                if val.is_null() {
                    continue;
                }

                if let Some(expected_type) = prop_def.get("type").and_then(|t| t.as_str()) {
                    match expected_type {
                        "string" => {
                            if !val.is_string() {
                                return Err(crate::error::AppError::BadRequest(format!(
                                    "Parameter '{}' expects type 'string', got: {}",
                                    prop_name, val
                                )));
                            }
                        }
                        "integer" => {
                            let is_int = val.is_i64() || val.is_u64() || (val.is_number() && val.as_f64().map_or(false, |f| f.fract() == 0.0));
                            if !is_int {
                                return Err(crate::error::AppError::BadRequest(format!(
                                    "Parameter '{}' expects type 'integer', got: {}",
                                    prop_name, val
                                )));
                            }
                        }
                        "number" => {
                            if !val.is_number() {
                                return Err(crate::error::AppError::BadRequest(format!(
                                    "Parameter '{}' expects type 'number', got: {}",
                                    prop_name, val
                                )));
                            }
                        }
                        "boolean" => {
                            if !val.is_boolean() {
                                return Err(crate::error::AppError::BadRequest(format!(
                                    "Parameter '{}' expects type 'boolean', got: {}",
                                    prop_name, val
                                )));
                            }
                        }
                        "array" => {
                            if !val.is_array() {
                                return Err(crate::error::AppError::BadRequest(format!(
                                    "Parameter '{}' expects type 'array', got: {}",
                                    prop_name, val
                                )));
                            }
                        }
                        "object" => {
                            if !val.is_object() {
                                return Err(crate::error::AppError::BadRequest(format!(
                                    "Parameter '{}' expects type 'object', got: {}",
                                    prop_name, val
                                )));
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    Ok(())
}


// Metadata: [mod]

// Metadata: [mod]

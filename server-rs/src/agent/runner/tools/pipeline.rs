//! @docs ARCHITECTURE:Runner
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Agent Runner / Tool Execution Pipeline
//!
//! ### AI Assist Note
//! - Resolve paths and enforce capability, oversight, schema, cache, and execution order in one pipeline.
//! - Redact blocked arguments before they enter mirror-mode telemetry.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: ToolExecutionError, AppError
//! - **Telemetry Targets**: [tools]

use super::{
    capability::Permission,
    error::ToolExecutionError,
    resolved_policy::{ToolEffects, ToolPolicyDescriptor},
    validation,
    validation::{extract_path_from_args, validate_json_schema},
    AgentRunner, CapabilityToken, McpToolWrapper, RunContext, Tool, ToolContext, ToolLeaseGuard,
};
use crate::error::AppError;
use std::sync::Arc;

impl AgentRunner {
    /// Manages the Zero-Trust sequence (WAL -> CBS -> Execute)
    pub(super) async fn run_zero_trust_pipeline(
        &self,
        ctx: &RunContext,
        fc: &crate::agent::types::ToolCall,
        usage: &mut Option<crate::agent::types::TokenUsage>,
        user_message: &str,
        token: CapabilityToken,
    ) -> Result<String, ToolExecutionError> {
        let mut normalized_call = fc.clone();
        match fc.name.as_str() {
            "read_file"
            | "get_file_contents"
            | "write_file"
            | "delete_file"
            | "restore_file_version"
            | "get_file_history"
            | "list_files"
            | "grep_search" => {
                let breadcrumbs = matches!(
                    fc.name.as_str(),
                    "read_file"
                        | "get_file_contents"
                        | "write_file"
                        | "delete_file"
                        | "restore_file_version"
                        | "get_file_history"
                );
                let path = Self::resolve_path_arg(ctx, fc, breadcrumbs)?;
                if !path.is_empty() {
                    for alias in validation::PATH_ALIASES {
                        if normalized_call.args.get(*alias).is_some() {
                            normalized_call.args[*alias] = serde_json::json!(path);
                        }
                    }
                    normalized_call.args["filename"] = serde_json::json!(path);
                    normalized_call.args["path"] = serde_json::json!(path);
                    if matches!(fc.name.as_str(), "list_files" | "grep_search") {
                        normalized_call.args["dir"] = serde_json::json!(path);
                    }
                }
            }
            "read_codebase_file" | "list_file_symbols" | "get_symbol_body" => {
                if let Some(path) =
                    validation::extract_path_from_args(&fc.args, &ctx.env.workspace_root)?
                {
                    // Codebase dispatchers resolve relative paths from the repository;
                    // preserve the absolute workspace target verified by the capability.
                    for alias in validation::PATH_ALIASES {
                        if normalized_call.args.get(*alias).is_some() {
                            normalized_call.args[*alias] = serde_json::json!(path);
                        }
                    }
                    normalized_call.args["path"] = serde_json::json!(path);
                    normalized_call.args["filename"] = serde_json::json!(path);
                }
            }
            "execute_shell" => {
                let (executable, args) = validation::extract_shell_command(&fc.args)
                    .map_err(ToolExecutionError::AppError)?;
                let current_dir = ctx.current_dir.lock().clone();
                let cwd = validation::shell_working_directory(
                    &fc.args,
                    &ctx.env.workspace_root,
                    current_dir.as_deref(),
                )
                .map_err(ToolExecutionError::AppError)?;
                normalized_call
                    .args
                    .as_object_mut()
                    .ok_or_else(|| {
                        ToolExecutionError::SecurityBlocked(
                            "Tool arguments must be an object".to_string(),
                        )
                    })?
                    .remove("command");
                normalized_call.args["executable"] = serde_json::json!(executable);
                normalized_call.args["args"] = serde_json::json!(args);
                normalized_call.args["cwd"] = serde_json::json!(cwd);
            }
            _ => {}
        }
        let fc = &normalized_call;
        let call_id = uuid::Uuid::new_v4().to_string();
        let args_str = serde_json::to_string(&fc.args).unwrap_or_default();
        let redacted_args = self.state.security.secret_redactor.redact(&args_str);
        let mission_id_opt = Some(ctx.mission_state.mission_id.clone());
        tracing::info!(
            "📦 [tools] Executing tool: {} (call_id: {})",
            fc.name,
            call_id
        );

        // The model-facing toolbelt is not an authorization boundary by itself:
        // providers can return undeclared function names. Enforce the agent's MCP
        // capability declaration again immediately before wrapper construction.
        if crate::agent::mcp::decode_mcp_tool_name(&fc.name).is_some()
            && !crate::agent::mcp::is_mcp_tool_authorized(&ctx.mcp_tools, &fc.name)
        {
            return Err(ToolExecutionError::SecurityBlocked(format!(
                "Agent '{}' is not authorized to execute MCP tool '{}'",
                ctx.identity.agent_id, fc.name
            )));
        }

        // Retrieve tool from registry (optional early lookup, fallback to MCP wrapper if not found in static registry)
        let tool = match self.state.registry.tool_registry.get(&fc.name) {
            Some(t) => Some(t),
            None => {
                let mcp_host = self.state.registry.mcp_host.clone();
                let has_mcp = mcp_host.has_native_tool(&fc.name).await
                    || fc.name.starts_with("mcp_")
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
        let descriptor = ToolPolicyDescriptor::with_effects(
            &fc.name,
            ToolEffects {
                cacheable: tool.as_ref().map(|t| t.is_cacheable()).unwrap_or(false),
                mutating: tool.as_ref().map(|t| t.is_mutating()).unwrap_or(false),
                dangerous: tool.as_ref().map(|t| t.is_dangerous()).unwrap_or(false),
            },
        );
        let is_cacheable = descriptor.effects.cacheable;
        let is_mutating = descriptor.effects.mutating;

        if self.state.mirror_mode && is_mutating {
            let alert_id = uuid::Uuid::new_v4().to_string();
            let timestamp = chrono::Utc::now().to_rfc3339();
            let alert_value = serde_json::json!({
                "id": alert_id,
                "timestamp": timestamp,
                "agent_id": ctx.identity.agent_id,
                "mission_id": ctx.mission_state.mission_id,
                "alert_type": "mutating_tool_blocked",
                "detail": format!("Blocked mutating tool call '{}' with arguments: {}", fc.name, redacted_args)
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
        // We MUST record the intent before execution with correlation call_id.
        let intent_params = serde_json::json!({
            "call_id": &call_id,
            "args": serde_json::from_str::<serde_json::Value>(&redacted_args)
                .unwrap_or_else(|_| serde_json::json!(redacted_args)),
        })
        .to_string();

        self.state
            .security
            .audit_trail
            .record(
                &ctx.identity.agent_id,
                mission_id_opt.as_deref(),
                ctx.mission_state.user_id.as_deref(),
                &format!("[INTENT] {}", fc.name),
                &intent_params,
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
            descriptor.required_permission(&fc.args, &ctx.env.workspace_root)?;

        if !token.verify(&required_permission) {
            return Err(ToolExecutionError::SecurityBlocked(format!(
                "Capability Token verification failed for tool '{}': missing required permission {:?}",
                fc.name, required_permission
            )));
        }

        // 3. Security Manager & Oversight Gates
        self.validate_security_and_oversight(ctx, fc, user_message, &descriptor)
            .await?;

        // 4. Construct the engine context only after capability and security gates.
        let tool_ctx = ToolContext::from_run_context(ctx, self.state.clone());

        let workspace_root_str = tool_ctx.workspace_root.to_string_lossy().to_string();

        // Validate before cache lookup so cached outputs never bypass the tool schema.
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
        let target_path = if matches!(
            required_permission,
            Permission::FileRead(_) | Permission::FileWrite(_)
        ) {
            extract_path_from_args(&fc.args, &tool_ctx.workspace_root)?
        } else {
            None
        };

        let mut lease_guard = None;
        if is_mutating {
            if fc.name == "execute_shell" {
                self.state.resources.tool_cache.lock().clear();
            } else if let Some(ref path) = target_path {
                self.state.resources.tool_cache.lock().invalidate_path(path);
                let guard = ToolLeaseGuard::acquire(
                    self.state.resources.conflict_manager.clone(),
                    path.clone(),
                    ctx.identity.agent_id.clone(),
                )?;
                lease_guard = Some(guard);
            }
        }

        self.execute_with_retry(
            &call_id,
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
            &ctx.identity.agent_id,
            ctx.mission_state.user_id.as_deref(),
            &ctx.modified_files,
            &ctx.last_mutation_timestamp_ms,
        )
        .await
    }
}

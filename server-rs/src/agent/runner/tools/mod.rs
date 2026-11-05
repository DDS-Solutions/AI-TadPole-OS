//! @docs ARCHITECTURE:Runner
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Agent Runner / mod
//!
//! ### AI Assist Note
//! Tool intent, permission, oversight, cache, and execution share normalized arguments.
//! Native handlers also share the caller's mission constraints and failure tracking.
//!
//! ### 🔍 Debugging & Observability
//! - **Telemetry Targets**: none declared

pub mod cache;
pub mod capability;
pub mod dispatcher;
pub mod error;
pub mod manifest;
pub mod plugin;
pub mod registry;
pub(crate) mod resolved_policy;
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

mod execution;
mod oversight;
mod pipeline;

use crate::error::AppError;
pub use capability::{CapabilityToken, ZeroTrustGuard};
pub use trait_tool::{Tool, ToolContext};

use super::{AgentRunner, RunContext};
use error::ToolExecutionError;
use std::sync::Arc;

pub use lease::ToolLeaseGuard;
pub(crate) use mcp_wrapper::McpToolWrapper;

#[async_trait::async_trait]
impl super::service_traits::ToolExecutor for AgentRunner {
    async fn execute_tool(
        &self,
        ctx: &RunContext,
        fc: &crate::agent::types::ToolCall,
        user_message: &str,
    ) -> Result<(String, Option<crate::agent::types::TokenUsage>), AppError> {
        // 1. Mint Capability Token for this specific call with explicit agent skills
        let token = ZeroTrustGuard::mint_token(
            &ctx.identity.agent_id,
            &ctx.mission_state.mission_id,
            ctx.identity.authority_level,
            ctx.allowed_files.as_deref(),
            Some(&ctx.skills),
            &ctx.env.workspace_root,
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
                if let ToolExecutionError::AppError(ref ae) = e {
                    if matches!(
                        ae,
                        AppError::Unauthorized(_) | AppError::Forbidden(_) | AppError::RateLimit(_)
                    ) {
                        // Propagate security/billing blocks directly to halt execution
                        return Err(e.into());
                    }
                }

                tracing::error!(
                    "Tool '{}' failed for agent {}: {} (recovery: {:?})",
                    fc.name,
                    ctx.identity.agent_id,
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

    fn get_provider_timeout_secs(&self) -> u64 {
        self.state
            .governance
            .provider_timeout_secs
            .load(std::sync::atomic::Ordering::Relaxed) as u64
    }

    fn get_tool_timeout_secs(&self) -> u64 {
        let timeout = self.get_provider_timeout_secs();
        if timeout > 0 {
            timeout.clamp(5, 600)
        } else {
            60
        }
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

    fn verify_execution_witnesses(
        &self,
        records: &[super::ProcessExecutionRecord],
        last_mutation_ms: u64,
    ) -> bool {
        self.verify_execution_witnesses(records, last_mutation_ms)
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
        ctx.identity.agent_id = "worker-1".to_string();
        ctx.skills = vec!["allowed_skill".to_string(), "forbidden_skill".to_string()];

        let mut agent = EngineAgent::default();
        agent.identity.id = ctx.identity.agent_id.clone();
        agent.capabilities.skills = vec!["allowed_skill".to_string()];
        state
            .registry
            .agents
            .insert(ctx.identity.agent_id.clone(), agent);

        let fc = ToolCall {
            name: "forbidden_skill".to_string(),
            args: serde_json::json!({}),
        };

        let token = ZeroTrustGuard::mint_token(
            &ctx.identity.agent_id,
            &ctx.mission_state.mission_id,
            ctx.identity.authority_level,
            ctx.allowed_files.as_deref(),
            Some(&ctx.skills),
            &ctx.env.workspace_root,
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
        let mut mock_state = AppState::new_minimal_mock().await;
        mock_state.mirror_mode = true;
        let state = Arc::new(mock_state);

        assert!(state.mirror_mode);

        let runner = AgentRunner::new(state.clone());
        let mut ctx = RunContext::default();
        ctx.identity.agent_id = "worker-1".to_string();

        let mut agent = EngineAgent::default();
        agent.identity.id = ctx.identity.agent_id.clone();
        agent.capabilities.skills = vec!["write_file".to_string()];
        state
            .registry
            .agents
            .insert(ctx.identity.agent_id.clone(), agent);

        let fc = ToolCall {
            name: "write_file".to_string(),
            args: serde_json::json!({
                "path": "test_file.txt",
                "content": "hello"
            }),
        };

        let token = ZeroTrustGuard::mint_token(
            &ctx.identity.agent_id,
            &ctx.mission_state.mission_id,
            ctx.identity.authority_level,
            ctx.allowed_files.as_deref(),
            Some(&ctx.skills),
            &ctx.env.workspace_root,
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
        ctx.identity.agent_id = AGENT_CEO.to_string();
        ctx.identity.authority_level = crate::agent::types::RoleAuthorityLevel::Executive;

        let mut agent = EngineAgent::default();
        agent.identity.id = ctx.identity.agent_id.clone();
        state
            .registry
            .agents
            .insert(ctx.identity.agent_id.clone(), agent);

        let fc = ToolCall {
            name: "spawn_subagent".to_string(),
            args: serde_json::json!({"agent_id": "worker"}),
        };

        let token = ZeroTrustGuard::mint_token(
            &ctx.identity.agent_id,
            &ctx.mission_state.mission_id,
            ctx.identity.authority_level,
            ctx.allowed_files.as_deref(),
            Some(&ctx.skills),
            &ctx.env.workspace_root,
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
        let mut ctx = RunContext::default();
        ctx.skills = vec!["risky_tool".to_string()];

        // Set policy to Deny for a specific tool
        let _ = state
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
            &ctx.identity.agent_id,
            &ctx.mission_state.mission_id,
            ctx.identity.authority_level,
            ctx.allowed_files.as_deref(),
            Some(&ctx.skills),
            &ctx.env.workspace_root,
        );

        let mut usage = None;
        let result = runner
            .run_zero_trust_pipeline(&ctx, &fc, &mut usage, "", token)
            .await;

        assert!(
            matches!(
                result,
                Err(ToolExecutionError::SecurityBlocked(ref msg)) if msg.contains("policy") && msg.contains("risky_tool")
            ),
            "Expected policy denial, received {result:?}"
        );
    }

    #[tokio::test]
    async fn test_update_working_memory() {
        let state = Arc::new(AppState::new_minimal_mock().await);
        let runner = AgentRunner::new(state.clone());
        let mut ctx = RunContext::default();
        ctx.identity.agent_id = "memory-agent".to_string();

        let mut agent = EngineAgent::default();
        agent.identity.id = ctx.identity.agent_id.clone();

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
            .find(|a| a.identity.id == ctx.identity.agent_id)
            .unwrap();

        state
            .registry
            .agents
            .insert(ctx.identity.agent_id.clone(), agent);

        let fc = ToolCall {
            name: "update_working_memory".to_string(),
            args: serde_json::json!({"memory": {"last_step": "initialized"}}),
        };

        let mut output = String::new();
        let result = runner
            .handle_update_working_memory(&ctx, &fc, &mut output)
            .await;

        assert!(result.is_ok());
        let agent = state.registry.agents.get(&ctx.identity.agent_id).unwrap();
        assert_eq!(agent.state.working_memory["last_step"], "initialized");
    }

    #[tokio::test]
    async fn test_unknown_mutating_tool_default_denied() {
        let state = Arc::new(AppState::new_minimal_mock().await);
        let runner = AgentRunner::new(state.clone());
        let mut ctx = RunContext::default();
        ctx.identity.agent_id = "test-agent".to_string();
        ctx.mcp_tools = vec!["git:create_repository".to_string()];

        let mut agent = EngineAgent::default();
        agent.identity.id = ctx.identity.agent_id.clone();
        agent.capabilities.skills = vec!["mcp_git_create_repository".to_string()];
        agent.capabilities.mcp_tools = ctx.mcp_tools.clone();
        state
            .registry
            .agents
            .insert(ctx.identity.agent_id.clone(), agent);

        let fc = ToolCall {
            name: "mcp_git_create_repository".to_string(), // Mutating tool name
            args: serde_json::json!({}),
        };

        let token = ZeroTrustGuard::mint_token(
            &ctx.identity.agent_id,
            &ctx.mission_state.mission_id,
            ctx.identity.authority_level,
            ctx.allowed_files.as_deref(),
            Some(&ctx.skills),
            &ctx.env.workspace_root,
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
        ctx.safe_mode = true;
        ctx.identity.agent_id = "test-agent".to_string();
        ctx.skills = vec!["mcp_docs_query_docs".to_string()];
        ctx.mcp_tools = vec!["docs:query_docs".to_string()];

        let mut agent = EngineAgent::default();
        agent.identity.id = ctx.identity.agent_id.clone();
        agent.capabilities.skills = vec!["mcp_docs_query_docs".to_string()];
        agent.capabilities.mcp_tools = ctx.mcp_tools.clone();
        state
            .registry
            .agents
            .insert(ctx.identity.agent_id.clone(), agent);

        let fc = ToolCall {
            name: "mcp_docs_query_docs".to_string(), // Non-mutating tool name
            args: serde_json::json!({}),
        };

        let token = ZeroTrustGuard::mint_token(
            &ctx.identity.agent_id,
            &ctx.mission_state.mission_id,
            ctx.identity.authority_level,
            ctx.allowed_files.as_deref(),
            Some(&ctx.skills),
            &ctx.env.workspace_root,
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

    #[tokio::test]
    async fn test_undeclared_mcp_tool_blocked_before_dispatch() {
        let state = Arc::new(AppState::new_minimal_mock().await);
        let runner = AgentRunner::new(state);
        let mut ctx = RunContext::default();
        ctx.identity.agent_id = "least-privilege-agent".to_string();

        let fc = ToolCall {
            name: "mcp_docs_query_docs".to_string(),
            args: serde_json::json!({}),
        };
        let token = ZeroTrustGuard::mint_token(
            &ctx.identity.agent_id,
            &ctx.mission_state.mission_id,
            ctx.identity.authority_level,
            ctx.allowed_files.as_deref(),
            Some(&ctx.skills),
            &ctx.env.workspace_root,
        );

        let mut usage = None;
        let result = runner
            .run_zero_trust_pipeline(&ctx, &fc, &mut usage, "", token)
            .await;

        assert!(matches!(
            result,
            Err(ToolExecutionError::SecurityBlocked(message))
                if message.contains("not authorized to execute MCP tool")
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
        assert!(is_dangerous_native_operation("resolve_x402_challenge"));

        // Should NOT flag mutating tools (those are in is_mutating_tool_name)
        assert!(!is_dangerous_native_operation("write_file"));
        assert!(!is_dangerous_native_operation("execute_shell"));
        assert!(!is_dangerous_native_operation("propose_asset_transaction"));

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

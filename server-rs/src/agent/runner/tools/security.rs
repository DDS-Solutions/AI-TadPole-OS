//! @docs ARCHITECTURE:Registry
//!
//! ### AI Assist Note
//! - Grouped execution context owns identity, mission state and environment; retain scope across callers.
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Agent Runner / security
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: none declared

use super::error::ToolExecutionError;
use super::resolved_policy::{PolicyBoundary, ToolPolicyDescriptor};
use crate::agent::constants::{AGENT_ALPHA, AGENT_CEO, AGENT_COO};
use crate::agent::runner::AgentRunner;
use crate::agent::runner::RunContext;
use crate::agent::types::ToolCall;

pub struct ValidationResult {
    pub oversight_required: bool,
    pub oversight_reason: String,
}

#[async_trait::async_trait]
pub trait SecurityManager: Send + Sync {
    async fn pre_validate(
        &self,
        runner: &AgentRunner,
        ctx: &RunContext,
        fc: &ToolCall,
    ) -> Result<ValidationResult, ToolExecutionError> {
        let descriptor = ToolPolicyDescriptor::for_tool(&fc.name);
        self.pre_validate_resolved(runner, ctx, fc, &descriptor)
            .await
    }

    async fn pre_validate_resolved(
        &self,
        runner: &AgentRunner,
        ctx: &RunContext,
        fc: &ToolCall,
        descriptor: &ToolPolicyDescriptor,
    ) -> Result<ValidationResult, ToolExecutionError>;
}

pub struct DefaultSecurityManager;

#[async_trait::async_trait]
impl SecurityManager for DefaultSecurityManager {
    async fn pre_validate_resolved(
        &self,
        runner: &AgentRunner,
        ctx: &RunContext,
        fc: &ToolCall,
        descriptor: &ToolPolicyDescriptor,
    ) -> Result<ValidationResult, ToolExecutionError> {
        // This guard is defense in depth for direct dispatch paths. The normal
        // runner rejects Plane 2 before mission setup because it has no
        // company-scoped memory or tool context yet.
        if ctx.identity.is_plane2() {
            return Err(ToolExecutionError::SecurityBlocked(format!(
                "{} agents are not enabled in the generic runner until company-scoped execution is implemented",
                crate::agent::runner::PLANE2_DEPARTMENT_MARKER
            )));
        }

        let mut trigger_oversight = false;
        let mut oversight_reason = String::new();

        // 1. [Hierarchy Guard] Enforce strategic delegation for CEO/COO unconditionally
        if matches!(
            fc.name.as_str(),
            "spawn_subagent" | "recruit_specialist" | "send_agent_envelope"
        ) {
            if ctx.identity.agent_id == AGENT_CEO {
                tracing::warn!("🛡️ [Hierarchy Guard] CEO (ID: {}) blocked from spawning specialists or sending direct envelopes.", AGENT_CEO);
                runner.broadcast_sys("🛡️ Hierarchy Guard: CEO blocked from direct recruitment or messaging. Use 'issue_alpha_directive' instead.", "warning", Some(ctx.mission_state.mission_id.clone()));
                return Err(ToolExecutionError::HierarchyBlocked("As CEO, you are prohibited from direct worker recruitment or messaging. You MUST use 'issue_alpha_directive' to delegate complex missions to the COO.".to_string()));
            }
            if ctx.identity.agent_id == AGENT_COO {
                if fc.name.as_str() == "send_agent_envelope" {
                    let target = fc
                        .args
                        .get("target_agent_id")
                        .and_then(|v| v.as_str())
                        .unwrap_or("");
                    if !target.eq_ignore_ascii_case(AGENT_ALPHA) {
                        tracing::warn!("🛡️ [Hierarchy Guard] COO (ID: {}) blocked from sending envelope directly to '{}'. Envelopes must target the Alpha Node.", AGENT_COO, target);
                        runner.broadcast_sys("🛡️ Hierarchy Guard: COO blocked from sending envelopes directly to workers. Envelopes must target the Alpha Node.", "warning", Some(ctx.mission_state.mission_id.clone()));
                        return Err(ToolExecutionError::HierarchyBlocked("As COO, you are prohibited from sending direct envelopes to worker agents. You MUST communicate via the Alpha Node (ID: alpha).".to_string()));
                    }
                } else {
                    let target = fc
                        .args
                        .get("agent_id")
                        .and_then(|v| v.as_str())
                        .unwrap_or("");
                    if !target.eq_ignore_ascii_case(AGENT_ALPHA) {
                        tracing::warn!("🛡️ [Hierarchy Guard] COO (ID: {}) blocked from spawning specialist '{}' directly.", AGENT_COO, target);
                        runner.broadcast_sys("🛡️ Hierarchy Guard: COO blocked from direct worker recruitment. Use Alpha Node commander instead.", "warning", Some(ctx.mission_state.mission_id.clone()));
                        return Err(ToolExecutionError::HierarchyBlocked("As COO, you are prohibited from direct worker recruitment. You MUST recruit an Alpha Node (ID: alpha) to serve as Swarm Mission Commander.".to_string()));
                    }
                }
            }
        }

        // 2. [CBS] Skill-Based Security Allowlist
        if let Some(agent) = runner.state.registry.agents.get(&ctx.identity.agent_id) {
            let allowed_skills = &agent.value().capabilities.skills;
            let is_builtin = super::registry::BUILTIN_TOOLS.contains(&fc.name.as_str());

            if !is_builtin && !allowed_skills.contains(&fc.name) {
                tracing::warn!(
                    "🛡️ [CBS] Agent {} attempted unauthorized skill: {}",
                    ctx.identity.agent_id,
                    fc.name
                );
                runner.broadcast_sys(
                    &format!(
                        "🛡️ CBS: {} attempted unauthorized skill: {}",
                        ctx.identity.name, fc.name
                    ),
                    "error",
                    Some(ctx.mission_state.mission_id.clone()),
                );

                let suggestion = if allowed_skills.contains(&"spawn_subagent".to_string())
                    || allowed_skills.contains(&"send_mission_directive".to_string())
                    || allowed_skills.contains(&"issue_alpha_directive".to_string())
                    || ctx.identity.agent_id == AGENT_CEO
                    || ctx.identity.agent_id == AGENT_COO
                {
                    format!(
                        " As Swarm Commander/Orchestrator, you do not have direct permission to execute '{}'. You MUST call `spawn_subagent` or `send_mission_directive` to recruit or delegate this task to an authorized worker node (e.g. 'coder' or 'audit_specialist').",
                        fc.name
                    )
                } else {
                    format!(
                        " Authorized skills in your allowlist: {:?}.",
                        allowed_skills
                    )
                };

                return Err(ToolExecutionError::SecurityBlocked(format!(
                    "Skill '{}' not in agent allowlist.{}",
                    fc.name, suggestion
                )));
            }
        }

        // 3. [Dynamic Policy] Evaluate granular capability decision with SEC-06 security floor clamping
        let agent_role = runner
            .state
            .registry
            .agents
            .get(&ctx.identity.agent_id)
            .map(|a| a.value().identity.role.clone());

        let decision = descriptor
            .decision(
                &runner.state.security.permission_policy,
                Some(&ctx.identity.agent_id),
                agent_role.as_deref(),
                PolicyBoundary::Runner,
            )
            .await;

        match decision.mode {
            crate::security::permissions::PermissionMode::Deny => {
                let reason = decision
                    .reason
                    .unwrap_or_else(|| format!("Policy for '{}' is set to DENY", fc.name));
                return Err(ToolExecutionError::SecurityBlocked(reason));
            }
            crate::security::permissions::PermissionMode::Prompt => {
                let is_safe_skill = !descriptor.effects.mutating && !descriptor.effects.dangerous;
                let auto_approve = runner
                    .state
                    .governance
                    .auto_approve_safe_skills
                    .load(std::sync::atomic::Ordering::Acquire);

                if is_safe_skill && (ctx.safe_mode || auto_approve) {
                    tracing::info!(
                        "🛡️ [Governance] Auto-approving safe read-only tool '{}' for agent '{}' (safe_mode={}, auto_approve={})",
                        fc.name,
                        ctx.identity.agent_id,
                        ctx.safe_mode,
                        auto_approve
                    );
                } else {
                    trigger_oversight = true;
                    oversight_reason = decision.reason.unwrap_or_else(|| {
                        format!("Sovereign Policy requires 'Prompt' for tool: {}", fc.name)
                    });
                }
            }
            crate::security::permissions::PermissionMode::Allow => {}
        }

        if !trigger_oversight {
            // 4. [Security Gate] Skill Manifest Validation
            let mut manifest_requires = false;

            let manifest_opt = runner
                .state
                .registry
                .skill_registry
                .get(&fc.name)
                .or_else(|| {
                    descriptor
                        .manifest_alias
                        .and_then(|alias| runner.state.registry.skill_registry.get(alias))
                });

            if let Some(manifest) = manifest_opt {
                if manifest.requires_oversight {
                    manifest_requires = true;
                }

                // 4b. [Schema Gate] Validate arguments against skill parameter manifest
                if !manifest.parameters.is_empty() {
                    if let Err(e) = manifest.validate_arguments(&fc.args) {
                        tracing::warn!(
                            "🛡️ [Schema Gate] Argument validation failed for skill '{}': {}",
                            fc.name,
                            e
                        );
                        return Err(ToolExecutionError::SecurityBlocked(format!(
                            "Argument validation failed: {}",
                            e
                        )));
                    }
                }
            }
            if !manifest_requires {
                if let Some(skill) = runner.state.registry.skills.skills.get(&fc.name) {
                    if skill.oversight_required {
                        manifest_requires = true;
                    }
                }
            }

            if manifest_requires {
                trigger_oversight = true;
                oversight_reason = format!("Security Gate triggered by manifest for: {}", fc.name);
            }
        }

        // 5. [Agent-Level Oversight]
        if let Some(agent) = runner.state.registry.agents.get(&ctx.identity.agent_id) {
            if agent.value().requires_oversight {
                trigger_oversight = true;
                oversight_reason = format!(
                    "Mandatory oversight enabled for agent: {}",
                    ctx.identity.name
                );
            }
        }

        Ok(ValidationResult {
            oversight_required: trigger_oversight,
            oversight_reason,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::types::EngineAgent;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_pre_validate_hierarchy_ceo_blocked() {
        let state = Arc::new(crate::state::AppState::new_minimal_mock().await);
        let runner = AgentRunner::new(state.clone());
        let mut ctx = RunContext::default();
        ctx.identity.agent_id = AGENT_CEO.to_string();

        let fc = ToolCall {
            name: "spawn_subagent".to_string(),
            args: serde_json::json!({ "agent_id": "worker" }),
        };

        let manager = DefaultSecurityManager;
        let result = manager.pre_validate(&runner, &ctx, &fc).await;

        assert!(matches!(
            result,
            Err(ToolExecutionError::HierarchyBlocked(msg)) if msg.contains("prohibited from direct worker recruitment")
        ));
    }

    #[tokio::test]
    async fn test_coo_alpha_target_case_preserves_hierarchy_boundary() {
        let state = Arc::new(crate::state::AppState::new_minimal_mock().await);
        let runner = AgentRunner::new(state);
        let mut ctx = RunContext::default();
        ctx.identity.agent_id = AGENT_COO.into();
        let manager = DefaultSecurityManager;
        for (tool, key) in [
            ("spawn_subagent", "agent_id"),
            ("send_agent_envelope", "target_agent_id"),
        ] {
            for target in ["alpha", "Alpha", "ALPHA"] {
                let fc = ToolCall {
                    name: tool.into(),
                    args: serde_json::json!({(key): target}),
                };
                let result = manager.pre_validate(&runner, &ctx, &fc).await;
                assert!(result.is_ok(), "{tool}: {target}");
            }
            for target in ["coder", "Alpha-worker", "", "1"] {
                let fc = ToolCall {
                    name: tool.into(),
                    args: serde_json::json!({(key): target}),
                };
                assert!(
                    matches!(
                        manager.pre_validate(&runner, &ctx, &fc).await,
                        Err(ToolExecutionError::HierarchyBlocked(_))
                    ),
                    "{tool}: {target}"
                );
            }
        }
    }

    #[tokio::test]
    async fn test_pre_validate_cbs_unauthorized_skill_blocked() {
        let state = Arc::new(crate::state::AppState::new_minimal_mock().await);
        let runner = AgentRunner::new(state.clone());
        let mut ctx = RunContext::default();
        ctx.identity.agent_id = "specialist-1".to_string();

        let mut agent = EngineAgent::default();
        agent.identity.id = ctx.identity.agent_id.clone();
        agent.capabilities.skills = vec!["custom_allowed_skill".to_string()];
        state
            .registry
            .agents
            .insert(ctx.identity.agent_id.clone(), agent);

        let fc = ToolCall {
            name: "unauthorized_skill".to_string(),
            args: serde_json::json!({}),
        };

        let manager = DefaultSecurityManager;
        let result = manager.pre_validate(&runner, &ctx, &fc).await;

        assert!(matches!(
            result,
            Err(ToolExecutionError::SecurityBlocked(msg)) if msg.contains("not in agent allowlist")
        ));
    }

    #[tokio::test]
    async fn test_pre_validate_safe_mode_auto_approve_safe_skill() {
        let state = Arc::new(crate::state::AppState::new_minimal_mock().await);
        state
            .governance
            .auto_approve_safe_skills
            .store(true, std::sync::atomic::Ordering::Release);

        let runner = AgentRunner::new(state.clone());
        let mut ctx = RunContext::default();
        ctx.safe_mode = true;
        ctx.identity.agent_id = "specialist-1".to_string();

        let mut agent = EngineAgent::default();
        agent.identity.id = ctx.identity.agent_id.clone();
        agent.capabilities.skills = vec!["get_system_metrics".to_string()];
        state
            .registry
            .agents
            .insert(ctx.identity.agent_id.clone(), agent);

        let fc = ToolCall {
            name: "get_system_metrics".to_string(),
            args: serde_json::json!({ "timeframe": "year_to_date" }),
        };

        let manager = DefaultSecurityManager;
        let result = manager.pre_validate(&runner, &ctx, &fc).await;

        assert!(result.is_ok());
        let val = result.unwrap();
        assert!(
            !val.oversight_required,
            "Safe read-only skill in safe mode with auto_approve must not require oversight"
        );
    }

    #[tokio::test]
    async fn test_pre_validate_safe_mode_still_requires_oversight_for_mutating_tool() {
        let state = Arc::new(crate::state::AppState::new_minimal_mock().await);
        state
            .governance
            .auto_approve_safe_skills
            .store(true, std::sync::atomic::Ordering::Release);

        let runner = AgentRunner::new(state.clone());
        let mut ctx = RunContext::default();
        ctx.safe_mode = true;
        ctx.identity.agent_id = "specialist-1".to_string();

        let mut agent = EngineAgent::default();
        agent.identity.id = ctx.identity.agent_id.clone();
        agent.capabilities.skills = vec!["write_file".to_string()];
        state
            .registry
            .agents
            .insert(ctx.identity.agent_id.clone(), agent);

        let fc = ToolCall {
            name: "write_file".to_string(),
            args: serde_json::json!({ "filename": "test.txt", "content": "hello" }),
        };

        let manager = DefaultSecurityManager;
        let result = manager.pre_validate(&runner, &ctx, &fc).await;

        assert!(result.is_ok());
        let val = result.unwrap();
        assert!(
            val.oversight_required,
            "Mutating tool must still require oversight even in safe mode"
        );
    }
}

//! @docs ARCHITECTURE:Registry
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Agent Runner / security
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Type-safe state handling and bounded execution without unhandled panics.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: none declared
//! - **Witness Tests**: tests::test_pre_validate_hierarchy_ceo_blocked, tests::test_pre_validate_cbs_unauthorized_skill_blocked

use super::error::ToolExecutionError;
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
    ) -> Result<ValidationResult, ToolExecutionError>;
}

pub struct DefaultSecurityManager;

#[async_trait::async_trait]
impl SecurityManager for DefaultSecurityManager {
    async fn pre_validate(
        &self,
        runner: &AgentRunner,
        ctx: &RunContext,
        fc: &ToolCall,
    ) -> Result<ValidationResult, ToolExecutionError> {
        let mut trigger_oversight = false;
        let mut oversight_reason = String::new();

        // 1. [Hierarchy Guard] Enforce strategic delegation for CEO/COO unconditionally
        if matches!(
            fc.name.as_str(),
            "spawn_subagent" | "recruit_specialist" | "send_agent_envelope"
        ) {
            if ctx.agent_id == AGENT_CEO {
                tracing::warn!("🛡️ [Hierarchy Guard] CEO (ID: {}) blocked from spawning specialists or sending direct envelopes.", AGENT_CEO);
                runner.broadcast_sys("🛡️ Hierarchy Guard: CEO blocked from direct recruitment or messaging. Use 'issue_alpha_directive' instead.", "warning", Some(ctx.mission_id.clone()));
                return Err(ToolExecutionError::HierarchyBlocked("As CEO, you are prohibited from direct worker recruitment or messaging. You MUST use 'issue_alpha_directive' to delegate complex missions to the COO.".to_string()));
            }
            if ctx.agent_id == AGENT_COO {
                if fc.name.as_str() == "send_agent_envelope" {
                    let target = fc
                        .args
                        .get("target_agent_id")
                        .and_then(|v| v.as_str())
                        .unwrap_or("");
                    if target != AGENT_ALPHA {
                        tracing::warn!("🛡️ [Hierarchy Guard] COO (ID: {}) blocked from sending envelope directly to '{}'. Envelopes must target the Alpha Node.", AGENT_COO, target);
                        runner.broadcast_sys("🛡️ Hierarchy Guard: COO blocked from sending envelopes directly to workers. Envelopes must target the Alpha Node.", "warning", Some(ctx.mission_id.clone()));
                        return Err(ToolExecutionError::HierarchyBlocked("As COO, you are prohibited from sending direct envelopes to worker agents. You MUST communicate via the Alpha Node (ID: alpha).".to_string()));
                    }
                } else {
                    let target = fc
                        .args
                        .get("agent_id")
                        .and_then(|v| v.as_str())
                        .unwrap_or("");
                    if target != AGENT_ALPHA {
                        tracing::warn!("🛡️ [Hierarchy Guard] COO (ID: {}) blocked from spawning specialist '{}' directly.", AGENT_COO, target);
                        runner.broadcast_sys("🛡️ Hierarchy Guard: COO blocked from direct worker recruitment. Use Alpha Node commander instead.", "warning", Some(ctx.mission_id.clone()));
                        return Err(ToolExecutionError::HierarchyBlocked("As COO, you are prohibited from direct worker recruitment. You MUST recruit an Alpha Node (ID: alpha) to serve as Swarm Mission Commander.".to_string()));
                    }
                }
            }
        }

        // 2. [CBS] Skill-Based Security Allowlist
        if let Some(agent) = runner.state.registry.agents.get(&ctx.agent_id) {
            let allowed_skills = &agent.value().capabilities.skills;
            let is_builtin = super::registry::BUILTIN_TOOLS.contains(&fc.name.as_str());

            if !is_builtin && !allowed_skills.contains(&fc.name) {
                tracing::warn!(
                    "🛡️ [CBS] Agent {} attempted unauthorized skill: {}",
                    ctx.agent_id,
                    fc.name
                );
                runner.broadcast_sys(
                    &format!(
                        "🛡️ CBS: {} attempted unauthorized skill: {}",
                        ctx.name, fc.name
                    ),
                    "error",
                    Some(ctx.mission_id.clone()),
                );

                let suggestion = if allowed_skills.contains(&"spawn_subagent".to_string())
                    || allowed_skills.contains(&"send_mission_directive".to_string())
                    || allowed_skills.contains(&"issue_alpha_directive".to_string())
                    || ctx.agent_id == AGENT_CEO
                    || ctx.agent_id == AGENT_COO
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

        // 3. [Dynamic Policy] Check SQLite-backed PermissionPolicy first
        let agent_role = runner
            .state
            .registry
            .agents
            .get(&ctx.agent_id)
            .map(|a| a.value().identity.role.clone());
        let policy_mode = runner
            .state
            .security
            .permission_policy
            .get_mode(Some(&ctx.agent_id), agent_role.as_deref(), &fc.name)
            .await;
        match policy_mode {
            crate::security::permissions::PermissionMode::Deny => {
                return Err(ToolExecutionError::SecurityBlocked(format!(
                    "Policy for '{}' is set to DENY",
                    fc.name
                )));
            }
            crate::security::permissions::PermissionMode::Prompt => {
                trigger_oversight = true;
                oversight_reason =
                    format!("Sovereign Policy requires 'Prompt' for tool: {}", fc.name);
            }
            crate::security::permissions::PermissionMode::Allow => {}
        }

        if !trigger_oversight {
            // 4. [Security Gate] Skill Manifest Validation
            let mut manifest_requires = false;

            if let Some(manifest) = runner.state.registry.skill_registry.get(&fc.name) {
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
        if let Some(agent) = runner.state.registry.agents.get(&ctx.agent_id) {
            if agent.value().requires_oversight {
                trigger_oversight = true;
                oversight_reason = format!("Mandatory oversight enabled for agent: {}", ctx.name);
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
        ctx.agent_id = AGENT_CEO.to_string();

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
    async fn test_pre_validate_cbs_unauthorized_skill_blocked() {
        let state = Arc::new(crate::state::AppState::new_minimal_mock().await);
        let runner = AgentRunner::new(state.clone());
        let mut ctx = RunContext::default();
        ctx.agent_id = "specialist-1".to_string();

        let mut agent = EngineAgent::default();
        agent.identity.id = ctx.agent_id.clone();
        agent.capabilities.skills = vec!["custom_allowed_skill".to_string()];
        state.registry.agents.insert(ctx.agent_id.clone(), agent);

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
}

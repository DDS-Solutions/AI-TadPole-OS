//! @docs ARCHITECTURE:Registry
//! 
//! ### AI Assist Note
//! **Core technical resource for the Tadpole OS Sovereign infrastructure.**
//! This module implements high-fidelity logic for the Sovereign Reality layer.
//! 
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: Runtime logic error, state desynchronization, or resource exhaustion.
//! - **Telemetry Link**: Search `[security]` in tracing logs.

use crate::agent::runner::RunContext;
use crate::agent::types::ToolCall;
use crate::agent::runner::AgentRunner;
use crate::agent::constants::{AGENT_CEO, AGENT_COO, AGENT_ALPHA};
use super::error::ToolExecutionError;

pub struct ValidationResult {
    pub oversight_required: bool,
    pub oversight_reason: String,
}

#[async_trait::async_trait]
pub trait SecurityManager: Send + Sync {
    async fn pre_validate(&self, runner: &AgentRunner, ctx: &RunContext, fc: &ToolCall) -> Result<ValidationResult, ToolExecutionError>;
}

pub struct DefaultSecurityManager;

#[async_trait::async_trait]
impl SecurityManager for DefaultSecurityManager {
    async fn pre_validate(&self, runner: &AgentRunner, ctx: &RunContext, fc: &ToolCall) -> Result<ValidationResult, ToolExecutionError> {
        let mut trigger_oversight = false;
        let mut oversight_reason = String::new();

        // 1. [CBS] Skill-Based Security Allowlist
        if let Some(agent) = runner.state.registry.agents.get(&ctx.agent_id) {
            let allowed_skills = &agent.value().capabilities.skills;
            let is_builtin = super::registry::BUILTIN_TOOLS.contains(&fc.name.as_str());

            if !is_builtin && !allowed_skills.is_empty() && !allowed_skills.contains(&fc.name) {
                tracing::warn!("🛡️ [CBS] Agent {} attempted unauthorized skill: {}", ctx.agent_id, fc.name);
                runner.broadcast_sys(
                    &format!("🛡️ CBS: {} attempted unauthorized skill: {}", ctx.name, fc.name),
                    "error",
                    Some(ctx.mission_id.clone()),
                );
                return Err(ToolExecutionError::SecurityBlocked(format!("Skill '{}' not in agent allowlist", fc.name)));
            }

            // 2. [Hierarchy Guard] Enforce strategic delegation for CEO/COO
            if matches!(fc.name.as_str(), "spawn_subagent" | "recruit_specialist") {
                if ctx.agent_id == AGENT_CEO {
                    tracing::warn!("🛡️ [Hierarchy Guard] CEO (ID: {}) blocked from spawning specialists directly.", AGENT_CEO);
                    runner.broadcast_sys("🛡️ Hierarchy Guard: CEO blocked from direct worker recruitment. Use 'issue_alpha_directive' instead.", "warning", Some(ctx.mission_id.clone()));
                    return Err(ToolExecutionError::HierarchyBlocked("As CEO, you are prohibited from direct worker recruitment. You MUST use 'issue_alpha_directive' to delegate complex missions to the COO.".to_string()));
                }
                if ctx.agent_id == AGENT_COO {
                    let target = fc.args.get("agent_id").and_then(|v| v.as_str()).unwrap_or("");
                    if target != AGENT_ALPHA {
                        tracing::warn!("🛡️ [Hierarchy Guard] COO (ID: {}) blocked from spawning specialist '{}' directly.", AGENT_COO, target);
                        runner.broadcast_sys("🛡️ Hierarchy Guard: COO blocked from direct worker recruitment. Use Alpha Node commander instead.", "warning", Some(ctx.mission_id.clone()));
                        return Err(ToolExecutionError::HierarchyBlocked("As COO, you are prohibited from direct worker recruitment. You MUST recruit an Alpha Node (ID: alpha) to serve as Swarm Mission Commander.".to_string()));
                    }
                }
            }
        }

        // 3. [Dynamic Policy] Check SQLite-backed PermissionPolicy first
        let policy_mode = runner.state.security.permission_policy.get_mode(&fc.name).await;
        match policy_mode {
            crate::security::permissions::PermissionMode::Deny => {
                return Err(ToolExecutionError::SecurityBlocked(format!("Policy for '{}' is set to DENY", fc.name)));
            }
            crate::security::permissions::PermissionMode::Prompt => {
                trigger_oversight = true;
                oversight_reason = format!("Sovereign Policy requires 'Prompt' for tool: {}", fc.name);
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


// Metadata: [security]

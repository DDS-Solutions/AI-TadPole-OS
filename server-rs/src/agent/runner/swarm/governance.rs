//! @docs ARCHITECTURE:Runner
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Agent Runner / governance
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: `[Governance]`
//!
//! ### AI Assist Note
//! - Keep cancellation ownership and usage persistence aligned with the existing execution paths.

use crate::agent::constants::{AGENT_ALPHA, AGENT_COO};
use crate::agent::runner::tools::error::ToolExecutionError;
use crate::agent::runner::{AgentRunner, RunContext};

impl AgentRunner {
    /// Pre-flight validation & dual-gate reactivation for suspended agents.
    pub(crate) async fn validate_and_reactivate_agents(
        &self,
        target_ids: &[String],
        ctx: &RunContext,
    ) -> Result<(), ToolExecutionError> {
        for sub_agent_id in target_ids {
            let token_lower = sub_agent_id.trim().to_lowercase();
            let query_tokens: Vec<&str> = token_lower
                .split(|c: char| !c.is_alphanumeric())
                .filter(|w| !w.is_empty())
                .collect();
            // Deterministic candidate matching across ID, name, role, skills and workflows (Defects #12 & #15)
            let mut matched_agent: Option<crate::agent::types::EngineAgent> = None;
            let mut best_score: u32 = 0;

            for kv in self.state.registry.agents.iter() {
                let a = kv.value();
                if let Ok(Some(score)) = super::recruitment::score_candidate(
                    a,
                    &token_lower,
                    &query_tokens,
                    Some(&ctx.mission_state.mission_id),
                ) {
                    if score > best_score {
                        best_score = score;
                        matched_agent = Some(a.clone());
                    }
                }
            }

            if let Some(agent) = matched_agent {
                if agent.health.status == "suspended" {
                    tracing::info!("⚖️ [Governance] Pre-flight: Intercepted recruitment for suspended agent '{}' (ID: {}). Requesting authorization.", agent.identity.name, agent.identity.id);

                    let mut coo_approved = false;
                    let mut coo_rationale = "Initiator is the COO.".to_string();

                    // Check if the initiator is the COO via ID constants or exact role match
                    let is_initiator_coo = ctx.identity.agent_id == AGENT_COO
                        || ctx.identity.agent_id == AGENT_ALPHA
                        || self
                            .state
                            .registry
                            .agents
                            .get(&ctx.identity.agent_id)
                            .map(|a| {
                                let role = a.identity.role.to_lowercase();
                                role.split(|c: char| !c.is_alphanumeric())
                                    .any(|w| w == "coo")
                                    || role.contains("chief operations officer")
                                    || role.contains("chief operating officer")
                            })
                            .unwrap_or(false);

                    if is_initiator_coo {
                        coo_approved = true;
                    } else {
                        // Gate 1: COO Approval Check
                        let coo_agent = self
                            .state
                            .registry
                            .agents
                            .get(AGENT_ALPHA)
                            .map(|entry| entry.value().clone())
                            .or_else(|| {
                                self.state
                                    .registry
                                    .agents
                                    .get(AGENT_COO)
                                    .map(|entry| entry.value().clone())
                            });

                        let mut coo_available = false;
                        if let Some(coo_ref) = coo_agent {
                            let coo = &coo_ref;
                            if coo.health.status != "suspended" {
                                match coo.resolve_provider_context(
                                    self.state.base_dir.clone(),
                                    ctx.mission_state.cluster_id.as_deref(),
                                ) {
                                    Ok(mut coo_ctx) => {
                                        coo_available = true;
                                        coo_ctx.mission_state.mission_id =
                                            ctx.mission_state.mission_id.clone();
                                        coo_ctx.mission_state.budget_usd =
                                            ctx.mission_state.budget_usd;
                                        coo_ctx.mission_state.current_cost_usd =
                                            ctx.mission_state.current_cost_usd;
                                        coo_ctx.execution_metrics = ctx.execution_metrics.clone();
                                        coo_ctx.dispatch_accounted = ctx.dispatch_accounted.clone();

                                        let system_prompt = "You are the Tadpole OS COO (Tadpole Alpha). You oversee all specialist recruitments in the swarm. A task needs the specialized capabilities of a suspended agent. Review and respond with exactly 'YES' or 'NO' (followed by a 1-sentence rationale) to authorize or deny the reactivation of this suspended agent.";
                                        let user_message = format!(
                                            "Caller Agent: '{}'\nTarget Agent for Reactivation: '{}' ({})\nTarget Role: '{}'\nTarget Description: '{}'\n\nDo you authorize this reactivation?",
                                            ctx.identity.agent_id, agent.identity.name, agent.identity.id, agent.identity.role, agent.identity.description
                                        );

                                        match self
                                            .call_provider(
                                                &coo_ctx,
                                                system_prompt,
                                                &user_message,
                                                None,
                                            )
                                            .await
                                        {
                                            Ok(resp) => {
                                                let trimmed = resp.0.trim();
                                                if trimmed.to_uppercase().starts_with("YES") {
                                                    coo_approved = true;
                                                    coo_rationale = trimmed.to_string();
                                                    tracing::info!("⚖️ [Governance] COO approved reactivation of '{}'. Rationale: {}", agent.identity.id, coo_rationale);
                                                } else {
                                                    coo_rationale = trimmed.to_string();
                                                    tracing::warn!("⚖️ [Governance] COO denied reactivation of '{}'. Rationale: {}", agent.identity.id, coo_rationale);
                                                }
                                            }
                                            Err(e) => {
                                                tracing::error!("⚖️ [Governance] Failed to contact COO for approval: {}", e);
                                                coo_rationale =
                                                    format!("Failed to contact COO: {}", e);
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        tracing::warn!("⚠️ [Governance] Failed to resolve COO provider context: {}. Delegating to Overlord.", e);
                                        // coo_available remains false; falls through to Gate 2 Overlord
                                    }
                                }
                            }
                        }

                        // COO Paradox Fallback: If COO is suspended, not in registry, or provider context unavailable
                        if !coo_available {
                            coo_approved = true; // Bypass to Gate 2 Overlord
                            coo_rationale = "COO is suspended or unavailable. Authorization delegated directly to Overlord.".to_string();
                            tracing::warn!("⚖️ [Governance] COO is suspended or unavailable. Bypassing COO authorization and delegating directly to Overlord.");
                        }
                    }

                    if !coo_approved {
                        return Err(ToolExecutionError::ExecutionFailed(format!(
                            "RECRUITMENT_FAILED: Reactivation of suspended agent '{}' was denied by COO. Rationale: {}",
                            agent.identity.id, coo_rationale
                        )));
                    }

                    // Gate 2: Overlord (User) Approval Check
                    tracing::info!(
                        "⚖️ [Governance] COO approved. Requesting Overlord (User) sign-off."
                    );
                    let tool_call_audit = crate::agent::types::ToolCallAudit {
                        id: uuid::Uuid::new_v4().to_string(),
                        mission_id: Some(ctx.mission_state.mission_id.clone()),
                        agent_id: ctx.identity.agent_id.clone(),
                        skill: "reactivate_agent".to_string(),
                        params: serde_json::json!({
                            "target_agent_id": agent.identity.id,
                            "target_name": agent.identity.name,
                            "role": agent.identity.role,
                            "coo_rationale": coo_rationale
                        }),
                        department: "governance".to_string(),
                        description: format!(
                            "Authorize reactivation of suspended agent '{}' ({})",
                            agent.identity.name, agent.identity.id
                        ),
                        timestamp: chrono::Utc::now().to_rfc3339(),
                    };

                    let user_approved = match self
                        .submit_oversight(
                            tool_call_audit,
                            Some(ctx.mission_state.mission_id.clone()),
                        )
                        .await
                    {
                        Ok(approved) => approved,
                        Err(e) => {
                            tracing::error!("⚖️ [Governance] Oversight system failed: {}", e);
                            return Err(ToolExecutionError::ExecutionFailed(format!(
                                "OVERSIGHT_FAILED: The oversight approval system was unreachable or failed: {}",
                                e
                            )));
                        }
                    };

                    if !user_approved {
                        tracing::warn!(
                            "⚖️ [Governance] Overlord (User) denied reactivation of '{}'.",
                            agent.identity.id
                        );
                        return Err(ToolExecutionError::ExecutionFailed(format!(
                            "RECRUITMENT_FAILED: Reactivation of suspended agent '{}' was denied by Overlord (User).",
                            agent.identity.id
                        )));
                    }

                    tracing::info!(
                        "⚖️ [Governance] Reactivation of '{}' approved; persistence will occur in the short recruitment transaction.",
                        agent.identity.id
                    );
                }
            }
        }
        Ok(())
    }
}

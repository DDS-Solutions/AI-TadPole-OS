//! @docs ARCHITECTURE:Runner
//!
//! ### AI Assist Note
//! **Swarm Governance**: Coordinates pre-flight recruitment validation, COO approval checks,
//! and Overlord/User signature-gate authorization for suspended agents.
//!
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: COO agent unreachable, database transaction lock contention, user rejection.

use crate::agent::runner::{AgentRunner, RunContext};
use crate::agent::constants::{AGENT_COO, AGENT_ALPHA};
use crate::agent::runner::tools::error::ToolExecutionError;

impl AgentRunner {
    /// Pre-flight validation & dual-gate reactivation for suspended agents.
    pub(crate) async fn validate_and_reactivate_agents(
        &self,
        conn: &mut sqlx::SqliteConnection,
        target_ids: &[String],
        ctx: &RunContext,
    ) -> Result<(), ToolExecutionError> {
        for sub_agent_id in target_ids {
            let mut matched_agent = None;
            for kv in self.state.registry.agents.iter() {
                let a = kv.value();
                let is_match = a.identity.id == *sub_agent_id
                    || a.identity.name.eq_ignore_ascii_case(sub_agent_id)
                    || a.identity.role.to_lowercase().contains(&sub_agent_id.to_lowercase());
                if is_match {
                    matched_agent = Some(a.clone());
                    break;
                }
            }

            if let Some(mut agent) = matched_agent {
                if agent.health.status == "suspended" {
                    tracing::info!("⚖️ [Governance] Pre-flight: Intercepted recruitment for suspended agent '{}' (ID: {}). Requesting authorization.", agent.identity.name, agent.identity.id);
                    
                    let mut coo_approved = false;
                    let mut coo_rationale = "Initiator is the COO.".to_string();

                    // Check if the initiator is the COO via ID constants or role
                    let is_initiator_coo = ctx.agent_id == AGENT_COO 
                        || ctx.agent_id == AGENT_ALPHA
                        || self.state.registry.agents.get(&ctx.agent_id)
                            .map(|a| a.identity.role.to_lowercase().contains("coo") || a.identity.role.to_lowercase().contains("chief operations officer"))
                            .unwrap_or(false);

                    if is_initiator_coo {
                        coo_approved = true;
                    } else {
                        // Gate 1: COO Approval Check
                        let coo_agent = self.state.registry.agents.get(AGENT_ALPHA)
                            .or_else(|| self.state.registry.agents.get(AGENT_COO));
                        
                        let mut coo_available = false;
                        if let Some(coo_ref) = coo_agent {
                            let coo = coo_ref.value();
                            if coo.health.status != "suspended" {
                                coo_available = true;
                                let mut coo_ctx = coo.resolve_provider_context(self.state.base_dir.clone());
                                coo_ctx.mission_id = ctx.mission_id.clone();
                                
                                let client = (*self.state.resources.http_client).clone();
                                let provider = self.resolve_provider(&coo_ctx, client).await;
                                
                                let system_prompt = "You are the Tadpole OS COO (Tadpole Alpha). You oversee all specialist recruitments in the swarm. A task needs the specialized capabilities of a suspended agent. Review and respond with exactly 'YES' or 'NO' (followed by a 1-sentence rationale) to authorize or deny the reactivation of this suspended agent.";
                                let user_message = format!(
                                    "Caller Agent: '{}'\nTarget Agent for Reactivation: '{}' ({})\nTarget Role: '{}'\nTarget Description: '{}'\n\nDo you authorize this reactivation?",
                                    ctx.agent_id, agent.identity.name, agent.identity.id, agent.identity.role, agent.identity.description
                                );
                                
                                match provider.generate(system_prompt, &user_message, None).await {
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
                                        coo_rationale = format!("Failed to contact COO: {}", e);
                                    }
                                }
                            }
                        }

                        // COO Paradox Fallback: If COO is suspended or not in registry
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
                    tracing::info!("⚖️ [Governance] COO approved. Requesting Overlord (User) sign-off.");
                    let tool_call_audit = crate::agent::types::ToolCallAudit {
                        id: uuid::Uuid::new_v4().to_string(),
                        mission_id: Some(ctx.mission_id.clone()),
                        agent_id: ctx.agent_id.clone(),
                        skill: "reactivate_agent".to_string(),
                        params: serde_json::json!({
                            "target_agent_id": agent.identity.id,
                            "target_name": agent.identity.name,
                            "role": agent.identity.role,
                            "coo_rationale": coo_rationale
                        }),
                        department: "governance".to_string(),
                        description: format!("Authorize reactivation of suspended agent '{}' ({})", agent.identity.name, agent.identity.id),
                        timestamp: chrono::Utc::now().to_rfc3339(),
                    };

                    let user_approved = match self.submit_oversight(tool_call_audit, Some(ctx.mission_id.clone())).await {
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
                        tracing::warn!("⚖️ [Governance] Overlord (User) denied reactivation of '{}'.", agent.identity.id);
                        return Err(ToolExecutionError::ExecutionFailed(format!(
                            "RECRUITMENT_FAILED: Reactivation of suspended agent '{}' was denied by Overlord (User).",
                            agent.identity.id
                        )));
                    }

                    tracing::info!("⚖️ [Governance] Reactivating agent '{}' in registry and database.", agent.identity.id);
                    agent.health.status = "idle".to_string();
                    
                    // Update in database using shared transaction/connection
                    crate::agent::persistence::save_agent_db_in_tx(conn, &mut agent).await.map_err(|e| ToolExecutionError::ExecutionFailed(e.to_string()))?;

                    // Update in-memory registry
                    self.state.registry.agents.insert(agent.identity.id.clone(), agent.clone());
                }
            }
        }
        Ok(())
    }
}

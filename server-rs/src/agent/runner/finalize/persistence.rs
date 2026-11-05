//! @docs ARCHITECTURE:Runner
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Agent Runner / finalize / persistence
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: `[Runner]`

use crate::agent::runner::service_traits::mission_state;
use crate::agent::runner::{safe_truncate_str, AgentRunner, RunContext};
use crate::error::AppError;

impl AgentRunner {
    /// Finalizes mission state and logs the steps to SQLite.
    pub(crate) async fn finalize_mission_persistence(
        &self,
        ctx: &RunContext,
        output_text: &str,
        usage: &Option<crate::agent::types::TokenUsage>,
        final_cumulative_cost: f64,
    ) -> Result<(), AppError> {
        let accounted = ctx
            .dispatch_accounted
            .load(std::sync::atomic::Ordering::Acquire);
        if accounted {
            self.state.security.budget_guard.flush_to_db().await?;
        }
        let pending_cost = if accounted {
            0.0
        } else {
            final_cumulative_cost
        };
        let shares_parent_mission = ctx.mission_state.depth > 0
            && crate::agent::mission::get_mission_by_id(
                &self.state.resources.pool,
                &ctx.mission_state.mission_id,
            )
            .await?
            .is_some_and(|mission| mission.agent_id != ctx.identity.agent_id);
        let mut has_partial_failure = ctx
            .swarm_partial_failure
            .load(std::sync::atomic::Ordering::Acquire);

        // For root missions, check if any steps for this mission logged an error or branch failure.
        // Queries mission_logs by severity = 'error'. Fails closed (pauses) on database query errors.
        if !shares_parent_mission && !has_partial_failure {
            match sqlx::query_scalar::<_, i64>(
                "SELECT COUNT(1) FROM mission_logs WHERE mission_id = ?1 AND severity = 'error'",
            )
            .bind(&ctx.mission_state.mission_id)
            .fetch_one(&self.state.resources.pool)
            .await
            {
                Ok(count) => {
                    if count > 0 {
                        has_partial_failure = true;
                        ctx.swarm_partial_failure
                            .store(true, std::sync::atomic::Ordering::Release);
                    }
                }
                Err(err) => {
                    tracing::error!(
                        "⚠️ [Runner] Failed to query error steps for mission {}: {}. Failing closed by pausing mission.",
                        ctx.mission_state.mission_id,
                        err
                    );
                    has_partial_failure = true;
                    ctx.swarm_partial_failure
                        .store(true, std::sync::atomic::Ordering::Release);
                }
            }
        }

        let human_review_approved = ctx
            .swarm_review_approved
            .load(std::sync::atomic::Ordering::Acquire);
        let verification = mission_state::verification_status(ctx);
        let requires_human_review = (has_partial_failure && !human_review_approved)
            || verification == mission_state::VerificationStatus::ReviewRequired;

        let (receipt_status, step_status) = if shares_parent_mission {
            crate::agent::mission::add_mission_cost(
                &self.state.resources.pool,
                &ctx.mission_state.mission_id,
                pending_cost,
            )
            .await?;
            ("branch_completed", "success")
        } else {
            let mission_status = if requires_human_review {
                crate::agent::types::MissionStatus::Paused
            } else {
                crate::agent::types::MissionStatus::Completed
            };
            crate::agent::mission::update_mission(
                &self.state.resources.pool,
                &ctx.mission_state.mission_id,
                mission_status,
                Some(pending_cost),
            )
            .await?;
            if requires_human_review {
                ("paused_for_human_review", "warning")
            } else {
                ("completed", "success")
            }
        };

        let modified = ctx.modified_files.lock().clone();
        let accessed = ctx.last_accessed_files.lock().clone();
        let read_files: Vec<String> = accessed
            .into_iter()
            .filter(|f| !modified.contains(f))
            .collect();

        let receipt = serde_json::json!({
            "receipt": {
                "read_files": read_files,
                "modified_files": modified,
                "commands_run": ctx.commands_run.lock().clone(),
                "token_usage": usage,
                "cost_usd": final_cumulative_cost,
                "cost_is_estimate": true,
                "status": receipt_status,
                "branch_depth": ctx.mission_state.depth,
                "requires_human_review": requires_human_review,
                "verification_status": verification.as_str()
            }
        });

        crate::agent::mission::log_step(
            &self.state.resources.pool,
            &ctx.mission_state.mission_id,
            &ctx.identity.agent_id,
            "Agent",
            output_text,
            step_status,
            Some(receipt),
        )
        .await?;

        // Complete any pending directives that targeted this agent
        if receipt_status == "completed" {
            if let Err(e) =
                crate::agent::runner::swarm_persistence::complete_pending_directives_for_agent(
                    &self.state.resources.pool,
                    &ctx.identity.agent_id,
                    Some(output_text),
                )
                .await
            {
                tracing::warn!(
                    "⚠️ [Runner] Failed to complete pending directives for agent {}: {}",
                    ctx.identity.agent_id,
                    e
                );
            }
        }

        // ── TrustGraph Mission Resolution Hook ─────────────────────
        let tg_pool = self.state.resources.pool.clone();
        let tg_mission_id = ctx.mission_state.mission_id.clone();
        let tg_receipt_status = receipt_status.to_string();
        let tg_summary = if output_text.len() > 256 {
            format!("{}...", safe_truncate_str(output_text, 256))
        } else {
            output_text.to_string()
        };

        tokio::spawn(async move {
            let engine = crate::agent::trustgraph::TrustGraphEngine::new(tg_pool);
            let mission_node = crate::agent::trustgraph::TrustGraphNode {
                id: format!("mission:{}", tg_mission_id),
                name: format!("Mission {}", tg_mission_id),
                r#type: "MISSION".to_string(),
                description: Some(format!("Mission run {}", tg_mission_id)),
                mission_id: Some(tg_mission_id.clone()),
            };
            if let Err(e) = engine.add_node(&mission_node).await {
                tracing::warn!("Failed to record mission node in TrustGraph: {}", e);
                return;
            }
            let outcome_id = format!("outcome:{}:{}", tg_mission_id, tg_receipt_status);
            let outcome_node = crate::agent::trustgraph::TrustGraphNode {
                id: outcome_id.clone(),
                name: format!("Outcome: {}", tg_receipt_status),
                r#type: "OUTCOME".to_string(),
                description: Some(tg_summary),
                mission_id: Some(tg_mission_id.clone()),
            };
            if let Err(e) = engine.add_node(&outcome_node).await {
                tracing::warn!("Failed to record outcome node in TrustGraph: {}", e);
                return;
            }
            let rel = crate::agent::trustgraph::TrustGraphRelation {
                id: format!("rel:resolved:{}:{}", tg_mission_id, uuid::Uuid::new_v4()),
                source_entity_id: mission_node.id,
                target_entity_id: outcome_node.id,
                relation_type: "RESOLVED".to_string(),
                weight: 1.0,
                mission_id: Some(tg_mission_id),
            };
            if let Err(e) = engine.add_relation(&rel).await {
                tracing::warn!("Failed to record RESOLVED relation in TrustGraph: {}", e);
            }
        });

        Ok(())
    }
}

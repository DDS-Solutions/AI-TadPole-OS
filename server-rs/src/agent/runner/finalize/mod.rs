//! @docs ARCHITECTURE:Runner
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Agent Runner / finalize
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: `[Runner]`
//!
//! ### AI Assist Note
//! - Keep cancellation ownership and usage persistence aligned with the existing execution paths.
//! - A child closes its own mission row while leaving a shared parent ledger open.

pub mod accounting;
pub mod failures;
pub mod memory;
pub mod persistence;

#[cfg(test)]
mod tests;

use super::analysis::spawn_post_mission_analysis;
use super::service_traits::mission_state::{persist_agent_outcome, AgentOutcome, OutcomeHealth};
use super::{AgentRunner, RunContext};
use crate::error::AppError;

impl AgentRunner {
    /// Finalizes the run: persists mission state, updates token usage, broadcasts results.
    ///
    /// Non-negotiable invariant: Mission outcome must be persisted to the database BEFORE
    /// broadcasting the assistant delivery or transitioning the agent status to `idle`.
    pub(crate) async fn finalize_run(
        &self,
        ctx: &RunContext,
        output_text: &str,
        usage: &Option<crate::agent::types::TokenUsage>,
    ) -> Result<String, AppError> {
        tracing::info!(
            "🔔 [DIAGNOSTIC] Finalizing run for agent {}. Content length: {}",
            ctx.identity.agent_id,
            output_text.len()
        );
        tracing::info!(
            "✅ [Runner] Provider responded successfully ({} tokens). Output length: {}",
            usage.as_ref().map(|u| u.total_tokens).unwrap_or(0),
            output_text.len()
        );
        if output_text.is_empty() {
            tracing::warn!(
                "⚠️ [Runner] final_delivery is EMPTY for agent {}",
                ctx.identity.agent_id
            );
        }

        let effective_model = self.effective_model_id(ctx);
        let provider_str = ctx.model_config.provider.to_string();
        let turn_cost = usage
            .as_ref()
            .map(|u| {
                crate::agent::rates::calculate_cost_with_custom_rates(
                    ctx.model_config.input_cost_per_m,
                    ctx.model_config.output_cost_per_m,
                    Some(&provider_str),
                    &effective_model,
                    u.input_tokens,
                    u.output_tokens,
                )
            })
            .unwrap_or(0.0);
        let dispatch_accounted = ctx
            .dispatch_accounted
            .load(std::sync::atomic::Ordering::Acquire);
        let turn_cost = if dispatch_accounted {
            ctx.execution_metrics
                .snapshot(&ctx.mission_state.mission_id, &ctx.identity.agent_id)
                .total_cost_usd
        } else {
            turn_cost
        };
        let economic_cost = if dispatch_accounted { 0.0 } else { turn_cost };

        // Delivery formatting and tag sanitization
        let mut final_delivery = output_text.trim().to_string();
        if final_delivery.is_empty() {
            ctx.swarm_partial_failure
                .store(true, std::sync::atomic::Ordering::Release);
            ctx.swarm_review_approved
                .store(false, std::sync::atomic::Ordering::Release);
            final_delivery = "MISSION_REVIEW_REQUIRED: No final report was produced. Inspect the execution evidence before retrying.".to_string();
        }
        if super::service_traits::mission_state::verification_status(ctx)
            == super::service_traits::mission_state::VerificationStatus::ReviewRequired
            && !final_delivery.starts_with("MISSION_REVIEW_REQUIRED")
        {
            final_delivery = format!("MISSION_REVIEW_REQUIRED: No fresh passing execution witness was recorded.\n\n{final_delivery}");
        }
        final_delivery = Self::scrub_mythos_tags(&final_delivery);

        tracing::debug!(
            "DEBUG [Runner] final_delivery sanitized content: {:?}",
            final_delivery
        );

        // 💾 PERSISTENCE FIRST: Persist mission outcome and steps BEFORE broadcasting or setting idle.
        // If persistence fails, route through fail_mission cleanly rather than leaving state inconsistent.
        if let Err(e) = self
            .finalize_mission_persistence(ctx, &final_delivery, usage, turn_cost)
            .await
        {
            tracing::error!(
                "❌ [Runner] Failed to persist mission completion for agent {}: {}",
                ctx.identity.agent_id,
                e
            );
            let _ = self.fail_mission(ctx, &e, usage).await;
            return Err(e);
        }

        persist_agent_outcome(
            &self.state,
            &ctx.identity.agent_id,
            ctx.execution_generation,
            AgentOutcome {
                health: OutcomeHealth::Succeeded,
                usage: usage.clone(),
                tokens_delta: if dispatch_accounted {
                    0
                } else {
                    usage.as_ref().map_or(0, |u| u.total_tokens)
                },
                cost_delta: economic_cost,
                dispatch_accounted,
            },
        )
        .await?;

        // Record to persistent budget guard if turn incurred a cost
        if !dispatch_accounted && turn_cost > 0.0 {
            if let Err(e) = self
                .state
                .security
                .budget_guard
                .record_usage(&ctx.identity.agent_id, turn_cost)
                .await
            {
                tracing::error!(
                    "❌ [Runner] Failed to persist budget usage for agent {} (cost: {}): {}",
                    ctx.identity.agent_id,
                    turn_cost,
                    e
                );
            }
        }

        // 📢 BROADCAST & STATUS: Broadcast assistant message and transition to idle
        // ONLY after database persistence has succeeded.
        self.broadcast_agent_message(
            &ctx.identity.agent_id,
            &ctx.mission_state.mission_id,
            &final_delivery,
            "assistant",
            0,
        );

        // 📊 [Execution Metrics Rollup] Emit final mission metrics snapshot
        let final_metrics = ctx
            .execution_metrics
            .snapshot(&ctx.mission_state.mission_id, &ctx.identity.agent_id);
        let _ = self.state.comms.telemetry_tx.send(serde_json::json!({
            "type": "agent:execution_metrics",
            "metrics": final_metrics,
        }));

        self.update_status(
            &ctx.identity.agent_id,
            &ctx.mission_state.mission_id,
            "idle",
            None,
        );

        #[cfg(feature = "vector-memory")]
        self.archive_lance_db_memory(ctx, &final_delivery);

        // 🧠 MISSION ANALYSIS TRIGGER
        if ctx.analysis && ctx.identity.agent_id != super::analysis::QA_AUDITOR_ID {
            spawn_post_mission_analysis(self.clone(), ctx.clone(), final_delivery.clone());
        }

        Ok(final_delivery)
    }
}

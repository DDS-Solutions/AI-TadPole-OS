//! @docs ARCHITECTURE:Runner
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Agent Runner / finalize / failures
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: `[Runner]`

use crate::agent::runner::service_traits::mission_state::{
    persist_agent_outcome, AgentOutcome, OutcomeHealth,
};
use crate::agent::runner::{AgentRunner, RunContext};
use crate::error::AppError;

impl AgentRunner {
    /// Centralized mission failure handler. Sets status, logs error, and updates agent health.
    ///
    /// Non-negotiable invariant: Persist mission failure state and audit log step to SQLite
    /// BEFORE broadcasting failure to the UI or transitioning agent status to idle.
    pub(crate) async fn fail_mission(
        &self,
        ctx: &RunContext,
        e: &AppError,
        usage: &Option<crate::agent::types::TokenUsage>,
    ) -> Result<(), AppError> {
        let safe_error = self
            .state
            .security
            .secret_redactor
            .redact(&format!("{}", e));
        tracing::error!(
            "❌ [Runner] Mission failure for agent {}: {}",
            ctx.identity.agent_id,
            safe_error
        );

        let model_id = self.effective_model_id(ctx);
        let provider_str = ctx.model_config.provider.to_string();
        let turn_cost = usage
            .as_ref()
            .map(|u| {
                crate::agent::rates::calculate_cost_with_custom_rates(
                    ctx.model_config.input_cost_per_m,
                    ctx.model_config.output_cost_per_m,
                    Some(&provider_str),
                    &model_id,
                    u.input_tokens,
                    u.output_tokens,
                )
            })
            .unwrap_or(0.0);
        let turn_tokens = usage.as_ref().map(|u| u.total_tokens).unwrap_or(0);
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
        let turn_tokens = if dispatch_accounted { 0 } else { turn_tokens };
        if dispatch_accounted {
            self.state.security.budget_guard.flush_to_db().await?;
        }

        // Propagate branch failure to parent context if running within a swarm branch
        if ctx.mission_state.depth > 0 {
            ctx.swarm_partial_failure
                .store(true, std::sync::atomic::Ordering::SeqCst);
        }

        let modified = ctx.modified_files.lock().clone();
        let accessed = ctx.last_accessed_files.lock().clone();
        let read_files: Vec<String> = accessed
            .into_iter()
            .filter(|f| !modified.contains(f))
            .collect();

        // 💾 PERSISTENCE FIRST: Persist mission failure state and audit log step to SQLite
        // BEFORE broadcasting failure to the UI or transitioning agent status to idle.
        if ctx.mission_state.depth > 0 {
            crate::agent::mission::add_mission_cost(
                &self.state.resources.pool,
                &ctx.mission_state.mission_id,
                economic_cost,
            )
            .await?;
        } else {
            crate::agent::mission::update_mission(
                &self.state.resources.pool,
                &ctx.mission_state.mission_id,
                crate::agent::types::MissionStatus::Failed,
                Some(economic_cost),
            )
            .await?;
        }

        let receipt = serde_json::json!({
            "receipt": {
                "read_files": read_files,
                "modified_files": modified,
                "commands_run": ctx.commands_run.lock().clone(),
                "token_usage": usage,
                "cost_usd": turn_cost,
                "status": if ctx.mission_state.depth > 0 { "branch_failed" } else { "failed" },
                "branch_depth": ctx.mission_state.depth,
                "error": safe_error
            }
        });

        crate::agent::mission::log_step(
            &self.state.resources.pool,
            &ctx.mission_state.mission_id,
            &ctx.identity.agent_id,
            "System",
            &format!("❌ Error: {}", safe_error),
            "error",
            Some(receipt),
        )
        .await?;

        persist_agent_outcome(
            &self.state,
            &ctx.identity.agent_id,
            ctx.execution_generation,
            AgentOutcome {
                health: OutcomeHealth::Failed,
                usage: None,
                tokens_delta: turn_tokens,
                cost_delta: economic_cost,
                dispatch_accounted,
            },
        )
        .await?;

        // Record final cost if usage was provided and cost > 0
        if !dispatch_accounted && turn_cost > 0.0 {
            if let Err(e) = self
                .state
                .security
                .budget_guard
                .record_usage(&ctx.identity.agent_id, turn_cost)
                .await
            {
                tracing::error!(
                    "❌ [Runner] Failed to persist budget usage during failure finalization for agent {} (cost: {}): {}",
                    ctx.identity.agent_id,
                    turn_cost,
                    e
                );
            }
        }

        // 3. 📢 BROADCAST & STATUS: Broadcast error message and transition to idle
        // ONLY after database persistence has succeeded.
        self.broadcast_agent_message(
            &ctx.identity.agent_id,
            &ctx.mission_state.mission_id,
            &format!("❌ Error: {}", safe_error),
            "system",
            0,
        );

        self.update_status(
            &ctx.identity.agent_id,
            &ctx.mission_state.mission_id,
            "idle",
            None,
        );

        Ok(())
    }

    /// Centralized mission failure handler for setup and pre-flight failures occurring
    /// before a full `RunContext` is constructed.
    ///
    /// Ensures agent health tracking, SQLite status transition to `Failed`, and
    /// concurrency/status teardown run reliably for all post-initialize errors.
    pub(crate) async fn fail_unprepared_mission(
        &self,
        agent_id: &str,
        mission_id: &str,
        generation: u64,
        e: &AppError,
    ) -> Result<(), AppError> {
        let safe_error = self
            .state
            .security
            .secret_redactor
            .redact(&format!("{}", e));
        tracing::error!(
            "❌ [Runner] Mission setup failure for agent {}: {}",
            agent_id,
            safe_error
        );

        let pool = self.state.resources.pool.clone();

        // 💾 PERSISTENCE FIRST: Update mission row and log error step in SQLite first
        let _ = crate::agent::mission::update_mission(
            &pool,
            mission_id,
            crate::agent::types::MissionStatus::Failed,
            None,
        )
        .await;

        let receipt = serde_json::json!({
            "receipt": {
                "status": "setup_failed",
                "error": safe_error
            }
        });

        let _ = crate::agent::mission::log_step(
            &pool,
            mission_id,
            agent_id,
            "System",
            &format!("❌ Error: {}", safe_error),
            "error",
            Some(receipt),
        )
        .await;

        persist_agent_outcome(
            &self.state,
            agent_id,
            Some(generation),
            AgentOutcome {
                health: OutcomeHealth::Failed,
                usage: None,
                tokens_delta: 0,
                cost_delta: 0.0,
                dispatch_accounted: false,
            },
        )
        .await?;

        // 📢 BROADCAST & STATUS: Broadcast error message and transition to idle
        self.broadcast_agent_message(
            agent_id,
            mission_id,
            &format!("❌ Error: {}", safe_error),
            "system",
            0,
        );

        // UNIFIED STATUS SYNC: transition through failed to idle
        self.update_status(agent_id, mission_id, "failed", Some(&safe_error));
        self.update_status(agent_id, mission_id, "idle", None);

        Ok(())
    }
}

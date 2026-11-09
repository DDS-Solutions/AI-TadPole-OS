//! @docs ARCHITECTURE:Runner
//!
//! ### AI Assist Note
//! - **Subsystem**: Agent context / ordered runtime preparation
//! - Preserve ordered preparation, workspace/privacy scope and shared usage accounting.
//!
//! ### 🔍 Debugging & Observability
//! - Failures retain their existing AppError or nonfatal tracing behavior.

use super::super::{AgentRunner, RunContext};
use crate::agent::sanitizer::Sanitizer;
use crate::agent::types::TaskPayload;
use crate::error::AppError;

impl AgentRunner {
    /// Prepares the runtime context, including remote memory (RAG) synchronization.
    pub(crate) async fn prepare_run_context(
        &self,
        agent_id: &str,
        payload: &TaskPayload,
        mission_id: &str,
        depth: u32,
        lineage: &[String],
        generation: u64,
    ) -> Result<RunContext, AppError> {
        let mut ctx = self
            .resolve_agent_context(agent_id, payload, mission_id, depth, lineage)
            .await?;
        ctx.execution_generation = Some(generation);

        // Sync the new budget back to mission_history so frontend/logs stay aligned.
        // Moved here from resolve_agent_context to keep context resolution strictly read-only (SRP).
        if mission_id != "system-internal" && !mission_id.is_empty() {
            if let Ok(Some(m)) =
                crate::agent::mission::get_mission_by_id(&self.state.resources.pool, mission_id)
                    .await
            {
                if ctx.mission_state.budget_usd > m.budget_usd {
                    if let Err(e) =
                        sqlx::query("UPDATE mission_history SET budget_usd = ?1 WHERE id = ?2")
                            .bind(ctx.mission_state.budget_usd)
                            .bind(mission_id)
                            .execute(&self.state.resources.pool)
                            .await
                    {
                        tracing::error!(
                            mission_id = %mission_id,
                            error = %e,
                            "❌ [Runner] Failed to sync updated budget_usd to mission_history"
                        );
                    }
                }
            }
        }

        ctx.structured_output = payload.structured_output.unwrap_or(false);

        // Auto-mount fresh playbooks to workspace on execution
        if let Err(e) = crate::system::okf_gate::mount_playbooks_to_workspace(
            &self.state.resources.pool,
            &ctx.mission_state.mission_id,
            &ctx.env.workspace_root,
        )
        .await
        {
            tracing::warn!(
                "⚠️ [OKF Gate] Failed to mount playbooks to workspace: {:?}",
                e
            );
        }
        let backlog = self
            .state
            .registry
            .mission_backlogs
            .entry(mission_id.to_string())
            .or_insert_with(|| {
                std::sync::Arc::new(parking_lot::Mutex::new(
                    crate::agent::backlog::MissionBacklog::new(mission_id),
                ))
            })
            .clone();
        ctx.backlog = Some(backlog);

        // --- 🏗️ [System 2] Relational Code Indexing ---
        // We only perform deep indexing for non-safe missions to preserve resources.
        if !ctx.safe_mode {
            let blueprint = self
                .state
                .resources
                .get_blueprint(ctx.env.workspace_root.clone())
                .await?;
            ctx.recent_findings = Some(format!(
                "{}\n\n### 🏗️ Codebase Blueprint (System 2 Index)\n{}",
                ctx.recent_findings.clone().unwrap_or_default(),
                serde_json::to_string(&*blueprint).unwrap_or_default()
            ));
        }

        // 1. Process Mission History & Summarization (4k Threshold)
        self.collect_mission_summary(&ctx, mission_id)
            .await
            .apply(&mut ctx);

        // Use the same redacted message for every provider-backed preparation
        // step. The original payload remains available to the local runner, but
        // must not reach embeddings or planning before the prompt sanitizer.
        let sanitized_message = Sanitizer::sanitize_for_prompt(&payload.message);

        // 2. Synchronize Remote Memory (Hybrid RAG)
        self.collect_rag_memory(&ctx, &sanitized_message)
            .await
            .apply(&mut ctx);

        if ctx.mission_state.depth == 0
            && !ctx.safe_mode
            && !Self::is_fast_path_query(&sanitized_message)
        {
            if Self::is_conductor_plan_required(&sanitized_message) {
                match self.generate_conductor_plan(&ctx, &sanitized_message).await {
                    Ok((plan, planning_usage)) => {
                        ctx.conductor_plan = Some(plan);
                        ctx.planning_usage = planning_usage;
                    }
                    Err((e, planning_usage)) => {
                        // Invalid plans still consumed model tokens and must remain in
                        // the parent mission's cost/usage accounting.
                        ctx.planning_usage = planning_usage;
                        tracing::warn!("⚠️ [Conductor] Failed to generate Conductor plan: {:?}", e);
                    }
                }
            } else {
                tracing::debug!(
                    "🎯 [Conductor] Single-agent/localized task detected. Bypassing Conductor DAG planning."
                );
            }
        }

        Ok(ctx)
    }
}

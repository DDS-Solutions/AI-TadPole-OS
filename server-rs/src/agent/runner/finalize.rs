//! @docs ARCHITECTURE:Runner
//!
//! ### AI Assist Note
//! **Run Finalization (Post-Mission Orchestrator)**: Orchestrates the
//! result persistence, memory archival, and cleanup for the Tadpole
//! OS agent runner. Features **Cumulative Token Accounting**:
//! updates the global agent state and budget guard with final usage
//! metrics (TPM/Cost). Implements **Durable Mission Finalization**:
//! commits the final output and status (Completed/Failed) to the
//! SQLite mission ledger. Includes **Automatic Memory Archival**:
//! spawns background tasks to index mission results into the
//! LanceDB vector store for long-term institutional knowledge. AI
//! agents should monitor the `finalize_run` telemetry to confirm
//! successful archival before closing the mission context (RUN-03).
//!
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: Partial persistence due to database locks during
//!   finalization, memory archival failures in LanceDB, or
//!   telemetry broadcast drops causing UI desync.
//! - **Trace Scope**: `server-rs::agent::runner::finalize`

use super::analysis::spawn_post_mission_analysis;
use super::{AgentRunner, RunContext};
use crate::error::AppError;

impl AgentRunner {
    // ─────────────────────────────────────────────────────────
    //  FINALIZATION
    // ─────────────────────────────────────────────────────────

    /// Finalizes the run: updates token usage, persists mission state, broadcasts results.
    pub(crate) async fn finalize_run(
        &self,
        ctx: &RunContext,
        output_text: &str,
        usage: &Option<crate::agent::types::TokenUsage>,
    ) -> Result<String, AppError> {
        tracing::info!(
            "🔔 [DIAGNOSTIC] Finalizing run for agent {}. Content length: {}",
            ctx.agent_id,
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
                ctx.agent_id
            );
        }
        tracing::debug!("DEBUG [Runner] final_delivery content: {:?}", output_text);

        // Update global agent state
        let mut agent_clone = {
            if let Some(mut entry) = self.state.registry.agents.get_mut(&ctx.agent_id) {
                let agent = entry.value_mut();
                if let Some(ref u) = usage {
                    agent.economics.token_usage = u.clone(); // Use the cumulative turn usage
                    agent.economics.tokens_used += u.total_tokens;
                }

                // Re-calculate turn cost from final cumulative usage
                let turn_cost = usage
                    .as_ref()
                    .map(|u| {
                        crate::agent::rates::calculate_cost(
                            &ctx.model_config.model_id,
                            u.input_tokens,
                            u.output_tokens,
                        )
                    })
                    .unwrap_or(0.0);
                agent.economics.cost_usd += turn_cost;
                // [Health Monitoring] Reset failure count on success
                agent.health.failure_count = 0;
                // Status is handled by update_status() at the end of the run

                // Record to persistent budget guard
                let budget_guard = self.state.security.budget_guard.clone();
                let agent_id = ctx.agent_id.clone();
                tokio::spawn(async move {
                    let _ = budget_guard.record_usage(&agent_id, turn_cost).await;
                });

                agent.clone()
            } else {
                return Err(AppError::NotFound(format!(
                    "Agent {} not found",
                    ctx.agent_id
                )));
            }
        };

        // Sync to persistence with optimistic concurrency conflict retry
        let pool = self.state.resources.pool.clone();
        let mut current_version = agent_clone.version;
        let mut retries = 0;
        loop {
            match crate::agent::persistence::save_agent_db(&pool, &mut agent_clone).await {
                Ok(_) => break,
                Err(AppError::Conflict(_)) if retries < 5 => {
                    retries += 1;
                    if let Some(db_agent) = crate::agent::persistence::load_agents_db(&pool)
                        .await?
                        .into_iter()
                        .find(|a| a.identity.id == ctx.agent_id)
                    {
                        let mut updated_agent = db_agent;
                        let tokens_used_delta = usage.as_ref().map(|u| u.total_tokens).unwrap_or(0);
                        let turn_cost = usage
                            .as_ref()
                            .map(|u| {
                                crate::agent::rates::calculate_cost(
                                    &ctx.model_config.model_id,
                                    u.input_tokens,
                                    u.output_tokens,
                                )
                            })
                            .unwrap_or(0.0);
                        updated_agent.economics.tokens_used += tokens_used_delta;
                        updated_agent.economics.cost_usd += turn_cost;
                        updated_agent.economics.token_usage =
                            usage.as_ref().cloned().unwrap_or_default();
                        updated_agent.health.failure_count = 0;
                        current_version = updated_agent.version;
                        agent_clone = updated_agent;
                    }
                    tokio::time::sleep(std::time::Duration::from_millis(50 * retries)).await;
                }
                Err(e) => return Err(e),
            }
        }

        // Re-align memory registry with updated version
        if let Some(mut entry) = self.state.registry.agents.get_mut(&ctx.agent_id) {
            let agent = entry.value_mut();
            if agent.version == current_version {
                *agent = agent_clone.clone();
            } else {
                // Concurrency conflict: reload from DB
                if let Some(db_agent) = crate::agent::persistence::load_agents_db(&pool)
                    .await?
                    .into_iter()
                    .find(|a| a.identity.id == ctx.agent_id)
                {
                    *agent = db_agent;
                }
            }

            self.state.emit_event(serde_json::json!({
                "type": "agent:update",
                "agent_id": ctx.agent_id,
                "data": *agent
            }));
        }

        // Delivery formatting
        let mut final_delivery = output_text.trim().to_string();
        if final_delivery.is_empty() {
            final_delivery =
                "(Agent completed its actions without a final conversational response.)"
                    .to_string();
        }

        final_delivery = Self::scrub_mythos_tags(&final_delivery);

        self.broadcast_agent_message(
            &ctx.agent_id,
            &ctx.mission_id,
            &final_delivery,
            "assistant",
            0,
        );
        self.update_status(&ctx.agent_id, &ctx.mission_id, "idle", None);

        // 💾 PERSISTENCE & MEMORY
        self.finalize_mission_persistence(ctx, output_text, usage)
            .await?;
        #[cfg(feature = "vector-memory")]
        self.archive_lance_db_memory(ctx, output_text);

        // 🧠 MISSION ANALYSIS TRIGGER
        if ctx.analysis && ctx.agent_id != "99" {
            spawn_post_mission_analysis(self.clone(), ctx.clone(), output_text.to_string());
        }

        Ok(final_delivery)
    }

    /// Finalizes mission state and logs the steps to SQLite.
    async fn finalize_mission_persistence(
        &self,
        ctx: &RunContext,
        output_text: &str,
        usage: &Option<crate::agent::types::TokenUsage>,
    ) -> Result<(), AppError> {
        let final_cumulative_cost = crate::agent::rates::calculate_cost(
            &ctx.model_config.model_id,
            usage.as_ref().map(|u| u.input_tokens).unwrap_or(0),
            usage.as_ref().map(|u| u.output_tokens).unwrap_or(0),
        );

        crate::agent::mission::update_mission(
            &self.state.resources.pool,
            &ctx.mission_id,
            crate::agent::types::MissionStatus::Completed,
            Some(final_cumulative_cost),
        )
        .await?;

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
                "status": "success"
            }
        });

        crate::agent::mission::log_step(
            &self.state.resources.pool,
            &ctx.mission_id,
            &ctx.agent_id,
            "Agent",
            output_text,
            "success",
            Some(receipt),
        )
        .await?;
        Ok(())
    }

    /// Saves the final output to the agent's permanent institutional knowledge in LanceDB.
    #[cfg(feature = "vector-memory")]
    fn archive_lance_db_memory(&self, ctx: &RunContext, output_text: &str) {
        let (_, agent_memory_dir, _) = ctx.resolve_paths();
        let mem_output = output_text.to_string();
        let mem_mission_id = ctx.mission_id.clone();
        let api_key = ctx.model_config.api_key.clone().unwrap_or_default();
        let http_client = self.state.resources.http_client.clone();

        let dedupe_threshold = std::env::var("LANCEDB_DEDUPE_THRESHOLD")
            .unwrap_or_else(|_| "0.2".to_string())
            .parse::<f32>()
            .unwrap_or(0.2);

        tokio::spawn(async move {
            match crate::agent::memory::VectorMemory::connect(&agent_memory_dir, "memories").await {
                Ok(mem) => {
                    if let Ok(vec) = crate::agent::memory::get_gemini_embedding(
                        &http_client,
                        &api_key,
                        &mem_output,
                    )
                    .await
                    {
                        match mem
                            .check_memory_duplicate(vec.clone(), dedupe_threshold)
                            .await
                        {
                            Ok(true) => {
                                tracing::info!("🧠 [Memory] Duplicate detected (dist < {}), skipping LanceDB insertion for mission {}", dedupe_threshold, mem_mission_id);
                            }
                            _ => {
                                let id = uuid::Uuid::new_v4().to_string();
                                let _ =
                                    mem.add_memory(&id, &mem_output, &mem_mission_id, vec).await;
                                tracing::info!(
                                    "🧠 [Memory] Archived final result to LanceDB for mission {}",
                                    mem_mission_id
                                );
                            }
                        }
                    }
                }
                Err(e) => tracing::error!("❌ [Memory] Failed to archive LanceDB memory: {}", e),
            }
        });
    }

    /// Centralized mission failure handler. Sets status, logs error, and updates agent health.
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
            ctx.agent_id,
            safe_error
        );
        self.broadcast_agent_message(
            &ctx.agent_id,
            &ctx.mission_id,
            &format!("❌ Error: {}", safe_error),
            "system",
            0,
        );

        let mut agent_clone = {
            if let Some(mut entry) = self.state.registry.agents.get_mut(&ctx.agent_id) {
                let agent = entry.value_mut();
                agent.health.failure_count += 1;
                agent.health.last_failure_at = Some(chrono::Utc::now());
                agent.clone()
            } else {
                return Err(AppError::NotFound(format!(
                    "Agent {} not found",
                    ctx.agent_id
                )));
            }
        };

        // Sync to DB with optimistic concurrency conflict retry
        let pool = self.state.resources.pool.clone();
        let mut current_version = agent_clone.version;
        let mut retries = 0;
        loop {
            match crate::agent::persistence::save_agent_db(&pool, &mut agent_clone).await {
                Ok(_) => break,
                Err(AppError::Conflict(_)) if retries < 5 => {
                    retries += 1;
                    if let Some(db_agent) = crate::agent::persistence::load_agents_db(&pool)
                        .await?
                        .into_iter()
                        .find(|a| a.identity.id == ctx.agent_id)
                    {
                        let mut updated_agent = db_agent;
                        updated_agent.health.failure_count += 1;
                        updated_agent.health.last_failure_at = Some(chrono::Utc::now());
                        current_version = updated_agent.version;
                        agent_clone = updated_agent;
                    }
                    tokio::time::sleep(std::time::Duration::from_millis(50 * retries)).await;
                }
                Err(e) => return Err(e),
            }
        }

        // Re-align memory registry with updated version
        if let Some(mut entry) = self.state.registry.agents.get_mut(&ctx.agent_id) {
            let agent = entry.value_mut();
            if agent.version == current_version {
                *agent = agent_clone.clone();
            } else {
                // Concurrency conflict: reload from DB
                if let Some(db_agent) = crate::agent::persistence::load_agents_db(&pool)
                    .await?
                    .into_iter()
                    .find(|a| a.identity.id == ctx.agent_id)
                {
                    *agent = db_agent;
                }
            }

            self.state.emit_event(serde_json::json!({
                "type": "agent:update",
                "agent_id": ctx.agent_id,
                "data": *agent
            }));
        }

        // Record final cost if usage was provided
        if let Some(u) = usage {
            let turn_cost = crate::agent::rates::calculate_cost(
                &ctx.model_config.model_id,
                u.input_tokens,
                u.output_tokens,
            );
            let budget_guard = self.state.security.budget_guard.clone();
            let agent_id = ctx.agent_id.clone();
            tokio::spawn(async move {
                let _ = budget_guard.record_usage(&agent_id, turn_cost).await;
            });
        }

        // 🕵️ UNIFIED STATUS SYNC
        // We use update_status AFTER the registry block to ensure the lock is released
        // and telemetry is broadcast correctly for the "Idle" state.
        self.update_status(&ctx.agent_id, &ctx.mission_id, "idle", None);

        crate::agent::mission::update_mission(
            &self.state.resources.pool,
            &ctx.mission_id,
            crate::agent::types::MissionStatus::Failed,
            None,
        )
        .await?;

        let modified = ctx.modified_files.lock().clone();
        let accessed = ctx.last_accessed_files.lock().clone();
        let read_files: Vec<String> = accessed
            .into_iter()
            .filter(|f| !modified.contains(f))
            .collect();
        let turn_cost = usage
            .as_ref()
            .map(|u| {
                crate::agent::rates::calculate_cost(
                    &ctx.model_config.model_id,
                    u.input_tokens,
                    u.output_tokens,
                )
            })
            .unwrap_or(0.0);

        let receipt = serde_json::json!({
            "receipt": {
                "read_files": read_files,
                "modified_files": modified,
                "commands_run": ctx.commands_run.lock().clone(),
                "token_usage": usage,
                "cost_usd": turn_cost,
                "status": "failed",
                "error": safe_error
            }
        });

        crate::agent::mission::log_step(
            &self.state.resources.pool,
            &ctx.mission_id,
            &ctx.agent_id,
            "System",
            &format!("❌ Error: {}", safe_error),
            "error",
            Some(receipt),
        )
        .await?;

        Ok(())
    }

    /// Strips internal LLM control tokens from output text before broadcasting to the UI.
    fn scrub_mythos_tags(text: &str) -> String {
        text.replace("<halting_signal/>", "")
            .replace("<halt/>", "")
            .replace("<thinking>", "")
            .replace("</thinking>", "")
            .trim()
            .to_string()
    }
}

// Metadata: [finalize]

#[cfg(test)]
mod tests {
    use crate::agent::runner::{AgentRunner, RunContext};
    use crate::agent::types::{EngineAgent, TokenUsage};
    use crate::state::AppState;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_finalize_run_metrics() {
        let state = Arc::new(AppState::new_minimal_mock().await);
        let runner = AgentRunner::new(state.clone());
        let mut ctx = RunContext::default();
        ctx.agent_id = "test-agent".to_string();

        // Setup mock agent
        let mut agent = EngineAgent::default();
        agent.identity.id = ctx.agent_id.clone();
        state
            .registry
            .agents
            .insert(ctx.agent_id.clone(), agent.clone());

        // Sync agent to DB first so create_mission doesn't fail
        crate::agent::persistence::save_agent_db(&state.resources.pool, &mut agent)
            .await
            .unwrap();

        // Create a mock mission in the DB to satisfy persistence calls
        let mission = crate::agent::mission::create_mission(
            &state.resources.pool,
            &ctx.agent_id,
            "Test Mission",
            1.0,
        )
        .await
        .unwrap();
        ctx.mission_id = mission.id;

        let usage = Some(TokenUsage {
            input_tokens: 100,
            output_tokens: 50,
            total_tokens: 150,
        });

        let result = runner.finalize_run(&ctx, "Final output", &usage).await;
        assert_eq!(result.unwrap(), "Final output");

        // Verify metrics
        let agent = state.registry.agents.get(&ctx.agent_id).unwrap();
        assert_eq!(agent.economics.tokens_used, 150);
        assert!(agent.economics.cost_usd > 0.0);
        assert_eq!(agent.health.failure_count, 0);
    }
}

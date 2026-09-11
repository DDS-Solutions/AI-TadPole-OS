//! @docs ARCHITECTURE:Runner
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Agent Runner / finalize
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Type-safe state handling and bounded execution without unhandled panics.
//! - `[Behavioral]` Mission outcome is persisted before success broadcast and idle status transition (enforced_by: `test_finalize_run_metrics`, `test_finalize_run_resilient_to_missing_agent`).
//! - `[Behavioral]` Failure teardown completes database updates even when agent is missing from registry (enforced_by: `test_fail_mission_without_registry_agent`).
//! - `[Behavioral]` Subtask step errors cause root mission to pause for review (enforced_by: `test_branch_error_step_causes_parent_to_pause`).
//! - `[Behavioral]` Resolved model ID is used for cost pricing over static config (enforced_by: `test_effective_model_id_pricing`).
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: `[Runner]`, `[Memory]`
//! - **Witness Tests**: `test_finalize_run_metrics`, `branch_finalization_does_not_close_parent_mission`, `partial_root_result_pauses_for_human_review`, `test_fail_mission_without_registry_agent`, `test_branch_error_step_causes_parent_to_pause`, `test_effective_model_id_pricing`, `test_finalize_run_resilient_to_missing_agent`

use super::analysis::spawn_post_mission_analysis;
use super::{AgentRunner, RunContext};
use crate::error::AppError;

impl AgentRunner {
    // ─────────────────────────────────────────────────────────
    //  FINALIZATION
    // ─────────────────────────────────────────────────────────

    /// Returns the effective model ID used during inference, preferring the vetted/resolved
    /// model ID over the original static configuration (e.g. for Privacy Shield local fallbacks).
    pub(crate) fn effective_model_id<'a>(&self, ctx: &'a RunContext) -> String {
        ctx.resolved_model_id
            .lock()
            .clone()
            .unwrap_or_else(|| ctx.model_config.model_id.clone())
    }

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

        let effective_model = self.effective_model_id(ctx);
        let turn_cost = usage
            .as_ref()
            .map(|u| {
                crate::agent::rates::calculate_cost(
                    &effective_model,
                    u.input_tokens,
                    u.output_tokens,
                )
            })
            .unwrap_or(0.0);

        // Delivery formatting and tag sanitization
        let mut final_delivery = output_text.trim().to_string();
        if final_delivery.is_empty() {
            final_delivery =
                "(Agent completed its actions without a final conversational response.)"
                    .to_string();
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
                ctx.agent_id,
                e
            );
            let _ = self.fail_mission(ctx, &e, usage).await;
            return Err(e);
        }

        // Update global agent state (resilient to missing in-memory agent)
        let pool = self.state.resources.pool.clone();
        let agent_opt = if let Some(mut entry) = self.state.registry.agents.get_mut(&ctx.agent_id) {
            let agent = entry.value_mut();
            if let Some(ref u) = usage {
                agent.economics.token_usage = u.clone();
                agent.economics.tokens_used += u.total_tokens;
            }
            agent.economics.cost_usd += turn_cost;
            agent.health.failure_count = 0;
            Some(agent.clone())
        } else {
            match crate::agent::persistence::load_agent_by_id_db(&pool, &ctx.agent_id).await {
                Ok(Some(mut db_agent)) => {
                    if let Some(ref u) = usage {
                        db_agent.economics.token_usage = u.clone();
                        db_agent.economics.tokens_used += u.total_tokens;
                    }
                    db_agent.economics.cost_usd += turn_cost;
                    db_agent.health.failure_count = 0;
                    Some(db_agent)
                }
                Ok(None) => {
                    tracing::warn!(
                        "⚠️ [Runner] Agent {} not found in registry or DB after mission persist",
                        ctx.agent_id
                    );
                    None
                }
                Err(err) => {
                    tracing::warn!(
                        "⚠️ [Runner] Failed to lookup agent {} in DB after mission persist: {}",
                        ctx.agent_id,
                        err
                    );
                    None
                }
            }
        };

        if let Some(mut agent_clone) = agent_opt {
            let mut current_version = agent_clone.version;
            let mut retries = 0;
            loop {
                match crate::agent::persistence::save_agent_db(&pool, &mut agent_clone).await {
                    Ok(_) => break,
                    Err(AppError::Conflict(_)) if retries < 5 => {
                        retries += 1;
                        if let Ok(Some(db_agent)) =
                            crate::agent::persistence::load_agent_by_id_db(&pool, &ctx.agent_id)
                                .await
                        {
                            let mut updated_agent = db_agent;
                            let tokens_used_delta =
                                usage.as_ref().map(|u| u.total_tokens).unwrap_or(0);
                            updated_agent.economics.tokens_used += tokens_used_delta;
                            updated_agent.economics.cost_usd += turn_cost;
                            updated_agent.economics.token_usage =
                                usage.as_ref().cloned().unwrap_or_default();
                            updated_agent.health.failure_count = 0;
                            current_version = updated_agent.version;
                            agent_clone = updated_agent;
                        } else {
                            break;
                        }
                        tokio::time::sleep(std::time::Duration::from_millis(50 * retries)).await;
                    }
                    Err(e) => {
                        tracing::warn!(
                            "⚠️ [Runner] Failed to persist agent economics after mission persist: {}",
                            e
                        );
                        break;
                    }
                }
            }

            // Re-align memory registry with updated version
            if let Some(mut entry) = self.state.registry.agents.get_mut(&ctx.agent_id) {
                let agent = entry.value_mut();
                if agent.version == current_version {
                    *agent = agent_clone.clone();
                } else if let Ok(Some(db_agent)) =
                    crate::agent::persistence::load_agent_by_id_db(&pool, &ctx.agent_id).await
                {
                    *agent = db_agent;
                }

                // SEC: Emit sanitized public view without raw API keys
                self.state.emit_event(serde_json::json!({
                    "type": "agent:update",
                    "agent_id": ctx.agent_id,
                    "data": crate::routes::agent::AgentResponse::from(&*agent)
                }));
            }
        }

        // Record to persistent budget guard if turn incurred a cost
        if turn_cost > 0.0 {
            let budget_guard = self.state.security.budget_guard.clone();
            let agent_id = ctx.agent_id.clone();
            tokio::spawn(async move {
                let _ = budget_guard.record_usage(&agent_id, turn_cost).await;
            });
        }

        // 📢 BROADCAST & STATUS: Broadcast assistant message and transition to idle
        // ONLY after database persistence has succeeded.
        self.broadcast_agent_message(
            &ctx.agent_id,
            &ctx.mission_id,
            &final_delivery,
            "assistant",
            0,
        );
        self.update_status(&ctx.agent_id, &ctx.mission_id, "idle", None);

        #[cfg(feature = "vector-memory")]
        self.archive_lance_db_memory(ctx, &final_delivery);

        // 🧠 MISSION ANALYSIS TRIGGER
        if ctx.analysis && ctx.agent_id != super::analysis::QA_AUDITOR_ID {
            spawn_post_mission_analysis(self.clone(), ctx.clone(), final_delivery.clone());
        }

        Ok(final_delivery)
    }

    /// Finalizes mission state and logs the steps to SQLite.
    async fn finalize_mission_persistence(
        &self,
        ctx: &RunContext,
        output_text: &str,
        usage: &Option<crate::agent::types::TokenUsage>,
        final_cumulative_cost: f64,
    ) -> Result<(), AppError> {
        let is_branch = ctx.depth > 0;
        let mut has_partial_failure = ctx
            .swarm_partial_failure
            .load(std::sync::atomic::Ordering::Acquire);

        // For root missions, check if any steps for this mission logged an error or branch failure.
        // Queries mission_logs by severity = 'error'. Fails closed (pauses) on database query errors.
        if !is_branch && !has_partial_failure {
            match sqlx::query_scalar::<_, i64>(
                "SELECT COUNT(1) FROM mission_logs WHERE mission_id = ?1 AND severity = 'error'",
            )
            .bind(&ctx.mission_id)
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
                        ctx.mission_id,
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
        let requires_human_review = has_partial_failure && !human_review_approved;

        let (receipt_status, step_status) = if is_branch {
            crate::agent::mission::add_mission_cost(
                &self.state.resources.pool,
                &ctx.mission_id,
                final_cumulative_cost,
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
                &ctx.mission_id,
                mission_status,
                Some(final_cumulative_cost),
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
                "status": receipt_status,
                "branch_depth": ctx.depth,
                "requires_human_review": requires_human_review
            }
        });

        crate::agent::mission::log_step(
            &self.state.resources.pool,
            &ctx.mission_id,
            &ctx.agent_id,
            "Agent",
            output_text,
            step_status,
            Some(receipt),
        )
        .await?;
        Ok(())
    }

    /// Saves the final output to the agent's permanent institutional knowledge in LanceDB.
    /// Under Privacy Mode, cloud embeddings are strictly bypassed to prevent data leakage.
    #[cfg(feature = "vector-memory")]
    fn archive_lance_db_memory(&self, ctx: &RunContext, output_text: &str) {
        if self.is_privacy_active(ctx) {
            tracing::info!(
                "🔒 [Privacy Shield] Skipping cloud vector memory embedding for mission {}",
                ctx.mission_id
            );
            return;
        }

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
            ctx.agent_id,
            safe_error
        );

        let model_id = self.effective_model_id(ctx);
        let turn_cost = usage
            .as_ref()
            .map(|u| {
                crate::agent::rates::calculate_cost(&model_id, u.input_tokens, u.output_tokens)
            })
            .unwrap_or(0.0);
        let turn_tokens = usage.as_ref().map(|u| u.total_tokens).unwrap_or(0);

        // Propagate branch failure to parent context if running within a swarm branch
        if ctx.depth > 0 {
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
        if ctx.depth > 0 {
            crate::agent::mission::add_mission_cost(
                &self.state.resources.pool,
                &ctx.mission_id,
                turn_cost,
            )
            .await?;
        } else {
            crate::agent::mission::update_mission(
                &self.state.resources.pool,
                &ctx.mission_id,
                crate::agent::types::MissionStatus::Failed,
                Some(turn_cost),
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
                "status": if ctx.depth > 0 { "branch_failed" } else { "failed" },
                "branch_depth": ctx.depth,
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

        // 2. Update and persist agent health and economics
        let pool = self.state.resources.pool.clone();
        let agent_opt = if let Some(mut entry) = self.state.registry.agents.get_mut(&ctx.agent_id) {
            let agent = entry.value_mut();
            agent.health.failure_count += 1;
            agent.health.last_failure_at = Some(chrono::Utc::now());
            agent.economics.tokens_used += turn_tokens;
            agent.economics.cost_usd += turn_cost;
            Some(agent.clone())
        } else {
            match crate::agent::persistence::load_agent_by_id_db(&pool, &ctx.agent_id).await {
                Ok(Some(mut db_agent)) => {
                    db_agent.health.failure_count += 1;
                    db_agent.health.last_failure_at = Some(chrono::Utc::now());
                    db_agent.economics.tokens_used += turn_tokens;
                    db_agent.economics.cost_usd += turn_cost;
                    Some(db_agent)
                }
                Ok(None) => {
                    tracing::warn!(
                        "⚠️ [Runner] Agent {} not found in registry or DB during fail_mission",
                        ctx.agent_id
                    );
                    None
                }
                Err(err) => {
                    tracing::warn!(
                        "⚠️ [Runner] Failed to lookup agent {} in DB during fail_mission: {}",
                        ctx.agent_id,
                        err
                    );
                    None
                }
            }
        };

        if let Some(mut agent_clone) = agent_opt {
            let mut current_version = agent_clone.version;
            let mut retries = 0;
            loop {
                match crate::agent::persistence::save_agent_db(&pool, &mut agent_clone).await {
                    Ok(_) => break,
                    Err(AppError::Conflict(_)) if retries < 5 => {
                        retries += 1;
                        if let Ok(Some(db_agent)) =
                            crate::agent::persistence::load_agent_by_id_db(&pool, &ctx.agent_id)
                                .await
                        {
                            let mut updated_agent = db_agent;
                            updated_agent.health.failure_count += 1;
                            updated_agent.health.last_failure_at = Some(chrono::Utc::now());
                            updated_agent.economics.tokens_used += turn_tokens;
                            updated_agent.economics.cost_usd += turn_cost;
                            current_version = updated_agent.version;
                            agent_clone = updated_agent;
                        } else {
                            break;
                        }
                        tokio::time::sleep(std::time::Duration::from_millis(50 * retries)).await;
                    }
                    Err(e) => {
                        tracing::warn!(
                            "⚠️ [Runner] Failed to persist agent health on fail_mission: {}",
                            e
                        );
                        break;
                    }
                }
            }

            // Re-align memory registry with updated version
            if let Some(mut entry) = self.state.registry.agents.get_mut(&ctx.agent_id) {
                let agent = entry.value_mut();
                if agent.version == current_version {
                    *agent = agent_clone.clone();
                } else if let Ok(Some(db_agent)) =
                    crate::agent::persistence::load_agent_by_id_db(&pool, &ctx.agent_id).await
                {
                    *agent = db_agent;
                }

                self.state.emit_event(serde_json::json!({
                    "type": "agent:update",
                    "agent_id": ctx.agent_id,
                    "data": crate::routes::agent::AgentResponse::from(&*agent)
                }));
            }
        }

        // Record final cost if usage was provided and cost > 0
        if turn_cost > 0.0 {
            let budget_guard = self.state.security.budget_guard.clone();
            let agent_id = ctx.agent_id.clone();
            tokio::spawn(async move {
                let _ = budget_guard.record_usage(&agent_id, turn_cost).await;
            });
        }

        // 3. 📢 BROADCAST & STATUS: Broadcast error message and transition to idle
        // ONLY after database persistence has succeeded.
        self.broadcast_agent_message(
            &ctx.agent_id,
            &ctx.mission_id,
            &format!("❌ Error: {}", safe_error),
            "system",
            0,
        );

        self.update_status(&ctx.agent_id, &ctx.mission_id, "idle", None);

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

        // Update agent health if present in registry or DB
        let agent_opt = if let Some(mut entry) = self.state.registry.agents.get_mut(agent_id) {
            let agent = entry.value_mut();
            agent.health.failure_count += 1;
            agent.health.last_failure_at = Some(chrono::Utc::now());
            Some(agent.clone())
        } else {
            match crate::agent::persistence::load_agent_by_id_db(&pool, agent_id).await {
                Ok(Some(mut db_agent)) => {
                    db_agent.health.failure_count += 1;
                    db_agent.health.last_failure_at = Some(chrono::Utc::now());
                    Some(db_agent)
                }
                Ok(None) => {
                    tracing::warn!(
                        "⚠️ [Runner] Agent {} not found in registry or DB during fail_unprepared_mission",
                        agent_id
                    );
                    None
                }
                Err(err) => {
                    tracing::warn!(
                        "⚠️ [Runner] Failed to lookup agent {} in DB during fail_unprepared_mission: {}",
                        agent_id,
                        err
                    );
                    None
                }
            }
        };

        if let Some(mut agent_clone) = agent_opt {
            let mut current_version = agent_clone.version;
            let mut retries = 0;
            loop {
                match crate::agent::persistence::save_agent_db(&pool, &mut agent_clone).await {
                    Ok(_) => break,
                    Err(AppError::Conflict(_)) if retries < 5 => {
                        retries += 1;
                        if let Ok(Some(db_agent)) =
                            crate::agent::persistence::load_agent_by_id_db(&pool, agent_id).await
                        {
                            let mut updated_agent = db_agent;
                            updated_agent.health.failure_count += 1;
                            updated_agent.health.last_failure_at = Some(chrono::Utc::now());
                            current_version = updated_agent.version;
                            agent_clone = updated_agent;
                        } else {
                            break;
                        }
                        tokio::time::sleep(std::time::Duration::from_millis(50 * retries)).await;
                    }
                    Err(err) => {
                        tracing::warn!(
                            "⚠️ [Runner] Failed to persist agent health on setup failure: {}",
                            err
                        );
                        break;
                    }
                }
            }

            if let Some(mut entry) = self.state.registry.agents.get_mut(agent_id) {
                let agent = entry.value_mut();
                if agent.version == current_version {
                    *agent = agent_clone.clone();
                } else if let Ok(Some(db_agent)) =
                    crate::agent::persistence::load_agent_by_id_db(&pool, agent_id).await
                {
                    *agent = db_agent;
                }
                self.state.emit_event(serde_json::json!({
                    "type": "agent:update",
                    "agent_id": agent_id,
                    "data": crate::routes::agent::AgentResponse::from(&*agent)
                }));
            }
        }

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

    /// Strips internal LLM control tokens and thinking tags (including attributes) from output text before broadcasting to the UI.
    fn scrub_mythos_tags(text: &str) -> String {
        static TAG_REGEX: std::sync::LazyLock<regex::Regex> = std::sync::LazyLock::new(|| {
            regex::Regex::new(r"(?is)<thought(\s+[^>]*)?>.*?</thought>|<(thinking|/thinking|thought|/thought|halting_signal|halt)(\s+[^>]*)?\s*/?>|<\|im_end\|>|<\|thought\|>|<\|endoftext\|>").unwrap()
        });
        TAG_REGEX.replace_all(text, "").trim().to_string()
    }
}

// Metadata: [finalize]

#[cfg(test)]
mod tests {
    use crate::agent::runner::{AgentRunner, RunContext};
    use crate::agent::types::{EngineAgent, TokenUsage};
    use crate::error::AppError;
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

    #[tokio::test]
    async fn branch_finalization_does_not_close_parent_mission() {
        let state = Arc::new(AppState::new_minimal_mock().await);
        let runner = AgentRunner::new(state.clone());
        let mut parent = EngineAgent::default();
        parent.identity.id = "parent-agent".to_string();
        crate::agent::persistence::save_agent_db(&state.resources.pool, &mut parent)
            .await
            .unwrap();
        let mission = crate::agent::mission::create_mission(
            &state.resources.pool,
            "parent-agent",
            "Shared branch ledger",
            5.0,
        )
        .await
        .unwrap();
        let ctx = RunContext {
            agent_id: "child-agent".to_string(),
            mission_id: mission.id.clone(),
            depth: 1,
            ..Default::default()
        };

        runner
            .finalize_mission_persistence(&ctx, "branch result", &None, 0.0)
            .await
            .unwrap();

        let persisted =
            crate::agent::mission::get_mission_by_id(&state.resources.pool, &mission.id)
                .await
                .unwrap()
                .unwrap();
        assert_eq!(
            persisted.status,
            crate::agent::types::MissionStatus::Pending
        );
    }

    #[tokio::test]
    async fn partial_root_result_pauses_for_human_review() {
        let state = Arc::new(AppState::new_minimal_mock().await);
        let runner = AgentRunner::new(state.clone());
        let mut parent = EngineAgent::default();
        parent.identity.id = "parent-agent".to_string();
        crate::agent::persistence::save_agent_db(&state.resources.pool, &mut parent)
            .await
            .unwrap();
        let mission = crate::agent::mission::create_mission(
            &state.resources.pool,
            "parent-agent",
            "Human review ledger",
            5.0,
        )
        .await
        .unwrap();
        let ctx = RunContext {
            agent_id: "parent-agent".to_string(),
            mission_id: mission.id.clone(),
            swarm_partial_failure: Arc::new(std::sync::atomic::AtomicBool::new(true)),
            ..Default::default()
        };

        runner
            .finalize_mission_persistence(&ctx, "partial result", &None, 0.0)
            .await
            .unwrap();

        let persisted =
            crate::agent::mission::get_mission_by_id(&state.resources.pool, &mission.id)
                .await
                .unwrap()
                .unwrap();
        assert_eq!(persisted.status, crate::agent::types::MissionStatus::Paused);
    }

    #[tokio::test]
    async fn test_fail_mission_without_registry_agent() {
        let state = Arc::new(AppState::new_minimal_mock().await);
        let runner = AgentRunner::new(state.clone());
        let agent_id = "missing-agent";

        let mut agent = EngineAgent::default();
        agent.identity.id = agent_id.to_string();
        crate::agent::persistence::save_agent_db(&state.resources.pool, &mut agent)
            .await
            .unwrap();

        let mission = crate::agent::mission::create_mission(
            &state.resources.pool,
            agent_id,
            "Ghost Mission",
            1.0,
        )
        .await
        .unwrap();

        // Ensure missing from in-memory registry
        state.registry.agents.remove(agent_id);

        let ctx = RunContext {
            agent_id: agent_id.to_string(),
            mission_id: mission.id.clone(),
            ..Default::default()
        };

        let err = AppError::BadRequest("Execution failed".to_string());
        let result = runner.fail_mission(&ctx, &err, &None).await;
        assert!(
            result.is_ok(),
            "fail_mission must succeed even if agent is missing from registry"
        );

        // Verify mission status transitioned to Failed
        let persisted =
            crate::agent::mission::get_mission_by_id(&state.resources.pool, &mission.id)
                .await
                .unwrap()
                .unwrap();
        assert_eq!(persisted.status, crate::agent::types::MissionStatus::Failed);

        // Verify error step logged
        let steps = crate::agent::mission::get_mission_logs(&state.resources.pool, &mission.id)
            .await
            .unwrap();
        assert!(steps.iter().any(|s| s.severity == "error"));
    }

    #[tokio::test]
    async fn test_branch_error_step_causes_parent_to_pause() {
        let state = Arc::new(AppState::new_minimal_mock().await);
        let runner = AgentRunner::new(state.clone());
        let mut parent = EngineAgent::default();
        parent.identity.id = "root-agent".to_string();
        crate::agent::persistence::save_agent_db(&state.resources.pool, &mut parent)
            .await
            .unwrap();

        let mut child = EngineAgent::default();
        child.identity.id = "child-worker".to_string();
        crate::agent::persistence::save_agent_db(&state.resources.pool, &mut child)
            .await
            .unwrap();

        let mission = crate::agent::mission::create_mission(
            &state.resources.pool,
            "root-agent",
            "Swarm Tree with Branch Failure",
            5.0,
        )
        .await
        .unwrap();

        // Execute fail_mission on a child context (depth: 1)
        let child_ctx = RunContext {
            agent_id: "child-worker".to_string(),
            mission_id: mission.id.clone(),
            depth: 1,
            ..Default::default()
        };
        let child_err = AppError::BadRequest("Child worker crashed".to_string());
        runner
            .fail_mission(&child_ctx, &child_err, &None)
            .await
            .unwrap();

        // Now root finalizes
        let root_ctx = RunContext {
            agent_id: "root-agent".to_string(),
            mission_id: mission.id.clone(),
            depth: 0,
            ..Default::default()
        };

        runner
            .finalize_mission_persistence(&root_ctx, "root result", &None, 0.0)
            .await
            .unwrap();

        let persisted =
            crate::agent::mission::get_mission_by_id(&state.resources.pool, &mission.id)
                .await
                .unwrap()
                .unwrap();
        assert_eq!(persisted.status, crate::agent::types::MissionStatus::Paused);
    }

    #[tokio::test]
    async fn test_effective_model_id_pricing() {
        let state = Arc::new(AppState::new_minimal_mock().await);
        let runner = AgentRunner::new(state.clone());

        let mut ctx = RunContext::default();
        ctx.agent_id = "pricing-agent".to_string();
        // Static configuration points to expensive Opus model
        ctx.model_config.model_id = "claude-4.8-opus".to_string();

        assert_eq!(runner.effective_model_id(&ctx), "claude-4.8-opus");

        // Resolved model is set to gpt-4o-mini (e.g. redirected or dynamically resolved)
        *ctx.resolved_model_id.lock() = Some("gpt-4o-mini".to_string());
        assert_eq!(runner.effective_model_id(&ctx), "gpt-4o-mini");

        let usage = Some(TokenUsage {
            input_tokens: 1000,
            output_tokens: 1000,
            total_tokens: 2000,
        });

        let mut agent = EngineAgent::default();
        agent.identity.id = ctx.agent_id.clone();
        state
            .registry
            .agents
            .insert(ctx.agent_id.clone(), agent.clone());
        crate::agent::persistence::save_agent_db(&state.resources.pool, &mut agent)
            .await
            .unwrap();

        let mission = crate::agent::mission::create_mission(
            &state.resources.pool,
            &ctx.agent_id,
            "Pricing Mission",
            1.0,
        )
        .await
        .unwrap();
        ctx.mission_id = mission.id;

        runner.finalize_run(&ctx, "output", &usage).await.unwrap();

        let updated_agent = state.registry.agents.get(&ctx.agent_id).unwrap();
        assert_eq!(updated_agent.economics.tokens_used, 2000);
        // gpt-4o-mini rate: 1000 * 0.00015/1k + 1000 * 0.0006/1k = 0.00075 USD
        // (Opus rate would have been 0.090 USD)
        assert!((updated_agent.economics.cost_usd - 0.00075).abs() < 1e-6);
        assert!((updated_agent.economics.cost_usd - 0.090).abs() > 0.01);
    }

    #[tokio::test]
    async fn test_finalize_run_resilient_to_missing_agent() {
        let state = Arc::new(AppState::new_minimal_mock().await);
        let runner = AgentRunner::new(state.clone());
        let agent_id = "missing-agent-success";

        let mut agent = EngineAgent::default();
        agent.identity.id = agent_id.to_string();
        crate::agent::persistence::save_agent_db(&state.resources.pool, &mut agent)
            .await
            .unwrap();

        let mission = crate::agent::mission::create_mission(
            &state.resources.pool,
            agent_id,
            "Success Mission",
            1.0,
        )
        .await
        .unwrap();

        // Ensure missing from in-memory registry
        state.registry.agents.remove(agent_id);

        let ctx = RunContext {
            agent_id: agent_id.to_string(),
            mission_id: mission.id.clone(),
            ..Default::default()
        };

        let result = runner.finalize_run(&ctx, "Success output", &None).await;
        assert!(
            result.is_ok(),
            "finalize_run must succeed and broadcast even if agent is missing from registry"
        );

        let persisted =
            crate::agent::mission::get_mission_by_id(&state.resources.pool, &mission.id)
                .await
                .unwrap()
                .unwrap();
        assert_eq!(
            persisted.status,
            crate::agent::types::MissionStatus::Completed
        );
    }

    #[test]
    fn test_scrub_mythos_tags_extended() {
        let input = "Hello <thinking depth=\"2\">internal</thinking> <thought>reasoning</thought> world!<|im_end|><|thought|><|endoftext|><halting_signal/>";
        let cleaned = AgentRunner::scrub_mythos_tags(input);
        assert_eq!(cleaned, "Hello internal  world!");
    }
}

//! Mission Lifecycle — Setup, validation and initialization
//!
//! Extracted from `runner/mod.rs` for maintainability. All methods remain 
//! on `impl AgentRunner` — this is a structural split, not a behavioral change.
//!
//! @docs ARCHITECTURE:Runner

use crate::agent::types::TaskPayload;

use super::AgentRunner;

impl AgentRunner {
    // ─────────────────────────────────────────────────────────
    //  SETUP & VALIDATION
    // ─────────────────────────────────────────────────────────

    /// Performs initial safety and tracing setup.
    ///
    /// - Extracts OTel `traceparent` for cross-service observability.
    /// - Scans user input for prompt injection via the `Sanitizer`.
    /// - Checks global health metrics and recursion lineage.
    pub(crate) fn setup_and_validate(&self, agent_id: &str, payload: &TaskPayload) -> anyhow::Result<()> {
        let span = tracing::Span::current();

        // Setup OTel Trace Propagation
        if let Some(tp) = &payload.traceparent {
            use opentelemetry::global;
            use tracing_opentelemetry::OpenTelemetrySpanExt;

            let mut extractor = std::collections::HashMap::new();
            extractor.insert("traceparent".to_string(), tp.clone());
            let parent_cx = global::get_text_map_propagator(|prop| prop.extract(&extractor));
            span.set_parent(parent_cx);

            // Parse trace_id directly from traceparent (00-traceid-spanid-01)
            let parts: Vec<&str> = tp.split('-').collect();
            if parts.len() >= 4 {
                span.record("trace_id", parts[1]);
            } else {
                use opentelemetry::trace::TraceContextExt;
                let cx = span.context();
                let otel_span = cx.span();
                let otel_trace_id = otel_span.span_context().trace_id().to_string();
                span.record("trace_id", &otel_trace_id);
            }
        } else {
            use opentelemetry::trace::TraceContextExt;
            use tracing_opentelemetry::OpenTelemetrySpanExt;
            let cx = span.context();
            let otel_span = cx.span();
            let otel_trace_id = otel_span.span_context().trace_id().to_string();
            span.record("trace_id", &otel_trace_id);
        }

        // Core validation logic
        self.validate_input(agent_id, payload)?;

        // Security Sanitization
        if let crate::agent::sanitizer::SanitizationResult::Alert(msg) =
            crate::agent::sanitizer::Sanitizer::scan(&payload.message)
        {
            tracing::warn!("🛡️ [Security] Blocked potential injection: {}", msg);
            self.state.broadcast_sys(
                &format!("🛡️ Security: {}", msg),
                "error",
                payload.cluster_id.clone(),
            );
            return Err(anyhow::anyhow!("❌ Security Violation: {}", msg));
        }

        Ok(())
    }

    /// Validates input constraints before execution begins.
    pub(crate) fn validate_input(&self, agent_id: &str, payload: &TaskPayload) -> anyhow::Result<()> {
        let max_task_length = self
            .state
            .governance
            .max_task_length
            .load(std::sync::atomic::Ordering::Relaxed);
        if payload.message.len() > max_task_length {
            return Err(anyhow::anyhow!(
                "❌ Task message too long ({} bytes, max {})",
                payload.message.len(),
                max_task_length
            ));
        }

        let depth = payload.swarm_depth.unwrap_or(0);
        let lineage = payload.swarm_lineage.as_deref().unwrap_or(&[]);

        // CODE-01 FIX: Use iterator instead of to_string() allocation on hot path.
        if lineage.iter().any(|id| id == agent_id) {
            let path = lineage.join(" -> ");
            return Err(anyhow::anyhow!("🐝 CIRCULAR RECURSION DETECTED: Agent '{}' has already participated in this mission chain (Lineage: {} -> {}). Recruitment aborted.", agent_id, path, agent_id));
        }

        let max_swarm_depth = self
            .state
            .governance
            .max_swarm_depth
            .load(std::sync::atomic::Ordering::Relaxed);
        if depth >= max_swarm_depth {
            return Err(anyhow::anyhow!("🐝 Swarm depth limit exceeded (current depth: {})! To prevent infinite recursions, this agent cannot spawn more sub-agents.", depth));
        }

        // Check max agents limit (only for new recruitment, not for existing agents)
        if !self.state.registry.agents.contains_key(agent_id) {
            let max_agents = self
                .state
                .governance
                .max_agents
                .load(std::sync::atomic::Ordering::Relaxed);
            if self.state.registry.agents.len() as u32 >= max_agents {
                return Err(anyhow::anyhow!(
                    "🐝 Swarm agent limit reached (max: {}). New agent recruitment denied.",
                    max_agents
                ));
            }
        }

        Ok(())
    }

    // ─────────────────────────────────────────────────────────
    //  MISSION INITIALIZATION
    // ─────────────────────────────────────────────────────────

    /// Registers a new mission in the database and enforces budget gates.
    ///
    /// This method ensures the agent's failure rate is within healthy limits and
    /// that sufficient USD credits remain in the persistent `BudgetGuard`
    /// before any tokens are consumed.
    pub(crate) async fn initialize_mission_state(
        &self,
        agent_id: &str,
        payload: &TaskPayload,
    ) -> anyhow::Result<crate::agent::types::Mission> {
        // 🛡️ [Resilience] Ensure agent exists in database (auto-sync if only in registry)
        let db_pool = &self.state.resources.pool;
        let db_check: Option<i64> = sqlx::query_scalar::<sqlx::Sqlite, i64>("SELECT 1 FROM agents WHERE id = ?")
            .bind(agent_id)
            .fetch_optional(db_pool)
            .await
            .unwrap_or(None);

        if db_check.is_none() {
            if let Some(agent) = self.state.registry.agents.get(agent_id) {
                tracing::info!(
                    "🔄 [Runner] Agent {} not in DB but found in registry. Self-healing...",
                    agent_id
                );
                let _ = crate::agent::persistence::save_agent_db(db_pool, agent.value()).await;
            }
        }

        let depth = payload.swarm_depth.unwrap_or(0);
        let _ = self
            .state
            .governance
            .max_swarm_depth
            .fetch_max(depth, std::sync::atomic::Ordering::Relaxed);

        let mission_title = payload.message.chars().take(50).collect::<String>() + "...";

        // 🏥 [Health Check] Failure Rate Throttling
        if let Some(agent) = self.state.registry.agents.get(agent_id) {
            if agent.value().failure_count >= 5 {
                let last_fail = agent
                    .value()
                    .last_failure_at
                    .unwrap_or_else(chrono::Utc::now);
                let cooldown = chrono::Duration::minutes(15);
                if chrono::Utc::now() - last_fail < cooldown {
                    tracing::warn!(
                        "🏥 [Health] Agent {} is degraded (Failure Count: {})",
                        agent_id,
                        agent.value().failure_count
                    );
                    self.state.broadcast_sys(
                        &format!(
                            "🏥 Health: {} is in self-heal cooldown (Failure Count: {}).",
                            agent_id,
                            agent.value().failure_count
                        ),
                        "warning",
                        payload.cluster_id.clone(),
                    );
                    return Err(anyhow::anyhow!(
                        "❌ Agent Degraded. Self-heal cooldown active."
                    ));
                }
            }
        }

        // [Budget Gate] Persistent Budget Check (OpenFang Pattern)
        let has_budget = self
            .state
            .security
            .budget_guard
            .check_budget(agent_id, 0.0)
            .await?;
        if !has_budget {
            tracing::warn!(
                "🛡️ [Governance] Agent {} persistent budget exhausted",
                agent_id
            );
            self.state.broadcast_sys(
                &format!(
                    "🛡️ Budget Guard: {} has exceeded its periodic quota.",
                    agent_id
                ),
                "error",
                payload.cluster_id.clone(),
            );
            return Err(anyhow::anyhow!(
                "❌ Persistent Quota Exhausted. Check Security Dashboard."
            ));
        }

        // Keep legacy agent-local budget for backward compatibility / double gating
        let mut agent_budget = 0.0;
        let mut agent_cost = 0.0;
        if let Some(agent) = self.state.registry.agents.get(agent_id) {
            agent_budget = agent.value().budget_usd;
            agent_cost = agent.value().cost_usd;
        }

        if agent_budget > 0.0 && agent_cost >= agent_budget {
            tracing::warn!(
                "⚠️ [Governance] Agent {} local budget exhausted (${:.4}/${:.4})",
                agent_id,
                agent_cost,
                agent_budget
            );
            return Err(anyhow::anyhow!(
                "❌ Local Agent Budget Exhausted (${:.4}/${:.4})",
                agent_cost,
                agent_budget
            ));
        }

        let mission_budget = payload.budget_usd.unwrap_or_else(|| {
            if agent_budget > 0.0 {
                (agent_budget - agent_cost).max(0.001) // Ensure at least some budget if not explicitly over
            } else {
                1.0 // Default fallback
            }
        });

        let mission = crate::agent::mission::create_mission(
            &self.state.resources.pool,
            agent_id,
            &mission_title,
            mission_budget,
        )
        .await?;

        crate::agent::mission::update_mission(
            &self.state.resources.pool,
            &mission.id,
            crate::agent::types::MissionStatus::Active,
            0.0,
        )
        .await?;
        crate::agent::mission::log_step(
            &self.state.resources.pool,
            &mission.id,
            agent_id,
            "User",
            &payload.message,
            "info",
            None,
        )
        .await?;

        Ok(mission)
    }
}

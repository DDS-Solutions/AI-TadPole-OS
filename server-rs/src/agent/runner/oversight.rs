//! @docs ARCHITECTURE:Runner
//!
//! ### AI Assist Note
//! **Oversight Gate**: The human-in-the-loop safety mechanism. Intercepts
//! sensitive tool calls and skill proposals, rerouting them to the
//! **Sapphire Gate** UI for manual approval. Uses `tokio::sync::oneshot`
//! to pause agent execution until a decision is received.
//!
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: Oversight channel timeout, duplicate entry IDs,
//!   or telemetry broadcast failure during status updates.
//! - **Trace Scope**: `server-rs::agent::runner::oversight`

use super::AgentRunner;

impl AgentRunner {
    // ─────────────────────────────────────────────────────────
    //  OVERSIGHT (HUMAN-IN-THE-LOOP)
    // ─────────────────────────────────────────────────────────

    pub async fn submit_oversight_resolution(
        &self,
        mut tool_call: crate::agent::types::ToolCallAudit,
        mission_id: Option<String>,
    ) -> Result<crate::agent::types::OversightResolution, crate::error::AppError> {
        let entry_id = uuid::Uuid::new_v4().to_string();

        tool_call.mission_id = mission_id.clone();

        let entry = crate::agent::types::OversightEntry {
            id: entry_id.clone(),
            mission_id: mission_id.clone(),
            tool_call: Some(tool_call.clone()),
            skill_proposal: None,
            status: "pending".to_string(),
            created_at: chrono::Utc::now().to_rfc3339(),
        };

        // 1. Create a channel for the decision and register it IMMEDIATELY
        let (tx, rx) = tokio::sync::oneshot::channel();
        self.state
            .comms
            .oversight_resolvers
            .insert(entry_id.clone(), tx);

        // 2. Register in the queue for UI discovery
        self.state
            .comms
            .oversight_queue
            .insert(entry_id.clone(), entry.clone());

        // 3. [Persistence] Record action attempt in SQLite for audit history
        let payload_json = serde_json::to_string(&tool_call).unwrap_or_default();
        let params_json = serde_json::to_string(&tool_call.params).unwrap_or_default();

        sqlx::query(
            "INSERT INTO oversight_log (id, mission_id, agent_id, entry_type, skill, params, status, payload) VALUES (?, ?, ?, 'tool_call', ?, ?, 'pending', ?)"
        )
        .bind(&entry_id)
        .bind(&mission_id)
        .bind(&tool_call.agent_id)
        .bind(&tool_call.skill)
        .bind(params_json)
        .bind(payload_json)
        .execute(&self.state.resources.pool)
        .await?;

        // 4. Notify the UI
        self.state.emit_event(serde_json::json!({
            "type": "oversight:new",
            "entry": entry
        }));

        // 5. Await the user's click in the dashboard with a safety timeout (300s)
        let timeout_dur = std::time::Duration::from_secs(300);
        let res = match tokio::time::timeout(timeout_dur, rx).await {
            Ok(Ok(resolution)) => resolution,
            Ok(Err(_)) => crate::agent::types::OversightResolution {
                approved: false,
                override_slot: None,
            },
            Err(_) => {
                tracing::warn!(
                    "⚠️ [Oversight] Timeout ({}s) waiting for oversight decision on entry {}. Rejecting by default.",
                    timeout_dur.as_secs(),
                    entry_id
                );
                self.state.comms.oversight_resolvers.remove(&entry_id);
                self.state.comms.oversight_queue.remove(&entry_id);
                
                let _ = sqlx::query(
                    "UPDATE oversight_log SET status = 'rejected', decision = 'timed_out', decided_at = datetime('now'), decided_by = 'system' WHERE id = ?"
                )
                .bind(&entry_id)
                .execute(&self.state.resources.pool)
                .await;

                crate::agent::types::OversightResolution {
                    approved: false,
                    override_slot: None,
                }
            }
        };

        Ok(res)
    }

    /// Submits a tool call for manual user approval.
    /// Returns true if approved, false if rejected.
    pub async fn submit_oversight(
        &self,
        tool_call: crate::agent::types::ToolCallAudit,
        mission_id: Option<String>,
    ) -> Result<bool, crate::error::AppError> {
        let res = self
            .submit_oversight_resolution(tool_call, mission_id)
            .await?;
        Ok(res.approved)
    }

    // ─────────────────────────────────────────────────────────
    //  TELEMETRY HELPERS
    // ─────────────────────────────────────────────────────────

    pub(crate) fn broadcast_agent_status(&self, agent_id: &str, mission_id: &str, status: &str) {
        let (task, tokens_used) = self
            .state
            .registry
            .agents
            .get(agent_id)
            .map(|a| (
                a.state.current_task.clone(),
                a.economics.tokens_used,
            ))
            .unwrap_or((None, 0));

        // Derive elapsed_ms from active_mission.started_at if available
        let elapsed_ms: Option<u64> = self
            .state
            .registry
            .agents
            .get(agent_id)
            .and_then(|a| a.state.active_mission.clone())
            .and_then(|m| m.get("started_at").and_then(|v| v.as_u64()))
            .map(|started_at| {
                let now_ms = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis() as u64;
                now_ms.saturating_sub(started_at)
            });

        let _ = self.state.comms.telemetry_tx.send(serde_json::json!({
            "type": "agent:status",
            "agent_id": agent_id,
            "mission_id": mission_id,
            "status": status,
            "current_task": task,
            "tokens_used_so_far": tokens_used,
            "elapsed_ms": elapsed_ms,
        }));
    }

    /// Centralized status and task update that syncs registry AND broadcasts telemetry.
    pub(crate) fn update_status(
        &self,
        agent_id: &str,
        mission_id: &str,
        status: &str,
        task: Option<&str>,
    ) {
        if let Some(mut entry) = self.state.registry.agents.get_mut(agent_id) {
            let agent = entry.value_mut();
            agent.health.status = status.to_string();
            agent.state.current_task = task.map(|t| t.to_string());

            // Sync active mission for high-speed pulse telemetry
            if status == "idle" {
                agent.state.active_mission = None;
            } else if agent.state.active_mission.is_none() {
                let now_ms = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis() as u64;
                agent.state.active_mission = Some(serde_json::json!({
                    "id": mission_id,
                    "started_at": now_ms
                }));
            }
        }
        self.broadcast_agent_status(agent_id, mission_id, status);
    }

    pub(crate) fn broadcast_agent_message(
        &self,
        agent_id: &str,
        mission_id: &str,
        text: &str,
        role: &str,
        turn_index: usize,
    ) {
        // Approximate token count without a tokenizer (whitespace split)
        let token_count = text.split_whitespace().count();
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        let _ = self.state.comms.telemetry_tx.send(serde_json::json!({
            "type": "agent:message",
            "agent_id": agent_id,
            "mission_id": mission_id,
            "message_id": uuid::Uuid::new_v4().to_string(),
            "role": role,
            "content": text,
            "turn_index": turn_index,
            "token_count": token_count,
            "timestamp": timestamp,
        }));
    }

    /// ### ✅ [System 2] Success Sentinel
    /// Checks the current observation buffer for evidence of successful verification.
    /// This is the "Sentinel Gate" that prevents agents from reporting success
    /// without deterministic proof (e.g. a passing test).
    pub(crate) fn verify_mission_success(&self, observation: &str) -> bool {
        let obs_lower = observation.to_lowercase();

        // Evidence of a test being run and passing
        let has_test_trigger = obs_lower.contains("test")
            || obs_lower.contains("pytest")
            || obs_lower.contains("cargo test");
        let has_success_indicator = obs_lower.contains("passed")
            || obs_lower.contains("success")
            || obs_lower.contains("ok (");

        has_test_trigger && has_success_indicator
    }
}

// Metadata: [oversight]

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::types::EngineAgent;
    use crate::state::AppState;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_update_status() {
        let state = Arc::new(AppState::new_minimal_mock().await);
        let runner = AgentRunner::new(state.clone());
        let agent_id = "test-agent";

        let mut agent = EngineAgent::default();
        agent.identity.id = agent_id.to_string();
        state.registry.agents.insert(agent_id.to_string(), agent);

        runner.update_status(agent_id, "mission-1", "busy", Some("Thinking..."));

        let agent = state.registry.agents.get(agent_id).unwrap();
        assert_eq!(agent.health.status, "busy");
        assert_eq!(agent.state.current_task.as_deref(), Some("Thinking..."));
    }

    #[tokio::test]
    async fn test_verify_mission_success_sentinel() {
        let state = Arc::new(AppState::new_minimal_mock().await);
        let runner = AgentRunner::new(state.clone());

        // Happy path: contains test trigger and success indicator
        assert!(runner.verify_mission_success("Running cargo test... test result: ok (3 passed)"));
        assert!(runner.verify_mission_success("pytest output: 12 passed, 0 failed"));
        assert!(runner.verify_mission_success("Test status: success"));

        // Failure path: missing test trigger
        assert!(!runner.verify_mission_success("All operations completed successfully without errors."));

        // Failure path: missing success indicator
        assert!(!runner.verify_mission_success("Running cargo test... test result: failed"));

        // Case insensitivity check
        assert!(runner.verify_mission_success("CARGO TEST PASSED"));
    }

    #[tokio::test]
    async fn test_broadcast_agent_message() {
        let state = Arc::new(AppState::new_minimal_mock().await);
        let runner = AgentRunner::new(state.clone());
        let mut rx = state.comms.telemetry_tx.subscribe();

        runner.broadcast_agent_message("agent-1", "mission-1", "hello world", "assistant", 0);

        let msg = rx.try_recv().unwrap();
        assert_eq!(msg["type"], "agent:message");
        assert_eq!(msg["agent_id"], "agent-1");
        assert_eq!(msg["content"], "hello world");
        assert_eq!(msg["role"], "assistant");
        assert_eq!(msg["turn_index"], 0);
        assert!(msg["timestamp"].as_u64().unwrap() > 0);
    }
}


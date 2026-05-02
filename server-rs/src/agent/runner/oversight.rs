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

    /// Submits a tool call for manual user approval.
    /// Returns true if approved, false if rejected.
    pub async fn submit_oversight(
        &self,
        mut tool_call: crate::agent::types::ToolCallAudit,
        mission_id: Option<String>,
    ) -> Result<bool, crate::error::AppError> {
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

        // 5. Await the user's click in the dashboard
        Ok(rx.await.unwrap_or_default())
    }


    // ─────────────────────────────────────────────────────────
    //  TELEMETRY HELPERS
    // ─────────────────────────────────────────────────────────

    pub(crate) fn broadcast_agent_status(&self, agent_id: &str, mission_id: &str, status: &str) {
        let task = self
            .state
            .registry
            .agents
            .get(agent_id)
            .and_then(|a| a.state.current_task.clone());

        let _ = self.state.comms.telemetry_tx.send(serde_json::json!({
            "type": "agent:status",
            "agent_id": agent_id,
            "mission_id": mission_id,
            "status": status,
            "current_task": task
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
        if let Some(mut agent) = self.state.registry.agents.get_mut(agent_id) {
            agent.health.status = status.to_string();
            agent.state.current_task = task.map(|t| t.to_string());
        }
        self.broadcast_agent_status(agent_id, mission_id, status);
    }

    pub(crate) fn broadcast_agent_message(&self, agent_id: &str, mission_id: &str, text: &str) {
        let _ = self.state.comms.telemetry_tx.send(serde_json::json!({
            "type": "agent:message",
            "agent_id": agent_id,
            "mission_id": mission_id,
            "text": text,
            "message_id": uuid::Uuid::new_v4().to_string()
        }));
    }
}

// Metadata: [oversight]

// Metadata: [oversight]

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::AppState;
    use crate::agent::types::EngineAgent;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_update_status() {
        let state = Arc::new(AppState::new_minimal_mock().await);
        let runner = AgentRunner::new(state.clone());
        let agent_id = "test-agent";
        
        let agent = EngineAgent {
            identity: crate::agent::types::AgentIdentity {
                id: agent_id.to_string(),
                ..Default::default()
            },
            ..EngineAgent::default()
        };
        state.registry.agents.insert(agent_id.to_string(), agent);

        runner.update_status(agent_id, "mission-1", "busy", Some("Thinking..."));

        let agent = state.registry.agents.get(agent_id).unwrap();
        assert_eq!(agent.health.status, "busy");
        assert_eq!(agent.state.current_task.as_deref(), Some("Thinking..."));
    }

    #[tokio::test]
    async fn test_broadcast_agent_message() {
        let state = Arc::new(AppState::new_minimal_mock().await);
        let runner = AgentRunner::new(state.clone());
        let mut rx = state.comms.telemetry_tx.subscribe();

        runner.broadcast_agent_message("agent-1", "mission-1", "Hello World");

        let msg = rx.recv().await.unwrap();
        assert_eq!(msg["type"], "agent:message");
        assert_eq!(msg["text"], "Hello World");
    }
}

//! @docs ARCHITECTURE:SecurityModel
//!
//! ### AI Assist Note
//! - **Subsystem**: Security / Oversight prompt lifecycle
//! - The cleanup guard owns resolver and queue removal on completion, timeout, or cancellation.
//!
//! ### 🔍 Debugging & Observability
//! - **Telemetry Targets**: none declared

use super::PermissionMode;
use anyhow::Result;
use sqlx::SqlitePool;
use std::{sync::Arc, time::Duration};

#[async_trait::async_trait]
pub trait PermissionPrompter: Send + Sync {
    /// Prompts the user for a decision on a pending tool execution.
    /// This may be implemented via a Tauri modal or an asynchronous event bus.
    async fn prompt_user(&self, tool_name: &str, arguments: &str) -> Result<PermissionMode>;

    /// Prompts the user with agent identity context for fine-grained oversight tracking.
    async fn prompt_agent(
        &self,
        _agent_id: Option<&str>,
        tool_name: &str,
        arguments: &str,
    ) -> Result<PermissionMode> {
        self.prompt_user(tool_name, arguments).await
    }
}

struct OversightCleanupGuard {
    comms: Arc<crate::state::hubs::comm::CommunicationHub>,
    id: String,
}

impl Drop for OversightCleanupGuard {
    fn drop(&mut self) {
        self.comms.oversight_queue.remove(&self.id);
        self.comms.oversight_resolvers.remove(&self.id);
    }
}

/// Asynchronous event-driven prompter that routes permission prompts through the unified oversight queue.
/// Writes pending request to `oversight_log`, enqueues in `oversight_queue`, registers oneshot in `oversight_resolvers`,
/// emits `oversight:new` / `oversight:request` over event_tx, and awaits resolution with safety timeout and cleanup.
pub struct AsyncOversightPrompter {
    pub pool: SqlitePool,
    pub comms: Arc<crate::state::hubs::comm::CommunicationHub>,
    pub event_tx: Option<tokio::sync::broadcast::Sender<serde_json::Value>>,
}

impl AsyncOversightPrompter {
    pub fn new(
        pool: SqlitePool,
        comms: Arc<crate::state::hubs::comm::CommunicationHub>,
        event_tx: Option<tokio::sync::broadcast::Sender<serde_json::Value>>,
    ) -> Self {
        Self {
            pool,
            comms,
            event_tx,
        }
    }
}

#[async_trait::async_trait]
impl PermissionPrompter for AsyncOversightPrompter {
    async fn prompt_user(&self, tool_name: &str, arguments: &str) -> Result<PermissionMode> {
        self.prompt_agent(None, tool_name, arguments).await
    }

    async fn prompt_agent(
        &self,
        agent_id: Option<&str>,
        tool_name: &str,
        arguments: &str,
    ) -> Result<PermissionMode> {
        let entry_id = uuid::Uuid::new_v4().to_string();
        let agent = agent_id.unwrap_or("unknown_agent").to_string();

        let tool_call = crate::agent::types::ToolCallAudit {
            id: entry_id.clone(),
            mission_id: None,
            agent_id: agent.clone(),
            skill: tool_name.to_string(),
            params: serde_json::json!({ "arguments": arguments }),
            department: "security".to_string(),
            description: format!("MCP tool '{}' requires authorization", tool_name),
            timestamp: chrono::Utc::now().to_rfc3339(),
        };

        let entry = crate::agent::types::OversightEntry {
            id: entry_id.clone(),
            mission_id: None,
            tool_call: Some(tool_call.clone()),
            skill_proposal: None,
            status: "pending".to_string(),
            created_at: chrono::Utc::now().to_rfc3339(),
        };

        let params_json =
            serde_json::to_string(&tool_call.params).unwrap_or_else(|_| "{}".to_string());
        let payload_json = serde_json::to_string(&entry).unwrap_or_else(|_| "{}".to_string());

        // 1. Write to SQLite oversight_log
        let _ = sqlx::query(
            "INSERT INTO oversight_log (id, mission_id, agent_id, entry_type, skill, params, status, payload) VALUES (?, ?, ?, 'tool_call', ?, ?, 'pending', ?)"
        )
        .bind(&entry_id)
        .bind(None::<String>)
        .bind(&agent)
        .bind(tool_name)
        .bind(&params_json)
        .bind(&payload_json)
        .execute(&self.pool)
        .await;

        // 2. Setup oneshot resolver and queue entry with cleanup guard
        let (tx, rx) = tokio::sync::oneshot::channel();
        self.comms.oversight_resolvers.insert(entry_id.clone(), tx);
        self.comms
            .oversight_queue
            .insert(entry_id.clone(), entry.clone());

        let _guard = OversightCleanupGuard {
            comms: self.comms.clone(),
            id: entry_id.clone(),
        };

        // 3. Emit event bus notifications
        if let Some(ref event_tx) = self.event_tx {
            let _ = event_tx.send(serde_json::json!({
                "type": "oversight:new",
                "entry": entry
            }));
            let _ = event_tx.send(serde_json::json!({
                "type": "oversight:request",
                "approval_id": entry_id,
                "tool_name": tool_name,
                "arguments": arguments,
                "timestamp": chrono::Utc::now().to_rfc3339()
            }));
        }

        // 4. Bounded wait for human decision (300s timeout)
        match tokio::time::timeout(Duration::from_secs(300), rx).await {
            Ok(Ok(resolution)) => {
                if resolution.approved {
                    Ok(PermissionMode::Allow)
                } else {
                    Ok(PermissionMode::Deny)
                }
            }
            Ok(Err(_)) => Ok(PermissionMode::Deny),
            Err(_) => {
                let _ = sqlx::query(
                    "UPDATE oversight_log SET status = 'rejected', decision = 'timeout', decided_at = CURRENT_TIMESTAMP WHERE id = ?"
                )
                .bind(&entry_id)
                .execute(&self.pool)
                .await;
                Ok(PermissionMode::Deny)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn cancelled_prompt_releases_queue_and_resolver() {
        let state = crate::state::AppState::new_minimal_mock().await;
        let prompter =
            AsyncOversightPrompter::new(state.resources.pool.clone(), state.comms.clone(), None);
        let task =
            tokio::spawn(async move { prompter.prompt_agent(Some("agent"), "tool", "{}").await });
        tokio::time::timeout(Duration::from_secs(5), async {
            while state.comms.oversight_queue.is_empty() {
                tokio::task::yield_now().await;
            }
        })
        .await
        .unwrap();
        task.abort();
        let _ = task.await;
        assert!(state.comms.oversight_queue.is_empty());
        assert!(state.comms.oversight_resolvers.is_empty());
    }
}

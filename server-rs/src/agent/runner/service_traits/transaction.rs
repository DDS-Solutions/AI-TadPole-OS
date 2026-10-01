//! @docs ARCHITECTURE:Persistence
//!
//! ### AI Assist Note
//! - **Subsystem**: Sovereign Engine / Agent Runner / service_traits / transaction
//! - **Architecture**: `@docs ARCHITECTURE:Persistence` — State Rollback & Atomic Transactions
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural] [ACID]` Every `StateTransaction` MUST be either `.commit()`-ed or `.rollback()`-ed before drop.
//! - `[Structural] [Safety]` `Drop` impl provides an implicit async rollback guard when commitment is missed.
//! - `[Structural] [Resilience]` All rollback errors are aggregated — partial rollback failures surface as one error.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: `StateTransaction rollback partially failed: <msgs>`
//! - **Telemetry Targets**: `tracing::warn!` on implicit drop rollback, `tracing::error!` on op failure.
//! - **Witness Tests**: `test_state_transaction_rollback`, `test_state_transaction_commit`,
//!   `test_state_transaction_explicit_rollback`, `test_state_transaction_multi_op_spec_rollback`,
//!   `test_state_transaction_rollback_error_aggregation`

/// Undo operations for state transactions (Fix 2: Formalized Rollback).
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum UndoOp {
    RestoreAgentRegistry {
        agent_id: String,
        status: String,
        task: Option<String>,
        active_mission: Option<serde_json::Value>,
    },
    RestoreMissionSpec {
        mission_id: String,
        agent_id: String,
        prior_spec: Option<String>,
    },
    RestoreMissionStatus {
        mission_id: String,
        status: crate::agent::types::MissionStatus,
    },
}

impl UndoOp {
    pub async fn execute(
        &self,
        state: &crate::state::AppState,
    ) -> Result<(), crate::error::AppError> {
        match self {
            UndoOp::RestoreAgentRegistry {
                agent_id,
                status,
                task,
                active_mission,
            } => {
                if let Some(mut entry) = state.registry.agents.get_mut(agent_id) {
                    let agent = entry.value_mut();
                    agent.health.status = status.clone();
                    agent.state.current_task = task.clone();
                    agent.state.active_mission = active_mission.clone();

                    // Persist restored state to SQLite database to mirror forward update persistence
                    let pool = state.resources.pool.clone();
                    let registry = state.registry.clone();
                    let mut agent_clone = agent.clone();
                    let agent_id_owned = agent_id.clone();
                    tokio::spawn(async move {
                        if let Err(e) =
                            crate::agent::persistence::save_agent_db(&pool, &mut agent_clone).await
                        {
                            tracing::error!(
                                "❌ [StateTransaction] Failed to sync agent status rollback to database: {}",
                                e
                            );
                        } else if let Some(mut entry) = registry.agents.get_mut(&agent_id_owned) {
                            if entry.version < agent_clone.version {
                                entry.version = agent_clone.version;
                            }
                        }
                    });
                }
                let _ = state.comms.telemetry_tx.send(serde_json::json!({
                    "type": "agent:status",
                    "agent_id": agent_id,
                    "mission_id": "",
                    "status": status,
                    "current_task": task
                }));
                Ok(())
            }
            UndoOp::RestoreMissionSpec {
                mission_id,
                agent_id,
                prior_spec,
            } => {
                if let Some(spec) = prior_spec {
                    crate::agent::mission::set_mission_spec(
                        &state.resources.pool,
                        mission_id,
                        agent_id,
                        spec,
                    )
                    .await?;
                } else {
                    sqlx::query::<sqlx::Sqlite>(
                        "DELETE FROM swarm_context WHERE mission_id = ?1 AND agent_id = ?2 AND topic = 'system::spec'"
                    )
                    .bind(mission_id)
                    .bind(agent_id)
                    .execute(&state.resources.pool)
                    .await?;
                }
                Ok(())
            }
            UndoOp::RestoreMissionStatus { mission_id, status } => {
                let status_str = match status {
                    crate::agent::types::MissionStatus::Pending => "pending",
                    crate::agent::types::MissionStatus::SpecReview => "spec_review",
                    crate::agent::types::MissionStatus::Active => "active",
                    crate::agent::types::MissionStatus::Completed => "completed",
                    crate::agent::types::MissionStatus::Failed => "failed",
                    crate::agent::types::MissionStatus::Paused => "paused",
                };
                sqlx::query::<sqlx::Sqlite>(
                    "UPDATE mission_history SET status = ?1, updated_at = ?2 WHERE id = ?3",
                )
                .bind(status_str)
                .bind(chrono::Utc::now())
                .bind(mission_id)
                .execute(&state.resources.pool)
                .await?;
                Ok(())
            }
        }
    }
}

pub struct StateTransaction {
    #[allow(dead_code)]
    manager: std::sync::Arc<dyn super::ports::MissionStateManager>,
    state: std::sync::Arc<crate::state::AppState>,
    agent_id: String,
    mission_id: String,
    committed: bool,
    pub(super) undo_ops: Vec<UndoOp>,
}

impl StateTransaction {
    pub fn new(
        manager: std::sync::Arc<dyn super::ports::MissionStateManager>,
        state: std::sync::Arc<crate::state::AppState>,
        agent_id: &str,
        mission_id: &str,
    ) -> Self {
        let (original_status, original_task, original_active_mission) =
            if let Some(entry) = state.registry.agents.get(agent_id) {
                let agent = entry.value();
                (
                    agent.health.status.clone(),
                    agent.state.current_task.clone(),
                    agent.state.active_mission.clone(),
                )
            } else {
                ("idle".to_string(), None, None)
            };

        let initial_agent_undo = UndoOp::RestoreAgentRegistry {
            agent_id: agent_id.to_string(),
            status: original_status,
            task: original_task,
            active_mission: original_active_mission,
        };

        Self {
            manager,
            state,
            agent_id: agent_id.to_string(),
            mission_id: mission_id.to_string(),
            committed: false,
            undo_ops: vec![initial_agent_undo],
        }
    }

    #[allow(dead_code)]
    pub fn record_agent_status_change(
        &mut self,
        _agent_id: &str,
        _status: &str,
        _task: Option<&str>,
    ) {
        // Registry status modifications are captured by the primary registry undo operation
    }

    pub fn record_mission_spec_change(
        &mut self,
        mission_id: &str,
        agent_id: &str,
        prior_spec: Option<String>,
    ) {
        self.undo_ops.push(UndoOp::RestoreMissionSpec {
            mission_id: mission_id.to_string(),
            agent_id: agent_id.to_string(),
            prior_spec,
        });
    }

    #[allow(dead_code)]
    pub fn record_mission_status_change(
        &mut self,
        mission_id: &str,
        status: crate::agent::types::MissionStatus,
    ) {
        self.undo_ops.push(UndoOp::RestoreMissionStatus {
            mission_id: mission_id.to_string(),
            status,
        });
    }

    pub fn commit(mut self) {
        self.committed = true;
    }

    pub async fn rollback(&mut self) -> Result<(), crate::error::AppError> {
        if !self.committed {
            self.committed = true;
            let ops = std::mem::take(&mut self.undo_ops);
            tracing::info!(
                "🔄 [StateTransaction] Explicit rollback triggered for agent {} on mission {}",
                self.agent_id,
                self.mission_id
            );
            let mut errors = Vec::new();
            for op in ops.into_iter().rev() {
                if let Err(e) = op.execute(&self.state).await {
                    tracing::error!("❌ [StateTransaction] Rollback operation failed: {:?}", e);
                    errors.push(e);
                }
            }
            if !errors.is_empty() {
                let msgs = errors
                    .into_iter()
                    .map(|e| e.to_string())
                    .collect::<Vec<_>>()
                    .join("; ");
                return Err(crate::error::AppError::InternalServerError(format!(
                    "StateTransaction rollback partially failed: {}",
                    msgs
                )));
            }
        }
        Ok(())
    }
}

impl Drop for StateTransaction {
    fn drop(&mut self) {
        if !self.committed {
            if let Ok(handle) = tokio::runtime::Handle::try_current() {
                let state = self.state.clone();
                let agent_id = self.agent_id.clone();
                let mission_id = self.mission_id.clone();
                let ops = std::mem::take(&mut self.undo_ops);

                handle.spawn(async move {
                    tracing::warn!(
                        "🚨 [StateTransaction] IMPLICIT ROLLBACK TRIGGERED INSIDE DROP for agent {} on mission {}. Use explicit .rollback() to ensure deterministic sequencing.",
                        agent_id,
                        mission_id
                    );
                    for op in ops.into_iter().rev() {
                        if let Err(e) = op.execute(&state).await {
                            tracing::error!(
                                "❌ [StateTransaction] Implicit rollback operation failed: {:?}",
                                e
                            );
                        }
                    }
                });
            } else {
                tracing::error!(
                    "❌ [StateTransaction] Dropped uncommitted transaction outside a Tokio runtime context! Cannot execute async rollback for agent {} on mission {}",
                    self.agent_id,
                    self.mission_id
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::runner::service_traits::mission_state::DefaultMissionStateManager;
    use crate::agent::runner::service_traits::ports::MissionStateManager;
    use crate::agent::types::EngineAgent;
    use crate::state::AppState;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_state_transaction_rollback() {
        let state = Arc::new(AppState::new_minimal_mock().await);
        let agent_id = "test-agent";
        let mission_id = "mission-1";

        let mut agent = EngineAgent::default();
        agent.identity.id = agent_id.to_string();
        agent.health.status = "idle".to_string();
        agent.state.current_task = None;
        state.registry.agents.insert(agent_id.to_string(), agent);

        let manager = Arc::new(DefaultMissionStateManager);

        {
            let _tx = StateTransaction::new(manager.clone(), state.clone(), agent_id, mission_id);
            // Simulate status update during execution
            manager.update_status(&state, agent_id, mission_id, "busy", Some("Running..."));

            let entry = state.registry.agents.get(agent_id).unwrap();
            assert_eq!(entry.value().health.status, "busy");
            assert_eq!(
                entry.value().state.current_task.as_deref(),
                Some("Running...")
            );
            // Drop without committing triggers rollback
        }

        // Poll deterministically for background rollback to complete
        let mut rolled_back = false;
        for _ in 0..100 {
            tokio::task::yield_now().await;
            if let Some(entry) = state.registry.agents.get(agent_id) {
                if entry.value().health.status == "idle"
                    && entry.value().state.current_task.is_none()
                {
                    rolled_back = true;
                    break;
                }
            }
            tokio::time::sleep(std::time::Duration::from_millis(5)).await;
        }
        assert!(
            rolled_back,
            "Background rollback did not complete within timeout"
        );
    }

    #[tokio::test]
    async fn test_state_transaction_explicit_rollback() {
        let state = Arc::new(AppState::new_minimal_mock().await);
        let agent_id = "test-agent";
        let mission_id = "mission-1";

        let mut agent = EngineAgent::default();
        agent.identity.id = agent_id.to_string();
        agent.health.status = "idle".to_string();
        agent.state.current_task = None;
        state.registry.agents.insert(agent_id.to_string(), agent);

        let manager = Arc::new(DefaultMissionStateManager);

        let mut tx = StateTransaction::new(manager.clone(), state.clone(), agent_id, mission_id);
        manager.update_status(&state, agent_id, mission_id, "busy", Some("Running..."));

        tx.rollback().await.unwrap();

        // Verify status rolled back immediately
        let entry = state.registry.agents.get(agent_id).unwrap();
        assert_eq!(entry.value().health.status, "idle");
        assert_eq!(entry.value().state.current_task, None);
    }

    #[tokio::test]
    async fn test_state_transaction_commit() {
        let state = Arc::new(AppState::new_minimal_mock().await);
        let agent_id = "test-agent";
        let mission_id = "mission-1";

        let mut agent = EngineAgent::default();
        agent.identity.id = agent_id.to_string();
        agent.health.status = "idle".to_string();
        agent.state.current_task = None;
        state.registry.agents.insert(agent_id.to_string(), agent);

        let manager = Arc::new(DefaultMissionStateManager);

        {
            let tx = StateTransaction::new(manager.clone(), state.clone(), agent_id, mission_id);
            manager.update_status(&state, agent_id, mission_id, "busy", Some("Running..."));
            tx.commit();
        }

        // Verify status did NOT roll back since it was committed
        let entry = state.registry.agents.get(agent_id).unwrap();
        assert_eq!(entry.value().health.status, "busy");
        assert_eq!(
            entry.value().state.current_task.as_deref(),
            Some("Running...")
        );
    }

    #[tokio::test]
    async fn test_state_transaction_multi_op_spec_rollback() {
        let state = Arc::new(AppState::new_minimal_mock().await);
        let agent_id = "test-agent";
        let mission_id = "mission-spec-1";

        let mut agent = EngineAgent::default();
        agent.identity.id = agent_id.to_string();
        agent.health.status = "idle".to_string();
        crate::agent::persistence::save_agent_db(&state.resources.pool, &mut agent)
            .await
            .unwrap();
        state.registry.agents.insert(agent_id.to_string(), agent);

        crate::agent::mission::create_mission_with_id(
            &state.resources.pool,
            mission_id,
            agent_id,
            "Test Mission",
            10.0,
        )
        .await
        .unwrap();

        let manager = Arc::new(DefaultMissionStateManager);
        let mut tx = StateTransaction::new(manager.clone(), state.clone(), agent_id, mission_id);

        // 1. Update status
        manager.update_status(&state, agent_id, mission_id, "working", Some("Spec Gen"));

        // 2. Set mission spec and record change with prior_spec = None
        manager
            .set_mission_spec(&state, mission_id, agent_id, "Spec Content v1")
            .await
            .unwrap();
        tx.record_mission_spec_change(mission_id, agent_id, None);

        // Rollback transaction
        tx.rollback().await.unwrap();

        // Check agent status is restored to idle
        let entry = state.registry.agents.get(agent_id).unwrap();
        assert_eq!(entry.value().health.status, "idle");

        // Check spec was deleted since prior_spec was None
        let spec_exists: Option<String> = sqlx::query_scalar(
            "SELECT finding FROM swarm_context WHERE mission_id = ?1 AND agent_id = ?2 AND topic = 'system::spec'"
        )
        .bind(mission_id)
        .bind(agent_id)
        .fetch_optional(&state.resources.pool)
        .await
        .unwrap();

        assert!(spec_exists.is_none());
    }

    #[tokio::test]
    async fn test_state_transaction_rollback_error_aggregation() {
        let state = Arc::new(AppState::new_minimal_mock().await);
        let agent_id = "test-agent-err";
        let mission_id = "mission-err-1";

        let manager = Arc::new(DefaultMissionStateManager);
        let mut tx = StateTransaction::new(manager.clone(), state.clone(), agent_id, mission_id);

        tx.undo_ops.push(UndoOp::RestoreMissionStatus {
            mission_id: "test-mission".to_string(),
            status: crate::agent::types::MissionStatus::Failed,
        });

        // Close pool to force database operations inside undo ops to fail
        state.resources.pool.close().await;

        let res = tx.rollback().await;
        assert!(res.is_err());
        let err_str = res.unwrap_err().to_string();
        assert!(err_str.contains("rollback partially failed"));
    }
}

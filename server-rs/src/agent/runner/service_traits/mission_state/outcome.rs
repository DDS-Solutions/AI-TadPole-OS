//! @docs ARCHITECTURE:State
//!
//! ### AI Assist Note
//! - **Subsystem**: Agent lifecycle / durable outcomes
//! - Rebase an outcome delta after optimistic conflicts; publish only outcome-owned fields.
//!
//! ### 🔍 Debugging & Observability
//! - Persistence failures propagate to the finalizer; stale execution ownership returns Conflict.

use crate::agent::types::{EngineAgent, TokenUsage};
use crate::error::AppError;
use crate::state::AppState;

#[derive(Clone, Copy)]
pub(crate) enum OutcomeHealth {
    Succeeded,
    Failed,
}

pub(crate) struct AgentOutcome {
    pub health: OutcomeHealth,
    pub usage: Option<TokenUsage>,
    pub tokens_delta: u64,
    pub cost_delta: f64,
    pub dispatch_accounted: bool,
}

impl AgentOutcome {
    fn apply(&self, agent: &mut EngineAgent) {
        if let Some(usage) = &self.usage {
            agent.economics.token_usage = usage.clone();
        }
        agent.economics.tokens_used += self.tokens_delta;
        agent.economics.cost_usd += self.cost_delta;
        match self.health {
            OutcomeHealth::Succeeded => agent.health.failure_count = 0,
            OutcomeHealth::Failed => {
                agent.health.failure_count += 1;
                agent.health.last_failure_at = Some(chrono::Utc::now());
            }
        }
        agent.health.status = "idle".to_string();
        agent.state.current_task = None;
        agent.state.active_mission = None;
    }
}

fn check_owner(
    state: &AppState,
    agent_id: &str,
    generation: u64,
    cancel: &tokio_util::sync::CancellationToken,
) -> Result<(), AppError> {
    if cancel.is_cancelled() || state.comms.agent_generation(agent_id) != generation {
        return Err(AppError::Conflict(
            "Agent outcome belongs to a superseded execution".into(),
        ));
    }
    Ok(())
}

/// Commit an owned outcome before exposing its state or completion event.
pub(crate) async fn persist_agent_outcome(
    state: &AppState,
    agent_id: &str,
    generation: Option<u64>,
    outcome: AgentOutcome,
) -> Result<(), AppError> {
    let (_execution, cancel) = state.comms.begin_execution(agent_id)?;
    let generation = generation.unwrap_or_else(|| state.comms.agent_generation(agent_id));
    let _write = state.comms.agent_write_gate.lock().await;
    check_owner(state, agent_id, generation, &cancel)?;

    let live = state
        .registry
        .agents
        .get(agent_id)
        .map(|entry| entry.value().clone());
    let settled_economics = live
        .as_ref()
        .filter(|_| outcome.dispatch_accounted)
        .map(|agent| agent.economics.clone());
    let mut baseline = match live {
        Some(agent) => agent,
        None => {
            match crate::agent::persistence::load_agent_by_id_db(&state.resources.pool, agent_id)
                .await?
            {
                Some(agent) => agent,
                // A deleted/missing registry agent must not prevent mission outcome persistence.
                None => return Ok(()),
            }
        }
    };

    for retry in 0..=5 {
        check_owner(state, agent_id, generation, &cancel)?;
        let mut candidate = baseline.clone();
        if let Some(economics) = &settled_economics {
            candidate.economics = economics.clone();
        }
        outcome.apply(&mut candidate);
        match crate::agent::persistence::save_agent_db(&state.resources.pool, &mut candidate).await
        {
            Ok(()) => {
                // Stop/replacement may have advanced ownership while SQLite was committing.
                // Always publish the committed version, but never restore its terminal fields
                // into a newer execution or overwrite concurrently edited configuration.
                let snapshot = {
                    let _gate = state.comms.execution_gate.lock();
                    let current = !cancel.is_cancelled()
                        && state.comms.agent_generation(agent_id) == generation;
                    state
                        .registry
                        .agents
                        .get_mut(agent_id)
                        .and_then(|mut live| {
                            live.version = live.version.max(candidate.version);
                            if !current {
                                return None;
                            }
                            live.economics = candidate.economics.clone();
                            live.health.failure_count = candidate.health.failure_count;
                            live.health.last_failure_at = candidate.health.last_failure_at;
                            live.health.status = candidate.health.status.clone();
                            live.state.current_task = None;
                            live.state.active_mission = None;
                            Some(crate::routes::agent::AgentResponse::from(&*live))
                        })
                };
                check_owner(state, agent_id, generation, &cancel)?;
                if let Some(snapshot) = snapshot {
                    state.emit_event(serde_json::json!({
                        "type": "agent:update", "agent_id": agent_id, "data": snapshot
                    }));
                }
                return Ok(());
            }
            Err(AppError::Conflict(_)) if retry < 5 => {
                baseline =
                    crate::agent::persistence::load_agent_by_id_db(&state.resources.pool, agent_id)
                        .await?
                        .ok_or_else(|| {
                            AppError::NotFound(format!(
                                "Agent '{agent_id}' disappeared during finalization"
                            ))
                        })?;
                tokio::time::sleep(std::time::Duration::from_millis(50 * (retry + 1))).await;
            }
            Err(error) => return Err(error),
        }
    }
    unreachable!("bounded retry returns on its final attempt")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn success() -> AgentOutcome {
        AgentOutcome {
            health: OutcomeHealth::Succeeded,
            usage: None,
            tokens_delta: 7,
            cost_delta: 0.25,
            dispatch_accounted: false,
        }
    }

    async fn seeded() -> AppState {
        let state = AppState::new_minimal_mock().await;
        let mut agent = EngineAgent::default();
        agent.identity.id = "outcome-owner".into();
        agent.health.status = "busy".into();
        agent.economics.tokens_used = 10;
        crate::agent::persistence::save_agent_db(&state.resources.pool, &mut agent)
            .await
            .unwrap();
        state
            .registry
            .agents
            .insert(agent.identity.id.clone(), agent);
        state
    }

    #[tokio::test]
    async fn conflicting_outcome_rebases_delta_once_and_publishes_after_save() {
        let state = seeded().await;
        let mut events = state.comms.event_tx.subscribe();
        let mut concurrent =
            crate::agent::persistence::load_agent_by_id_db(&state.resources.pool, "outcome-owner")
                .await
                .unwrap()
                .unwrap();
        concurrent.economics.tokens_used = 50;
        concurrent.economics.cost_usd = 5.0;
        crate::agent::persistence::save_agent_db(&state.resources.pool, &mut concurrent)
            .await
            .unwrap();
        persist_agent_outcome(&state, "outcome-owner", Some(0), success())
            .await
            .unwrap();
        let saved =
            crate::agent::persistence::load_agent_by_id_db(&state.resources.pool, "outcome-owner")
                .await
                .unwrap()
                .unwrap();
        assert_eq!(saved.economics.tokens_used, 57);
        assert_eq!(saved.economics.cost_usd, 5.25);
        assert_eq!(saved.health.status, "idle");
        assert_eq!(
            state.registry.agents.get("outcome-owner").unwrap().version,
            saved.version
        );
        assert_eq!(events.try_recv().unwrap()["type"], "agent:update");
    }

    #[tokio::test]
    async fn superseded_generation_cannot_publish_or_mutate_an_outcome() {
        let state = seeded().await;
        let mut events = state.comms.event_tx.subscribe();
        state.comms.advance_agent_generation("outcome-owner");
        assert!(matches!(
            persist_agent_outcome(&state, "outcome-owner", Some(0), success()).await,
            Err(AppError::Conflict(_))
        ));
        assert_eq!(
            state
                .registry
                .agents
                .get("outcome-owner")
                .unwrap()
                .health
                .status,
            "busy"
        );
        assert!(events.try_recv().is_err());
    }

    #[tokio::test]
    async fn failed_durable_write_keeps_registry_and_events_unchanged() {
        let state = seeded().await;
        let mut events = state.comms.event_tx.subscribe();
        state.resources.pool.close().await;
        assert!(
            persist_agent_outcome(&state, "outcome-owner", Some(0), success())
                .await
                .is_err()
        );
        let current = state.registry.agents.get("outcome-owner").unwrap();
        assert_eq!(current.health.status, "busy");
        assert_eq!(current.economics.tokens_used, 10);
        assert!(events.try_recv().is_err());
    }
}

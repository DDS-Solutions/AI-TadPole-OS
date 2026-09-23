//! @docs ARCHITECTURE:Networking
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / HTTP Routes / Agent State Recovery
//! - **Primary Entrypoints**: `recover_active_agents`, `reset_agent_to_idle_safe`
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none declared
//! - **Telemetry Targets**: none declared
//! - **Witness Tests**: none declared

use super::models::{STATUS_BUSY, STATUS_IDLE};
use super::tasks::spawn_agent_runner;
use crate::state::AppState;
use futures::StreamExt;
use std::sync::Arc;

const DEFAULT_RECOVERY_CONCURRENCY: usize = 2;
const DEFAULT_RECOVERY_STAGGER_MS: u64 = 1500;

fn get_recovery_concurrency() -> usize {
    std::env::var("STARTUP_RECOVERY_CONCURRENCY")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(DEFAULT_RECOVERY_CONCURRENCY)
}

fn get_recovery_stagger_ms() -> u64 {
    std::env::var("STARTUP_RECOVERY_STAGGER_MS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(DEFAULT_RECOVERY_STAGGER_MS)
}

/// Scans the database on startup and resumes runners for agents found in an active ("busy") state.
/// Implements progressive staggering and bounded concurrency to prevent upstream rate-limit (429) spikes.
pub async fn recover_active_agents(state: Arc<AppState>) {
    let agents: Vec<crate::agent::types::EngineAgent> = state
        .registry
        .agents
        .iter()
        .map(|kv| kv.value().clone())
        .collect();

    let busy_agents: Vec<_> = agents
        .into_iter()
        .filter(|a| {
            a.health.status == STATUS_BUSY
                || a.health.status == "working"
                || a.health.status == "thinking"
                || a.health.status == "active"
        })
        .collect();

    if busy_agents.is_empty() {
        return;
    }

    let concurrency = get_recovery_concurrency();
    let stagger_ms = get_recovery_stagger_ms();

    tracing::info!(
        "🔄 [State Recovery] Initiating staggered startup recovery for {} active agents (concurrency: {}, stagger: {}ms)...",
        busy_agents.len(),
        concurrency,
        stagger_ms
    );

    let indexed_agents: Vec<(usize, crate::agent::types::EngineAgent)> =
        busy_agents.into_iter().enumerate().collect();

    // Bounded concurrency recovery with progressive startup stagger (F-10, RATE-01)
    futures::stream::iter(indexed_agents)
        .for_each_concurrent(concurrency, |(idx, agent)| {
            let state = state.clone();
            async move {
                if stagger_ms > 0 && idx > 0 {
                    let delay = (idx as u64) * stagger_ms;
                    tracing::debug!(
                        "⏳ [State Recovery] Staggering agent recovery for {} by {}ms (slot {})...",
                        agent.identity.id,
                        delay,
                        idx
                    );
                    tokio::time::sleep(std::time::Duration::from_millis(delay)).await;
                }

                if let Some(task) = agent.state.current_task.clone() {
                    if !task.is_empty() {
                        let agent_id = agent.identity.id.clone();
                        let cluster_id = agent
                            .state
                            .active_mission
                            .as_ref()
                            .and_then(|m| m.get("id"))
                            .and_then(|id| id.as_str())
                            .map(|s| s.to_string());

                        tracing::info!(
                            "🔄 [State Recovery] Recovering active agent {} (slot {}) for task: {}",
                            agent_id,
                            idx,
                            task
                        );

                        let payload = crate::agent::types::TaskPayload {
                            message: task,
                            cluster_id,
                            ..Default::default()
                        };

                        let (join_handle, runner_handle) =
                            spawn_agent_runner(&state, &agent_id, payload);
                        state
                            .comms
                            .active_runners
                            .insert(agent_id.clone(), runner_handle);

                        let _ = join_handle.await;
                    } else {
                        reset_agent_to_idle_safe(&state, &agent.identity.id).await;
                    }
                } else {
                    reset_agent_to_idle_safe(&state, &agent.identity.id).await;
                }
            }
        })
        .await;
}

/// Safely resets an agent's status to IDLE in the database before updating memory registry.
pub async fn reset_agent_to_idle_safe(state: &Arc<AppState>, agent_id: &str) {
    let mut clone = state
        .registry
        .agents
        .get(agent_id)
        .map(|e| e.value().clone());
    if let Some(ref mut a) = clone {
        a.health.status = STATUS_IDLE.to_string();
        match crate::agent::persistence::save_agent_db(&state.resources.pool, a).await {
            Ok(()) => {
                if let Some(mut entry) = state.registry.agents.get_mut(agent_id) {
                    *entry = a.clone();
                }
            }
            Err(e) => {
                tracing::error!(
                    "❌ [State Recovery] Failed to reset agent {} to idle: {}. Memory NOT modified.",
                    agent_id, e
                );
            }
        }
    }
}

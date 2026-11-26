//! @docs ARCHITECTURE:State
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / comm
//!
//! ### 🔍 Debugging & Observability
//! - **Telemetry Targets**: none declared
//!
//! ### AI Assist Note
//! - Keep cancellation ownership and usage persistence aligned with the existing execution paths.

use crate::agent::types::OversightEntry;
use crate::types::LogEntry;
use dashmap::DashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, oneshot};

#[derive(Clone, Debug)]
pub struct RunnerHandle {
    pub abort_handle: tokio::task::AbortHandle,
    pub task_id: String,
}

/// Hub for real-time broadcast and event orchestration.
pub struct CommunicationHub {
    /// Broadcast system logs to all connected UI WebSockets.
    pub tx: broadcast::Sender<LogEntry>,
    /// Dedicated broadcast for Engine events (decisions, lifecycle changes).
    pub event_tx: broadcast::Sender<serde_json::Value>,
    /// Dedicated high-speed broadcast for agent telemetry (thinking, status).
    pub telemetry_tx: broadcast::Sender<serde_json::Value>,
    /// Dedicated high-speed broadcast for neural audio streams (PCM chunks).
    pub audio_stream_tx: broadcast::Sender<Vec<u8>>,
    /// High-speed binary pulse broadcasting for swarm visualization.
    pub pulse_tx: broadcast::Sender<Arc<crate::telemetry::pulse_types::SwarmPulse>>,
    /// Pending Oversight entries awaiting human decision.
    pub oversight_queue: DashMap<String, OversightEntry>,
    /// Resolvers for pending oversight promises.
    pub oversight_resolvers:
        DashMap<String, oneshot::Sender<crate::agent::types::OversightResolution>>,
    /// Active AbortHandles and task IDs for running agents, allowing for definitive task cancellation.
    pub active_runners: DashMap<String, RunnerHandle>,
    /// Direct runners and scheduled/workflow futures share cancellation ownership.
    pub active_executions: DashMap<String, (String, tokio_util::sync::CancellationToken)>,
    pub execution_generations: DashMap<String, u64>,
    pub stopped_agents: dashmap::DashSet<String>,
    pub engine_stopped: std::sync::atomic::AtomicBool,
    pub(crate) execution_gate: parking_lot::Mutex<()>,
    pub(crate) settings_write_gate: parking_lot::Mutex<()>,
    pub(crate) execution_control: tokio::sync::Mutex<()>,
    pub(crate) agent_write_gate: tokio::sync::Mutex<()>,
    pub execution_changed: tokio::sync::Notify,
    /// Sliding window cache of recently processed X-Request-Ids to prevent duplicate task execution.
    pub recent_requests: DashMap<String, std::time::Instant>,
    /// Semaphore to limit concurrent executing agents (runner pool throttle)
    pub runner_semaphore: tokio::sync::Semaphore,
    /// Monotonic sequence counter for outbound engine events.
    pub event_sequence: std::sync::atomic::AtomicU64,
}

pub(crate) struct ExecutionGuard {
    comms: Arc<CommunicationHub>,
    id: String,
}

impl Drop for ExecutionGuard {
    fn drop(&mut self) {
        self.comms.active_executions.remove(&self.id);
        self.comms.execution_changed.notify_waiters();
    }
}

impl CommunicationHub {
    /// Cleanup is registered even after stop so the operator waits for its persistence.
    pub(crate) fn begin_teardown(self: &Arc<Self>, agent_id: &str) -> ExecutionGuard {
        let _gate = self.execution_gate.lock();
        let id = uuid::Uuid::new_v4().to_string();
        self.active_executions.insert(
            id.clone(),
            (
                agent_id.to_string(),
                tokio_util::sync::CancellationToken::new(),
            ),
        );
        ExecutionGuard {
            comms: Arc::clone(self),
            id,
        }
    }

    pub(crate) fn begin_execution(
        self: &Arc<Self>,
        agent_id: &str,
    ) -> Result<(ExecutionGuard, tokio_util::sync::CancellationToken), crate::error::AppError> {
        let _gate = self.execution_gate.lock();
        if self
            .engine_stopped
            .load(std::sync::atomic::Ordering::Acquire)
            || self.stopped_agents.contains(agent_id)
        {
            return Err(crate::error::AppError::Conflict(
                "Execution is stopped; explicitly resume before dispatch".to_string(),
            ));
        }
        let id = uuid::Uuid::new_v4().to_string();
        let token = tokio_util::sync::CancellationToken::new();
        self.active_executions
            .insert(id.clone(), (agent_id.to_string(), token.clone()));
        Ok((
            ExecutionGuard {
                comms: Arc::clone(self),
                id,
            },
            token,
        ))
    }

    pub(crate) fn agent_generation(&self, agent_id: &str) -> u64 {
        *self
            .execution_generations
            .entry(agent_id.to_string())
            .or_insert(0)
    }

    pub(crate) fn advance_agent_generation(&self, agent_id: &str) {
        let mut entry = self
            .execution_generations
            .entry(agent_id.to_string())
            .or_insert(0);
        *entry = entry.wrapping_add(1);
    }

    pub(crate) fn resume_agent_execution(&self, agent_id: &str) {
        let _gate = self.execution_gate.lock();
        self.stopped_agents.remove(agent_id);
    }

    /// Cancel before changing terminal state; reject new work while teardown runs.
    pub(crate) async fn stop_executions(
        &self,
        agent_id: Option<&str>,
    ) -> Result<(), crate::error::AppError> {
        let aborted = {
            let _gate = self.execution_gate.lock();
            if let Some(id) = agent_id {
                self.stopped_agents.insert(id.to_string());
                self.advance_agent_generation(id);
            } else {
                self.engine_stopped
                    .store(true, std::sync::atomic::Ordering::Release);
                for mut generation in self.execution_generations.iter_mut() {
                    *generation = generation.wrapping_add(1);
                }
            }
            for execution in self.active_executions.iter() {
                if agent_id.is_none_or(|id| execution.value().0 == id) {
                    execution.value().1.cancel();
                }
            }
            let keys: Vec<_> = self
                .active_runners
                .iter()
                .filter(|entry| agent_id.is_none_or(|id| entry.key() == id))
                .map(|entry| entry.key().clone())
                .collect();
            keys.into_iter()
                .filter_map(|key| self.active_runners.remove(&key))
                .map(|(_, handle)| {
                    handle.abort_handle.abort();
                    handle.abort_handle
                })
                .collect::<Vec<_>>()
        };
        tokio::time::timeout(std::time::Duration::from_secs(5), async {
            loop {
                let changed = self.execution_changed.notified();
                tokio::pin!(changed);
                changed.as_mut().enable();
                let running = self
                    .active_executions
                    .iter()
                    .any(|entry| agent_id.is_none_or(|id| entry.value().0 == id));
                if !running && aborted.iter().all(tokio::task::AbortHandle::is_finished) {
                    break;
                }
                tokio::select! {
                    _ = changed => {},
                    _ = tokio::time::sleep(std::time::Duration::from_millis(5)) => {},
                }
            }
        })
        .await
        .map_err(|_| {
            crate::error::AppError::Conflict(
                "Execution teardown is still pending; stop remains latched".to_string(),
            )
        })
    }
}

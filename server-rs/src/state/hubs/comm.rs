//! @docs ARCHITECTURE:State
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / comm
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Type-safe state handling and bounded execution without unhandled panics.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: none declared
//! - **Witness Tests**: none declared

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
    /// Sliding window cache of recently processed X-Request-Ids to prevent duplicate task execution.
    pub recent_requests: DashMap<String, std::time::Instant>,
    /// Semaphore to limit concurrent executing agents (runner pool throttle)
    pub runner_semaphore: tokio::sync::Semaphore,
    /// Monotonic sequence counter for outbound engine events.
    pub event_sequence: std::sync::atomic::AtomicU64,
}

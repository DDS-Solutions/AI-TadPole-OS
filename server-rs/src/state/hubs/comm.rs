//! Communication Hub — Real-time event broadcasting and oversight resolution
//!
//! Manages low-latency WebSocket streams for telemetry, system logs,
//! and human-in-the-loop (Oversight) interaction.
//!
//! @docs ARCHITECTURE:State

use std::sync::Arc;
use dashmap::DashMap;
use tokio::sync::{broadcast, oneshot};
use crate::agent::types::OversightEntry;
use crate::types::LogEntry;

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
    pub oversight_resolvers: DashMap<String, oneshot::Sender<bool>>,
}

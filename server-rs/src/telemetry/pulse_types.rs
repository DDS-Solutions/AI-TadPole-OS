//! Swarm Pulse Types — Binary Telemetry Schemas
//!
//! @docs ARCHITECTURE:TelemetryBridge
//!
//! ### AI Assist Note
//! **Telemetry Schema**: Data structures for the Swarm Pulse real-time visualizer.
//! Facilitates high-frequency coordinate and status updates for active agents in the field.
//!
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: Pulse drift, overflow on coordinate mapping, or status code mismatch with UI.
//! - **Telemetry Link**: Search for `[PulseCollector]` or `SwarmPulse` in backend logs.

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PulseNode {
    pub id: String,
    pub x: f32,
    pub y: f32,
    pub status: u8, // 0: idle, 1: busy, 2: error, 3: degraded
    pub battery: u8,
    pub signal: u8,
    pub progress: f32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PulseConnection {
    pub source: String,
    pub target: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SwarmPulse {
    pub timestamp: u64,
    pub nodes: Vec<PulseNode>,
    pub edges: Vec<PulseConnection>,
}

impl SwarmPulse {
    pub fn new(timestamp: u64) -> Self {
        Self {
            timestamp,
            nodes: Vec::new(),
            edges: Vec::new(),
        }
    }
}

// Metadata: [pulse_types]

// Metadata: [pulse_types]

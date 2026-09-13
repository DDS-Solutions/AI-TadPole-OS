//! @docs ARCHITECTURE:TelemetryBridge
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / pulse_types
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Type-safe state handling and bounded execution without unhandled panics.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: none declared
//! - **Witness Tests**: none declared

use serde::{Deserialize, Serialize};

/// Standard status codes for Swarm Pulse visualizer nodes.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[repr(u8)]
pub enum PulseNodeStatus {
    Idle = 0,
    Busy = 1,
    Error = 2,
    Degraded = 3,
    MissionHub = 4,
}

impl PulseNodeStatus {
    pub fn as_u8(self) -> u8 {
        self as u8
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct PulseNode {
    pub id: String,
    pub x: f32,
    pub y: f32,
    pub status: u8, // 0: idle, 1: busy, 2: error, 3: degraded, 4: mission hub
    pub battery: u8,
    pub signal: u8,
    pub progress: f32,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct PulseConnection {
    pub source: String,
    pub target: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
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

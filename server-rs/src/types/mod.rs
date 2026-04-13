//! @docs ARCHITECTURE:DataTypes
//! @docs OPERATIONS_MANUAL:Telemetry
//! 
//! ### AI Assist Note
//! **Common Data Types (Engine's DNA)**: Orchestrates the centralized 
//! definitions for shared structures within the Tadpole OS engine. 
//! Features **Telemetry Parity**: the `LogEntry` structure mirrors the 
//! frontend expectations exactly to ensure seamless event 
//! visualization. Implements the **Subsystem Lifecycle State Machine**: 
//! `SubsystemStatus` provides the source of truth for the engine's 
//! boot sequence and reachability (`NotStarted` -> `Warming` -> 
//! `Ready`). AI agents should use the `@state` tags (e.g., 
//! `Initializing...`) to reason about component availability during 
//! autonomous mission execution (TYPE-01).
//! 
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: Status mismatch during the boot sequence, 
//!   malformed UUIDs in log entries, or severity level inconsistencies 
//!   causing UI filtering issues.
//! - **Telemetry Link**: Search for `[Status]` or `[Log]` in `tracing` 
//!   logs for lifecycle and event benchmarks.
//! - **Trace Scope**: `server-rs::types`

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub mod rag_scoring;

/// Exact parity with the `LogEntry` frontend interface.
/// Represents a single telemetry or system event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    #[serde(rename = "type")]
    pub event_type: String,
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub source: String,
    pub severity: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mission_id: Option<String>,
    pub text: String,
}

impl LogEntry {
    /// Creates a new log entry with a unique UUID and current timestamp.
    pub fn new(source: &str, text: &str, severity: &str, mission_id: Option<String>) -> Self {
        Self {
            event_type: "log".to_string(),
            id: Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            source: source.to_string(),
            text: text.to_string(),
            severity: severity.to_string(),
            agent_id: None,
            agent_name: None,
            mission_id,
        }
    }
}

/// Represents the initialization state of an engine subsystem.
///
/// This enum defines the lifecycle state machine for core components (CodeGraph, Audio, etc.).
/// Transitions typically follow: `NotStarted` -> `Warming(f32)` -> `Ready` | `Failed(String)`.
///
/// ### AI Assist Note
/// Subsystems in `Warming` state may respond with "Initializing..." if called.
/// Use `ready()` or `warming()` helper methods to check reachability.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "status", content = "data")]
pub enum SubsystemStatus {
    /// Subsystem is not yet started or explicitly skipped in Fast-Path.
    /// @state: Initial
    NotStarted,
    /// Subsystem is currently warming up (payload is progress 0.0 to 1.0).
    /// @state: Transitioning
    Warming(f32),
    /// Subsystem is fully initialized and ready for mission execution.
    /// @state: Terminal(Success)
    Ready,
    /// Subsystem failed to initialize. Payload contains the error message.
    /// @state: Terminal(Failure)
    Failed(String),
}

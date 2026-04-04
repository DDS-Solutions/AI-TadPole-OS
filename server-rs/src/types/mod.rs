//! Common Data Types & Telemetry Schemas - The Engine's DNA
//!
//! Centralized definitions for shared data structures to ensure consistency 
//! across backend modules and documentation.
//!
//! @docs ARCHITECTURE:DataTypes
//! @docs OPERATIONS_MANUAL:Telemetry
//!
//! ### AI Assist Note
//! **Telemetry Parity**: `LogEntry` structure mirrors the frontend expectation exactly. 
//! `SubsystemStatus` is the source of truth for the engine's boot sequence. Use 
//! the `@state` tags to reason about the availability of the vector DB or audio stack.
//!
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: Status mismatch during boot, or malformed UUIDs in log entries.
//! - **Telemetry Link**: Search for `[Status]` or `[Log]` in `tracing` logs.
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

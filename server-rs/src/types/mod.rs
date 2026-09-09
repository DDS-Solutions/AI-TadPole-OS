//! @docs ARCHITECTURE:DataTypes
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / mod
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Type-safe state handling and bounded execution without unhandled panics.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: none declared
//! - **Witness Tests**: none declared

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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kind: Option<u32>,
    pub text: String,
}

/// Standardized Telemetry Event Kinds
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[repr(u32)]
pub enum TelemetryKind {
    AgentThought = 101,
    ToolCall = 102,
    A2aLedgerTx = 201,
    SecurityViolation = 301,
}

impl TelemetryKind {
    #[allow(dead_code)]
    pub fn as_u32(self) -> u32 {
        self as u32
    }
}

pub const KIND_AGENT_THOUGHT: u32 = TelemetryKind::AgentThought as u32;
#[allow(dead_code)]
pub const KIND_TOOL_CALL: u32 = TelemetryKind::ToolCall as u32;
#[allow(dead_code)]
pub const KIND_A2A_LEDGER_TX: u32 = TelemetryKind::A2aLedgerTx as u32;
#[allow(dead_code)]
pub const KIND_SECURITY_VIOLATION: u32 = TelemetryKind::SecurityViolation as u32;

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
            kind: None,
        }
    }

    /// Sets the telemetry kind code.
    pub fn with_kind(mut self, kind: u32) -> Self {
        self.kind = Some(kind);
        self
    }
}

/// Represents the initialization state of an engine subsystem.
///
/// This enum defines the lifecycle state machine for core components (CodeGraph, Audio, etc.).
/// Transitions typically follow: `NotStarted` -> `Warming(f32)` -> `Ready` | `Failed(String)`.
///
/// **Note**: Subsystems in `Warming` state may respond with "Initializing..." if called.
/// Use `is_reachable()` or `is_ready()` helper methods to check reachability.
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

impl SubsystemStatus {
    /// Creates a warming status with progress clamped to `[0.0, 1.0]`.
    #[allow(dead_code)]
    pub fn warming(progress: f32) -> Self {
        Self::Warming(if progress.is_nan() {
            0.0
        } else {
            progress.clamp(0.0, 1.0)
        })
    }

    /// Returns true if the subsystem is ready or warming (accessible for requests/metrics).
    #[allow(dead_code)]
    pub fn is_reachable(&self) -> bool {
        matches!(self, SubsystemStatus::Ready | SubsystemStatus::Warming(_))
    }

    /// Returns true if the subsystem is fully ready.
    #[allow(dead_code)]
    pub fn is_ready(&self) -> bool {
        matches!(self, SubsystemStatus::Ready)
    }

    /// Returns the warming progress if in the Warming state.
    #[allow(dead_code)]
    pub fn progress(&self) -> Option<f32> {
        match self {
            SubsystemStatus::Warming(p) => Some(*p),
            _ => None,
        }
    }

    /// Returns the error message if in the Failed state.
    #[allow(dead_code)]
    pub fn error(&self) -> Option<&str> {
        match self {
            SubsystemStatus::Failed(err) => Some(err.as_str()),
            _ => None,
        }
    }
}

/// Represents the overall health state of the Swarm Engine.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum SystemHealthState {
    /// Some critical subsystems are still warming up, and no failures have occurred.
    Warming,
    /// All critical subsystems are fully operational.
    Ready,
    /// One or more critical subsystems failed to initialize or encountered a fatal error.
    Degraded,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_entry_kind_builder() {
        let entry =
            LogEntry::new("test_source", "test text", "info", None).with_kind(KIND_AGENT_THOUGHT);
        assert_eq!(entry.kind, Some(101));

        let json = serde_json::to_string(&entry).unwrap();
        assert!(json.contains("\"kind\":101"));
    }

    #[test]
    fn test_log_entry_omits_none_kind() {
        let entry = LogEntry::new("test_source", "test text", "info", None);
        let json = serde_json::to_string(&entry).unwrap();
        assert!(!json.contains("\"kind\""));
    }

    #[test]
    fn test_subsystem_status_helpers_and_clamping() {
        let not_started = SubsystemStatus::NotStarted;
        assert!(!not_started.is_reachable());
        assert!(!not_started.is_ready());
        assert_eq!(not_started.progress(), None);
        assert_eq!(not_started.error(), None);

        let warming_clamped = SubsystemStatus::warming(1.5);
        assert_eq!(warming_clamped, SubsystemStatus::Warming(1.0));
        assert!(warming_clamped.is_reachable());
        assert!(!warming_clamped.is_ready());
        assert_eq!(warming_clamped.progress(), Some(1.0));

        let warming_nan = SubsystemStatus::warming(f32::NAN);
        assert_eq!(warming_nan, SubsystemStatus::Warming(0.0));

        let ready = SubsystemStatus::Ready;
        assert!(ready.is_reachable());
        assert!(ready.is_ready());
        assert_eq!(ready.progress(), None);

        let failed = SubsystemStatus::Failed("Disk full".to_string());
        assert!(!failed.is_reachable());
        assert!(!failed.is_ready());
        assert_eq!(failed.error(), Some("Disk full"));
    }
}

//! @docs ARCHITECTURE:Registry
//! 
//! ### AI Assist Note
//! **Core technical module for the Tadpole OS hardened engine.**
//! This module implements high-fidelity logic for the Sovereign Reality layer.
//! 
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: Runtime logic error, state desynchronization, or resource exhaustion.
//! - **Telemetry Link**: Search `[constants.rs]` in tracing logs.
//!
//! @docs ARCHITECTURE:Agent Identities
//!
//! ### AI Assist Note
//! **Agent Constants**: Centralizes the unique identifiers for common
//! agents in the Tadpole OS swarm. Orchestrators and system-level nodes
//! should reference these constants rather than hardcoded strings to
//! ensure consistency across the synthesis and lifecycle layers (ID-01).

/// The CEO (Orchestrator-in-Chief). Primary node for mission planning and delegation.
pub const AGENT_CEO: &str = "1";

/// The COO (Chief Operations Officer). Secondary node for mission management.
pub const AGENT_COO: &str = "2";

/// The Alpha Agent. Legacy identifier or special core utility agent.
pub const AGENT_ALPHA: &str = "alpha";

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

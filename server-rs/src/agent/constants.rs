//! @docs ARCHITECTURE:Registry
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Agent Runner / constants
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Type-safe state handling and bounded execution without unhandled panics.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: none declared
//! - **Witness Tests**: none declared

pub const AGENT_CEO: &str = "1";

/// The COO (Chief Operations Officer). Secondary node for mission management.
pub const AGENT_COO: &str = "2";

/// The Alpha Agent. Legacy identifier or special core utility agent.
pub const AGENT_ALPHA: &str = "alpha";

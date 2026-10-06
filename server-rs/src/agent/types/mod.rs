//! @docs ARCHITECTURE:Agent
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Agent Runner / mod
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Type-safe state handling and bounded execution without unhandled panics.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: none declared
//! - **Witness Tests**: none declared

pub mod agent;
pub mod mission;
pub mod model;
pub mod oversight;
pub mod swarm;
pub mod tool;

#[cfg(test)]
pub mod tests;

pub use crate::agent::merge::AgentConfigUpdate;
pub use agent::*;
pub use mission::*;
pub use model::*;
pub use oversight::*;
pub use swarm::*;
pub use tool::*;

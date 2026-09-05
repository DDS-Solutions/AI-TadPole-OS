//! @docs ARCHITECTURE:Continuity:Workflow
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

pub mod engine;
pub mod executor;
pub mod helpers;
pub mod types;

#[cfg(test)]
mod tests;

pub use engine::WorkflowEngine;
pub use types::*;

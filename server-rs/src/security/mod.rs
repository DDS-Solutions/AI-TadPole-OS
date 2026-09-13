//! @docs ARCHITECTURE:ShieldLayer
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Security & Governance / mod
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Type-safe state handling and bounded execution without unhandled panics.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: none declared
//! - **Witness Tests**: none declared

pub mod audit;
pub mod conflict;
pub mod dependency_guard;
pub mod metering;
pub mod monitoring;
pub mod normalizer;
pub mod permissions;
pub mod scanner;
pub mod signed_capability;
pub mod skillspector;

#[cfg(test)]
mod permission_tests;
#[cfg(test)]
mod signed_capability_tests;

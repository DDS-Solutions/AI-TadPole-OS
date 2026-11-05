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
pub mod command_guard;
pub mod conflict;
pub mod dependency_guard;
pub mod metering;
pub mod monitoring;
pub mod normalizer;
pub mod path_guard;
pub mod permissions;
pub mod remote_protocol;
pub mod scanner;
pub mod signed_capability;
pub mod skillspector;
pub mod ssrf_guard;

#[cfg(test)]
mod permission_tests;
#[cfg(test)]
mod signed_capability_tests;

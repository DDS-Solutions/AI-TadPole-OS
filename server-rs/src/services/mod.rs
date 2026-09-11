//! @docs ARCHITECTURE:Core:Services
//!
//! ### AI Context Alignment
//! - **Subsystem**: Frontend Service Layer / mod
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Asynchronous service calls normalize response envelopes and propagate typed errors.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: none declared
//! - **Witness Tests**: none declared

pub mod acl_service;
pub mod blueprint_service;
pub mod bm25_memory;
pub mod cas;
pub mod cognitive_memory;
pub mod discovery;
pub mod privacy;

// Facade re-exports for primary service entry points
#[allow(unused_imports)]
pub use cas::{capture_pre_mutation, get_file_history, restore_file_version};

#[cfg(test)]
mod tests;

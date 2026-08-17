//! @docs ARCHITECTURE:State
//!
//! ### AI Assist Note
//! **System Services**: Orchestrates the core background processes and
//! long-running logic for the Tadpole OS engine. Features the
//! **Service Layer** pattern: separates business rules from
//! transport-specific route handlers. Includes **Discovery** (agent
//! registry scanning) and **Privacy** (data anonymization) services.
//! AI agents should utilize these services for cross-cutting logic
//! that exceeds the scope of a single request handler (SERV-01).
//!
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: Service-init timeouts, state-drift between
//!   in-memory registries and persistent storage, or performance
//!   bottlenecks during large-scale discovery scans.
//! - **Trace Scope**: `server-rs::services`

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

// Metadata: [mod]

//! @docs ARCHITECTURE:Agent
//!
//! ### AI Assist Note
//! **Unified Schemas**: Defines the source-of-truth data contracts for agents,
//! missions, and telemetry. Ensures **Serialization Parity** with the TypeScript
//! frontend via strict `serde` renaming (snake_case/camelCase bridge).
//! Features **IMR-01 (Intelligent Model Registry)** logic for automated model
//! discovery and capability inference (Vision, Tools, Reasoning).
//!
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: JSON deserialization mismatch (422 Unprocessable Entity),
//!   missing model config defaults leading to `None` pointer dereference
//!   logic errors, or invalid rate limit parsing from environment variables.
//! - **IMR-01 Integrity**: Verify that `ModelCapabilities` defaults match the
//!   conservative inference logic in `capability_matrix.rs`.
//! - **Trace Scope**: `server-rs::agent::types`

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

// Metadata: [types]

//! @docs ARCHITECTURE:Agent
//! @docs OPERATIONS_MANUAL:SwarmManagement
//!
//! ### AI Assist Note
//! **Swarm Orchestrator**: Primary namespace for agentic logic, LLM provider
//! abstraction, and the high-concurrency swarm runner. Coordinates between
//! `runner` (Execution), `types` (State), and `mission` (Persistence).
//! Features **Logical Hierarchy** enforcement to ensure agent-fluent
//! observability across the mission lifecycle.
//!
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: Provider timeouts, mission state corruption (hanging
//!   active missions), or rate-limit saturation.
//! - **Trace Scope**: `server-rs::agent` (Search for `[Agent]` or `[Mission]` tags)

// Sub-Modules (The Swarm Engine)
pub mod anthropic;
pub mod audio;
pub mod audio_cache;
pub mod backlog;
pub mod benchmarks;
pub mod capability_matrix;
pub mod connectors;
pub mod constants;
pub mod context_manager;
pub mod continuity;
pub mod gemini;
pub mod groq;
pub mod hooks;
pub mod mcp;
pub mod memory;
pub mod merge;
pub mod mission;
pub mod null_provider;
pub mod openai;
pub mod parser;
pub mod persistence;
pub mod provider_trait;
pub mod rate_limiter;
pub mod rates;
pub mod reaper;
pub mod recipes;
pub mod registry;
pub mod runner;
pub mod sanitizer;
pub mod script_skills;
pub mod skill_manifest;
pub mod types;
pub mod workflows;

// Verification Suite (Swarm Safety)
#[cfg(test)]
mod mcp_tests;

#[cfg(test)]
mod test_oversight;
#[cfg(test)]
mod test_sanitizer;
#[cfg(test)]
mod tests_governance;
#[cfg(test)]
mod tests_rate_limiter;
#[cfg(test)]
mod tests_skills;

// Duplicate SyncManifest removed. Canonical version is in crate::agent::types.

// Metadata: [mod]

// Metadata: [mod]

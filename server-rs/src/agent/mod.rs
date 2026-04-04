//! Missions & Swarms — Core agentic lifecycle orchestration
//!
//! Serves as the primary namespace for agentic logic, including LLM provider 
//! abstraction, mission lifecycle management, semantic memory, and the 
//! high-concurrency swarm runner.
//!
//! @docs ARCHITECTURE:Agent
//! @docs OPERATIONS_MANUAL:SwarmManagement
//!
//! @state SwarmState: (Initialized | Running | QuotaExhausted | Shutdown)
//! @state RegistryReady: (Loading | Syncing | Healthy | Corrupt)
//!
//! ### AI Assist Note
//! **Logic Hierarchy**: This module coordinates between `runner` (Execution), 
//! `types` (State), and `mission` (Persistence). Agents should prioritize 
//! `runner` for behavior modification and `types` for schema updates.
//!
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: Provider timeouts, mission state corruption (hanging active missions), or rate-limit saturation.
//! - **Telemetry Link**: Search for `[Agent]` or `[Mission]` in `tracing` logs.
//! - **Trace Scope**: `server-rs::agent`

// Sub-Modules (The Swarm Engine)
pub mod audio;
pub mod audio_cache;
pub mod backlog;
pub mod benchmarks;
pub mod connectors;
pub mod context_manager;
pub mod continuity;
pub mod gemini;
pub mod groq;
pub mod hooks;
pub mod mcp;
#[cfg(feature = "vector-memory")]
pub mod memory;
pub mod mission;
pub mod null_provider;
pub mod openai;
pub mod parser;
pub mod persistence;
pub mod provider_trait;
pub mod rate_limiter;
pub mod rates;
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
#[cfg(all(test, feature = "vector-memory"))]
mod persistence_tests;
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

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// SME Data Connector Sync Manifest
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct SyncManifest {
    pub id: String,
    pub agent_id: String,
    pub source_type: String,
    pub source_uri: String,
    pub last_sync_at: DateTime<Utc>,
    pub checksum: Option<String>,
    pub status: String,
    pub metadata: Option<String>,
}

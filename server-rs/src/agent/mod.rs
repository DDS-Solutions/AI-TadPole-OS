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
pub mod constants;
pub mod connectors;
pub mod context_manager;
pub mod continuity;
pub mod gemini;
pub mod groq;
pub mod hooks;
pub mod mcp;
pub mod merge;
#[cfg(feature = "vector-memory")]
pub mod memory;
pub mod mission;
pub mod null_provider;
pub mod openai;
pub mod recipes;
pub mod parser;
pub mod persistence;
pub mod reaper;
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
/// 
/// ### 🛰️ Data Orchestration
/// Represents a tracked external resource (JSON, PDF, CSV, API) that a 
/// specialist agent is monitoring. The `last_sync_at` and `checksum` fields 
/// are used to ensure that the engine only ingest data that has changed 
/// since the last heartbeat (SYNC-01).
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct SyncManifest {
    /// Unique ID for this manifest entry.
    pub id: String,
    /// The agent responsible for this data stream.
    pub agent_id: String,
    pub source_type: String,
    pub source_uri: String,
    pub last_sync_at: DateTime<Utc>,
    /// Cryptographic checksum of the source data to detect delta changes.
    pub checksum: Option<String>,
    pub status: String,
    pub metadata: Option<String>,
}

// Metadata: [mod]

// Metadata: [mod]

//! Autonomous agent orchestration and reasoning system.
//!
//! This module serves as the primary namespace for all agent-related logic,
//! including LLM provider integrations, mission lifecycle management,
//! memory persistence, and the high-concurrency swarm runner.

pub mod audio;
pub mod audio_cache;
pub mod benchmarks;
pub mod connectors;
pub mod context_manager;
pub mod continuity;
pub mod gemini;
pub mod groq;
pub mod hooks;
pub mod mcp;
#[cfg(test)]
mod mcp_tests;
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
pub mod types;
pub mod workflows;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

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

//! @docs ARCHITECTURE:Registry
//! 
//! ### AI Assist Note
//! **Core technical resource for the Tadpole OS Sovereign infrastructure.**
//! This module implements high-fidelity logic for the Sovereign Reality layer.
//! 
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: Runtime logic error, state desynchronization, or resource exhaustion.
//! - **Telemetry Link**: Search `[mission]` in tracing logs.

use serde::{Deserialize, Serialize};
use super::model::ModelProvider;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, sqlx::Type, specta::Type)]
#[sqlx(rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum MissionStatus {
    Pending,
    SpecReview,
    Active,
    Completed,
    Failed,
    Paused,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct TaskPayload {
    pub message: String,
    pub cluster_id: Option<String>,
    pub department: Option<String>,
    pub provider: Option<ModelProvider>,
    pub model_id: Option<String>,
    pub api_key: Option<String>,
    pub base_url: Option<String>,
    pub rpm: Option<u32>,
    pub tpm: Option<u32>,
    pub rpd: Option<u32>,
    pub tpd: Option<u32>,
    pub budget_usd: Option<f64>,
    pub swarm_depth: Option<u32>,
    pub swarm_lineage: Option<Vec<String>>,
    pub external_id: Option<String>,
    pub safe_mode: Option<bool>,
    pub analysis: Option<bool>,
    pub traceparent: Option<String>,
    pub user_id: Option<String>,
    #[serde(default)]
    pub context_files: Option<Vec<String>>,
    #[serde(default)]
    pub recent_findings: Option<String>,
    #[serde(default)]
    pub structured_output: Option<bool>,
    #[serde(default, alias = "primaryGoal")]
    pub primary_goal: Option<String>,
    #[serde(default, alias = "allowedFiles")]
    pub allowed_files: Option<Vec<String>>,
    #[serde(default, alias = "visibleTranscript")]
    pub visible_transcript: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow, specta::Type)]
pub struct Mission {
    pub id: String,
    pub agent_id: String,
    pub title: String,
    pub status: MissionStatus,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub budget_usd: f64,
    pub cost_usd: f64,
    pub is_degraded: Option<bool>,
    pub is_pinned: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow, specta::Type)]
pub struct MissionLog {
    pub id: String,
    pub mission_id: String,
    pub agent_id: String,
    pub source: String,
    pub text: String,
    pub severity: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub metadata: Option<serde_json::Value>,
    pub hash: Option<String>,
    pub prev_hash: Option<String>,
}

/// Enhanced memory entry with scoring metadata for Advanced RAG.
#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct MemoryEntryDetailed {
    pub id: String,
    pub text: String,
    pub mission_id: String,
    pub timestamp: i64,
    /// Raw semantic distance from vector search.
    pub distance: f32,
    /// Final calculated Multi-Factor Score.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub score: Option<crate::types::rag_scoring::RagScore>,
}

// Metadata: [mission]

//! @docs ARCHITECTURE:Registry
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Agent Runner / mission
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Type-safe state handling and bounded execution without unhandled panics.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: none declared
//! - **Witness Tests**: none declared

use super::model::ModelProvider;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type, specta::Type)]
#[sqlx(rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum MissionStatus {
    Pending,
    #[serde(alias = "specreview", alias = "spec_review", alias = "spec-review")]
    #[sqlx(rename = "spec_review")]
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
    #[serde(default, alias = "cluster_id")]
    pub cluster_id: Option<String>,
    pub department: Option<String>,
    pub provider: Option<ModelProvider>,
    #[serde(default, alias = "model_id")]
    pub model_id: Option<String>,
    #[serde(default, alias = "api_key")]
    pub api_key: Option<String>,
    #[serde(default, alias = "base_url")]
    pub base_url: Option<String>,
    pub rpm: Option<u32>,
    pub tpm: Option<u32>,
    pub rpd: Option<u32>,
    pub tpd: Option<u32>,
    #[serde(default, alias = "budget_usd")]
    pub budget_usd: Option<f64>,
    #[serde(default, alias = "sub_budget_usd")]
    pub sub_budget_usd: Option<f64>,
    #[serde(default, alias = "swarm_depth")]
    pub swarm_depth: Option<u32>,
    #[serde(default, alias = "swarm_lineage")]
    pub swarm_lineage: Option<Vec<String>>,
    #[serde(default, alias = "external_id")]
    pub external_id: Option<String>,
    #[serde(default, alias = "safe_mode")]
    pub safe_mode: Option<bool>,
    pub analysis: Option<bool>,
    pub traceparent: Option<String>,
    #[serde(default, alias = "user_id")]
    pub user_id: Option<String>,
    #[serde(default, alias = "active_model_slot")]
    pub active_model_slot: Option<String>,
    #[serde(default, alias = "context_files")]
    pub context_files: Option<Vec<String>>,
    #[serde(default, alias = "recent_findings")]
    pub recent_findings: Option<String>,
    #[serde(default, alias = "structured_output")]
    pub structured_output: Option<bool>,
    #[serde(default, alias = "primaryGoal", alias = "primary_goal")]
    pub primary_goal: Option<String>,
    #[serde(default, alias = "allowedFiles", alias = "allowed_files")]
    pub allowed_files: Option<Vec<String>>,
    #[serde(default, alias = "visibleTranscript", alias = "visible_transcript")]
    pub visible_transcript: Option<Vec<String>>,
    #[serde(default, alias = "skipSocraticGate", alias = "skip_socratic_gate")]
    pub skip_socratic_gate: Option<bool>,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mission_status_serialization_and_deserialization() {
        assert_eq!(
            serde_json::to_string(&MissionStatus::SpecReview).unwrap(),
            "\"spec_review\""
        );
        assert_eq!(
            serde_json::to_string(&MissionStatus::Pending).unwrap(),
            "\"pending\""
        );
        assert_eq!(
            serde_json::to_string(&MissionStatus::Active).unwrap(),
            "\"active\""
        );
        assert_eq!(
            serde_json::to_string(&MissionStatus::Completed).unwrap(),
            "\"completed\""
        );
        assert_eq!(
            serde_json::to_string(&MissionStatus::Failed).unwrap(),
            "\"failed\""
        );
        assert_eq!(
            serde_json::to_string(&MissionStatus::Paused).unwrap(),
            "\"paused\""
        );

        // Test deserialization with snake_case and aliases
        assert_eq!(
            serde_json::from_str::<MissionStatus>("\"spec_review\"").unwrap(),
            MissionStatus::SpecReview
        );
        assert_eq!(
            serde_json::from_str::<MissionStatus>("\"specreview\"").unwrap(),
            MissionStatus::SpecReview
        );
        assert_eq!(
            serde_json::from_str::<MissionStatus>("\"spec-review\"").unwrap(),
            MissionStatus::SpecReview
        );
        assert_eq!(
            serde_json::from_str::<MissionStatus>("\"pending\"").unwrap(),
            MissionStatus::Pending
        );
        assert_eq!(
            serde_json::from_str::<MissionStatus>("\"active\"").unwrap(),
            MissionStatus::Active
        );
    }
}

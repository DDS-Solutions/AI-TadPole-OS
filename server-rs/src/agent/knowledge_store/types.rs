//! @docs ARCHITECTURE:Registry
//!
//! ### AI Assist Note
//! **A single entry in the Institutional Knowledge Store.**
//! This module implements high-fidelity logic for the Sovereign Reality layer.
//!
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: Runtime logic error, state desynchronization, or resource exhaustion.
//! - **Telemetry Link**: Search `[types]` in tracing logs.

/// A single entry in the Institutional Knowledge Store.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct KnowledgeEntry {
    pub id: String,
    pub text: String,
    pub topic: String,
    pub cluster_id: Option<String>,
    pub source_node_id: Option<String>,
    pub source_agent_id: Option<String>,
    /// SHA-256 hex of `text` — used for dedup and P2P idempotency.
    pub content_hash: String,
    /// 0.0–1.0 quality signal; decays 0.01/day for unconfirmed entries.
    pub confidence: f32,
    /// True if a human explicitly approved this entry via /confirm.
    pub human_confirmed: bool,
    /// Unix expiry timestamp; NULL = never expires (human-confirmed entries).
    pub ttl: Option<i64>,
    pub created_at: i64,
    pub access_count: i64,
    // --- OKF Extensions ---
    pub concept_type: String,
    pub title: Option<String>,
    pub description: Option<String>,
    pub resource_uri: Option<String>,
    pub tags: Option<String>,
    // --- Lean OKF (v0.1.5) Extensions ---
    pub security_tier: String,
    pub parent_id: Option<String>,
}

/// Parameters for writing a new knowledge entry.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct AddKnowledgeRequest {
    pub text: String,
    pub topic: String,
    pub cluster_id: Option<String>,
    /// The remote Bunker node that authored this entry (P2P sync). None = local write.
    pub source_node_id: Option<String>,
    pub source_agent_id: Option<String>,
    /// 0.0–1.0. Defaults to 1.0.
    pub confidence: Option<f32>,
    /// Days until expiry. Omit for the system default (90d for agents).
    /// Pass None explicitly in JSON to create a permanent entry.
    pub ttl_days: Option<i64>,
    /// If true, entry is immediately human-confirmed (ttl cleared, confidence = 1.0).
    pub human_confirmed: Option<bool>,
    // --- OKF Extensions ---
    pub concept_type: Option<String>,
    pub title: Option<String>,
    pub description: Option<String>,
    pub resource_uri: Option<String>,
    pub tags: Option<String>,
    // --- Lean OKF (v0.1.5) Extensions ---
    pub security_tier: Option<String>,
    pub parent_id: Option<String>,
}

/// Search parameters for semantic retrieval.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct KnowledgeSearchRequest {
    pub query: String,
    /// Pre-filter by topic before vector search.
    pub topic: Option<String>,
    /// NULL = search global + cluster-scoped entries; "global" = global only.
    pub cluster_id: Option<String>,
    /// Max results to return. Default: 10.
    pub limit: Option<usize>,
    /// Minimum confidence threshold. Default: 0.3.
    pub min_confidence: Option<f32>,
    // --- OKF Extensions ---
    pub concept_type: Option<String>,
    pub security_tier: Option<String>,
}

/// Default TTL for agent-written entries (Q3 decision: 90 days).
pub const DEFAULT_TTL_DAYS: i64 = 90;

// Metadata: [types]

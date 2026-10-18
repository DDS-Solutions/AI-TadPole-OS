//! @docs ARCHITECTURE:IKS
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Agent Runner / types
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Type-safe state handling and bounded execution without unhandled panics.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: none declared
//! - **Witness Tests**: none declared

use serde::{Deserialize, Serialize};

/// Classification tiers for institutional knowledge.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SecurityTier {
    BronzeAdhoc,
    SilverVerified,
    GoldSovereign,
    Internal,
}

impl SecurityTier {
    pub fn as_str(&self) -> &'static str {
        match self {
            SecurityTier::BronzeAdhoc => "BRONZE_ADHOC",
            SecurityTier::SilverVerified => "SILVER_VERIFIED",
            SecurityTier::GoldSovereign => "GOLD_SOVEREIGN",
            SecurityTier::Internal => "INTERNAL",
        }
    }

    pub fn from_str_lossless(s: &str) -> Self {
        match s.trim().to_uppercase().as_str() {
            "SILVER_VERIFIED" | "SILVER" => SecurityTier::SilverVerified,
            "GOLD_SOVEREIGN" | "GOLD" => SecurityTier::GoldSovereign,
            "INTERNAL" => SecurityTier::Internal,
            _ => SecurityTier::BronzeAdhoc,
        }
    }
}

impl Default for SecurityTier {
    fn default() -> Self {
        SecurityTier::BronzeAdhoc
    }
}

/// A single entry in the Institutional Knowledge Store.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeEntry {
    pub id: String,
    pub text: String,
    pub topic: String,
    pub cluster_id: Option<String>,
    pub source_node_id: Option<String>,
    pub source_agent_id: Option<String>,
    /// SHA-256 hex of scoped content (topic:cluster:text) — used for dedup and P2P idempotency.
    pub content_hash: String,
    /// 0.0–1.0 quality signal; decays 0.01/day for unconfirmed entries.
    pub confidence: f32,
    /// True if a human explicitly approved this entry via /knowledge/{id}/confirm.
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
#[derive(Debug, Clone, Deserialize)]
pub struct AddKnowledgeRequest {
    pub text: String,
    pub topic: String,
    pub cluster_id: Option<String>,
    /// The remote Bunker node that authored this entry (P2P sync). None = local write.
    pub source_node_id: Option<String>,
    pub source_agent_id: Option<String>,
    /// 0.0–1.0 quality score. Defaults to 0.70 for agent submissions (capped at 0.80).
    pub confidence: Option<f32>,
    /// Days until expiry (1..=3650). Omit for system default (90 days).
    /// Note: Unconfirmed entries always expire. Only human confirmation grants permanent retention.
    pub ttl_days: Option<i64>,
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
#[derive(Debug, Clone, Deserialize)]
pub struct KnowledgeSearchRequest {
    pub query: String,
    /// Pre-filter by topic before vector search.
    pub topic: Option<String>,
    /// NULL = search all entries (global + cluster); "global" = global only (cluster_id IS NULL); or specific cluster_id.
    pub cluster_id: Option<String>,
    /// Max results to return. Default: 10 (clamped to 1..=100).
    pub limit: Option<usize>,
    /// Minimum confidence threshold. Default: 0.3.
    pub min_confidence: Option<f32>,
    // --- OKF Extensions ---
    pub concept_type: Option<String>,
    pub security_tier: Option<String>,
}

/// Default TTL for unconfirmed agent-written entries (Q3 decision: 90 days).
pub const DEFAULT_TTL_DAYS: i64 = 90;
/// Default initial confidence for agent submissions.
pub const DEFAULT_AGENT_CONFIDENCE: f32 = 0.70;
/// Maximum initial confidence permitted for unconfirmed agent submissions.
pub const MAX_UNCONFIRMED_CONFIDENCE: f32 = 0.80;

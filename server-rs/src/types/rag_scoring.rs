//! @docs ARCHITECTURE:Core
//! 
//! ### AI Assist Note
//! **Core technical module for the Tadpole OS hardened engine.**
//! This module implements high-fidelity logic for the Sovereign Reality layer.
//! 
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: Runtime logic error, state desynchronization, or resource exhaustion.
//! - **Telemetry Link**: Search `[rag_scoring.rs]` in tracing logs.

//!   @docs ARCHITECTURE:Retrieval
//!
//! ### AI Assist Note
//! **RAG Scoring Types**: Data structures for the neural relevance engine.
//! Implements Multi-Factor Scoring (MFS) combining semantic distance, 
//! mission affinity, and temporal recency.

use serde::Serialize;

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, serde::Deserialize, specta::Type, Default)]
pub struct RagScore {
    pub semantic_score: f32,
    pub mission_affinity: f32,
    pub temporal_score: f32,
    pub final_score: f32,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ScoringConfig {
    pub affinity_boost: f32,
    pub recency_weight: f32,
    pub semantic_weight: f32,
}

impl Default for ScoringConfig {
    fn default() -> Self {
        Self {
            affinity_boost: 0.2,
            recency_weight: 0.1,
            semantic_weight: 0.7,
        }
    }
}

#[allow(dead_code)]
/// ### 🧬 Multi-Factor Scoring (MFS)
/// Calculates the final relevance score for a RAG hit.
pub fn calculate_mfs(
    distance: f32,
    hit_mission_id: &str,
    affinity_mission_id: Option<&str>,
    timestamp: i64,
    config: &ScoringConfig,
) -> RagScore {
    let semantic_score = 1.0 / (1.0 + distance);
    let mission_affinity = if let Some(affinity) = affinity_mission_id {
        if hit_mission_id == affinity { config.affinity_boost } else { 0.0 }
    } else {
        0.0
    };
    let now = chrono::Utc::now().timestamp();
    let age = (now - timestamp).max(0) as f32;
    let max_age = 172800.0; 
    let temporal_score = (1.0 - (age / max_age)).max(0.0) * config.recency_weight;
    let final_score = (semantic_score * config.semantic_weight) + mission_affinity + temporal_score;

    RagScore {
        semantic_score,
        mission_affinity,
        temporal_score,
        final_score,
    }
}

// Metadata: [rag_scoring]

// Metadata: [rag_scoring]

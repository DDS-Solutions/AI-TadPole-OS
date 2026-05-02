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

// ─────────────────────────────────────────────────────────
//  UNIT TESTS
// ─────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_mfs_semantic_only() {
        let config = ScoringConfig {
            semantic_weight: 1.0,
            affinity_boost: 0.0,
            recency_weight: 0.0,
        };
        // distance 0.0 -> 1.0 score
        let score = calculate_mfs(0.0, "m1", None, Utc::now().timestamp(), &config);
        assert_eq!(score.semantic_score, 1.0);
        assert_eq!(score.final_score, 1.0);

        // distance 1.0 -> 0.5 score
        let score2 = calculate_mfs(1.0, "m1", None, Utc::now().timestamp(), &config);
        assert_eq!(score2.semantic_score, 0.5);
        assert_eq!(score2.final_score, 0.5);
    }

    #[test]
    fn test_mfs_mission_affinity_boost() {
        let config = ScoringConfig {
            semantic_weight: 0.5,
            affinity_boost: 0.5,
            recency_weight: 0.0,
        };
        
        // Match
        let score = calculate_mfs(0.0, "mission-alpha", Some("mission-alpha"), Utc::now().timestamp(), &config);
        assert_eq!(score.mission_affinity, 0.5);
        assert_eq!(score.final_score, 1.0); // 0.5 (semantic) + 0.5 (affinity)

        // Mismatch
        let score2 = calculate_mfs(0.0, "mission-alpha", Some("mission-beta"), Utc::now().timestamp(), &config);
        assert_eq!(score2.mission_affinity, 0.0);
        assert_eq!(score2.final_score, 0.5);
    }

    #[test]
    fn test_mfs_temporal_decay() {
        let config = ScoringConfig {
            semantic_weight: 0.0,
            affinity_boost: 0.0,
            recency_weight: 1.0,
        };

        let now = Utc::now().timestamp();
        
        // Fresh (0s old)
        let score = calculate_mfs(0.0, "m1", None, now, &config);
        assert_eq!(score.temporal_score, 1.0);

        // Half-way (86400s / 1 day)
        let score2 = calculate_mfs(0.0, "m1", None, now - 86400, &config);
        assert_eq!(score2.temporal_score, 0.5);

        // Expired (172800s / 2 days)
        let score3 = calculate_mfs(0.0, "m1", None, now - 200000, &config);
        assert_eq!(score3.temporal_score, 0.0);
    }

    #[test]
    fn test_mfs_combined_weights() {
        let config = ScoringConfig::default(); // 0.7 semantic, 0.2 affinity, 0.1 recency
        let now = Utc::now().timestamp();

        // Perfect hit
        let score = calculate_mfs(0.0, "mission-1", Some("mission-1"), now, &config);
        // (1.0 * 0.7) + 0.2 + (1.0 * 0.1) = 1.0
        assert!((score.final_score - 1.0).abs() < 0.001);

        // Distant, mismatch, old hit
        let score2 = calculate_mfs(1.0, "mission-1", Some("mission-2"), now - 172800, &config);
        // (0.5 * 0.7) + 0.0 + (0.0 * 0.1) = 0.35
        assert!((score2.final_score - 0.35).abs() < 0.001);
    }
}

// Metadata: [rag_scoring]

// Metadata: [rag_scoring]

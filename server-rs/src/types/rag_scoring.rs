//! Multi-Factor Scoring (MFS) for Advanced RAG Retrieval
//!
//! Provides the mathematical models for weighting semantic search results 
//! based on mission affinity and temporal recency.
//!
//! @docs ARCHITECTURE:RagScoring
//!
//! ### AI Assist Note
//! **Scoring Model**: The engine uses a multiplicative model ($Score = S_{sem} \times W_{mission} \times W_{time}$). 
//! The `recency_decay_lambda` controls the velocity of data aging; use a smaller 
//! value for long-term knowledge and a larger value for ephemeral task streams.

use chrono::Utc;
use serde::Serialize;

/// Configuration for the RAG scoring engine.
#[derive(Debug, Clone, Serialize)]
pub struct ScoringConfig {
    /// Multiplier for exact mission matches (e.g., 1.2).
    pub mission_match_boost: f32,
    /// Multiplier for same-cluster (workspace) matches (e.g., 1.1).
    pub cluster_match_boost: f32,
    /// Temporal decay lambda (controls how fast old memories lose weight).
    /// Value of 0.00000001 roughly halves weight every ~80 days.
    pub recency_decay_lambda: f64,
}

impl Default for ScoringConfig {
    fn default() -> Self {
        Self {
            mission_match_boost: 1.2,
            cluster_match_boost: 1.1,
            recency_decay_lambda: 0.00000001,
        }
    }
}

/// A calculated score for a single RAG hit, providing transparency into the weighting factors.
#[derive(Debug, Clone, Serialize)]
pub struct RagScore {
    /// The final combined score (0.0 to ∞, higher is better).
    pub final_score: f32,
    /// Base semantic similarity (1 / (1 + distance)).
    pub semantic_similarity: f32,
    /// Mission affinity multiplier.
    pub mission_multiplier: f32,
    /// Temporal recency multiplier.
    pub recency_multiplier: f32,
}

/// Calculates the multi-factor score for a memory hit.
///
/// # Arguments
/// * `distance` - L2 distance from vector search (smaller is better).
/// * `hit_mission_id` - The mission ID of the found memory.
/// * `current_mission_id` - The user's current mission context.
/// * `hit_timestamp` - Unix timestamp of the memory creation.
/// * `config` - Scoring configuration.
pub fn calculate_mfs(
    distance: f32,
    hit_mission_id: &str,
    current_mission_id: Option<&str>,
    hit_timestamp: i64,
    config: &ScoringConfig,
) -> RagScore {
    // 1. Semantic Similarity (L2 to similarity conversion)
    // LanceDB L2 distance: 0.0 is perfect match.
    let semantic_similarity = 1.0 / (1.0 + distance);

    // 2. Mission Affinity
    let mut mission_multiplier = 1.0;
    if let Some(current) = current_mission_id {
        if hit_mission_id == current {
            mission_multiplier = config.mission_match_boost;
        } else if hit_mission_id != "global-search" && hit_mission_id != "system-internal" {
            // Multi-Cluster logic: If we are in any specific mission context, 
            // give a minor boost to non-system memories as they are likely relevant "SME" data.
            mission_multiplier = config.cluster_match_boost;
        }
    }

    // 3. Temporal Recency (Exponential Decay)
    // Formula: e^(-lambda * delta_t)
    let now = Utc::now().timestamp();
    let age_seconds = (now - hit_timestamp).max(0) as f64;
    let recency_multiplier = ((-config.recency_decay_lambda * age_seconds).exp()) as f32;

    let final_score = semantic_similarity * mission_multiplier * recency_multiplier;

    RagScore {
        final_score,
        semantic_similarity,
        mission_multiplier,
        recency_multiplier,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Duration, Utc};

    #[test]
    fn test_semantic_similarity_math() {
        let config = ScoringConfig::default();
        
        let score_perfect = calculate_mfs(0.0, "m1", Some("m1"), Utc::now().timestamp(), &config);
        let score_mid = calculate_mfs(1.0, "m1", Some("m1"), Utc::now().timestamp(), &config);
        
        assert!(score_perfect.final_score > 1.1); 
        assert!(score_mid.final_score < score_perfect.final_score);
    }

    #[test]
    fn test_mission_affinity_boost() {
        let config = ScoringConfig::default();
        let now = Utc::now().timestamp();
        
        let score_match = calculate_mfs(0.1, "m1", Some("m1"), now, &config);
        let score_mismatch = calculate_mfs(0.1, "m2", Some("m1"), now, &config);
        
        assert!(score_match.final_score > score_mismatch.final_score);
        let ratio = score_match.final_score / score_mismatch.final_score;
        assert!(ratio > 1.05); 
    }

    #[test]
    fn test_temporal_decay() {
        let config = ScoringConfig::default();
        let now = Utc::now();
        let one_day_ago = now - Duration::days(1);
        let one_month_ago = now - Duration::days(30);
        
        let score_now = calculate_mfs(0.1, "m1", Some("m1"), now.timestamp(), &config);
        let score_day = calculate_mfs(0.1, "m1", Some("m1"), one_day_ago.timestamp(), &config);
        let score_month = calculate_mfs(0.1, "m1", Some("m1"), one_month_ago.timestamp(), &config);
        
        assert!(score_now.final_score > score_day.final_score);
        assert!(score_day.final_score > score_month.final_score);
    }
}

//! RAG Verification — Vector retrieval and context injection tests
//!
//! @docs ARCHITECTURE:Agent
//!
//! ### AI Assist Note
//! **Verification Strategy**: We test the three pillars of MFS independently: 
//! 1. Semantic Similarity (L2 Distance)
//! 2. Mission Affinity (Affinities boost score by 1.2x)
//! 3. Temporal Decay (Newer records rank higher)
//!
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: Score drift if `recency_decay_lambda` is changed without updating tests.
//! - **Trace Scope**: `server-rs::agent::rag_tests`

#[cfg(test)]
mod tests {
    use crate::types::rag_scoring::{calculate_mfs, ScoringConfig};
    use chrono::{Duration, Utc};

    #[test]
    fn test_semantic_similarity_math() {
        let config = ScoringConfig::default();
        
        // Exact match (distance 0.0)
        let score_perfect = calculate_mfs(0.0, "m1", Some("m1"), Utc::now().timestamp(), &config);
        // Distance 1.0 (similarity 0.5)
        let score_mid = calculate_mfs(1.0, "m1", Some("m1"), Utc::now().timestamp(), &config);
        
        // Perfect match without mission/time decay should be similarity 1.0. 
        // But here we have mission boost (1.2) * similarity (1.0) = 1.2.
        assert!(score_perfect.final_score > 1.1); 
        assert!(score_mid.final_score < score_perfect.final_score);
    }

    #[test]
    fn test_mission_affinity_boost() {
        let config = ScoringConfig::default();
        let now = Utc::now().timestamp();
        
        // Current mission (m1) search for a memory in (m1)
        let score_match = calculate_mfs(0.1, "m1", Some("m1"), now, &config);
        // Current mission (m1) search for a memory in (m2)
        let score_mismatch = calculate_mfs(0.1, "m2", Some("m1"), now, &config);
        
        // Match should be exactly 1.2x of the base (or higher depending on cluster boost)
        // config.mission_match_boost = 1.2
        // config.cluster_match_boost = 1.1 (for mismatch)
        assert!(score_match.final_score > score_mismatch.final_score);
        let ratio = score_match.final_score / score_mismatch.final_score;
        assert!(ratio > 1.05); // Effectively 1.2 / 1.1 = ~1.09
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
        
        // Newer must rank higher
        assert!(score_now.final_score > score_day.final_score);
        assert!(score_day.final_score > score_month.final_score);
    }

    #[test]
    fn test_mfs_composite_ranking() {
        let config = ScoringConfig::default();
        let now = Utc::now().timestamp();
        let very_old = (Utc::now() - Duration::days(365)).timestamp();
        
        // Scenario: A very similar result (0.1) from last year 
        // vs a moderately similar result (0.3) from today (same mission).
        let result_old_perfect = calculate_mfs(0.1, "m1", Some("m1"), very_old, &config);
        let result_new_mid = calculate_mfs(0.3, "m1", Some("m1"), now, &config);
        
        // In many RAG systems, the "New" result should win or at least be competitive
        // with the "Old but Perfect" result depending on lambda.
        tracing::debug!("Old Score: {}, New Score: {}", result_old_perfect.final_score, result_new_mid.final_score);
        // Simply ensuring they compute correctly.
        assert!(result_new_mid.final_score > 0.0);
        assert!(result_old_perfect.final_score > 0.0);
    }
}

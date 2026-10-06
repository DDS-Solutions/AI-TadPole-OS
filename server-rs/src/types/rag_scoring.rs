//! @docs ARCHITECTURE:RAGSystems
//!
//! ### AI Context Alignment
//! - **Subsystem**: Data Systems / rag_scoring
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Type-safe state handling and bounded execution without unhandled panics.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: none declared
//! - **Witness Tests**: none declared

use serde::Serialize;

#[derive(Debug, Clone, Serialize, serde::Deserialize, specta::Type, Default)]
pub struct RagScore {
    pub semantic_score: f32,
    pub mission_affinity: f32,
    pub temporal_score: f32,
    pub final_score: f32,
}

#[derive(Debug, Clone)]
pub struct ScoringConfig {
    pub affinity_boost: f32,
    pub recency_weight: f32,
    pub semantic_weight: f32,
    pub max_age_secs: f32,
}

impl Default for ScoringConfig {
    fn default() -> Self {
        Self {
            affinity_boost: 0.2,
            recency_weight: 0.1,
            semantic_weight: 0.7,
            max_age_secs: 172_800.0, // 48 hours
        }
    }
}

/// ### 🧬 Multi-Factor Scoring (MFS)
/// Calculates the final relevance score for a RAG hit at the current time.
#[allow(dead_code)]
pub fn calculate_mfs(
    distance: f32,
    hit_mission_id: &str,
    affinity_mission_id: Option<&str>,
    timestamp: i64,
    config: &ScoringConfig,
) -> RagScore {
    calculate_mfs_at(
        distance,
        hit_mission_id,
        affinity_mission_id,
        timestamp,
        config,
        chrono::Utc::now().timestamp(),
    )
}

/// Pure deterministic Multi-Factor Scoring with explicit `now` timestamp.
pub fn calculate_mfs_at(
    distance: f32,
    hit_mission_id: &str,
    affinity_mission_id: Option<&str>,
    timestamp: i64,
    config: &ScoringConfig,
    now: i64,
) -> RagScore {
    // Clamp distance to >= 0.0 to prevent division by zero or inf/NaN propagation
    let safe_distance = distance.max(0.0);
    let semantic_score = 1.0 / (1.0 + safe_distance);
    let mission_affinity = if let Some(affinity) = affinity_mission_id {
        if hit_mission_id == affinity {
            config.affinity_boost
        } else {
            0.0
        }
    } else {
        0.0
    };
    let age = (now - timestamp).max(0) as f32;
    let max_age = config.max_age_secs.max(1.0);
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

    const EPSILON: f32 = 1e-5;

    #[test]
    fn test_mfs_semantic_only() {
        let config = ScoringConfig {
            semantic_weight: 1.0,
            affinity_boost: 0.0,
            recency_weight: 0.0,
            max_age_secs: 172_800.0,
        };
        let now = 1_700_000_000;

        // distance 0.0 -> 1.0 score
        let score = calculate_mfs_at(0.0, "m1", None, now, &config, now);
        assert!((score.semantic_score - 1.0).abs() < EPSILON);
        assert!((score.final_score - 1.0).abs() < EPSILON);

        // distance 1.0 -> 0.5 score
        let score2 = calculate_mfs_at(1.0, "m1", None, now, &config, now);
        assert!((score2.semantic_score - 0.5).abs() < EPSILON);
        assert!((score2.final_score - 0.5).abs() < EPSILON);

        // Negative distance clamped -> 1.0 score without inf/NaN
        let score_neg = calculate_mfs_at(-1.0, "m1", None, now, &config, now);
        assert!((score_neg.semantic_score - 1.0).abs() < EPSILON);
        assert!(!score_neg.final_score.is_nan());
        assert!(!score_neg.final_score.is_infinite());
    }

    #[test]
    fn test_mfs_mission_affinity_boost() {
        let config = ScoringConfig {
            semantic_weight: 0.5,
            affinity_boost: 0.5,
            recency_weight: 0.0,
            max_age_secs: 172_800.0,
        };
        let now = 1_700_000_000;

        // Match
        let score = calculate_mfs_at(
            0.0,
            "mission-alpha",
            Some("mission-alpha"),
            now,
            &config,
            now,
        );
        assert!((score.mission_affinity - 0.5).abs() < EPSILON);
        assert!((score.final_score - 1.0).abs() < EPSILON); // 0.5 (semantic) + 0.5 (affinity)

        // Mismatch
        let score2 = calculate_mfs_at(
            0.0,
            "mission-alpha",
            Some("mission-beta"),
            now,
            &config,
            now,
        );
        assert!((score2.mission_affinity - 0.0).abs() < EPSILON);
        assert!((score2.final_score - 0.5).abs() < EPSILON);
    }

    #[test]
    fn test_mfs_temporal_decay() {
        let config = ScoringConfig {
            semantic_weight: 0.0,
            affinity_boost: 0.0,
            recency_weight: 1.0,
            max_age_secs: 172_800.0,
        };

        let now = 1_700_000_000;

        // Fresh (0s old)
        let score = calculate_mfs_at(0.0, "m1", None, now, &config, now);
        assert!((score.temporal_score - 1.0).abs() < EPSILON);

        // Half-way (86400s / 1 day)
        let score2 = calculate_mfs_at(0.0, "m1", None, now - 86400, &config, now);
        assert!((score2.temporal_score - 0.5).abs() < EPSILON);

        // Expired (172800s / 2 days)
        let score3 = calculate_mfs_at(0.0, "m1", None, now - 200000, &config, now);
        assert!((score3.temporal_score - 0.0).abs() < EPSILON);
    }

    #[test]
    fn test_mfs_combined_weights() {
        let config = ScoringConfig::default(); // 0.7 semantic, 0.2 affinity, 0.1 recency
        let now = 1_700_000_000;

        // Perfect hit
        let score = calculate_mfs_at(0.0, "mission-1", Some("mission-1"), now, &config, now);
        // (1.0 * 0.7) + 0.2 + (1.0 * 0.1) = 1.0
        assert!((score.final_score - 1.0).abs() < 0.001);

        // Distant, mismatch, old hit
        let score2 = calculate_mfs_at(
            1.0,
            "mission-1",
            Some("mission-2"),
            now - 172800,
            &config,
            now,
        );
        // (0.5 * 0.7) + 0.0 + (0.0 * 0.1) = 0.35
        assert!((score2.final_score - 0.35).abs() < 0.001);
    }
}

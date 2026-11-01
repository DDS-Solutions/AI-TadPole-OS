//! @docs ARCHITECTURE:RAGSystems
//!
//! ### AI Context Alignment
//! - **Subsystem**: Data Systems / data_weighting
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Type-safe state handling and bounded execution without unhandled panics.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: none declared
//! - **Witness Tests**: none declared

use crate::agent::types::MemoryEntryDetailed;
use crate::types::rag_scoring::{calculate_mfs, ScoringConfig};
use std::collections::HashMap;

pub const DEFAULT_MFS_WEIGHT: f32 = 0.7;
pub const DEFAULT_MODALITY_WEIGHT: f32 = 0.3;

pub struct DataWeighting;

impl DataWeighting {
    /// Returns the default weights for various context components.
    /// Higher values mean higher importance (more likely to be preserved).
    pub fn default_weights() -> HashMap<String, f32> {
        let mut weights = HashMap::new();
        weights.insert("identity".to_string(), 2.0);
        weights.insert("mission_goal".to_string(), 2.0);
        weights.insert("directives".to_string(), 1.8);
        weights.insert("findings".to_string(), 1.5);
        weights.insert("history".to_string(), 1.2);
        weights.insert("repo_map".to_string(), 0.8);
        weights.insert("memory".to_string(), 0.7);
        weights.insert("swarm_context".to_string(), 1.0);
        weights
    }

    /// Dynamically infers the source type of a LanceDB memory entry based on its ID.
    pub fn get_source_modality_weight(id: &str) -> f32 {
        if id.starts_with("directive_") || id.starts_with("op_") {
            1.0 // High priority: direct human/operator commands
        } else if id.starts_with("archived_") || id.starts_with("system_") {
            0.8 // Medium-high priority: system summaries and verified findings
        } else if id.starts_with("tool_") {
            0.5 // Medium-low priority: raw execution logs and command telemetry
        } else {
            0.6 // Default: agent intermediate thoughts and standard logs
        }
    }

    /// Ranks memory entries by integrating vector MFS scores with modality weights.
    pub fn rank_memories(
        mut entries: Vec<MemoryEntryDetailed>,
        affinity_mission_id: Option<&str>,
        scoring_config: &ScoringConfig,
    ) -> Vec<MemoryEntryDetailed> {
        for entry in &mut entries {
            // 1. Calculate standard Multi-Factor Score (MFS)
            let mut score = calculate_mfs(
                entry.distance,
                &entry.mission_id,
                affinity_mission_id,
                entry.timestamp,
                scoring_config,
            );

            // 2. Adjust with Modality/Source priority (clamping inputs to [0.0, 1.0])
            let mfs_clamped = score.final_score.clamp(0.0, 1.0);
            let source_weight = Self::get_source_modality_weight(&entry.id).clamp(0.0, 1.0);
            score.final_score =
                (mfs_clamped * DEFAULT_MFS_WEIGHT) + (source_weight * DEFAULT_MODALITY_WEIGHT);

            entry.score = Some(score);
        }

        // Sort descending by final score with deterministic total ordering
        entries.sort_by(|a, b| {
            let score_a = a.score.as_ref().map(|s| s.final_score).unwrap_or(0.0);
            let score_b = b.score.as_ref().map(|s| s.final_score).unwrap_or(0.0);
            score_b.total_cmp(&score_a)
        });

        entries
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::types::MemoryEntryDetailed;

    #[test]
    fn test_modality_weights() {
        assert_eq!(
            DataWeighting::get_source_modality_weight("directive_1"),
            1.0
        );
        assert_eq!(DataWeighting::get_source_modality_weight("op_1"), 1.0);
        assert_eq!(DataWeighting::get_source_modality_weight("archived_1"), 0.8);
        assert_eq!(DataWeighting::get_source_modality_weight("tool_1"), 0.5);
        assert_eq!(DataWeighting::get_source_modality_weight("agent_1"), 0.6);
    }

    #[test]
    fn test_rank_memories() {
        let scoring_config = ScoringConfig::default();
        let now = chrono::Utc::now().timestamp();

        let entries = vec![
            MemoryEntryDetailed {
                id: "tool_1".to_string(),
                text: "Raw tool output".to_string(),
                mission_id: "m1".to_string(),
                timestamp: now,
                distance: 0.1,
                score: None,
            },
            MemoryEntryDetailed {
                id: "directive_1".to_string(),
                text: "Important operator command".to_string(),
                mission_id: "m1".to_string(),
                timestamp: now,
                distance: 0.2, // Slightly worse distance than tool_1
                score: None,
            },
        ];

        let ranked = DataWeighting::rank_memories(entries, Some("m1"), &scoring_config);

        assert_eq!(ranked.len(), 2);
        // Even though directive_1 has slightly worse distance (0.2 vs 0.1),
        // its modality weight is much higher (1.0 vs 0.5), so it should rank first.
        assert_eq!(ranked[0].id, "directive_1");
        assert_eq!(ranked[1].id, "tool_1");
    }
}

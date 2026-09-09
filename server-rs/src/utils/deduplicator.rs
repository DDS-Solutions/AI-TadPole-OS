//! @docs ARCHITECTURE:Core
//!
//! ### AI Context Alignment
//! - **Subsystem**: Frontend Utilities / deduplicator
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Type-safe state handling and bounded execution without unhandled panics.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: none declared
//! - **Witness Tests**: none declared

use crate::agent::types::MemoryEntryDetailed;
use std::collections::hash_map::DefaultHasher;
use std::hash::Hasher;

/// Threshold for considering a line "long" (heuristic: avoids hashing short tokens like error codes)
const LONG_LINE_THRESHOLD: usize = 50;

/// Window size for syntactic deduplication (chosen to capture common duplicate patterns like
/// "error: ...", "warning: ...", or multi-line compiler messages)
const WINDOW_SIZE: usize = 3;

/// Unified deduplicator for active prompts and memories.
pub struct SwarmDeduplicator;

impl SwarmDeduplicator {
    /// Strips duplicate terminal commands, compiler errors, or file read outputs using rolling window hashing.
    pub fn deduplicate_syntax(text: &str) -> String {
        let lines: Vec<&str> = text.lines().collect();
        if lines.is_empty() {
            return String::new();
        }

        let n = lines.len();
        let mut skip = vec![false; n];
        let mut seen_single = std::collections::HashSet::new();
        let mut seen_window = std::collections::HashSet::new();

        // Mark long duplicate lines
        for i in 0..n {
            let line = lines[i].trim();
            if line.is_empty() {
                continue;
            }
            if line.len() > LONG_LINE_THRESHOLD {
                let mut hasher = DefaultHasher::new();
                hasher.write(line.as_bytes());
                let h = hasher.finish();
                if seen_single.contains(&h) {
                    skip[i] = true;
                } else {
                    seen_single.insert(h);
                }
            }
        }

        // Pass 1: Find which windows are duplicates
        let mut is_dup_window = vec![false; n];
        for i in 0..n.saturating_sub(WINDOW_SIZE - 1) {
            if lines[i].trim().is_empty() {
                continue;
            }

            let window = format!(
                "{}\n{}\n{}",
                lines[i].trim(),
                lines[i + 1].trim(),
                lines[i + 2].trim()
            );
            let mut hasher = DefaultHasher::new();
            hasher.write(window.as_bytes());
            let h = hasher.finish();

            if seen_window.contains(&h) {
                is_dup_window[i] = true;
            } else {
                seen_window.insert(h);
            }
        }

        // Pass 2: Determine which lines are covered by unique and duplicate windows
        let mut is_in_unique_window = vec![false; n];
        let mut is_in_dup_window = vec![false; n];

        for w in 0..n.saturating_sub(WINDOW_SIZE - 1) {
            if lines[w].trim().is_empty() {
                continue;
            }
            if is_dup_window[w] {
                is_in_dup_window[w] = true;
                is_in_dup_window[w + 1] = true;
                is_in_dup_window[w + 2] = true;
            } else {
                is_in_unique_window[w] = true;
                is_in_unique_window[w + 1] = true;
                is_in_unique_window[w + 2] = true;
            }
        }

        // Pass 3: Mark duplicate window lines that aren't part of any unique windows
        for i in 0..n {
            if is_in_dup_window[i] && !is_in_unique_window[i] {
                skip[i] = true;
            }
        }

        // Build result from non-skipped lines
        let mut result = Vec::new();
        for i in 0..n {
            if !skip[i] {
                result.push(lines[i]);
            }
        }

        result.join("\n")
    }

    /// Computes cosine similarity of RAG context vectors and filters out entries exceeding a similarity threshold,
    /// returning the deduplicated entries along with their associated embedding vectors.
    #[allow(dead_code)]
    pub fn deduplicate_semantic_with_vectors(
        entries: &[MemoryEntryDetailed],
        embeddings: &[Vec<f32>],
        threshold: f32,
    ) -> Vec<(MemoryEntryDetailed, Vec<f32>)> {
        if entries.is_empty() || embeddings.is_empty() {
            return Vec::new();
        }

        if entries.len() != embeddings.len() {
            tracing::warn!(
                "Entries count ({}) != embeddings count ({}) in semantic deduplication; aborting",
                entries.len(),
                embeddings.len()
            );
            return Vec::new();
        }

        let mut clamped_threshold = threshold;
        if !(0.0..=1.0).contains(&clamped_threshold) {
            tracing::warn!("Similarity threshold {} outside [0,1]; clamping", threshold);
            clamped_threshold = clamped_threshold.clamp(0.0, 1.0);
        }

        let mut kept: Vec<(MemoryEntryDetailed, Vec<f32>)> = Vec::new();

        for (entry, vector) in entries.iter().zip(embeddings.iter()) {
            let mut is_duplicate = false;
            for (_, existing_vec) in &kept {
                let similarity = Self::cosine_similarity(vector, existing_vec);
                if similarity >= clamped_threshold {
                    is_duplicate = true;
                    break;
                }
            }

            if !is_duplicate {
                kept.push((entry.clone(), vector.clone()));
            }
        }

        kept
    }

    /// Computes cosine similarity of RAG context vectors and filters out entries exceeding a similarity threshold.
    #[allow(dead_code)]
    pub fn deduplicate_semantic(
        entries: &[MemoryEntryDetailed],
        embeddings: &[Vec<f32>],
        threshold: f32,
    ) -> Vec<MemoryEntryDetailed> {
        Self::deduplicate_semantic_with_vectors(entries, embeddings, threshold)
            .into_iter()
            .map(|(e, _)| e)
            .collect()
    }

    /// Computes cosine similarity of two vectors of equal length.
    #[allow(dead_code)]
    pub fn cosine_similarity(v1: &[f32], v2: &[f32]) -> f32 {
        if v1.len() != v2.len() || v1.is_empty() {
            return 0.0;
        }
        let mut dot = 0.0;
        let mut norm_a = 0.0;
        let mut norm_b = 0.0;
        for (a, b) in v1.iter().zip(v2.iter()) {
            dot += a * b;
            norm_a += a * a;
            norm_b += b * b;
        }
        if norm_a == 0.0 || norm_b == 0.0 {
            return 0.0;
        }
        dot / (norm_a.sqrt() * norm_b.sqrt())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_overlapping_windows() {
        let text = "A\nB\nA\nB\nA\nB";
        // After deduping [A,B,A] windows, should keep first occurrence: "A\nB\nA\nB"
        assert_eq!(SwarmDeduplicator::deduplicate_syntax(text), "A\nB\nA\nB");
    }

    #[test]
    fn test_semantic_thresholds() {
        let entry1 = MemoryEntryDetailed {
            id: "1".to_string(),
            text: "hello world".to_string(),
            mission_id: "m1".to_string(),
            timestamp: 123456789,
            distance: 0.0,
            score: None,
        };
        let entry2 = MemoryEntryDetailed {
            id: "2".to_string(),
            text: "hello world duplicate".to_string(),
            mission_id: "m1".to_string(),
            timestamp: 123456790,
            distance: 0.0,
            score: None,
        };
        let entries = vec![entry1, entry2];
        let embeddings = vec![vec![1.0, 0.0], vec![1.0, 0.0]]; // identical vectors

        // threshold=1.0 deduplicates identical vectors (similarity 1.0 >= 1.0)
        let dedup_1 = SwarmDeduplicator::deduplicate_semantic(&entries, &embeddings, 1.0);
        assert_eq!(dedup_1.len(), 1);

        // threshold=0.9 also dedupes because 1.0 >= 0.9 is true.
        let dedup_09 = SwarmDeduplicator::deduplicate_semantic(&entries, &embeddings, 0.9);
        assert_eq!(dedup_09.len(), 1);

        // Mismatched length returns empty
        let mismatched = SwarmDeduplicator::deduplicate_semantic(&entries, &embeddings[..1], 0.9);
        assert_eq!(mismatched.len(), 0);
    }

    #[test]
    fn test_long_lines_syntactic() {
        let long_line_1 =
            "This is a very long line that exceeds fifty characters of length for testing.";
        let long_line_2 =
            "This is another very long line that also exceeds fifty characters of length.";
        let text = format!(
            "{}\n{}\n{}\n{}",
            long_line_1, long_line_2, long_line_1, long_line_2
        );
        let expected = format!("{}\n{}", long_line_1, long_line_2);
        assert_eq!(SwarmDeduplicator::deduplicate_syntax(&text), expected);
    }
}

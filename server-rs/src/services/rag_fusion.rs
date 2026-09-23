//! @docs ARCHITECTURE:Services:RAG
//!
//! ### AI Assist Note
//! **Reciprocal Rank Fusion (RRF) RAG Engine**: Unifies the Hybrid RAG Triad
//! (LanceDB Vector + BM25 Lexical + TrustGraph Entity/GraphRAG) into a single,
//! highly calibrated ranking. Eliminates prompt context dilution and boosts
//! search precision.
//!
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: Score tie resolution, empty candidate lists, or identifier collisions.
//! - **Telemetry Link**: Search `[rag_fusion]` in tracing logs.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use tracing::{debug, warn};

/// A retrieval candidate from an individual search engine.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RagCandidate {
    pub id: String,
    pub title: String,
    pub content: String,
    pub relative_path: Option<String>,
    pub source: String,
    pub metadata: Option<serde_json::Value>,
}

/// Configuration weights for the Hybrid RAG Triad.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RagEngineWeights {
    pub vector_weight: f32,
    pub bm25_weight: f32,
    pub trustgraph_weight: f32,
    pub k_constant: f32,
}

impl Default for RagEngineWeights {
    fn default() -> Self {
        Self {
            vector_weight: 0.40,
            bm25_weight: 0.35,
            trustgraph_weight: 0.25,
            k_constant: 60.0,
        }
    }
}

/// A unified, deduplicated and score-calibrated search result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FusedSearchResult {
    pub id: String,
    pub title: String,
    pub content: String,
    pub relative_path: Option<String>,
    pub sources: Vec<String>,
    pub rrf_score: f32,
    pub metadata: Option<serde_json::Value>,
}

struct MergedCandidateAccumulator {
    id: String,
    title: String,
    content: String,
    relative_path: Option<String>,
    sources: Vec<String>,
    total_rrf_score: f32,
    metadata: Option<serde_json::Value>,
}

/// Fuses ranked results from Vector, BM25, and TrustGraph engines using Reciprocal Rank Fusion.
///
/// # Contract: Each slice must be ordered best-first (descending relevance).
/// Formula:
/// $$RRF(d) = \sum_{e \in \{Vector, BM25, Graph\}} \frac{w_e}{k + rank_e(d)}$$
pub fn fuse_search_results(
    vector_results: &[RagCandidate],
    bm25_results: &[RagCandidate],
    graph_results: &[RagCandidate],
    weights: &RagEngineWeights,
    top_k: usize,
) -> Vec<FusedSearchResult> {
    debug!(
        target: "rag_fusion",
        vector_count = vector_results.len(),
        bm25_count = bm25_results.len(),
        graph_count = graph_results.len(),
        top_k = top_k,
        "Fusing hybrid RAG candidate streams"
    );

    let mut accumulator_map: HashMap<String, MergedCandidateAccumulator> = HashMap::new();

    let k = if weights.k_constant.is_finite() && weights.k_constant > 0.0 {
        weights.k_constant
    } else {
        warn!(target: "rag_fusion", "[rag_fusion] invalid k_constant ({}), clamping to default 60.0", weights.k_constant);
        60.0
    };

    let sanitize_weight = |name: &str, w: f32| -> f32 {
        if w.is_finite() && w >= 0.0 {
            w
        } else {
            warn!(target: "rag_fusion", "[rag_fusion] invalid {} weight ({}), clamping to 0.0", name, w);
            0.0
        }
    };

    let w_vec = sanitize_weight("vector", weights.vector_weight);
    let w_bm25 = sanitize_weight("bm25", weights.bm25_weight);
    let w_graph = sanitize_weight("trustgraph", weights.trustgraph_weight);

    let engines: [(&[RagCandidate], f32, &str); 3] = [
        (vector_results, w_vec, "vector"),
        (bm25_results, w_bm25, "bm25"),
        (graph_results, w_graph, "trustgraph"),
    ];

    for (candidate_list, engine_weight, default_engine_name) in engines {
        let mut seen_in_engine = HashSet::new();
        for (index, candidate) in candidate_list.iter().enumerate() {
            // Keep only best (highest) rank per engine per document
            if !seen_in_engine.insert(candidate.id.as_str()) {
                continue;
            }

            let rank = (index + 1) as f32; // 1-indexed rank
            let rrf_component = engine_weight / (k + rank);
            let engine_name = default_engine_name.to_string();

            accumulator_map
                .entry(candidate.id.clone())
                .and_modify(|acc| {
                    acc.total_rrf_score += rrf_component;
                    if !acc.sources.contains(&engine_name) {
                        acc.sources.push(engine_name.clone());
                    }
                    if acc.content.is_empty() && !candidate.content.is_empty() {
                        acc.content = candidate.content.clone();
                    }
                    if acc.title.is_empty() && !candidate.title.is_empty() {
                        acc.title = candidate.title.clone();
                    }
                    if acc.relative_path.is_none() && candidate.relative_path.is_some() {
                        acc.relative_path = candidate.relative_path.clone();
                    }
                })
                .or_insert_with(|| MergedCandidateAccumulator {
                    id: candidate.id.clone(),
                    title: candidate.title.clone(),
                    content: candidate.content.clone(),
                    relative_path: candidate.relative_path.clone(),
                    sources: vec![engine_name],
                    total_rrf_score: rrf_component,
                    metadata: candidate.metadata.clone(),
                });
        }
    }

    let mut fused: Vec<FusedSearchResult> = accumulator_map
        .into_values()
        .map(|acc| FusedSearchResult {
            id: acc.id,
            title: acc.title,
            content: acc.content,
            relative_path: acc.relative_path,
            sources: acc.sources,
            rrf_score: acc.total_rrf_score,
            metadata: acc.metadata,
        })
        .collect();

    // Deterministic tie-breaking:
    // 1. Highest RRF score first (total_cmp)
    // 2. Broader engine agreement / consensus wins ties
    // 3. Alphabetical ID as canonical final tie-break
    fused.sort_by(|a, b| {
        b.rrf_score
            .total_cmp(&a.rrf_score)
            .then_with(|| b.sources.len().cmp(&a.sources.len()))
            .then_with(|| a.id.cmp(&b.id))
    });

    fused.truncate(top_k);

    fused
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rrf_deduplication_boosts_intersecting_items() {
        let weights = RagEngineWeights::default();

        let item_a = RagCandidate {
            id: "doc_a".to_string(),
            title: "Kernel Architecture".to_string(),
            content: "Rust Axum engine details".to_string(),
            relative_path: Some("docs/ARCHITECTURE.md".to_string()),
            source: "vector".to_string(),
            metadata: None,
        };

        let item_b = RagCandidate {
            id: "doc_b".to_string(),
            title: "Actor Supervision".to_string(),
            content: "OTP supervisor patterns".to_string(),
            relative_path: Some("docs/ACTORS.md".to_string()),
            source: "bm25".to_string(),
            metadata: None,
        };

        let item_a_bm25 = RagCandidate {
            id: "doc_a".to_string(),
            title: "Kernel Architecture".to_string(),
            content: "Rust Axum engine details".to_string(),
            relative_path: Some("docs/ARCHITECTURE.md".to_string()),
            source: "bm25".to_string(),
            metadata: None,
        };

        let vector_list = vec![item_a.clone()];
        // In BM25, item_b is rank 1, item_a is rank 2
        let bm25_list = vec![item_b.clone(), item_a_bm25];
        let graph_list = vec![];

        let results = fuse_search_results(&vector_list, &bm25_list, &graph_list, &weights, 10);

        assert_eq!(results.len(), 2);
        // doc_a appeared in BOTH Vector (#1) and BM25 (#2), so its combined score should beat doc_b (only BM25 #1)
        assert_eq!(results[0].id, "doc_a");
        assert_eq!(results[1].id, "doc_b");
        assert_eq!(results[0].sources.len(), 2);
        assert!(results[0].sources.contains(&"vector".to_string()));
        assert!(results[0].sources.contains(&"bm25".to_string()));
    }

    #[test]
    fn test_rrf_top_k_truncation() {
        let weights = RagEngineWeights::default();
        let mut candidates = Vec::new();
        for i in 1..=10 {
            candidates.push(RagCandidate {
                id: format!("doc_{}", i),
                title: format!("Doc {}", i),
                content: format!("Content {}", i),
                relative_path: None,
                source: "bm25".to_string(),
                metadata: None,
            });
        }

        let results = fuse_search_results(&[], &candidates, &[], &weights, 3);
        assert_eq!(results.len(), 3);
        assert_eq!(results[0].id, "doc_1");
        assert_eq!(results[1].id, "doc_2");
        assert_eq!(results[2].id, "doc_3");
    }

    #[test]
    fn test_rrf_triad_three_way_fusion_precedence() {
        let weights = RagEngineWeights::default();

        let triad_item = RagCandidate {
            id: "triad_doc".to_string(),
            title: "Triad Architecture".to_string(),
            content: "Complete 3-way match".to_string(),
            relative_path: Some("docs/TRIAD.md".to_string()),
            source: "".to_string(),
            metadata: Some(serde_json::json!({"triad": true})),
        };

        let vector_only = RagCandidate {
            id: "vector_doc".to_string(),
            title: "Vector Only".to_string(),
            content: "Vector match only".to_string(),
            relative_path: None,
            source: "".to_string(),
            metadata: None,
        };

        let bm25_only = RagCandidate {
            id: "bm25_doc".to_string(),
            title: "BM25 Only".to_string(),
            content: "BM25 match only".to_string(),
            relative_path: None,
            source: "".to_string(),
            metadata: None,
        };

        let vector_list = vec![vector_only, triad_item.clone()];
        let bm25_list = vec![bm25_only, triad_item.clone()];
        let graph_list = vec![triad_item];

        let results = fuse_search_results(&vector_list, &bm25_list, &graph_list, &weights, 5);

        assert_eq!(results.len(), 3);
        // triad_doc appears in all 3 engines, scoring highest
        assert_eq!(results[0].id, "triad_doc");
        assert_eq!(results[0].sources.len(), 3);
        assert!(results[0].sources.contains(&"vector".to_string()));
        assert!(results[0].sources.contains(&"bm25".to_string()));
        assert!(results[0].sources.contains(&"trustgraph".to_string()));
        assert_eq!(
            results[0].metadata,
            Some(serde_json::json!({"triad": true}))
        );
    }

    #[test]
    fn test_rrf_empty_inputs_returns_empty() {
        let weights = RagEngineWeights::default();
        let results = fuse_search_results(&[], &[], &[], &weights, 10);
        assert!(
            results.is_empty(),
            "Empty input lists must return empty results"
        );
    }

    #[test]
    fn test_rrf_custom_weights_influences_ranking() {
        let lexical_heavy = RagEngineWeights {
            vector_weight: 0.10,
            bm25_weight: 0.80,
            trustgraph_weight: 0.10,
            k_constant: 60.0,
        };

        let vec_doc = RagCandidate {
            id: "vec_doc".to_string(),
            title: "Vector Doc".to_string(),
            content: "Vec".to_string(),
            relative_path: None,
            source: "vector".to_string(),
            metadata: None,
        };

        let bm25_doc = RagCandidate {
            id: "bm25_doc".to_string(),
            title: "BM25 Doc".to_string(),
            content: "BM25".to_string(),
            relative_path: None,
            source: "bm25".to_string(),
            metadata: None,
        };

        let results = fuse_search_results(&[vec_doc], &[bm25_doc], &[], &lexical_heavy, 5);
        assert_eq!(results.len(), 2);
        // With 80% BM25 weight vs 10% Vector weight, bm25_doc must be ranked first
        assert_eq!(results[0].id, "bm25_doc");
        assert_eq!(results[1].id, "vec_doc");
    }

    #[test]
    fn test_rrf_adversarial_nan_and_negative_k_failsafe() {
        let adversarial_weights = RagEngineWeights {
            vector_weight: f32::NAN,
            bm25_weight: -10.0,
            trustgraph_weight: 1.0,
            k_constant: -100.0, // negative or invalid k
        };

        let doc = RagCandidate {
            id: "safe_doc".to_string(),
            title: "Safe Title".to_string(),
            content: "Content".to_string(),
            relative_path: None,
            source: "trustgraph".to_string(),
            metadata: None,
        };

        // Must not panic, must not return NaN scores
        let results = fuse_search_results(&[], &[], &[doc], &adversarial_weights, 5);
        assert_eq!(results.len(), 1);
        assert!(results[0].rrf_score.is_finite());
        assert!(results[0].rrf_score > 0.0);
    }

    #[test]
    fn test_rrf_duplicate_id_in_single_engine_ignored() {
        let weights = RagEngineWeights::default();

        let dup1 = RagCandidate {
            id: "dup_doc".to_string(),
            title: "First Rank".to_string(),
            content: "Primary".to_string(),
            relative_path: None,
            source: "".to_string(),
            metadata: None,
        };
        let dup2 = RagCandidate {
            id: "dup_doc".to_string(),
            title: "Second Rank Glitch".to_string(),
            content: "Glitch".to_string(),
            relative_path: None,
            source: "".to_string(),
            metadata: None,
        };

        // If duplicate was counted twice, score would be w/(k+1) + w/(k+2)
        // With single-engine deduplication, score must be exactly w/(k+1)
        let results = fuse_search_results(&[dup1, dup2], &[], &[], &weights, 5);
        assert_eq!(results.len(), 1);
        let expected_score = weights.vector_weight / (weights.k_constant + 1.0);
        assert!((results[0].rrf_score - expected_score).abs() < 1e-6);
    }

    #[test]
    fn test_rrf_deterministic_tie_breaking() {
        // Equal weights between vector and bm25
        let tie_weights = RagEngineWeights {
            vector_weight: 0.5,
            bm25_weight: 0.5,
            trustgraph_weight: 0.0,
            k_constant: 60.0,
        };

        // doc_z is rank 1 in vector (0.5/61), doc_w is rank 1 in bm25 (0.5/61) -> exact score tie!
        let doc_z = RagCandidate {
            id: "doc_z".to_string(),
            title: "Z".to_string(),
            content: "".to_string(),
            relative_path: None,
            source: "".to_string(),
            metadata: None,
        };
        let doc_w = RagCandidate {
            id: "doc_w".to_string(),
            title: "W".to_string(),
            content: "".to_string(),
            relative_path: None,
            source: "".to_string(),
            metadata: None,
        };

        let tied_results = fuse_search_results(&[doc_z], &[doc_w], &[], &tie_weights, 5);
        assert_eq!(tied_results.len(), 2);
        assert_eq!(tied_results[0].rrf_score, tied_results[1].rrf_score);
        // Alphabetical tie-break: doc_w beats doc_z
        assert_eq!(tied_results[0].id, "doc_w");
        assert_eq!(tied_results[1].id, "doc_z");
    }

    #[test]
    fn test_rrf_top_k_zero() {
        let weights = RagEngineWeights::default();
        let doc = RagCandidate {
            id: "doc".to_string(),
            title: "Doc".to_string(),
            content: "".to_string(),
            relative_path: None,
            source: "".to_string(),
            metadata: None,
        };
        let results = fuse_search_results(&[doc], &[], &[], &weights, 0);
        assert!(results.is_empty());
    }
}

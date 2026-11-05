//! @docs ARCHITECTURE:Runner
//!
//! ### AI Assist Note
//! - **Subsystem**: Agent context / history and memory enrichment
//! - Preserve ordered preparation, workspace/privacy scope and shared usage accounting.
//!
//! ### 🔍 Debugging & Observability
//! - Failures retain their existing AppError or nonfatal tracing behavior.

use super::super::{AgentRunner, RunContext};
#[cfg(feature = "vector-memory")]
use super::{MAX_SCOPE_ENTRIES, SEMANTIC_DEDUP_SIMILARITY};
use super::{MISSION_LOG_RETRIEVAL_LIMIT, TOKEN_SUMMARIZATION_THRESHOLD};
use crate::agent::context_manager::ContextManager;
use crate::agent::sanitizer::Sanitizer;

#[derive(Default)]
pub(super) struct MemoryAugmentation {
    pub summarized_history: Option<String>,
    pub graph_context: Option<String>,
}

impl MemoryAugmentation {
    pub(super) fn apply(self, ctx: &mut RunContext) {
        if let Some(summary) = self.summarized_history {
            ctx.execution_metrics.record_summarization();
            ctx.summarized_history = Some(summary);
        }
        if let Some(graph) = self.graph_context {
            ctx.graph_context = Some(graph);
        }
    }
}

impl AgentRunner {
    pub(super) async fn collect_mission_summary(
        &self,
        ctx: &RunContext,
        mission_id: &str,
    ) -> MemoryAugmentation {
        let mut augmentation = MemoryAugmentation::default();
        let logs = match crate::agent::mission::get_recent_mission_logs(
            &self.state.resources.pool,
            mission_id,
            MISSION_LOG_RETRIEVAL_LIMIT,
        )
        .await
        {
            Ok(logs) => logs,
            Err(e) => {
                tracing::warn!(
                    "⚠️ [Runner] Failed to fetch mission logs for summarization: {}",
                    e
                );
                return augmentation;
            }
        };

        let history_text: String = logs
            .iter()
            .filter(|l| l.severity != "debug")
            .map(|l| format!("[{}]: {}", l.source, l.text))
            .collect::<Vec<_>>()
            .join("\n");
        let history_text = Sanitizer::sanitize_for_prompt(&history_text);

        if ContextManager::calculate_tokens(&history_text) > TOKEN_SUMMARIZATION_THRESHOLD {
            tracing::info!(
                "💡 [Runner] Mission history exceeds 4k tokens for {}. Summarizing...",
                mission_id
            );
            match ContextManager::summarize_history(self, ctx, &history_text).await {
                Ok(summary) => {
                    augmentation.summarized_history =
                        Some(Sanitizer::sanitize_for_prompt(&summary));
                    tracing::info!("✅ [Runner] Context summarized for {}.", mission_id);
                }
                Err(e) => {
                    // SEC-FAILSAFE: Summarization failure should not block the mission.
                    // We log a warning and continue, accepting potential token cutoff.
                    tracing::warn!(
                        "⚠️ [Runner] Context summarization failed (continuing without summary): {}",
                        e
                    );
                }
            }
        }
        augmentation
    }

    /// Synchronizes remote memory via hybrid search, TrustGraph BFS traversal, and multi-factor reranking.
    pub(super) async fn collect_rag_memory(
        &self,
        ctx: &RunContext,
        initial_prompt: &str,
    ) -> MemoryAugmentation {
        #[allow(unused_mut)]
        let mut augmentation = MemoryAugmentation::default();
        #[cfg(not(feature = "vector-memory"))]
        let _ = (ctx, initial_prompt);

        #[cfg(feature = "vector-memory")]
        {
            let (_, agent_memory_dir, mission_scope_dir) = ctx.resolve_paths();
            let agent_mem =
                match crate::agent::memory::VectorMemory::connect(&agent_memory_dir, "memories")
                    .await
                {
                    Ok(m) => m,
                    Err(e) => {
                        tracing::warn!("⚠️ [RAG] Failed to connect to agent VectorMemory: {}", e);
                        return augmentation;
                    }
                };
            let mission_mem = match crate::agent::memory::VectorMemory::connect(
                &mission_scope_dir,
                "scope",
            )
            .await
            {
                Ok(m) => m,
                Err(e) => {
                    tracing::warn!(
                        "⚠️ [RAG] Failed to connect to mission VectorMemory scope: {}",
                        e
                    );
                    return augmentation;
                }
            };

            let client = (*self.state.resources.http_client).clone();
            let provider = self.resolve_provider(ctx, client).await;

            let sanitized_prompt = Sanitizer::sanitize_for_prompt(initial_prompt);
            let vec = match provider.embed(&sanitized_prompt).await {
                Ok(v) => v,
                Err(e) => {
                    tracing::error!(
                        "🚨 [RAG] Failed to generate initial prompt embedding: {}",
                        e
                    );
                    return augmentation;
                }
            };

            // 1. High-Fidelity Vector Retrieval
            let raw_entries = match agent_mem.search_knowledge_full(vec.clone(), 15).await {
                Ok(entries) => entries,
                Err(e) => {
                    tracing::error!("🚨 [RAG] Failed to retrieve vector memories: {}", e);
                    return augmentation;
                }
            };

            // 2. Multi-Factor Reranking (modality source-aware)
            let scoring_config = crate::types::rag_scoring::ScoringConfig::default();
            let ranked_entries = crate::utils::data_weighting::DataWeighting::rank_memories(
                raw_entries,
                Some(&ctx.mission_state.mission_id),
                &scoring_config,
            );

            // 3. Extract BFS adjacent context from SQLite TrustGraph
            let seed_ids: Vec<String> = ranked_entries
                .iter()
                .take(5)
                .map(|e| e.id.clone())
                .collect();
            let graph_engine =
                crate::agent::trustgraph::TrustGraphEngine::new(self.state.resources.pool.clone());

            let graph_context = match graph_engine.traverse_subgraph(&seed_ids, 2).await {
                Ok(subgraph) => {
                    let mut ctx_str = String::new();
                    if !subgraph.nodes.is_empty() {
                        ctx_str.push_str("#### Semantic Nodes:\n");
                        for node in &subgraph.nodes {
                            ctx_str.push_str(&format!(
                                "- Entity [{}] ({}) - Name: {}, Description: {}\n",
                                node.id,
                                node.r#type,
                                node.name,
                                node.description
                                    .as_deref()
                                    .unwrap_or("No details available")
                            ));
                        }
                    }
                    if !subgraph.relations.is_empty() {
                        ctx_str.push_str("\n#### Semantic Relations:\n");
                        for rel in &subgraph.relations {
                            ctx_str.push_str(&format!(
                                "- Node [{}] --({})--> Node [{}] (Weight: {})\n",
                                rel.source_entity_id,
                                rel.relation_type,
                                rel.target_entity_id,
                                rel.weight
                            ));
                        }
                    }
                    Sanitizer::sanitize_for_prompt(&ctx_str)
                }
                Err(e) => {
                    tracing::error!("🚨 [RAG] TrustGraph traversal failed: {}", e);
                    String::new()
                }
            };

            if !graph_context.is_empty() {
                augmentation.graph_context = Some(graph_context);
            }

            // 4. Synthesize context and inject into active mission scope vector memory
            let top_results: Vec<_> = ranked_entries
                .into_iter()
                .take(5)
                .map(|mut entry| {
                    entry.text = Sanitizer::sanitize_for_prompt(&entry.text);
                    entry
                })
                .collect();
            let embed_futures = top_results.iter().map(|entry| provider.embed(&entry.text));
            let results = futures::future::join_all(embed_futures).await;

            let mut embeddings = Vec::new();
            let mut final_results = Vec::new();

            for (entry, res) in top_results.into_iter().zip(results) {
                match res {
                    Ok(v) => {
                        embeddings.push(v);
                        final_results.push(entry);
                    }
                    Err(e) => {
                        tracing::warn!(
                            "⚠️ [RAG] Failed to generate embedding for entry: {}, skipping to avoid semantic pollution",
                            e
                        );
                    }
                }
            }
            let deduplicated_results =
                crate::utils::deduplicator::SwarmDeduplicator::deduplicate_semantic_with_vectors(
                    &final_results,
                    &embeddings,
                    SEMANTIC_DEDUP_SIMILARITY,
                );
            let top_results_len = deduplicated_results.len();

            // Sliding Window Eviction
            if let Ok(existing) = mission_mem
                .get_all_memories(&ctx.mission_state.mission_id)
                .await
            {
                if existing.len() + top_results_len > MAX_SCOPE_ENTRIES {
                    let to_delete_count =
                        (existing.len() + top_results_len).saturating_sub(MAX_SCOPE_ENTRIES);
                    let to_delete_ids: Vec<String> = existing
                        .iter()
                        .take(to_delete_count)
                        .map(|(id, _)| id.clone())
                        .collect();
                    if !to_delete_ids.is_empty() {
                        let _ = mission_mem.delete_memories(to_delete_ids).await;
                    }
                }
            }

            let mut count = 0;
            for (entry, entry_vec) in deduplicated_results {
                let final_context = Sanitizer::sanitize_for_prompt(&entry.text);
                let unique_id = format!(
                    "mem-{}-{}",
                    ctx.mission_state.mission_id,
                    uuid::Uuid::new_v4()
                );
                if mission_mem
                    .add_memory(
                        &unique_id,
                        &final_context,
                        &ctx.mission_state.mission_id,
                        entry_vec,
                    )
                    .await
                    .is_ok()
                {
                    count += 1;
                }
            }
            tracing::info!("🧠 [RAG] Hybrid GraphRAG + Multi-Factor Rerank: Injected {} refined findings into mission scope", count);
        }
        augmentation
    }
}

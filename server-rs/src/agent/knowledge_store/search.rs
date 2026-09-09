//! @docs ARCHITECTURE:Registry
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Agent Runner / search
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Type-safe state handling and bounded execution without unhandled panics.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: `[IKS]`
//! - **Witness Tests**: none declared

use super::store::KnowledgeStore;
use super::types::{KnowledgeEntry, KnowledgeSearchRequest};
use crate::error::AppError;
#[cfg(feature = "vector-memory")]
use std::collections::HashMap;

const HUMAN_CONFIRMED_SCORE_MULTIPLIER: f32 = 2.0;
const SIMILARITY_RANK_WEIGHT: f32 = 0.5;

impl KnowledgeStore {
    /// Hybrid search: LanceDB k-NN with native metadata pre-filter → SQLite hydration → re-rank.
    ///
    /// LanceDB schema contains: `id`, `text`, `mission_id`, `timestamp`, `vector`.
    /// Relational metadata (`topic`, `confidence`, `ttl`, `cluster_id`, `concept_type`, `security_tier`)
    /// is strictly verified during SQLite hydration.
    pub async fn search(
        &self,
        req: &KnowledgeSearchRequest,
        http_client: reqwest::Client,
    ) -> Result<Vec<KnowledgeEntry>, AppError> {
        let limit = (req.limit.unwrap_or(10) as i64).clamp(1, 100);
        let min_confidence = (req.min_confidence.unwrap_or(0.3) as f64).clamp(0.0, 1.0);
        let trimmed_query = req.query.trim();

        // ── Empty Query Fast-Path ──────────────────────────────────────────
        // When query is empty, short-circuit directly to relational listing without embedding.
        if trimmed_query.is_empty() {
            return self
                .search_sqlite_fallback(req, limit, min_confidence)
                .await;
        }

        // ── Vector Path (LanceDB + SQLite Hydration) ───────────────────────
        #[cfg(feature = "vector-memory")]
        {
            let privacy_mode = std::env::var("PRIVACY_MODE")
                .map(|v| v.to_lowercase() == "true" || v == "1")
                .unwrap_or(false);

            if privacy_mode {
                tracing::info!("[IKS] PRIVACY_MODE active — falling back to SQLite text search");
                return self
                    .search_sqlite_fallback(req, limit, min_confidence)
                    .await;
            }

            let api_key = match std::env::var("GOOGLE_API_KEY") {
                Ok(key) if !key.is_empty() => key,
                _ => {
                    tracing::warn!(
                        "[IKS] GOOGLE_API_KEY missing — falling back to SQLite text search"
                    );
                    return self
                        .search_sqlite_fallback(req, limit, min_confidence)
                        .await;
                }
            };

            let query_vector =
                crate::agent::memory::get_gemini_embedding(&http_client, &api_key, trimmed_query)
                    .await?;

            // LanceDB schema has id, text, mission_id, timestamp, vector.
            let mut predicates: Vec<String> = Vec::new();
            if let Some(topic) = &req.topic {
                let escaped_topic = topic
                    .to_lowercase()
                    .replace('\\', "\\\\")
                    .replace('\'', "''");
                predicates.push(format!("mission_id = '{}'", escaped_topic));
            }
            let predicate = predicates.join(" AND ");

            let lance = self.get_lance().await?;
            lance.ensure_table().await?;

            // Over-fetch from ANN index so post-hydration relational filters don't starve results
            let fetch_limit = (limit as usize).saturating_mul(3).max(30);
            let hits = lance
                .search_knowledge_filtered(query_vector, fetch_limit, &predicate)
                .await?;

            if hits.is_empty() {
                return Ok(vec![]);
            }

            // Map hit index to compute semantic rank score
            let hit_count = hits.len() as f32;
            let mut semantic_ranks: HashMap<String, f32> = HashMap::new();
            let mut hit_text: HashMap<String, String> = HashMap::new();
            for (idx, h) in hits.into_iter().enumerate() {
                let rank_weight = 1.0 - (idx as f32 / hit_count);
                semantic_ranks.insert(h.id.clone(), rank_weight);
                hit_text.insert(h.id, h.text);
            }

            let now_unix = chrono::Utc::now().timestamp();
            let hit_ids: Vec<String> = semantic_ranks.keys().cloned().collect();
            let placeholders = hit_ids.iter().map(|_| "?").collect::<Vec<_>>().join(", ");

            // Hydrate metadata from SQLite using parameterized bindings.
            // Security Note: Only '?' placeholders are dynamically generated in the IN-clause.
            let hydrate_sql = format!(
                r#"SELECT id, text, content_hash, topic, cluster_id, source_node_id,
                          source_agent_id, confidence, ttl, human_confirmed,
                          created_at, access_count,
                          concept_type, title, description, resource_uri, tags, security_tier, parent_id
                   FROM knowledge_store_meta
                   WHERE id IN ({})
                     AND confidence >= ?
                     AND (ttl IS NULL OR ttl > ?)
                     AND (? IS NULL OR topic = ?)
                     AND (? IS NULL OR (? = 'global' AND cluster_id IS NULL) OR (? != 'global' AND (cluster_id = ? OR cluster_id IS NULL)))
                     AND (? IS NULL OR concept_type = ?)
                     AND (? IS NULL OR security_tier = ?)
                   ORDER BY confidence DESC"#,
                placeholders
            );

            let mut q = sqlx::query(sqlx::AssertSqlSafe(&*hydrate_sql));
            for id in &hit_ids {
                q = q.bind(id);
            }
            q = q
                .bind(min_confidence)
                .bind(now_unix)
                .bind(&req.topic)
                .bind(&req.topic)
                .bind(&req.cluster_id)
                .bind(&req.cluster_id)
                .bind(&req.cluster_id)
                .bind(&req.cluster_id)
                .bind(&req.concept_type)
                .bind(&req.concept_type)
                .bind(&req.security_tier)
                .bind(&req.security_tier);

            let rows = q.fetch_all(&self.pool).await.map_err(|e| {
                AppError::InternalServerError(format!("[IKS] search hydration failed: {}", e))
            })?;

            let mut results: Vec<KnowledgeEntry> = rows
                .into_iter()
                .map(|r| {
                    let mut entry = Self::entry_from_row(r).map_err(|e| {
                        AppError::InternalServerError(format!("[IKS] row decode failed: {}", e))
                    })?;
                    // Fall back to LanceDB hit text for pre-migration entries if SQLite text is empty
                    if entry.text.is_empty() {
                        if let Some(t) = hit_text.get(&entry.id) {
                            entry.text = t.clone();
                        }
                    }
                    Ok(entry)
                })
                .collect::<Result<Vec<_>, AppError>>()?;

            // Filter out any entries that ended up with empty text
            results.retain(|e| !e.text.trim().is_empty());

            // Blend semantic rank with confidence + deterministic tie-breaking
            results.sort_by(|a, b| {
                let sim_a = semantic_ranks.get(&a.id).copied().unwrap_or(0.0);
                let sim_b = semantic_ranks.get(&b.id).copied().unwrap_or(0.0);

                let score_a = (if a.human_confirmed {
                    HUMAN_CONFIRMED_SCORE_MULTIPLIER
                } else {
                    1.0
                }) * a.confidence
                    + (SIMILARITY_RANK_WEIGHT * sim_a);

                let score_b = (if b.human_confirmed {
                    HUMAN_CONFIRMED_SCORE_MULTIPLIER
                } else {
                    1.0
                }) * b.confidence
                    + (SIMILARITY_RANK_WEIGHT * sim_b);

                score_b
                    .partial_cmp(&score_a)
                    .unwrap_or(std::cmp::Ordering::Equal)
                    .then_with(|| b.created_at.cmp(&a.created_at))
                    .then_with(|| a.id.cmp(&b.id))
            });

            results.truncate(limit as usize);
            return Ok(results);
        }

        // ── Fallback Path (Non-vector build) ────────────────────────────────
        #[cfg(not(feature = "vector-memory"))]
        {
            let _ = http_client;
            self.search_sqlite_fallback(req, limit, min_confidence)
                .await
        }
    }

    /// Internal SQLite relational search fallback.
    pub(crate) async fn search_sqlite_fallback(
        &self,
        req: &KnowledgeSearchRequest,
        limit: i64,
        min_confidence: f64,
    ) -> Result<Vec<KnowledgeEntry>, AppError> {
        let trimmed_query = req.query.trim();
        let escaped_query = trimmed_query
            .replace('\\', "\\\\")
            .replace('%', "\\%")
            .replace('_', "\\_");
        let query_pattern = format!("%{}%", escaped_query);
        let has_query = !trimmed_query.is_empty();

        let candidate_rows = sqlx::query(
            r#"SELECT id, text, content_hash, topic, cluster_id, source_node_id,
                      source_agent_id, confidence, ttl, human_confirmed,
                      created_at, access_count,
                      concept_type, title, description, resource_uri, tags, security_tier, parent_id
               FROM knowledge_store_meta
               WHERE (? = 0 OR text LIKE ? ESCAPE '\' OR title LIKE ? ESCAPE '\' OR description LIKE ? ESCAPE '\')
                 AND (? IS NULL OR topic = ?)
                 AND (? IS NULL OR (? = 'global' AND cluster_id IS NULL) OR (? != 'global' AND (cluster_id = ? OR cluster_id IS NULL)))
                 AND (? IS NULL OR concept_type = ?)
                 AND (? IS NULL OR security_tier = ?)
                 AND confidence >= ?
                 AND (ttl IS NULL OR ttl > unixepoch())
               ORDER BY confidence DESC, created_at DESC, id ASC
               LIMIT ?"#,
        )
        .bind(if has_query { 1 } else { 0 })
        .bind(&query_pattern)
        .bind(&query_pattern)
        .bind(&query_pattern)
        .bind(&req.topic)
        .bind(&req.topic)
        .bind(&req.cluster_id)
        .bind(&req.cluster_id)
        .bind(&req.cluster_id)
        .bind(&req.cluster_id)
        .bind(&req.concept_type)
        .bind(&req.concept_type)
        .bind(&req.security_tier)
        .bind(&req.security_tier)
        .bind(min_confidence)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| {
            AppError::InternalServerError(format!("[IKS] search (sqlite fallback) failed: {}", e))
        })?;

        let results: Vec<KnowledgeEntry> = candidate_rows
            .into_iter()
            .map(Self::entry_from_row)
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| {
                AppError::InternalServerError(format!("[IKS] search row decode failed: {}", e))
            })?;

        Ok(results)
    }
}

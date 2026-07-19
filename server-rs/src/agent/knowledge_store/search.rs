//! @docs ARCHITECTURE:Registry
//!
//! ### AI Assist Note
//! **Core technical resource for the Tadpole OS Sovereign infrastructure.**
//! This module implements high-fidelity logic for the Sovereign Reality layer.
//!
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: Runtime logic error, state desynchronization, or resource exhaustion.
//! - **Telemetry Link**: Search `[search]` in tracing logs.

use super::store::KnowledgeStore;
use super::types::{KnowledgeEntry, KnowledgeSearchRequest};
use crate::error::AppError;
use sqlx::Row;

impl KnowledgeStore {
    /// Hybrid search: LanceDB k-NN with native metadata pre-filter → SQLite hydration → re-rank.
    ///
    /// Filters (`topic`, `cluster_id`, `min_confidence`, TTL) are pushed into LanceDB's
    /// `.only_if()` predicate so the ANN index only scans rows that already satisfy them.
    /// Results are hydrated from SQLite (which holds text + all metadata) and re-ranked.
    ///
    /// When `vector-memory` feature is disabled, falls back to SQLite-only
    /// confidence/topic filtering (no semantic ranking).
    pub async fn search(
        &self,
        req: &KnowledgeSearchRequest,
        http_client: reqwest::Client,
    ) -> Result<Vec<KnowledgeEntry>, AppError> {
        let limit = req.limit.unwrap_or(10) as i64;
        let min_confidence = req.min_confidence.unwrap_or(0.3) as f64;

        // ── Vector path (LanceDB with native filter) ───────────────────────
        #[cfg(feature = "vector-memory")]
        {
            let query_vector = {
                let api_key = std::env::var("GOOGLE_API_KEY").map_err(|_| {
                    AppError::InternalServerError(
                        "[IKS] GOOGLE_API_KEY required for embedding. Set it in .env.".to_string(),
                    )
                })?;
                crate::agent::memory::get_gemini_embedding(&http_client, &api_key, &req.query)
                    .await?
            };

            // Build the LanceDB WHERE predicate from the request filters.
            // LanceDB supports SQL-like predicates via `.only_if()`.
            let now_unix = chrono::Utc::now().timestamp();
            let mut predicates: Vec<String> = vec![
                format!("confidence >= {}", min_confidence),
                format!("(ttl IS NULL OR ttl > {})", now_unix),
            ];
            if let Some(topic) = &req.topic {
                // topic is lowercased on write, enforce here too.
                predicates.push(format!("mission_id = '{}'", topic.to_lowercase()));
            }
            if let Some(cluster) = &req.cluster_id {
                // cluster_id is not in LanceDB schema — this filter is applied in SQLite
                // hydration below. LanceDB only stores id, text, mission_id, timestamp, vector.
                let _ = cluster;
            }
            let predicate = predicates.join(" AND ");

            let lance = self.get_lance().await?;
            lance.ensure_table().await?;

            // Use internal LanceDB filtering to avoid the lossy Rust set intersection.
            let hits = lance
                .search_knowledge_filtered(query_vector, limit as usize, &predicate)
                .await?;

            if hits.is_empty() {
                return Ok(vec![]);
            }

            // Hydrate full metadata (including cluster_id, confidence, human_confirmed)
            // from SQLite using a single IN query.
            let hit_ids: Vec<String> = hits.iter().map(|h| h.id.clone()).collect();
            let placeholders = hit_ids.iter().map(|_| "?").collect::<Vec<_>>().join(", ");
            let hydrate_sql = format!(
                r#"SELECT id, text, content_hash, topic, cluster_id, source_node_id,
                          source_agent_id, confidence, ttl, human_confirmed,
                          created_at, access_count,
                          concept_type, title, description, resource_uri, tags
                   FROM knowledge_store_meta
                   WHERE id IN ({})
                     AND confidence >= {}
                     AND (ttl IS NULL OR ttl > {})
                     {}
                     {}
                   ORDER BY confidence DESC"#,
                placeholders,
                min_confidence,
                now_unix,
                req.cluster_id
                    .as_deref()
                    .map(|c| format!(
                        "AND (cluster_id = '{}' OR cluster_id IS NULL)",
                        c.replace('\'', "''")
                    ))
                    .unwrap_or_default(),
                req.concept_type
                    .as_deref()
                    .map(|ct| format!("AND concept_type = '{}'", ct.replace('\'', "''")))
                    .unwrap_or_default(),
            );
            let mut q = sqlx::query(&hydrate_sql);
            for id in &hit_ids {
                q = q.bind(id);
            }
            let rows = q.fetch_all(&self.pool).await.map_err(|e| {
                AppError::InternalServerError(format!("[IKS] search hydration failed: {}", e))
            })?;

            // Build a text map from vector hits for entries whose SQLite text may be
            // empty (pre-migration entries). LanceDB text is the source of truth for
            // entries written before migration 20260601000101.
            let hit_text: std::collections::HashMap<String, String> =
                hits.into_iter().map(|h| (h.id, h.text)).collect();

            let mut results: Vec<KnowledgeEntry> = rows
                .into_iter()
                .map(|r| {
                    // Prefer SQLite text; fall back to LanceDB hit text for pre-migration rows.
                    let sqlite_text: String = r.try_get("text").unwrap_or_default();
                    let text = if sqlite_text.is_empty() {
                        hit_text
                            .get(r.try_get::<String, _>("id").as_deref().unwrap_or(""))
                            .cloned()
                            .unwrap_or_default()
                    } else {
                        sqlite_text
                    };
                    KnowledgeEntry {
                        id: r.try_get("id").unwrap_or_default(),
                        text,
                        topic: r.try_get("topic").unwrap_or_default(),
                        cluster_id: r.try_get("cluster_id").ok(),
                        source_node_id: r.try_get("source_node_id").ok(),
                        source_agent_id: r.try_get("source_agent_id").ok(),
                        content_hash: r.try_get("content_hash").unwrap_or_default(),
                        confidence: r.try_get::<f64, _>("confidence").unwrap_or(0.0) as f32,
                        human_confirmed: r.try_get::<i64, _>("human_confirmed").unwrap_or(0) != 0,
                        ttl: r.try_get("ttl").ok(),
                        created_at: r.try_get("created_at").unwrap_or(0),
                        access_count: r.try_get("access_count").unwrap_or(0),
                        concept_type: r
                            .try_get("concept_type")
                            .unwrap_or_else(|_| "general".to_string()),
                        title: r.try_get("title").ok(),
                        description: r.try_get("description").ok(),
                        resource_uri: r.try_get("resource_uri").ok(),
                        tags: r.try_get("tags").ok(),
                    }
                })
                .collect();

            // Re-rank: human-confirmed + high confidence first
            results.sort_by(|a, b| {
                let score_a = if a.human_confirmed { 2.0_f32 } else { 1.0 } * a.confidence;
                let score_b = if b.human_confirmed { 2.0_f32 } else { 1.0 } * b.confidence;
                score_b
                    .partial_cmp(&score_a)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
            results.truncate(limit as usize);
            return Ok(results);
        }

        // ── Fallback: SQLite-only (no vector-memory feature) ───────────────
        #[cfg(not(feature = "vector-memory"))]
        {
            let _ = http_client;
            let candidate_rows = sqlx::query(
                r#"SELECT id, text, content_hash, topic, cluster_id, source_node_id,
                          source_agent_id, confidence, ttl, human_confirmed,
                          created_at, access_count,
                          concept_type, title, description, resource_uri, tags
                   FROM knowledge_store_meta
                   WHERE (? IS NULL OR topic = ?)
                     AND (? IS NULL OR cluster_id = ? OR cluster_id IS NULL)
                     AND (? IS NULL OR concept_type = ?)
                     AND confidence >= ?
                     AND (ttl IS NULL OR ttl > unixepoch())
                   ORDER BY confidence DESC
                   LIMIT ?"#,
            )
            .bind(&req.topic)
            .bind(&req.topic)
            .bind(&req.cluster_id)
            .bind(&req.cluster_id)
            .bind(&req.concept_type)
            .bind(&req.concept_type)
            .bind(min_confidence)
            .bind(limit)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| {
                AppError::InternalServerError(format!("[IKS] search (sqlite-only) failed: {}", e))
            })?;

            let results: Vec<KnowledgeEntry> = candidate_rows
                .into_iter()
                .map(Self::entry_from_row)
                .collect::<Result<Vec<_>, _>>()
                .map_err(|e| {
                    AppError::InternalServerError(format!("[IKS] search row decode failed: {}", e))
                })?;
            return Ok(results);
        }
    }
}

// Metadata: [search]

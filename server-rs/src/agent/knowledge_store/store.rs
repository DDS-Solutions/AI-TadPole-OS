//! @docs ARCHITECTURE:Registry
//!
//! ### AI Assist Note
//! **Core technical resource for the Tadpole OS Sovereign infrastructure.**
//! This module implements high-fidelity logic for the Sovereign Reality layer.
//!
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: Runtime logic error, state desynchronization, or resource exhaustion.
//! - **Telemetry Link**: Search `[store]` in tracing logs.

use super::types::{AddKnowledgeRequest, KnowledgeEntry, DEFAULT_TTL_DAYS};
use crate::error::AppError;
use sqlx::{Row, SqlitePool};
use std::sync::Arc;

pub struct KnowledgeStore {
    pub(crate) pool: SqlitePool,
    #[cfg(feature = "vector-memory")]
    pub(crate) lance: tokio::sync::OnceCell<Arc<crate::agent::memory::VectorMemory>>,
}

impl KnowledgeStore {
    /// Creates a new KnowledgeStore backed by the given pool.
    /// The LanceDB connection is initialized lazily on first `add_entry` call.
    pub fn new(pool: SqlitePool) -> Self {
        Self {
            pool,
            #[cfg(feature = "vector-memory")]
            lance: tokio::sync::OnceCell::new(),
        }
    }

    /// Lazily initializes and returns the LanceDB vector store.
    #[cfg(feature = "vector-memory")]
    pub(super) async fn get_lance(
        &self,
    ) -> Result<Arc<crate::agent::memory::VectorMemory>, AppError> {
        let lance = self
            .lance
            .get_or_try_init(|| async {
                let v = crate::agent::memory::VectorMemory::connect(
                    "data/iks/knowledge_store",
                    "knowledge_store",
                )
                .await?;
                Ok::<_, AppError>(Arc::new(v))
            })
            .await?;
        Ok(lance.clone())
    }

    /// Computes a SHA-256 hex hash of the given text for dedup and P2P idempotency.
    pub(super) fn sha256_hash(text: &str) -> String {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(text.as_bytes());
        let result = hasher.finalize();
        hex::encode(result)
    }

    /// Computes the TTL unix timestamp for a new entry.
    /// Q3 decision: agent default = 90d, human-confirmed = NULL (never).
    pub(super) fn compute_ttl(
        human_confirmed: bool,
        ttl_days: Option<i64>,
        now_unix: i64,
    ) -> Option<i64> {
        match (human_confirmed, ttl_days) {
            (true, _) => None,                               // human-confirmed → never expires
            (false, Some(d)) => Some(now_unix + d * 86_400), // caller-supplied
            (false, None) => Some(now_unix + DEFAULT_TTL_DAYS * 86_400), // agent default: 90d
        }
    }

    pub(super) fn entry_from_row(
        row: sqlx::sqlite::SqliteRow,
    ) -> Result<KnowledgeEntry, sqlx::Error> {
        Ok(KnowledgeEntry {
            id: row.try_get("id")?,
            text: row.try_get("text")?,
            topic: row.try_get("topic")?,
            cluster_id: row.try_get("cluster_id")?,
            source_node_id: row.try_get("source_node_id")?,
            source_agent_id: row.try_get("source_agent_id")?,
            content_hash: row.try_get("content_hash")?,
            confidence: row.try_get::<f64, _>("confidence")? as f32,
            human_confirmed: row.try_get::<i64, _>("human_confirmed")? != 0,
            ttl: row.try_get("ttl")?,
            created_at: row.try_get("created_at")?,
            access_count: row.try_get("access_count")?,
            concept_type: row.try_get("concept_type")?,
            title: row.try_get("title")?,
            description: row.try_get("description")?,
            resource_uri: row.try_get("resource_uri")?,
            tags: row.try_get("tags")?,
        })
    }

    /// Write a new knowledge entry.
    ///
    /// Deduplicates by `content_hash` — returns the existing entry unchanged if
    /// the same text has already been stored.
    ///
    /// Embedding is always computed via `GOOGLE_API_KEY` regardless of the
    /// calling agent's provider config (dimensional consistency, Q1 decision).
    ///
    /// Returns `Err` if `GOOGLE_API_KEY` is absent or `PRIVACY_MODE=true`.
    pub async fn add_entry(
        &self,
        req: AddKnowledgeRequest,
        http_client: reqwest::Client,
    ) -> Result<KnowledgeEntry, AppError> {
        // ── Privacy guard ──────────────────────────────────────────────────
        let privacy_mode = std::env::var("PRIVACY_MODE")
            .map(|v| v.to_lowercase() == "true")
            .unwrap_or(false);
        if privacy_mode {
            tracing::warn!(
                topic = %req.topic,
                "[IKS] PRIVACY_MODE active — skipping knowledge store write"
            );
            return Err(AppError::InternalServerError(
                "[IKS] Writes require cloud embedding (PRIVACY_MODE=true)".to_string(),
            ));
        }

        // ── Dedup check ────────────────────────────────────────────────────
        let content_hash = Self::sha256_hash(&req.text);
        let existing =
            sqlx::query("SELECT id FROM knowledge_store_meta WHERE content_hash = ? LIMIT 1")
                .bind(&content_hash)
                .fetch_optional(&self.pool)
                .await
                .map_err(|e| {
                    AppError::InternalServerError(format!("[IKS] Dedup check failed: {}", e))
                })?;

        if let Some(row) = existing {
            let existing_id: String = row.try_get("id").map_err(|e| {
                AppError::InternalServerError(format!("[IKS] Dedup row decode failed: {}", e))
            })?;
            tracing::debug!(id = %existing_id, "[IKS] Dedup hit — returning existing entry");
            return self
                .get_by_id(&existing_id)
                .await?
                .ok_or_else(|| AppError::NotFound("[IKS] Dedup entry vanished".to_string()));
        }

        // ── Compute embedding ──────────────────────────────────────────────
        #[cfg(feature = "vector-memory")]
        let vector = {
            let api_key = std::env::var("GOOGLE_API_KEY").map_err(|_| {
                AppError::InternalServerError(
                    "[IKS] GOOGLE_API_KEY required for embedding. Set it in .env.".to_string(),
                )
            })?;
            crate::agent::memory::get_gemini_embedding(&http_client, &api_key, &req.text).await?
        };

        // ── Prepare metadata ───────────────────────────────────────────────
        let id = uuid::Uuid::new_v4().to_string();
        let now_unix = chrono::Utc::now().timestamp();
        let human_confirmed = req.human_confirmed.unwrap_or(false);
        let confidence = if human_confirmed {
            1.0_f32
        } else {
            req.confidence.unwrap_or(1.0).clamp(0.0, 1.0)
        };
        let ttl = Self::compute_ttl(human_confirmed, req.ttl_days, now_unix);
        let human_confirmed_int: i64 = if human_confirmed { 1 } else { 0 };
        let topic = req.topic.to_lowercase();
        let concept_type = req
            .concept_type
            .unwrap_or_else(|| "general".to_string())
            .to_lowercase();

        // ── Insert SQLite metadata row (text stored here, not only LanceDB) ─
        sqlx::query(
            r#"INSERT INTO knowledge_store_meta
               (id, text, content_hash, topic, cluster_id, source_node_id, source_agent_id,
                confidence, ttl, human_confirmed, created_at, updated_at,
                concept_type, title, description, resource_uri, tags)
               VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"#,
        )
        .bind(&id)
        .bind(&req.text)
        .bind(&content_hash)
        .bind(&topic)
        .bind(&req.cluster_id)
        .bind(&req.source_node_id)
        .bind(&req.source_agent_id)
        .bind(confidence)
        .bind(ttl)
        .bind(human_confirmed_int)
        .bind(now_unix)
        .bind(now_unix)
        .bind(&concept_type)
        .bind(&req.title)
        .bind(&req.description)
        .bind(&req.resource_uri)
        .bind(&req.tags)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::InternalServerError(format!("[IKS] SQLite insert failed: {}", e)))?;

        // ── Insert LanceDB vector row ──────────────────────────────────────
        #[cfg(feature = "vector-memory")]
        {
            let lance = self.get_lance().await?;
            lance.ensure_table().await?;
            lance
                .add_memory(&id, &req.text, &topic, vector)
                .await
                .map_err(|e| {
                    AppError::InternalServerError(format!("[IKS] LanceDB insert failed: {}", e))
                })?;
        }

        tracing::info!(
            id = %id,
            topic = %topic,
            human_confirmed = human_confirmed,
            "[IKS] New knowledge entry stored"
        );

        Ok(KnowledgeEntry {
            id,
            text: req.text,
            topic,
            cluster_id: req.cluster_id,
            source_node_id: req.source_node_id,
            source_agent_id: req.source_agent_id,
            content_hash,
            confidence,
            human_confirmed,
            ttl,
            created_at: now_unix,
            access_count: 0,
            concept_type,
            title: req.title,
            description: req.description,
            resource_uri: req.resource_uri,
            tags: req.tags,
        })
    }

    /// Fetch a single entry by ID. Increments `access_count` and updates
    /// `last_accessed_at` as a side effect.
    ///
    /// `text` is read directly from the SQLite `knowledge_store_meta.text` column.
    pub async fn get_by_id(&self, id: &str) -> Result<Option<KnowledgeEntry>, AppError> {
        let row = sqlx::query(
            r#"SELECT id, text, content_hash, topic, cluster_id, source_node_id,
                      source_agent_id, confidence, ttl, human_confirmed,
                      created_at, access_count,
                      concept_type, title, description, resource_uri, tags
               FROM knowledge_store_meta WHERE id = ?"#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::InternalServerError(format!("[IKS] get_by_id failed: {}", e)))?;

        if let Some(r) = row {
            let now = chrono::Utc::now().timestamp();
            let _ = sqlx::query(
                "UPDATE knowledge_store_meta SET access_count = access_count + 1, last_accessed_at = ? WHERE id = ?",
            )
            .bind(now)
            .bind(id)
            .execute(&self.pool)
            .await;

            let mut entry = Self::entry_from_row(r).map_err(|e| {
                AppError::InternalServerError(format!("[IKS] get_by_id row decode failed: {}", e))
            })?;
            entry.access_count += 1;
            Ok(Some(entry))
        } else {
            Ok(None)
        }
    }

    /// Paginated list of entries with optional topic/cluster/type filters.
    pub async fn list(
        &self,
        topic: Option<&str>,
        cluster_id: Option<&str>,
        concept_type: Option<&str>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<KnowledgeEntry>, AppError> {
        let rows = sqlx::query(
            r#"SELECT id, text, content_hash, topic, cluster_id, source_node_id,
                      source_agent_id, confidence, ttl, human_confirmed,
                      created_at, access_count,
                      concept_type, title, description, resource_uri, tags
               FROM knowledge_store_meta
               WHERE (? IS NULL OR topic = ?)
                 AND (? IS NULL OR cluster_id = ? OR cluster_id IS NULL)
                 AND (? IS NULL OR concept_type = ?)
               ORDER BY created_at DESC
               LIMIT ? OFFSET ?"#,
        )
        .bind(topic)
        .bind(topic)
        .bind(cluster_id)
        .bind(cluster_id)
        .bind(concept_type)
        .bind(concept_type)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::InternalServerError(format!("[IKS] list failed: {}", e)))?;

        Ok(rows
            .into_iter()
            .map(Self::entry_from_row)
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| {
                AppError::InternalServerError(format!("[IKS] list row decode failed: {}", e))
            })?)
    }

    /// P2P sync: return all entries written since `since` (unix timestamp).
    pub async fn get_entries_since(&self, since: i64) -> Result<Vec<KnowledgeEntry>, AppError> {
        let rows = sqlx::query(
            r#"SELECT id, text, content_hash, topic, cluster_id, source_node_id,
                      source_agent_id, confidence, ttl, human_confirmed,
                      created_at, access_count,
                      concept_type, title, description, resource_uri, tags
               FROM knowledge_store_meta
               WHERE created_at > ?
               ORDER BY created_at ASC"#,
        )
        .bind(since)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| {
            AppError::InternalServerError(format!("[IKS] get_entries_since failed: {}", e))
        })?;

        Ok(rows
            .into_iter()
            .map(Self::entry_from_row)
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| {
                AppError::InternalServerError(format!("[IKS] sync row decode failed: {}", e))
            })?)
    }

    /// Delete an entry by ID. Removes from both SQLite and LanceDB.
    ///
    /// LanceDB deletion is routed through the shared `VectorMemory` instance
    /// to avoid creating a redundant connection pool on every call.
    pub async fn delete(&self, id: &str) -> Result<(), AppError> {
        sqlx::query("DELETE FROM knowledge_store_meta WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| {
                AppError::InternalServerError(format!("[IKS] delete SQLite failed: {}", e))
            })?;

        #[cfg(feature = "vector-memory")]
        {
            if let Ok(lance) = self.get_lance().await {
                // delete_memories handles the connection and query construction.
                if let Err(e) = lance.delete_memories(vec![id.to_string()]).await {
                    tracing::warn!(id = %id, error = %e, "[IKS] LanceDB delete failed (SQLite row already removed)");
                }
            }
        }

        tracing::info!(id = %id, "[IKS] Entry deleted");
        Ok(())
    }

    /// Removes an entry by ID. Refuses to delete human-confirmed entries
    /// unless `force` is set to true.
    pub async fn remove(&self, id: &str, force: bool) -> Result<(), AppError> {
        if !force {
            if let Some(entry) = self.get_by_id(id).await? {
                if entry.human_confirmed {
                    return Err(AppError::Conflict(
                        "[IKS] Cannot delete human-confirmed entry without force=true".to_string(),
                    ));
                }
            }
        }
        self.delete(id).await
    }

    /// Mark an entry as human-confirmed. Clears TTL and sets confidence = 1.0.
    /// Idempotent — calling on an already-confirmed entry is a safe no-op.
    ///
    /// This is the Q3 "human-confirmed = never expire" enforcement point.
    pub async fn confirm(&self, id: &str) -> Result<KnowledgeEntry, AppError> {
        sqlx::query(
            r#"UPDATE knowledge_store_meta
               SET human_confirmed = 1,
                   ttl = NULL,
                   confidence = 1.0,
                   updated_at = unixepoch()
               WHERE id = ?"#,
        )
        .bind(id)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::InternalServerError(format!("[IKS] confirm failed: {}", e)))?;

        tracing::info!(id = %id, "[IKS] Entry confirmed by human — TTL cleared");

        self.get_by_id(id).await?.ok_or_else(|| {
            AppError::NotFound(format!("[IKS] Entry {} not found after confirm", id))
        })
    }

    /// Finds similar knowledge entries (peers) based on vector distance, excluding the node itself.
    pub async fn get_peers(
        &self,
        id: &str,
        limit: usize,
        http_client: reqwest::Client,
    ) -> Result<Vec<KnowledgeEntry>, AppError> {
        let entry = self
            .get_by_id(id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("[IKS] Entry {} not found", id)))?;

        #[cfg(feature = "vector-memory")]
        {
            let api_key = std::env::var("GOOGLE_API_KEY").map_err(|_| {
                AppError::InternalServerError(
                    "[IKS] GOOGLE_API_KEY required for embedding. Set it in .env.".to_string(),
                )
            })?;
            let query_vector =
                crate::agent::memory::get_gemini_embedding(&http_client, &api_key, &entry.text)
                    .await?;

            let lance = self.get_lance().await?;
            lance.ensure_table().await?;

            // Retrieve top peers (limit + 1 to account for self-exclusion)
            // L2 distance limit of 0.5 corresponds to high similarity
            let predicate = format!(
                "id != '{}' AND confidence >= 0.0 AND (ttl IS NULL OR ttl > {})",
                id,
                chrono::Utc::now().timestamp()
            );
            let hits = lance
                .search_knowledge_filtered(query_vector, limit + 1, &predicate)
                .await?;

            if hits.is_empty() {
                return Ok(vec![]);
            }

            let hit_ids: Vec<String> = hits.iter().map(|h| h.id.clone()).collect();
            let placeholders = hit_ids.iter().map(|_| "?").collect::<Vec<_>>().join(", ");
            let hydrate_sql = format!(
                r#"SELECT id, text, content_hash, topic, cluster_id, source_node_id,
                          source_agent_id, confidence, ttl, human_confirmed,
                          created_at, access_count,
                          concept_type, title, description, resource_uri, tags
                   FROM knowledge_store_meta
                   WHERE id IN ({})
                   ORDER BY confidence DESC"#,
                placeholders
            );
            let mut q = sqlx::query(&hydrate_sql);
            for hit_id in &hit_ids {
                q = q.bind(hit_id);
            }
            let rows = q.fetch_all(&self.pool).await.map_err(|e| {
                AppError::InternalServerError(format!("[IKS] peer hydration failed: {}", e))
            })?;

            let mut results: Vec<KnowledgeEntry> = rows
                .into_iter()
                .map(Self::entry_from_row)
                .collect::<Result<Vec<_>, _>>()
                .map_err(|e| {
                    AppError::InternalServerError(format!("[IKS] peer row decode failed: {}", e))
                })?;

            // Limit results
            results.truncate(limit);
            Ok(results)
        }

        #[cfg(not(feature = "vector-memory"))]
        {
            let _ = http_client;
            let _ = limit;
            // Fallback: list entries of same topic, excluding self
            let rows = sqlx::query(
                r#"SELECT id, text, content_hash, topic, cluster_id, source_node_id,
                          source_agent_id, confidence, ttl, human_confirmed,
                          created_at, access_count,
                          concept_type, title, description, resource_uri, tags
                   FROM knowledge_store_meta
                   WHERE id != ? AND topic = ?
                   LIMIT ?"#,
            )
            .bind(id)
            .bind(&entry.topic)
            .bind(limit as i64)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| {
                AppError::InternalServerError(format!("[IKS] get_peers fallback failed: {}", e))
            })?;

            let results = rows
                .into_iter()
                .map(Self::entry_from_row)
                .collect::<Result<Vec<_>, _>>()
                .map_err(|e| {
                    AppError::InternalServerError(format!(
                        "[IKS] peer fallback decode failed: {}",
                        e
                    ))
                })?;
            Ok(results)
        }
    }

    /// TTL eviction: delete all expired entries where `human_confirmed = 0`.
    ///
    /// The `human_confirmed = 0` guard is the critical safety clause —
    /// even if a confirmed entry somehow had a TTL set, it will not be deleted.
    pub async fn evict_expired(&self) -> Result<u64, AppError> {
        let now = chrono::Utc::now().timestamp();
        let result = sqlx::query(
            r#"DELETE FROM knowledge_store_meta
               WHERE ttl IS NOT NULL
                 AND ttl < ?
                 AND human_confirmed = 0"#,
        )
        .bind(now)
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::InternalServerError(format!("[IKS] evict_expired failed: {}", e)))?;

        let evicted = result.rows_affected();
        if evicted > 0 {
            tracing::info!(count = evicted, "[IKS] Evicted expired knowledge entries");
        }
        Ok(evicted)
    }

    /// Confidence decay: reduce confidence based on actual time elapsed since last update.
    ///
    /// Rate: 0.01 per day (time-aware). An entry not touched for 10 days will lose
    /// 0.10 confidence in a single cron run, catching up to the correct value.
    /// Running the cron twice in one day is safe — the guard clause
    /// (`updated_at < unixepoch() - 86400`) prevents double-decay within 24h.
    ///
    /// Human-confirmed entries are never decayed.
    pub async fn decay_confidence(&self) -> Result<(), AppError> {
        sqlx::query(
            r#"UPDATE knowledge_store_meta
               SET confidence = MAX(0.0, confidence - (0.01 * CAST((unixepoch() - updated_at) / 86400.0 AS REAL))),
                   updated_at = unixepoch()
               WHERE human_confirmed = 0
                 AND updated_at < unixepoch() - 86400"#,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| {
            AppError::InternalServerError(format!("[IKS] decay_confidence failed: {}", e))
        })?;

        tracing::debug!("[IKS] Time-aware confidence decay applied to unconfirmed entries");
        Ok(())
    }
}

// Metadata: [store]

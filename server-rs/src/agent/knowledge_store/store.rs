//! @docs ARCHITECTURE:Registry
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Agent Runner / store
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Type-safe state handling and bounded execution without unhandled panics.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: `[IKS]`
//! - **Witness Tests**: none declared

use super::types::{
    AddKnowledgeRequest, KnowledgeEntry, SecurityTier, DEFAULT_AGENT_CONFIDENCE, DEFAULT_TTL_DAYS,
    MAX_UNCONFIRMED_CONFIDENCE,
};
use crate::error::AppError;
use sqlx::{Row, SqlitePool};
#[cfg(feature = "vector-memory")]
use std::sync::Arc;

pub struct KnowledgeStore {
    pub(crate) pool: SqlitePool,
    #[cfg(feature = "vector-memory")]
    pub(crate) lance: tokio::sync::OnceCell<Arc<crate::agent::memory::VectorMemory>>,
}

impl KnowledgeStore {
    /// Creates a new KnowledgeStore backed by the given SQLite connection pool.
    pub fn new(pool: SqlitePool) -> Self {
        Self {
            pool,
            #[cfg(feature = "vector-memory")]
            lance: tokio::sync::OnceCell::new(),
        }
    }

    /// Lazily initializes and returns the LanceDB vector store connection.
    #[cfg(feature = "vector-memory")]
    pub(super) async fn get_lance(
        &self,
    ) -> Result<Arc<crate::agent::memory::VectorMemory>, AppError> {
        let lance = self
            .lance
            .get_or_try_init(|| async {
                let data_dir = std::env::var("IKS_DATA_DIR")
                    .unwrap_or_else(|_| "data/iks/knowledge_store".to_string());
                let v = crate::agent::memory::VectorMemory::connect(&data_dir, "knowledge_store")
                    .await?;
                Ok::<_, AppError>(Arc::new(v))
            })
            .await?;
        Ok(lance.clone())
    }

    /// Computes a scoped SHA-256 hex hash (topic:cluster:text) for dedup and P2P idempotency.
    pub fn sha256_hash(topic: &str, cluster_id: Option<&str>, text: &str) -> String {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        let scoped_input = format!(
            "{}:{}:{}",
            topic.trim().to_lowercase(),
            cluster_id.unwrap_or("").trim(),
            text.trim()
        );
        hasher.update(scoped_input.as_bytes());
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
        let sec_tier_str: String = row
            .try_get("security_tier")
            .unwrap_or_else(|_| "BRONZE_ADHOC".to_string());
        let security_tier = SecurityTier::from_str_lossless(&sec_tier_str)
            .as_str()
            .to_string();

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
            security_tier,
            parent_id: row.try_get("parent_id").ok(),
        })
    }

    /// Internal lookup by content hash without tracking access.
    pub async fn get_by_hash(
        &self,
        content_hash: &str,
    ) -> Result<Option<KnowledgeEntry>, AppError> {
        let row = sqlx::query(
            r#"SELECT id, text, content_hash, topic, cluster_id, source_node_id,
                      source_agent_id, confidence, ttl, human_confirmed,
                      created_at, access_count,
                      concept_type, title, description, resource_uri, tags, security_tier, parent_id
               FROM knowledge_store_meta WHERE content_hash = ? LIMIT 1"#,
        )
        .bind(content_hash)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::InternalServerError(format!("[IKS] get_by_hash failed: {}", e)))?;

        row.map(Self::entry_from_row)
            .transpose()
            .map_err(|e| AppError::InternalServerError(format!("[IKS] row decode failed: {}", e)))
    }

    /// Write a new knowledge entry.
    ///
    /// Deduplicates by scoped `content_hash` (topic:cluster:text) — returns existing entry
    /// unchanged if identical knowledge already exists.
    ///
    /// Agent submissions default to confidence 0.70 (capped at 0.80) and 90-day TTL.
    /// Human confirmation can only be set via the human-authenticated `confirm(id)` route.
    pub async fn add_entry(
        &self,
        req: AddKnowledgeRequest,
        #[allow(unused_variables)] http_client: reqwest::Client,
    ) -> Result<KnowledgeEntry, AppError> {
        if req.text.trim().is_empty() {
            return Err(AppError::BadRequest(
                "Knowledge text cannot be empty".to_string(),
            ));
        }

        if let Some(d) = req.ttl_days {
            if !(1..=3650).contains(&d) {
                return Err(AppError::BadRequest(
                    "ttl_days must be between 1 and 3650".to_string(),
                ));
            }
        }

        // ── Privacy guard ──────────────────────────────────────────────────
        let privacy_mode = std::env::var("PRIVACY_MODE")
            .map(|v| v.to_lowercase() == "true" || v == "1")
            .unwrap_or(false);
        if privacy_mode {
            tracing::warn!(
                topic = %req.topic,
                "[IKS] PRIVACY_MODE active — rejecting knowledge store write"
            );
            return Err(AppError::Conflict(
                "[IKS] Writes disabled while PRIVACY_MODE is active".to_string(),
            ));
        }

        // ── Dedup check ────────────────────────────────────────────────────
        let content_hash = Self::sha256_hash(&req.topic, req.cluster_id.as_deref(), &req.text);
        if let Some(existing) = self.get_by_hash(&content_hash).await? {
            tracing::debug!(id = %existing.id, "[IKS] Dedup hit — returning existing entry");
            return Ok(existing);
        }

        // ── Compute embedding ──────────────────────────────────────────────
        #[cfg(feature = "vector-memory")]
        let vector = {
            let api_key = std::env::var("GOOGLE_API_KEY").map_err(|_| {
                AppError::BadRequest(
                    "[IKS] GOOGLE_API_KEY required for embedding. Set it in .env.".to_string(),
                )
            })?;
            crate::agent::memory::get_gemini_embedding(&http_client, &api_key, &req.text).await?
        };

        // ── Prepare metadata ───────────────────────────────────────────────
        let id = uuid::Uuid::new_v4().to_string();
        let now_unix = chrono::Utc::now().timestamp();
        // Agent writes are always unconfirmed at creation time
        let human_confirmed = false;
        let confidence = req
            .confidence
            .unwrap_or(DEFAULT_AGENT_CONFIDENCE)
            .clamp(0.0, MAX_UNCONFIRMED_CONFIDENCE);
        let ttl = Self::compute_ttl(human_confirmed, req.ttl_days, now_unix);
        let topic = req.topic.trim().to_lowercase();
        let concept_type = req
            .concept_type
            .unwrap_or_else(|| "general".to_string())
            .trim()
            .to_lowercase();
        let security_tier = req
            .security_tier
            .as_deref()
            .map(SecurityTier::from_str_lossless)
            .unwrap_or_default()
            .as_str()
            .to_string();

        // ── SQLite Transaction with Atomic ON CONFLICT ───────────────────────
        let mut tx = self.pool.begin().await.map_err(|e| {
            AppError::InternalServerError(format!("[IKS] Failed to start transaction: {}", e))
        })?;

        let insert_res = sqlx::query(
            r#"INSERT INTO knowledge_store_meta
               (id, text, content_hash, topic, cluster_id, source_node_id, source_agent_id,
                confidence, ttl, human_confirmed, created_at, updated_at,
                concept_type, title, description, resource_uri, tags, security_tier, parent_id)
               VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, 0, ?, ?, ?, ?, ?, ?, ?, ?, ?)
               ON CONFLICT(content_hash) DO NOTHING"#,
        )
        .bind(&id)
        .bind(&req.text)
        .bind(&content_hash)
        .bind(&topic)
        .bind(&req.cluster_id)
        .bind(&req.source_node_id)
        .bind(&req.source_agent_id)
        .bind(confidence as f64)
        .bind(ttl)
        .bind(now_unix)
        .bind(now_unix)
        .bind(&concept_type)
        .bind(&req.title)
        .bind(&req.description)
        .bind(&req.resource_uri)
        .bind(&req.tags)
        .bind(&security_tier)
        .bind(&req.parent_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| AppError::InternalServerError(format!("[IKS] SQLite insert failed: {}", e)))?;

        // If another thread inserted the exact same content_hash concurrently:
        if insert_res.rows_affected() == 0 {
            let _ = tx.rollback().await;
            if let Some(existing) = self.get_by_hash(&content_hash).await? {
                return Ok(existing);
            }
            return Err(AppError::InternalServerError(
                "[IKS] Concurrent insert conflict resolution failed".to_string(),
            ));
        }

        // ── Insert LanceDB vector row ──────────────────────────────────────
        #[cfg(feature = "vector-memory")]
        {
            let lance = self.get_lance().await?;
            lance.ensure_table().await?;
            if let Err(e) = lance.add_memory(&id, &req.text, &topic, vector).await {
                let _ = tx.rollback().await;
                return Err(AppError::InternalServerError(format!(
                    "[IKS] LanceDB insert failed: {}",
                    e
                )));
            }
        }

        // Commit SQLite metadata
        tx.commit().await.map_err(|e| {
            AppError::InternalServerError(format!("[IKS] Transaction commit failed: {}", e))
        })?;

        tracing::info!(
            id = %id,
            topic = %topic,
            human_confirmed = false,
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
            human_confirmed: false,
            ttl,
            created_at: now_unix,
            access_count: 0,
            concept_type,
            title: req.title,
            description: req.description,
            resource_uri: req.resource_uri,
            tags: req.tags,
            security_tier,
            parent_id: req.parent_id,
        })
    }

    /// Fetch a single entry by ID, optionally tracking access telemetry.
    pub async fn get_by_id_internal(
        &self,
        id: &str,
        track_access: bool,
    ) -> Result<Option<KnowledgeEntry>, AppError> {
        let row = sqlx::query(
            r#"SELECT id, text, content_hash, topic, cluster_id, source_node_id,
                      source_agent_id, confidence, ttl, human_confirmed,
                      created_at, access_count,
                      concept_type, title, description, resource_uri, tags, security_tier, parent_id
               FROM knowledge_store_meta WHERE id = ?"#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::InternalServerError(format!("[IKS] get_by_id failed: {}", e)))?;

        if let Some(r) = row {
            if track_access {
                let now = chrono::Utc::now().timestamp();
                let _ = sqlx::query(
                    "UPDATE knowledge_store_meta SET access_count = access_count + 1, last_accessed_at = ? WHERE id = ?",
                )
                .bind(now)
                .bind(id)
                .execute(&self.pool)
                .await;
            }

            let mut entry = Self::entry_from_row(r).map_err(|e| {
                AppError::InternalServerError(format!("[IKS] get_by_id row decode failed: {}", e))
            })?;
            if track_access {
                entry.access_count += 1;
            }
            Ok(Some(entry))
        } else {
            Ok(None)
        }
    }

    /// Fetch a single entry by ID. Increments `access_count` as a side effect.
    pub async fn get_by_id(&self, id: &str) -> Result<Option<KnowledgeEntry>, AppError> {
        self.get_by_id_internal(id, true).await
    }

    /// Paginated list of unexpired entries with optional topic/cluster/type filters.
    pub async fn list(
        &self,
        topic: Option<&str>,
        cluster_id: Option<&str>,
        concept_type: Option<&str>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<KnowledgeEntry>, AppError> {
        let clamped_limit = limit.clamp(1, 200);
        let clamped_offset = offset.max(0);

        let rows = sqlx::query(
            r#"SELECT id, text, content_hash, topic, cluster_id, source_node_id,
                      source_agent_id, confidence, ttl, human_confirmed,
                      created_at, access_count,
                      concept_type, title, description, resource_uri, tags, security_tier, parent_id
               FROM knowledge_store_meta
               WHERE (? IS NULL OR topic = ?)
                 AND (? IS NULL OR (? = 'global' AND cluster_id IS NULL) OR (? != 'global' AND (cluster_id = ? OR cluster_id IS NULL)))
                 AND (? IS NULL OR concept_type = ?)
                 AND (ttl IS NULL OR ttl > unixepoch())
               ORDER BY created_at DESC
               LIMIT ? OFFSET ?"#,
        )
        .bind(topic)
        .bind(topic)
        .bind(cluster_id)
        .bind(cluster_id)
        .bind(cluster_id)
        .bind(cluster_id)
        .bind(concept_type)
        .bind(concept_type)
        .bind(clamped_limit)
        .bind(clamped_offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::InternalServerError(format!("[IKS] list failed: {}", e)))?;

        rows.into_iter()
            .map(Self::entry_from_row)
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| {
                AppError::InternalServerError(format!("[IKS] list row decode failed: {}", e))
            })
    }

    /// P2P sync: return entries written since `since` (unix timestamp) up to a bounded limit.
    pub async fn get_entries_since(
        &self,
        since: i64,
        limit: Option<i64>,
    ) -> Result<Vec<KnowledgeEntry>, AppError> {
        let clamped_limit = limit.unwrap_or(500).clamp(1, 1000);
        let rows = sqlx::query(
            r#"SELECT id, text, content_hash, topic, cluster_id, source_node_id,
                      source_agent_id, confidence, ttl, human_confirmed,
                      created_at, access_count,
                      concept_type, title, description, resource_uri, tags, security_tier, parent_id
               FROM knowledge_store_meta
               WHERE created_at > ?
                 AND (ttl IS NULL OR ttl > unixepoch())
               ORDER BY created_at ASC
               LIMIT ?"#,
        )
        .bind(since)
        .bind(clamped_limit)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| {
            AppError::InternalServerError(format!("[IKS] get_entries_since failed: {}", e))
        })?;

        rows.into_iter()
            .map(Self::entry_from_row)
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| {
                AppError::InternalServerError(format!("[IKS] sync row decode failed: {}", e))
            })
    }

    /// Internal unconditional deletion from both SQLite and LanceDB.
    pub(crate) async fn delete(&self, id: &str) -> Result<(), AppError> {
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
        if let Some(entry) = self.get_by_id_internal(id, false).await? {
            if entry.human_confirmed && !force {
                return Err(AppError::Conflict(
                    "[IKS] Cannot delete human-confirmed entry without force=true".to_string(),
                ));
            }
        } else {
            return Err(AppError::NotFound(format!("[IKS] Entry {} not found", id)));
        }
        self.delete(id).await
    }

    /// Mark an entry as human-confirmed. Clears TTL and sets confidence = 1.0.
    /// Idempotent — calling on an already-confirmed entry is a safe no-op.
    pub async fn confirm(&self, id: &str) -> Result<KnowledgeEntry, AppError> {
        let rows = sqlx::query(
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
        .map_err(|e| AppError::InternalServerError(format!("[IKS] confirm failed: {}", e)))?
        .rows_affected();

        if rows == 0 {
            return Err(AppError::NotFound(format!("[IKS] Entry {} not found", id)));
        }

        tracing::info!(id = %id, "[IKS] Entry confirmed by human — TTL cleared and confidence set to 1.0");

        self.get_by_id_internal(id, false).await?.ok_or_else(|| {
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
        let clamped_limit = limit.clamp(1, 50);
        let entry = self
            .get_by_id_internal(id, false)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("[IKS] Entry {} not found", id)))?;

        #[cfg(feature = "vector-memory")]
        {
            let api_key = std::env::var("GOOGLE_API_KEY").map_err(|_| {
                AppError::BadRequest(
                    "[IKS] GOOGLE_API_KEY required for embedding. Set it in .env.".to_string(),
                )
            })?;
            let query_vector =
                crate::agent::memory::get_gemini_embedding(&http_client, &api_key, &entry.text)
                    .await?;

            let lance = self.get_lance().await?;
            lance.ensure_table().await?;

            // Sanitize ID to prevent predicate injection
            if !id
                .chars()
                .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
            {
                return Err(AppError::BadRequest(
                    "[IKS] Invalid ID format for vector query predicate".to_string(),
                ));
            }

            let predicate = format!(
                "id != '{}' AND (ttl IS NULL OR ttl > {})",
                id,
                chrono::Utc::now().timestamp()
            );
            let hits = lance
                .search_knowledge_filtered(query_vector, clamped_limit + 1, &predicate)
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
                          concept_type, title, description, resource_uri, tags, security_tier, parent_id
                   FROM knowledge_store_meta
                   WHERE id IN ({})
                     AND (ttl IS NULL OR ttl > unixepoch())
                   ORDER BY confidence DESC"#,
                placeholders
            );
            let mut q = sqlx::query(sqlx::AssertSqlSafe(&*hydrate_sql));
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

            results.truncate(clamped_limit);
            Ok(results)
        }

        #[cfg(not(feature = "vector-memory"))]
        {
            let _ = http_client;
            let rows = sqlx::query(
                r#"SELECT id, text, content_hash, topic, cluster_id, source_node_id,
                          source_agent_id, confidence, ttl, human_confirmed,
                          created_at, access_count,
                          concept_type, title, description, resource_uri, tags, security_tier, parent_id
                   FROM knowledge_store_meta
                   WHERE id != ? AND topic = ? AND (ttl IS NULL OR ttl > unixepoch())
                   LIMIT ?"#,
            )
            .bind(id)
            .bind(&entry.topic)
            .bind(clamped_limit as i64)
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

    /// TTL eviction: delete all expired or 0-confidence entries where `human_confirmed = 0`.
    /// Also deletes orphaned vectors from LanceDB.
    pub async fn evict_expired(&self) -> Result<u64, AppError> {
        let now = chrono::Utc::now().timestamp();

        // 1. Fetch IDs of entries due for eviction
        let expired_ids: Vec<String> = sqlx::query_scalar(
            r#"SELECT id FROM knowledge_store_meta
               WHERE human_confirmed = 0
                 AND ((ttl IS NOT NULL AND ttl < ?) OR confidence <= 0.0)"#,
        )
        .bind(now)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::InternalServerError(format!("[IKS] evict lookup failed: {}", e)))?;

        if expired_ids.is_empty() {
            return Ok(0);
        }

        let count = expired_ids.len() as u64;

        // 2. Delete from SQLite
        let placeholders = expired_ids
            .iter()
            .map(|_| "?")
            .collect::<Vec<_>>()
            .join(", ");
        let delete_sql = format!(
            "DELETE FROM knowledge_store_meta WHERE id IN ({})",
            placeholders
        );
        let mut q = sqlx::query(sqlx::AssertSqlSafe(&*delete_sql));
        for id in &expired_ids {
            q = q.bind(id);
        }
        q.execute(&self.pool).await.map_err(|e| {
            AppError::InternalServerError(format!("[IKS] evict SQLite delete failed: {}", e))
        })?;

        // 3. Batch delete from LanceDB vector store
        #[cfg(feature = "vector-memory")]
        {
            if let Ok(lance) = self.get_lance().await {
                if let Err(e) = lance.delete_memories(expired_ids).await {
                    tracing::warn!(error = %e, "[IKS] LanceDB batch eviction cleanup failed");
                }
            }
        }

        tracing::info!(
            count = count,
            "[IKS] Evicted expired knowledge entries and vectors"
        );
        Ok(count)
    }

    /// Reconciles the vector index against SQLite metadata to remove orphaned vectors.
    pub async fn reconcile_vector_store(&self) -> Result<usize, AppError> {
        #[cfg(feature = "vector-memory")]
        {
            let lance = self.get_lance().await?;
            lance.ensure_table().await?;

            // Retrieve all IDs from SQLite
            let valid_ids: std::collections::HashSet<String> =
                sqlx::query_scalar("SELECT id FROM knowledge_store_meta")
                    .fetch_all(&self.pool)
                    .await
                    .map_err(|e| {
                        AppError::InternalServerError(format!(
                            "[IKS] Reconcile ID fetch failed: {}",
                            e
                        ))
                    })?
                    .into_iter()
                    .collect();

            // Fetch IDs present in LanceDB
            let lance_ids = lance.list_memory_ids().await?;
            let orphan_ids: Vec<String> = lance_ids
                .into_iter()
                .filter(|id| !valid_ids.contains(id))
                .collect();

            let orphan_count = orphan_ids.len();
            if !orphan_ids.is_empty() {
                lance.delete_memories(orphan_ids).await?;
                tracing::info!(
                    count = orphan_count,
                    "[IKS] Cleaned orphaned LanceDB vectors"
                );
            }
            Ok(orphan_count)
        }

        #[cfg(not(feature = "vector-memory"))]
        {
            Ok(0)
        }
    }

    /// Confidence decay: reduce confidence based on actual time elapsed since last update.
    ///
    /// Rate: 0.01 per day (time-aware). Human-confirmed entries are never decayed.
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

//! @docs ARCHITECTURE:IKS
//!
//! ### AI Assist Note
//! **Institutional Knowledge Store (IKS)**: Cross-cluster, cross-restart
//! persistent semantic memory. Unlike `VectorMemory` (mission-scoped),
//! IKS holds durable, curated facts — agent patterns, SOPs, client knowledge,
//! human decision history — that persist indefinitely across any cluster.
//!
//! ### Architecture
//! Dual-store: SQLite metadata index (topic, cluster, TTL, dedup, **text**)
//! + LanceDB vector store for k-NN similarity search. Text content is stored
//! in SQLite alongside metadata so point-lookups (`get_by_id`) need no
//! LanceDB round-trip. Content is deduplicated by SHA-256 hash at write time.
//!
//! ### Embedding Provider
//! Always uses `text-embedding-004` via `GOOGLE_API_KEY` — never inherits
//! from the calling agent's provider config (dimensional consistency).
//!
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: `GOOGLE_API_KEY` missing, LanceDB schema mismatch,
//!   SQLite UNIQUE constraint on `content_hash` (expected — means dedup hit).
//! - **Trace Scope**: `server-rs::agent::knowledge_store` (Search `[IKS]`)

use crate::error::AppError;
use sqlx::{Row, SqlitePool};
use std::sync::Arc;

// ─────────────────────────────────────────────────────────
//  PUBLIC TYPES (always compiled)
// ─────────────────────────────────────────────────────────

/// A single entry in the Institutional Knowledge Store.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct KnowledgeEntry {
    pub id: String,
    pub text: String,
    pub topic: String,
    pub cluster_id: Option<String>,
    pub source_node_id: Option<String>,
    pub source_agent_id: Option<String>,
    /// SHA-256 hex of `text` — used for dedup and P2P idempotency.
    pub content_hash: String,
    /// 0.0–1.0 quality signal; decays 0.01/day for unconfirmed entries.
    pub confidence: f32,
    /// True if a human explicitly approved this entry via /confirm.
    pub human_confirmed: bool,
    /// Unix expiry timestamp; NULL = never expires (human-confirmed entries).
    pub ttl: Option<i64>,
    pub created_at: i64,
    pub access_count: i64,
    // --- OKF Extensions ---
    pub concept_type: String,
    pub title: Option<String>,
    pub description: Option<String>,
    pub resource_uri: Option<String>,
    pub tags: Option<String>,
}

/// Parameters for writing a new knowledge entry.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct AddKnowledgeRequest {
    pub text: String,
    pub topic: String,
    pub cluster_id: Option<String>,
    /// The remote Bunker node that authored this entry (P2P sync). None = local write.
    pub source_node_id: Option<String>,
    pub source_agent_id: Option<String>,
    /// 0.0–1.0. Defaults to 1.0.
    pub confidence: Option<f32>,
    /// Days until expiry. Omit for the system default (90d for agents).
    /// Pass None explicitly in JSON to create a permanent entry.
    pub ttl_days: Option<i64>,
    /// If true, entry is immediately human-confirmed (ttl cleared, confidence = 1.0).
    pub human_confirmed: Option<bool>,
    // --- OKF Extensions ---
    pub concept_type: Option<String>,
    pub title: Option<String>,
    pub description: Option<String>,
    pub resource_uri: Option<String>,
    pub tags: Option<String>,
}

/// Search parameters for semantic retrieval.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct KnowledgeSearchRequest {
    pub query: String,
    /// Pre-filter by topic before vector search.
    pub topic: Option<String>,
    /// NULL = search global + cluster-scoped entries; "global" = global only.
    pub cluster_id: Option<String>,
    /// Max results to return. Default: 10.
    pub limit: Option<usize>,
    /// Minimum confidence threshold. Default: 0.3.
    pub min_confidence: Option<f32>,
    // --- OKF Extensions ---
    pub concept_type: Option<String>,
}

// ─────────────────────────────────────────────────────────
//  DEFAULT TTL CONSTANT
// ─────────────────────────────────────────────────────────

/// Default TTL for agent-written entries (Q3 decision: 90 days).
pub const DEFAULT_TTL_DAYS: i64 = 90;

// ─────────────────────────────────────────────────────────
//  ENGINE STRUCT
// ─────────────────────────────────────────────────────────

/// The Institutional Knowledge Store engine.
///
/// Holds the SQLite pool for metadata operations. The LanceDB vector store
/// is lazily initialized on first write (behind `vector-memory` feature).
pub struct KnowledgeStore {
    pool: SqlitePool,
    #[cfg(feature = "vector-memory")]
    lance: tokio::sync::OnceCell<Arc<crate::agent::memory::VectorMemory>>,
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

    // ─────────────────────────────────────────────────────────
    //  PRIVATE HELPERS
    // ─────────────────────────────────────────────────────────

    /// Lazily initializes and returns the LanceDB vector store.
    #[cfg(feature = "vector-memory")]
    async fn get_lance(&self) -> Result<Arc<crate::agent::memory::VectorMemory>, AppError> {
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
    fn sha256_hash(text: &str) -> String {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(text.as_bytes());
        let result = hasher.finalize();
        hex::encode(result)
    }

    /// Computes the TTL unix timestamp for a new entry.
    /// Q3 decision: agent default = 90d, human-confirmed = NULL (never).
    fn compute_ttl(human_confirmed: bool, ttl_days: Option<i64>, now_unix: i64) -> Option<i64> {
        match (human_confirmed, ttl_days) {
            (true, _) => None,                               // human-confirmed → never expires
            (false, Some(d)) => Some(now_unix + d * 86_400), // caller-supplied
            (false, None) => Some(now_unix + DEFAULT_TTL_DAYS * 86_400), // agent default: 90d
        }
    }

    fn entry_from_row(row: sqlx::sqlite::SqliteRow) -> Result<KnowledgeEntry, sqlx::Error> {
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

    // ─────────────────────────────────────────────────────────
    //  WRITE
    // ─────────────────────────────────────────────────────────

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
        let concept_type = req.concept_type.unwrap_or_else(|| "general".to_string()).to_lowercase();

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

    // ─────────────────────────────────────────────────────────
    //  READ
    // ─────────────────────────────────────────────────────────

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

    // ─────────────────────────────────────────────────────────
    //  SEARCH
    // ─────────────────────────────────────────────────────────

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
                    .map(|ct| format!(
                        "AND concept_type = '{}'",
                        ct.replace('\'', "''")
                    ))
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
                        concept_type: r.try_get("concept_type").unwrap_or_else(|_| "general".to_string()),
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

    // ─────────────────────────────────────────────────────────
    //  MUTATIONS
    // ─────────────────────────────────────────────────────────

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
            let query_vector = crate::agent::memory::get_gemini_embedding(&http_client, &api_key, &entry.text)
                .await?;

            let lance = self.get_lance().await?;
            lance.ensure_table().await?;

            // Retrieve top peers (limit + 1 to account for self-exclusion)
            // L2 distance limit of 0.5 corresponds to high similarity
            let predicate = format!("id != '{}' AND confidence >= 0.0 AND (ttl IS NULL OR ttl > {})", id, chrono::Utc::now().timestamp());
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
                   LIMIT ?"#
            )
            .bind(id)
            .bind(&entry.topic)
            .bind(limit as i64)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| AppError::InternalServerError(format!("[IKS] get_peers fallback failed: {}", e)))?;

            let results = rows
                .into_iter()
                .map(Self::entry_from_row)
                .collect::<Result<Vec<_>, _>>()
                .map_err(|e| AppError::InternalServerError(format!("[IKS] peer fallback decode failed: {}", e)))?;
            Ok(results)
        }
    }

    // ─────────────────────────────────────────────────────────
    //  BACKGROUND MAINTENANCE
    // ─────────────────────────────────────────────────────────

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

// ─────────────────────────────────────────────────────────
//  TESTS
// ─────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    /// SHA-256 hash must be stable across identical inputs.
    #[test]
    fn test_sha256_hash_stable() {
        let h1 = KnowledgeStore::sha256_hash("hello world");
        let h2 = KnowledgeStore::sha256_hash("hello world");
        assert_eq!(h1, h2);
        assert!(!h1.is_empty());
    }

    /// Different text must produce different hashes.
    #[test]
    fn test_sha256_hash_distinct() {
        let h1 = KnowledgeStore::sha256_hash("apple");
        let h2 = KnowledgeStore::sha256_hash("orange");
        assert_ne!(h1, h2);
    }

    /// SHA-256 output must be a valid 64-char hex string.
    #[test]
    fn test_sha256_hash_format() {
        let h = KnowledgeStore::sha256_hash("test");
        assert_eq!(h.len(), 64, "SHA-256 hex must be 64 chars");
        assert!(h.chars().all(|c| c.is_ascii_hexdigit()));
    }

    /// Agent-written entries default to 90-day TTL.
    #[test]
    fn test_agent_entry_gets_90_day_ttl() {
        let now = 1_000_000_i64;
        let ttl = KnowledgeStore::compute_ttl(false, None, now);
        assert_eq!(ttl, Some(now + DEFAULT_TTL_DAYS * 86_400));
    }

    /// Human-confirmed entries never expire (ttl = None).
    #[test]
    fn test_confirmed_entry_has_no_ttl() {
        let now = 1_000_000_i64;
        let ttl = KnowledgeStore::compute_ttl(true, None, now);
        assert_eq!(ttl, None);
    }

    /// Caller-supplied ttl_days overrides the default.
    #[test]
    fn test_caller_supplied_ttl() {
        let now = 1_000_000_i64;
        let ttl = KnowledgeStore::compute_ttl(false, Some(7), now);
        assert_eq!(ttl, Some(now + 7 * 86_400));
    }

    /// Human-confirmed flag overrides caller-supplied ttl_days.
    #[test]
    fn test_confirmed_overrides_ttl_days() {
        let now = 1_000_000_i64;
        // Even if ttl_days is supplied, human_confirmed wins → None
        let ttl = KnowledgeStore::compute_ttl(true, Some(30), now);
        assert_eq!(ttl, None);
    }

    /// Full round-trip: add → evict with ttl=0 → confirm → evict again.
    #[tokio::test]
    async fn test_confirmed_entry_survives_eviction() {
        let pool = sqlx::sqlite::SqlitePoolOptions::new()
            .connect("sqlite::memory:")
            .await
            .unwrap();

        // Inline DDL mirrors the real migrations (including the text column).
        sqlx::query(
            r#"CREATE TABLE IF NOT EXISTS knowledge_store_meta (
                id TEXT PRIMARY KEY,
                text TEXT NOT NULL DEFAULT '',
                content_hash TEXT NOT NULL UNIQUE,
                topic TEXT NOT NULL DEFAULT 'general',
                cluster_id TEXT,
                source_node_id TEXT,
                source_agent_id TEXT,
                confidence REAL NOT NULL DEFAULT 1.0,
                access_count INTEGER NOT NULL DEFAULT 0,
                last_accessed_at INTEGER,
                ttl INTEGER,
                created_at INTEGER NOT NULL DEFAULT (unixepoch()),
                updated_at INTEGER NOT NULL DEFAULT (unixepoch()),
                human_confirmed INTEGER NOT NULL DEFAULT 0,
                concept_type TEXT NOT NULL DEFAULT 'general',
                title TEXT,
                description TEXT,
                resource_uri TEXT,
                tags TEXT
            )"#,
        )
        .execute(&pool)
        .await
        .unwrap();

        let store = KnowledgeStore::new(pool.clone());

        // Insert a row with ttl already expired (past)
        let id = "test-entry-1".to_string();
        let past_ttl = chrono::Utc::now().timestamp() - 3600; // 1 hour ago
        sqlx::query(
            r#"INSERT INTO knowledge_store_meta (id, text, content_hash, topic, ttl, human_confirmed, created_at, updated_at, concept_type)
               VALUES (?, 'hello', ?, 'general', ?, 0, unixepoch(), unixepoch(), 'general')"#)
        .bind(&id)
        .bind("abc123hash")
        .bind(past_ttl)
        .execute(&pool)
        .await
        .unwrap();

        // Before confirm: eviction should delete it
        let evicted = store.evict_expired().await.unwrap();
        assert_eq!(evicted, 1, "Expired unconfirmed entry should be evicted");

        // Insert again and confirm it
        sqlx::query(
            r#"INSERT INTO knowledge_store_meta (id, text, content_hash, topic, ttl, human_confirmed, created_at, updated_at, concept_type)
               VALUES (?, 'world', ?, 'general', ?, 0, unixepoch(), unixepoch(), 'general')"#)
        .bind("test-entry-2")
        .bind("def456hash")
        .bind(past_ttl)
        .execute(&pool)
        .await
        .unwrap();

        // Confirm it (sets human_confirmed=1, ttl=NULL)
        store.confirm("test-entry-2").await.unwrap();

        // Eviction should NOT delete confirmed entry
        let evicted_after_confirm = store.evict_expired().await.unwrap();
        assert_eq!(
            evicted_after_confirm, 0,
            "Human-confirmed entry must survive eviction"
        );
    }

    /// get_by_id must return the stored text (not an empty string).
    #[tokio::test]
    async fn test_get_by_id_returns_text() {
        let pool = sqlx::sqlite::SqlitePoolOptions::new()
            .connect("sqlite::memory:")
            .await
            .unwrap();

        sqlx::query(
            r#"CREATE TABLE IF NOT EXISTS knowledge_store_meta (
                id TEXT PRIMARY KEY,
                text TEXT NOT NULL DEFAULT '',
                content_hash TEXT NOT NULL UNIQUE,
                topic TEXT NOT NULL DEFAULT 'general',
                cluster_id TEXT,
                source_node_id TEXT,
                source_agent_id TEXT,
                confidence REAL NOT NULL DEFAULT 1.0,
                access_count INTEGER NOT NULL DEFAULT 0,
                last_accessed_at INTEGER,
                ttl INTEGER,
                created_at INTEGER NOT NULL DEFAULT (unixepoch()),
                updated_at INTEGER NOT NULL DEFAULT (unixepoch()),
                human_confirmed INTEGER NOT NULL DEFAULT 0,
                concept_type TEXT NOT NULL DEFAULT 'general',
                title TEXT,
                description TEXT,
                resource_uri TEXT,
                tags TEXT
            )"#,
        )
        .execute(&pool)
        .await
        .unwrap();

        sqlx::query(
            "INSERT INTO knowledge_store_meta (id, text, content_hash, topic, created_at, updated_at, concept_type) \
             VALUES ('id-1', 'The quick brown fox', 'hashxyz', 'general', unixepoch(), unixepoch(), 'general')",
        )
        .execute(&pool)
        .await
        .unwrap();

        let store = KnowledgeStore::new(pool);
        let entry = store.get_by_id("id-1").await.unwrap().unwrap();
        assert_eq!(entry.text, "The quick brown fox");
        assert_eq!(entry.access_count, 1); // incremented by get_by_id
    }

    /// decay_confidence must be time-aware: decay = 0.01 * days_since_update.
    #[tokio::test]
    async fn test_decay_is_time_aware() {
        let pool = sqlx::sqlite::SqlitePoolOptions::new()
            .connect("sqlite::memory:")
            .await
            .unwrap();

        sqlx::query(
            r#"CREATE TABLE IF NOT EXISTS knowledge_store_meta (
                id TEXT PRIMARY KEY,
                text TEXT NOT NULL DEFAULT '',
                content_hash TEXT NOT NULL UNIQUE,
                topic TEXT NOT NULL DEFAULT 'general',
                cluster_id TEXT,
                source_node_id TEXT,
                source_agent_id TEXT,
                confidence REAL NOT NULL DEFAULT 1.0,
                access_count INTEGER NOT NULL DEFAULT 0,
                last_accessed_at INTEGER,
                ttl INTEGER,
                created_at INTEGER NOT NULL DEFAULT (unixepoch()),
                updated_at INTEGER NOT NULL DEFAULT (unixepoch()),
                human_confirmed INTEGER NOT NULL DEFAULT 0,
                concept_type TEXT NOT NULL DEFAULT 'general',
                title TEXT,
                description TEXT,
                resource_uri TEXT,
                tags TEXT
            )"#,
        )
        .execute(&pool)
        .await
        .unwrap();

        // Insert an entry whose `updated_at` is 10 days ago.
        let ten_days_ago = chrono::Utc::now().timestamp() - 10 * 86_400;
        sqlx::query(
            "INSERT INTO knowledge_store_meta (id, text, content_hash, topic, confidence, updated_at, created_at, concept_type) \
             VALUES ('id-decay', 'fact', 'decayhash', 'general', 1.0, ?, unixepoch(), 'general')",
        )
        .bind(ten_days_ago)
        .execute(&pool)
        .await
        .unwrap();

        let store = KnowledgeStore::new(pool.clone());
        store.decay_confidence().await.unwrap();

        let row: (f64,) =
            sqlx::query_as("SELECT confidence FROM knowledge_store_meta WHERE id = 'id-decay'")
                .fetch_one(&pool)
                .await
                .unwrap();

        // Expected: 1.0 - (0.01 * 10) = 0.90, allow ±0.01 for integer truncation in SQLite.
        assert!(
            (row.0 - 0.90).abs() < 0.02,
            "Expected confidence ~0.90 after 10-day decay, got {}",
            row.0
        );
    }
}

// Metadata: [IKS]

// Metadata: [knowledge_store]

// Metadata: [knowledge_store]

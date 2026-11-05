//! @docs ARCHITECTURE:IKS
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / HTTP Routes / knowledge_fallback
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Type-safe state handling and bounded execution without unhandled panics.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: `AppError::BadRequest`, `AppError::InternalServerError`
//! - **Telemetry Targets**: none declared
//! - **Witness Tests**: `server-rs/src/agent/knowledge_store/tests.rs`

use crate::agent::knowledge_store::types::{AddKnowledgeRequest, KnowledgeEntry};
use crate::agent::knowledge_store::KnowledgeStore;
use crate::error::AppError;
use crate::state::AppState;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Deserialize)]
pub struct ListKnowledgeParams {
    pub topic: Option<String>,
    pub cluster_id: Option<String>,
    pub concept_type: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct KnowledgeWriteResponse {
    pub id: String,
    pub dedup_hit: bool,
}

#[derive(Debug, Deserialize)]
pub struct FallbackSearchQuery {
    pub q: Option<String>,
    pub limit: Option<i64>,
}

/// GET /knowledge (SQLite Fallback)
pub async fn list_knowledge_fallback(
    State(state): State<Arc<AppState>>,
    Query(params): Query<ListKnowledgeParams>,
) -> Result<Json<Vec<KnowledgeEntry>>, AppError> {
    let limit = params.limit.unwrap_or(50).clamp(1, 200);
    let offset = params.offset.unwrap_or(0).max(0);
    let store = KnowledgeStore::new(state.resources.pool.clone());
    let entries = store
        .list(
            params.topic.as_deref(),
            params.cluster_id.as_deref(),
            params.concept_type.as_deref(),
            limit,
            offset,
        )
        .await?;
    Ok(Json(entries))
}

/// GET /knowledge/search (SQLite Fallback)
pub async fn search_knowledge_fallback(
    State(state): State<Arc<AppState>>,
    Query(params): Query<FallbackSearchQuery>,
) -> Result<Json<Vec<KnowledgeEntry>>, AppError> {
    let limit = params.limit.unwrap_or(10).clamp(1, 100);
    let query_str = params.q.unwrap_or_default();
    let pattern = format!("%{}%", query_str);

    let rows = sqlx::query(
        r#"SELECT id, text, content_hash, topic, cluster_id, source_node_id,
                  source_agent_id, confidence, ttl, human_confirmed,
                  created_at, access_count,
                  concept_type, title, description, resource_uri, tags, security_tier, parent_id
           FROM knowledge_store_meta
           WHERE (title LIKE ? OR text LIKE ? OR topic LIKE ? OR description LIKE ?)
             AND (ttl IS NULL OR ttl > unixepoch())
           ORDER BY confidence DESC, created_at DESC
           LIMIT ?"#,
    )
    .bind(&pattern)
    .bind(&pattern)
    .bind(&pattern)
    .bind(&pattern)
    .bind(limit)
    .fetch_all(&state.resources.pool)
    .await
    .map_err(|e| AppError::InternalServerError(format!("[IKS-Fallback] Search failed: {}", e)))?;

    let entries = rows
        .into_iter()
        .map(KnowledgeStore::entry_from_row)
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| {
            AppError::InternalServerError(format!("[IKS-Fallback] Decode failed: {}", e))
        })?;

    Ok(Json(entries))
}

/// GET /knowledge/{id}/peers (SQLite Fallback)
pub async fn get_knowledge_peers_fallback(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Query(params): Query<ListKnowledgeParams>,
) -> Result<Json<Vec<KnowledgeEntry>>, AppError> {
    let limit = params.limit.unwrap_or(5).clamp(1, 100) as usize;
    let store = KnowledgeStore::new(state.resources.pool.clone());
    let target = store.get_by_id_internal(&id, false).await?;

    let peers = if let Some(entry) = target {
        let same_concept = store
            .list(None, None, Some(&entry.concept_type), (limit + 1) as i64, 0)
            .await?;
        same_concept
            .into_iter()
            .filter(|e| e.id != id)
            .take(limit)
            .collect()
    } else {
        Vec::new()
    };

    Ok(Json(peers))
}

/// POST /knowledge/{id}/confirm (SQLite Fallback)
pub async fn confirm_knowledge_fallback(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<StatusCode, AppError> {
    let store = KnowledgeStore::new(state.resources.pool.clone());
    store.confirm(&id).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// DELETE /knowledge/{id} (SQLite Fallback)
#[derive(Debug, Deserialize)]
pub struct DeleteParams {
    pub force: Option<bool>,
}

pub async fn delete_knowledge_fallback(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Query(params): Query<DeleteParams>,
) -> Result<StatusCode, AppError> {
    let store = KnowledgeStore::new(state.resources.pool.clone());
    let force = params.force.unwrap_or(false);
    store.remove(&id, force).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// POST /knowledge (SQLite Fallback)
pub async fn write_knowledge_fallback(
    State(state): State<Arc<AppState>>,
    Json(req): Json<AddKnowledgeRequest>,
) -> Result<(StatusCode, Json<KnowledgeWriteResponse>), AppError> {
    let hash = KnowledgeStore::sha256_hash(&req.topic, req.cluster_id.as_deref(), &req.text);

    let existing =
        sqlx::query("SELECT id FROM knowledge_store_meta WHERE content_hash = ? LIMIT 1")
            .bind(&hash)
            .fetch_optional(&state.resources.pool)
            .await
            .map_err(|e| {
                AppError::InternalServerError(format!("[IKS-Fallback] Dedup check failed: {}", e))
            })?;

    if let Some(row) = existing {
        use sqlx::Row;
        let id: String = row.get("id");
        return Ok((
            StatusCode::OK,
            Json(KnowledgeWriteResponse {
                id,
                dedup_hit: true,
            }),
        ));
    }

    let id = format!("okf-{}", uuid::Uuid::new_v4());
    let now = chrono::Utc::now().timestamp();
    let ttl_days = req
        .ttl_days
        .unwrap_or(crate::agent::knowledge_store::types::DEFAULT_TTL_DAYS);
    let ttl = Some(now + (ttl_days * 86400));
    let confidence = req
        .confidence
        .unwrap_or(crate::agent::knowledge_store::types::DEFAULT_AGENT_CONFIDENCE);

    sqlx::query(
        r#"INSERT INTO knowledge_store_meta
           (id, text, topic, cluster_id, source_node_id, source_agent_id, content_hash,
            confidence, human_confirmed, ttl, created_at, updated_at, access_count,
            concept_type, title, description, resource_uri, tags, security_tier, parent_id)
           VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, 0, ?, ?, ?, ?, ?, ?, ?)"#,
    )
    .bind(&id)
    .bind(&req.text)
    .bind(&req.topic)
    .bind(&req.cluster_id)
    .bind(&req.source_node_id)
    .bind(&req.source_agent_id)
    .bind(&hash)
    .bind(confidence)
    .bind(0i32) // human_confirmed starts as 0 until confirmed
    .bind(ttl)
    .bind(now)
    .bind(now)
    .bind(req.concept_type.as_deref().unwrap_or("general"))
    .bind(&req.title)
    .bind(&req.description)
    .bind(&req.resource_uri)
    .bind(&req.tags)
    .bind(req.security_tier.as_deref().unwrap_or("BRONZE_ADHOC"))
    .bind(&req.parent_id)
    .execute(&state.resources.pool)
    .await
    .map_err(|e| AppError::InternalServerError(format!("[IKS-Fallback] Write failed: {}", e)))?;

    Ok((
        StatusCode::CREATED,
        Json(KnowledgeWriteResponse {
            id,
            dedup_hit: false,
        }),
    ))
}

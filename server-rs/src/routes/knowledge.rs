//! @docs ARCHITECTURE:IKS
//!
//! ### AI Assist Note
//! **IKS REST API**: CRUD + semantic search surface for the Institutional
//! Knowledge Store. All routes are auth-gated via the standard NEURAL_TOKEN
//! middleware applied at the router level.
//!
//! ### Endpoints
//! | Method | Path                        | Description                          |
//! |--------|-----------------------------|--------------------------------------|
//! | POST   | /knowledge                  | Write a new entry (dedup by hash)    |
//! | GET    | /knowledge                  | List entries (topic/cluster filter)  |
//! | GET    | /knowledge/search           | Semantic k-NN search                 |
//! | POST   | /knowledge/{id}/confirm     | Human-confirm an entry               |
//! | DELETE | /knowledge/{id}             | Remove an entry by ID                |
//!
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: `GOOGLE_API_KEY` missing (embedding), LanceDB lock, or
//!   SQLite UNIQUE constraint hit on dedup (returns 200, not 409).
//! - **Trace Scope**: `server-rs::routes::knowledge` (Search `[IKS]`)

use crate::agent::knowledge_store::{AddKnowledgeRequest, KnowledgeSearchRequest};
use crate::error::AppError;
use crate::state::AppState;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

// ─────────────────────────────────────────────────────────
//  Request / Response DTOs
// ─────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct ListKnowledgeParams {
    pub topic: Option<String>,
    pub cluster_id: Option<String>,
    /// Max results (default 50, capped at 200)
    pub limit: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct KnowledgeWriteResponse {
    pub id: String,
    pub dedup_hit: bool,
}

// ─────────────────────────────────────────────────────────
//  Handlers
// ─────────────────────────────────────────────────────────

/// POST /knowledge
/// Writes a new entry to the IKS. Idempotent: duplicate text returns the
/// existing entry's ID with `dedup_hit: true`.
pub async fn write_knowledge(
    State(state): State<Arc<AppState>>,
    Json(req): Json<AddKnowledgeRequest>,
) -> Result<(StatusCode, Json<KnowledgeWriteResponse>), AppError> {
    let ks = state.resources.get_knowledge_store().await?;
    
    // Detect dedup hit before insert
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(req.text.as_bytes());
    let hash = hex::encode(hasher.finalize());
    
    let existing = sqlx::query!("SELECT id FROM knowledge_store_meta WHERE content_hash = ? LIMIT 1", hash)
        .fetch_optional(&state.resources.pool)
        .await
        .map_err(|e| AppError::InternalServerError(format!("[IKS] Dedup pre-check failed: {}", e)))?;
        
    let dedup_hit = existing.is_some();
    let entry = ks.add_entry(req, &state.resources.http_client).await?;
    
    Ok((
        StatusCode::CREATED,
        Json(KnowledgeWriteResponse { id: entry.id, dedup_hit }),
    ))
}

/// GET /knowledge
/// Lists entries with optional topic/cluster filter.
pub async fn list_knowledge(
    State(state): State<Arc<AppState>>,
    Query(params): Query<ListKnowledgeParams>,
) -> Result<Json<Vec<crate::agent::knowledge_store::KnowledgeEntry>>, AppError> {
    let ks = state.resources.get_knowledge_store().await?;
    let limit = params.limit.unwrap_or(50).min(200);
    let entries = ks
        .list(params.topic.as_deref(), params.cluster_id.as_deref(), limit, 0)
        .await?;
    Ok(Json(entries))
}

/// GET /knowledge/search?q=...&limit=5
/// Semantic k-NN search across all IKS entries.
pub async fn search_knowledge(
    State(state): State<Arc<AppState>>,
    Query(params): Query<KnowledgeSearchRequest>,
) -> Result<Json<Vec<crate::agent::knowledge_store::KnowledgeEntry>>, AppError> {
    let ks = state.resources.get_knowledge_store().await?;
    let results = ks.search(&params, &state.resources.http_client).await?;
    Ok(Json(results))
}

/// POST /knowledge/{id}/confirm
/// Human-confirms an entry: clears TTL, sets confidence to 1.0, sets
/// `human_confirmed = true`. Protected against accidental eviction.
pub async fn confirm_knowledge(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<StatusCode, AppError> {
    let ks = state.resources.get_knowledge_store().await?;
    ks.confirm(&id).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// DELETE /knowledge/{id}
/// Removes an entry by ID. Refuses to delete human-confirmed entries
/// (returns 409 Conflict) — use `?force=true` to override.
#[derive(Debug, Deserialize)]
pub struct DeleteParams {
    pub force: Option<bool>,
}

pub async fn delete_knowledge(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Query(params): Query<DeleteParams>,
) -> Result<StatusCode, AppError> {
    let ks = state.resources.get_knowledge_store().await?;
    let force = params.force.unwrap_or(false);
    ks.remove(&id, force).await?;
    Ok(StatusCode::NO_CONTENT)
}

// Metadata: [knowledge]

// Metadata: [knowledge]

//! @docs ARCHITECTURE:IKS
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / HTTP Routes / knowledge
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Type-safe state handling and bounded execution without unhandled panics.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: `AppError::BadRequest`, `AppError::InternalServerError`, `AppError::Conflict`
//! - **Telemetry Targets**: none declared
//! - **Witness Tests**: `knowledge::tests::test_knowledge_pagination_clamping`

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
    pub concept_type: Option<String>,
    /// Max results (default 50, clamped to 1..=200)
    pub limit: Option<i64>,
    pub offset: Option<i64>,
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
/// existing entry's ID with `dedup_hit: true` and HTTP 200 OK.
#[tracing::instrument(skip(state, req), name = "knowledge::write")]
pub async fn write_knowledge(
    State(state): State<Arc<AppState>>,
    Json(req): Json<AddKnowledgeRequest>,
) -> Result<(StatusCode, Json<KnowledgeWriteResponse>), AppError> {
    let ks = state.resources.get_knowledge_store().await?;

    // Detect dedup hit before insert
    let hash = crate::agent::knowledge_store::KnowledgeStore::sha256_hash(
        &req.topic,
        req.cluster_id.as_deref(),
        &req.text,
    );

    let existing =
        sqlx::query("SELECT id FROM knowledge_store_meta WHERE content_hash = ? LIMIT 1")
            .bind(hash)
            .fetch_optional(&state.resources.pool)
            .await
            .map_err(|e| {
                AppError::InternalServerError(format!("[IKS] Dedup pre-check failed: {}", e))
            })?;

    let dedup_hit = existing.is_some();
    let entry = ks
        .add_entry(req, (*state.resources.http_client).clone())
        .await?;

    let status = if dedup_hit {
        StatusCode::OK
    } else {
        StatusCode::CREATED
    };

    Ok((
        status,
        Json(KnowledgeWriteResponse {
            id: entry.id,
            dedup_hit,
        }),
    ))
}

/// GET /knowledge
/// Lists entries with optional topic/cluster/type filters and pagination.
#[tracing::instrument(skip(state), name = "knowledge::list")]
pub async fn list_knowledge(
    State(state): State<Arc<AppState>>,
    Query(params): Query<ListKnowledgeParams>,
) -> Result<Json<Vec<crate::agent::knowledge_store::KnowledgeEntry>>, AppError> {
    let ks = state.resources.get_knowledge_store().await?;
    let limit = params.limit.unwrap_or(50).clamp(1, 200);
    let offset = params.offset.unwrap_or(0).max(0);
    let entries = ks
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

/// GET /knowledge/search?q=...&limit=5
/// Semantic k-NN search across all IKS entries.
#[tracing::instrument(skip(state), name = "knowledge::search")]
pub async fn search_knowledge(
    State(state): State<Arc<AppState>>,
    Query(mut params): Query<KnowledgeSearchRequest>,
) -> Result<Json<Vec<crate::agent::knowledge_store::KnowledgeEntry>>, AppError> {
    if params.limit == 0 {
        params.limit = 5;
    } else {
        params.limit = params.limit.min(50);
    }

    let ks = state.resources.get_knowledge_store().await?;
    let results = ks
        .search(&params, (*state.resources.http_client).clone())
        .await?;
    Ok(Json(results))
}

/// POST /knowledge/{id}/confirm
/// Human-confirms an entry: clears TTL, sets confidence to 1.0, sets
/// `human_confirmed = true`. Protected against accidental eviction.
#[tracing::instrument(skip(state), name = "knowledge::confirm")]
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

#[tracing::instrument(skip(state), name = "knowledge::delete")]
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

/// GET /knowledge/{id}/peers
/// Retrieves semantic peer nodes for a specific OKF knowledge entry.
#[tracing::instrument(skip(state), name = "knowledge::get_peers")]
pub async fn get_knowledge_peers(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Query(params): Query<ListKnowledgeParams>,
) -> Result<Json<Vec<crate::agent::knowledge_store::KnowledgeEntry>>, AppError> {
    let ks = state.resources.get_knowledge_store().await?;
    let limit = params.limit.unwrap_or(5).clamp(1, 100) as usize;
    let peers = ks
        .get_semantic_peers(&id, limit, (*state.resources.http_client).clone())
        .await?;
    Ok(Json(peers))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_knowledge_pagination_clamping() {
        let neg_limit = Some(-10i64);
        let clamped_limit = neg_limit.unwrap_or(50).clamp(1, 200);
        assert_eq!(clamped_limit, 1);

        let over_limit = Some(500i64);
        let clamped_over = over_limit.unwrap_or(50).clamp(1, 200);
        assert_eq!(clamped_over, 200);

        let neg_offset = Some(-5i64);
        let clamped_offset = neg_offset.unwrap_or(0).max(0);
        assert_eq!(clamped_offset, 0);
    }
}

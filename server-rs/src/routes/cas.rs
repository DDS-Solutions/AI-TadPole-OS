//! @docs ARCHITECTURE:Networking
//!
//! ### AI Assist Note
//! **CAS Versioning REST API Routes**
//! Handlers for querying file revision histories and triggering 1-click restorations.
//!
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: Invalid workspace path, un-tracked file, or revision mismatch.
//! - **Telemetry Link**: Search `[cas_routes]` in tracing logs.

use crate::{
    error::AppError,
    services::cas::{self, RevisionSummary},
    state::AppState,
};
use axum::{
    extract::{Query, State},
    Json,
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Debug, Deserialize)]
pub struct HistoryQuery {
    pub workspace_root: Option<String>,
    pub file_path: String,
}

#[derive(Debug, Deserialize)]
pub struct RestoreRequest {
    pub workspace_root: Option<String>,
    pub file_path: String,
    pub version_num: i64,
}

#[derive(Debug, Serialize)]
pub struct CasResponse<T> {
    pub success: bool,
    pub data: T,
}

/// GET /api/v1/cas/history
/// Retrieves revision history for a specific workspace file.
pub async fn get_cas_history_handler(
    State(state): State<Arc<AppState>>,
    Query(query): Query<HistoryQuery>,
) -> Result<Json<CasResponse<Vec<RevisionSummary>>>, AppError> {
    let ws_root = query
        .workspace_root
        .map(PathBuf::from)
        .unwrap_or_else(|| state.base_dir.clone());

    let pool = &state.resources.pool;
    let history = cas::get_file_history(pool, &ws_root, &query.file_path).await?;

    Ok(Json(CasResponse {
        success: true,
        data: history,
    }))
}

/// POST /api/v1/cas/restore
/// Restores a file to a specific revision version number.
pub async fn restore_cas_version_handler(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<RestoreRequest>,
) -> Result<Json<CasResponse<RevisionSummary>>, AppError> {
    let ws_root = payload
        .workspace_root
        .map(PathBuf::from)
        .unwrap_or_else(|| state.base_dir.clone());

    let pool = &state.resources.pool;
    let summary =
        cas::restore_file_version(pool, &ws_root, &payload.file_path, payload.version_num).await?;

    Ok(Json(CasResponse {
        success: true,
        data: summary,
    }))
}

// Metadata: [cas_routes]

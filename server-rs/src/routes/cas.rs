//! @docs ARCHITECTURE:Networking
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / HTTP Routes / cas
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Type-safe state handling and bounded execution without unhandled panics.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: `AppError::NotFound`, `AppError::BadRequest`, `AppError::InternalServerError`
//! - **Telemetry Targets**: none declared
//! - **Witness Tests**: `cas::tests::test_cas_path_traversal_rejected`, `cas::tests::test_cas_negative_version_rejected`

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
use serde_json::json;
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

fn resolve_ws_root(state: &AppState, ws: Option<&str>) -> Result<PathBuf, AppError> {
    match ws {
        Some(w) => {
            let safe_p = crate::utils::security::validate_path(&state.base_dir, w)?;
            Ok(safe_p.to_path_buf())
        }
        None => Ok(state.base_dir.clone()),
    }
}

/// GET /api/v1/cas/history
/// Retrieves revision history for a specific workspace file.
#[tracing::instrument(skip(state), name = "cas::get_history")]
pub async fn get_cas_history_handler(
    State(state): State<Arc<AppState>>,
    Query(query): Query<HistoryQuery>,
) -> Result<Json<CasResponse<Vec<RevisionSummary>>>, AppError> {
    let ws_root = resolve_ws_root(&state, query.workspace_root.as_deref())?;

    // 🛡️ [Path Traversal Defense] Validate file_path relative to ws_root
    let safe_file = crate::utils::security::validate_path(&ws_root, &query.file_path)?;
    let rel_file = safe_file
        .strip_prefix(&ws_root)
        .unwrap_or(safe_file.as_path())
        .to_string_lossy()
        .replace('\\', "/");

    let pool = &state.resources.pool;
    let history = cas::get_file_history(pool, &ws_root, &rel_file).await?;

    Ok(Json(CasResponse {
        success: true,
        data: history,
    }))
}

/// POST /api/v1/cas/restore
/// Restores a file to a specific revision version number.
#[tracing::instrument(skip(state, payload), name = "cas::restore_version")]
pub async fn restore_cas_version_handler(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<RestoreRequest>,
) -> Result<Json<CasResponse<RevisionSummary>>, AppError> {
    if payload.version_num <= 0 {
        return Err(AppError::BadRequest(
            "version_num must be a positive integer".to_string(),
        ));
    }

    let ws_root = resolve_ws_root(&state, payload.workspace_root.as_deref())?;

    // 🛡️ [Path Traversal Defense] Validate file_path relative to ws_root
    let safe_file = crate::utils::security::validate_path(&ws_root, &payload.file_path)?;
    let rel_file = safe_file
        .strip_prefix(&ws_root)
        .unwrap_or(safe_file.as_path())
        .to_string_lossy()
        .replace('\\', "/");

    let pool = &state.resources.pool;
    let summary = cas::restore_file_version(pool, &ws_root, &rel_file, payload.version_num).await?;

    state.emit_event(json!({
        "type": "cas:file_restored",
        "file_path": rel_file,
        "version_num": payload.version_num,
        "restored_version": summary.version_num,
        "hash": summary.hash,
        "timestamp": chrono::Utc::now().to_rfc3339()
    }));

    tracing::info!(
        "🔄 [CAS] Restored '{}' to v{} (new revision v{})",
        rel_file,
        payload.version_num,
        summary.version_num
    );

    Ok(Json(CasResponse {
        success: true,
        data: summary,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cas_path_traversal_rejected() {
        let temp = tempfile::tempdir().unwrap();
        let base_dir = temp.path().to_path_buf();

        let malicious = "../../etc/cron.d/malicious";
        assert!(crate::utils::security::validate_path(&base_dir, malicious).is_err());
    }

    #[tokio::test]
    async fn test_cas_negative_version_rejected() {
        let state = Arc::new(AppState::new_minimal_mock().await);
        let payload = RestoreRequest {
            workspace_root: None,
            file_path: "src/main.rs".to_string(),
            version_num: -1,
        };

        let res = restore_cas_version_handler(State(state), Json(payload)).await;
        assert!(res.is_err());
        assert!(res
            .unwrap_err()
            .to_string()
            .contains("version_num must be a positive integer"));
    }
}

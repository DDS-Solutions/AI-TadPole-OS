//! @docs ARCHITECTURE:Networking
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / HTTP Routes / missions / pins
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - [Structural] Deterministic internal state integrity and strict interface contract compliance.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: AppError::BadRequest, AppError::NotFound, AppError::Sqlx
//! - **Telemetry Targets**: none declared

use crate::error::AppError;
use crate::state::AppState;
use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
use serde_json::json;
use std::sync::Arc;

/// POST /v1/missions/:id/pin
/// Toggles the `is_pinned` retention flag for a mission atomically.
#[tracing::instrument(skip(state, headers), name = "missions::toggle_mission_pin")]
pub async fn toggle_mission_pin(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let pool = &state.resources.pool;
    let mission_id = id.trim();
    if mission_id.is_empty() {
        return Err(AppError::BadRequest(
            "Mission ID cannot be empty".to_string(),
        ));
    }

    // S-06 & Defect #11: Atomic update without corrupting updated_at
    let updated: Option<(i64,)> = sqlx::query_as(
        r#"UPDATE mission_history 
        SET is_pinned = CASE WHEN COALESCE(is_pinned, 0) = 0 THEN 1 ELSE 0 END 
        WHERE id = ? 
        RETURNING is_pinned"#,
    )
    .bind(mission_id)
    .fetch_optional(pool)
    .await?;

    let (is_pinned_val,) = match updated {
        Some(v) => v,
        None => {
            return Err(AppError::NotFound(format!(
                "Mission not found: {}",
                mission_id
            )))
        }
    };

    let new_pinned = is_pinned_val == 1;
    // Defect #30: Extract actor for audit trail and broadcast
    let actor = headers
        .get("X-Actor-Id")
        .or_else(|| headers.get("X-Principal"))
        .and_then(|v| v.to_str().ok())
        .unwrap_or("operator");

    state.emit_event(serde_json::json!({
        "type": "mission_update",
        "mission_id": mission_id,
        "action": "pin_toggled",
        "is_pinned": new_pinned,
        "actor": actor,
        "timestamp": chrono::Utc::now().to_rfc3339()
    }));

    Ok((
        StatusCode::OK,
        Json(json!({
            "success": true,
            "mission_id": mission_id,
            "is_pinned": new_pinned,
            "actor": actor
        })),
    ))
}

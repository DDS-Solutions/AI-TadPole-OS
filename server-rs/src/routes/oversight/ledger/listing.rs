//! @docs ARCHITECTURE:Networking
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / HTTP Routes / oversight ledger
//!
//! ### AI Assist Note
//! - Keep each oversight boundary in its own focused route module.
//!
//! ### 🔍 Debugging & Observability
//! - **Telemetry Targets**: none declared

use super::*;

/// GET /v1/oversight/pending
///
/// Returns a collection of all actions (file edits, network requests, etc.)
/// currently paused and awaiting human verification.
///
/// @docs API_REFERENCE:GetPendingOversight
#[tracing::instrument(skip(state), fields(trace_id = tracing::field::Empty, request_id = tracing::field::Empty, http_status = tracing::field::Empty), name = "governance::list_pending")]
pub async fn get_pending(
    State(state): State<Arc<AppState>>,
    Query(params): Query<PaginationParams>,
) -> Result<impl IntoResponse, AppError> {
    let (page, per_page) = params.sanitize();
    let sanitized_params = PaginationParams { page, per_page };
    let offset = sanitized_params.offset();
    let limit = per_page as usize;

    let mut all_entries: Vec<OversightEntry> = state
        .comms
        .oversight_queue
        .iter()
        .map(|entry| entry.value().clone())
        .collect();
    all_entries.sort_by(|left, right| {
        left.created_at
            .cmp(&right.created_at)
            .then_with(|| left.id.cmp(&right.id))
    });
    let total = u32::try_from(all_entries.len()).unwrap_or(u32::MAX);
    let entries = all_entries.into_iter().skip(offset).take(limit).collect();

    Ok(Json(PaginatedResponse::from_pre_sliced(
        entries,
        total,
        &sanitized_params,
        "/v1/oversight/pending",
    )))
}

/// GET /v1/oversight/ledger
///
/// Provides a historical audit trail of all previous oversight decisions.
/// Directly queries the SQLite persistence layer with support for pagination.
///
/// @docs API_REFERENCE:GetOversightLedger
#[tracing::instrument(skip(state), fields(trace_id = tracing::field::Empty, request_id = tracing::field::Empty, http_status = tracing::field::Empty), name = "governance::list_ledger")]
pub async fn get_ledger(
    State(state): State<Arc<AppState>>,
    Query(params): Query<PaginationParams>,
) -> Result<impl IntoResponse, AppError> {
    let (page, per_page) = params.sanitize();
    let limit_i64 = per_page as i64;
    let offset_i64 = (page.saturating_sub(1) as i64).saturating_mul(limit_i64);
    let sanitized_params = PaginationParams { page, per_page };

    // COUNT(*) first so total_pages is accurate for frontend pagination
    let total: i64 = tokio::time::timeout(
        DB_QUERY_TIMEOUT,
        sqlx::query_scalar("SELECT COUNT(*) FROM oversight_log WHERE status != 'pending'")
            .fetch_one(&state.resources.pool),
    )
    .await
    .map_err(|_| AppError::InternalServerError("Database query timed out".to_string()))?
    .map_err(AppError::Sqlx)?;

    let rows = tokio::time::timeout(
        DB_QUERY_TIMEOUT,
        sqlx::query(
            "SELECT id, mission_id, agent_id, entry_type, skill, params, status, decision, decided_at, decided_by, created_at, payload 
             FROM oversight_log 
             WHERE status != 'pending' 
             ORDER BY created_at DESC 
             LIMIT ? OFFSET ?"
        )
        .bind(limit_i64)
        .bind(offset_i64)
        .fetch_all(&state.resources.pool),
    )
    .await
    .map_err(|_| AppError::InternalServerError("Database query timed out".to_string()))?
    .map_err(AppError::Sqlx)?;

    let entries_result: Result<Vec<serde_json::Value>, AppError> = rows
        .into_iter()
        .map(|row| {
            use sqlx::Row;
            let id: String = row.try_get("id").map_err(AppError::Sqlx)?;
            let mission_id: Option<String> = row.try_get("mission_id").map_err(AppError::Sqlx)?;
            let agent_id: String = row.try_get("agent_id").map_err(AppError::Sqlx)?;
            let entry_type: String = row.try_get("entry_type").map_err(AppError::Sqlx)?;
            let skill: Option<String> = row.try_get("skill").map_err(AppError::Sqlx)?;
            let params_raw: String = row.try_get("params").map_err(AppError::Sqlx)?;
            let status_str: String = row.try_get("status").map_err(AppError::Sqlx)?;
            let decision_str: Option<String> = row.try_get("decision").map_err(AppError::Sqlx)?;
            let decided_at: Option<chrono::DateTime<chrono::Utc>> =
                row.try_get("decided_at").map_err(AppError::Sqlx)?;
            let decided_by_str: Option<String> =
                row.try_get("decided_by").map_err(AppError::Sqlx)?;
            let created_at: Option<chrono::DateTime<chrono::Utc>> =
                row.try_get("created_at").map_err(AppError::Sqlx)?;
            let payload_raw: Option<String> = row.try_get("payload").map_err(AppError::Sqlx)?;

            let is_auto = decision_str.as_deref() == Some("auto_approved")
                || status_str == "auto_approved"
                || decided_by_str.as_deref() == Some("auto_policy")
                || decided_by_str.as_deref() == Some("system");

            let params_json = serde_json::from_str::<serde_json::Value>(&params_raw)
                .unwrap_or_else(|e| {
                    tracing::warn!(
                        "⚠️ Corrupt params JSON in oversight_log row '{}': {}",
                        id,
                        e
                    );
                    serde_json::Value::Null
                });
            let payload_json = payload_raw
                .as_deref()
                .and_then(|p| serde_json::from_str::<serde_json::Value>(p).ok())
                .unwrap_or(serde_json::Value::Null);

            Ok(serde_json::json!({
                "id": id,
                "mission_id": mission_id,
                "tool_call": {
                    "id": format!("tc-{}", id),
                    "agent_id": agent_id,
                    "skill": skill,
                    "params": params_json,
                    "description": payload_raw.unwrap_or_else(|| "Historical action".to_string()),
                },
                "type": entry_type,
                "status": status_str,
                "decision": decision_str,
                "decided_at": decided_at,
                "decided_by": decided_by_str,
                "auto_approved": is_auto,
                "approval_type": if is_auto { "auto" } else { "hitl" },
                "requires_oversight": !is_auto,
                "created_at": created_at,
                "timestamp": created_at,
                "payload": payload_json,
            }))
        })
        .collect();
    let entries = entries_result?;

    // Use from_pre_sliced — SQL already applied LIMIT/OFFSET, so we provide the real total safely
    let safe_total = u32::try_from(total.max(0)).unwrap_or(u32::MAX);
    Ok(Json(PaginatedResponse::from_pre_sliced(
        entries,
        safe_total,
        &sanitized_params,
        "/v1/oversight/ledger",
    )))
}

//! @docs ARCHITECTURE:Networking
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / HTTP Routes / missions / catalog
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - [Structural] Deterministic internal state integrity and strict interface contract compliance.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: AppError::BadRequest, AppError::Sqlx
//! - **Telemetry Targets**: none declared

use super::contracts::*;
use crate::error::AppError;
use crate::state::AppState;
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use sqlx::{QueryBuilder, Row, Sqlite};
use std::sync::Arc;

/// Escapes SQL LIKE wildcards (% and _) and backslashes for exact substring matching.
pub(crate) fn escape_like_pattern(input: &str) -> String {
    let escaped = input
        .replace('\\', "\\\\")
        .replace('%', "\\%")
        .replace('_', "\\_");
    format!("%{}%", escaped)
}

/// GET /v1/missions
/// Retrieves a paginated catalog of all missions with aggregated log and diff counts.
#[tracing::instrument(skip(state), name = "missions::get_missions_catalog")]
pub async fn get_missions_catalog(
    State(state): State<Arc<AppState>>,
    Query(query): Query<CatalogQuery>,
) -> Result<impl IntoResponse, AppError> {
    let pool = &state.resources.pool;
    let page = query.page.unwrap_or(1).max(1);
    let per_page = query.per_page.unwrap_or(25).clamp(1, 100);
    let offset = (page.saturating_sub(1) as i64).saturating_mul(per_page as i64);

    let status_filter = query.status.as_deref().unwrap_or("").trim();
    let search_filter = query.search.as_deref().unwrap_or("").trim();

    let mut count_qb =
        QueryBuilder::<Sqlite>::new("SELECT COUNT(*) FROM mission_history WHERE 1=1");
    if !status_filter.is_empty() && status_filter != "all" {
        count_qb.push(" AND LOWER(status) = LOWER(");
        count_qb.push_bind(status_filter);
        count_qb.push(")");
    }
    if let Some(pinned) = query.is_pinned {
        count_qb.push(" AND COALESCE(is_pinned, 0) = ");
        count_qb.push_bind(if pinned { 1 } else { 0 });
    }
    if !search_filter.is_empty() {
        let pattern = escape_like_pattern(search_filter);
        count_qb.push(" AND (title LIKE ");
        count_qb.push_bind(pattern.clone());
        count_qb.push(" ESCAPE '\\' OR id LIKE ");
        count_qb.push_bind(pattern.clone());
        count_qb.push(" ESCAPE '\\' OR agent_id LIKE ");
        count_qb.push_bind(pattern);
        count_qb.push(" ESCAPE '\\')");
    }

    let total: i64 = count_qb
        .build_query_scalar()
        .fetch_one(pool)
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "Failed to execute mission catalog count query");
            AppError::from(e)
        })?;

    let mut sel_qb = QueryBuilder::<Sqlite>::new(
        r#"SELECT 
            id, 
            agent_id, 
            title, 
            status, 
            budget_usd, 
            cost_usd, 
            COALESCE(is_pinned, 0) as is_pinned,
            strftime('%Y-%m-%dT%H:%M:%SZ', created_at) as created_at_iso,
            strftime('%Y-%m-%dT%H:%M:%SZ', updated_at) as updated_at_iso,
            (SELECT COUNT(*) FROM mission_logs l WHERE l.mission_id = mission_history.id) as log_count,
            (SELECT COUNT(*) FROM file_revisions r WHERE r.mission_id = mission_history.id) as diff_count
        FROM mission_history
        WHERE 1=1"#,
    );

    if !status_filter.is_empty() && status_filter != "all" {
        sel_qb.push(" AND LOWER(status) = LOWER(");
        sel_qb.push_bind(status_filter);
        sel_qb.push(")");
    }
    if let Some(pinned) = query.is_pinned {
        sel_qb.push(" AND COALESCE(is_pinned, 0) = ");
        sel_qb.push_bind(if pinned { 1 } else { 0 });
    }
    if !search_filter.is_empty() {
        let pattern = escape_like_pattern(search_filter);
        sel_qb.push(" AND (title LIKE ");
        sel_qb.push_bind(pattern.clone());
        sel_qb.push(" ESCAPE '\\' OR id LIKE ");
        sel_qb.push_bind(pattern.clone());
        sel_qb.push(" ESCAPE '\\' OR agent_id LIKE ");
        sel_qb.push_bind(pattern);
        sel_qb.push(" ESCAPE '\\')");
    }

    sel_qb.push(" ORDER BY is_pinned DESC, created_at DESC, id DESC LIMIT ");
    sel_qb.push_bind(per_page as i64);
    sel_qb.push(" OFFSET ");
    sel_qb.push_bind(offset);

    let rows = sel_qb.build().fetch_all(pool).await?;
    let mut data = Vec::with_capacity(rows.len());

    for r in rows {
        let is_pinned_raw: i64 = r.try_get("is_pinned").unwrap_or(0);
        let created_at_raw: Option<String> = r.try_get("created_at_iso").ok();
        let updated_at_raw: Option<String> = r.try_get("updated_at_iso").ok();

        data.push(MissionSummaryItem {
            id: r.try_get("id").unwrap_or_default(),
            agent_id: r.try_get("agent_id").unwrap_or_default(),
            title: r.try_get("title").unwrap_or_default(),
            status: r.try_get("status").unwrap_or_default(),
            budget_usd: r.try_get("budget_usd").unwrap_or(0.0),
            cost_usd: r.try_get("cost_usd").unwrap_or(0.0),
            is_pinned: is_pinned_raw == 1,
            created_at: created_at_raw.unwrap_or_else(|| chrono::Utc::now().to_rfc3339()),
            updated_at: updated_at_raw.unwrap_or_else(|| chrono::Utc::now().to_rfc3339()),
            log_count: r.try_get("log_count").unwrap_or(0),
            diff_count: r.try_get("diff_count").unwrap_or(0),
        });
    }

    Ok((
        StatusCode::OK,
        Json(CatalogResponse {
            success: true,
            data,
            total,
            page,
            per_page,
        }),
    ))
}

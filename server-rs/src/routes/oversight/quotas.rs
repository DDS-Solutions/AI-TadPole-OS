//! @docs ARCHITECTURE:Networking
//!
//! ### AI Assist Note
//! **Budget & Quota Management**: Implements security metering, agent quota
//! updates, and mission-level budget governance endpoints.
//!
//! ### 🔍 Debugging & Observability
//! - **Trace Scope**: `server-rs::routes::oversight::quotas`

use crate::error::AppError;
use crate::security::metering::{Quota, ResetPeriod};
use crate::state::AppState;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use std::sync::Arc;

use super::DB_QUERY_TIMEOUT;

#[derive(serde::Deserialize, Debug)]
pub struct UpdateQuotaPayload {
    pub budget_usd: f64,
    pub reset_period: Option<crate::security::metering::ResetPeriod>,
}

/// PUT /oversight/security/quotas/:entity_id
///
/// Updates the budget quota and reset period for a specific agent.
///
/// @docs API_REFERENCE:UpdateAgentQuota
#[tracing::instrument(skip(state), fields(trace_id = tracing::field::Empty, request_id = tracing::field::Empty, http_status = tracing::field::Empty), name = "governance::update_agent_quota")]
pub async fn update_agent_quota(
    Path(entity_id): Path<String>,
    State(state): State<Arc<AppState>>,
    _admin: crate::middleware::auth::RequireAdmin,
    Json(payload): Json<UpdateQuotaPayload>,
) -> Result<impl IntoResponse, AppError> {
    state
        .security
        .budget_guard
        .update_quota(&entity_id, payload.budget_usd, payload.reset_period)
        .await
        .map_err(|e| AppError::InternalServerError(format!("Failed to update quota: {}", e)))?;

    tracing::info!(
        "🛡️ [Budget] Quota updated for agent {}: ${}",
        entity_id,
        payload.budget_usd
    );
    Ok((StatusCode::OK, Json(serde_json::json!({ "status": "ok" }))))
}

/// GET /oversight/security/quotas
///
/// Returns global budget telemetry, including total spent, remaining,
/// and system defense metrics.
///
/// @docs API_REFERENCE:GetQuotas
#[tracing::instrument(skip(state), fields(trace_id = tracing::field::Empty, request_id = tracing::field::Empty, http_status = tracing::field::Empty), name = "security_governance::get_quotas")]
pub async fn get_security_quotas(
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, AppError> {
    let mut total_budget = 0.0;
    let mut total_spent = 0.0;

    for entry in state.registry.agents.iter() {
        total_budget += entry.value().economics.budget_usd;
        total_spent += entry.value().economics.cost_usd;
    }

    let agent_quotas: Vec<crate::security::metering::Quota> = state
        .security
        .budget_guard
        .get_all_quotas()
        .await
        .unwrap_or_default();

    let system_defense = state.security.system_monitor.get_system_defense_stats();
    let verify_res = tokio::time::timeout(
        DB_QUERY_TIMEOUT,
        state.security.audit_trail.verify_last_n(10, None),
    )
    .await;
    let merkle_integrity = match verify_res {
        Ok(Ok((v, t))) if v == t && t > 0 => 1.0,
        Ok(Ok((v, t))) if t > 0 => v as f64 / t as f64,
        _ => 1.0,
    };

    Ok(Json(serde_json::json!({
        "total_budget": total_budget,
        "total_spent": total_spent,
        "remaining": total_budget - total_spent,
        "efficiency": if total_budget > 0.0 { (total_spent / total_budget) * 100.0 } else { 0.0 },
        "agent_quotas": agent_quotas,
        "system_defense": {
            "memory_pressure": system_defense.memory_pressure,
            "cpu_load": system_defense.cpu_load,
            "sandbox_status": system_defense.sandbox_status,
            "sandbox_type": system_defense.sandbox_type,
            "merkle_integrity": merkle_integrity
        }
    })))
}

/// GET /v1/oversight/security/missions/quotas
#[tracing::instrument(skip(state), fields(trace_id = tracing::field::Empty, request_id = tracing::field::Empty, http_status = tracing::field::Empty), name = "security_governance::get_mission_quotas")]
pub async fn get_mission_quotas(
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, AppError> {
    let rows = tokio::time::timeout(
        DB_QUERY_TIMEOUT,
        sqlx::query("SELECT * FROM mission_quotas").fetch_all(&state.resources.pool),
    )
    .await
    .map_err(|_| AppError::InternalServerError("Database query timed out".to_string()))?
    .map_err(AppError::Sqlx)?;

    let quotas: Vec<Quota> = rows
        .into_iter()
        .map(|r| {
            use sqlx::Row;
            let period_str: String = r.get("reset_period");
            let period = match period_str.as_str() {
                "daily" => ResetPeriod::Daily,
                "monthly" => ResetPeriod::Monthly,
                _ => ResetPeriod::Never,
            };

            let budget_micros: i64 = r.get("budget_usd");
            let used_micros: i64 = r.get("used_usd");

            Quota {
                id: r.get("id"),
                entity_id: r.get("cluster_id"),
                budget_usd: (budget_micros as f64) / 1_000_000.0,
                used_usd: (used_micros as f64) / 1_000_000.0,
                reset_period: period,
                last_reset_at: r.get("last_reset_at"),
                next_reset_at: r.get("next_reset_at"),
            }
        })
        .collect();

    Ok(Json(serde_json::json!({ "quotas": quotas })))
}

/// PUT /v1/oversight/security/missions/:id/quota
#[tracing::instrument(
    skip(state, payload),
    name = "security_governance::update_mission_quota"
)]
pub async fn update_mission_quota(
    Path(cluster_id): Path<String>,
    State(state): State<Arc<AppState>>,
    _admin: crate::middleware::auth::RequireAdmin,
    Json(payload): Json<UpdateQuotaPayload>,
) -> Result<impl IntoResponse, AppError> {
    state
        .security
        .budget_guard
        .update_mission_quota(&cluster_id, payload.budget_usd, payload.reset_period)
        .await
        .map_err(|e| {
            AppError::InternalServerError(format!("Failed to update mission quota: {}", e))
        })?;

    tracing::info!(
        "🛡️ [Budget] Quota updated for mission {}: ${}",
        cluster_id,
        payload.budget_usd
    );
    Ok((StatusCode::OK, Json(serde_json::json!({ "status": "ok" }))))
}

// Metadata: [oversight::quotas]

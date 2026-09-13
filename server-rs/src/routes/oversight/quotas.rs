//! @docs ARCHITECTURE:Networking
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / HTTP Routes / quotas
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Type-safe state handling and bounded execution without unhandled panics.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: `AppError::BadRequest`, `AppError::InternalServerError`, `AppError::Sqlx`
//! - **Telemetry Targets**: none declared
//! - **Witness Tests**: `quotas::test_validate_budget_bounds`, `quotas::test_build_quota_summary`

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

pub fn validate_budget(budget: f64) -> Result<(), AppError> {
    if !budget.is_finite() || !(0.0..=100_000_000.0).contains(&budget) {
        return Err(AppError::BadRequest(
            "budget_usd must be a finite, non-negative number <= 100,000,000.0".to_string(),
        ));
    }
    Ok(())
}

/// Builds a unified quota and system defense summary.
/// Shared across `get_security_quotas` and `get_security_snapshot`.
pub async fn build_quota_summary(state: &AppState) -> serde_json::Value {
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
        .unwrap_or_else(|e| {
            tracing::warn!("⚠️ Failed to fetch agent quotas from budget guard: {:?}", e);
            Vec::new()
        });

    let system_defense = state.security.system_monitor.get_system_defense_stats();
    let verify_res = tokio::time::timeout(
        DB_QUERY_TIMEOUT,
        state.security.audit_trail.verify_last_n(10, None),
    )
    .await;

    // Fail-Closed: Infrastructure errors and timeouts report -1.0 / non-secure status (H1)
    let (merkle_integrity, merkle_status) = match verify_res {
        Ok(Ok((_v, t))) if t == 0 => (1.0, "SECURE"),
        Ok(Ok((v, t))) if v == t => (1.0, "SECURE"),
        Ok(Ok((v, t))) => (v as f64 / t as f64, "TAMPERED"),
        Ok(Err(e)) => {
            tracing::error!("🚨 Audit trail integrity verification failed: {:?}", e);
            (-1.0, "ERROR")
        }
        Err(_) => {
            tracing::error!("🚨 Audit trail integrity verification timed out");
            (-1.0, "TIMEOUT")
        }
    };

    let utilization = if total_budget > 0.0 {
        (total_spent / total_budget) * 100.0
    } else {
        0.0
    };

    serde_json::json!({
        "total_budget": total_budget,
        "total_spent": total_spent,
        "remaining": total_budget - total_spent,
        "efficiency": utilization,
        "utilization": utilization,
        "agent_quotas": agent_quotas,
        "system_defense": {
            "memory_pressure": system_defense.memory_pressure,
            "cpu_load": system_defense.cpu_load,
            "sandbox_status": system_defense.sandbox_status,
            "sandbox_type": system_defense.sandbox_type,
            "merkle_integrity": merkle_integrity,
            "merkle_status": merkle_status,
        }
    })
}

/// PUT /v1/oversight/security/quotas/:entity_id
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
    validate_budget(payload.budget_usd)?;

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

/// GET /v1/oversight/security/quotas
///
/// Returns global budget telemetry, including total spent, remaining,
/// and system defense metrics.
///
/// @docs API_REFERENCE:GetQuotas
#[tracing::instrument(skip(state), fields(trace_id = tracing::field::Empty, request_id = tracing::field::Empty, http_status = tracing::field::Empty), name = "security_governance::get_quotas")]
pub async fn get_security_quotas(
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, AppError> {
    let summary = build_quota_summary(&state).await;
    Ok(Json(summary))
}

/// GET /v1/oversight/security/missions/quotas
///
/// Retrieves cluster/mission level budget allocations.
#[tracing::instrument(skip(state), fields(trace_id = tracing::field::Empty, request_id = tracing::field::Empty, http_status = tracing::field::Empty), name = "security_governance::get_mission_quotas")]
pub async fn get_mission_quotas(
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, AppError> {
    let rows = tokio::time::timeout(
        DB_QUERY_TIMEOUT,
        sqlx::query(
            "SELECT id, cluster_id, budget_usd, used_usd, reset_period, last_reset_at, next_reset_at 
             FROM mission_quotas 
             ORDER BY id ASC"
        )
        .fetch_all(&state.resources.pool),
    )
    .await
    .map_err(|_| AppError::InternalServerError("Database query timed out".to_string()))?
    .map_err(AppError::Sqlx)?;

    let quotas: Vec<Quota> = rows
        .into_iter()
        .filter_map(|r| {
            use sqlx::Row;
            let id: String = r.try_get("id").ok()?;
            let cluster_id: String = r.try_get("cluster_id").unwrap_or_else(|_| "default".to_string());
            let period_str: String = r.try_get("reset_period").unwrap_or_else(|_| "never".to_string());
            let period = match period_str.as_str() {
                "daily" => ResetPeriod::Daily,
                "monthly" => ResetPeriod::Monthly,
                "never" => ResetPeriod::Never,
                other => {
                    tracing::warn!(
                        "⚠️ Unrecognized reset_period '{}' in mission_quotas row '{}', defaulting to Never",
                        other, id
                    );
                    ResetPeriod::Never
                }
            };

            let budget_micros: i64 = r.try_get("budget_usd").unwrap_or(0);
            let used_micros: i64 = r.try_get("used_usd").unwrap_or(0);
            let last_reset_at: chrono::DateTime<chrono::Utc> = r
                .try_get("last_reset_at")
                .unwrap_or_else(|_| chrono::Utc::now());
            let next_reset_at: chrono::DateTime<chrono::Utc> = r
                .try_get("next_reset_at")
                .unwrap_or_else(|_| {
                    crate::security::metering::compute_next_reset(chrono::Utc::now(), period)
                });

            Some(Quota {
                id,
                entity_id: cluster_id,
                budget_usd: (budget_micros as f64) / 1_000_000.0,
                used_usd: (used_micros as f64) / 1_000_000.0,
                reset_period: period,
                last_reset_at,
                next_reset_at,
            })
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
    validate_budget(payload.budget_usd)?;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_budget_bounds() {
        assert!(validate_budget(0.0).is_ok());
        assert!(validate_budget(50.0).is_ok());
        assert!(validate_budget(100_000_000.0).is_ok());

        assert!(validate_budget(-1.0).is_err());
        assert!(validate_budget(f64::NAN).is_err());
        assert!(validate_budget(f64::INFINITY).is_err());
        assert!(validate_budget(f64::NEG_INFINITY).is_err());
        assert!(validate_budget(100_000_001.0).is_err());
    }

    #[tokio::test]
    async fn test_build_quota_summary_structure() {
        let state = AppState::new_minimal_mock().await;
        let summary = build_quota_summary(&state).await;

        assert!(summary.get("total_budget").is_some());
        assert!(summary.get("total_spent").is_some());
        assert!(summary.get("remaining").is_some());
        assert!(summary.get("system_defense").is_some());

        let system_defense = summary.get("system_defense").unwrap();
        assert!(system_defense.get("merkle_integrity").is_some());
        assert!(system_defense.get("merkle_status").is_some());
    }
}

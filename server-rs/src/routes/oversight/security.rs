//! @docs ARCHITECTURE:Networking
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / HTTP Routes / security
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Type-safe state handling and bounded execution without unhandled panics.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: `AppError::BadRequest`, `AppError::InternalServerError`, `AppError::Sqlx`
//! - **Telemetry Targets**: none declared
//! - **Witness Tests**: `security::test_get_integrity_status_empty_ledger`, `security::test_policy_mode_serialization`

use crate::error::AppError;
use crate::routes::pagination::{PaginatedResponse, PaginationParams};
use crate::security::audit::AuditEntry;
use crate::state::AppState;
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::Serialize;
use serde_json::json;
use std::sync::Arc;

use super::{build_quota_summary, DB_QUERY_TIMEOUT};

pub const UNHEALTHY_FAILURE_THRESHOLD: u32 = 5;
pub const THROTTLED_FAILURE_THRESHOLD: u32 = 3;

#[derive(Serialize)]
pub struct OversightAuditEntry {
    pub id: String,
    pub agent_id: String,
    pub skill: Option<String>,
    pub status: String,
    pub decision: Option<String>,
    pub decided_at: Option<chrono::DateTime<chrono::Utc>>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub is_verified: bool,
}

#[derive(Serialize)]
pub struct SecurityIntegrityResponse {
    pub integrity_score: f64,
    pub status: String,
    pub verified_count: usize,
    pub total_count: usize,
}

#[derive(Serialize)]
pub struct PolicyItem {
    pub tool_name: String,
    pub mode: String,
}

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum PolicyMode {
    Allow,
    Deny,
    Prompt,
}

#[derive(serde::Deserialize, Debug)]
pub struct UpdatePolicyPayload {
    pub tool_name: String,
    pub mode: PolicyMode,
}

// ==========================================
// Shared Security Helpers (DRY Extraction)
// ==========================================

pub(crate) fn fetch_agent_health_data(state: &AppState) -> Vec<serde_json::Value> {
    state
        .registry
        .agents
        .iter()
        .map(|entry| {
            let agent = entry.value();
            serde_json::json!({
                "agent_id": agent.identity.id,
                "name": agent.identity.name,
                "status": agent.health.status,
                "failure_count": agent.health.failure_count,
                "last_failure_at": agent.health.last_failure_at,
                "is_healthy": agent.health.failure_count < UNHEALTHY_FAILURE_THRESHOLD,
                "is_throttled": agent.health.failure_count >= THROTTLED_FAILURE_THRESHOLD,
                // Budget of 0.0 represents unmetered development agents
                "is_bankrupt": agent.economics.cost_usd >= agent.economics.budget_usd && agent.economics.budget_usd > 0.0,
            })
        })
        .collect()
}

pub(crate) async fn fetch_audit_logs(
    state: &AppState,
    params: &PaginationParams,
) -> Result<(Vec<OversightAuditEntry>, u32), AppError> {
    let (page, per_page) = params.sanitize();
    let limit = per_page as i32;
    let offset = ((page as i32) - 1) * limit;

    let total: i64 = tokio::time::timeout(
        DB_QUERY_TIMEOUT,
        sqlx::query_scalar("SELECT COUNT(*) FROM audit_trail").fetch_one(&state.resources.pool),
    )
    .await
    .map_err(|_| AppError::InternalServerError("Database query timed out".to_string()))?
    .map_err(AppError::Sqlx)?;

    let entries: Vec<AuditEntry> = tokio::time::timeout(
        DB_QUERY_TIMEOUT,
        sqlx::query_as(
            "SELECT id, seq, prev_hash, record_hash, timestamp, actor, mission_id, agent_id, action, params, status 
             FROM audit_trail 
             ORDER BY seq DESC, timestamp DESC 
             LIMIT ? OFFSET ?"
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&state.resources.pool),
    )
    .await
    .map_err(|_| AppError::InternalServerError("Database query timed out".to_string()))?
    .map_err(AppError::Sqlx)?;

    let response: Vec<OversightAuditEntry> = entries
        .into_iter()
        .map(|entry| {
            let is_verified = state.security.audit_trail.verify_record(&entry);
            OversightAuditEntry {
                id: entry.id,
                agent_id: entry.agent_id,
                skill: Some(entry.action),
                status: "recorded".to_string(),
                decision: None,
                decided_at: None,
                created_at: entry.timestamp,
                is_verified,
            }
        })
        .collect();

    Ok((response, total as u32))
}

// ==========================================
// Route Handlers
// ==========================================

/// GET /v1/oversight/security/audit-trail
///
/// Retrieves the tamper-evident Merkle hash-chain logs with accurate total count pagination.
///
/// @docs API_REFERENCE:GetAuditTrail
#[tracing::instrument(skip(state), fields(trace_id = tracing::field::Empty, request_id = tracing::field::Empty, http_status = tracing::field::Empty), name = "governance::get_audit_trail")]
pub async fn get_audit_trail(
    State(state): State<Arc<AppState>>,
    Query(params): Query<PaginationParams>,
) -> Result<impl IntoResponse, AppError> {
    let (entries, total) = fetch_audit_logs(&state, &params).await?;

    Ok(Json(PaginatedResponse::from_pre_sliced(
        entries,
        total,
        &params,
        "/v1/oversight/security/audit-trail",
    )))
}

/// GET /v1/oversight/security/integrity
///
/// Verifies the last N records in the Merkle chain and returns an integrity score.
/// Propagates database infrastructure errors cleanly instead of triggering false-positive tamper alarms.
///
/// @docs API_REFERENCE:GetIntegrityStatus
#[tracing::instrument(skip(state), fields(trace_id = tracing::field::Empty, request_id = tracing::field::Empty, http_status = tracing::field::Empty), name = "governance::get_integrity")]
pub async fn get_integrity_status(
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, AppError> {
    let verify_res = tokio::time::timeout(
        DB_QUERY_TIMEOUT,
        state.security.audit_trail.verify_last_n(50, None),
    )
    .await
    .map_err(|_| {
        AppError::InternalServerError("Database integrity verification timed out".to_string())
    })?
    .map_err(|e| AppError::InternalServerError(format!("Integrity verification failed: {}", e)))?;

    let (verified_count, total_count) = verify_res;

    let is_secure = verified_count == total_count && total_count > 0;
    let score = if total_count > 0 {
        verified_count as f64 / total_count as f64
    } else {
        1.0 // Empty ledger is conceptually intact
    };

    Ok(Json(SecurityIntegrityResponse {
        integrity_score: score,
        status: if is_secure || total_count == 0 {
            "SECURE".to_string()
        } else {
            "TAMPERED".to_string()
        },
        verified_count,
        total_count,
    }))
}

/// GET /v1/oversight/security/health
///
/// Returns health metrics for all registered agents.
///
/// @docs OPERATIONS_MANUAL:AgentHealth
#[tracing::instrument(skip(state), fields(trace_id = tracing::field::Empty, request_id = tracing::field::Empty, http_status = tracing::field::Empty), name = "security_governance::get_health")]
pub async fn get_agent_health(
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, AppError> {
    let health_data = fetch_agent_health_data(&state);
    Ok(Json(serde_json::json!({ "agents": health_data })))
}

/// GET /v1/oversight/security/snapshot
///
/// Returns a consolidated system security snapshot including quotas,
/// agent health, and the latest audit trail logs.
#[tracing::instrument(skip(state), fields(trace_id = tracing::field::Empty, request_id = tracing::field::Empty, http_status = tracing::field::Empty), name = "security_governance::get_security_snapshot")]
pub async fn get_security_snapshot(
    State(state): State<Arc<AppState>>,
    Query(params): Query<PaginationParams>,
) -> Result<impl IntoResponse, AppError> {
    // 1. Quotas & Metrics via shared helper (M4)
    let quotas_json = build_quota_summary(&state).await;

    // 2. Agent Health (via shared helper)
    let health_data = fetch_agent_health_data(&state);

    // 3. Audit Trail (via shared helper with accurate COUNT(*))
    let (response, total) = fetch_audit_logs(&state, &params).await?;

    Ok(Json(serde_json::json!({
        "quotas": quotas_json,
        "agent_health": health_data,
        "audit_trail": {
            "data": response,
            "total": total
        }
    })))
}

/// GET /v1/oversight/security/policies
#[tracing::instrument(skip(state), fields(trace_id = tracing::field::Empty, request_id = tracing::field::Empty, http_status = tracing::field::Empty), name = "governance::get_policies")]
pub async fn get_policies(
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, AppError> {
    let rows: Vec<(String, String)> = tokio::time::timeout(
        DB_QUERY_TIMEOUT,
        sqlx::query_as("SELECT tool_name, mode FROM permission_policies ORDER BY tool_name ASC")
            .fetch_all(&state.resources.pool),
    )
    .await
    .map_err(|_| AppError::InternalServerError("Database query timed out".to_string()))?
    .map_err(AppError::Sqlx)?;

    let policies: Vec<PolicyItem> = rows
        .into_iter()
        .map(|(name, mode)| PolicyItem {
            tool_name: name,
            mode,
        })
        .collect();

    Ok(Json(policies))
}

/// PUT /v1/oversight/security/policies
#[tracing::instrument(skip(state, payload), name = "governance::update_policy")]
pub async fn update_policy(
    State(state): State<Arc<AppState>>,
    _admin: crate::middleware::auth::RequireAdmin,
    Json(payload): Json<UpdatePolicyPayload>,
) -> Result<impl IntoResponse, AppError> {
    let tool_name = payload.tool_name.trim().to_lowercase();
    if tool_name.is_empty() {
        return Err(AppError::BadRequest(
            "tool_name cannot be empty".to_string(),
        ));
    }

    let mode_str = match payload.mode {
        PolicyMode::Allow => "allow",
        PolicyMode::Deny => "deny",
        PolicyMode::Prompt => "prompt",
    };

    tokio::time::timeout(
        DB_QUERY_TIMEOUT,
        sqlx::query(
            "INSERT INTO permission_policies (tool_name, mode) VALUES (?, ?) 
             ON CONFLICT(tool_name) DO UPDATE SET mode = excluded.mode",
        )
        .bind(&tool_name)
        .bind(mode_str)
        .execute(&state.resources.pool),
    )
    .await
    .map_err(|_| AppError::InternalServerError("Database write timed out".to_string()))?
    .map_err(AppError::Sqlx)?;

    // Refresh cache with timeout and explicit error log
    if let Err(err) = tokio::time::timeout(
        DB_QUERY_TIMEOUT,
        state.security.permission_policy.refresh_cache(),
    )
    .await
    {
        tracing::error!("🚨 Permission cache refresh timed out: {:?}", err);
    }

    state.broadcast_sys(
        &format!("🛡️ Security Policy Updated: {} -> {}", tool_name, mode_str),
        "info",
        None,
    );

    Ok((
        StatusCode::OK,
        Json(json!({ "status": "ok", "tool_name": tool_name, "mode": mode_str })),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_policy_mode_serialization() {
        let allow_json = serde_json::to_string(&PolicyMode::Allow).unwrap();
        assert_eq!(allow_json, "\"allow\"");

        let deny_json = serde_json::to_string(&PolicyMode::Deny).unwrap();
        assert_eq!(deny_json, "\"deny\"");

        let prompt_json = serde_json::to_string(&PolicyMode::Prompt).unwrap();
        assert_eq!(prompt_json, "\"prompt\"");
    }

    #[tokio::test]
    async fn test_get_integrity_status_empty_ledger() {
        let state = Arc::new(AppState::new_minimal_mock().await);
        let res = get_integrity_status(State(state))
            .await
            .expect("Failed to get integrity status");
        let (parts, body) = res.into_response().into_parts();
        assert_eq!(parts.status, StatusCode::OK);

        let bytes = axum::body::to_bytes(body, 64 * 1024)
            .await
            .expect("Failed to read body");
        let parsed: serde_json::Value =
            serde_json::from_slice(&bytes).expect("Failed to parse JSON");

        assert_eq!(parsed["status"], "SECURE");
        assert_eq!(parsed["integrity_score"], 1.0);
    }
}

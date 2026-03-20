use crate::state::AppState;
use crate::agent::types::{OversightDecision, OversightEntry};
use crate::routes::error::AppError;
use crate::routes::pagination::{PaginatedResponse, PaginationParams};
use crate::security::audit::AuditEntry;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::Serialize;
use serde_json::json;
use std::sync::Arc;


#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
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
#[serde(rename_all = "camelCase")]
pub struct SecurityIntegrityResponse {
    pub integrity_score: f64,
    pub status: String,
    pub verified_count: usize,
    pub total_count: usize,
}

/// GET /v1/oversight/pending
/// 
/// Returns a collection of all actions (file edits, network requests, etc.) 
/// currently paused and awaiting human verification.
///
/// @docs API_REFERENCE:GetPendingOversight
pub async fn get_pending(
    State(state): State<Arc<AppState>>,
    Query(params): Query<PaginationParams>,
) -> Result<impl IntoResponse, AppError> {
    let entries: Vec<OversightEntry> = state.comms.oversight_queue
        .iter()
        .map(|entry| entry.value().clone())
        .collect();

    Ok(Json(PaginatedResponse::from_vec(
        entries,
        &params,
        "/v1/oversight/pending",
    )))
}

/// GET /v1/oversight/ledger
/// 
/// Provides a historical audit trail of all previous oversight decisions.
/// Directly queries the SQLite persistence layer with support for pagination.
///
/// @docs API_REFERENCE:GetOversightLedger
pub async fn get_ledger(
    State(state): State<Arc<AppState>>,
    Query(params): Query<PaginationParams>,
) -> Result<impl IntoResponse, AppError> {
    let (page, per_page) = params.sanitize();
    let limit = per_page as i32;
    let offset = ((page as i32) - 1) * limit;

    let rows = sqlx::query(
        "SELECT id, mission_id, agent_id, entry_type, skill, params, status, decision, decided_at, decided_by, created_at, payload 
         FROM oversight_log 
         WHERE status != 'pending' 
         ORDER BY created_at DESC 
         LIMIT ? OFFSET ?"
    )
    .bind(limit)
    .bind(offset)
    .fetch_all(&state.resources.pool)
    .await
    .map_err(|e| AppError::InternalServerError(format!("Database error: {}", e)))?;

    let entries: Vec<serde_json::Value> = rows.into_iter().map(|row| {
        use sqlx::Row;
        serde_json::json!({
            "id": row.get::<String, _>("id"),
            "missionId": row.get::<Option<String>, _>("mission_id"),
            "toolCall": {
                "id": format!("tc-{}", row.get::<String, _>("id")), // Synthetic toolCall ID for history
                "agentId": row.get::<String, _>("agent_id"),
                "skill": row.get::<Option<String>, _>("skill"),
                "params": serde_json::from_str::<serde_json::Value>(&row.get::<String, _>("params")).unwrap_or_default(),
                "description": row.get::<Option<String>, _>("payload")
                    .unwrap_or_else(|| "Historical action".to_string()),
            },
            "type": row.get::<String, _>("entry_type"),
            "status": row.get::<String, _>("status"),
            "decision": row.get::<Option<String>, _>("decision"),
            "decidedAt": row.get::<Option<chrono::DateTime<chrono::Utc>>, _>("decided_at"),
            "decidedBy": row.get::<Option<String>, _>("decided_by"),
            "createdAt": row.get::<chrono::DateTime<chrono::Utc>, _>("created_at"),
            "payload": serde_json::from_str::<serde_json::Value>(&row.get::<String, _>("payload")).unwrap_or_default(),
        })
    }).collect();

    Ok(Json(PaginatedResponse::from_vec(
        entries,
        &params,
        "/v1/oversight/ledger",
    )))
}

#[derive(serde::Deserialize)]
pub struct OversightSettingsPayload {
    #[serde(rename = "autoApproveSafeSkills")]
    pub auto_approve_safe_skills: bool,
    #[serde(rename = "maxAgents")]
    pub max_agents: Option<u32>,
    #[serde(rename = "maxClusters")]
    pub max_clusters: Option<u32>,
    #[serde(rename = "maxSwarmDepth")]
    pub max_swarm_depth: Option<u32>,
    #[serde(rename = "maxTaskLength")]
    pub max_task_length: Option<usize>,
    #[serde(rename = "defaultBudgetUsd")]
    pub default_budget_usd: Option<f64>,
}

/// PUT /v1/oversight/settings
/// 
/// Updates global governance constraints, including swarm depth, 
/// auto-approval policies, and budget limitations.
///
/// @docs OPERATIONS_MANUAL:GovernanceSettings
pub async fn update_settings(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<OversightSettingsPayload>,
) -> impl IntoResponse {
    state.governance.auto_approve_safe_skills.store(
        payload.auto_approve_safe_skills,
        std::sync::atomic::Ordering::Relaxed,
    );

    if let Some(val) = payload.max_agents {
        state.governance
            .max_agents
            .store(val, std::sync::atomic::Ordering::Relaxed);
    }
    if let Some(val) = payload.max_clusters {
        state.governance
            .max_clusters
            .store(val, std::sync::atomic::Ordering::Relaxed);
    }
    if let Some(val) = payload.max_swarm_depth {
        state.governance
            .max_swarm_depth
            .store(val, std::sync::atomic::Ordering::Relaxed);
    }
    if let Some(val) = payload.max_task_length {
        state.governance
            .max_task_length
            .store(val, std::sync::atomic::Ordering::Relaxed);
    }
    if let Some(val) = payload.default_budget_usd {
        *state.governance.default_budget_usd.write() = val;
    }

    tracing::info!(
        "🛡️ Governance updated: Auto-Approve={}, MaxAgents={:?}, MaxClusters={:?}, MaxDepth={:?}",
        payload.auto_approve_safe_skills,
        payload.max_agents,
        payload.max_clusters,
        payload.max_swarm_depth
    );

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "status": "ok",
            "autoApproveSafeSkills": payload.auto_approve_safe_skills,
            "maxAgents": state.governance.max_agents.load(std::sync::atomic::Ordering::Relaxed),
            "maxClusters": state.governance.max_clusters.load(std::sync::atomic::Ordering::Relaxed),
            "maxSwarmDepth": state.governance.max_swarm_depth.load(std::sync::atomic::Ordering::Relaxed),
            "maxTaskLength": state.governance.max_task_length.load(std::sync::atomic::Ordering::Relaxed),
            "defaultBudgetUsd": *state.governance.default_budget_usd.read()
        })),
    )
}

/// POST /v1/oversight/:id/decide
/// 
/// Commits a human decision (Approve/Reject) for a specific oversight request.
/// Triggers the internal resolution channel to unblock or kill the agent task.
///
/// @docs API_REFERENCE:DecideOversight
pub async fn decide_oversight(
    Path(entry_id): Path<String>,
    State(state): State<Arc<AppState>>,
    Json(payload): Json<OversightDecision>,
) -> Result<impl IntoResponse, AppError> {
    let approved = payload.decision == "approved";

    tracing::info!(
        "⚖️ [Oversight] Decision received for {}: decision={}",
        entry_id,
        payload.decision
    );

    // 1. Remove from queue
    let removed_entry = state.comms.oversight_queue.remove(&entry_id)
        .map(|(_, entry)| entry);

    if let Some(entry) = removed_entry {
        // 2. Resolve the waiting promise
        if let Some((_, shooter)) = state.comms.oversight_resolvers.remove(&entry_id) {
            let _ = shooter.send(approved);
        }

        // 3. Log the decision
        let decision_label = if approved { "APPROVED" } else { "REJECTED" };
        state.broadcast_sys(
            &format!(
                "⚖️ Oversight: Decision for {} recorded as {}.",
                entry.id,
                decision_label
            ),
            if approved { "success" } else { "warning" },
        );

        // 4. Update audit trail
        let audit = state.security.audit_trail.clone();
        let eid = entry.id.clone();
        let mission_id = entry.mission_id.clone();
        tokio::spawn(async move {
            let params = serde_json::to_string(&json!({
                "entry_id": eid,
                "approved": approved
            })).unwrap_or_default();
            let _ = audit.record(
                "oversight",
                mission_id.as_deref(),
                None,
                "oversight_decision",
                &params,
            ).await;
        });

        // 5. Emit event for UI
        state.emit_event(json!({
            "type": "oversight:decision",
            "data": {
                "id": entry_id,
                "decision": payload.decision
            }
        }));

        Ok(Json(json!({ "status": "ok", "decision": payload.decision })))
    } else {
        Err(AppError::NotFound("Oversight entry not found or already resolved".to_string()))
    }
}
#[derive(serde::Deserialize)]
pub struct UpdateQuotaPayload {
    #[serde(rename = "budgetUsd")]
    pub budget_usd: f64,
    #[serde(rename = "resetPeriod")]
    pub reset_period: Option<crate::security::metering::ResetPeriod>,
}

/// PUT /oversight/security/quotas/:entity_id
/// 
/// Updates the budget quota and reset period for a specific agent.
///
/// @docs API_REFERENCE:UpdateAgentQuota
pub async fn update_agent_quota(
    Path(entity_id): Path<String>,
    State(state): State<Arc<AppState>>,
    Json(payload): Json<UpdateQuotaPayload>,
) -> Result<impl IntoResponse, AppError> {
    state.security.budget_guard.update_quota(
        &entity_id,
        payload.budget_usd,
        payload.reset_period
    ).await
    .map_err(|e| AppError::InternalServerError(format!("Failed to update quota: {}", e)))?;

    tracing::info!("🛡️ [Budget] Quota updated for agent {}: ${}", entity_id, payload.budget_usd);
    Ok((StatusCode::OK, Json(serde_json::json!({ "status": "ok" }))))
}

/// GET /oversight/security/quotas
/// 
/// Returns global budget telemetry, including total spent, remaining,
/// and system defense metrics.
///
/// @docs API_REFERENCE:GetQuotas
pub async fn get_security_quotas(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let mut total_budget = 0.0;
    let mut total_spent = 0.0;

    for entry in state.registry.agents.iter() {
        total_budget += entry.value().budget_usd;
        total_spent += entry.value().cost_usd;
    }

    let agent_quotas: Vec<crate::security::metering::Quota> = state.security.budget_guard.get_all_quotas().await.unwrap_or_default();

    let system_defense = state.security.system_monitor.get_system_defense_stats();
    let merkle_integrity = if state.security.audit_trail.verify_last_n(10, None).await.unwrap_or(false) { 1.0 } else { 0.0 };

    Json(serde_json::json!({
        "totalBudget": total_budget,
        "totalSpent": total_spent,
        "remaining": total_budget - total_spent,
        "efficiency": if total_budget > 0.0 { (total_spent / total_budget) * 100.0 } else { 0.0 },
        "agentQuotas": agent_quotas,
        "systemDefense": {
            "memoryPressure": system_defense.memory_pressure,
            "cpuLoad": system_defense.cpu_load,
            "sandboxStatus": system_defense.sandbox_status,
            "sandboxType": system_defense.sandbox_type,
            "merkleIntegrity": merkle_integrity
        }
    }))
}

/// GET /oversight/security/audit-trail
/// 
/// Retrieves the tamper-evident Merkle hash-chain logs.
///
/// @docs API_REFERENCE:GetAuditTrail
pub async fn get_audit_trail(
    State(state): State<Arc<AppState>>,
    Query(params): Query<PaginationParams>,
) -> Result<impl IntoResponse, AppError> {
    let (page, per_page) = params.sanitize();
    let limit = per_page as i32;
    let offset = ((page as i32) - 1) * limit;

    // We pull directly from audit_trail instead of oversight_log for "top-tier" integrity
    let entries: Vec<AuditEntry> = sqlx::query_as(
        "SELECT * FROM audit_trail ORDER BY timestamp DESC LIMIT ? OFFSET ?"
    )
    .bind(limit)
    .bind(offset)
    .fetch_all(&state.resources.pool)
    .await
    .map_err(|e| AppError::InternalServerError(format!("Database error: {}", e)))?;

    let response: Vec<OversightAuditEntry> = entries.into_iter().map(|entry| {
        let is_verified = state.security.audit_trail.verify_record(&entry);
        OversightAuditEntry {
            id: entry.id,
            agent_id: entry.agent_id,
            skill: Some(entry.action),
            status: "recorded".to_string(), // Merkle chain status
            decision: None,
            decided_at: None,
            created_at: entry.timestamp,
            is_verified,
        }
    }).collect();

    Ok(Json(PaginatedResponse::from_vec(
        response,
        &params,
        "/v1/oversight/security/audit-trail",
    )))
}

/// GET /v1/oversight/security/integrity
/// 
/// Verifies the last N records in the Merkle chain and returns an integrity score.
///
/// @docs API_REFERENCE:GetIntegrityStatus
pub async fn get_integrity_status(
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, AppError> {
    let result = state.security.audit_trail.verify_last_n(50, None).await
        .map_err(|e| AppError::InternalServerError(format!("Verification failed: {}", e)))?;
    
    // For now, we return 1.0 if verified, 0.0 if not.
    // In a future update, verify_last_n could return exact counts.
    Ok(Json(SecurityIntegrityResponse {
        integrity_score: if result { 1.0 } else { 0.0 },
        status: if result { "SECURE".to_string() } else { "TAMPERED".to_string() },
        verified_count: if result { 50 } else { 0 }, // Simplified
        total_count: 50,
    }))
}

/// GET /oversight/security/health
/// 
/// Returns health metrics for all registered agents.
///
/// @docs OPERATIONS_MANUAL:AgentHealth
pub async fn get_agent_health(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let health_data: Vec<serde_json::Value> = state.registry.agents.iter().map(|entry| {
        let agent = entry.value();
        serde_json::json!({
            "agentId": agent.id,
            "name": agent.name,
            "status": agent.status,
            "failureCount": agent.failure_count,
            "lastFailureAt": agent.last_failure_at,
            "isHealthy": agent.failure_count < 5,
            "isThrottled": agent.failure_count >= 3,
        })
    }).collect();

    Json(serde_json::json!({ "agents": health_data }))
}

//! @docs ARCHITECTURE:Networking
//!
//! ### AI Assist Note
//! **Governance & Oversight Gateway**: Orchestrates the REST surface
//! for **Human-in-the-Loop** verification and global engine
//! constraints. Features **Oversight Decision Flows**: manages the
//! unblocking and termination of autonomous agent tasks based on
//! human approval. Implements **Merkle Hash-Chain Recording**: every
//! oversight decision is committed to a tamper-evident audit ledger
//! with digital signature verification. AI agents should monitor the
//! `oversight_queue` and provide clear technical context for all
//! pending decisions to minimize human review friction (GOV-01).
//!
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: 404 on resolution due to resolver timeout,
//!   decisions stalling in the queue due to missing human input, or
//!   Merkle integrity failures on tampered log entries.
//! - **Telemetry Link**: Search for `⚖️ [Oversight]` in `tracing` logs
//!   for decision lifecycle events.
//! - **Trace Scope**: `server-rs::routes::oversight`

use crate::agent::types::{OversightDecision, OversightEntry};
use crate::error::AppError;
use crate::routes::pagination::{PaginatedResponse, PaginationParams};
use crate::security::audit::AuditEntry;
use crate::security::metering::{Quota, ResetPeriod};
use crate::state::AppState;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::Serialize;
use serde_json::json;
use std::sync::Arc;

const OVS_SIG_WINDOW_MS: i64 = 300_000;
const DB_QUERY_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(5);

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

/// GET /v1/oversight/pending
///
/// Returns a collection of all actions (file edits, network requests, etc.)
/// currently paused and awaiting human verification.
///
/// @docs API_REFERENCE:GetPendingOversight
#[tracing::instrument(skip(state), name = "governance::list_pending")]
pub async fn get_pending(
    State(state): State<Arc<AppState>>,
    Query(params): Query<PaginationParams>,
) -> Result<impl IntoResponse, AppError> {
    let total = state.comms.oversight_queue.len() as u32;
    let offset = params.offset();
    let (_, per_page) = params.sanitize();
    let limit = per_page as usize;

    let entries: Vec<OversightEntry> = state
        .comms
        .oversight_queue
        .iter()
        .skip(offset)
        .take(limit)
        .map(|entry| entry.value().clone())
        .collect();

    Ok(Json(PaginatedResponse::from_pre_sliced(
        entries,
        total,
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
#[tracing::instrument(skip(state), name = "governance::list_ledger")]
pub async fn get_ledger(
    State(state): State<Arc<AppState>>,
    Query(params): Query<PaginationParams>,
) -> Result<impl IntoResponse, AppError> {
    let (page, per_page) = params.sanitize();
    let limit = per_page as i32;
    let offset = ((page as i32) - 1) * limit;

    let rows = tokio::time::timeout(
        DB_QUERY_TIMEOUT,
        sqlx::query(
            "SELECT id, mission_id, agent_id, entry_type, skill, params, status, decision, decided_at, decided_by, created_at, payload 
             FROM oversight_log 
             WHERE status != 'pending' 
             ORDER BY created_at DESC 
             LIMIT ? OFFSET ?"
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&state.resources.pool),
    )
    .await
    .map_err(|_| AppError::InternalServerError("Database query timed out".to_string()))?
    .map_err(AppError::Sqlx)?;

    let entries: Vec<serde_json::Value> = rows.into_iter().map(|row| {
        use sqlx::Row;
        serde_json::json!({
            "id": row.get::<String, _>("id"),
            "mission_id": row.get::<Option<String>, _>("mission_id"),
            "tool_call": {
                "id": format!("tc-{}", row.get::<String, _>("id")), // Synthetic toolCall ID for history
                "agent_id": row.get::<String, _>("agent_id"),
                "skill": row.get::<Option<String>, _>("skill"),
                "params": serde_json::from_str::<serde_json::Value>(&row.get::<String, _>("params")).unwrap_or_default(),
                "description": row.get::<Option<String>, _>("payload")
                    .unwrap_or_else(|| "Historical action".to_string()),
            },
            "type": row.get::<String, _>("entry_type"),
            "status": row.get::<String, _>("status"),
            "decision": row.get::<Option<String>, _>("decision"),
            "decided_at": row.get::<Option<chrono::DateTime<chrono::Utc>>, _>("decided_at"),
            "decided_by": row.get::<Option<String>, _>("decided_by"),
            "created_at": row.get::<chrono::DateTime<chrono::Utc>, _>("created_at"),
            "payload": serde_json::from_str::<serde_json::Value>(&row.get::<String, _>("payload")).unwrap_or_default(),
        })
    }).collect();

    Ok(Json(PaginatedResponse::from_vec(
        entries,
        &params,
        "/v1/oversight/ledger",
    )))
}

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
pub struct OversightSettingsPayload {
    pub auto_approve_safe_skills: bool,
    pub privacy_mode: Option<bool>,
    pub max_agents: Option<u32>,
    pub max_clusters: Option<u32>,
    pub max_swarm_depth: Option<u32>,
    pub max_task_length: Option<usize>,
    pub default_budget_usd: Option<f64>,
}

/// PUT /v1/oversight/settings
///
/// Updates global governance constraints, including swarm depth,
/// auto-approval policies, and budget limitations.
///
/// @docs OPERATIONS_MANUAL:GovernanceSettings
#[tracing::instrument(skip(state, payload), name = "governance::update_settings")]
pub async fn update_settings(
    State(state): State<Arc<AppState>>,
    _admin: crate::middleware::auth::RequireAdmin,
    Json(payload): Json<OversightSettingsPayload>,
) -> Result<impl IntoResponse, AppError> {
    state.governance.auto_approve_safe_skills.store(
        payload.auto_approve_safe_skills,
        std::sync::atomic::Ordering::Release,
    );

    if let Some(val) = payload.privacy_mode {
        state
            .governance
            .privacy_mode
            .store(val, std::sync::atomic::Ordering::Release);

        if val {
            tracing::info!("🛡️ Privacy Shield ENABLED: Hard-blocking external API requests (OpenAI/Anthropic/Google).");
        }

        state.broadcast_sys(
            &format!(
                "🛡️ Privacy Shield: Mode set to {}. {}",
                if val {
                    "ON (Local-First)"
                } else {
                    "OFF (Hybrid)"
                },
                if val {
                    "External APIs are hard-blocked."
                } else {
                    ""
                }
            ),
            if val { "success" } else { "info" },
            None,
        );
    }

    if let Some(val) = payload.max_agents {
        state
            .governance
            .max_agents
            .store(val, std::sync::atomic::Ordering::Release);
    }
    if let Some(val) = payload.max_clusters {
        state
            .governance
            .max_clusters
            .store(val, std::sync::atomic::Ordering::Release);
    }
    if let Some(val) = payload.max_swarm_depth {
        state
            .governance
            .max_swarm_depth
            .store(val, std::sync::atomic::Ordering::Release);
    }
    if let Some(val) = payload.max_task_length {
        state
            .governance
            .max_task_length
            .store(val, std::sync::atomic::Ordering::Release);
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
    let state_clone = state.clone();
    tokio::time::timeout(
        DB_QUERY_TIMEOUT,
        tokio::task::spawn_blocking(move || {
            state_clone.save_governance_settings();
        })
    )
    .await
    .map_err(|_| AppError::InternalServerError("Saving settings timed out".to_string()))?
    .map_err(|e| AppError::InternalServerError(format!("Spawn blocking failed: {}", e)))?;

    Ok((
        StatusCode::OK,
        Json(serde_json::json!({
            "status": "ok",
            "auto_approve_safe_skills": payload.auto_approve_safe_skills,
            "max_agents": state.governance.max_agents.load(std::sync::atomic::Ordering::Acquire),
            "max_clusters": state.governance.max_clusters.load(std::sync::atomic::Ordering::Acquire),
            "max_swarm_depth": state.governance.max_swarm_depth.load(std::sync::atomic::Ordering::Acquire),
            "max_task_length": state.governance.max_task_length.load(std::sync::atomic::Ordering::Acquire),
            "default_budget_usd": *state.governance.default_budget_usd.read(),
            "privacy_mode": state.governance.privacy_mode.load(std::sync::atomic::Ordering::Acquire)
        })),
    ))
}

pub(crate) fn verify_oversight_signature(
    entry_id: &str,
    decision: &str,
    signature_hex: &str,
    pubkey_hex: &str,
    timestamp: Option<i64>,
    nonce: Option<&str>,
) -> Result<(), String> {
    use ed25519_dalek::{Signature, Verifier, VerifyingKey};

    let sig_bytes =
        hex::decode(signature_hex).map_err(|e| format!("Invalid signature hex: {}", e))?;
    let pubkey_bytes =
        hex::decode(pubkey_hex).map_err(|e| format!("Invalid public key hex: {}", e))?;

    if let Ok(pinned_key) = std::env::var("OVERSIGHT_PUBLIC_KEY") {
        if pubkey_hex != pinned_key {
            return Err("Provided public key does not match the pinned OVERSIGHT_PUBLIC_KEY".to_string());
        }
    } else {
        tracing::warn!("⚠️ OVERSIGHT_PUBLIC_KEY is not configured in environment. Oversight signatures are verified but NOT pinned to an authorized operator!");
    }

    let signature = Signature::from_bytes(
        &sig_bytes
            .try_into()
            .map_err(|_| "Signature must be 64 bytes".to_string())?,
    );
    let verifying_key = VerifyingKey::from_bytes(
        &pubkey_bytes
            .try_into()
            .map_err(|_| "Public key must be 32 bytes".to_string())?,
    )
    .map_err(|e| format!("Invalid verifying key: {}", e))?;

    let ts = timestamp.ok_or_else(|| "Missing signature timestamp".to_string())?;
    let n = nonce.ok_or_else(|| "Missing signature nonce".to_string())?;

    // Enforce timestamp window: ±5 minutes
    let now_ms_u: u64 = chrono::Utc::now().timestamp_millis().max(0) as u64;
    let ts_u: u64 = ts.max(0) as u64;
    let diff = now_ms_u.abs_diff(ts_u);
    if diff > OVS_SIG_WINDOW_MS as u64 {
        return Err("Signature timestamp is expired or too far in the future".to_string());
    }
    let payload_str = format!("ovs:v1|{}|{}|{}|{}", entry_id, decision, ts, n);

    verifying_key
        .verify(payload_str.as_bytes(), &signature)
        .map_err(|e| format!("Signature verification failed: {}", e))?;

    Ok(())
}

pub(crate) async fn resolve_oversight_decision(
    state: &AppState,
    entry_id: &str,
    payload: &OversightDecision,
) -> Result<serde_json::Value, AppError> {
    if let (Some(sig), Some(pk)) = (&payload.signature, &payload.verifying_key) {
        if let Err(e) = verify_oversight_signature(
            entry_id,
            &payload.decision,
            sig,
            pk,
            payload.timestamp,
            payload.nonce.as_deref(),
        ) {
            return Err(AppError::Forbidden(format!(
                "Cryptographic signature verification failed: {}",
                e
            )));
        }
    } else {
        return Err(AppError::Forbidden("Cryptographic signature and verifying public key are required for oversight decisions.".to_string()));
    }

    let approved = payload.decision == "approved";

    tracing::info!(
        "⚖️ [Oversight] Decision received for {}: decision={}",
        entry_id,
        payload.decision
    );

    // 1. Remove from queue first. If entry is not in the queue, we return 404
    // without executing any DB operations (preventing nonce-burning on non-existent IDs).
    let removed_entry = state
        .comms
        .oversight_queue
        .remove(entry_id)
        .map(|(_, entry)| entry);

    let entry = match removed_entry {
        Some(e) => e,
        None => {
            return Err(AppError::NotFound(
                "Oversight entry not found or already resolved".to_string(),
            ));
        }
    };

    // 2. Prevent replay attacks. If the DB operation fails, we restore the entry
    // back to the queue so the request can be retried safely.
    if let Some(ref nonce) = payload.nonce {
        if let Some(ts) = payload.timestamp {
            let res = tokio::time::timeout(
                DB_QUERY_TIMEOUT,
                sqlx::query("INSERT INTO used_nonces (nonce, timestamp) VALUES (?, ?)")
                    .bind(nonce)
                    .bind(ts)
                    .execute(&state.resources.pool),
            )
            .await
            .map_err(|_| {
                // Restore queue state on DB timeout
                state.comms.oversight_queue.insert(entry_id.to_string(), entry.clone());
                AppError::InternalServerError("Database nonce verification timed out".to_string())
            })?;

            if let Err(e) = res {
                let is_unique_violation = if let Some(db_err) = e.as_database_error() {
                    db_err.code().map(|c| c == "2067" || c == "1555" || c == "23000").unwrap_or(false)
                } else {
                    false
                } || e.to_string().contains("UNIQUE constraint failed");

                if is_unique_violation {
                    // UNIQUE violation (replay attack) -> do NOT restore the queue
                    return Err(AppError::Forbidden("Replay attack detected: nonce already used".to_string()));
                }

                // For other transient database errors, restore the queue state
                state.comms.oversight_queue.insert(entry_id.to_string(), entry);
                return Err(AppError::Sqlx(e));
            }
        }
    }

    // 3. Resolve the waiting promise
    if let Some((_, shooter)) = state.comms.oversight_resolvers.remove(entry_id) {
        let _ = shooter.send(crate::agent::types::OversightResolution {
            approved,
            override_slot: payload.override_slot.clone(),
        });
    }

    // Update database record in oversight_log table
    let now = chrono::Utc::now();
    let status_val = if approved { "approved" } else { "rejected" };
    let decision_val = if approved { "approved" } else { "rejected" };
    let decided_by_val = "user";

    let db_query = sqlx::query(
        "UPDATE oversight_log 
         SET status = ?, decision = ?, decided_at = ?, decided_by = ? 
         WHERE id = ?"
    )
    .bind(status_val)
    .bind(decision_val)
    .bind(now)
    .bind(decided_by_val)
    .bind(entry_id);

    tokio::time::timeout(DB_QUERY_TIMEOUT, db_query.execute(&state.resources.pool))
        .await
        .map_err(|_| AppError::InternalServerError("Database update timed out".to_string()))?
        .map_err(AppError::Sqlx)?;

    // 4. Log the decision
    let decision_label = if approved { "APPROVED" } else { "REJECTED" };
    state.broadcast_sys(
        &format!(
            "⚖️ Oversight: Decision for {} recorded as {}.",
            entry.id, decision_label
        ),
        if approved { "success" } else { "warning" },
        entry.mission_id.clone(),
    );

    // 5. Update audit trail
    let audit = state.security.audit_trail.clone();
    let eid = entry.id.clone();
    let mission_id = entry.mission_id.clone();
    tokio::spawn(async move {
        let params = serde_json::to_string(&json!({
            "entry_id": eid,
            "approved": approved
        }))
        .unwrap_or_default();
        let record_fut = audit.record(
            "oversight",
            mission_id.as_deref(),
            None,
            "oversight_decision",
            &params,
        );
        if let Ok(res) = tokio::time::timeout(DB_QUERY_TIMEOUT, record_fut).await {
            if let Err(e) = res {
                tracing::error!("🚨 Failed to record audit log: {:?}", e);
            }
        } else {
            tracing::error!("🚨 Audit log recording timed out");
        }
    });

    // 6. Emit event for UI
    state.emit_event(json!({
        "type": "oversight:decision",
        "data": {
            "id": entry_id,
            "decision": payload.decision
        }
    }));

    Ok(json!({ "status": "ok", "decision": payload.decision }))
}

/// POST /v1/oversight/:id/decide
///
/// Commits a human decision (Approve/Reject) for a specific oversight request.
/// Triggers the internal resolution channel to unblock or kill the agent task.
///
/// @docs API_REFERENCE:DecideOversight
#[tracing::instrument(skip(state), fields(action_id = %entry_id, decision = %payload.decision), name = "governance::resolve_pending")]
pub async fn decide_oversight(
    Path(entry_id): Path<String>,
    State(state): State<Arc<AppState>>,
    Json(payload): Json<OversightDecision>,
) -> Result<impl IntoResponse, AppError> {
    let res = resolve_oversight_decision(&state, &entry_id, &payload).await?;
    Ok((StatusCode::OK, Json(res)))
}

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
#[tracing::instrument(skip(state), name = "governance::update_agent_quota")]
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
#[tracing::instrument(skip(state), name = "security_governance::get_quotas")]
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
#[tracing::instrument(skip(state), name = "security_governance::get_mission_quotas")]
pub async fn get_mission_quotas(
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, AppError> {
    let rows = tokio::time::timeout(
        DB_QUERY_TIMEOUT,
        sqlx::query("SELECT * FROM mission_quotas")
            .fetch_all(&state.resources.pool),
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

            Quota {
                id: r.get("id"),
                entity_id: r.get("cluster_id"),
                budget_usd: r.get("budget_usd"),
                used_usd: r.get("used_usd"),
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

/// GET /oversight/security/audit-trail
///
/// Retrieves the tamper-evident Merkle hash-chain logs.
///
/// @docs API_REFERENCE:GetAuditTrail
#[tracing::instrument(skip(state), name = "governance::get_audit_trail")]
pub async fn get_audit_trail(
    State(state): State<Arc<AppState>>,
    Query(params): Query<PaginationParams>,
) -> Result<impl IntoResponse, AppError> {
    let (page, per_page) = params.sanitize();
    let limit = per_page as i32;
    let offset = ((page as i32) - 1) * limit;

    // We pull directly from audit_trail instead of oversight_log for "top-tier" integrity
    let entries: Vec<AuditEntry> = tokio::time::timeout(
        DB_QUERY_TIMEOUT,
        sqlx::query_as("SELECT * FROM audit_trail ORDER BY timestamp DESC LIMIT ? OFFSET ?")
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
                status: "recorded".to_string(), // Merkle chain status
                decision: None,
                decided_at: None,
                created_at: entry.timestamp,
                is_verified,
            }
        })
        .collect();

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
#[tracing::instrument(skip(state), name = "governance::get_integrity")]
pub async fn get_integrity_status(
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, AppError> {
    let verify_res = tokio::time::timeout(
        DB_QUERY_TIMEOUT,
        state.security.audit_trail.verify_last_n(50, None),
    )
    .await;
    let (verified_count, total_count) = match verify_res {
        Ok(Ok((v, t))) => (v, t),
        _ => (0, 50),
    };

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

/// GET /oversight/security/health
///
/// Returns health metrics for all registered agents.
///
/// @docs OPERATIONS_MANUAL:AgentHealth
#[tracing::instrument(skip(state), name = "security_governance::get_health")]
pub async fn get_agent_health(
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, AppError> {
    let health_data: Vec<serde_json::Value> = state
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
                "is_healthy": agent.health.failure_count < 5,
                "is_throttled": agent.health.failure_count >= 3,
                "is_bankrupt": agent.economics.cost_usd >= agent.economics.budget_usd && agent.economics.budget_usd > 0.0,
            })
        })
        .collect();

    Ok(Json(serde_json::json!({ "agents": health_data })))
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

/// GET /v1/oversight/security/policies
#[tracing::instrument(skip(state), name = "governance::get_policies")]
pub async fn get_policies(
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, AppError> {
    let rows: Vec<(String, String)> = tokio::time::timeout(
        DB_QUERY_TIMEOUT,
        sqlx::query_as("SELECT tool_name, mode FROM permission_policies")
            .fetch_all(&state.resources.pool),
    )
    .await
    .map_err(|_| AppError::InternalServerError("Database query timed out".to_string()))?
    .map_err(AppError::Sqlx)?;

    let policies: Vec<serde_json::Value> = rows
        .into_iter()
        .map(|(name, mode)| json!({ "tool_name": name, "mode": mode }))
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
        .bind(&payload.tool_name)
        .bind(mode_str)
        .execute(&state.resources.pool),
    )
    .await
    .map_err(|_| AppError::InternalServerError("Database write timed out".to_string()))?
    .map_err(AppError::Sqlx)?;

    // Refresh cache with timeout
    tokio::time::timeout(
        DB_QUERY_TIMEOUT,
        state.security.permission_policy.refresh_cache(),
    )
    .await
    .map_err(|_| AppError::InternalServerError("Refreshing permission cache timed out".to_string()))?
    .map_err(|e| {
        AppError::InternalServerError(format!("Failed to refresh permission cache: {}", e))
    })?;

    let mode_str = match payload.mode {
        PolicyMode::Allow => "allow",
        PolicyMode::Deny => "deny",
        PolicyMode::Prompt => "prompt",
    };

    state.broadcast_sys(
        &format!(
            "🛡️ Security Policy Updated: {} -> {}",
            payload.tool_name, mode_str
        ),
        "info",
        None,
    );

    Ok((StatusCode::OK, Json(json!({ "status": "ok" }))))
}

// Metadata: [oversight]

// Metadata: [oversight]

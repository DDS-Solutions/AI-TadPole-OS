//! @docs ARCHITECTURE:Networking
//!
//! ### AI Assist Note
//! **Oversight Ledger & Decision Handlers**: Implements the pending queue,
//! historical ledger retrieval, governance settings update, and the
//! cryptographic oversight decision resolution pipeline.
//!
//! ### 🔍 Debugging & Observability
//! - **Trace Scope**: `server-rs::routes::oversight::ledger`

use crate::agent::types::{OversightDecision, OversightEntry};
use crate::error::AppError;
use crate::routes::pagination::{PaginatedResponse, PaginationParams};
use crate::state::AppState;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde_json::json;
use std::sync::Arc;

use super::{is_production_env, DB_QUERY_TIMEOUT, OVS_SIG_WINDOW_MS};

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
pub struct OversightSettingsPayload {
    pub auto_approve_safe_skills: bool,
    pub privacy_mode: Option<bool>,
    pub cluster_privacy_policies: Option<std::collections::HashMap<String, bool>>,
    pub max_agents: Option<u32>,
    pub max_clusters: Option<u32>,
    pub max_swarm_depth: Option<u32>,
    pub max_task_length: Option<usize>,
    pub default_budget_usd: Option<f64>,
    pub default_model: Option<String>,
    pub default_provider: Option<String>,
}

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
#[tracing::instrument(skip(state), fields(trace_id = tracing::field::Empty, request_id = tracing::field::Empty, http_status = tracing::field::Empty), name = "governance::list_ledger")]
pub async fn get_ledger(
    State(state): State<Arc<AppState>>,
    Query(params): Query<PaginationParams>,
) -> Result<impl IntoResponse, AppError> {
    let (page, per_page) = params.sanitize();
    let limit = per_page as i32;
    let offset = ((page as i32) - 1) * limit;

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
        .bind(limit)
        .bind(offset)
        .fetch_all(&state.resources.pool),
    )
    .await
    .map_err(|_| AppError::InternalServerError("Database query timed out".to_string()))?
    .map_err(AppError::Sqlx)?;

    let entries: Vec<serde_json::Value> = rows.into_iter().map(|row| {
        use sqlx::Row;
        let status_str = row.get::<String, _>("status");
        let decision_str = row.get::<Option<String>, _>("decision").unwrap_or_default();
        let decided_by_str = row.get::<Option<String>, _>("decided_by").unwrap_or_default();
        let is_auto = decision_str == "auto_approved" 
            || status_str == "auto_approved" 
            || decided_by_str == "auto_policy" 
            || decided_by_str == "system";

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
            "status": status_str,
            "decision": row.get::<Option<String>, _>("decision"),
            "decided_at": row.get::<Option<chrono::DateTime<chrono::Utc>>, _>("decided_at"),
            "decided_by": row.get::<Option<String>, _>("decided_by"),
            "auto_approved": is_auto,
            "approval_type": if is_auto { "auto" } else { "hitl" },
            "requires_oversight": !is_auto,
            "created_at": row.get::<chrono::DateTime<chrono::Utc>, _>("created_at"),
            "timestamp": row.get::<chrono::DateTime<chrono::Utc>, _>("created_at"),
            "payload": serde_json::from_str::<serde_json::Value>(&row.get::<String, _>("payload")).unwrap_or_default(),
        })
    }).collect();

    // Use from_pre_sliced — SQL already applied LIMIT/OFFSET, so we provide the real total
    Ok(Json(PaginatedResponse::from_pre_sliced(
        entries,
        total as u32,
        &params,
        "/v1/oversight/ledger",
    )))
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
            tracing::info!(
                "🛡️ Privacy Shield ENABLED: Hard-blocking external API requests (OpenAI/Anthropic/Google)."
            );
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

    if let Some(policies) = payload.cluster_privacy_policies.clone() {
        for (cluster_id, mode) in policies {
            state
                .governance
                .cluster_privacy_policies
                .insert(cluster_id.clone(), mode);
            tracing::info!(
                "🛡️ Cluster Privacy Shield updated: cluster '{}' privacy_mode={}",
                cluster_id,
                mode
            );
        }
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
    let default_model_val = if let Some(ref val) = payload.default_model {
        *state.governance.default_model.write() = val.clone();
        val.clone()
    } else {
        state.governance.default_model.read().clone()
    };
    let default_provider_val = if let Some(ref val) = payload.default_provider {
        *state.governance.default_provider.write() = val.clone();
        val.clone()
    } else {
        state.governance.default_provider.read().clone()
    };

    tracing::info!(
        "🛡️ Governance updated: Auto-Approve={}, MaxAgents={:?}, MaxClusters={:?}, MaxDepth={:?}, DefaultModel={}, DefaultProvider={}",
        payload.auto_approve_safe_skills,
        payload.max_agents,
        payload.max_clusters,
        payload.max_swarm_depth,
        default_model_val,
        default_provider_val
    );
    let state_clone = state.clone();
    tokio::time::timeout(
        DB_QUERY_TIMEOUT,
        tokio::task::spawn_blocking(move || {
            state_clone.save_governance_settings();
        }),
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
            "default_model": state.governance.default_model.read().clone(),
            "default_provider": state.governance.default_provider.read().clone(),
            "privacy_mode": state.governance.privacy_mode.load(std::sync::atomic::Ordering::Acquire)
        })),
    ))
}

/// C-03: Verifies an ed25519 oversight decision signature.
///
/// `pinned_key` is the hex-encoded public key loaded from `OVERSIGHT_PUBLIC_KEY`
/// at startup (via `state.security.oversight_public_key`). When `Some`, only
/// the pinned key is accepted. When `require_pinned_key` is true, missing
/// startup pinning fails closed before any signature is accepted.
pub(crate) fn verify_oversight_signature(
    entry_id: &str,
    decision: &str,
    signature_hex: &str,
    pubkey_hex: &str,
    timestamp: Option<i64>,
    nonce: Option<&str>,
    pinned_key: Option<&str>,
    require_pinned_key: bool,
) -> Result<(), String> {
    use ed25519_dalek::{Signature, Verifier, VerifyingKey};

    let sig_bytes =
        hex::decode(signature_hex).map_err(|e| format!("Invalid signature hex: {}", e))?;
    let pubkey_bytes =
        hex::decode(pubkey_hex).map_err(|e| format!("Invalid public key hex: {}", e))?;

    if require_pinned_key && pinned_key.is_none() {
        return Err(
            "OVERSIGHT_PUBLIC_KEY is required in production for oversight decisions".to_string(),
        );
    }

    // C-03: Check against pinned key from state (loaded once at startup).
    // Dev/test direct calls can still fall back to env var for backward compatibility.
    let effective_pinned = pinned_key.map(str::to_string).or_else(|| {
        if require_pinned_key {
            None
        } else {
            std::env::var("OVERSIGHT_PUBLIC_KEY").ok()
        }
    });

    if let Some(ref pinned) = effective_pinned {
        if pubkey_hex != pinned.as_str() {
            return Err(
                "Provided public key does not match the pinned OVERSIGHT_PUBLIC_KEY".to_string(),
            );
        }
    } else {
        tracing::warn!(
            "⚠️ OVERSIGHT_PUBLIC_KEY is not configured in environment. Oversight signatures are verified but NOT pinned to an authorized operator!"
        );
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
    if is_production_env() {
        if let (Some(sig), Some(pk)) = (&payload.signature, &payload.verifying_key) {
            if let Err(e) = verify_oversight_signature(
                entry_id,
                &payload.decision,
                sig,
                pk,
                payload.timestamp,
                payload.nonce.as_deref(),
                state.security.oversight_public_key.as_deref(),
                true,
            ) {
                return Err(AppError::Forbidden(format!(
                    "Cryptographic signature verification failed: {}",
                    e
                )));
            }
        } else {
            return Err(AppError::Forbidden("Cryptographic signature and verifying public key are required for oversight decisions in production.".to_string()));
        }
    } else {
        // Dev/Local mode: Best effort signature verification if credentials are provided,
        // otherwise permit raw unsigned user decisions.
        if let (Some(sig), Some(pk)) = (&payload.signature, &payload.verifying_key) {
            if let Err(e) = verify_oversight_signature(
                entry_id,
                &payload.decision,
                sig,
                pk,
                payload.timestamp,
                payload.nonce.as_deref(),
                state.security.oversight_public_key.as_deref(),
                false,
            ) {
                return Err(AppError::Forbidden(format!(
                    "Cryptographic signature verification failed: {}",
                    e
                )));
            }
        }
    }

    let approved = payload.decision == "approved";

    tracing::info!(
        "⚖️ [Oversight] Decision received for {}: decision={}",
        entry_id,
        payload.decision
    );

    // 1. Check that the entry exists in the queue first.
    if !state.comms.oversight_queue.contains_key(entry_id) {
        return Err(AppError::NotFound(
            "Oversight entry not found or already resolved".to_string(),
        ));
    }

    // 2. Prevent replay attacks via database nonce checking BEFORE queue removal.
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
                AppError::InternalServerError("Database nonce verification timed out".to_string())
            })?;

            if let Err(e) = res {
                let is_unique_violation = if let Some(db_err) = e.as_database_error() {
                    db_err
                        .code()
                        .map(|c| c == "2067" || c == "1555" || c == "23000")
                        .unwrap_or(false)
                } else {
                    false
                } || e.to_string().contains("UNIQUE constraint failed");

                if is_unique_violation {
                    // UNIQUE violation (replay attack) -> entry remains untouched in queue
                    return Err(AppError::Forbidden(
                        "Replay attack detected: nonce already used".to_string(),
                    ));
                }

                return Err(AppError::Sqlx(e));
            }
        }
    }

    // 3. Remove from queue ONLY after all security and replay validations pass.
    let entry = state
        .comms
        .oversight_queue
        .remove(entry_id)
        .map(|(_, e)| e)
        .ok_or_else(|| {
            AppError::NotFound("Oversight entry not found or already resolved".to_string())
        })?;

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
         WHERE id = ?",
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
#[tracing::instrument(skip(state), fields(action_id = %entry_id, decision = %payload.decision, trace_id = tracing::field::Empty, request_id = tracing::field::Empty, http_status = tracing::field::Empty), name = "governance::resolve_pending")]
pub async fn decide_oversight(
    Path(entry_id): Path<String>,
    State(state): State<Arc<AppState>>,
    Json(payload): Json<OversightDecision>,
) -> Result<impl IntoResponse, AppError> {
    let res = resolve_oversight_decision(&state, &entry_id, &payload).await?;
    Ok((StatusCode::OK, Json(res)))
}

// Metadata: [oversight::ledger]

//! @docs ARCHITECTURE:Networking
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / HTTP Routes / ledger
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Type-safe state handling and bounded execution without unhandled panics.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: `AppError::BadRequest`, `AppError::Forbidden`, `AppError::NotFound`, `AppError::Sqlx`
//! - **Telemetry Targets**: none declared
//! - **Witness Tests**: `governance::get_settings (state reflection)`, `security::verify_oversight_signature (ed25519 cryptography, timestamp windowing, pinned key parity)`

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
    pub auto_approve_safe_skills: Option<bool>,
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

    let entries: Vec<serde_json::Value> = rows
        .into_iter()
        .filter_map(|row| {
            use sqlx::Row;
            let id: String = row.try_get("id").ok()?;
            let mission_id: Option<String> = row.try_get("mission_id").unwrap_or_default();
            let agent_id: String = row
                .try_get("agent_id")
                .unwrap_or_else(|_| "unknown".to_string());
            let entry_type: String = row
                .try_get("entry_type")
                .unwrap_or_else(|_| "action".to_string());
            let skill: Option<String> = row.try_get("skill").unwrap_or_default();
            let params_raw: String = row.try_get("params").unwrap_or_default();
            let status_str: String = row
                .try_get("status")
                .unwrap_or_else(|_| "unknown".to_string());
            let decision_str: Option<String> = row.try_get("decision").unwrap_or_default();
            let decided_at: Option<chrono::DateTime<chrono::Utc>> =
                row.try_get("decided_at").unwrap_or_default();
            let decided_by_str: Option<String> = row.try_get("decided_by").unwrap_or_default();
            let created_at: Option<chrono::DateTime<chrono::Utc>> =
                row.try_get("created_at").unwrap_or_default();
            let payload_raw: Option<String> = row.try_get("payload").unwrap_or_default();

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

            Some(serde_json::json!({
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

    // Use from_pre_sliced — SQL already applied LIMIT/OFFSET, so we provide the real total safely
    let safe_total = u32::try_from(total.max(0)).unwrap_or(u32::MAX);
    Ok(Json(PaginatedResponse::from_pre_sliced(
        entries,
        safe_total,
        &params,
        "/v1/oversight/ledger",
    )))
}

/// GET /v1/oversight/settings
///
/// Retrieves global governance constraints, including swarm limits,
/// auto-approval policies, and per-cluster privacy shield policies.
///
/// @docs OPERATIONS_MANUAL:GovernanceSettings
#[tracing::instrument(skip(state), name = "governance::get_settings")]
pub async fn get_settings(
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, AppError> {
    let cluster_policies: std::collections::HashMap<String, bool> = state
        .governance
        .cluster_privacy_policies
        .iter()
        .map(|kv| (kv.key().clone(), *kv.value()))
        .collect();

    let payload = OversightSettingsPayload {
        auto_approve_safe_skills: Some(
            state
                .governance
                .auto_approve_safe_skills
                .load(std::sync::atomic::Ordering::Acquire),
        ),
        privacy_mode: Some(
            state
                .governance
                .privacy_mode
                .load(std::sync::atomic::Ordering::Acquire),
        ),
        cluster_privacy_policies: Some(cluster_policies),
        max_agents: Some(
            state
                .governance
                .max_agents
                .load(std::sync::atomic::Ordering::Acquire),
        ),
        max_clusters: Some(
            state
                .governance
                .max_clusters
                .load(std::sync::atomic::Ordering::Acquire),
        ),
        max_swarm_depth: Some(
            state
                .governance
                .max_swarm_depth
                .load(std::sync::atomic::Ordering::Acquire),
        ),
        max_task_length: Some(
            state
                .governance
                .max_task_length
                .load(std::sync::atomic::Ordering::Acquire),
        ),
        default_budget_usd: Some(*state.governance.default_budget_usd.read()),
        default_model: Some(state.governance.default_model.read().clone()),
        default_provider: Some(state.governance.default_provider.read().clone()),
    };

    Ok(Json(payload))
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
    // 1. Validation First (Atomic Guard: never partially apply invalid configurations)
    if let Some(val) = payload.max_agents {
        if !(1..=1000).contains(&val) {
            return Err(AppError::BadRequest(
                "max_agents must be between 1 and 1000".to_string(),
            ));
        }
    }
    if let Some(val) = payload.max_clusters {
        if !(1..=50).contains(&val) {
            return Err(AppError::BadRequest(
                "max_clusters must be between 1 and 50".to_string(),
            ));
        }
    }
    if let Some(val) = payload.max_swarm_depth {
        if !(1..=20).contains(&val) {
            return Err(AppError::BadRequest(
                "max_swarm_depth must be between 1 and 20".to_string(),
            ));
        }
    }
    if let Some(val) = payload.max_task_length {
        if !(100..=1_000_000).contains(&val) {
            return Err(AppError::BadRequest(
                "max_task_length must be between 100 and 1,000,000".to_string(),
            ));
        }
    }
    if let Some(val) = payload.default_budget_usd {
        if !val.is_finite() || !(0.0..=100_000.0).contains(&val) {
            return Err(AppError::BadRequest(
                "default_budget_usd must be a finite non-negative number <= 100,000.0".to_string(),
            ));
        }
    }

    // 2. Apply Validated Settings
    if let Some(val) = payload.auto_approve_safe_skills {
        state
            .governance
            .auto_approve_safe_skills
            .store(val, std::sync::atomic::Ordering::Release);
    }

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

    if let Some(ref policies) = payload.cluster_privacy_policies {
        for (cluster_id, mode) in policies {
            state
                .governance
                .cluster_privacy_policies
                .insert(cluster_id.clone(), *mode);
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
        "🛡️ Governance updated: Auto-Approve={:?}, MaxAgents={:?}, MaxClusters={:?}, MaxDepth={:?}, DefaultModel={}, DefaultProvider={}",
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
            "auto_approve_safe_skills": state.governance.auto_approve_safe_skills.load(std::sync::atomic::Ordering::Acquire),
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
        hex::decode(signature_hex.trim()).map_err(|e| format!("Invalid signature hex: {}", e))?;
    let pubkey_bytes =
        hex::decode(pubkey_hex.trim()).map_err(|e| format!("Invalid public key hex: {}", e))?;

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
        let pinned_bytes = hex::decode(pinned.trim())
            .map_err(|e| format!("Invalid pinned public key hex: {}", e))?;
        if pubkey_bytes != pinned_bytes {
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
    // Validate decision string up-front (H2)
    let approved = match payload.decision.as_str() {
        "approved" => true,
        "rejected" => false,
        other => {
            return Err(AppError::BadRequest(format!(
                "Invalid oversight decision '{}'. Expected 'approved' or 'rejected'.",
                other
            )));
        }
    };

    if let Some(ref slot) = payload.override_slot {
        match slot.as_str() {
            "1" | "2" | "3" | "execution" | "planning" => {}
            other => {
                return Err(AppError::BadRequest(format!(
                    "Invalid override_slot '{}'. Expected '1', '2', '3', 'execution', or 'planning'.",
                    other
                )));
            }
        }
    }

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

    // 3. Update database record in oversight_log table FIRST (H1: durable record before unblocking agent)
    let now = chrono::Utc::now();
    let status_val = if approved { "approved" } else { "rejected" };
    let decision_val = status_val;
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

    let query_result =
        tokio::time::timeout(DB_QUERY_TIMEOUT, db_query.execute(&state.resources.pool))
            .await
            .map_err(|_| AppError::InternalServerError("Database update timed out".to_string()))?
            .map_err(AppError::Sqlx)?;

    if query_result.rows_affected() == 0 {
        tracing::warn!(
            "⚠️ [Oversight] UPDATE oversight_log affected 0 rows for entry_id '{}'",
            entry_id
        );
    }

    // 4. Remove from queue ONLY after all security, replay, and durable DB updates pass.
    let entry = state
        .comms
        .oversight_queue
        .remove(entry_id)
        .map(|(_, e)| e)
        .ok_or_else(|| {
            AppError::NotFound("Oversight entry not found or already resolved".to_string())
        })?;

    // 5. Resolve the waiting promise (unblocks or terminates the agent runner)
    if let Some((_, shooter)) = state.comms.oversight_resolvers.remove(entry_id) {
        let _ = shooter.send(crate::agent::types::OversightResolution {
            approved,
            override_slot: payload.override_slot.clone(),
        });
    }

    // 6. Log decision, update audit trail, and emit WebSocket events
    let decision_label = if approved { "APPROVED" } else { "REJECTED" };
    state.broadcast_sys(
        &format!(
            "⚖️ Oversight: Decision for {} recorded as {}.",
            entry.id, decision_label
        ),
        if approved { "success" } else { "warning" },
        entry.mission_id.clone(),
    );

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

#[cfg(test)]
mod tests {
    use super::*;
    use axum::response::IntoResponse;
    use ed25519_dalek::{Signer, SigningKey};

    async fn fetch_settings_payload(state: &Arc<AppState>) -> OversightSettingsPayload {
        let response = get_settings(State(state.clone()))
            .await
            .expect("get_settings failed");
        let (parts, body) = response.into_response().into_parts();
        assert_eq!(parts.status, StatusCode::OK);

        let bytes = axum::body::to_bytes(body, 64 * 1024)
            .await
            .expect("failed to read body");
        serde_json::from_slice(&bytes).expect("failed to deserialize")
    }

    #[tokio::test]
    async fn test_get_settings_reflects_state() {
        let state = Arc::new(AppState::new_minimal_mock().await);

        // Pre-condition assertion: initial cluster policies should be empty
        let initial = fetch_settings_payload(&state).await;
        let initial_policies = initial.cluster_privacy_policies.unwrap_or_default();
        assert!(
            initial_policies.is_empty(),
            "Expected empty initial policies"
        );

        // Mutate state
        state
            .governance
            .cluster_privacy_policies
            .insert("cl-command".to_string(), false);
        state
            .governance
            .cluster_privacy_policies
            .insert("cl-secure".to_string(), true);

        // Post-mutation assertion
        let updated = fetch_settings_payload(&state).await;
        let policies = updated
            .cluster_privacy_policies
            .expect("missing cluster policies");
        assert_eq!(policies.get("cl-command"), Some(&false));
        assert_eq!(policies.get("cl-secure"), Some(&true));
    }

    #[test]
    fn test_verify_oversight_signature_valid() {
        let signing_key = SigningKey::generate(&mut rand::rng());
        let verifying_key = signing_key.verifying_key();
        let pubkey_hex = hex::encode(verifying_key.as_bytes());

        let entry_id = "act-123";
        let decision = "approved";
        let now = chrono::Utc::now().timestamp_millis();
        let nonce = "nonce-abc-789";

        let payload_str = format!("ovs:v1|{}|{}|{}|{}", entry_id, decision, now, nonce);
        let signature = signing_key.sign(payload_str.as_bytes());
        let signature_hex = hex::encode(signature.to_bytes());

        let result = verify_oversight_signature(
            entry_id,
            decision,
            &signature_hex,
            &pubkey_hex,
            Some(now),
            Some(nonce),
            Some(&pubkey_hex),
            true,
        );

        assert!(result.is_ok(), "Valid signature should pass: {:?}", result);
    }

    #[test]
    fn test_verify_oversight_signature_expired_timestamp() {
        let signing_key = SigningKey::generate(&mut rand::rng());
        let verifying_key = signing_key.verifying_key();
        let pubkey_hex = hex::encode(verifying_key.as_bytes());

        let entry_id = "act-123";
        let decision = "approved";
        // 10 minutes in the past (> 5 min window)
        let expired_ts = chrono::Utc::now().timestamp_millis() - 600_000;
        let nonce = "nonce-abc-789";

        let payload_str = format!("ovs:v1|{}|{}|{}|{}", entry_id, decision, expired_ts, nonce);
        let signature = signing_key.sign(payload_str.as_bytes());
        let signature_hex = hex::encode(signature.to_bytes());

        let result = verify_oversight_signature(
            entry_id,
            decision,
            &signature_hex,
            &pubkey_hex,
            Some(expired_ts),
            Some(nonce),
            Some(&pubkey_hex),
            true,
        );

        assert!(result.is_err(), "Expired timestamp should fail");
        assert!(result.unwrap_err().contains("expired"));
    }

    #[test]
    fn test_verify_oversight_signature_pinned_key_case_insensitivity() {
        let signing_key = SigningKey::generate(&mut rand::rng());
        let verifying_key = signing_key.verifying_key();
        let pubkey_hex = hex::encode(verifying_key.as_bytes());
        let pinned_hex_uppercase = pubkey_hex.to_uppercase();

        let entry_id = "act-123";
        let decision = "approved";
        let now = chrono::Utc::now().timestamp_millis();
        let nonce = "nonce-case-test";

        let payload_str = format!("ovs:v1|{}|{}|{}|{}", entry_id, decision, now, nonce);
        let signature = signing_key.sign(payload_str.as_bytes());
        let signature_hex = hex::encode(signature.to_bytes());

        let result = verify_oversight_signature(
            entry_id,
            decision,
            &signature_hex,
            &pubkey_hex,
            Some(now),
            Some(nonce),
            Some(&pinned_hex_uppercase),
            true,
        );

        assert!(
            result.is_ok(),
            "Decoded byte comparison should handle uppercase hex: {:?}",
            result
        );
    }

    #[test]
    fn test_verify_oversight_signature_pinned_mismatch() {
        let signing_key = SigningKey::generate(&mut rand::rng());
        let verifying_key = signing_key.verifying_key();
        let pubkey_hex = hex::encode(verifying_key.as_bytes());

        let other_key = SigningKey::generate(&mut rand::rng());
        let other_pubkey_hex = hex::encode(other_key.verifying_key().as_bytes());

        let entry_id = "act-123";
        let decision = "approved";
        let now = chrono::Utc::now().timestamp_millis();
        let nonce = "nonce-mismatch-test";

        let payload_str = format!("ovs:v1|{}|{}|{}|{}", entry_id, decision, now, nonce);
        let signature = signing_key.sign(payload_str.as_bytes());
        let signature_hex = hex::encode(signature.to_bytes());

        let result = verify_oversight_signature(
            entry_id,
            decision,
            &signature_hex,
            &pubkey_hex,
            Some(now),
            Some(nonce),
            Some(&other_pubkey_hex),
            true,
        );

        assert!(result.is_err(), "Key mismatch against pinned key must fail");
        assert!(result.unwrap_err().contains("does not match"));
    }

    #[test]
    fn test_verify_oversight_signature_require_pinned_key_fail_closed() {
        let signing_key = SigningKey::generate(&mut rand::rng());
        let verifying_key = signing_key.verifying_key();
        let pubkey_hex = hex::encode(verifying_key.as_bytes());

        let entry_id = "act-123";
        let decision = "approved";
        let now = chrono::Utc::now().timestamp_millis();
        let nonce = "nonce-failclosed-test";

        let payload_str = format!("ovs:v1|{}|{}|{}|{}", entry_id, decision, now, nonce);
        let signature = signing_key.sign(payload_str.as_bytes());
        let signature_hex = hex::encode(signature.to_bytes());

        let result = verify_oversight_signature(
            entry_id,
            decision,
            &signature_hex,
            &pubkey_hex,
            Some(now),
            Some(nonce),
            None,
            true, // require_pinned_key
        );

        assert!(
            result.is_err(),
            "Missing pinned key when required must fail closed"
        );
    }

    #[tokio::test]
    async fn test_resolve_oversight_decision_invalid_decision_string() {
        let state = AppState::new_minimal_mock().await;
        let payload = OversightDecision {
            decision: "Approve".to_string(), // Invalid: capitalized
            signature: None,
            verifying_key: None,
            timestamp: None,
            nonce: None,
            override_slot: None,
        };

        let result = resolve_oversight_decision(&state, "act-test-id", &payload).await;
        match result {
            Err(AppError::BadRequest(msg)) => {
                assert!(msg.contains("Invalid oversight decision"));
            }
            other => panic!(
                "Expected BadRequest for invalid decision string, got: {:?}",
                other
            ),
        }
    }
}

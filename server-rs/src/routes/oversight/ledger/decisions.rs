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

pub(crate) async fn resolve_oversight_decision(
    state: &AppState,
    entry_id: &str,
    payload: &OversightDecision,
) -> Result<serde_json::Value, AppError> {
    resolve_oversight_decision_inner(state, entry_id, payload, None, "user").await
}

/// Commits a remote decision after the paired-device protocol has verified its
/// distinct canonical signature and one-time nonce.
pub(crate) async fn resolve_verified_remote_oversight_decision(
    state: &AppState,
    entry_id: &str,
    payload: &OversightDecision,
    verified_public_key: &str,
    actor: &str,
) -> Result<serde_json::Value, AppError> {
    if payload.verifying_key.as_deref() != Some(verified_public_key) {
        return Err(AppError::Forbidden(
            "Verified remote signer does not match the paired device key".to_string(),
        ));
    }

    resolve_oversight_decision_inner(state, entry_id, payload, Some(verified_public_key), actor)
        .await
}

async fn resolve_oversight_decision_inner(
    state: &AppState,
    entry_id: &str,
    payload: &OversightDecision,
    verified_remote_key: Option<&str>,
    actor: &str,
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

    if let Some(verified_key) = verified_remote_key {
        if payload.verifying_key.as_deref() != Some(verified_key) {
            return Err(AppError::Forbidden(
                "Verified remote signer does not match the paired device key".to_string(),
            ));
        }
    } else if is_production_env() {
        if let (Some(sig), Some(pk)) = (&payload.signature, &payload.verifying_key) {
            if let Err(e) = verify_oversight_signature_canonical(
                entry_id,
                &payload.decision,
                payload.user_answer.as_deref(),
                payload.override_slot.as_deref(),
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
            if let Err(e) = verify_oversight_signature_canonical(
                entry_id,
                &payload.decision,
                payload.user_answer.as_deref(),
                payload.override_slot.as_deref(),
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

    // 2. Open an atomic database transaction for nonce and oversight log updates.
    // If either fails, the transaction aborts automatically, preserving state consistency.
    let mut tx = tokio::time::timeout(DB_QUERY_TIMEOUT, state.resources.pool.begin())
        .await
        .map_err(|_| {
            AppError::InternalServerError("Database transaction begin timed out".to_string())
        })?
        .map_err(AppError::Sqlx)?;

    if let Some(ref nonce) = payload.nonce {
        if let Some(ts) = payload.timestamp {
            let nonce_insert =
                sqlx::query("INSERT INTO used_nonces (nonce, timestamp) VALUES (?, ?)")
                    .bind(nonce)
                    .bind(ts)
                    .execute(&mut *tx);
            let res = tokio::time::timeout(DB_QUERY_TIMEOUT, nonce_insert)
                .await
                .map_err(|_| {
                    AppError::InternalServerError("Oversight nonce insert timed out".to_string())
                })?;

            if let Err(e) = res {
                let is_unique_violation = if let Some(db_err) = e.as_database_error() {
                    db_err.is_unique_violation()
                        || db_err
                            .code()
                            .map(|c| c == "2067" || c == "1555" || c == "23000")
                            .unwrap_or(false)
                } else {
                    false
                } || e.to_string().contains("UNIQUE constraint failed");

                if is_unique_violation {
                    // UNIQUE violation (replay attack) -> transaction rolls back, entry remains untouched
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
    let decided_by_val = actor;

    let db_query = sqlx::query(
        "UPDATE oversight_log 
         SET status = ?, decision = ?, decided_at = ?, decided_by = ? 
         WHERE id = ? AND status = 'pending'",
    )
    .bind(status_val)
    .bind(decision_val)
    .bind(now)
    .bind(decided_by_val)
    .bind(entry_id);

    let query_result = tokio::time::timeout(DB_QUERY_TIMEOUT, db_query.execute(&mut *tx))
        .await
        .map_err(|_| AppError::InternalServerError("Database update timed out".to_string()))?
        .map_err(AppError::Sqlx)?;

    if query_result.rows_affected() == 0 {
        return Err(AppError::Conflict(format!(
            "Oversight entry '{}' has already been resolved or does not exist",
            entry_id
        )));
    }

    // 4. Claim from in-memory queue BEFORE committing transaction.
    // If the entry was already claimed/resolved by a concurrent request, abort transaction and return Conflict.
    let entry = match state.comms.oversight_queue.remove(entry_id) {
        Some((_, e)) => e,
        None => {
            let _ = tx.rollback().await;
            return Err(AppError::Conflict(
                "Oversight entry was resolved by a concurrent request".to_string(),
            ));
        }
    };

    // 5. Claim the waiter if one is actively blocked
    let maybe_shooter = state.comms.oversight_resolvers.remove(entry_id);

    // Commit transaction atomically only after in-memory claim succeeds
    match tokio::time::timeout(DB_QUERY_TIMEOUT, tx.commit()).await {
        Ok(Ok(_)) => {}
        Ok(Err(sqlx_err)) => {
            state
                .comms
                .oversight_queue
                .insert(entry_id.to_string(), entry);
            if let Some((_, shooter)) = maybe_shooter {
                state
                    .comms
                    .oversight_resolvers
                    .insert(entry_id.to_string(), shooter);
            }
            return Err(AppError::Sqlx(sqlx_err));
        }
        Err(_) => {
            state
                .comms
                .oversight_queue
                .insert(entry_id.to_string(), entry);
            if let Some((_, shooter)) = maybe_shooter {
                state
                    .comms
                    .oversight_resolvers
                    .insert(entry_id.to_string(), shooter);
            }
            return Err(AppError::InternalServerError(
                "Database commit timed out".to_string(),
            ));
        }
    }

    // 6. Resolve the waiting runner oneshot channel
    if let Some((_, shooter)) = maybe_shooter {
        let answer = payload.user_answer.clone().or_else(|| {
            if payload.decision != "approved" && payload.decision != "rejected" {
                Some(payload.decision.clone())
            } else {
                None
            }
        });
        if let Err(unreceived) = shooter.send(crate::agent::types::OversightResolution {
            approved,
            override_slot: payload.override_slot.clone(),
            user_answer: answer,
        }) {
            tracing::warn!(
                "⚠️ [Oversight] Runner dropped oneshot receiver for entry '{}' before decision was delivered (approved={})",
                entry_id,
                unreceived.approved
            );
        }
    }

    // 7. Log decision, update audit trail, and emit WebSocket events
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
    let audit_actor = actor.to_string();
    let params = serde_json::to_string(&json!({
        "entry_id": eid,
        "approved": approved
    }))
    .unwrap_or_default();
    let record_fut = audit.record(
        &audit_actor,
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
#[tracing::instrument(skip(state, payload), fields(action_id = %entry_id, decision = %payload.decision, trace_id = tracing::field::Empty, request_id = tracing::field::Empty, http_status = tracing::field::Empty), name = "governance::resolve_pending")]
pub async fn decide_oversight(
    Path(entry_id): Path<String>,
    State(state): State<Arc<AppState>>,
    _admin: crate::middleware::auth::RequireAdmin,
    Json(payload): Json<OversightDecision>,
) -> Result<impl IntoResponse, AppError> {
    let res = resolve_oversight_decision(&state, &entry_id, &payload).await?;
    Ok((StatusCode::OK, Json(res)))
}

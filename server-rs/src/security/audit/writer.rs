//! @docs SECURITY:Core
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Security & Governance / audit / writer
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Single-writer task owning chain head and sequence authority.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: none declared

use super::hasher::compute_canonical_hash;
use super::types::*;
use anyhow::{anyhow, Result};
use chrono::Utc;
use ed25519_dalek::{Signer, SigningKey};
use futures::FutureExt;
use secrecy::{ExposeSecret, SecretBox};
use sqlx::SqlitePool;
use std::time::Duration;
use tokio::sync::mpsc;
use tracing::{error, info, warn};
use zeroize::Zeroizing;

pub fn load_signing_key_from_env() -> Result<Option<SecretBox<SecretSigningKey>>> {
    let require_signed = match std::env::var("AUDIT_REQUIRE_SIGNED") {
        Ok(v) => v == "1" || v.eq_ignore_ascii_case("true"),
        Err(_) => false,
    };
    let raw = std::env::var("AUDIT_PRIVATE_KEY").ok();
    load_signing_key_inner(raw.as_deref(), require_signed)
}

pub fn load_signing_key_inner(
    raw_key: Option<&str>,
    require_signed: bool,
) -> Result<Option<SecretBox<SecretSigningKey>>> {
    let raw = match raw_key {
        Some(s) if !s.trim().is_empty() => Zeroizing::new(s.trim().to_string()),
        _ => {
            if require_signed {
                return Err(anyhow!(AuditError::BadKey)
                    .context("AUDIT_REQUIRE_SIGNED=1 but AUDIT_PRIVATE_KEY is unset or empty"));
            }
            return Ok(None);
        }
    };

    let bytes_vec = match hex::decode(raw.as_str()) {
        Ok(v) => Zeroizing::new(v),
        Err(e) => {
            if require_signed {
                return Err(anyhow!(AuditError::BadKey)
                    .context(format!("Failed to hex-decode AUDIT_PRIVATE_KEY: {e}")));
            }
            warn!("AUDIT_PRIVATE_KEY hex decode failed ({e}); falling back to unsigned audit mode");
            return Ok(None);
        }
    };

    let arr: [u8; 32] = match bytes_vec.as_slice().try_into() {
        Ok(a) => a,
        Err(_) => {
            if require_signed {
                return Err(anyhow!(AuditError::BadKey)
                    .context("AUDIT_PRIVATE_KEY must be exactly 32 bytes (64 hex characters)"));
            }
            warn!("AUDIT_PRIVATE_KEY is not 32 bytes; falling back to unsigned audit mode");
            return Ok(None);
        }
    };

    let sig_key = SigningKey::from_bytes(&arr);
    Ok(Some(SecretBox::new(Box::new(SecretSigningKey(sig_key)))))
}

pub(crate) fn spawn_writer(
    pool: SqlitePool,
    signing_key: Option<SecretBox<SecretSigningKey>>,
    mut audit_rx: mpsc::Receiver<AuditWriteRequest>,
) -> Result<tokio::task::JoinHandle<()>> {
    let handle = tokio::spawn(async move {
        info!("audit writer started [audit]");

        let _ = migrate_schema(&pool).await;

        let mut local_last_hash: String = GENESIS_PREV_HASH.to_string();
        let mut startup_ok = false;

        for attempt in 1..=3 {
            match tokio::time::timeout(
                WORKER_STARTUP_TIMEOUT,
                sqlx::query_as::<_, (String,)>(
                    "SELECT current_hash FROM audit_trail ORDER BY seq DESC LIMIT 1",
                )
                .fetch_optional(&pool),
            )
            .await
            {
                Ok(Ok(Some((h,)))) => {
                    local_last_hash = h;
                    startup_ok = true;
                    break;
                }
                Ok(Ok(None)) => {
                    local_last_hash = GENESIS_PREV_HASH.to_string();
                    startup_ok = true;
                    break;
                }
                Ok(Err(e)) => {
                    warn!(
                        attempt = attempt,
                        error = %e,
                        "audit writer startup DB read failed; retrying"
                    );
                }
                Err(_) => {
                    warn!(
                        attempt = attempt,
                        "audit writer startup DB read timed out; retrying"
                    );
                }
            }
            tokio::time::sleep(Duration::from_millis(50 * attempt as u64)).await;
        }

        if !startup_ok {
            error!("audit writer startup DB read failed permanently after 3 attempts; aborting");
            return;
        }

        loop {
            let req = match audit_rx.recv().await {
                Some(r) => r,
                None => {
                    info!("audit writer channel closed; shutting down");
                    return;
                }
            };

            let result = std::panic::AssertUnwindSafe(process_one(
                &pool,
                signing_key.as_ref(),
                &local_last_hash,
                req,
            ))
            .catch_unwind()
            .await;

            match result {
                Ok(Ok((_entry, new_head))) => {
                    local_last_hash = new_head;
                }
                Ok(Err(kind)) => {
                    warn!(error_kind = %kind, "audit write failed");
                }
                Err(_panic) => {
                    error!("audit writer panicked processing request; recovering head from DB");
                    if let Ok(Ok(Some((h,)))) = tokio::time::timeout(
                        Duration::from_secs(3),
                        sqlx::query_as::<_, (String,)>(
                            "SELECT current_hash FROM audit_trail ORDER BY seq DESC LIMIT 1",
                        )
                        .fetch_optional(&pool),
                    )
                    .await
                    {
                        local_last_hash = h;
                    }
                }
            }
        }
    });
    Ok(handle)
}

async fn process_one(
    pool: &SqlitePool,
    signing_key: Option<&SecretBox<SecretSigningKey>>,
    prev_hash: &str,
    req: AuditWriteRequest,
) -> Result<(AuditEntry, String), AuditError> {
    let id = uuid::Uuid::new_v4().to_string();
    let timestamp = Utc::now();
    let timestamp_str = timestamp.to_rfc3339();

    let current_hash = compute_canonical_hash(
        prev_hash,
        &req.agent_id,
        req.mission_id.as_deref(),
        req.user_id.as_deref(),
        &req.action,
        &req.params,
        &timestamp_str,
    );

    let signature = signing_key.map(|k| {
        let sig_key = &k.expose_secret().0;
        let sig = sig_key.sign(current_hash.as_bytes());
        hex::encode(sig.to_bytes())
    });

    let mut entry = AuditEntry {
        seq: None,
        id: id.clone(),
        timestamp,
        agent_id: req.agent_id,
        mission_id: req.mission_id,
        user_id: req.user_id,
        action: req.action,
        params: req.params,
        prev_hash: prev_hash.to_string(),
        current_hash: current_hash.clone(),
        signature,
    };

    let mut attempts = 0;
    let max_attempts = 3;
    let res = loop {
        attempts += 1;
        let query_res = sqlx::query(
            "INSERT INTO audit_trail              (id, agent_id, mission_id, user_id, action, params, prev_hash, current_hash, timestamp, signature)              VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)              RETURNING seq",
        )
        .bind(&entry.id)
        .bind(&entry.agent_id)
        .bind(&entry.mission_id)
        .bind(&entry.user_id)
        .bind(&entry.action)
        .bind(&entry.params)
        .bind(&entry.prev_hash)
        .bind(&entry.current_hash)
        .bind(entry.timestamp)
        .bind(&entry.signature)
        .fetch_optional(pool)
        .await;

        match query_res {
            Ok(row_opt) => break Ok(row_opt),
            Err(e) if attempts < max_attempts => {
                warn!(
                    attempt = attempts,
                    error = %e,
                    "audit DB insert transient storage contention, retrying"
                );
                tokio::time::sleep(tokio::time::Duration::from_millis(50 * attempts as u64)).await;
            }
            Err(e) => break Err(e),
        }
    };

    match res {
        Ok(Some(row)) => {
            use sqlx::Row;
            entry.seq = row.try_get::<i64, _>("seq").ok();
            let _ = req.resp_tx.send(Ok(entry.clone()));
            Ok((entry, current_hash))
        }
        Ok(None) => {
            let _ = req.resp_tx.send(Ok(entry.clone()));
            Ok((entry, current_hash))
        }
        Err(e) => {
            error!(error_kind = "db_insert", error = %e, "audit DB insert failed after retries");
            let _ = req.resp_tx.send(Err(AuditError::Storage));
            Err(AuditError::Storage)
        }
    }
}

pub(crate) async fn migrate_schema(pool: &SqlitePool) -> Result<()> {
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS audit_trail (
            seq INTEGER PRIMARY KEY AUTOINCREMENT,
            id TEXT NOT NULL UNIQUE,
            agent_id TEXT NOT NULL,
            action TEXT NOT NULL,
            params TEXT NOT NULL,
            prev_hash TEXT NOT NULL,
            current_hash TEXT NOT NULL,
            timestamp DATETIME NOT NULL,
            signature TEXT,
            mission_id TEXT,
            user_id TEXT,
            trace_id TEXT,
            http_status INTEGER,
            cost_micros INTEGER NOT NULL DEFAULT 0,
            content TEXT,
            created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
        )",
    )
    .execute(pool)
    .await?;
    Ok(())
}

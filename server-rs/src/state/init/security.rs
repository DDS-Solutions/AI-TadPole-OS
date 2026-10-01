//! @docs ARCHITECTURE:State
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / State / Init Security
//! - **Primary Entrypoints**: `load_security_tokens`, `load_oversight_public_key`, `verify_boot_audit_ledger`
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none declared
//! - **Telemetry Targets**: none declared
//! - **Witness Tests**: none declared

use crate::error::AppError;
use std::sync::Arc;

pub struct SecurityTokens {
    pub deploy_token: String,
    pub admin_token: String,
}

pub fn load_security_tokens() -> Result<SecurityTokens, AppError> {
    tracing::info!("🔑 [Auth] Loading Neural Token...");
    let deploy_token = match std::env::var("NEURAL_ENGINE_ACCESS_TOKEN")
        .or_else(|_| std::env::var("NEURAL_TOKEN"))
    {
        Ok(token) => token.trim().to_string(),
        Err(_) if cfg!(test) => "ci-test-token-placeholder".to_string(),
        Err(_) => return Err(AppError::Unauthorized(
            "🚨 FATAL: NEURAL_TOKEN or NEURAL_ENGINE_ACCESS_TOKEN environment variable MUST be set for the engine to start.".to_string()
        )),
    };
    let is_production = crate::utils::security::is_production_env();
    let admin_token = match std::env::var("NEURAL_ADMIN_TOKEN")
        .or_else(|_| std::env::var("ADMIN_TOKEN"))
    {
        Ok(token) => token.trim().to_string(),
        Err(_) if cfg!(test) => "ci-test-admin-token-placeholder".to_string(),
        Err(_) if is_production => {
            return Err(AppError::Unauthorized(
                "🚨 FATAL: NEURAL_ADMIN_TOKEN or ADMIN_TOKEN environment variable MUST be set in production."
                    .to_string(),
            ));
        }
        Err(_) => {
            tracing::warn!(
                "⚠️ NEURAL_ADMIN_TOKEN is not configured. Falling back to NEURAL_TOKEN for local development only."
            );
            deploy_token.clone()
        }
    };
    if admin_token.is_empty() {
        return Err(AppError::Unauthorized(
            "🚨 FATAL: administrative token cannot be empty.".to_string(),
        ));
    }
    if is_production && admin_token == deploy_token {
        return Err(AppError::Unauthorized(
            "🚨 FATAL: NEURAL_ADMIN_TOKEN must differ from NEURAL_TOKEN in production.".to_string(),
        ));
    }

    Ok(SecurityTokens {
        deploy_token,
        admin_token,
    })
}

pub fn load_oversight_public_key() -> Option<String> {
    let key = std::env::var("OVERSIGHT_PUBLIC_KEY").ok();
    if let Some(ref k) = key {
        let fingerprint = if k.len() >= 8 { &k[..8] } else { k };
        tracing::info!(
            "🔑 [Security] Oversight public key pinned (fingerprint: {}...)",
            fingerprint
        );
    } else {
        let is_production = std::env::var("TADPOLE_ENV")
            .or_else(|_| std::env::var("ENV"))
            .map(|v| v.eq_ignore_ascii_case("production"))
            .unwrap_or(false);
        if is_production {
            tracing::error!("🚨 SECURITY: OVERSIGHT_PUBLIC_KEY is not set in production! Oversight decisions will be rejected until a pinned key is configured.");
        } else {
            tracing::warn!("⚠️ OVERSIGHT_PUBLIC_KEY is not configured. Oversight signatures are verified but NOT pinned to an authorized operator.");
        }
    }
    key
}

pub fn verify_boot_audit_ledger(audit_trail: Arc<crate::security::audit::MerkleAuditTrail>) {
    tokio::spawn(async move {
        match audit_trail.verify_last_n(100, None).await {
            Ok((verified, total)) => {
                if total > 0 && verified == total {
                    tracing::info!(
                        "🛡️ [Audit Ledger] Boot integrity verified ({} of {} records valid).",
                        verified,
                        total
                    );
                } else if total > 0 {
                    tracing::error!(
                        "🚨 [Audit Ledger] TAMPER DETECTION ALERT: Boot integrity check failed! ({} of {} valid)",
                        verified,
                        total
                    );
                } else {
                    tracing::info!("🛡️ [Audit Ledger] Boot integrity verified (ledger is empty).");
                }
            }
            Err(e) => {
                tracing::warn!(
                    "⚠️ [Audit Ledger] Boot integrity verification encountered error: {}",
                    e
                );
            }
        }
    });
}

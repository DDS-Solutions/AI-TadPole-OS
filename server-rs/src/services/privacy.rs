//! @docs ARCHITECTURE:Core:Privacy
//!
//! ### AI Context Alignment
//! - **Subsystem**: Frontend Service Layer / privacy
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Asynchronous service calls normalize response envelopes and propagate typed errors.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: `[PrivacyGuard]`
//! - **Witness Tests**: none declared

use crate::state::AppState;
use chrono::Utc;
use serde_json::json;
use std::sync::Arc;

pub async fn start_privacy_guard(app_state: Arc<AppState>) {
    tracing::info!("🛡️ [PrivacyGuard] Air-Gap Monitor Active.");

    let client = match reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(3))
        .build()
    {
        Ok(c) => c,
        Err(e) => {
            tracing::error!(
                "🚨 [PrivacyGuard] Failed to build HTTP client: {}. Air-Gap Monitor disabled.",
                e
            );
            return;
        }
    };

    let canaries = [
        "https://www.google.com",
        "https://10.0.0.1",
        "https://10.0.0.1",
    ];

    let mut is_breached = false;
    let mut interval = tokio::time::interval(std::time::Duration::from_secs(30));
    interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

    loop {
        interval.tick().await;

        let privacy_mode = app_state
            .governance
            .privacy_mode
            .load(std::sync::atomic::Ordering::Relaxed);

        if privacy_mode {
            // Concurrent canary check to avoid serial timeout penalties
            let mut canary_futures = Vec::new();
            for &canary in &canaries {
                let cl = client.clone();
                canary_futures.push(async move {
                    match cl.head(canary).send().await {
                        Ok(resp) if resp.status().is_success() => true,
                        _ => false,
                    }
                });
            }

            let results = futures::future::join_all(canary_futures).await;
            let current_breach = results.into_iter().any(|reachable| reachable);

            if current_breach && !is_breached {
                // Edge trigger: Transition to Breached
                is_breached = true;
                tracing::warn!("🚨 [PrivacyGuard] BREACH: External network reachable while Privacy Mode is ON!");
                app_state.emit_event(json!({
                    "type": "engine:privacy_breach",
                    "severity": "CRITICAL",
                    "message": "Shield Compromised: External internet access detected during Air-Gap mode.",
                    "timestamp": Utc::now().to_rfc3339()
                }));
            } else if !current_breach && is_breached {
                // Edge trigger: Transition to Recovered
                is_breached = false;
                tracing::info!("🛡️ [PrivacyGuard] Air-Gap isolation restored.");
                app_state.emit_event(json!({
                    "type": "engine:privacy_restored",
                    "severity": "INFO",
                    "message": "Shield Intact: Air-Gap isolation re-verified.",
                    "timestamp": Utc::now().to_rfc3339()
                }));
            } else if !current_breach {
                tracing::debug!("[PrivacyGuard] Air-Gap verified.");
            }
        } else {
            is_breached = false;
        }
    }
}

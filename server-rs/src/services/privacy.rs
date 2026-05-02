//! @docs ARCHITECTURE:State
//!
//! ### AI Assist Note
//! **Privacy Shield Service**: Orchestrates the local-only traffic
//! enforcement and data sovereignty for the Tadpole OS engine. Features
//! the **Air-Gap Monitor**: a background loop that periodically
//! attempts to reach external endpoints (e.g., Google DNS) when
//! `privacy_mode` is enabled. Implements **Breach Detection**: if
//! external connectivity is detected during air-gap mode, a critical
//! `engine:privacy_breach` event is emitted. AI agents should respect
//! the `privacy_mode` flag and avoid any tool invocations that require
//! non-local network access when the shield is active (PRIV-01).
//!
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: False positive breach detection due to local
//!   proxy misconfiguration, or failure to detect connectivity if the
//!   canary endpoint (google.com) is blocked/down.
//! - **Trace Scope**: `server-rs::services::privacy`

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
            tracing::error!("🚨 [PrivacyGuard] Failed to build HTTP client: {}. Air-Gap Monitor disabled.", e);
            return;
        }
    };

    loop {
        tokio::time::sleep(std::time::Duration::from_secs(30)).await;

        let privacy_mode = app_state
            .governance
            .privacy_mode
            .load(std::sync::atomic::Ordering::Relaxed);
        if privacy_mode {
            // Attempt to reach known external endpoints (Canaries)
            // If ANY are reachable, it's a breach. We use multiple to avoid false negatives.
            let canaries = [
                "https://www.google.com",
                "https://10.0.0.1",
                "https://10.0.0.1",
            ];
            let mut breach = false;

            for canary in canaries {
                match client.head(canary).send().await {
                    Ok(resp) if resp.status().is_success() => {
                        breach = true;
                        break;
                    }
                    _ => continue,
                }
            }

            if breach {
                tracing::warn!("🚨 [PrivacyGuard] BREACH: External network reachable while Privacy Mode is ON!");
                app_state.emit_event(json!({
                    "type": "engine:privacy_breach",
                    "severity": "CRITICAL",
                    "message": "Shield Compromised: External internet access detected during Air-Gap mode.",
                    "timestamp": Utc::now().to_rfc3339()
                }));
            } else {
                tracing::debug!("[PrivacyGuard] Air-Gap verified.");
            }
        }
    }
}

// Metadata: [privacy]

// Metadata: [privacy]

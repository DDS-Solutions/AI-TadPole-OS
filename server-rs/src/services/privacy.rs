use crate::state::AppState;
use std::sync::Arc;
use chrono::Utc;
use serde_json::json;

pub async fn start_privacy_guard(app_state: Arc<AppState>) {
    tracing::info!("🛡️ [PrivacyGuard] Air-Gap Monitor Active.");
    
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(3))
        .build()
        .expect("Failed to build PrivacyGuard HTTP client");

    loop {
        tokio::time::sleep(std::time::Duration::from_secs(30)).await;
        
        let privacy_mode = app_state.governance.privacy_mode.load(std::sync::atomic::Ordering::Relaxed);
        if privacy_mode {
            // Attempt to reach a known external address (Google DNS or similar)
            // We use a simple HEAD request to minimize traffic
            match client.head("https://www.google.com").send().await {
                Ok(resp) if resp.status().is_success() => {
                    tracing::warn!("🚨 [PrivacyGuard] BREACH: External network reachable while Privacy Mode is ON!");
                    app_state.emit_event(json!({
                        "type": "engine:privacy_breach",
                        "severity": "CRITICAL",
                        "message": "Shield Compromised: External internet access detected during Air-Gap mode.",
                        "timestamp": Utc::now().to_rfc3339()
                    }));
                }
                _ => {
                    // Safe or timeout/error (which is good in air-gap)
                    tracing::debug!("[PrivacyGuard] Air-Gap verified.");
                }
            }
        }
    }
}

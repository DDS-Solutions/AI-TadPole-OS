//! @docs ARCHITECTURE:Networking
//! 
//! ### AI Assist Note
//! **Discord Adapter**: Enables the AI swarm to broadcast outbound 
//! notifications and system alerts via **Secure Webhook Integration**. 
//! Features **High-Fidelity Formatting** for mission-critical updates. 
//! **Discord Rate Limits**: Webhooks are subject to global and channel-level 
//! limits; avoid rapid-fire notifications in tight loops to prevent 
//! `429 Too Many Requests` (LMT-04).
//! 
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: Invalid Webhook URL (404), Rate Limit Exceeded 
//!   (429), or JSON payload rejection due to over-sized content buffers 
//!   (DISC-01).
//! - **Trace Scope**: `server-rs::adapter::discord`

use anyhow::Result;
use reqwest::Client;
use serde::Serialize;

/// Adapter for sending high-fidelity notifications to Discord.
pub struct DiscordAdapter {
    /// The target Discord Webhook URL.
    pub webhook_url: String,
    /// Shared HTTP client for asynchronous transmission.
    client: Client,
}

#[derive(Debug, Serialize)]
struct DiscordMessage {
    content: String,
    username: String,
    avatar_url: Option<String>,
}

impl DiscordAdapter {
    pub fn new(webhook_url: String) -> Self {
        Self {
            webhook_url,
            client: Client::builder()
                .user_agent("TadpoleOS/1.1.4")
                .build()
                .unwrap_or_else(|_| Client::new()),
        }
    }

    /// Sends a notification to a Discord channel via webhook.
    pub async fn notify(&self, agent_name: &str, text: &str) -> Result<()> {
        let msg = DiscordMessage {
            content: text.to_string(),
            username: format!("Tadpole OS: {}", agent_name),
            avatar_url: None, // Could use specific icons for roles
        };

        let res = self
            .client
            .post(&self.webhook_url)
            .json(&msg)
            .send()
            .await?;

        if !res.status().is_success() {
            let err = res.text().await?;
            return Err(anyhow::anyhow!("Discord Webhook Error: {}", err));
        }

        Ok(())
    }
}

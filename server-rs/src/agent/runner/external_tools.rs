//! @docs ARCHITECTURE:Runner
//! 
//! ### AI Assist Note
//! **External Tools**: Interfaces with public APIs and non-workspace services. 
//! Includes **Discord Notifications**, **URL Fetching**, and **Financial Auditing**. 
//! Implements **Privacy Guard** (SEC-04) and **Resource Quotas** for external 
//! retrieval. Requires **Oversight** for all outbound traffic to prevent 
//! data exfiltration.
//! 
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: Webhook missing/invalid, external URL timeout, oversight 
//!   rejection, or database error during financial log retrieval.
//! - **Trace Scope**: `server-rs::agent::runner::external_tools`

use super::{AgentRunner, RunContext};

impl AgentRunner {
    /// Handles `notify_discord`: sends a webhook notification after oversight.
    pub(crate) async fn handle_notify_discord(
        &self,
        ctx: &RunContext,
        fc: &crate::agent::types::GeminiFunctionCall,
        output_text: &mut String,
    ) -> anyhow::Result<()> {
        let msg = fc
            .args
            .get("message")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        tracing::info!(
            "🔔 [Surface] Agent {} requesting Discord notification...",
            ctx.agent_id
        );
        self.broadcast_agent(
            ctx,
            "🔔 Oversight: wants to notify Discord.",
            "warning",
        );

        let approved = self
            .submit_oversight(
                crate::agent::types::ToolCall {
                    id: uuid::Uuid::new_v4().to_string(),
                    agent_id: ctx.agent_id.clone(),
                    mission_id: Some(ctx.mission_id.clone()),
                    skill: "notify_discord".to_string(),
                    params: fc.args.clone(),
                    department: ctx.department.clone(),
                    description: "Sending an external notification via Discord.".to_string(),
                    timestamp: chrono::Utc::now().to_rfc3339(),
                },
                Some(ctx.mission_id.clone()),
            )
            .await;

        if approved {
            if let Ok(webhook) = std::env::var("DISCORD_WEBHOOK") {
                let adapter = crate::adapter::discord::DiscordAdapter::new(webhook);
                adapter.notify(&ctx.name, msg).await?;
                self.broadcast_agent(
                    ctx,
                    "🔔 Surface: sent Discord alert",
                    "success",
                );
                *output_text = format!("(Notified Discord) {}", output_text);
            } else {
                *output_text =
                    format!("(Discord notification failed - no webhook) {}", output_text);
            }
        } else {
            *output_text = format!(
                "(Discord notification REJECTED by Oversight) {}",
                output_text
            );
        }

        Ok(())
    }

    /// Handles `fetch_url`: retrieves text content from a public URL.
    pub(crate) async fn handle_fetch_url(
        &self,
        ctx: &RunContext,
        fc: &crate::agent::types::GeminiFunctionCall,
        output_text: &mut String,
        _usage: &mut Option<crate::agent::types::TokenUsage>,
    ) -> anyhow::Result<()> {
        let url = fc.args.get("url").and_then(|v| v.as_str()).unwrap_or("");
        tracing::info!("🌐 [Surface] Agent {} fetching URL: {}", ctx.agent_id, url);

        self.broadcast_agent(
            ctx,
            "🔒 Oversight: wants to fetch external URL. Review required.",
            "warning",
        );

        let approved = self
            .submit_oversight(
                crate::agent::types::ToolCall {
                    id: uuid::Uuid::new_v4().to_string(),
                    agent_id: ctx.agent_id.clone(),
                    mission_id: Some(ctx.mission_id.clone()),
                    skill: "fetch_url".to_string(),
                    params: fc.args.clone(),
                    department: ctx.department.clone(),
                    description: format!("External retrieval from: {}", url),
                    timestamp: chrono::Utc::now().to_rfc3339(),
                },
                Some(ctx.mission_id.clone()),
            )
            .await;

        if !approved {
            *output_text = format!("(Fetch REJECTED by Oversight) {}", output_text);
            return Ok(());
        }

        self.broadcast_agent(
            ctx,
            &format!("🌐 Surface: researching {}...", url),
            "info",
        );

        match self.state.resources.http_client.get(url).send().await {
            Ok(r) => {
                let text = r
                    .text()
                    .await
                    .unwrap_or_else(|_| "Error reading text".to_string());
                let truncated = self.safe_truncate(&text, 8000);
                *output_text = format!("(FETCHED CONTENT FROM {}):\n\n{}", url, truncated);
            }
            Err(e) => {
                *output_text = format!("(FETCH FAILED: {}) {}", e, output_text);
            }
        }

        Ok(())
    }

    /// Handles `query_financial_logs`: retrieves and analyzes mission cost history.
    pub(crate) async fn handle_query_financial_logs(
        &self,
        ctx: &RunContext,
        fc: &crate::agent::types::GeminiFunctionCall,
        output_text: &mut String,
        _usage: &mut Option<crate::agent::types::TokenUsage>,
    ) -> anyhow::Result<()> {
        let limit = fc.args.get("limit").and_then(|v| v.as_i64()).unwrap_or(10);

        tracing::info!(
            "📊 [Governance] Agent {} querying financial history (limit: {})...",
            ctx.agent_id,
            limit
        );
        self.broadcast_agent(
            ctx,
            "📊 Audit: reviewing fiscal logs...",
            "info",
        );

        let history =
            crate::agent::mission::get_recent_missions(&self.state.resources.pool, limit).await?;
        let history_json = serde_json::to_string_pretty(&history).unwrap_or_default();

        *output_text = format!("MISSION HISTORY RETRIEVED:\n\n{}\n\nPlease analyze this history for cost anomalies, burn rates, or optimization opportunities.", history_json);

        Ok(())
    }
}

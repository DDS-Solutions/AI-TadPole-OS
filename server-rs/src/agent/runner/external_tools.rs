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
use crate::error::AppError;
use crate::agent::runner::tools::error::ToolExecutionError;

impl AgentRunner {
    /// Handles `notify_discord`: sends a webhook notification after oversight.
    pub(crate) async fn handle_notify_discord(
        &self,
        ctx: &RunContext,
        fc: &crate::agent::types::ToolCall,
        ) -> Result<String, ToolExecutionError> {
        let msg = fc
            .args
            .get("message")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        tracing::info!(
            "🔔 [Surface] Agent {} requesting Discord notification...",
            ctx.agent_id
        );
        self.broadcast_agent(ctx, "🔔 Oversight: wants to notify Discord.", "warning");

        let approved = self
            .submit_oversight(
                crate::agent::types::ToolCallAudit {
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
            .await?;

        if approved {
            if let Ok(webhook) = std::env::var("DISCORD_WEBHOOK") {
                let adapter = crate::adapter::discord::DiscordAdapter::new(webhook);
                adapter.notify(&ctx.name, msg).await.map_err(AppError::from)?;
                self.broadcast_agent(ctx, "🔔 Surface: sent Discord alert", "success");
                Ok("(Notified Discord)".to_string())
            } else {
                Ok("(Discord notification failed - no webhook)".to_string())
            }
        } else {
            Ok("(Discord notification REJECTED by Oversight)".to_string())
        }
    }

    /// Handles `fetch_url`: retrieves text content from a public URL.
    pub(crate) async fn handle_fetch_url(
        &self,
        ctx: &RunContext,
        fc: &crate::agent::types::ToolCall,
        _usage: &mut Option<crate::agent::types::TokenUsage>,
    ) -> Result<String, ToolExecutionError> {
        let url = fc.args.get("url").and_then(|v| v.as_str()).unwrap_or("");
        tracing::info!("🌐 [Surface] Agent {} fetching URL: {}", ctx.agent_id, url);

        self.broadcast_agent(
            ctx,
            "🔒 Oversight: wants to fetch external URL. Review required.",
            "warning",
        );

        let approved = self
            .submit_oversight(
                crate::agent::types::ToolCallAudit {
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
            .await?;

        if !approved {
            Ok("(Fetch REJECTED by Oversight)".to_string())
        } else {
            self.broadcast_agent(ctx, &format!("🌐 Surface: researching {}...", url), "info");

            match self.state.resources.http_client.get(url).send().await {
                Ok(r) => {
                    let text = r
                        .text()
                        .await
                        .unwrap_or_else(|_| "Error reading text".to_string());
                    let truncated = self.safe_truncate(&text, 8000);
                    Ok(format!("(FETCHED CONTENT FROM {}):\n\n{}", url, truncated))
                }
                Err(e) => {
                    Ok(format!("(FETCH FAILED: {})", e))
                }
            }
        }
    }

    /// Handles `query_financial_logs`: retrieves and analyzes mission cost history.
    pub(crate) async fn handle_query_financial_logs(
        &self,
        ctx: &RunContext,
        fc: &crate::agent::types::ToolCall,
        _usage: &mut Option<crate::agent::types::TokenUsage>,
    ) -> Result<String, ToolExecutionError> {
        let limit = fc.args.get("limit").and_then(|v| v.as_i64()).unwrap_or(10);

        tracing::info!(
            "📊 [Governance] Agent {} querying financial history (limit: {})...",
            ctx.agent_id,
            limit
        );
        self.broadcast_agent(ctx, "📊 Audit: reviewing fiscal logs...", "info");

        let history = crate::agent::mission::get_recent_missions(&self.state.resources.pool, limit).await?;
        let history_json = serde_json::to_string_pretty(&history).unwrap_or_default();

        Ok(format!("MISSION HISTORY RETRIEVED:\n\n{}\n\nPlease analyze this history for cost anomalies, burn rates, or optimization opportunities.", history_json))
    }

    /// Handles `search_web`: performs a web search and returns snippets.
    pub(crate) async fn handle_search_web(
        &self,
        ctx: &RunContext,
        fc: &crate::agent::types::ToolCall,
        _usage: &mut Option<crate::agent::types::TokenUsage>,
    ) -> Result<String, ToolExecutionError> {
        let query = fc.args.get("query").and_then(|v| v.as_str()).unwrap_or("");
        tracing::info!("🔍 [Surface] Agent {} searching web for: {}", ctx.agent_id, query);

        self.broadcast_agent(
            ctx,
            &format!("🔍 Oversight: wants to search the web for: {}. Review required.", query),
            "warning",
        );

        let approved = self
            .submit_oversight(
                crate::agent::types::ToolCallAudit {
                    id: uuid::Uuid::new_v4().to_string(),
                    agent_id: ctx.agent_id.clone(),
                    mission_id: Some(ctx.mission_id.clone()),
                    skill: "search_web".to_string(),
                    params: fc.args.clone(),
                    department: ctx.department.clone(),
                    description: format!("Web search query: {}", query),
                    timestamp: chrono::Utc::now().to_rfc3339(),
                },
                Some(ctx.mission_id.clone()),
            )
            .await?;

        if !approved {
            return Ok("(Search REJECTED by Oversight)".to_string());
        }

        self.broadcast_agent(ctx, &format!("🔍 Surface: searching web for '{}'...", query), "info");

        // Simple DuckDuckGo HTML fallback for immediate value
        let search_url = format!("https://html.duckduckgo.com/html/?q={}", urlencoding::encode(query));
        
        match self.state.resources.http_client.get(&search_url).send().await {
            Ok(r) => {
                let html = r.text().await.unwrap_or_default();
                // Simple regex to extract search result snippets from DDG HTML
                let re = regex::Regex::new(r#"class="result__snippet"[^>]*>([^<]+)</span>"#).unwrap();
                let mut snippets = Vec::new();
                for cap in re.captures_iter(&html) {
                    snippets.push(cap[1].to_string());
                    if snippets.len() >= 5 { break; }
                }

                if snippets.is_empty() {
                    Ok(format!("(SEARCH RESULTS FOR '{}'): No direct snippets found. Use 'fetch_url' to visit the search page directly: {}", query, search_url))
                } else {
                    let combined = snippets.join("\n\n---\n\n");
                    Ok(format!("(SEARCH RESULTS FOR '{}'):\n\n{}\n\nUse 'fetch_url' to visit specific sites for more detail.", query, combined))
                }
            }
            Err(e) => {
                Ok(format!("(SEARCH FAILED: {})", e))
            }
        }
    }
}

// Metadata: [external_tools]

// Metadata: [external_tools]

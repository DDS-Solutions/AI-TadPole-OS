//! @docs ARCHITECTURE:Runner
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Agent Runner / external_tools
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Type-safe state handling and bounded execution without unhandled panics.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: `[Surface]`
//! - **Witness Tests**: none declared

use super::{AgentRunner, RunContext};
use crate::agent::runner::tools::error::ToolExecutionError;
use crate::error::AppError;
use once_cell::sync::Lazy;
use regex::Regex;

static DDG_SNIPPET_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r#"class="result__snippet"[^>]*>([^<]+)</span>"#).unwrap());

fn decode_html_entities(s: &str) -> String {
    s.replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&nbsp;", " ")
}

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
                adapter
                    .notify(&ctx.name, msg)
                    .await
                    .map_err(AppError::from)?;
                self.broadcast_agent(ctx, "🔔 Surface: sent Discord alert", "success");
                Ok("(Notified Discord)".to_string())
            } else {
                Err(ToolExecutionError::ExecutionFailed(
                    "Discord notification aborted: DISCORD_WEBHOOK is not configured".to_string(),
                ))
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
        let validated_target = match crate::utils::security::validate_public_http_url(url).await {
            Ok(target) => target,
            Err(e) => return Ok(format!("(FETCH BLOCKED: {})", e)),
        };

        let host = &validated_target.host;
        let port = validated_target.port;
        let validated_ip = validated_target.ip;

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

            // Build client with pinned IP and disable redirects to prevent SSRF bypasses via 30x headers
            let client = match reqwest::Client::builder()
                .resolve_to_addrs(host, &[std::net::SocketAddr::new(validated_ip, port)])
                .redirect(reqwest::redirect::Policy::none())
                .timeout(std::time::Duration::from_secs(30))
                .build()
            {
                Ok(c) => c,
                Err(e) => return Ok(format!("(CLIENT BUILD ERROR: {})", e)),
            };

            match client.get(url).send().await {
                Ok(r) => {
                    let bytes = r.bytes().await.map_err(|e| {
                        ToolExecutionError::ExecutionFailed(format!(
                            "Error reading response: {}",
                            e
                        ))
                    })?;

                    // Bound response byte length to 64 KB before string decoding to prevent memory spikes
                    let bounded_bytes = if bytes.len() > 65_536 {
                        &bytes[..65_536]
                    } else {
                        &bytes[..]
                    };

                    let text = String::from_utf8_lossy(bounded_bytes);
                    let truncated = self.safe_truncate(&text, 8000);
                    Ok(format!("(FETCHED CONTENT FROM {}):\n\n{}", url, truncated))
                }
                Err(e) => Ok(format!("(FETCH FAILED: {})", e)),
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
        let raw_limit = fc.args.get("limit").and_then(|v| v.as_i64()).unwrap_or(10);
        let limit = raw_limit.clamp(1, 50);

        tracing::info!(
            "📊 [Governance] Agent {} querying financial history (limit: {})...",
            ctx.agent_id,
            limit
        );
        self.broadcast_agent(ctx, "📊 Audit: reviewing fiscal logs...", "info");

        let approved = self
            .submit_oversight(
                crate::agent::types::ToolCallAudit {
                    id: uuid::Uuid::new_v4().to_string(),
                    agent_id: ctx.agent_id.clone(),
                    mission_id: Some(ctx.mission_id.clone()),
                    skill: "query_financial_logs".to_string(),
                    params: fc.args.clone(),
                    department: ctx.department.clone(),
                    description: format!("Querying financial history logs (limit: {})", limit),
                    timestamp: chrono::Utc::now().to_rfc3339(),
                },
                Some(ctx.mission_id.clone()),
            )
            .await?;

        if !approved {
            return Ok("(Financial log query REJECTED by Oversight)".to_string());
        }

        let history =
            crate::agent::mission::get_recent_missions(&self.state.resources.pool, limit).await?;
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
        tracing::info!(
            "🔍 [Surface] Agent {} searching web for: {}",
            ctx.agent_id,
            query
        );

        self.broadcast_agent(
            ctx,
            &format!(
                "🔍 Oversight: wants to search the web for: {}. Review required.",
                query
            ),
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

        self.broadcast_agent(
            ctx,
            &format!("🔍 Surface: searching web for '{}'...", query),
            "info",
        );

        // Simple DuckDuckGo HTML fallback for immediate value
        let search_url = format!(
            "https://html.duckduckgo.com/html/?q={}",
            urlencoding::encode(query)
        );

        match self
            .state
            .resources
            .http_client
            .get(&search_url)
            .send()
            .await
        {
            Ok(r) => {
                let html = r.text().await.unwrap_or_default();
                let mut snippets = Vec::new();
                for cap in DDG_SNIPPET_REGEX.captures_iter(&html) {
                    let raw_snippet = cap[1].to_string();
                    let decoded = decode_html_entities(&raw_snippet);
                    snippets.push(decoded);
                    if snippets.len() >= 5 {
                        break;
                    }
                }

                if snippets.is_empty() {
                    Ok(format!("(SEARCH RESULTS FOR '{}'): No direct snippets found. Use 'fetch_url' to visit the search page directly: {}", query, search_url))
                } else {
                    let combined = snippets.join("\n\n---\n\n");
                    Ok(format!("(SEARCH RESULTS FOR '{}'):\n\n{}\n\nUse 'fetch_url' to visit specific sites for more detail.", query, combined))
                }
            }
            Err(e) => Ok(format!("(SEARCH FAILED: {})", e)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_html_entities() {
        let input = "Rust &amp; Axum &lt;Web&gt; &quot;Framework&quot; &#39;Fast&#39;&nbsp;!";
        let decoded = decode_html_entities(input);
        assert_eq!(decoded, "Rust & Axum <Web> \"Framework\" 'Fast' !");
    }
}

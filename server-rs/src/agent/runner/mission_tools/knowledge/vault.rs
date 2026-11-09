//! @docs ARCHITECTURE:Runner
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Agent Runner / mission_tools / knowledge / vault
//!
//! ### AI Assist Note
//! Global vault semantic persistence across cross-mission swarm history.
//!
//! ### 🔍 Debugging & Observability
//! - **Telemetry Targets**: `[Global Vault]`

use super::super::{require_str, require_str_opt};
use crate::agent::runner::tools::error::ToolExecutionError;
use crate::agent::runner::{AgentRunner, RunContext};

#[cfg(feature = "vector-memory")]
const RAG_TOP_K: usize = 5;

fn format_archive_entry(topic: &str, summary: &str, content: &str) -> (String, String) {
    let index_text = if topic.is_empty() || topic == "General" {
        summary.to_string()
    } else {
        format!("[{}] {}", topic, summary)
    };
    let stored_text = format!(
        "### [{}] Archival Nugget\n**Summary**: {}\n\n**Content**:\n{}",
        topic, summary, content
    );
    (index_text, stored_text)
}

impl AgentRunner {
    #[cfg(feature = "vector-memory")]
    async fn persist_to_global_vault(
        &self,
        ctx: &RunContext,
        topic: &str,
        summary: &str,
        content: &str,
    ) -> Result<String, ToolExecutionError> {
        let (index_text, stored_text) = format_archive_entry(topic, summary, content);
        let Some(vec) = self.gemini_embed_or_log(ctx, &index_text, "archive").await else {
            return Ok(
                "(GLOBAL ARCHIVE FAILED): Ensure vector-memory is enabled and API keys are valid."
                    .to_string(),
            );
        };
        let Some(vault) = self.connect_global_vault().await else {
            return Ok(
                "(GLOBAL ARCHIVE FAILED): Ensure vector-memory is enabled and API keys are valid."
                    .to_string(),
            );
        };

        let id = format!("global-{}", uuid::Uuid::new_v4());
        match vault
            .add_memory(&id, &stored_text, &ctx.mission_state.mission_id, vec)
            .await
        {
            Ok(_) => {
                self.broadcast_agent(
                    ctx,
                    &format!("🏛️ Global Vault: nugget archived on {}", topic),
                    "success",
                );
                Ok(format!("(GLOBAL ARCHIVE SUCCESS): Nugget on '{}' added to the swarm intelligence vault.", topic))
            }
            Err(e) => {
                tracing::warn!(
                    "⚠️ [Global Vault] Failed to add memory entry for mission {}: {}",
                    ctx.mission_state.mission_id,
                    e
                );
                Ok(format!("(GLOBAL ARCHIVE FAILED: {})", e))
            }
        }
    }

    /// Handles `archive_to_global_vault`: persists a mission nugget to the global swarm vault.
    pub(crate) async fn handle_archive_to_global_vault(
        &self,
        ctx: &RunContext,
        fc: &crate::agent::types::ToolCall,
    ) -> Result<String, ToolExecutionError> {
        let content = require_str(ctx, &fc.args, "content", "archive_to_global_vault")?;
        let summary = require_str(ctx, &fc.args, "summary", "archive_to_global_vault")?;
        let topic = require_str_opt(ctx, &fc.args, "topic", "archive_to_global_vault")?
            .unwrap_or_else(|| "General".to_string());

        tracing::info!(
            "🏛️ [Global Vault] Agent {} archiving nugget on {}.",
            ctx.identity.agent_id,
            topic
        );

        #[cfg(feature = "vector-memory")]
        {
            self.persist_to_global_vault(ctx, &topic, &summary, &content)
                .await
        }

        #[cfg(not(feature = "vector-memory"))]
        {
            let _ = (content, summary, topic);
            Ok(
                "(GLOBAL ARCHIVE FAILED): Ensure vector-memory is enabled and API keys are valid."
                    .to_string(),
            )
        }
    }

    #[cfg(feature = "vector-memory")]
    async fn execute_vault_search(
        &self,
        ctx: &RunContext,
        query: &str,
    ) -> Result<String, ToolExecutionError> {
        let Some(vec) = self.gemini_embed_or_log(ctx, query, "search").await else {
            return Ok(
                "(GLOBAL SEARCH FAILED): Ensure vector-memory is enabled and API keys are valid."
                    .to_string(),
            );
        };
        let Some(vault) = self.connect_global_vault().await else {
            return Ok(
                "(GLOBAL SEARCH FAILED): Ensure vector-memory is enabled and API keys are valid."
                    .to_string(),
            );
        };

        let results = match vault.search_knowledge(vec, RAG_TOP_K).await {
            Ok(r) => r,
            Err(e) => {
                tracing::warn!(
                    "⚠️ [Global Vault] Failed to search global vault records: {}",
                    e
                );
                return Ok(format!("(GLOBAL SEARCH FAILED: {})", e));
            }
        };

        if results.is_empty() {
            Ok(format!(
                "(GLOBAL SEARCH): No relevant intelligence found for '{}'.",
                query
            ))
        } else {
            Ok(format!(
                "(GLOBAL INTELLIGENCE RETRIEVED for '{}'):\n\n{}",
                query,
                results.join("\n\n----- \n\n")
            ))
        }
    }

    /// Handles `search_global_vault`: performs a semantic search across all mission histories.
    pub(crate) async fn handle_search_global_vault(
        &self,
        ctx: &RunContext,
        fc: &crate::agent::types::ToolCall,
    ) -> Result<String, ToolExecutionError> {
        let query = require_str(ctx, &fc.args, "query", "search_global_vault")?;
        tracing::info!(
            "🏛️ [Global Vault] Agent {} searching global vault for: {}",
            ctx.identity.agent_id,
            query
        );

        #[cfg(feature = "vector-memory")]
        {
            self.execute_vault_search(ctx, &query).await
        }

        #[cfg(not(feature = "vector-memory"))]
        {
            let _ = query;
            Ok(
                "(GLOBAL SEARCH FAILED): Ensure vector-memory is enabled and API keys are valid."
                    .to_string(),
            )
        }
    }
}

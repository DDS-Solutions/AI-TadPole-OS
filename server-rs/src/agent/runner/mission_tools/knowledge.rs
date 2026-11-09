//! @docs ARCHITECTURE:Runner
//!
//! ### AI Assist Note
//! - Grouped execution context owns identity, mission state and environment; retain scope across callers.
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Agent Runner / knowledge
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Type-safe state handling and bounded execution without unhandled panics.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: none declared

pub mod iks;
pub mod vault;

use super::require_str;
use crate::agent::runner::tools::error::ToolExecutionError;
use crate::agent::runner::{AgentRunner, RunContext};

#[cfg(feature = "vector-memory")]
const RAG_TOP_K: usize = 5;

fn get_search_miss_hint(query: &str) -> &'static str {
    let lower = query.to_lowercase();
    let is_financial = lower.contains("budget")
        || lower.contains("cost")
        || lower.contains("limit")
        || lower.contains("usd");
    if is_financial {
        "HINT: This query appears to relate to live financial metrics. Vector RAG only contains static shared findings. Use 'get_agent_metrics' to see your own current budget/costs, or 'query_financial_logs' to review overall mission history."
    } else {
        "This query might be reference a physical file or keyword in the workspace. Since you have technical tools, you should now use 'list_files' or 'grep_search' to locate the target and then 'read_file' or 'read_codebase_file' to inspect it directly."
    }
}

fn get_provider_google_key(config: &crate::agent::types::ModelConfig) -> Option<String> {
    let provider = config.provider.to_string().to_lowercase();
    if provider == "google" || provider.contains("gemini") {
        config.api_key.clone().filter(|k| !k.trim().is_empty())
    } else {
        None
    }
}

impl AgentRunner {
    #[cfg(feature = "vector-memory")]
    async fn query_mission_memory(
        &self,
        ctx: &RunContext,
        vec: Vec<f32>,
    ) -> Result<String, String> {
        let Some(mission_mem) = self.connect_mission_memory(ctx).await else {
            return Ok(String::new());
        };
        let results = mission_mem.search_knowledge(vec, RAG_TOP_K).await
            .map_err(|e| format!("(MISSION KNOWLEDGE SEARCH ERROR: Memory store query failed: {}. Use workspace inspection tools.)", e))?;

        let mut text = String::new();
        for (i, item) in results.into_iter().enumerate() {
            text.push_str(&format!("[Result {}]: {}\n", i + 1, item));
        }
        Ok(text)
    }

    #[cfg(feature = "vector-memory")]
    async fn execute_mission_knowledge_search(
        &self,
        ctx: &RunContext,
        query: &str,
    ) -> Result<String, ToolExecutionError> {
        let Some(api_key) = self.resolve_google_api_key(ctx) else {
            return Ok("(MISSION KNOWLEDGE SEARCH UNAVAILABLE: Gemini API key not configured for vector embeddings. Set GEMINI_API_KEY or GOOGLE_API_KEY to enable semantic memory search. Use 'list_files' or 'grep_search' to inspect workspace files directly.)".to_string());
        };

        let http_client = self.state.resources.http_client.clone();
        let query_owned = query.to_string();
        let vec = match crate::agent::memory::get_gemini_embedding(
            &http_client,
            &api_key,
            &query_owned,
        )
        .await
        {
            Ok(v) => v,
            Err(e) => {
                tracing::warn!(
                    "Failed to generate query embedding for mission knowledge search: {}",
                    e
                );
                return Ok(format!("(MISSION KNOWLEDGE SEARCH ERROR: Failed to generate query embedding: {}. Knowledge search temporarily unavailable.)", e));
            }
        };

        let results_text = match self.query_mission_memory(ctx, vec).await {
            Ok(t) => t,
            Err(e) => return Ok(e),
        };

        if results_text.is_empty() {
            let hint = get_search_miss_hint(query);
            Ok(format!(
                "(RESOURCE NOT FOUND: No relevant shared findings found for '{}'. {})",
                query, hint
            ))
        } else {
            Ok(format!(
                "(SEARCH RESULTS FOR '{}'):\n{}",
                query, results_text
            ))
        }
    }

    /// Handles `search_mission_knowledge`: vector search across LanceDB memory scope.
    pub(crate) async fn handle_search_mission_knowledge(
        &self,
        ctx: &RunContext,
        fc: &crate::agent::types::ToolCall,
    ) -> Result<String, ToolExecutionError> {
        let query = require_str(ctx, &fc.args, "query", "search_mission_knowledge")?;
        tracing::info!(
            "🧠 [Memory] Agent {} searching knowledge for: {}",
            ctx.identity.agent_id,
            query
        );

        #[cfg(feature = "vector-memory")]
        {
            self.execute_mission_knowledge_search(ctx, &query).await
        }

        #[cfg(not(feature = "vector-memory"))]
        {
            let _ = (ctx, query);
            Ok("(MISSION KNOWLEDGE SEARCH UNAVAILABLE: vector-memory feature is not enabled in this build. Use 'list_files' or 'grep_search' to inspect workspace files directly.)".to_string())
        }
    }

    /// Helper to connect to mission memory
    #[cfg(feature = "vector-memory")]
    pub(crate) async fn connect_mission_memory(
        &self,
        ctx: &RunContext,
    ) -> Option<crate::agent::memory::VectorMemory> {
        let cluster_name = ctx
            .env
            .workspace_root
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| "default".to_string());
        let mission_scope_dir = format!(
            "data/workspaces/{}/missions/{}/scope.lance",
            cluster_name, ctx.mission_state.mission_id
        );
        let conn_res =
            crate::agent::memory::VectorMemory::connect(&mission_scope_dir, "scope").await;
        if let Err(ref e) = conn_res {
            tracing::warn!(
                "⚠️ [Mission Archival] Failed to connect for mission {}: {}",
                ctx.mission_state.mission_id,
                e
            );
        }
        conn_res.ok()
    }

    /// Connects to the global swarm vault.
    #[cfg(feature = "vector-memory")]
    pub(crate) async fn connect_global_vault(&self) -> Option<crate::agent::memory::VectorMemory> {
        let global_vault_path = self
            .state
            .base_dir
            .join("data/intelligence/global_vault.lance");
        let conn_res = crate::agent::memory::VectorMemory::connect(
            &global_vault_path.to_string_lossy(),
            "global",
        )
        .await;
        if let Err(ref e) = conn_res {
            tracing::warn!("⚠️ [Global Vault] Failed to connect: {}", e);
        }
        conn_res.ok()
    }

    /// Resolves the Google/Gemini API key from environment, provider registry, or active model config if Google.
    #[cfg(feature = "vector-memory")]
    pub(crate) fn resolve_google_api_key(&self, ctx: &RunContext) -> Option<String> {
        if let Ok(k) = std::env::var("GEMINI_API_KEY") {
            if !k.trim().is_empty() {
                return Some(k);
            }
        }
        if let Ok(k) = std::env::var("GOOGLE_API_KEY") {
            if !k.trim().is_empty() {
                return Some(k);
            }
        }
        if let Some(p) = self.state.registry.providers.get("google") {
            let key = p.api_key.as_deref().unwrap_or("").trim();
            if !key.is_empty() {
                return Some(key.to_string());
            }
        }
        get_provider_google_key(&ctx.model_config)
    }

    /// Helper to get Gemini embedding for a string, or log warning on failure.
    #[cfg(feature = "vector-memory")]
    pub(crate) async fn gemini_embed_or_log(
        &self,
        ctx: &RunContext,
        text: &str,
        operation: &str,
    ) -> Option<Vec<f32>> {
        let Some(api_key) = self.resolve_google_api_key(ctx) else {
            tracing::error!(
                "❌ [Global Vault] No Gemini/Google API key configured for {}. Aborting {}.",
                ctx.mission_state.mission_id,
                operation
            );
            return None;
        };
        let http_client = self.state.resources.http_client.clone();
        let text_owned = text.to_string();
        let embed_res =
            crate::agent::memory::get_gemini_embedding(&http_client, &api_key, &text_owned).await;
        if let Err(ref e) = embed_res {
            tracing::warn!(
                "⚠️ [Global Vault] Failed embedding for {}: {}",
                operation,
                e
            );
        }
        embed_res.ok()
    }
}

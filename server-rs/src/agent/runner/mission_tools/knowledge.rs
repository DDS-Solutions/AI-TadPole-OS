//! @docs ARCHITECTURE:Runner
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
//! - **Witness Tests**: none declared

use super::{require_str, require_str_opt};
use crate::agent::runner::tools::error::ToolExecutionError;
use crate::agent::runner::{AgentRunner, RunContext};

#[cfg(feature = "vector-memory")]
const RAG_TOP_K: usize = 5;
#[cfg(feature = "vector-memory")]
const IKS_MIN_CONFIDENCE: f32 = 0.3;
const OVERSIGHT_DESC_PREVIEW_BYTES: usize = 120;
const SENSITIVE_IKS_TOPICS: &[&str] = &["finance", "legal", "payroll", "pii", "medical"];

impl AgentRunner {
    /// Handles `search_mission_knowledge`: vector search across LanceDB memory scope.
    ///
    /// ### 🧩 RAG Fallback
    /// If no semantic findings are found, this function provides "Hints" to the
    /// agent to try physical filesystem tools (`list_files`, `grep_search`).
    pub(crate) async fn handle_search_mission_knowledge(
        &self,
        ctx: &RunContext,
        fc: &crate::agent::types::ToolCall,
    ) -> Result<String, ToolExecutionError> {
        let query = require_str(ctx, &fc.args, "query", "search_mission_knowledge")?;
        tracing::info!(
            "🧠 [Memory] Agent {} searching knowledge for: {}",
            ctx.agent_id,
            query
        );

        #[cfg(not(feature = "vector-memory"))]
        {
            let _ = ctx;
            Ok("(MISSION KNOWLEDGE SEARCH UNAVAILABLE: vector-memory feature is not enabled in this build. Use 'list_files' or 'grep_search' to inspect workspace files directly.)".to_string())
        }

        #[cfg(feature = "vector-memory")]
        {
            let api_key = match self.resolve_google_api_key(ctx) {
                Some(k) => k,
                None => {
                    return Ok(format!(
                        "(MISSION KNOWLEDGE SEARCH UNAVAILABLE: Gemini API key not configured for vector embeddings. Set GEMINI_API_KEY or GOOGLE_API_KEY to enable semantic memory search. Use 'list_files' or 'grep_search' to inspect workspace files directly.)"
                    ));
                }
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
                    return Ok(format!(
                        "(MISSION KNOWLEDGE SEARCH ERROR: Failed to generate query embedding: {}. Knowledge search temporarily unavailable.)",
                        e
                    ));
                }
            };

            let mut results_text = String::new();
            if let Some(mission_mem) = self.connect_mission_memory(ctx).await {
                match mission_mem.search_knowledge(vec, RAG_TOP_K).await {
                    Ok(results) => {
                        for (i, text) in results.into_iter().enumerate() {
                            results_text.push_str(&format!("[Result {}]: {}\n", i + 1, text));
                        }
                    }
                    Err(e) => {
                        tracing::warn!("Failed to search mission memory database: {}", e);
                        return Ok(format!(
                            "(MISSION KNOWLEDGE SEARCH ERROR: Memory store query failed: {}. Use workspace inspection tools.)",
                            e
                        ));
                    }
                }
            }

            if results_text.is_empty() {
                let lower_query = query.to_lowercase();
                let is_financial = lower_query.contains("budget")
                    || lower_query.contains("cost")
                    || lower_query.contains("limit")
                    || lower_query.contains("usd");

                let hint = if is_financial {
                    "HINT: This query appears to relate to live financial metrics. Vector RAG only contains static shared findings. Use 'get_agent_metrics' to see your own current budget/costs, or 'query_financial_logs' to review overall mission history."
                } else {
                    "This query might be reference a physical file or keyword in the workspace. Since you have technical tools, you should now use 'list_files' or 'grep_search' to locate the target and then 'read_file' or 'read_codebase_file' to inspect it directly."
                };

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
            ctx.agent_id,
            topic
        );
        #[cfg(feature = "vector-memory")]
        {
            let index_text = if topic.is_empty() || topic == "General" {
                summary.clone()
            } else {
                format!("[{}] {}", topic, summary)
            };

            if let Some(vec) = self.gemini_embed_or_log(ctx, &index_text, "archive").await {
                if let Some(vault) = self.connect_global_vault().await {
                    let id = format!("global-{}", uuid::Uuid::new_v4());
                    let stored_text = format!(
                        "### [{}] Archival Nugget\n**Summary**: {}\n\n**Content**:\n{}",
                        topic, summary, content
                    );
                    match vault
                        .add_memory(&id, &stored_text, &ctx.mission_id, vec)
                        .await
                    {
                        Ok(_) => {
                            self.broadcast_agent(
                                ctx,
                                &format!("🏛️ Global Vault: nugget archived on {}", topic),
                                "success",
                            );
                            return Ok(format!("(GLOBAL ARCHIVE SUCCESS): Nugget on '{}' added to the swarm intelligence vault.", topic));
                        }
                        Err(e) => {
                            tracing::warn!(
                                "⚠️ [Global Vault] Failed to add memory entry to global vault for mission {}: {}",
                                ctx.mission_id,
                                e
                            );
                        }
                    }
                }
            }
        }

        #[cfg(not(feature = "vector-memory"))]
        {
            let _ = (content, summary);
        }

        Ok(
            "(GLOBAL ARCHIVE FAILED): Ensure vector-memory is enabled and API keys are valid."
                .to_string(),
        )
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
            ctx.agent_id,
            query
        );

        #[cfg(feature = "vector-memory")]
        {
            if let Some(vec) = self.gemini_embed_or_log(ctx, &query, "search").await {
                if let Some(vault) = self.connect_global_vault().await {
                    match vault.search_knowledge(vec, RAG_TOP_K).await {
                        Ok(results) => {
                            if results.is_empty() {
                                return Ok(format!(
                                    "(GLOBAL SEARCH): No relevant intelligence found for '{}'.",
                                    query
                                ));
                            } else {
                                return Ok(format!(
                                    "(GLOBAL INTELLIGENCE RETRIEVED for '{}'):\n\n{}",
                                    query,
                                    results.join("\n\n----- \n\n")
                                ));
                            }
                        }
                        Err(e) => {
                            tracing::warn!(
                                "⚠️ [Global Vault] Failed to search global vault records: {}",
                                e
                            );
                        }
                    }
                }
            }
        }

        Ok(
            "(GLOBAL SEARCH FAILED): Ensure vector-memory is enabled and API keys are valid."
                .to_string(),
        )
    }

    /// Handles `store_knowledge`: writes a new curated knowledge entry to the IKS.
    /// Deduplicates by content hash. Escapes to oversight if topic is sensitive.
    pub(crate) async fn handle_store_knowledge(
        &self,
        ctx: &RunContext,
        fc: &crate::agent::types::ToolCall,
    ) -> Result<String, ToolExecutionError> {
        let text = require_str(ctx, &fc.args, "text", "store_knowledge")?;
        let topic = require_str(ctx, &fc.args, "topic", "store_knowledge")?;

        // Topic-ACL Check
        let topic_lower = topic.to_lowercase();
        let is_sensitive = SENSITIVE_IKS_TOPICS
            .iter()
            .any(|&s| topic_lower.contains(s));

        if is_sensitive {
            self.broadcast_agent(
                ctx,
                &format!(
                    "🧠 Oversight: wants to store sensitive knowledge under '{}'. Review required.",
                    topic
                ),
                "warning",
            );

            let approved = self
                .submit_oversight(
                    crate::agent::types::ToolCallAudit {
                        id: uuid::Uuid::new_v4().to_string(),
                        agent_id: ctx.agent_id.clone(),
                        mission_id: Some(ctx.mission_id.clone()),
                        skill: "store_knowledge".to_string(),
                        params: fc.args.clone(),
                        department: ctx.department.clone(),
                        description: format!(
                            "Storing sensitive knowledge on topic '{}': {}",
                            topic,
                            {
                                let mut end = OVERSIGHT_DESC_PREVIEW_BYTES;
                                if text.len() > end {
                                    while end > 0 && !text.is_char_boundary(end) {
                                        end -= 1;
                                    }
                                    &text[..end]
                                } else {
                                    &text
                                }
                            }
                        ),
                        timestamp: chrono::Utc::now().to_rfc3339(),
                    },
                    Some(ctx.mission_id.clone()),
                )
                .await?;

            if !approved {
                return Ok("(IKS write REJECTED by Oversight)".to_string());
            }
        }

        #[cfg(feature = "vector-memory")]
        {
            let cluster_id = fc
                .args
                .get("cluster_id")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            let confidence = fc
                .args
                .get("confidence")
                .and_then(|v| v.as_f64())
                .map(|f| f as f32);
            let ttl_days = fc.args.get("ttl_days").and_then(|v| v.as_i64());

            let req = crate::agent::knowledge_store::AddKnowledgeRequest {
                text,
                topic: topic.clone(),
                cluster_id,
                source_node_id: None,
                source_agent_id: Some(ctx.agent_id.clone()),
                confidence,
                ttl_days,
                concept_type: None,
                title: None,
                description: None,
                resource_uri: None,
                tags: None,
                security_tier: None,
                parent_id: None,
            };

            match self.state.resources.get_knowledge_store().await {
                Ok(ks) => {
                    match ks
                        .add_entry(req, self.state.resources.http_client.as_ref().clone())
                        .await
                    {
                        Ok(entry) => {
                            if is_sensitive {
                                let _ = ks.confirm(&entry.id).await;
                            }
                            let msg = format!(
                                "(STORE KNOWLEDGE SUCCESS): Entry stored with ID: {}",
                                entry.id
                            );
                            self.broadcast_agent(
                                ctx,
                                &format!("🧠 Curated fact stored in IKS on topic '{}'", topic),
                                "success",
                            );
                            return Ok(msg);
                        }
                        Err(e) => {
                            return Ok(format!("(STORE KNOWLEDGE FAILED: {})", e));
                        }
                    }
                }
                Err(e) => {
                    return Ok(format!(
                        "(STORE KNOWLEDGE FAILED: Could not acquire store: {})",
                        e
                    ));
                }
            }
        }

        #[cfg(not(feature = "vector-memory"))]
        {
            Ok("(STORE KNOWLEDGE FAILED: Vector memory is disabled on this node.)".to_string())
        }
    }

    /// Handles `search_knowledge`: searches across the cross-cluster persistent Institutional Knowledge Store.
    pub(crate) async fn handle_search_knowledge(
        &self,
        ctx: &RunContext,
        fc: &crate::agent::types::ToolCall,
    ) -> Result<String, ToolExecutionError> {
        let query = require_str(ctx, &fc.args, "query", "search_knowledge")?;

        tracing::info!(
            "🧠 [IKS] Agent {} searching Institutional Knowledge Store for: {}",
            ctx.agent_id,
            query
        );

        #[cfg(feature = "vector-memory")]
        {
            let topic = require_str_opt(ctx, &fc.args, "topic", "search_knowledge")?;
            let limit = fc
                .args
                .get("limit")
                .and_then(|v| v.as_u64())
                .map(|n| n as usize);

            let req = crate::agent::knowledge_store::KnowledgeSearchRequest {
                query,
                topic,
                cluster_id: None, // Global + local cluster scoped
                limit,
                min_confidence: Some(IKS_MIN_CONFIDENCE),
                concept_type: None,
                security_tier: None,
            };

            match self.state.resources.get_knowledge_store().await {
                Ok(ks) => {
                    match ks
                        .search(&req, self.state.resources.http_client.as_ref().clone())
                        .await
                    {
                        Ok(results) => {
                            if results.is_empty() {
                                return Ok(format!("(IKS SEARCH): No relevant institutional knowledge found for '{}'.", req.query));
                            } else {
                                let mut lines = Vec::new();
                                for (i, entry) in results.into_iter().enumerate() {
                                    lines.push(format!(
                                        "[Entry {} (ID: {}, Topic: '{}', Confidence: {:.2})]:\n{}",
                                        i + 1,
                                        entry.id,
                                        entry.topic,
                                        entry.confidence,
                                        entry.text
                                    ));
                                }
                                return Ok(format!(
                                    "(INSTITUTIONAL KNOWLEDGE RETRIEVED for '{}'):\n\n{}",
                                    req.query,
                                    lines.join("\n\n----- \n\n")
                                ));
                            }
                        }
                        Err(e) => {
                            return Ok(format!("(SEARCH KNOWLEDGE FAILED: {})", e));
                        }
                    }
                }
                Err(e) => {
                    return Ok(format!(
                        "(SEARCH KNOWLEDGE FAILED: Could not acquire store: {})",
                        e
                    ));
                }
            }
        }

        #[cfg(not(feature = "vector-memory"))]
        {
            Ok("(SEARCH KNOWLEDGE FAILED: Vector memory is disabled on this node.)".to_string())
        }
    }

    /// Helper to connect to mission memory
    #[cfg(feature = "vector-memory")]
    pub(crate) async fn connect_mission_memory(
        &self,
        ctx: &RunContext,
    ) -> Option<crate::agent::memory::VectorMemory> {
        let cluster_name = ctx
            .workspace_root
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| "default".to_string());
        let mission_scope_dir = format!(
            "data/workspaces/{}/missions/{}/scope.lance",
            cluster_name, ctx.mission_id
        );
        match crate::agent::memory::VectorMemory::connect(&mission_scope_dir, "scope").await {
            Ok(mem) => Some(mem),
            Err(e) => {
                tracing::warn!(
                    "⚠️ [Mission Archival] Failed to connect to mission scope vector memory for mission {}: {}",
                    ctx.mission_id,
                    e
                );
                None
            }
        }
    }

    /// Connects to the global swarm vault.
    #[cfg(feature = "vector-memory")]
    pub(crate) async fn connect_global_vault(&self) -> Option<crate::agent::memory::VectorMemory> {
        let global_vault_path = self
            .state
            .base_dir
            .join("data/intelligence/global_vault.lance");
        match crate::agent::memory::VectorMemory::connect(
            &global_vault_path.to_string_lossy(),
            "global",
        )
        .await
        {
            Ok(vault) => Some(vault),
            Err(e) => {
                tracing::warn!(
                    "⚠️ [Global Vault] Failed to connect to global vault at {:?}: {}",
                    global_vault_path,
                    e
                );
                None
            }
        }
    }

    /// Resolves the Google/Gemini API key from environment, provider registry, or active model config if Google.
    #[cfg(feature = "vector-memory")]
    pub(crate) fn resolve_google_api_key(&self, ctx: &RunContext) -> Option<String> {
        std::env::var("GEMINI_API_KEY")
            .or_else(|_| std::env::var("GOOGLE_API_KEY"))
            .ok()
            .filter(|k| !k.trim().is_empty())
            .or_else(|| {
                self.state
                    .registry
                    .providers
                    .get("google")
                    .and_then(|p| p.api_key.clone())
                    .filter(|k| !k.trim().is_empty())
            })
            .or_else(|| {
                let provider_name = ctx.model_config.provider.to_string().to_lowercase();
                if provider_name == "google" || provider_name.contains("gemini") {
                    ctx.model_config
                        .api_key
                        .clone()
                        .filter(|k| !k.trim().is_empty())
                } else {
                    None
                }
            })
    }

    /// Helper to get Gemini embedding for a string, or log warning on failure.
    #[cfg(feature = "vector-memory")]
    pub(crate) async fn gemini_embed_or_log(
        &self,
        ctx: &RunContext,
        text: &str,
        operation: &str,
    ) -> Option<Vec<f32>> {
        let api_key = match self.resolve_google_api_key(ctx) {
            Some(k) => k,
            None => {
                tracing::error!(
                    "❌ [Global Vault] No Gemini/Google API key configured (mission {}). Aborting {}.",
                    ctx.mission_id,
                    operation
                );
                return None;
            }
        };
        let http_client = self.state.resources.http_client.clone();
        let text_owned = text.to_string();
        match crate::agent::memory::get_gemini_embedding(&http_client, &api_key, &text_owned).await
        {
            Ok(vec) => Some(vec),
            Err(e) => {
                tracing::warn!(
                    "⚠️ [Global Vault] Failed to generate embedding for {} in mission {}: {}",
                    operation,
                    ctx.mission_id,
                    e
                );
                None
            }
        }
    }
}

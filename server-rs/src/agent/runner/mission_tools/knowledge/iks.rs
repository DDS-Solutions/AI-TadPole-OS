//! @docs ARCHITECTURE:Runner
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Agent Runner / mission_tools / knowledge / iks
//!
//! ### AI Assist Note
//! Institutional Knowledge Store (IKS) curated facts storage and search.
//!
//! ### 🔍 Debugging & Observability
//! - **Telemetry Targets**: `[IKS]`

use super::super::{require_str, require_str_opt};
use crate::agent::runner::tools::error::ToolExecutionError;
use crate::agent::runner::{AgentRunner, RunContext};

#[cfg(feature = "vector-memory")]
const IKS_MIN_CONFIDENCE: f32 = 0.3;
const OVERSIGHT_DESC_PREVIEW_BYTES: usize = 120;
const SENSITIVE_IKS_TOPICS: &[&str] = &["finance", "legal", "payroll", "pii", "medical"];

fn is_topic_sensitive(topic: &str) -> bool {
    let lower = topic.to_lowercase();
    SENSITIVE_IKS_TOPICS.iter().any(|&s| lower.contains(s))
}

fn truncate_for_oversight(text: &str) -> &str {
    let mut end = OVERSIGHT_DESC_PREVIEW_BYTES;
    if text.len() <= end {
        return text;
    }
    while end > 0 && !text.is_char_boundary(end) {
        end -= 1;
    }
    &text[..end]
}

#[cfg(feature = "vector-memory")]
fn format_iks_search_results(
    query: &str,
    results: Vec<crate::agent::knowledge_store::KnowledgeSearchResult>,
) -> String {
    if results.is_empty() {
        return format!(
            "(IKS SEARCH): No relevant institutional knowledge found for '{}'.",
            query
        );
    }
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
    format!(
        "(INSTITUTIONAL KNOWLEDGE RETRIEVED for '{}'):\n\n{}",
        query,
        lines.join("\n\n----- \n\n")
    )
}

impl AgentRunner {
    async fn request_iks_oversight(
        &self,
        ctx: &RunContext,
        fc: &crate::agent::types::ToolCall,
        topic: &str,
        text: &str,
    ) -> Result<bool, ToolExecutionError> {
        self.broadcast_agent(
            ctx,
            &format!(
                "🧠 Oversight: wants to store sensitive knowledge under '{}'. Review required.",
                topic
            ),
            "warning",
        );

        let preview = truncate_for_oversight(text);
        let audit = crate::agent::types::ToolCallAudit {
            id: uuid::Uuid::new_v4().to_string(),
            agent_id: ctx.identity.agent_id.clone(),
            mission_id: Some(ctx.mission_state.mission_id.clone()),
            skill: "store_knowledge".to_string(),
            params: fc.args.clone(),
            department: ctx.identity.department.clone(),
            description: format!(
                "Storing sensitive knowledge on topic '{}': {}",
                topic, preview
            ),
            timestamp: chrono::Utc::now().to_rfc3339(),
        };

        self.submit_oversight(audit, Some(ctx.mission_state.mission_id.clone()))
            .await
            .map_err(ToolExecutionError::AppError)
    }

    #[cfg(feature = "vector-memory")]
    async fn insert_iks_entry(
        &self,
        ctx: &RunContext,
        fc: &crate::agent::types::ToolCall,
        text: String,
        topic: String,
        is_sensitive: bool,
    ) -> Result<String, ToolExecutionError> {
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
            source_agent_id: Some(ctx.identity.agent_id.clone()),
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

        let ks = self
            .state
            .resources
            .get_knowledge_store()
            .await
            .map_err(|e| {
                ToolExecutionError::AppError(crate::error::AppError::Anyhow(anyhow::anyhow!(e)))
            })?;

        let entry = ks
            .add_entry(req, self.state.resources.http_client.as_ref().clone())
            .await
            .map_err(|e| {
                ToolExecutionError::AppError(crate::error::AppError::Anyhow(anyhow::anyhow!(e)))
            })?;

        if is_sensitive {
            let _ = ks.confirm(&entry.id).await;
        }

        self.broadcast_agent(
            ctx,
            &format!("🧠 Curated fact stored in IKS on topic '{}'", topic),
            "success",
        );
        Ok(format!(
            "(STORE KNOWLEDGE SUCCESS): Entry stored with ID: {}",
            entry.id
        ))
    }

    /// Handles `store_knowledge`: writes a new curated knowledge entry to the IKS.
    pub(crate) async fn handle_store_knowledge(
        &self,
        ctx: &RunContext,
        fc: &crate::agent::types::ToolCall,
    ) -> Result<String, ToolExecutionError> {
        let text = require_str(ctx, &fc.args, "text", "store_knowledge")?;
        let topic = require_str(ctx, &fc.args, "topic", "store_knowledge")?;
        let is_sensitive = is_topic_sensitive(&topic);

        if is_sensitive && !self.request_iks_oversight(ctx, fc, &topic, &text).await? {
            return Ok("(IKS write REJECTED by Oversight)".to_string());
        }

        #[cfg(feature = "vector-memory")]
        {
            match self
                .insert_iks_entry(ctx, fc, text, topic, is_sensitive)
                .await
            {
                Ok(msg) => Ok(msg),
                Err(e) => Ok(format!("(STORE KNOWLEDGE FAILED: {})", e)),
            }
        }

        #[cfg(not(feature = "vector-memory"))]
        {
            let _ = (text, topic, is_sensitive);
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
            ctx.identity.agent_id,
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
                query: query.clone(),
                topic,
                cluster_id: None,
                limit,
                min_confidence: Some(IKS_MIN_CONFIDENCE),
                concept_type: None,
                security_tier: None,
            };

            let ks = match self.state.resources.get_knowledge_store().await {
                Ok(k) => k,
                Err(e) => {
                    return Ok(format!(
                        "(SEARCH KNOWLEDGE FAILED: Could not acquire store: {})",
                        e
                    ))
                }
            };

            match ks
                .search(&req, self.state.resources.http_client.as_ref().clone())
                .await
            {
                Ok(results) => Ok(format_iks_search_results(&query, results)),
                Err(e) => Ok(format!("(SEARCH KNOWLEDGE FAILED: {})", e)),
            }
        }

        #[cfg(not(feature = "vector-memory"))]
        {
            let _ = query;
            Ok("(SEARCH KNOWLEDGE FAILED: Vector memory is disabled on this node.)".to_string())
        }
    }
}

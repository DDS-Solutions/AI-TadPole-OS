//! @docs ARCHITECTURE:Runner
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Agent Runner / context
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Type-safe state handling and bounded execution without unhandled panics.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: none declared
//! - **Witness Tests**: none declared

use crate::agent::runner::RunContext;
use crate::error::AppError;
use crate::state::AppState;

const MAX_DIRECTIVE_ITEM_LEN: usize = 1000;
const MAX_REVIEW_ITEM_LEN: usize = 1000;
#[allow(dead_code)]
const MAX_VAULT_RESULTS: usize = 3;
#[allow(dead_code)]
const IKS_MIN_CONFIDENCE: f32 = 0.4;

/// Result container for pending directives and peer reviews.
#[derive(Debug, Clone)]
pub struct ExternalContext {
    pub directives: String,
    pub reviews: String,
}

impl From<(String, String)> for ExternalContext {
    fn from((directives, reviews): (String, String)) -> Self {
        Self {
            directives,
            reviews,
        }
    }
}

impl From<ExternalContext> for (String, String) {
    fn from(ctx: ExternalContext) -> Self {
        (ctx.directives, ctx.reviews)
    }
}

/// Fetches pending directives and review tasks for an agent with safe fencing, length limits,
/// and explicit failure state handling (failing closed rather than falsely asserting absence).
pub async fn fetch_external_context(ctx: &RunContext, state: &AppState) -> (String, String) {
    let directives_res = super::super::swarm_persistence::get_pending_directives(
        &state.resources.pool,
        &ctx.agent_id,
    )
    .await;

    let reviews_res =
        super::super::swarm_persistence::get_pending_reviews(&state.resources.pool, &ctx.agent_id)
            .await;

    let directives_str = match directives_res {
        Ok(pending) if pending.is_empty() => {
            "No active directives. Proceed with mission objectives.".to_string()
        }
        Ok(pending) => pending
            .iter()
            .map(|d| {
                let sanitized_instruction =
                    crate::agent::runner::safe_truncate_str(&d.instruction, MAX_DIRECTIVE_ITEM_LEN);
                format!(
                    "- [Directive] From {}: \"\"\"\n{}\n\"\"\"",
                    d.source_agent_id, sanitized_instruction
                )
            })
            .collect::<Vec<_>>()
            .join("\n"),
        Err(e) => {
            tracing::warn!(
                "⚠️ [Global Intelligence] Failed to query pending directives for agent {}: {}",
                ctx.agent_id,
                e
            );
            "⚠️ DIRECTIVE_LOOKUP_UNAVAILABLE: Could not query pending directives (database error). Do not assume absence of oversight directives.".to_string()
        }
    };

    let reviews_str = match reviews_res {
        Ok(pending) if pending.is_empty() => {
            "No peer reviews pending. Maintain standard quality protocols.".to_string()
        }
        Ok(pending) => pending
            .iter()
            .map(|r| {
                let sanitized_content = crate::agent::runner::safe_truncate_str(
                    &r.content_to_review,
                    MAX_REVIEW_ITEM_LEN,
                );
                format!(
                    "- [Review Task] Target: {}. Requirement: \"\"\"\n{}\n\"\"\"",
                    r.requester_id, sanitized_content
                )
            })
            .collect::<Vec<_>>()
            .join("\n"),
        Err(e) => {
            tracing::warn!(
                "⚠️ [Global Intelligence] Failed to query pending reviews for agent {}: {}",
                ctx.agent_id,
                e
            );
            "⚠️ PEER_REVIEW_LOOKUP_UNAVAILABLE: Could not query peer reviews (database error)."
                .to_string()
        }
    };

    (directives_str, reviews_str)
}

pub async fn fetch_identity(state: &AppState) -> Result<String, AppError> {
    state.resources.get_identity_context().await
}

pub async fn fetch_memory(state: &AppState) -> Result<String, AppError> {
    state.resources.get_memory_context().await
}

pub async fn fetch_mission_context(ctx: &RunContext, state: &AppState) -> Result<String, AppError> {
    crate::agent::mission::get_mission_context(&state.resources.pool, &ctx.mission_id).await
}

/// Gathers global intelligence across semantic vault and Institutional Knowledge Store (IKS).
///
/// Ensures strict credential safety (Google API key only used for Gemini embeddings when explicitly configured),
/// scoped cluster/clearance boundaries, and graceful degradation to text-only IKS retrieval.
pub async fn gather_global_intelligence(
    ctx: &RunContext,
    state: &AppState,
    query: &str,
) -> Result<String, AppError> {
    tracing::debug!(
        "🧠 [Global Intelligence] Agent {} querying knowledge vault (query length: {} chars)",
        ctx.agent_id,
        query.len()
    );

    #[cfg(feature = "vector-memory")]
    {
        let mut intelligence_parts = Vec::new();

        // 1. Swarm Vault (Raw mission embeddings)
        // Probe vault availability first before spending an embedding call
        let vault_opt = state.resources.get_swarm_vault().await.ok();

        // Explicit Google credential resolution: NEVER send foreign provider keys (Anthropic/Groq/Mistral) to Google
        let google_api_key = std::env::var("GEMINI_API_KEY")
            .or_else(|_| std::env::var("GOOGLE_API_KEY"))
            .ok()
            .filter(|k| !k.trim().is_empty())
            .or_else(|| {
                state
                    .registry
                    .providers
                    .get("google")
                    .and_then(|p| p.api_key.clone())
                    .filter(|k| !k.trim().is_empty())
            });

        if let (Some(vault), Some(api_key)) = (vault_opt, google_api_key) {
            let http_client = state.resources.http_client.clone();
            let query_owned = query.to_string();

            match crate::agent::memory::get_gemini_embedding(&http_client, &api_key, &query_owned)
                .await
            {
                Ok(vec) => match vault.search_knowledge(vec, MAX_VAULT_RESULTS).await {
                    Ok(results) if !results.is_empty() => {
                        intelligence_parts.push(format!(
                            "Relevant Swarm Vault intelligence:\n- {}",
                            results.join("\n- ")
                        ));
                    }
                    Ok(_) => {}
                    Err(e) => {
                        tracing::warn!("⚠️ [Global Intelligence] Vault search query failed: {}", e);
                    }
                },
                Err(e) => {
                    tracing::warn!(
                        "⚠️ [Global Intelligence] Gemini embedding failed (degrading to text IKS search): {}",
                        e
                    );
                }
            }
        } else {
            tracing::debug!(
                "ℹ️ [Global Intelligence] Gemini embedding skipped (no Google API key or vault unavailable); using IKS text search."
            );
        }

        // 2. Institutional Knowledge Store (IKS: Curated, scoped facts)
        if let Ok(ks) = state.resources.get_knowledge_store().await {
            // Scope search to the agent's active cluster to maintain multi-tenant privacy boundaries
            let iks_req = crate::agent::knowledge_store::KnowledgeSearchRequest {
                query: query.to_string(),
                topic: None,
                cluster_id: ctx.cluster_id.clone(),
                limit: Some(MAX_VAULT_RESULTS),
                min_confidence: Some(IKS_MIN_CONFIDENCE),
                concept_type: None,
                security_tier: None,
            };

            match ks
                .search(&iks_req, state.resources.http_client.as_ref().clone())
                .await
            {
                Ok(results) if !results.is_empty() => {
                    let iks_lines: Vec<String> = results
                        .into_iter()
                        .map(|entry| {
                            format!(
                                "- [Topic: '{}', Conf: {:.2}] {}",
                                entry.topic, entry.confidence, entry.text
                            )
                        })
                        .collect();
                    intelligence_parts.push(format!(
                        "Institutional Knowledge (IKS):\n{}",
                        iks_lines.join("\n")
                    ));
                }
                Ok(_) => {}
                Err(e) => {
                    tracing::warn!("⚠️ [Global Intelligence] IKS search failed: {}", e);
                }
            }
        }

        if !intelligence_parts.is_empty() {
            return Ok(intelligence_parts.join("\n\n"));
        }
    }

    #[cfg(not(feature = "vector-memory"))]
    {
        let _ = state;
    }

    Ok("No relevant global intelligence found in the vault.".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::runner::RunContext;
    use crate::state::AppState;

    #[tokio::test]
    async fn test_fetch_external_context_empty() {
        let state = AppState::new_minimal_mock().await;
        let ctx = RunContext {
            agent_id: "test_agent".to_string(),
            mission_id: "test_mission".to_string(),
            ..RunContext::default()
        };

        let (directives, reviews) = fetch_external_context(&ctx, &state).await;
        assert!(directives.contains("No active directives"));
        assert!(reviews.contains("No peer reviews pending"));
    }

    #[tokio::test]
    async fn test_gather_global_intelligence_safe_fallback() {
        let state = AppState::new_minimal_mock().await;
        let ctx = RunContext {
            agent_id: "claude_agent".to_string(),
            mission_id: "test_mission".to_string(),
            cluster_id: Some("cluster_alpha".to_string()),
            ..RunContext::default()
        };

        // Even with non-Google provider and no Google key, it must not crash or leak
        let res = gather_global_intelligence(&ctx, &state, "test query").await;
        assert!(res.is_ok());
    }
}

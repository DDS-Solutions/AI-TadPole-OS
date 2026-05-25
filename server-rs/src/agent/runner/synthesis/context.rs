//! @docs ARCHITECTURE:Registry
//! 
//! ### AI Assist Note
//! **Core technical resource for the Tadpole OS Sovereign infrastructure.**
//! This module implements high-fidelity logic for the Sovereign Reality layer.
//! 
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: Runtime logic error, state desynchronization, or resource exhaustion.
//! - **Telemetry Link**: Search `[context]` in tracing logs.

use crate::agent::runner::RunContext;
use crate::state::AppState;
use crate::error::AppError;

pub async fn fetch_external_context(ctx: &RunContext, state: &AppState) -> (String, String) {
    let pending_directives = super::super::swarm_persistence::get_pending_directives(
        &state.resources.pool,
        &ctx.agent_id,
    )
    .await
    .unwrap_or_default();
    let pending_reviews = super::super::swarm_persistence::get_pending_reviews(
        &state.resources.pool,
        &ctx.agent_id,
    )
    .await
    .unwrap_or_default();

    let directives_str = if pending_directives.is_empty() {
        "No active directives. Proceed with mission objectives.".to_string()
    } else {
        pending_directives
            .iter()
            .map(|d| {
                format!(
                    "- [Directive] From {}: {}",
                    d.source_agent_id, d.instruction
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    };

    let reviews_str = if pending_reviews.is_empty() {
        "No peer reviews pending. Maintain standard quality protocols.".to_string()
    } else {
        pending_reviews
            .iter()
            .map(|r| {
                format!(
                    "- [Review Task] Target: {}. Requirement: {}",
                    r.requester_id, r.content_to_review
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    };

    (directives_str, reviews_str)
}

pub async fn fetch_identity(state: &AppState) -> Result<String, AppError> {
    state.resources.get_identity_context().await
}

pub async fn fetch_memory(state: &AppState) -> Result<String, AppError> {
    state.resources.get_memory_context().await
}

pub async fn fetch_mission_context(
    ctx: &RunContext,
    state: &AppState,
) -> Result<String, AppError> {
    crate::agent::mission::get_mission_context(&state.resources.pool, &ctx.mission_id)
        .await
}

pub async fn gather_global_intelligence(
    ctx: &RunContext,
    _state: &AppState,
    query: &str,
) -> Result<String, AppError> {
    tracing::info!(
        "🧠 [Global Intelligence] Agent {} querying global vault for: {}",
        ctx.agent_id,
        query
    );

    #[cfg(feature = "vector-memory")]
    {
        let api_key = ctx.model_config.api_key.clone().unwrap_or_else(|| {
            _state.registry.providers
                .get(&ctx.model_config.provider.to_string())
                .and_then(|p| p.api_key.clone())
                .unwrap_or_default()
        });
        let http_client = _state.resources.http_client.clone();

        let vec = crate::agent::memory::get_gemini_embedding(&http_client, &api_key, query).await?;
        let vault = _state.resources.get_swarm_vault().await?;
        let results = vault.search_knowledge(vec, 3).await?;

        if !results.is_empty() {
            return Ok(format!(
                "Relevant global intelligence retrieved:\n- {}",
                results.join("\n- ")
            ));
        }
    }

    Ok("No relevant global intelligence found in the vault.".to_string())
}

// Metadata: [context]

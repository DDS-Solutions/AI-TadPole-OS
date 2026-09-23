//! @docs ARCHITECTURE:State
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / State / Init Services
//! - **Primary Entrypoints**: `create_payment_router`, `init_http_client`, `init_audio_cache`, `validate_tool_configurations`
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none declared
//! - **Telemetry Targets**: none declared
//! - **Witness Tests**: none declared

use crate::error::AppError;
use std::path::Path;
use std::sync::Arc;

pub fn init_http_client() -> Result<Arc<reqwest::Client>, AppError> {
    tracing::info!("🚀 [Engines] Initializing HTTP Client...");
    let client = reqwest::Client::builder()
        .user_agent(concat!("TadpoleOS/", env!("CARGO_PKG_VERSION")))
        .pool_max_idle_per_host(100) // 🛡️ [Hardening] Increase pool for 10+ agent swarms
        .pool_idle_timeout(std::time::Duration::from_secs(60))
        .timeout(std::time::Duration::from_secs(300))
        .build()
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;
    Ok(Arc::new(client))
}

pub async fn init_audio_cache(base_dir: &Path) -> Arc<crate::agent::audio_cache::BunkerCache> {
    let audio_cache_path = base_dir.join("data").join("audio_cache.db");
    tracing::info!(
        "🚀 [Engines] Initializing Audio Cache at {}...",
        audio_cache_path.display()
    );
    match crate::agent::audio_cache::BunkerCache::new(audio_cache_path.clone()).await {
        Ok(cache) => Arc::new(cache),
        Err(e) => {
            tracing::warn!(
                "⚠️ [Engines] Audio Cache failed to initialize at {}: {:?}. Falling back to no-op mode.",
                audio_cache_path.display(),
                e
            );
            Arc::new(crate::agent::audio_cache::BunkerCache::new_noop().await)
        }
    }
}

pub fn create_payment_router(
    pool: sqlx::SqlitePool,
) -> Arc<crate::agent::runner::a2a_router::PaymentRouter> {
    let dev_adapter = Arc::new(crate::agent::runner::a2a_router::LocalMockAdapter);
    let staging_adapter = Arc::new(crate::agent::runner::a2a_router::LocalMockAdapter);

    let web3_rpc = std::env::var("A2A_WEB3_RPC_URL").unwrap_or_default();
    let web3_vault = std::env::var("A2A_WEB3_VAULT_ADDRESS").unwrap_or_default();

    let prod_adapter: Arc<dyn crate::agent::runner::a2a_router::A2APaymentAdapter> =
        if !web3_rpc.is_empty() && !web3_vault.is_empty() {
            Arc::new(crate::agent::runner::a2a_router::L3HybridAdapter::new(
                web3_rpc, web3_vault,
            ))
        } else {
            Arc::new(crate::agent::runner::a2a_router::LocalMockAdapter)
        };

    Arc::new(crate::agent::runner::a2a_router::PaymentRouter::new(
        pool,
        dev_adapter,
        staging_adapter,
        prod_adapter,
    ))
}

pub fn validate_tool_configurations(
    registry: &crate::state::hubs::reg::RegistryHub,
) -> Result<(), AppError> {
    let strict_tool_val = std::env::var("STRICT_TOOL_VALIDATION")
        .map(|s| s.to_lowercase() == "true" || s == "1")
        .unwrap_or(false);

    let known_tools: std::collections::HashSet<String> = registry
        .tool_registry
        .list_tools()
        .into_iter()
        .map(|t| t.name)
        .collect();

    for entry in registry.agents.iter() {
        let agent = entry.value();
        let mut candidate_tools = agent.capabilities.mcp_tools.clone();
        if let Some(mcp) = &agent.models.model.mcp_tools {
            candidate_tools.extend(mcp.clone());
        }
        if let Some(meta_tools) = agent.metadata.get("tools").and_then(|v| v.as_array()) {
            for t in meta_tools {
                if let Some(name) = t.as_str() {
                    candidate_tools.push(name.to_string());
                }
            }
        }

        for tool_name in &candidate_tools {
            if !known_tools.contains(tool_name) {
                tracing::warn!(
                    "🚨 [ToolValidator] Agent '{}' references unknown tool '{}' — likely typo or missing plugin!",
                    agent.identity.id,
                    tool_name
                );
                if strict_tool_val {
                    return Err(AppError::BadRequest(format!(
                        "Tool configuration error: Agent '{}' references unknown tool '{}'",
                        agent.identity.id, tool_name
                    )));
                }
            }
        }
    }

    Ok(())
}

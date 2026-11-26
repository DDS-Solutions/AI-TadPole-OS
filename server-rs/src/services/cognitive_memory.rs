//! @docs ARCHITECTURE:Core:CognitiveMemory
//!
//! ### AI Context Alignment
//! - **Subsystem**: Frontend Service Layer / cognitive_memory
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Asynchronous service calls normalize response envelopes and propagate typed errors.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: `[CognitiveMemory]`

use crate::startup::{SystemContext, SystemService};
use async_trait::async_trait;

pub struct CognitiveMemoryPipelineService;

#[cfg(feature = "vector-memory")]
fn should_terminate_pipeline(
    res: Result<(), tokio::sync::watch::error::RecvError>,
    shutdown_rx: &tokio::sync::watch::Receiver<bool>,
) -> bool {
    match res {
        Ok(_) if *shutdown_rx.borrow() => {
            tracing::info!("🛑 [CognitiveMemory] Pipeline shutting down gracefully.");
            true
        }
        Err(_) => {
            tracing::info!("🛑 [CognitiveMemory] Shutdown channel closed, terminating.");
            true
        }
        _ => false,
    }
}

#[cfg_attr(not(feature = "vector-memory"), allow(dead_code))]
fn extract_google_key_and_model(
    p: &crate::agent::types::ProviderConfig,
) -> Option<(String, Option<String>)> {
    if !matches!(
        p.protocol,
        crate::agent::types::ModelProvider::Google | crate::agent::types::ModelProvider::Gemini
    ) {
        return None;
    }
    let key = p
        .api_key
        .as_deref()
        .map(str::trim)
        .filter(|k| !k.is_empty())?;
    let model = p.default_config.as_ref().map(|c| c.model_id.clone());
    Some((key.to_string(), model))
}

#[cfg_attr(not(feature = "vector-memory"), allow(dead_code))]
pub(crate) fn resolve_consolidation_target(
    providers: &dashmap::DashMap<String, crate::agent::types::ProviderConfig>,
    privacy_mode: bool,
    env_google_key: Option<&str>,
) -> Option<(String, String)> {
    if privacy_mode {
        return None;
    }

    let direct = providers
        .get("google")
        .or_else(|| providers.get("gemini"))
        .and_then(|p| extract_google_key_and_model(&p));
    let found = direct.or_else(|| {
        providers
            .iter()
            .find_map(|entry| extract_google_key_and_model(entry.value()))
    });

    let (api_key, cfg_model) = match found {
        Some(pair) => pair,
        None => {
            let env_key = env_google_key.map(str::trim).filter(|k| !k.is_empty())?;
            (env_key.to_string(), None)
        }
    };

    let model_id = cfg_model
        .as_deref()
        .map(str::trim)
        .filter(|m| m.starts_with("gemini-"))
        .unwrap_or("gemini-2.0-flash")
        .to_string();

    Some((api_key, model_id))
}

#[cfg(feature = "vector-memory")]
async fn run_consolidation_cycle(app_state: &crate::routes::AppState) {
    tracing::debug!("🧠 [CognitiveMemory] Waking up for episodic consolidation...");

    let privacy_mode = std::env::var("PRIVACY_MODE")
        .map(|v| v.eq_ignore_ascii_case("true") || v == "1")
        .unwrap_or(false);
    let env_google_key = std::env::var("GOOGLE_API_KEY").ok();
    let Some((api_key, model_id)) = resolve_consolidation_target(
        &app_state.registry.providers,
        privacy_mode,
        env_google_key.as_deref(),
    ) else {
        tracing::debug!(
            "🧠 [CognitiveMemory] Consolidation skipped (privacy mode or no Gemini key)."
        );
        return;
    };

    let Ok(missions) =
        crate::agent::mission::get_recent_missions(&app_state.resources.pool, 50).await
    else {
        tracing::warn!("⚠️ [CognitiveMemory] Failed to query recent missions.");
        return;
    };

    let path = std::env::var("MEMORY_STORAGE_PATH")
        .unwrap_or_else(|_| "data/swarm/mission_scopes".to_string());
    let Ok(mem) = crate::agent::memory::VectorMemory::connect(&path, "mission_scopes").await else {
        tracing::warn!(
            "⚠️ [CognitiveMemory] Failed to connect to VectorMemory at '{}'.",
            path
        );
        return;
    };

    let client = &app_state.resources.http_client;
    for mission in missions
        .into_iter()
        .filter(|m| m.status == crate::agent::types::MissionStatus::Completed)
    {
        if let Err(e) = mem
            .summarize_and_archive(&mission.id, client, &api_key, &model_id)
            .await
        {
            tracing::warn!(
                "⚠️ [CognitiveMemory] Failed to consolidate mission {}: {}",
                mission.id,
                e
            );
        }
    }
}

#[cfg(feature = "vector-memory")]
async fn run_pipeline_loop(
    app_state: crate::routes::AppState,
    mut shutdown_rx: tokio::sync::watch::Receiver<bool>,
    interval_secs: u64,
) {
    let mut interval = tokio::time::interval(std::time::Duration::from_secs(interval_secs));
    interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

    loop {
        tokio::select! {
            res = shutdown_rx.changed() => {
                if should_terminate_pipeline(res, &shutdown_rx) { break; }
            }
            _ = interval.tick() => {
                run_consolidation_cycle(&app_state).await;
            }
        }
    }
}

#[async_trait]
impl SystemService for CognitiveMemoryPipelineService {
    fn name(&self) -> &'static str {
        "CognitiveMemoryPipeline"
    }

    async fn start(&self, context: SystemContext) -> Result<(), anyhow::Error> {
        let app_state = context.app_state;
        #[allow(unused_mut)]
        let mut shutdown_rx = context.shutdown_rx;

        let interval_secs = std::env::var("MEMORY_COMPRESSION_INTERVAL_SECS")
            .unwrap_or_else(|_| "300".to_string())
            .parse::<u64>()
            .unwrap_or(300);

        #[cfg(not(feature = "vector-memory"))]
        {
            app_state.resources.set_subsystem_status(
                "CognitiveMemoryPipeline",
                crate::types::SubsystemStatus::NotStarted,
            );
            tracing::info!(
                target: "cognitive_memory",
                "[cognitive_memory] Feature 'vector-memory' disabled; cognitive memory pipeline offline."
            );
            let _ = (shutdown_rx, interval_secs);
        }

        #[cfg(feature = "vector-memory")]
        {
            app_state.resources.set_subsystem_status(
                "CognitiveMemoryPipeline",
                crate::types::SubsystemStatus::Ready,
            );
            tokio::spawn(run_pipeline_loop(app_state, shutdown_rx, interval_secs));
            tracing::info!(
                "🧠 [CognitiveMemory] Pipeline launched (interval: {}s).",
                interval_secs
            );
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::types::{ModelProvider, ProviderConfig};
    use dashmap::DashMap;

    fn make_provider(id: &str, protocol: ModelProvider, key: &str) -> ProviderConfig {
        ProviderConfig {
            id: id.to_string(),
            name: id.to_string(),
            icon: None,
            api_key: Some(key.to_string()),
            base_url: None,
            protocol,
            external_id: None,
            custom_headers: None,
            default_config: None,
            supports_steering_vectors: false,
            audio_model: None,
        }
    }

    #[test]
    fn test_consolidation_never_exfiltrates_openai_or_anthropic_keys_to_gemini() {
        let providers = DashMap::new();
        providers.insert(
            "openai".to_string(),
            make_provider("openai", ModelProvider::Openai, "sk-openai-secret"),
        );
        providers.insert(
            "anthropic".to_string(),
            make_provider("anthropic", ModelProvider::Anthropic, "sk-ant-secret"),
        );

        // Without Google provider/key, must return None rather than leaking OpenAI/Anthropic secrets
        assert_eq!(resolve_consolidation_target(&providers, false, None), None);

        // With Google provider configured, selects Google key and Gemini model — unless PRIVACY_MODE=true
        providers.insert(
            "google".to_string(),
            make_provider("google", ModelProvider::Google, "goog-valid-key"),
        );
        assert_eq!(resolve_consolidation_target(&providers, true, None), None);
        let (key, model) = resolve_consolidation_target(&providers, false, None)
            .expect("expected Google consolidation config");
        assert_eq!(key, "goog-valid-key");
        assert!(
            model.starts_with("gemini-"),
            "expected Gemini model id, got {}",
            model
        );
    }
}

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

#[cfg(feature = "vector-memory")]
async fn run_consolidation_cycle(app_state: &crate::routes::AppState) {
    tracing::debug!("🧠 [CognitiveMemory] Waking up for episodic consolidation...");

    let missions =
        match crate::agent::mission::get_recent_missions(&app_state.resources.pool, 50).await {
            Ok(m) => m,
            Err(e) => {
                tracing::warn!(
                    "⚠️ [CognitiveMemory] Failed to query recent missions: {}",
                    e
                );
                return;
            }
        };

    let providers = &app_state.registry.providers;
    let default_provider = providers
        .get("openai")
        .or_else(|| providers.get("anthropic"));
    let Some((api_key, protocol)) =
        default_provider.and_then(|p| p.api_key.as_ref().map(|k| (k.clone(), p.protocol.clone())))
    else {
        tracing::debug!("🧠 [CognitiveMemory] No LLM provider/key configured; skipping cycle.");
        return;
    };

    let path = std::env::var("MEMORY_STORAGE_PATH")
        .unwrap_or_else(|_| "data/swarm/mission_scopes".to_string());
    let mem = match crate::agent::memory::VectorMemory::connect(&path, "mission_scopes").await {
        Ok(m) => m,
        Err(e) => {
            tracing::warn!(
                "⚠️ [CognitiveMemory] Failed to connect to VectorMemory at '{}': {}",
                path,
                e
            );
            return;
        }
    };

    let model_id = if protocol == crate::agent::types::ModelProvider::Openai {
        "gpt-4o-mini"
    } else {
        "claude-3-haiku-20240307"
    };

    for mission in missions {
        if mission.status != crate::agent::types::MissionStatus::Completed {
            continue;
        }
        let client = &app_state.resources.http_client;
        if let Err(e) = mem
            .summarize_and_archive(&mission.id, client, &api_key, model_id)
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

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
//! - **Witness Tests**: none declared

use crate::startup::{SystemContext, SystemService};
use async_trait::async_trait;

pub struct CognitiveMemoryPipelineService;

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

            tokio::spawn(async move {
                let mut interval =
                    tokio::time::interval(std::time::Duration::from_secs(interval_secs));
                interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

                loop {
                    tokio::select! {
                        res = shutdown_rx.changed() => {
                            match res {
                                Ok(_) if *shutdown_rx.borrow() => {
                                    tracing::info!("🛑 [CognitiveMemory] Pipeline shutting down gracefully.");
                                    break;
                                }
                                Err(_) => {
                                    tracing::info!("🛑 [CognitiveMemory] Shutdown channel closed, terminating.");
                                    break;
                                }
                                _ => {}
                            }
                        }
                        _ = interval.tick() => {
                            tracing::debug!("🧠 [CognitiveMemory] Waking up for episodic consolidation...");

                            // Fetch recent completed missions
                            let missions = match crate::agent::mission::get_recent_missions(&app_state.resources.pool, 50).await {
                                Ok(m) => m,
                                Err(e) => {
                                    tracing::warn!("⚠️ [CognitiveMemory] Failed to query recent missions: {}", e);
                                    continue;
                                }
                            };

                            let default_provider = app_state.registry.providers.get("openai").or_else(|| app_state.registry.providers.get("anthropic"));
                            let (api_key, protocol) = match default_provider.and_then(|p| p.api_key.as_ref().map(|k| (k.clone(), p.protocol.clone()))) {
                                Some((k, proto)) => (k, proto),
                                None => {
                                    tracing::debug!("🧠 [CognitiveMemory] No LLM provider/key configured for background summarization; skipping cycle.");
                                    continue;
                                }
                            };

                            let memory_storage_path = std::env::var("MEMORY_STORAGE_PATH").unwrap_or_else(|_| "data/swarm/mission_scopes".to_string());
                            let mem = match crate::agent::memory::VectorMemory::connect(&memory_storage_path, "mission_scopes").await {
                                Ok(m) => m,
                                Err(e) => {
                                    tracing::warn!("⚠️ [CognitiveMemory] Failed to connect to VectorMemory at '{}': {}", memory_storage_path, e);
                                    continue;
                                }
                            };

                            for mission in missions {
                                if mission.status == crate::agent::types::MissionStatus::Completed {
                                    let model_id = if protocol == crate::agent::types::ModelProvider::Openai {
                                        "gpt-4o-mini"
                                    } else {
                                        "claude-3-haiku-20240307"
                                    };

                                    if let Err(e) = mem.summarize_and_archive(
                                        &mission.id,
                                        &app_state.resources.http_client,
                                        &api_key,
                                        model_id
                                    ).await {
                                        tracing::warn!("⚠️ [CognitiveMemory] Failed to consolidate mission {}: {}", mission.id, e);
                                    }
                                }
                            }
                        }
                    }
                }
            });
            tracing::info!(
                "🧠 [CognitiveMemory] Pipeline launched (interval: {}s).",
                interval_secs
            );
        }

        Ok(())
    }
}

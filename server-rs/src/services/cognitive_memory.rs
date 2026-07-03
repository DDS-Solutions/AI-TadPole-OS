//! @docs ARCHITECTURE:UI-Services
//! 
//! ### AI Assist Note
//! **! @docs ARCHITECTURE:State**
//! This module implements high-fidelity logic for the Sovereign Reality layer.
//! 
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: Runtime logic error, state desynchronization, or resource exhaustion.
//! - **Telemetry Link**: Search `[cognitive_memory]` in tracing logs.

//! @docs ARCHITECTURE:State
//!
//! ### AI Assist Note
//! **Cognitive Memory Pipeline**: A background SystemService that periodically wakes up,
//! finds completed missions, and runs vector semantic consolidation. Replaces synchronous
//! archiving, allowing agents to complete missions instantly while background workers
//! consolidate episodic memories into dense semantic records.

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
        let shutdown_rx = context.shutdown_rx;
        
        let interval_secs = std::env::var("MEMORY_COMPRESSION_INTERVAL_SECS")
            .unwrap_or_else(|_| "300".to_string())
            .parse::<u64>()
            .unwrap_or(300);

        app_state
            .resources
            .set_subsystem_status("CognitiveMemoryPipeline", crate::types::SubsystemStatus::Ready);

        #[cfg(not(feature = "vector-memory"))]
        let _ = (app_state, shutdown_rx, interval_secs);

        #[cfg(feature = "vector-memory")]
        {
            tokio::spawn(async move {
                let mut interval = tokio::time::interval(std::time::Duration::from_secs(interval_secs));
                interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

                loop {
                    tokio::select! {
                        _ = shutdown_rx.changed() => {
                            if *shutdown_rx.borrow() {
                                tracing::info!("🛑 [CognitiveMemory] Pipeline shutting down gracefully.");
                                break;
                            }
                        }
                        _ = interval.tick() => {
                            tracing::debug!("🧠 [CognitiveMemory] Waking up for episodic consolidation...");
                            
                            // Fetch recent completed missions
                            if let Ok(missions) = crate::agent::mission::get_recent_missions(&app_state.resources.pool, 50).await {
                                for mission in missions {
                                    if mission.status == crate::agent::types::MissionStatus::Completed {
                                        // Attempt to summarize and archive. The VectorMemory implementation automatically
                                        // skips missions with < 3 memories (which includes already-archived missions that only have 1 summary).
                                        
                                        // We need an API key to run the summarization model. We try to grab the default one.
                                        let default_provider = app_state.registry.providers.get("openai").or_else(|| app_state.registry.providers.get("anthropic"));
                                        
                                        if let Some(p) = default_provider {
                                            if let Some(api_key) = &p.api_key {
                                                if let Ok(mem) = crate::agent::memory::VectorMemory::connect(&app_state.config.vector_db_path, "mission_scopes").await {
                                                    // Note: We use a default fast model for background summarization
                                                    let model_id = if p.provider == crate::agent::types::ModelProvider::OpenAI {
                                                        "gpt-4o-mini"
                                                    } else {
                                                        "claude-3-haiku-20240307"
                                                    };
                                                    
                                                    if let Err(e) = mem.summarize_and_archive(
                                                        &mission.id,
                                                        &app_state.resources.http_client,
                                                        api_key,
                                                        model_id
                                                    ).await {
                                                        tracing::warn!("⚠️ [CognitiveMemory] Failed to consolidate mission {}: {}", mission.id, e);
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            });
            tracing::info!("🧠 [CognitiveMemory] Pipeline launched (interval: {}s).", interval_secs);
        }

        Ok(())
    }
}

// Metadata: [cognitive_memory]

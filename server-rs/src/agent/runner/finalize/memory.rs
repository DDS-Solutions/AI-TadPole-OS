//! @docs ARCHITECTURE:Runner
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Agent Runner / finalize / memory
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: `[Memory]`

use crate::agent::runner::{AgentRunner, RunContext};

impl AgentRunner {
    /// Saves the final output to the agent's permanent institutional knowledge in LanceDB.
    /// Under Privacy Mode, cloud embeddings are strictly bypassed to prevent data leakage.
    #[cfg(feature = "vector-memory")]
    pub(crate) fn archive_lance_db_memory(&self, ctx: &RunContext, output_text: &str) {
        if self.is_privacy_active(ctx) {
            tracing::info!(
                "🔒 [Privacy Shield] Skipping cloud vector memory embedding for mission {}",
                ctx.mission_state.mission_id
            );
            return;
        }

        let (_, agent_memory_dir, _) = ctx.resolve_paths();
        let mem_output = output_text.to_string();
        let mem_mission_id = ctx.mission_state.mission_id.clone();
        let api_key = ctx.model_config.api_key.clone().unwrap_or_default();
        let http_client = self.state.resources.http_client.clone();

        let dedupe_threshold = std::env::var("LANCEDB_DEDUPE_THRESHOLD")
            .unwrap_or_else(|_| "0.2".to_string())
            .parse::<f32>()
            .unwrap_or(0.2);

        tokio::spawn(async move {
            match crate::agent::memory::VectorMemory::connect(&agent_memory_dir, "memories").await {
                Ok(mem) => {
                    if let Ok(vec) = crate::agent::memory::get_gemini_embedding(
                        &http_client,
                        &api_key,
                        &mem_output,
                    )
                    .await
                    {
                        match mem
                            .check_memory_duplicate(vec.clone(), dedupe_threshold)
                            .await
                        {
                            Ok(true) => {
                                tracing::info!("🧠 [Memory] Duplicate detected (dist < {}), skipping LanceDB insertion for mission {}", dedupe_threshold, mem_mission_id);
                            }
                            _ => {
                                let id = uuid::Uuid::new_v4().to_string();
                                let _ =
                                    mem.add_memory(&id, &mem_output, &mem_mission_id, vec).await;
                                tracing::info!(
                                    "🧠 [Memory] Archived final result to LanceDB for mission {}",
                                    mem_mission_id
                                );
                            }
                        }
                    }
                }
                Err(e) => tracing::error!("❌ [Memory] Failed to archive LanceDB memory: {}", e),
            }
        });
    }
}

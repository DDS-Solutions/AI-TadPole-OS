//! @docs ARCHITECTURE:Runner
//!
//! ### AI Assist Note
//! **Post-Mission Auditor**: Spawns background analysis tasks using **Agent 99 (QA)**
//! to debrief completed missions. Implements **Behavioral Drift Detection**
//! (SEC-06) by measuring semantic distance between agent actions and core identity.
//! Performs **Semantic Pruning** of large logs to optimize auditor prompt windows.
//!
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: lanceDB connection failure during drift analysis,
//!   embedding API timeout, or Agent 99 recruitment failure.
//! - **Trace Scope**: `server-rs::agent::runner::analysis`

use crate::agent::runner::{AgentRunner, RunContext};
use crate::agent::types::TaskPayload;

pub(crate) fn spawn_post_mission_analysis(
    runner: AgentRunner,
    ctx: RunContext,
    output_text: String,
) {
    let pool = runner.state.resources.pool.clone();
    let mission_id = ctx.mission_id.clone();
    let ctx_depth = ctx.depth;
    let ctx_agent_id = ctx.agent_id.clone();
    #[cfg(feature = "vector-memory")]
    let final_output = output_text.clone();
    #[cfg(not(feature = "vector-memory"))]
    let _ = output_text; // Consume to avoid unused param warning if needed, though it's used in outer fn

    let cluster_name = ctx
        .workspace_root
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();
    let traceparent = ctx.traceparent.clone();

    std::thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("Failed to build analysis thread runtime");

        rt.block_on(async move {
            tracing::info!("📡 [Analysis] Triggering post-mission debrief for mission {}...", mission_id);

            #[cfg(feature = "vector-memory")]
            let (agent_memory_dir, mission_scope_dir) = ctx.resolve_paths();

            #[cfg(feature = "vector-memory")]
            let api_key = ctx.model_config.api_key.clone().unwrap_or_default();

            #[cfg(feature = "vector-memory")]
            let identity_context = runner.state.resources.get_identity_context().await;

            #[cfg(feature = "vector-memory")]
            let mut past_patterns = String::new();

            // --- SEMANTIC AUDITING & GOVERNANCE (Behavioral Drift) ---
            #[cfg(feature = "vector-memory")]
            {
                let drift_threshold = std::env::var("LANCEDB_DRIFT_THRESHOLD")
                    .unwrap_or_else(|_| "0.85".to_string())
                    .parse::<f32>()
                    .unwrap_or(0.85);

                if let Ok(action_vec) = crate::agent::memory::get_gemini_embedding(&runner.state.resources.http_client, &api_key, &final_output).await {
                    if let Ok(identity_vec) = crate::agent::memory::get_gemini_embedding(&runner.state.resources.http_client, &api_key, &identity_context).await {
                        let dist: f32 = action_vec.iter().zip(identity_vec.iter())
                            .map(|(a, b)| (a - b).powi(2))
                            .sum::<f32>()
                            .sqrt();
                        if dist > drift_threshold {
                            runner.state.broadcast_sys(&format!("🚨 Behavioral Drift Detected for Agent {}! Actions diverged from core identity (distance: {:.2}).", ctx_agent_id, dist), "error", Some(mission_id.clone()));
                        }
                    }
                }
            }

            let logs = match crate::agent::mission::get_mission_logs(&pool, &mission_id).await {
                Ok(l) => l,
                Err(e) => {
                    tracing::error!("❌ [Analysis] Failed to fetch logs: {}", e);
                    return;
                }
            };

            let mut log_summary = String::new();
            for log in logs {
                log_summary.push_str(&format!("[{} @ {}]: {}\n", log.source, log.timestamp, log.text));
            }
            log_summary = runner.safe_truncate(&log_summary, 8000);

            // --- SEMANTIC PRUNING FOR DEBRIEF CONTEXT ---
            #[cfg(feature = "vector-memory")]
            {
                let log_tokens_est = log_summary.len() / 4;
                if log_tokens_est > 2000 {
                    tracing::info!("✂️ [Analysis] Logs exceed 2000 tokens (est. {}). Engaging semantic pruning.", log_tokens_est);
                    if let Ok(scope_mem) = crate::agent::memory::VectorMemory::connect(&mission_scope_dir, "scope").await {
                        for (i, line) in log_summary.split("\n\n").enumerate() {
                            if line.trim().len() < 10 { continue; }
                            if let Ok(vec) = crate::agent::memory::get_gemini_embedding(&runner.state.resources.http_client, &api_key, line).await {
                                let _ = scope_mem.add_memory(&format!("log-{}", i), line, &mission_id, vec).await;
                            }
                        }
                        let mut pruned_logs = Vec::new();
                        let keywords = ["error", "blocker", "decision", "final"];
                        for kw in keywords {
                            if let Ok(vec) = crate::agent::memory::get_gemini_embedding(&runner.state.resources.http_client, &api_key, kw).await {
                                if let Ok(results) = scope_mem.search_knowledge(vec, 3).await {
                                    for text in results {
                                        if !pruned_logs.contains(&text) {
                                            pruned_logs.push(text);
                                        }
                                    }
                                }
                            }
                        }
                        if !pruned_logs.is_empty() {
                            log_summary = format!("--- SEMANTICALLY PRUNED LOGS ---\n{}", pruned_logs.join("\n\n"));
                        }
                    }
                }
            }

            // --- CROSS-MISSION PATTERN RECOGNITION (Meta-Learning) ---
            #[cfg(feature = "vector-memory")]
            {
                if let Ok(agent_mem) = crate::agent::memory::VectorMemory::connect(&agent_memory_dir, "memories").await {
                    let query_text = format!("Find past errors, blockers, or lessons related to this mission: {}", log_summary.chars().take(500).collect::<String>());
                    if let Ok(vec) = crate::agent::memory::get_gemini_embedding(&runner.state.resources.http_client, &api_key, &query_text).await {
                        if let Ok(results) = agent_mem.search_knowledge(vec, 2).await {
                            for text in results {
                                past_patterns.push_str(&text);
                                past_patterns.push_str("\n---\n");
                            }
                        }
                    }
                }
            }

            #[cfg(feature = "vector-memory")]
            let recent_findings = (!past_patterns.trim().is_empty()).then_some(past_patterns);

            #[cfg(not(feature = "vector-memory"))]
            let recent_findings = None;

            let message = format!(
                "Analyze the following mission logs and determine if the mission was successful. Respond with 'Mission Successful' if the objective was met, otherwise describe the failure.\n\n### MISSION LOGS ###\n{}",
                log_summary
            );

            let payload = TaskPayload {
                message,
                cluster_id: Some(cluster_name.clone()),
                department: Some("Quality Assurance".to_string()),
                provider: None,
                model_id: None,
                api_key: None,
                base_url: None,
                rpm: None,
                tpm: None,
                rpd: None,
                tpd: None,
                budget_usd: None,
                swarm_depth: Some(ctx_depth + 1),
                swarm_lineage: Some(vec![ctx_agent_id.clone()]),
                external_id: None,
                safe_mode: Some(false),
                analysis: Some(false),
                traceparent: traceparent.clone(),
                user_id: None,
                context_files: None,
                recent_findings,
                structured_output: Some(false),
                primary_goal: ctx.primary_goal.clone(),
            };

            // Hardcoded "99" for QA Auditor
            match runner.run("99".to_string(), payload).await {
                Ok(_) => tracing::info!("✅ [Analysis] Mission report generated for ID {}", mission_id),
                Err(e) => tracing::error!("❌ [Analysis] Agent 99 failed: {}", e),
            }
        });
    });
}

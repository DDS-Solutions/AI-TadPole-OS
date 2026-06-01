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
use crate::error::AppError;

const QA_AUDITOR_ID: &str = "99";

/// Spawns a background post-mission analysis task on the existing Tokio executor.
pub(crate) fn spawn_post_mission_analysis(
    runner: AgentRunner,
    ctx: RunContext,
    output_text: String,
) {
    tokio::spawn(async move {
        let auditor = PostMissionAuditor::new(runner, ctx);
        if let Err(e) = auditor.run(output_text).await {
            tracing::error!("❌ [Analysis] Post-mission analysis failed: {}", e);
        }
    });
}

struct PostMissionAuditor {
    runner: AgentRunner,
    ctx: RunContext,
}

impl PostMissionAuditor {
    fn new(runner: AgentRunner, ctx: RunContext) -> Self {
        Self { runner, ctx }
    }

    async fn run(&self, output_text: String) -> Result<(), AppError> {
        tracing::info!(
            "📡 [Analysis] Triggering post-mission debrief for mission {}...",
            self.ctx.mission_id
        );

        // 1. Behavioral Drift Detection
        self.detect_behavioral_drift(&output_text).await;

        // 2. Fetch and Prune Logs
        let log_summary = self.fetch_and_prune_logs().await?;

        // 3. Cross-Mission Pattern Recognition
        let recent_findings = self.recognize_cross_mission_patterns(&log_summary).await;

        // 4. Trigger QA Agent 99 (Auditor)
        self.trigger_qa_agent(log_summary, recent_findings).await?;

        Ok(())
    }

    /// Measures the L2 semantic distance of agent actions from their core identity context.
    #[cfg(feature = "vector-memory")]
    async fn detect_behavioral_drift(&self, output_text: &str) {
        let api_key = self.ctx.model_config.api_key.clone().unwrap_or_default();
        let drift_threshold = std::env::var("LANCEDB_DRIFT_THRESHOLD")
            .unwrap_or_else(|_| "0.85".to_string())
            .parse::<f32>()
            .unwrap_or(0.85);

        if let Ok(action_vec) = crate::agent::memory::get_gemini_embedding(
            &self.runner.state.resources.http_client,
            &api_key,
            output_text,
        )
        .await
        {
            if let Ok(identity_context) = self.runner.state.resources.get_identity_context().await {
                if let Ok(identity_vec) = crate::agent::memory::get_gemini_embedding(
                    &self.runner.state.resources.http_client,
                    &api_key,
                    &identity_context,
                )
                .await
                {
                    let dist: f32 = action_vec
                        .iter()
                        .zip(identity_vec.iter())
                        .map(|(a, b)| (a - b).powi(2))
                        .sum::<f32>()
                        .sqrt();
                    if dist > drift_threshold {
                        self.runner.state.broadcast_sys(
                            &format!(
                                "🚨 Behavioral Drift Detected for Agent {}! Actions diverged from core identity (distance: {:.2}).",
                                self.ctx.agent_id, dist
                            ),
                            "error",
                            Some(self.ctx.mission_id.clone()),
                        );
                    }
                }
            }
        }
    }

    #[cfg(not(feature = "vector-memory"))]
    async fn detect_behavioral_drift(&self, _output_text: &str) {}

    /// Fetches all mission logs and performs semantic pruning if they exceed the context threshold.
    async fn fetch_and_prune_logs(&self) -> Result<String, AppError> {
        let logs = crate::agent::mission::get_mission_logs(
            &self.runner.state.resources.pool,
            &self.ctx.mission_id,
        )
        .await?;

        let mut log_summary = String::new();
        for log in logs {
            log_summary.push_str(&format!(
                "[{} @ {}]: {}\n",
                log.source, log.timestamp, log.text
            ));
        }
        log_summary = self.runner.safe_truncate(&log_summary, 8000);

        #[cfg(feature = "vector-memory")]
        {
            let log_tokens_est = log_summary.len() / 4;
            if log_tokens_est > 2000 {
                tracing::info!(
                    "✂️ [Analysis] Logs exceed 2000 tokens (est. {}). Engaging semantic pruning.",
                    log_tokens_est
                );
                let (_, _, mission_scope_dir) = self.ctx.resolve_paths();
                let api_key = self.ctx.model_config.api_key.clone().unwrap_or_default();

                if let Ok(scope_mem) = crate::agent::memory::VectorMemory::connect(
                    &mission_scope_dir,
                    "scope",
                )
                .await
                {
                    // Robust line splitting and chunk batching (5 lines per chunk) to avoid rate limits
                    let lines: Vec<&str> = log_summary
                        .lines()
                        .filter(|l| !l.trim().is_empty())
                        .collect();
                    let chunks: Vec<String> = lines
                        .chunks(5)
                        .map(|chunk| chunk.join("\n"))
                        .collect();

                    for (i, chunk) in chunks.iter().enumerate() {
                        if chunk.trim().len() < 10 {
                            continue;
                        }
                        if let Ok(vec) = crate::agent::memory::get_gemini_embedding(
                            &self.runner.state.resources.http_client,
                            &api_key,
                            chunk,
                        )
                        .await
                        {
                            let _ = scope_mem
                                .add_memory(
                                    &format!("log-{}", i),
                                    chunk,
                                    &self.ctx.mission_id,
                                    vec,
                                )
                                .await;
                        }
                    }

                    let mut pruned_logs = Vec::new();
                    let keywords = ["error", "blocker", "decision", "final"];
                    for kw in keywords {
                        if let Ok(vec) = crate::agent::memory::get_gemini_embedding(
                            &self.runner.state.resources.http_client,
                            &api_key,
                            kw,
                        )
                        .await
                        {
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
                        log_summary = format!(
                            "--- SEMANTICALLY PRUNED LOGS ---\n{}",
                            pruned_logs.join("\n\n")
                        );
                    }
                }
            }
        }

        Ok(log_summary)
    }

    /// Evaluates cross-mission patterns by querying agent long-term memory for recurring bottlenecks.
    #[cfg(feature = "vector-memory")]
    async fn recognize_cross_mission_patterns(&self, log_summary: &str) -> Option<String> {
        let (agent_memory_dir, _, _) = self.ctx.resolve_paths();
        let api_key = self.ctx.model_config.api_key.clone().unwrap_or_default();

        if let Ok(agent_mem) = crate::agent::memory::VectorMemory::connect(
            &agent_memory_dir,
            "memories",
        )
        .await
        {
            let query_text = format!(
                "Find past errors, blockers, or lessons related to this mission: {}",
                log_summary.chars().take(500).collect::<String>()
            );
            if let Ok(vec) = crate::agent::memory::get_gemini_embedding(
                &self.runner.state.resources.http_client,
                &api_key,
                &query_text,
            )
            .await
            {
                if let Ok(results) = agent_mem.search_knowledge(vec, 2).await {
                    let mut past_patterns = String::new();
                    for text in results {
                        past_patterns.push_str(&text);
                        past_patterns.push_str("\n---\n");
                    }
                    if !past_patterns.trim().is_empty() {
                        return Some(past_patterns);
                    }
                }
            }
        }
        None
    }

    #[cfg(not(feature = "vector-memory"))]
    async fn recognize_cross_mission_patterns(&self, _log_summary: &str) -> Option<String> {
        None
    }

    /// Triggers Agent 99 (QA Auditor) to analyze the logs and output success/failure reports.
    async fn trigger_qa_agent(
        &self,
        log_summary: String,
        recent_findings: Option<String>,
    ) -> Result<(), AppError> {
        let cluster_name = self.ctx
            .workspace_root
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        let message = format!(
            "Analyze the following mission logs and determine if the mission was successful. Respond with 'Mission Successful' if the objective was met, otherwise describe the failure.\n\n### MISSION LOGS ###\n{}",
            log_summary
        );

        let payload = TaskPayload {
            message,
            cluster_id: Some(cluster_name),
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
            swarm_depth: Some(self.ctx.depth + 1),
            swarm_lineage: Some(vec![self.ctx.agent_id.clone()]),
            external_id: None,
            safe_mode: Some(false),
            analysis: Some(false),
            traceparent: self.ctx.traceparent.clone(),
            user_id: None,
            context_files: None,
            recent_findings,
            structured_output: Some(false),
            primary_goal: self.ctx.primary_goal.clone(),
            allowed_files: None,
        };

        match self.runner.run(QA_AUDITOR_ID.to_string(), payload).await {
            Ok(_) => {
                tracing::info!(
                    "✅ [Analysis] Mission report generated for ID {}",
                    self.ctx.mission_id
                );
                Ok(())
            }
            Err(e) => {
                tracing::error!(
                    "❌ [Analysis] QA Auditor (Agent {}) failed: {}",
                    QA_AUDITOR_ID,
                    e
                );
                Err(e)
            }
        }
    }
}

// Metadata: [analysis]

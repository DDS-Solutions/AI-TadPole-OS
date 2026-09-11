//! @docs ARCHITECTURE:Runner
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Agent Runner / analysis
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Type-safe state handling and bounded execution without unhandled panics.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: `[Analysis]`
//! - **Witness Tests**: none declared

use crate::agent::runner::{AgentRunner, RunContext};
use crate::agent::types::TaskPayload;
use crate::error::AppError;

pub const QA_AUDITOR_ID: &str = "99";

/// Computes normalized cosine distance between two embedding vectors.
#[allow(dead_code)]
fn compute_cosine_distance(a: &[f32], b: &[f32]) -> Option<f32> {
    if a.len() != b.len() || a.is_empty() {
        return None;
    }
    let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm_a == 0.0 || norm_b == 0.0 {
        return None;
    }
    let cosine_sim = dot / (norm_a * norm_b);
    Some((1.0 - cosine_sim).max(0.0))
}

/// Spawns a background post-mission analysis task on the existing Tokio executor.
pub(crate) fn spawn_post_mission_analysis(
    runner: AgentRunner,
    ctx: RunContext,
    output_text: String,
) {
    // Recursion Guard: Prevent QA Auditor (Agent 99) from infinitely analyzing its own debrief missions
    if ctx.agent_id == QA_AUDITOR_ID || ctx.lineage.iter().any(|id| id == QA_AUDITOR_ID) {
        tracing::debug!(
            "ℹ️ [Analysis] Skipping post-mission analysis for QA Auditor (Agent {}) to prevent recursive audit loops",
            ctx.agent_id
        );
        return;
    }

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

        // 1. Behavioral Drift Detection (SEC-06)
        self.detect_behavioral_drift(&output_text).await;

        // 2. Fetch and Prune Logs (Prune first, then final truncate)
        let log_summary = self.fetch_and_prune_logs().await?;

        // 3. Cross-Mission Pattern Recognition
        let recent_findings = self.recognize_cross_mission_patterns(&log_summary).await;

        // 4. Trigger QA Agent 99 (Auditor)
        self.trigger_qa_agent(log_summary, recent_findings).await?;

        Ok(())
    }

    /// Measures the normalized cosine distance of agent actions from their core identity context.
    #[cfg(feature = "vector-memory")]
    async fn detect_behavioral_drift(&self, output_text: &str) {
        let api_key = match &self.ctx.model_config.api_key {
            Some(k) if !k.trim().is_empty() => k.clone(),
            _ => {
                tracing::warn!("⚠️ [Analysis] SEC-06 Behavioral Drift check skipped: no API key in model config");
                return;
            }
        };

        let drift_threshold = std::env::var("LANCEDB_DRIFT_THRESHOLD")
            .unwrap_or_else(|_| "0.85".to_string())
            .parse::<f32>()
            .unwrap_or(0.85);

        let output_text_owned = output_text.to_string();
        let http_client = self.runner.state.resources.http_client.clone();

        let action_vec = match crate::agent::memory::get_gemini_embedding(
            &http_client,
            &api_key,
            &output_text_owned,
        )
        .await
        {
            Ok(v) => v,
            Err(e) => {
                tracing::warn!(
                    "⚠️ [Analysis] SEC-06 Behavioral Drift check failed: action embedding error ({})",
                    e
                );
                return;
            }
        };

        let identity_context = match self.runner.state.resources.get_identity_context().await {
            Ok(ctx) => ctx,
            Err(e) => {
                tracing::warn!(
                    "⚠️ [Analysis] SEC-06 Behavioral Drift check failed: identity context load error ({})",
                    e
                );
                return;
            }
        };

        let identity_vec = match crate::agent::memory::get_gemini_embedding(
            &http_client,
            &api_key,
            &identity_context,
        )
        .await
        {
            Ok(v) => v,
            Err(e) => {
                tracing::warn!(
                    "⚠️ [Analysis] SEC-06 Behavioral Drift check failed: identity embedding error ({})",
                    e
                );
                return;
            }
        };

        if let Some(dist) = compute_cosine_distance(&action_vec, &identity_vec) {
            if dist > drift_threshold {
                tracing::warn!(
                    "🚨 [Analysis] Behavioral Drift Detected for Agent {}! Actions diverged from core identity (cosine distance: {:.3}, threshold: {:.3})",
                    self.ctx.agent_id, dist, drift_threshold
                );
                self.runner.state.broadcast_sys(
                    &format!(
                        "🚨 Behavioral Drift Detected for Agent {}! Actions diverged from core identity (cosine distance: {:.2}).",
                        self.ctx.agent_id, dist
                    ),
                    "error",
                    Some(self.ctx.mission_id.clone()),
                );
            }
        } else {
            tracing::warn!("⚠️ [Analysis] SEC-06 Behavioral Drift check failed: vector dimension mismatch or zero magnitude");
        }
    }

    #[cfg(not(feature = "vector-memory"))]
    async fn detect_behavioral_drift(&self, _output_text: &str) {}

    /// Fetches all mission logs and performs semantic pruning before final safeguard truncation.
    async fn fetch_and_prune_logs(&self) -> Result<String, AppError> {
        let logs = crate::agent::mission::get_mission_logs(
            &self.runner.state.resources.pool,
            &self.ctx.mission_id,
        )
        .await?;

        let mut raw_logs = String::new();
        for log in logs {
            raw_logs.push_str(&format!(
                "[{} @ {}]: {}\n",
                log.source, log.timestamp, log.text
            ));
        }

        #[allow(unused_mut)]
        let mut log_summary = raw_logs;

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
                let http_client = self.runner.state.resources.http_client.clone();

                if let Ok(scope_mem) =
                    crate::agent::memory::VectorMemory::connect(&mission_scope_dir, "scope").await
                {
                    // Line splitting and chunk batching
                    let lines: Vec<String> = log_summary
                        .lines()
                        .filter(|l| !l.trim().is_empty())
                        .map(|s| s.to_string())
                        .collect();
                    let chunks: Vec<String> =
                        lines.chunks(5).map(|chunk| chunk.join("\n")).collect();

                    for (i, chunk) in chunks.iter().enumerate() {
                        if chunk.trim().len() < 10 {
                            continue;
                        }
                        if let Ok(vec) = crate::agent::memory::get_gemini_embedding(
                            &http_client,
                            &api_key,
                            chunk,
                        )
                        .await
                        {
                            let chunk_id = format!("{}-log-{}", self.ctx.mission_id, i);
                            let _ = scope_mem
                                .add_memory(&chunk_id, chunk, &self.ctx.mission_id, vec)
                                .await;
                        }
                    }

                    let mut pruned_logs = Vec::new();
                    let keywords = vec![
                        "error".to_string(),
                        "blocker".to_string(),
                        "decision".to_string(),
                        "final".to_string(),
                    ];
                    for kw in keywords {
                        if let Ok(vec) =
                            crate::agent::memory::get_gemini_embedding(&http_client, &api_key, &kw)
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

        // Final safeguard truncation after semantic pruning
        let final_summary = self.runner.safe_truncate(&log_summary, 8000);
        Ok(final_summary)
    }

    /// Evaluates cross-mission patterns by querying agent long-term memory for recurring bottlenecks.
    #[cfg(feature = "vector-memory")]
    async fn recognize_cross_mission_patterns(&self, log_summary: &str) -> Option<String> {
        let (agent_memory_dir, _, _) = self.ctx.resolve_paths();
        let api_key = self.ctx.model_config.api_key.clone().unwrap_or_default();
        let http_client = self.runner.state.resources.http_client.clone();

        if let Ok(agent_mem) =
            crate::agent::memory::VectorMemory::connect(&agent_memory_dir, "memories").await
        {
            let query_text = format!(
                "Find past errors, blockers, or lessons related to this mission: {}",
                log_summary.chars().take(500).collect::<String>()
            );
            if let Ok(vec) =
                crate::agent::memory::get_gemini_embedding(&http_client, &api_key, &query_text)
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
        let cluster_name = self
            .ctx
            .workspace_root
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        let message = format!(
            "Analyze the following mission logs and determine if the mission was successful. Respond with 'Mission Successful' if the objective was met, otherwise describe the failure.\n\n### MISSION LOGS ###\n{}",
            log_summary
        );

        let mut extended_lineage = self.ctx.lineage.clone();
        extended_lineage.push(self.ctx.agent_id.clone());

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
            swarm_lineage: Some(extended_lineage),
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
            ..Default::default()
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_cosine_distance_identical() {
        let v1 = vec![1.0, 2.0, 3.0];
        let v2 = vec![1.0, 2.0, 3.0];
        let dist = compute_cosine_distance(&v1, &v2).unwrap();
        assert!(dist < 1e-5);
    }

    #[test]
    fn test_compute_cosine_distance_orthogonal() {
        let v1 = vec![1.0, 0.0, 0.0];
        let v2 = vec![0.0, 1.0, 0.0];
        let dist = compute_cosine_distance(&v1, &v2).unwrap();
        assert!((dist - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_compute_cosine_distance_mismatched_or_zero() {
        assert!(compute_cosine_distance(&[1.0, 2.0], &[1.0]).is_none());
        assert!(compute_cosine_distance(&[0.0, 0.0], &[1.0, 1.0]).is_none());
        assert!(compute_cosine_distance(&[], &[]).is_none());
    }
}

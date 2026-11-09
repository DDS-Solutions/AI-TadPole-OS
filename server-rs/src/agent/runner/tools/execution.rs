//! @docs ARCHITECTURE:Runner
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Agent Runner / Tool Retry and Audit
//!
//! ### AI Assist Note
//! - Retry transient tool errors and update mutation, cache, usage, and audit state.
//!
//! ### 🔍 Debugging & Observability
//! - **Telemetry Targets**: none declared

use super::{
    error::ToolExecutionError, trait_tool::ToolContext, AgentRunner, Tool, ToolLeaseGuard,
};
use std::sync::Arc;

impl AgentRunner {
    /// Executes the tool with retries on transient errors and registers caching/audit state.
    #[allow(clippy::too_many_arguments)]
    pub(super) async fn execute_with_retry(
        &self,
        call_id: &str,
        tool_instance: Arc<dyn Tool>,
        tool_ctx: &ToolContext,
        fc: &crate::agent::types::ToolCall,
        usage: &mut Option<crate::agent::types::TokenUsage>,
        lease_guard: Option<ToolLeaseGuard>,
        is_mutating: bool,
        is_cacheable: bool,
        target_path: &Option<std::path::PathBuf>,
        args_str: &str,
        workspace_root_str: &str,
        mission_id_opt: &Option<String>,
        agent_id: &str,
        user_id: Option<&str>,
        modified_files: &parking_lot::Mutex<Vec<String>>,
        last_mutation_timestamp_ms: &std::sync::atomic::AtomicU64,
    ) -> Result<String, ToolExecutionError> {
        let mut retry_count = 0;
        // Raised to 5 to give SQLite transient errors (BUSY/LOCKED) sufficient retry budget
        // during high-concurrency swarm spawning. Backoff: 500ms * retry_count = max ~7.5s total.
        let max_retries = 5;

        loop {
            let result = tool_instance
                .execute(tool_ctx, fc.args.clone(), usage)
                .await;

            match result {
                Ok(res) => {
                    if let Some(mut guard) = lease_guard {
                        guard.release();
                    }
                    if is_mutating {
                        if let Some(ref path) = target_path {
                            let now_ms = std::time::SystemTime::now()
                                .duration_since(std::time::UNIX_EPOCH)
                                .map(|d| d.as_millis() as u64)
                                .unwrap_or(0);
                            last_mutation_timestamp_ms
                                .store(now_ms, std::sync::atomic::Ordering::SeqCst);

                            let path_str = path.to_string_lossy().to_string();
                            let mut mod_files = modified_files.lock();
                            if !mod_files.contains(&path_str) {
                                mod_files.push(path_str.clone());
                            }

                            // ── TrustGraph Mutation Hook ───────────────────
                            let tg_pool = self.state.resources.pool.clone();
                            let tg_agent = agent_id.to_string();
                            let tg_path = path_str;
                            let tg_mission = mission_id_opt.clone();
                            tokio::spawn(async move {
                                let engine =
                                    crate::agent::trustgraph::TrustGraphEngine::new(tg_pool);
                                let agent_node = crate::agent::trustgraph::TrustGraphNode {
                                    id: format!("agent:{}", tg_agent),
                                    name: tg_agent.clone(),
                                    r#type: "AGENT".to_string(),
                                    description: Some(format!("Agent {}", tg_agent)),
                                    mission_id: tg_mission.clone(),
                                };
                                if let Err(e) = engine.add_node(&agent_node).await {
                                    tracing::warn!(
                                        "Failed to record agent node in TrustGraph: {}",
                                        e
                                    );
                                    return;
                                }
                                let file_node = crate::agent::trustgraph::TrustGraphNode {
                                    id: format!("file:{}", tg_path),
                                    name: tg_path.clone(),
                                    r#type: "FILE".to_string(),
                                    description: Some(format!("File at {}", tg_path)),
                                    mission_id: tg_mission.clone(),
                                };
                                if let Err(e) = engine.add_node(&file_node).await {
                                    tracing::warn!(
                                        "Failed to record file node in TrustGraph: {}",
                                        e
                                    );
                                    return;
                                }
                                let rel = crate::agent::trustgraph::TrustGraphRelation {
                                    id: format!(
                                        "rel:mutated:{}:{}",
                                        tg_agent,
                                        uuid::Uuid::new_v4()
                                    ),
                                    source_entity_id: agent_node.id,
                                    target_entity_id: file_node.id,
                                    relation_type: "MUTATED".to_string(),
                                    weight: 1.0,
                                    mission_id: tg_mission,
                                };
                                if let Err(e) = engine.add_relation(&rel).await {
                                    tracing::warn!(
                                        "Failed to record MUTATED relation in TrustGraph: {}",
                                        e
                                    );
                                }
                            });
                        }
                    }
                    if is_cacheable
                        && !res.starts_with("(READ FAILED")
                        && !res.starts_with("(FAILED")
                    {
                        let mut cache = self.state.resources.tool_cache.lock();
                        cache.insert(
                            &fc.name,
                            args_str,
                            workspace_root_str,
                            res.clone(),
                            usage.as_ref().cloned(),
                            target_path.clone(),
                        );
                    }
                    // Record successful completion in audit trail with output digest
                    use sha2::Digest;
                    let mut hasher = sha2::Sha256::new();
                    hasher.update(res.as_bytes());
                    let output_digest = hex::encode(hasher.finalize());

                    let success_params = serde_json::json!({
                        "call_id": call_id,
                        "status": "success",
                        "output_digest": output_digest,
                    })
                    .to_string();

                    if let Err(err) = self
                        .state
                        .security
                        .audit_trail
                        .record(
                            agent_id,
                            mission_id_opt.as_deref(),
                            user_id,
                            &format!("[SUCCESS] {}", fc.name),
                            &success_params,
                        )
                        .await
                    {
                        tracing::error!("Failed to record tool success in audit trail: {}", err);
                    }
                    return Ok(res);
                }
                Err(e) if e.is_transient() && retry_count < max_retries => {
                    retry_count += 1;
                    tokio::time::sleep(std::time::Duration::from_millis(500 * retry_count)).await;
                    continue;
                }
                Err(e) => {
                    if let Some(mut guard) = lease_guard {
                        guard.release();
                    }
                    // Record failure in audit trail (redacted with correlation call_id)
                    let redacted_error = self
                        .state
                        .security
                        .secret_redactor
                        .redact(&format!("Error: {}", e));
                    let failure_params = serde_json::json!({
                        "call_id": call_id,
                        "status": "failure",
                        "error": &redacted_error,
                    })
                    .to_string();

                    if let Err(err) = self
                        .state
                        .security
                        .audit_trail
                        .record(
                            agent_id,
                            mission_id_opt.as_deref(),
                            user_id,
                            &format!("[FAILURE] {}", fc.name),
                            &failure_params,
                        )
                        .await
                    {
                        tracing::error!("Failed to record tool failure in audit trail: {}", err);
                    }
                    return Err(e);
                }
            }
        }
    }
}

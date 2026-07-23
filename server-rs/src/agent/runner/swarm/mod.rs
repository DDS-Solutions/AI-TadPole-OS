//! @docs ARCHITECTURE:Runner
//!
//! ### AI Assist Note
//! **Swarm Coordinator**: Manages the recursive recruitment of specialized
//! sub-agents. Implements **Neural Handoff** (injecting parent strategic intent
//! into sub-tasks) and **Self-Healing Registry** (auto-registering missing
//! agents). Enforces **Hierarchy Protocols** (CEO->COO->Alpha) to ensure
//! strategic delegation.
//!
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: Circular recursion detected (SEC-01), max swarm depth
//!   exceeded, or sub-agent recruitment failure.
//! - **Trace Scope**: `server-rs::agent::runner::swarm`

pub(crate) mod recruitment;
pub(crate) mod governance;
pub(crate) mod dispatcher;

use super::{AgentRunner, RunContext};
use crate::agent::runner::tools::error::ToolExecutionError;
use crate::agent::types::{ModelConfig, TokenUsage, ToolCall};
use crate::error::AppError;

pub(crate) struct SubAgentOptions<'a> {
    pub agent_id: &'a str,
    pub parent_config: &'a ModelConfig,
    pub extra_skills: Option<&'a Vec<serde_json::Value>>,
    pub extra_workflows: Option<&'a Vec<serde_json::Value>>,
    pub role_override: Option<&'a str>,
}

impl AgentRunner {
    pub(crate) fn get_default_skills() -> Vec<String> {
        vec![
            "fetch_url".to_string(),
            "read_file".to_string(),
            "write_file".to_string(),
            "list_files".to_string(),
            "delete_file".to_string(),
            "get_file_contents".to_string(),
            "grep_search".to_string(),
            "get_agent_metrics".to_string(),
            "query_financial_logs".to_string(),
            "complete_mission".to_string(),
        ]
    }

    /// Extracts sub-agent IDs from the tool call arguments.
    fn resolve_target_agents(&self, fc: &ToolCall) -> Vec<String> {
        let mut target_ids = Vec::new();
        if let Some(ids) = fc.args.get("agent_ids").and_then(|v| v.as_array()) {
            for id in ids {
                if let Some(s) = id.as_str() {
                    target_ids.push(s.to_string());
                }
            }
        } else {
            let single_id = fc
                .args
                .get("agent_id")
                .and_then(|v| v.as_str())
                .unwrap_or("general");
            target_ids.push(single_id.to_string());
        }
        target_ids
    }

    /// Centralized prompt building for neural handoff strategic context.
    fn build_neural_handoff(&self, sub_message: &str, ctx: &RunContext) -> String {
        let primary_mission = format!(
            "\n\n### PRIMARY MISSION GOAL:\n{}",
            ctx.primary_goal
                .as_deref()
                .unwrap_or("See mission scope for details.")
        );
        if sub_message.len() < 10 {
            format!(
                "{}\n\n(Please assist with the mission goal listed above.)",
                primary_mission
            )
        } else {
            format!(
                "{}\n\n--- STRATEGIC CONTEXT ---\nPrimary Goal: {}\n--- END CONTEXT ---",
                sub_message, primary_mission
            )
        }
    }

    /// Synthesizes pooled sub-agent results.
    async fn synthesize_swarm_results(
        &self,
        results: Vec<String>,
        ctx: &RunContext,
        usage: &mut Option<TokenUsage>,
    ) -> Result<String, ToolExecutionError> {
        if results.is_empty() {
            return Ok(
                "ERROR: No sub-agents were spawned (Protocol Violation or empty IDs).".to_string(),
            );
        }

        let pooled_results = results.join("\n\n---\n\n");
        let synthesis_prompt = format!(
            "Your swarm reported back with these pooled results:\n\n{}\n\nPlease synthesize this data and provide your final response or take next steps.",
            pooled_results
        );

        if results.iter().all(|r| {
            r.contains("PROTOCOL_VIOLATION") || r.contains("FAILURE") || r.contains("ERROR")
        }) {
            tracing::warn!("⚠️ [Swarm] All sub-tasks failed or were blocked. Skipping synthesis.");
            return Ok(pooled_results);
        }

        let swarm_tools = self.build_tools(ctx).await;
        let (final_text, _, final_usage) = self
            .call_provider_for_synthesis(ctx, &synthesis_prompt, Some(vec![swarm_tools]))
            .await?;

        self.accumulate_usage(usage, final_usage);
        if let Some(ref vt) = ctx.visible_transcript {
            vt.lock()
                .push(format!("OBSERVATION (Spawn Sub-agents): {}", final_text));
        }
        Ok(final_text)
    }

    pub(crate) async fn handle_spawn_subagent(
        &self,
        ctx: &RunContext,
        fc: &ToolCall,
        usage: &mut Option<TokenUsage>,
    ) -> Result<String, ToolExecutionError> {
        let target_ids = self.resolve_target_agents(fc);

        let max_swarm_depth = self
            .state
            .governance
            .max_swarm_depth
            .load(std::sync::atomic::Ordering::Relaxed);
        if ctx.depth >= max_swarm_depth {
            tracing::warn!("🐝 [Swarm] Swarm depth limit exceeded (current: {}, max: {}). Blocking recruitment.", ctx.depth, max_swarm_depth);
            return Ok(format!("PROTOCOL_VIOLATION: Swarm depth limit exceeded (current depth: {}). You cannot spawn more sub-agents.", ctx.depth));
        }

        tracing::info!(
            "🐝 [Swarm] Agent {} spawning {} sub-agent(s): {:?}...",
            ctx.agent_id,
            target_ids.len(),
            target_ids
        );

        // Start transaction for the entire pre-flight phase
        let mut tx = self.state.resources.pool
            .begin()
            .await
            .map_err(|e| ToolExecutionError::ExecutionFailed(e.to_string()))?;

        // Pre-flight check & reactivate suspended agents inside transaction
        self.validate_and_reactivate_agents(&mut tx, &target_ids, ctx).await?;

        // Extract extra capabilities and role override if provided
        let extra_skills = fc.args.get("skills").and_then(|v| v.as_array());
        let extra_workflows = fc.args.get("workflows").and_then(|v| v.as_array());
        let role_override = fc.args.get("role").and_then(|v| v.as_str());

        // Ensure sub-agents exist in DB inside the same transaction
        let mut resolved_ids = Vec::new();
        for sub_agent_id in &target_ids {
            // Skip if lineage blocked (handled below)
            if ctx.lineage.contains(sub_agent_id) || ctx.agent_id == *sub_agent_id {
                resolved_ids.push(sub_agent_id.clone());
                continue;
            }

            let resolved_id = self.ensure_sub_agent_exists(
                &mut tx,
                SubAgentOptions {
                    agent_id: sub_agent_id,
                    parent_config: &ctx.model_config,
                    extra_skills,
                    extra_workflows,
                    role_override,
                },
            )
            .await?;
            resolved_ids.push(resolved_id);
        }

        // Commit transaction atomically
        tx.commit()
            .await
            .map_err(|e| ToolExecutionError::ExecutionFailed(e.to_string()))?;

        let sub_message = fc
            .args
            .get("message")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        // Dispatch execution
        let results = self.execute_swarm_dispatch(resolved_ids, sub_message, ctx).await?;

        // Synthesize results
        self.synthesize_swarm_results(results, ctx, usage).await
    }

    /// Handles `issue_alpha_directive`: delegates to Tadpole Alpha (ID: 2).
    pub(crate) async fn handle_alpha_directive(
        &self,
        ctx: &RunContext,
        fc: &ToolCall,
    ) -> Result<String, AppError> {
        let directive = fc
            .args
            .get("directive")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        tracing::info!("🧬 [Sovereignty] Agent of Nine issuing directive to Tadpole Alpha...");
        self.broadcast_agent(ctx, "🧬 Issuing directive to Tadpole Alpha...", "info");

        // 🧠 Proactive Neural Handoff for Alpha Directives
        let primary_mission = format!(
            "\n\n### PRIMARY MISSION GOAL:\n{}",
            ctx.primary_goal
                .as_deref()
                .unwrap_or("See mission scope for details.")
        );
        let final_directive = format!("{}\n\n{}", directive, primary_mission);

        let payload = ctx.derive_subtask_payload(final_directive);

        let sub_result = Box::pin(self.run("2".to_string(), payload)).await?;

        if let Some(ref vt) = ctx.visible_transcript {
            vt.lock()
                .push(format!("OBSERVATION (Alpha Directive): {}", sub_result));
        }

        Ok(format!(
            "Directive issued to Tadpole Alpha. Mission ID: {}\n\nResult: {}",
            ctx.mission_id, sub_result
        ))
    }
}

// ─────────────────────────────────────────────────────────
//  UNIT TESTS
// ─────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::runner::RunContext;
    use crate::agent::types::{ModelConfig, ToolCall};
    use crate::state::AppState;
    use std::sync::Arc;

    async fn setup_test_runner() -> (AgentRunner, Arc<AppState>) {
        let state = Arc::new(AppState::new_minimal_mock().await);
        let runner = AgentRunner::new(state.clone());
        (runner, state)
    }

    #[tokio::test]
    async fn test_proactive_recursion_block_parent() {
        let (runner, _) = setup_test_runner().await;
        let mut ctx = RunContext {
            agent_id: "weather_api".to_string(),
            lineage: vec![
                "2".to_string(),
                "alpha".to_string(),
                "weather_agent".to_string(),
            ],
            ..RunContext::default()
        };
        // Ensure some fields are ready
        ctx.model_config = ModelConfig::default();

        let mut output = String::new();
        let mut usage = None;
        let fc = ToolCall {
            name: "spawn_subagent".to_string(),
            args: serde_json::json!({
                "agent_id": "weather_agent",
                "message": "try again"
            }) as serde_json::Value,
        };

        let result = runner.handle_spawn_subagent(&ctx, &fc, &mut usage).await;

        if let Ok(res) = &result {
            output.push_str(res);
        }

        assert!(result.is_ok());
        assert!(output.contains("PROTOCOL_VIOLATION: CIRCULAR_RECRUITMENT"));
        assert!(output.contains("'weather_agent' is already in your recruitment lineage"));
    }

    #[tokio::test]
    async fn test_proactive_recursion_block_self() {
        let (runner, _) = setup_test_runner().await;
        let mut ctx = RunContext {
            agent_id: "weather_api".to_string(),
            lineage: vec!["2".to_string(), "alpha".to_string()],
            ..RunContext::default()
        };
        ctx.model_config = ModelConfig::default();

        let mut output = String::new();
        let mut usage = None;
        let fc = ToolCall {
            name: "spawn_subagent".to_string(),
            args: serde_json::json!({
                "agent_id": "weather_api",
                "message": "talk to myself"
            }) as serde_json::Value,
        };

        let result = runner.handle_spawn_subagent(&ctx, &fc, &mut usage).await;

        if let Ok(res) = &result {
            output.push_str(res);
        }

        assert!(output.contains("PROTOCOL_VIOLATION: CIRCULAR_RECRUITMENT"));
        assert!(output.contains("'weather_api' is already in your recruitment lineage"));
    }

    #[tokio::test]
    async fn test_recruitment_tier1_specialist() {
        let (runner, state) = setup_test_runner().await;

        // Setup Tier 1 Specialist (User category)
        let tier1_agent = crate::agent::types::EngineAgent {
            identity: crate::agent::types::AgentIdentity {
                id: "specialist_analyst".to_string(),
                name: "Expert Analyst".to_string(),
                role: "Analyst".to_string(),
                category: "user".to_string(),
                ..Default::default()
            },
            ..crate::agent::types::EngineAgent::default()
        };
        state
            .registry
            .agents
            .insert(tier1_agent.identity.id.clone(), tier1_agent);

        let parent_config = ModelConfig::default();
        let mut tx = state.resources.pool.begin().await.unwrap();
        runner
            .ensure_sub_agent_exists(
                &mut tx,
                SubAgentOptions {
                    agent_id: "analyst",
                    parent_config: &parent_config,
                    extra_skills: None,
                    extra_workflows: None,
                    role_override: None,
                },
            )
            .await
            .unwrap();
        tx.commit().await.unwrap();

        let analyst = state.registry.agents.get("specialist_analyst").unwrap();
        assert_eq!(
            analyst.metadata.get("has_participated_in_swarm").unwrap(),
            &serde_json::Value::Bool(true)
        );
        assert_eq!(analyst.health.status, "active");
    }

    #[tokio::test]
    async fn test_recruitment_tier2_swarm_pool() {
        let (runner, state) = setup_test_runner().await;

        // Setup Tier 2 Brain (AI category)
        let tier2_agent = crate::agent::types::EngineAgent {
            identity: crate::agent::types::AgentIdentity {
                id: "previous_brain".to_string(),
                name: "Experienced Coder".to_string(),
                role: "Coder".to_string(),
                category: "ai".to_string(),
                ..Default::default()
            },
            ..crate::agent::types::EngineAgent::default()
        };
        state
            .registry
            .agents
            .insert(tier2_agent.identity.id.clone(), tier2_agent);

        let parent_config = ModelConfig::default();
        let mut tx = state.resources.pool.begin().await.unwrap();
        runner
            .ensure_sub_agent_exists(
                &mut tx,
                SubAgentOptions {
                    agent_id: "coder",
                    parent_config: &parent_config,
                    extra_skills: None,
                    extra_workflows: None,
                    role_override: None,
                },
            )
            .await
            .unwrap();
        tx.commit().await.unwrap();

        let coder = state.registry.agents.get("previous_brain").unwrap();
        assert_eq!(
            coder.metadata.get("has_participated_in_swarm").unwrap(),
            &serde_json::Value::Bool(true)
        );
    }

    #[tokio::test]
    async fn test_recruitment_tier3_fabrication() {
        let (runner, state) = setup_test_runner().await;
        let parent_config = ModelConfig::default();
        let mut tx = state.resources.pool.begin().await.unwrap();
        runner
            .ensure_sub_agent_exists(
                &mut tx,
                SubAgentOptions {
                    agent_id: "researcher",
                    parent_config: &parent_config,
                    extra_skills: None,
                    extra_workflows: None,
                    role_override: None,
                },
            )
            .await
            .unwrap();
        tx.commit().await.unwrap();

        assert!(state.registry.agents.contains_key("researcher"));
        let researcher = state.registry.agents.get("researcher").unwrap();
        assert_eq!(researcher.identity.category, "ai");
    }

    #[tokio::test]
    async fn test_parallel_swarm_recruitment_logic() {
        let (runner, _state) = setup_test_runner().await;

        let ctx = RunContext {
            agent_id: "orchestrator".to_string(),
            mission_id: "test-parallel".to_string(),
            ..RunContext::default()
        };

        let fc = ToolCall {
            name: "spawn_subagent".to_string(),
            args: serde_json::json!({
                "agent_ids": ["researcher_1", "researcher_2"],
                "message": "Verify this in parallel."
            }),
        };

        let mut output = String::new();
        let mut usage = None;

        let result = runner.handle_spawn_subagent(&ctx, &fc, &mut usage).await;
        if let Ok(res) = &result {
            output.push_str(res);
        }

        // ASSERT: The agents should have been recruited/registered in state
        assert!(
            runner.state.registry.agents.contains_key("researcher_1"),
            "researcher_1 should be in registry"
        );
        assert!(
            runner.state.registry.agents.contains_key("researcher_2"),
            "researcher_2 should be in registry"
        );
    }
}

// Metadata: [swarm]

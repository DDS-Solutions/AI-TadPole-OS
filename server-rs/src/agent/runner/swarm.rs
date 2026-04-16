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

use super::{AgentRunner, RunContext};
use crate::agent::types::{ModelConfig, TokenUsage};

impl AgentRunner {
    // ─────────────────────────────────────────────────────────
    //  SWARM MANAGEMENT
    // ─────────────────────────────────────────────────────────

    /// Handles the `spawn_subagent` tool call: ensures sub-agent exists, recurses, synthesizes.
    pub(crate) async fn handle_spawn_subagent(
        &self,
        ctx: &RunContext,
        fc: &crate::agent::types::GeminiFunctionCall,
        output_text: &mut String,
        usage: &mut Option<TokenUsage>,
    ) -> anyhow::Result<()> {
        let sub_agent_id = fc
            .args
            .get("agent_id")
            .and_then(|v| v.as_str())
            .unwrap_or("general");
        let sub_message = fc
            .args
            .get("message")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        tracing::info!(
            "🐝 [Swarm] Agent {} spawning sub-agent {}...",
            ctx.agent_id,
            sub_agent_id
        );
        self.broadcast_agent(
            ctx,
            &format!("🐝 Handing off to sub-agent {}...", sub_agent_id),
            "info",
        );

        // Extract extra capabilities and role override if provided
        let extra_skills = fc.args.get("skills").and_then(|v| v.as_array());
        let extra_workflows = fc.args.get("workflows").and_then(|v| v.as_array());
        let extra_mcp = fc.args.get("mcp_tools").and_then(|v| v.as_array());
        let role_override = fc.args.get("role").and_then(|v| v.as_str());

        // 🛡️ [Harden Phase 4: Proactive Lineage Guard]
        // Prevent circular recursion before attempting recruitment.
        if ctx.lineage.contains(&sub_agent_id.to_string()) || ctx.agent_id == sub_agent_id {
            let violation_msg = format!(
                "PROTOCOL_VIOLATION: CIRCULAR_RECRUITMENT. Agent ID '{}' is already in your recruitment lineage ({:?}). \
                 You are FORBIDDEN from recruiting this agent. You MUST solve the task yourself or recruit a DIFFERENT specialist.",
                sub_agent_id, ctx.lineage
            );
            *output_text = violation_msg;
            return Ok(());
        }

        // Ensure sub-agent exists in persistence
        let sub_err = match self
            .ensure_sub_agent_exists(
                sub_agent_id,
                &ctx.model_config,
                extra_skills,
                extra_workflows,
                extra_mcp,
                role_override,
            )
            .await
        {
            Ok(_) => None,
            Err(e) => Some(format!("RECRUITMENT FAILURE: {}", e)),
        };

        if let Some(err_msg) = sub_err {
            let advisory = format!(
                "System Advisory: Recruitment of '{}' failed. Reason: {}\n\
                 You MUST resolve this sub-task yourself or recruit a DIFFERENT specialized ID that is not already in the lineage.",
                sub_agent_id, err_msg
            );
            *output_text = advisory;
            return Ok(());
        }

        // Neural Handoff: Inject parent's current strategic intent/thoughts into sub-message
        let strategic_intent = if !output_text.is_empty() {
            format!(
                "\n\n--- PARENT STRATEGIC INTENT ---\n{}\n--- END INTENT ---",
                output_text
            )
        } else {
            "".to_string()
        };

        // 🧠 Proactive Neural Handoff: Ensure sub-agents always receive the primary goal
        let primary_mission = format!(
            "\n\n### PRIMARY MISSION GOAL:\n{}",
            ctx.primary_goal
                .as_deref()
                .unwrap_or("See mission scope for details.")
        );

        let final_instruction = if sub_message.len() < 10 {
            format!("{}\n\n(No specific sub-instruction provided. Please assist with the mission goal listed above.)", primary_mission)
        } else {
            format!(
                "{}\n\n--- STRATEGIC CONTEXT ---\nPrimary Goal: {}\nParent Notes: {}\n--- END CONTEXT ---",
                sub_message, primary_mission, strategic_intent
            )
        };

        let payload = ctx.derive_subtask_payload(final_instruction);

        let sub_result = match Box::pin(self.run(sub_agent_id.to_string(), payload)).await {
            Ok(res) => res,
            Err(e) => format!("SUB-AGENT EXECUTION ERROR: {}", e),
        };

        // Feed sub-result back for synthesis
        let synthesis_prompt = format!(
            "Sub-agent {} reported back with this result:\n\n{}\n\nPlease synthesize this and provide your final response or take next steps.",
            sub_agent_id, sub_result
        );

        // 🛡️ [Harden Phase 4: Tactical PIVOT]
        // Providing tools to synthesis allows the agent to recover from sub-agent
        // failures (like recursion blocks) by picking a different strategy.
        let swarm_tools = self.build_tools(ctx).await;
        let (final_text, _, final_usage) = self
            .call_provider_for_synthesis(ctx, &synthesis_prompt, Some(vec![swarm_tools]))
            .await?;

        *output_text = final_text;
        self.accumulate_usage(usage, final_usage);

        Ok(())
    }

    /// Ensures a sub-agent exists in the state and database.
    pub(crate) async fn ensure_sub_agent_exists(
        &self,
        sub_agent_id: &str,
        parent_config: &ModelConfig,
        extra_skills: Option<&Vec<serde_json::Value>>,
        extra_workflows: Option<&Vec<serde_json::Value>>,
        _extra_mcp: Option<&Vec<serde_json::Value>>,
        role_override: Option<&str>,
    ) -> anyhow::Result<()> {
        // Register any new unique capabilities into AI Services registry
        if let Some(skills) = extra_skills {
            for s in skills {
                if let Ok(name) = self
                    .state
                    .registry
                    .skills
                    .register_capability("skill", s.clone(), "ai")
                    .await
                {
                    tracing::info!("Registered discovered skill: {}", name);
                }
            }
        }
        if let Some(workflows) = extra_workflows {
            for w in workflows {
                if let Ok(name) = self
                    .state
                    .registry
                    .skills
                    .register_capability("workflow", w.clone(), "ai")
                    .await
                {
                    tracing::info!("Registered discovered workflow: {}", name);
                }
            }
        }
        // Future: Implement MCP registration in McpHost if needed

        // MANAGER PRIORITY: Tiered Recruitment Search
        // 1. Tier 1: User Sector Specialists (Highest Priority)
        // 2. Tier 2: Existing AI Swarm Brains (Experienced Pool)
        // 3. Tier 3: Spawn New (Fallback)
        let mut target_id = sub_agent_id.to_string();
        let mut target_config = parent_config.clone();

        let registry_match = self
            .state
            .registry
            .agents
            .iter()
            .find(|kv| {
                let a = kv.value();
                a.category == "user"
                    && (a.name.eq_ignore_ascii_case(sub_agent_id)
                        || a.role.to_lowercase().contains(&sub_agent_id.to_lowercase()))
            })
            .map(|kv| kv.value().clone())
            .or_else(|| {
                // Tier 2: Search existing AI Swarm Pool
                self.state
                    .registry
                    .agents
                    .iter()
                    .find(|kv| {
                        let a = kv.value();
                        a.category == "ai"
                            && (a.name.eq_ignore_ascii_case(sub_agent_id)
                                || a.role.to_lowercase().contains(&sub_agent_id.to_lowercase()))
                    })
                    .map(|kv| kv.value().clone())
            });

        if let Some(mut matched_agent) = registry_match {
            tracing::info!("🎯 [Swarm] Priority Match: Found existing specialist '{}' ({}) in category '{}' to fulfill request for '{}'", 
                matched_agent.name, matched_agent.id, matched_agent.category, sub_agent_id);

            target_id = matched_agent.id.clone();

            // Persistent Swarm Tagging
            matched_agent.metadata.insert(
                "has_participated_in_swarm".to_string(),
                serde_json::Value::Bool(true),
            );
            matched_agent.status = "active".to_string(); // Force active status for UI visibility

            // Sync changes to Registry and DB
            self.state
                .registry
                .agents
                .insert(target_id.clone(), matched_agent.clone());
            let _ = crate::agent::persistence::save_agent_db(
                &self.state.resources.pool,
                &matched_agent,
            )
            .await
            .map_err(|e| {
                tracing::warn!(
                    "⚠️ [Swarm] Failed to persist recruited agent {} to DB: {}",
                    target_id,
                    e
                )
            });

            // If the matched agent has a specific model config, use it.
            if let Some(mid) = &matched_agent.model_id {
                target_config.model_id = mid.clone();
                target_config.provider = matched_agent.model.provider;
            }
        }

        if self.state.registry.agents.contains_key(&target_id) {
            return Ok(());
        }

        tracing::info!("🛠️ [Swarm] Registering missing sub-agent: {}", target_id);
        let mut base_skills = vec![
            "fetch_url".to_string(),
            "read_file".to_string(),
            "write_file".to_string(),
            "list_files".to_string(),
            "delete_file".to_string(),
            "get_file_contents".to_string(),
            "grep_search".to_string(),
            "get_agent_metrics".to_string(),
            "query_financial_logs".to_string(),
        ];

        // ... discovered skills logic ...
        if let Some(skills) = extra_skills {
            for s in skills {
                if let Some(name) = s.get("name").and_then(|v| v.as_str()) {
                    if !base_skills.contains(&name.to_string()) {
                        base_skills.push(name.to_string());
                    }
                }
            }
        }

        let (initial_role, initial_dept, initial_desc) = if let Some(r) = role_override {
            (
                r.to_string(),
                "Tactical Operations".to_string(),
                format!("Specialized agent with role override: {}", r),
            )
        } else {
            match target_id.to_lowercase().as_str() {
                "researcher" | "searcher" => ("Swarm Research Specialist".to_string(), "Intelligence".to_string(), "Expert in web discovery, data extraction, and information synthesis.".to_string()),
                "coder" | "developer" => ("Swarm Code Specialist".to_string(), "Engineering".to_string(), "Expert in Rust, TypeScript, and system architecture.".to_string()),
                "auditor" | "compliance" => ("Swarm Security Auditor".to_string(), "Compliance".to_string(), "Expert in vulnerability scanning, budget enforcement, and protocol verification.".to_string()),
                "alpha" => ("Swarm Mission Commander".to_string(), "Operations".to_string(), "The Alpha Node, responsible for coordinating multi-agent missions.".to_string()),
                _ => (format!("AI-{}", "General Intelligence Node"), "Swarm Core".to_string(), "Autonomous sub-agent spawned for specific task resolution.".to_string())
            }
        };

        let sub_agent = crate::agent::types::EngineAgent {
            id: target_id.clone(),
            name: target_id.clone(),
            role: initial_role,
            department: initial_dept,
            description: initial_desc,
            category: "ai".to_string(),
            model_id: Some(target_config.model_id.clone()),
            tokens_used: 0,
            status: "idle".to_string(),
            theme_color: Some("#4fd1c5".to_string()),
            budget_usd: 10.0,
            cost_usd: 0.0,
            skills: base_skills,
            skill_manifest: None,
            model_2: None,
            model_3: None,
            model_config2: None,
            model_config3: None,
            active_model_slot: None,
            token_usage: TokenUsage::default(),
            model: crate::agent::types::ModelConfig {
                provider: target_config.provider,

                model_id: parent_config.model_id.clone(),
                api_key: None,
                base_url: parent_config.base_url.clone(),
                system_prompt: None,
                temperature: None,
                max_tokens: None,
                external_id: None,
                rpm: parent_config.rpm,
                rpd: parent_config.rpd,
                tpm: parent_config.tpm,
                tpd: parent_config.tpd,
                skills: None,
                workflows: None,
                mcp_tools: None,
                connector_configs: None,
                extra_parameters: None,
            },
            active_mission: None,
            current_task: None,
            voice_id: None,
            voice_engine: None,
            failure_count: 0,
            last_failure_at: None,
            ..Default::default()
        };

        let _ = crate::agent::persistence::save_agent_db(&self.state.resources.pool, &sub_agent)
            .await
            .map_err(|e| {
                tracing::warn!(
                    "⚠️ [Swarm] Failed to persist sub-agent {} to DB: {}",
                    sub_agent_id,
                    e
                )
            });
        self.state
            .registry
            .agents
            .insert(sub_agent_id.to_string(), sub_agent);

        Ok(())
    }

    /// Handles `issue_alpha_directive`: delegates to Tadpole Alpha (ID: 2).
    pub(crate) async fn handle_alpha_directive(
        &self,
        ctx: &RunContext,
        fc: &crate::agent::types::GeminiFunctionCall,
    ) -> anyhow::Result<String> {
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
    use crate::agent::types::{GeminiFunctionCall, ModelConfig};
    use crate::state::AppState;
    use std::sync::Arc;

    async fn setup_test_runner() -> (AgentRunner, Arc<AppState>) {
        let state = Arc::new(AppState::new_mock().await);
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
        let fc = GeminiFunctionCall {
            name: "spawn_subagent".to_string(),
            args: serde_json::json!({
                "agent_id": "weather_agent",
                "message": "try again"
            }) as serde_json::Value,
        };

        let result = runner
            .handle_spawn_subagent(&ctx, &fc, &mut output, &mut usage)
            .await;

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
        let fc = GeminiFunctionCall {
            name: "spawn_subagent".to_string(),
            args: serde_json::json!({
                "agent_id": "weather_api",
                "message": "talk to myself"
            }) as serde_json::Value,
        };

        let _ = runner
            .handle_spawn_subagent(&ctx, &fc, &mut output, &mut usage)
            .await;

        assert!(output.contains("PROTOCOL_VIOLATION: CIRCULAR_RECRUITMENT"));
        assert!(output.contains("'weather_api' is already in your recruitment lineage"));
    }

    #[tokio::test]
    async fn test_tiered_recruitment_priority() {
        let (runner, state) = setup_test_runner().await;

        // Setup Tier 1 Specialist (User category)
        let tier1_agent = crate::agent::types::EngineAgent {
            id: "specialist_analyst".to_string(),
            name: "Expert Analyst".to_string(),
            role: "Analyst".to_string(),
            category: "user".to_string(),
            ..crate::agent::types::EngineAgent::default()
        };
        state
            .registry
            .agents
            .insert(tier1_agent.id.clone(), tier1_agent);

        // Setup Tier 2 Brain (AI category) - Same role but lower priority
        let tier2_agent = crate::agent::types::EngineAgent {
            id: "previous_brain".to_string(),
            name: "Experienced Coder".to_string(),
            role: "Coder".to_string(),
            category: "ai".to_string(),
            ..crate::agent::types::EngineAgent::default()
        };
        state
            .registry
            .agents
            .insert(tier2_agent.id.clone(), tier2_agent);

        let parent_config = ModelConfig::default();

        // 1. Test recruitment of Tier 1 (User Analyst)
        // Search by role "analyst"
        runner
            .ensure_sub_agent_exists("analyst", &parent_config, None, None, None, None)
            .await
            .unwrap();
        let analyst = state.registry.agents.get("specialist_analyst").unwrap();
        assert_eq!(
            analyst.metadata.get("has_participated_in_swarm").unwrap(),
            &serde_json::Value::Bool(true)
        );
        assert_eq!(analyst.status, "active");

        // 2. Test recruitment of Tier 2 (AI Coder)
        // If no user specialist exists but an AI one does, it should use the AI one.
        runner
            .ensure_sub_agent_exists("coder", &parent_config, None, None, None, None)
            .await
            .unwrap();
        let coder = state.registry.agents.get("previous_brain").unwrap();
        assert_eq!(
            coder.metadata.get("has_participated_in_swarm").unwrap(),
            &serde_json::Value::Bool(true)
        );

        // 3. Test Tier 3 (New Spawn)
        // If neither exists, it creates one.
        runner
            .ensure_sub_agent_exists("researcher", &parent_config, None, None, None, None)
            .await
            .unwrap();
        assert!(state.registry.agents.contains_key("researcher"));
        let researcher = state.registry.agents.get("researcher").unwrap();
        assert_eq!(researcher.category, "ai");
    }
}

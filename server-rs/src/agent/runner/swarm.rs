//! Swarm Intelligence — Multi-agent coordination and mission decomposition
//!
//! @docs ARCHITECTURE:Agent

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
            .get("agentId")
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
        self.broadcast_sys(
            &format!(
                "🐝 Agent {}: Handing off to sub-agent {}...",
                ctx.agent_id, sub_agent_id
            ),
            "info",
            Some(ctx.mission_id.clone()),
        );

        // Extract extra capabilities if provided
        let extra_skills = fc.args.get("skills").and_then(|v| v.as_array());
        let extra_workflows = fc.args.get("workflows").and_then(|v| v.as_array());
        let extra_mcp = fc.args.get("mcpTools").and_then(|v| v.as_array());

        // Ensure sub-agent exists in persistence
        self.ensure_sub_agent_exists(
            sub_agent_id,
            &ctx.model_config,
            extra_skills,
            extra_workflows,
            extra_mcp,
        )
        .await?;

        // Neural Handoff: Inject parent's current strategic intent/thoughts into sub-message
        let strategic_intent = if !output_text.is_empty() {
            format!(
                "\n\n--- PARENT STRATEGIC INTENT ---\n{}\n--- END INTENT ---",
                output_text
            )
        } else {
            "".to_string()
        };

        let payload = ctx.derive_subtask_payload(format!("{}{}", sub_message, strategic_intent));

        let sub_result = Box::pin(self.run(sub_agent_id.to_string(), payload)).await?;

        // Feed sub-result back for synthesis
        let synthesis_prompt = format!(
            "Sub-agent {} reported back with this result:\n\n{}\n\nPlease synthesize this and provide your final response.",
            sub_agent_id, sub_result
        );

        let (final_text, _, final_usage) = self
            .call_provider_for_synthesis(ctx, &synthesis_prompt)
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

        if self.state.registry.agents.contains_key(sub_agent_id) {
            return Ok(());
        }

        tracing::info!("🛠️ [Swarm] Registering missing sub-agent: {}", sub_agent_id);
        let mut base_skills = vec![
            "fetch_url".to_string(),
            "read_file".to_string(),
            "write_file".to_string(),
            "list_files".to_string(),
            "delete_file".to_string(),
            "filesystem".to_string(),
        ];

        // Add discovered skills to the agent's initial set if they were just registered
        if let Some(skills) = extra_skills {
            for s in skills {
                if let Some(name) = s.get("name").and_then(|v| v.as_str()) {
                    if !base_skills.contains(&name.to_string()) {
                        base_skills.push(name.to_string());
                    }
                }
            }
        }

        let sub_agent = crate::agent::types::EngineAgent {
            id: sub_agent_id.to_string(),
            name: sub_agent_id.to_string(),
            role: format!("AI-{}", "General Intelligence Node"),
            department: "Swarm Core".to_string(),
            description: "Autonomous sub-agent spawned for specific task resolution.".to_string(),
            category: "ai".to_string(),
            model_id: Some(parent_config.model_id.clone()),
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
                provider: parent_config.provider.clone(),
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

        crate::agent::persistence::save_agent_db(&self.state.resources.pool, &sub_agent).await?;
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
        self.broadcast_sys(
            "🧬 Agent of Nine: Handing off to Tadpole Alpha...",
            "info",
            Some(ctx.mission_id.clone()),
        );

        let payload = ctx.derive_subtask_payload(directive.to_string());

        let sub_result = Box::pin(self.run("2".to_string(), payload)).await?;

        Ok(format!(
            "Directive issued to Tadpole Alpha. Mission ID: {}\n\nResult: {}",
            ctx.mission_id, sub_result
        ))
    }
}

pub mod encoder;
pub mod pruner;
pub mod fragments;
pub mod toolbelt;
pub mod context;

use std::collections::HashMap;
use crate::agent::runner::RunContext;
use crate::state::AppState;
use crate::agent::constants::*;
use crate::agent::runner::synthesis::encoder::aaak_encode;
use crate::agent::runner::synthesis::fragments::*;

pub struct PromptAssemblerContext<'a> {
    pub directives: &'a str,
    pub reviews: &'a str,
    pub identity: &'a str,
    pub memory: &'a str,
    pub repo_map: &'a str,
    pub swarm_context: &'a str,
    pub criticality_score: i32,
    pub failure_context: Vec<String>,
}

pub fn assemble_prompt_variables(
    ctx: &RunContext,
    state: &AppState,
    p_ctx: PromptAssemblerContext<'_>,
) -> HashMap<&'static str, String> {
    let hierarchy_label = get_hierarchy_label(ctx.authority_level);
    let compressed_swarm_context = aaak_encode(p_ctx.swarm_context);

    let priority_str = if ctx.agent_id == AGENT_CEO {
        "STRATEGIC OVERLORD: You are the final authority. Your priority is mission synthesis and high-level routing."
    } else if ctx.agent_id == AGENT_ALPHA {
        "ALPHA MISSION COMMANDER: Your priority is tactical execution and specialist coordination."
    } else {
        "TACTICAL SPECIALIST: Your priority is focused task completion."
    };

    let compressed_findings = aaak_encode(
        ctx.recent_findings
            .as_deref()
            .unwrap_or("No recent findings inherited."),
    );
    let swarm_str = generate_swarm_protocols(ctx, state);

    let skill_fragments_str = generate_skill_fragments(ctx, state);
    let workflow_fragments_str = generate_workflow_fragments(ctx, state);

    let (
        cluster_directory,
        filesystem_bias_mandate,
        lineage_display,
        _is_orchestrator,
        safe_mode_prefix,
        tool_mode_prefix,
    ) = generate_structural_components(ctx, state);

    let mut vars = HashMap::new();
    vars.insert("name", ctx.name.clone());
    vars.insert("agent_id", ctx.agent_id.clone());
    vars.insert("role", ctx.role.clone());
    vars.insert("department", ctx.department.clone());
    vars.insert("description", ctx.description.clone());
    vars.insert("hierarchy_label", hierarchy_label.to_string());
    vars.insert("directives", p_ctx.directives.to_string());
    vars.insert("reviews", p_ctx.reviews.to_string());
    vars.insert("global_intelligence", "N/A (RAG-Enabled Swarm)".to_string());
    vars.insert("priority", priority_str.to_string());
    vars.insert(
        "personality",
        ctx.model_config
            .system_prompt
            .as_deref()
            .unwrap_or("No specific personality instructions.")
            .to_string(),
    );
    vars.insert("skill_fragments", skill_fragments_str.to_string());
    vars.insert("workflow_fragments", workflow_fragments_str.to_string());
    vars.insert("swarm_context", compressed_swarm_context.to_string());
    vars.insert(
        "breadcrumbs",
        generate_breadcrumbs_display(ctx).to_string(),
    );
    vars.insert("findings", compressed_findings.to_string());
    vars.insert(
        "primary_goal",
        ctx.primary_goal
            .clone()
            .unwrap_or_else(|| "See mission scope for details.".to_string()),
    );
    vars.insert("cluster_directory", cluster_directory.to_string());
    vars.insert("lineage", lineage_display.to_string());
    vars.insert("skills", format!("{:?}", ctx.skills));
    vars.insert("workflows", format!("{:?}", ctx.workflows));
    vars.insert("filesystem_bias", filesystem_bias_mandate.to_string());
    vars.insert("swarm_protocols", swarm_str.to_string());
    vars.insert("repo_map", p_ctx.repo_map.to_string());
    vars.insert("identity", p_ctx.identity.to_string());
    vars.insert("memory", p_ctx.memory.to_string());
    vars.insert(
        "working_memory",
        serde_json::to_string_pretty(&ctx.working_memory).unwrap_or_else(|_| "{}".to_string()),
    );
    vars.insert(
        "history",
        aaak_encode(
            ctx.summarized_history
                .as_deref()
                .unwrap_or("No history summarized yet."),
        )
        .to_string(),
    );
    vars.insert("safe_mode_prefix", safe_mode_prefix.clone());
    vars.insert("tool_mode_prefix", tool_mode_prefix.to_string());

    if p_ctx.criticality_score > 0 {
        let warning = format!("\n\n!!! SYSTEM WARNING: Critical services failed (Score: {}/3) !!!\n- {}\n!!! ACTION: Acknowledge these failures in your internal reasoning and attempt to recover missing context manually if possible. !!!\n", p_ctx.criticality_score, p_ctx.failure_context.join("\n- "));
        vars.insert(
            "safe_mode_prefix",
            format!("{}{}", safe_mode_prefix, warning),
        );
    }

    vars
}

impl super::AgentRunner {
    /// 📟 [AAAK Encoder]
    #[allow(dead_code)]
    pub fn aaak_encode(&self, text: &str) -> String {
        self::encoder::aaak_encode(text)
    }

    /// Constructs the final system prompt for an agent run.
    pub async fn build_system_prompt(&self, ctx: &RunContext, payload_message: &str) -> String {
        let (directives_str, reviews_str) = self::context::fetch_external_context(ctx, &self.state).await;

        let _global_intel_str = match self::context::gather_global_intelligence(ctx, &self.state, payload_message).await {
            Ok(intel) => intel,
            Err(e) => {
                tracing::warn!("⚠️ [Global Intelligence] Vault search failed: {}.", e);
                "GLOBAL_INTEL_FAILURE: Knowledge vault was unreachable.".to_string()
            }
        };

        let mut criticality_score = 0;
        let mut failure_context = Vec::new();

        let identity = match self::context::fetch_identity(&self.state).await {
            Ok(id) => id,
            Err(e) => {
                criticality_score += 1;
                failure_context.push(format!("IDENTITY_FAILURE: Core identity directives could not be loaded: {}.", e));
                tracing::error!("🚨 [Runner] Identity lookup failed: {}.", e);
                String::new()
            }
        };

        let memory = match self::context::fetch_memory(&self.state).await {
            Ok(mem) => mem,
            Err(e) => {
                criticality_score += 1;
                failure_context.push(format!("MEMORY_FAILURE: Long-term memory context could not be retrieved: {}.", e));
                tracing::error!("🚨 [Runner] Memory lookup failed: {}.", e);
                String::new()
            }
        };

        let swarm_context_str = match self::context::fetch_mission_context(ctx, &self.state).await {
            Ok(context) => context,
            Err(e) => {
                criticality_score += 1;
                failure_context.push(format!("SWARM_CONTEXT_FAILURE: Failed to retrieve mission context: {:?}.", e));
                tracing::error!("🚨 [Runner] Failed to retrieve mission context for {}: {:?}.", ctx.mission_id, e);
                String::from("⚠️ WARNING: Failed to load shared mission context.")
            }
        };

        let (_repo_summary, repo_map_text) = self.generate_repo_map(ctx, payload_message).await;

        let (pruned_repo_map, pruned_swarm_context_str) =
            self::pruner::prune_context(ctx, &identity, &memory, &repo_map_text, &swarm_context_str);

        let vars = assemble_prompt_variables(
            ctx,
            &self.state,
            PromptAssemblerContext {
                directives: &directives_str,
                reviews: &reviews_str,
                identity: &identity,
                memory: &memory,
                repo_map: &pruned_repo_map,
                swarm_context: &pruned_swarm_context_str,
                criticality_score,
                failure_context,
            },
        );

        let system_prompt = self.state.resources.renderer.render(
            self.state.resources.renderer.default_system_template(),
            &vars,
        );

        tracing::info!(
            "🏁 [Runner] Synthesizing system prompt for mission: {} (Depth: {})",
            ctx.mission_id,
            ctx.depth
        );
        system_prompt
    }

    /// Generates the Repo Map summary.
    pub async fn generate_repo_map(
        &self,
        ctx: &RunContext,
        payload_message: &str,
    ) -> (crate::utils::graph::GraphSummary, String) {
        let graph = self.state.resources.get_code_graph().await;
        let summary = graph.read().generate_summary();

        // Only include repo map for non-trivial tasks or technical roles to save tokens
        if payload_message.len() > 10 || ctx.department.contains("Technical") {
            (summary.clone(), summary.text)
        } else {
            (
                crate::utils::graph::GraphSummary {
                    text: "Architecture Map: Omitted for high-level greeting.".to_string(),
                    hot_paths: vec![],
                },
                "Architecture Map: Omitted for high-level greeting.".to_string(),
            )
        }
    }

    /// Manifests the agent's toolbelt.
    pub async fn build_tools(&self, ctx: &RunContext) -> crate::agent::types::ToolDefinition {
        self::toolbelt::build_tools(ctx, &self.state).await
    }
}

fn generate_breadcrumbs_display(ctx: &RunContext) -> String {
    let breadcrumbs = ctx.last_accessed_files.lock();
    if breadcrumbs.is_empty() {
        "None available.".to_string()
    } else {
        breadcrumbs.join("\n- ")
    }
}

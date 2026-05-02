//! @docs ARCHITECTURE:Runner
//!
//! ### AI Assist Note
//! **Modular Prompt Synthesizer**: Assembles the agent's "consciousness" through a
//! multi-stage modular pipeline. Links identity, memory, codebase maps, and
//! shared swarm findings.
//!
//! ### 🛡️ Hardened Protocols
//! - **Context Pruning** (PERF-07): Precise TPM-aware truncation using `tiktoken`.
//! - **Hierarchy Enforcement** (SEC-06): Specialized access control (e.g., blocking
//!   Agent 1 from direct filesystem tools to enforce delegation).
//! - **Identity Masking**: Specialists are anonymized in cluster directories for
//!   high-level managers (OVERLORD/ORCHESTRATOR) to prioritize strategic delegation.
//! - **Safe Mode** (SEC-05): Disables all mutation tools based on mission metadata.
//!
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: Tokenizer failure, mission context retrieval fault (match fallback),
//!   or identity lookup cache miss.
//! - **Trace Scope**: `server-rs::agent::runner::synthesis`
//!

// Modules/Crates
use std::collections::HashMap;
use std::num::NonZeroUsize;

// External Crates
use lru::LruCache;
use once_cell::sync::Lazy;
use parking_lot::Mutex;
use tiktoken_rs::cl100k_base;

// Internal Modules
use super::{AgentRunner, RunContext};
use crate::agent::constants::*;
use crate::agent::types::{FunctionDeclaration, ToolDefinition};

/// Context for prompt synthesis to group multiple arguments.
pub struct PromptSynthesisContext<'a> {
    pub directives: &'a str,
    pub reviews: &'a str,
    pub identity: &'a str,
    pub memory: &'a str,
    pub repo_map: &'a str,
    pub swarm_context: &'a str,
    pub criticality_score: i32,
    pub failure_context: Vec<String>,
}

impl AgentRunner {
    /// 📟 [AAAK Encoder]
    /// Compresses mission context using the MemPalace-inspired AAAK dialect
    /// to increase context fidelity while reducing token load.
    fn aaak_encode(&self, text: &str) -> String {
        let mut replacements = HashMap::new();
        replacements.insert("STATUS: ok", "*ok*");
        replacements.insert("STATUS: success", "*ok*");
        replacements.insert("STATUS: failed", "*err*");
        replacements.insert("STATUS: error", "*err*");
        replacements.insert("RESULT:", "RES:");
        replacements.insert("FINDING:", "FND:");
        replacements.insert("SOURCE:", "SRC:");
        replacements.insert("Location:", "LOC:");
        replacements.insert("Primary Goal:", "GOAL:");
        replacements.insert("Weather for zip", "WTR|");
        replacements.insert("degrees", "deg");
        replacements.insert("temperature", "temp");
        replacements.insert("Finding for mission", "FND|");
        replacements.insert("Strategic Intent:", "INT:");
        replacements.insert("Mission Complete", "*done*");
        replacements.insert("Task in progress", "*busy*");

        let mut encoded = text.to_string();
        for (pattern, replacement) in replacements {
            encoded = encoded.replace(pattern, replacement);
        }
        encoded
    }

    // ─────────────────────────────────────────────────────────

    //  SYSTEM PROMPT CONSTRUCTION (MODULARIZED)
    // ─────────────────────────────────────────────────────────

    /// Constructs the final system prompt for an agent run.
    ///
    /// Synthesizes identity, long-term memory, repo-map, and shared swarm
    /// findings into a single prompt, emphasizing modularity and robust error handling.
    pub async fn build_system_prompt(&self, ctx: &RunContext, payload_message: &str) -> String {
        let (directives_str, reviews_str) = self.fetch_external_context(ctx).await;
        
        let _global_intel_str = match self.gather_global_intelligence(ctx, payload_message).await {
            Ok(intel) => intel,
            Err(e) => {
                tracing::warn!("⚠️ [Global Intelligence] Vault search failed: {}.", e);
                "GLOBAL_INTEL_FAILURE: Knowledge vault was unreachable.".to_string()
            }
        };

        let mut criticality_score = 0;
        let mut failure_context = Vec::new();

        let identity = match self.fetch_identity().await {
            Ok(id) => id,
            Err(e) => {
                criticality_score += 1;
                failure_context.push(format!("IDENTITY_FAILURE: Core identity directives could not be loaded: {}.", e));
                tracing::error!("🚨 [Runner] Identity lookup failed: {}.", e);
                String::new()
            }
        };

        let memory = match self.fetch_memory().await {
            Ok(mem) => mem,
            Err(e) => {
                criticality_score += 1;
                failure_context.push(format!("MEMORY_FAILURE: Long-term memory context could not be retrieved: {}.", e));
                tracing::error!("🚨 [Runner] Memory lookup failed: {}.", e);
                String::new()
            }
        };

        let swarm_context_str = match self.fetch_mission_context(ctx).await {
            Ok(context) => context,
            Err(e) => {
                criticality_score += 1;
                failure_context.push(format!("SWARM_CONTEXT_FAILURE: Failed to retrieve mission context: {:?}.", e));
                tracing::error!("🚨 [Runner] Failed to retrieve mission context for {}: {:?}.", ctx.mission_id, e);
                String::from("⚠️ WARNING: Failed to load shared mission context.")
            }
        };

        let (_repo_summary, repo_map_text) = self.generate_repo_map(ctx, payload_message).await;

        let (pruned_repo_map, pruned_swarm_context_str) = self.prune_context(
            ctx,
            &identity,
            &memory,
            &repo_map_text,
            &swarm_context_str,
        );

        let context = PromptSynthesisContext {
            directives: &directives_str,
            reviews: &reviews_str,
            identity: &identity,
            memory: &memory,
            repo_map: &pruned_repo_map,
            swarm_context: &pruned_swarm_context_str,
            criticality_score,
            failure_context,
        };

        let vars = self.assemble_prompt_variables(ctx, context);

        let system_prompt = self.state.resources.renderer.render(self.state.resources.renderer.default_system_template(), &vars);

        tracing::info!(
            "🏁 [Runner] Synthesizing system prompt for mission: {} (Depth: {})",
            ctx.mission_id,
            ctx.depth
        );
        system_prompt
    }

    // =============================================================
    // PRIVATE HELPER METHODS FOR MODULARITY
    // =============================================================

    async fn fetch_external_context(&self, ctx: &RunContext) -> (String, String) {
        let pending_directives = super::swarm_persistence::get_pending_directives(&self.state.resources.pool, &ctx.agent_id)
            .await
            .unwrap_or_default();
        let pending_reviews = super::swarm_persistence::get_pending_reviews(&self.state.resources.pool, &ctx.agent_id)
            .await
            .unwrap_or_default();

        let directives_str = if pending_directives.is_empty() {
            "No active directives. Proceed with mission objectives.".to_string()
        } else {
            pending_directives
                .iter()
                .map(|d| format!("- [Directive] From {}: {}", d.source_agent_id, d.instruction))
                .collect::<Vec<_>>()
                .join("\n")
        };

        let reviews_str = if pending_reviews.is_empty() {
            "No peer reviews pending. Maintain standard quality protocols.".to_string()
        } else {
            pending_reviews
                .iter()
                .map(|r| format!("- [Review Task] Target: {}. Requirement: {}", r.requester_id, r.content_to_review))
                .collect::<Vec<_>>()
                .join("\n")
        };

        (directives_str, reviews_str)
    }

    async fn fetch_identity(&self) -> Result<String, crate::error::AppError> {
        let id = self.state.resources.get_identity_context().await?;
        Ok(id)
    }

    async fn fetch_memory(&self) -> Result<String, crate::error::AppError> {
        let mem = self.state.resources.get_memory_context().await?;
        Ok(mem)
    }

    async fn fetch_mission_context(&self, ctx: &RunContext) -> Result<String, crate::error::AppError> {
        let context = crate::agent::mission::get_mission_context(&self.state.resources.pool, &ctx.mission_id).await?;
        Ok(context)
    }

    fn assemble_prompt_variables(
        &self,
        ctx: &RunContext,
        context: PromptSynthesisContext<'_>,
    ) -> HashMap<&'static str, String> {
        let hierarchy_label = self.get_hierarchy_label(ctx.authority_level);
        let compressed_swarm_context = self.aaak_encode(context.swarm_context);
        
        let priority_str = if ctx.agent_id == AGENT_CEO {
            "STRATEGIC OVERLORD: You are the final authority. Your priority is mission synthesis and high-level routing."
        } else if ctx.agent_id == AGENT_ALPHA {
            "ALPHA MISSION COMMANDER: Your priority is tactical execution and specialist coordination."
        } else {
            "TACTICAL SPECIALIST: Your priority is focused task completion."
        };

        let compressed_findings = self.aaak_encode(ctx.recent_findings.as_deref().unwrap_or("No recent findings inherited."));
        let swarm_str = self.generate_swarm_protocols(ctx);

        let skill_fragments_str = self.generate_skill_fragments(ctx);
        let workflow_fragments_str = self.generate_workflow_fragments(ctx);

        let (
            cluster_directory,
            filesystem_bias_mandate,
            lineage_display,
            _is_orchestrator,
            safe_mode_prefix,
            tool_mode_prefix,
        ) = self.generate_structural_components(ctx);

        let mut vars = HashMap::new();
        vars.insert("name", ctx.name.clone());
        vars.insert("agent_id", ctx.agent_id.clone());
        vars.insert("role", ctx.role.clone());
        vars.insert("department", ctx.department.clone());
        vars.insert("description", ctx.description.clone());
        vars.insert("hierarchy_label", hierarchy_label.to_string());
        vars.insert("directives", context.directives.to_string());
        vars.insert("reviews", context.reviews.to_string());
        vars.insert("global_intelligence", "N/A (RAG-Enabled Swarm)".to_string());
        vars.insert("priority", priority_str.to_string());
        vars.insert("personality", ctx.model_config.system_prompt.as_deref().unwrap_or("No specific personality instructions.").to_string());
        vars.insert("skill_fragments", skill_fragments_str.to_string());
        vars.insert("workflow_fragments", workflow_fragments_str.to_string());
        vars.insert("swarm_context", compressed_swarm_context.to_string());
        vars.insert("breadcrumbs", self.generate_breadcrumbs_display(ctx).to_string());
        vars.insert("findings", compressed_findings.to_string());
        vars.insert("primary_goal", ctx.primary_goal.clone().unwrap_or_else(|| "See mission scope for details.".to_string()));
        vars.insert("cluster_directory", cluster_directory.to_string());
        vars.insert("lineage", lineage_display.to_string());
        vars.insert("skills", format!("{:?}", ctx.skills));
        vars.insert("workflows", format!("{:?}", ctx.workflows));
        vars.insert("filesystem_bias", filesystem_bias_mandate.to_string());
        vars.insert("swarm_protocols", swarm_str.to_string());
        vars.insert("repo_map", context.repo_map.to_string());
        vars.insert("identity", context.identity.to_string());
        vars.insert("memory", context.memory.to_string());
        vars.insert("working_memory", serde_json::to_string_pretty(&ctx.working_memory).unwrap_or_else(|_| "{}".to_string()));
        vars.insert("history", self.aaak_encode(ctx.summarized_history.as_deref().unwrap_or("No history summarized yet.")).to_string());
        vars.insert("safe_mode_prefix", safe_mode_prefix.clone());
        vars.insert("tool_mode_prefix", tool_mode_prefix.to_string());

        if context.criticality_score > 0 {
            let warning = format!("\n\n!!! SYSTEM WARNING: Critical services failed (Score: {}/3) !!!\n- {}\n!!! ACTION: Acknowledge these failures in your internal reasoning and attempt to recover missing context manually if possible. !!!\n", context.criticality_score, context.failure_context.join("\n- "));
            vars.insert("safe_mode_prefix", format!("{}{}", safe_mode_prefix, warning));
        }

        vars
    }

    /// Generates the Repo Map summary, applying pruning logic if needed.
    /// Generates a real-time repository map for the agent's context.
    /// This includes a structural overview of the codebase and a list of
    /// recently accessed files to provide localized context for modifications.
    async fn generate_repo_map(&self, ctx: &RunContext, payload_message: &str) -> (crate::utils::graph::GraphSummary, String) {
        let graph: std::sync::Arc<parking_lot::RwLock<crate::utils::graph::CodeGraph>> =
            self.state.resources.get_code_graph().await;

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

    /// Handles context pruning based on token count (TPM Protection).
    /// Prunes the assembled context to fit within the model's TPM (Tokens Per Minute) limit.
    /// Prioritizes identity and recent history, truncating the repo map and
    /// shared findings if necessary to ensure prompt validity.
    /// Handles semantic context pruning based on weights.
    /// Prioritizes high-weight resources (Identity, Mission) and prunes
    /// low-weight ones (Repo Map, Memory) when approaching TPM limits.
    fn prune_context(
        &self,
        ctx: &RunContext,
        identity: &str,
        memory: &str,
        repo_map: &str,
        swarm_context_str: &str,
    ) -> (String, String) {
        let bpe = match cl100k_base() {
            Ok(b) => b,
            Err(e) => {
                tracing::error!("🚨 [Pruning] Failed to initialize tokenizer: {:?}. Skipping pruning.", e);
                return (repo_map.to_string(), swarm_context_str.to_string());
            }
        };
        let tpm_limit = ctx.model_config.tpm.unwrap_or(100_000);
        let safe_limit = (tpm_limit as f32 * 0.85) as usize;

        let mut pruned_repo_map = repo_map.to_string();
        let mut pruned_swarm_context = swarm_context_str.to_string();

        let get_total_tokens = |repo: &str, swarm: &str| {
            // ASSUMPTION: For rough calculation, we assume the combined string length is roughly proportional to token count.
            let combined = format!("{}{}{}{}", identity, memory, repo, swarm);
            bpe.encode_with_special_tokens(&combined).len()
        };

        let mut current_tokens = get_total_tokens(&pruned_repo_map, &pruned_swarm_context);

        if current_tokens > safe_limit {
            tracing::warn!("⚠️ [Pruning] Context size {} exceeds safe limit {}. Applying semantic weights.", current_tokens, safe_limit);

            // 1. Prune Repo Map (Weight: 0.8)
            if current_tokens > safe_limit {
                let weight = ctx.resource_weights.get("repo_map").cloned().unwrap_or(0.8);
                if weight < 1.0 {
                    pruned_repo_map = "⚠️ Repo Map pruned due to context limits. Use 'list_files' for discovery.".to_string();
                    current_tokens = get_total_tokens(&pruned_repo_map, &pruned_swarm_context);
                }
            }

            // 2. Prune Swarm Context (Weight: 1.0)
            if current_tokens > safe_limit {
                let target_len = (pruned_swarm_context.len() as f32 * 0.5) as usize;
                if target_len > 500 {
                    pruned_swarm_context = self.safe_truncate(&pruned_swarm_context, target_len);
                    current_tokens = get_total_tokens(&pruned_repo_map, &pruned_swarm_context);
                }
            }

            // 3. Emergency Truncation
            if current_tokens > tpm_limit as usize {
                tracing::error!("🚨 [Pruning] EMERGENCY: Context still exceeds TPM limit ({}). Hard truncating swarm context.", current_tokens);
                pruned_swarm_context = self.safe_truncate(&pruned_swarm_context, 500);
            }
        }

        (pruned_repo_map, pruned_swarm_context)
    }

    pub(crate) fn get_hierarchy_label(&self, level: crate::agent::types::RoleAuthorityLevel) -> &str {
        match level {
            crate::agent::types::RoleAuthorityLevel::Executive => "OVERLORD (Tier 0)",
            crate::agent::types::RoleAuthorityLevel::Management => "ORCHESTRATOR (Tier 1)",
            crate::agent::types::RoleAuthorityLevel::Specialist => "SPECIALIST (Tier 2)",
            crate::agent::types::RoleAuthorityLevel::Observer => "AUDITOR (Tier 3)",
        }
    }

    /// Collects the necessary meta-info (protocols, etc.) for the prompt.
    /// Compiles the structural components of the prompt, including identity,
    /// long-term memory, working context, and the current repository map.
    fn generate_structural_components(
        &self,
        ctx: &RunContext,
    ) -> (
        String, // Cluster Directory
        String, // File System Bias Mandate
        String, // Lineage Display
        bool,   // Is Orchestrator
        String, // Safe Mode Prefix
        String, // Tool Mode Prefix
    ) {
        let cluster_specialists = self.state.registry.list_active_specialists();

        // --- Cluster Directory ---
        let cluster_directory = if cluster_specialists.is_empty() {
            "No specialized agents currently registered in the cluster pool. You are encouraged to recruit a NEW specialist ID (e.g. 'researcher', 'coder') if needed.".to_string()
        } else {
            let is_orchestrator =
                ctx.agent_id == AGENT_CEO || ctx.agent_id == AGENT_COO || ctx.agent_id == AGENT_ALPHA;

            cluster_specialists
                .into_iter()
                .map(|id| {
                    if is_orchestrator {
                        id
                    } else {
                        match id.as_str() {
                            AGENT_CEO => "[SYSTEM_OVERLORD]".to_string(),
                            AGENT_COO => "[SYSTEM_ORCHESTRATOR]".to_string(),
                            AGENT_ALPHA => "[SYSTEM_COMMANDER]".to_string(),
                            other => other.to_string(),
                        }
                    }
                })
                .collect::<Vec<_>>()
                .join("\n")
        };

        // --- Mission Bias ---
        let filesystem_bias_mandate = if self.has_file_system_capability(ctx) {
            "DATA_ACCESS: PERMITTED. You represent a specialist with localized I/O authority."
                .to_string()
        } else {
            "DATA_ACCESS: RESTRICTED. Direct filesystem mutation is disabled. Use swarm recruitment for I/O.".to_string()
        };

        // --- Lineage Display ---
        let is_orchestrator = ctx.agent_id == AGENT_CEO || ctx.agent_id == AGENT_COO || ctx.agent_id == AGENT_ALPHA;
        let lineage_display = if ctx.lineage.is_empty() {
            "None (You are the root node)".to_string()
        } else {
            let path_parts: Vec<String> = ctx
                .lineage
                .iter()
                .map(|id| {
                    if is_orchestrator {
                        id.clone()
                    } else {
                        match id.as_str() {
                            AGENT_CEO => "OVERLORD".to_string(),
                            AGENT_COO => "ORCHESTRATOR".to_string(),
                            AGENT_ALPHA => "COMMANDER".to_string(),
                            _ => id.clone(),
                        }
                    }
                })
                .collect();
            path_parts.join(" -> ")
        };

        // --- Prefixes ---
        let safe_mode_prefix = if ctx.safe_mode {
            "🔒 TOOLS FORBIDDEN: You are in CONVERSATIONAL MODE. Do NOT attempt to call any tools. \
             Respond directly with natural language. You are chatting with the Overlord (user). \
             Be helpful, insightful, and conversational. No tool calls, no mission completion — just talk.\n".to_string()
        } else {
            "🔓 SECURITY: ACTIVE. You have authority to propose and execute mutations within your workspace scope.\n".to_string()
        };

        let tool_mode_prefix = if ctx.analysis {
            "🛠️ TOOL EVOLUTION: You are in ANALYSIS mode. Propose architectural changes."
                .to_string()
        } else {
            "⚙️ TOOL REFINEMENT: Standard operation.".to_string()
        };

        (
            cluster_directory,
            filesystem_bias_mandate,
            lineage_display,
            is_orchestrator,
            safe_mode_prefix,
            tool_mode_prefix,
        )
    }

    /// Generates the shared Swarm Protocol rules using the ACL Service.
    fn generate_swarm_protocols(&self, ctx: &RunContext) -> String {
        let mut swarm_protocols = self.state.resources.acl.get_role_protocols(&ctx.agent_id, &ctx.role, ctx.authority_level);

        // Add shared/invariant swarm rules
        swarm_protocols.push("NO NARRATION: Do not explain strategy or tool options. CALL THE TOOLS. Text-only responses are MISSION FAILURE.".to_string());
        swarm_protocols.push("COMPLETION: Complete ALL steps autonomously before reporting back.".to_string());
        swarm_protocols.push("OVERSIGHT TRUST: Never ask for permission. The system handles approval flows automatically.".to_string());
        swarm_protocols.push("SOURCE OF TRUTH: Prioritize codebase tools (read_codebase_file) over general RAG for code queries.".to_string());
        swarm_protocols.push(format!("RECURSION PROTECTION: You are FORBIDDEN from recruiting agents in your LINEAGE ({:?}) or YOURSELF ({}).", &ctx.lineage, ctx.agent_id));

        swarm_protocols
            .iter()
            .enumerate()
            .map(|(i, p)| format!("{}. {}", i + 1, p))
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Generates the File Path Breadcrumbs context.
    fn generate_breadcrumbs_display(&self, ctx: &RunContext) -> String {
        let breadcrumbs = ctx.last_accessed_files.lock();
        if breadcrumbs.is_empty() {
            "None available.".to_string()
        } else {
            breadcrumbs.join("\n- ")
        }
    }

    /// Generates instructions for skills.
    fn generate_skill_fragments(&self, ctx: &RunContext) -> String {
        let mut skill_fragments = Vec::new();
        for skill_name in &ctx.skills {
            if let Some(skill_entry) = self.state.registry.skills.skills.get(skill_name) {
                let skill = skill_entry.value();
                if let Some(instructions) = &skill.full_instructions {
                    skill_fragments.push(format!(
                        "### [{}] Full Instructions:\n{}",
                        skill.name, instructions
                    ));
                }
                if let Some(constraints) = &skill.negative_constraints {
                    if !constraints.is_empty() {
                        skill_fragments.push(format!(
                            "### [{}] Negative Constraints (PROHIBITED USES):\n- {}",
                            skill.name,
                            constraints.join("\n- ")
                        ));
                    }
                }
            }
        }
        if skill_fragments.is_empty() {
            String::new()
        } else {
            format!(
                "\nSKILL-SPECIFIC DEEP INSTRUCTIONS:\n{}\n",
                skill_fragments.join("\n\n")
            )
        }
    }

    /// Generates instructions for workflows.
    fn generate_workflow_fragments(&self, ctx: &RunContext) -> String {
        let mut workflow_fragments = Vec::new();
        for workflow_name in &ctx.workflows {
            if let Some(wf_entry) = self.state.registry.skills.workflows.get(workflow_name) {
                let wf = wf_entry.value();
                workflow_fragments.push(format!(
                    "### [{}] Workflow Procedure:\n{}",
                    wf.name, wf.content
                ));
            }
        }
        if workflow_fragments.is_empty() {
            String::new()
        } else {
            format!(
                "\nWORKFLOW-SPECIFIC PROCEDURES:\n{}\n",
                workflow_fragments.join("\n\n")
            )
        }
    }

    /// 🧠 [AAAK Intelligence Loop] (Phase 4)
    /// Performs a semantic search across the global swarm intelligence vault
    /// to retrieve relevant findings from past missions.
    async fn gather_global_intelligence(&self, ctx: &RunContext, query: &str) -> Result<String, crate::error::AppError> {
        tracing::info!("🧠 [Global Intelligence] Agent {} querying global vault for: {}", ctx.agent_id, query);
        
        #[cfg(feature = "vector-memory")]
        {
            let api_key = ctx.model_config.api_key.clone().unwrap_or_else(|| {
                self.state.registry.providers.get(&ctx.model_config.provider.to_string())
                    .and_then(|p| p.api_key.clone())
                    .unwrap_or_default()
            });
            let http_client = self.state.resources.http_client.clone();

            let vec = crate::agent::memory::get_gemini_embedding(&http_client, &api_key, query).await?;
            let vault = self.state.resources.get_swarm_vault().await?;
            let results = vault.search_knowledge(vec, 3).await?;
            
            if !results.is_empty() {
                return Ok(format!("Relevant global intelligence retrieved:\n- {}", results.join("\n- ")));
            }
        }

        Ok("No relevant global intelligence found in the vault.".to_string())
    }
    /// Dynamically manifests tool definitions for the LLM provider.
    ///
    /// Maps enabled skills and MCP tools into Gemini-compatible functions,
    /// leveraging modular injectors for specialized capabilities.
    /// Manifests the agent's toolbelt, enforcing hierarchy-based access control.
    /// Injects core, filesystem, and advanced tools while programmatically
    /// blocking restricted tools for specific identities (e.g., Agent 1 CEO).
    pub(crate) async fn build_tools(&self, ctx: &RunContext) -> ToolDefinition {
        if !ctx.model_config.supports_native_tools() {
            return ToolDefinition {
                function_declarations: vec![],
            };
        }

        // Static cache for tool definitions, keyed by a combined hash of skills and safe_mode
        static TOOL_CACHE: Lazy<Mutex<LruCache<String, ToolDefinition>>> =
            Lazy::new(|| Mutex::new(LruCache::new(NonZeroUsize::new(64).unwrap())));

        let mut sorted_skills = ctx.skills.clone();
        sorted_skills.sort();
        let cache_key = format!(
            "{}:{}:{}",
            sorted_skills.join(","),
            ctx.safe_mode,
            ctx.agent_id
        );

        {
            let mut cache = TOOL_CACHE.lock();
            if let Some(cached) = cache.get(&cache_key) {
                return cached.clone();
            }
        }

        let mut function_declarations = Vec::new();

        // 1. Inject Registered Tools (Core, Filesystem, Advanced)
        let tool_list = self.state.registry.tool_registry.list_tools();
        for tool in tool_list {
            if self.state.resources.acl.is_tool_allowed(&ctx.agent_id, &ctx.role, ctx.authority_level, &tool.name) {
                // Specialized Skill Checks
                if tool.name == "execute_shell" && !(ctx.skills.contains(&"shell".to_string()) || ctx.skills.contains(&"terminal".to_string())) {
                    continue;
                }

                // Safe Mode Restrictions (Mutation Block)
                if ctx.safe_mode && (tool.name == "write_file" || tool.name == "delete_file" || tool.name == "execute_shell" || tool.name == "synthesize_micro_script") {
                    continue;
                }

                // Filesystem Capability Check
                if (tool.name == "read_file" || tool.name == "write_file" || tool.name == "list_files" || tool.name == "delete_file") && !self.has_file_system_capability(ctx) {
                    continue;
                }

                function_declarations.push(FunctionDeclaration {
                    name: tool.name.clone(),
                    description: tool.description.clone(),
                    parameters: tool.parameters.clone(),
                });
            }
        }

        // 2. Special Utility: Confidence Halting
        function_declarations.push(FunctionDeclaration {
            name: "set_confidence".to_string(),
            description: "Signals your current confidence in the answer. If score >= act_threshold, reasoning halts early.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "score": { "type": "number", "description": "Confidence score between 0.0 and 1.0." }
                },
                "required": ["score"]
            }),
        });

        // 4. Dynamic MCP Tools
        if !ctx.safe_mode {
            let mcp_tools = self
                .state
                .registry
                .mcp_host
                .list_tools(&ctx.skills, &self.state.registry.skills.skills)
                .await;
            for tool in mcp_tools {
                if self.state.resources.acl.is_tool_allowed(&ctx.agent_id, &ctx.role, ctx.authority_level, &tool.name) {
                    function_declarations.push(FunctionDeclaration {
                        name: tool.name,
                        description: tool.description,
                        parameters: tool.input_schema,
                    });
                }
            }
        }

        let final_definition = ToolDefinition {
            function_declarations,
        };

        TOOL_CACHE.lock().put(cache_key, final_definition.clone());
        final_definition
    }

    // --- Tool registration functions refactored into data-driven registry ---
    /// Central check for filesystem interaction permissions.
    fn has_file_system_capability(&self, ctx: &RunContext) -> bool {
        ctx.skills.iter().any(|s| {
            s == "filesystem"
                || s == "read_file"
                || s == "write_file"
                || s == "list_files"
                || s == "delete_file"
        })
    }
}

// Metadata: [synthesis]

// Metadata: [synthesis]

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::AppState;
    use crate::agent::runner::{AgentRunner, RunContext};
    use crate::agent::types::ModelConfig;
    use std::sync::Arc;
    use dashmap::DashMap;

    #[tokio::test]
    async fn test_aaak_encode() {
        let state = Arc::new(AppState::new_minimal_mock().await);
        let runner = AgentRunner::new(state);
        
        let raw = "STATUS: success. RESULT: Finding for mission was successful.";
        let encoded = runner.aaak_encode(raw);
        
        assert!(encoded.contains("*ok*"));
        assert!(encoded.contains("RES:"));
        assert!(encoded.contains("FND|"));
        assert!(!encoded.contains("STATUS: success"));
    }

    #[tokio::test]
    async fn test_context_pruning_basic() {
        let state = Arc::new(AppState::new_minimal_mock().await);
        let runner = AgentRunner::new(state);
        let ctx = RunContext {
            model_config: ModelConfig {
                tpm: Some(100), // Tiny limit
                ..Default::default()
            },
            ..RunContext::default()
        };

        let identity = "I am an agent. ".repeat(20);
        let memory = "I remember things. ".repeat(20);
        let repo_map = "This is a big repo map. ".repeat(100);
        let swarm_context = "This is shared findings. ".repeat(1000);

        let (pruned_map, pruned_swarm) = runner.prune_context(
            &ctx, &identity, &memory, &repo_map, &swarm_context
        );

        // Should have pruned something to fit within the 100 token limit
        assert!(pruned_swarm.len() < swarm_context.len());
        assert!(!pruned_map.is_empty());
    }

    #[tokio::test]
    async fn test_build_tools_ceo_restriction() {
        let state = Arc::new(AppState::new_minimal_mock().await);
        let runner = AgentRunner::new(state);
        let ctx = RunContext {
            agent_id: AGENT_CEO.to_string(),
            skills: vec!["read_file".to_string()],
            ..RunContext::default()
        };
        
        let tools = runner.build_tools(&ctx).await;
        
        // CEO should NOT have read_file (it's blocked for him to enforce delegation)
        let has_read_file = tools.function_declarations.iter().any(|f| f.name == "read_file");
        assert!(!has_read_file);
        
        // CEO SHOULD have issue_alpha_directive
        let has_directive = tools.function_declarations.iter().any(|f| f.name == "issue_alpha_directive");
        assert!(has_directive);
    }

    #[tokio::test]
    async fn test_build_tools_specialist_tools() {
        let state = Arc::new(AppState::new_minimal_mock().await);
        let runner = AgentRunner::new(state);
        let ctx = RunContext {
            agent_id: "specialist-1".to_string(),
            skills: vec!["read_file".to_string(), "write_file".to_string()],
            ..RunContext::default()
        };
        
        let tools = runner.build_tools(&ctx).await;
        
        // Specialist SHOULD have read_file and write_file
        let has_read = tools.function_declarations.iter().any(|f| f.name == "read_file");
        let has_write = tools.function_declarations.iter().any(|f| f.name == "write_file");
        assert!(has_read);
        assert!(has_write);
        
        // Specialist should NOT have issue_alpha_directive
        let has_directive = tools.function_declarations.iter().any(|f| f.name == "issue_alpha_directive");
        assert!(!has_directive);
    }

    #[tokio::test]
    async fn test_context_pruning_token_boundary() {
        let state = Arc::new(AppState::new_minimal_mock().await);
        let runner = AgentRunner::new(state);
        let ctx = RunContext {
            model_config: ModelConfig {
                tpm: Some(20), // Extremely tight
                ..Default::default()
            },
            ..RunContext::default()
        };

        let identity = "Identity ".repeat(20); 
        let memory = "Memory ".repeat(20);     
        let repo_map = "RepoMap ".repeat(100);
        let swarm_context = "Findings ".repeat(200); // 1800 chars, target_len will be 900 > 500

        let (pruned_map, pruned_swarm) = runner.prune_context(
            &ctx, &identity, &memory, &repo_map, &swarm_context
        );

        // repo_map should be significantly truncated or empty to fit the 20 token budget
        assert!(pruned_map.len() < repo_map.len());
        assert!(pruned_swarm.len() < swarm_context.len());
    }

    #[tokio::test]
    async fn test_build_tools_observer_restriction() {
        let state = Arc::new(AppState::new_minimal_mock().await);
        let runner = AgentRunner::new(state);
        let ctx = RunContext {
            agent_id: "observer-1".to_string(),
            authority_level: crate::agent::types::RoleAuthorityLevel::Observer,
            skills: vec!["read_file".to_string(), "write_file".to_string()],
            ..RunContext::default()
        };
        
        let tools = runner.build_tools(&ctx).await;
        
        let has_read = tools.function_declarations.iter().any(|f| f.name == "read_file");
        let has_write = tools.function_declarations.iter().any(|f| f.name == "write_file");
        
        assert!(has_read, "Observer should still be able to read");
        assert!(!has_write, "Observer should NOT be able to write even if skill is assigned");
    }

    #[tokio::test]
    async fn test_system_failure_context_injection() {
        // We use a mock state where base_dir points to a non-existent path to trigger file read failures
        let (tx, _) = tokio::sync::broadcast::channel(1);
        let (etx, _) = tokio::sync::broadcast::channel(1);
        let (atx, _) = tokio::sync::broadcast::channel(1);
        let (ttx, _) = tokio::sync::broadcast::channel(1);
        let (ptx, _) = tokio::sync::broadcast::channel(1);

        let base_dir = std::path::PathBuf::from("/non/existent/path/to/trigger/failure");

        let pool = crate::db::init_db("sqlite::memory:").await.unwrap();
        let permission_policy = Arc::new(crate::security::permissions::PermissionPolicy::new(pool.clone()));

        // Manual assembly of AppState to ensure no files exist
        let state = Arc::new(AppState {
            comms: Arc::new(crate::state::hubs::comm::CommunicationHub {
                tx, event_tx: etx, telemetry_tx: ttx, audio_stream_tx: atx, pulse_tx: ptx,
                oversight_queue: dashmap::DashMap::new(),
                oversight_resolvers: dashmap::DashMap::new(),
                active_runners: dashmap::DashMap::new(),
                event_sequence: std::sync::atomic::AtomicU64::new(0),
            }),
            governance: Arc::new(crate::state::hubs::gov::GovernanceHub {
                auto_approve_safe_skills: std::sync::atomic::AtomicBool::new(true),
                max_agents: std::sync::atomic::AtomicU32::new(10),
                max_clusters: std::sync::atomic::AtomicU32::new(5),
                max_swarm_depth: std::sync::atomic::AtomicU32::new(3),
                max_task_length: std::sync::atomic::AtomicUsize::new(4096),
                default_budget_usd: parking_lot::RwLock::new(0.5),
                active_agents: std::sync::atomic::AtomicU32::new(0),
                recruit_count: std::sync::atomic::AtomicU32::new(0),
                tpm_accumulator: std::sync::atomic::AtomicUsize::new(0),
                privacy_mode: std::sync::atomic::AtomicBool::new(false),
            }),
            registry: Arc::new(crate::state::hubs::reg::RegistryHub {
                agents: dashmap::DashMap::new(),
                providers: dashmap::DashMap::new(),
                provider_health: DashMap::new(),
                provider_failures: DashMap::new(),
                models: DashMap::new(),
                nodes: DashMap::new(),
                skills: Arc::new(crate::agent::script_skills::ScriptSkillsRegistry::mock(base_dir.clone())),
                skill_registry: Arc::new(crate::agent::skill_manifest::SkillRegistry::new()),
                tool_registry: Arc::new(crate::agent::runner::tools::dispatcher::Dispatcher::new().registry),
                mcp_host: Arc::new(crate::agent::mcp::McpHost::new(tokio::sync::broadcast::channel(1).0, None, permission_policy.clone())),
                hooks: Arc::new(crate::agent::hooks::HooksManager::new(&base_dir)),
            }),
            security: Arc::new(crate::state::hubs::sec::SecurityHub {
                audit_trail: Arc::new(crate::security::audit::MerkleAuditTrail::mock()),
                budget_guard: Arc::new(crate::security::metering::BudgetGuard::mock()),
                shell_scanner: Arc::new(crate::security::scanner::ShellScanner::mock()),
                secret_redactor: Arc::new(crate::secret_redactor::SecretRedactor::noop()),
                system_monitor: Arc::new(crate::security::monitoring::SecurityMonitor::new()),
                permission_policy,
                deploy_token: "test".to_string(),
            }),
            resources: Arc::new(crate::state::hubs::res::ResourceHub {
                pool,
                http_client: Arc::new(reqwest::Client::new()),
                audio_engine: tokio::sync::OnceCell::new(),
                audio_cache: Arc::new(crate::agent::audio_cache::BunkerCache::mock()),
                code_graph: tokio::sync::OnceCell::new(),
                identity_context: tokio::sync::OnceCell::new(),
                memory_context: tokio::sync::OnceCell::new(),
                rate_limiters: dashmap::DashMap::new(),
                initialization_registry: dashmap::DashMap::new(),
                hardware_profiler: Arc::new(crate::system::profiler::HardwareProfiler::new()),
                acl: Arc::new(crate::services::acl_service::AclService),
                renderer: Arc::new(crate::agent::runner::prompt_renderer::PromptRenderer),
                base_dir: base_dir.clone(),
            }),
            base_dir,
        });

        let runner = AgentRunner::new(state);
        let ctx = RunContext::default();
        let payload = crate::agent::types::TaskPayload {
            message: "Test failure".to_string(),
            ..Default::default()
        };

        let prompt = runner.build_system_prompt(&ctx, &payload.message).await;

        // The prompt should contain the system alert about identity failure
        assert!(prompt.contains("SYSTEM WARNING"), "Prompt should contain system warning alert");
        assert!(prompt.contains("IDENTITY_FAILURE"), "Prompt should mention IDENTITY_FAILURE");
        assert!(prompt.contains("MEMORY_FAILURE"), "Prompt should mention MEMORY_FAILURE");
    }
}

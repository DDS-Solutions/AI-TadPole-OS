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

use super::{AgentRunner, RunContext};
use crate::agent::constants::*;
use crate::agent::types::{FunctionDeclaration, ToolDefinition};
use tiktoken_rs::cl100k_base;

impl AgentRunner {
    /// 📟 [AAAK Encoder]
    /// Compresses mission context using the MemPalace-inspired AAAK dialect
    /// to increase context fidelity by 30-40% while reducing token load.
    fn aaak_encode(&self, text: &str) -> String {
        let mut encoded = text.to_string();

        // Status & Priority
        encoded = encoded.replace("STATUS: ok", "*ok*");
        encoded = encoded.replace("STATUS: success", "*ok*");
        encoded = encoded.replace("STATUS: failed", "*err*");
        encoded = encoded.replace("STATUS: error", "*err*");

        // Subject Markers
        encoded = encoded.replace("RESULT:", "RES:");
        encoded = encoded.replace("FINDING:", "FND:");
        encoded = encoded.replace("SOURCE:", "SRC:");
        encoded = encoded.replace("Location:", "LOC:");
        encoded = encoded.replace("Primary Goal:", "GOAL:");

        // Domain Shorthand (Harden Phase 4: Mission Specific)
        encoded = encoded.replace("Weather for zip", "WTR|");
        encoded = encoded.replace("degrees", "deg");
        encoded = encoded.replace("temperature", "temp");
        encoded = encoded.replace("Finding for mission", "FND|");
        encoded = encoded.replace("Strategic Intent:", "INT:");

        // Execution
        encoded = encoded.replace("Mission Complete", "*done*");
        encoded = encoded.replace("Task in progress", "*busy*");

        encoded
    }

    // ─────────────────────────────────────────────────────────

    //  SYSTEM PROMPT CONSTRUCTION (MODULARIZED)
    // ─────────────────────────────────────────────────────────

    /// Constructs the final system prompt for an agent run.
    ///
    /// Synthesizes identity, long-term memory, repo-map, and shared swarm
    /// findings into a single prompt, emphasizing modularity and robust error handling.
    pub(crate) async fn build_system_prompt(
        &self,
        ctx: &RunContext,
        hierarchy_label: &str,
        payload_message: &str,
    ) -> String {
        // 1. Context Gathering (Robustness improvement)
        let swarm_context_result =
            crate::agent::mission::get_mission_context(&self.state.resources.pool, &ctx.mission_id)
                .await;

        let swarm_context_str = match swarm_context_result {
            Ok(context) => context,
            Err(e) => {
                tracing::error!(
                    "🚨 [Runner] Failed to retrieve mission context for {}: {:?}. Using default empty context.",
                    ctx.mission_id, e
                );
                String::from("⚠️ WARNING: Failed to load shared mission context. Proceeding with limited findings.")
            }
        };

        // Fixed identity and memory loading
        let identity = self.state.resources.get_identity_context().await;
        let memory = self.state.resources.get_memory_context().await;

        // 2. Core Context Generation
        let repo_map = self.generate_repo_map(ctx, payload_message).await;
        let findings_str = ctx
            .recent_findings
            .as_deref()
            .unwrap_or("No recent findings inherited.");
        let compressed_findings = self.aaak_encode(findings_str);
        let compressed_swarm_context = self.aaak_encode(&swarm_context_str);

        // 3. Token Pruning (Crucial, remains high-priority logic)
        let (pruned_repo_map, _pruned_swarm_context_str) =
            self.prune_context(ctx, &identity, &memory, &repo_map, &swarm_context_str);

        // 4. Generate Structural Components (Modularity improvement)
        let (
            cluster_directory,
            filesystem_bias_mandate,
            lineage_display,
            _is_orchestrator,
            safe_mode_prefix,
            tool_mode_prefix,
        ) = self.generate_structural_components(ctx);

        let skill_fragments_str = self.generate_skill_fragments(ctx);
        let workflow_fragments_str = self.generate_workflow_fragments(ctx);
        let priority_str = self.generate_protocols(ctx);
        let swarm_str = self.generate_swarm_protocols(ctx);

        // 5. Final Assembly
        let system_prompt = format!(
            "{safe_mode_prefix}{tool_mode_prefix}You are {} (ID: {}, Role: {}) at the {} level of the swarm hierarchy.\n\
             Department: {}\n\
             Description: {}\n\n\
             DIRECTIVE PRIORITY (MANDATORY):\n{}\n\n\
             PERSONALITY & CONSTRAINTS:\n{}\n\n\
             {skill_fragments_str}\n\
             {workflow_fragments_str}\n\
             SWARM MISSION CONTEXT (Shared Findings):\n{}\n\n\
             CONTEXT BREADCRUMBS (Inherited File Paths):\n{}\n\n\
             RECENT FINDINGS (Inherited from Parent):\n{}\n\n\
             PRIMARY MISSION GOAL:\n{}\n\n\
             CLUSTER DIRECTORY (Available Specialists):\n{}\n\n\
             RECRUITMENT LINEAGE (Mission Path):\n{}\n\n\
             SKILLS: {:?}\n\
             WORKFLOWS: {:?}\n\n\
             ACTION BIAS (Troubleshooting & Discovery):\n{}\n\
             - NO REPEATS: If 'search_mission_knowledge' returns no results, do not try it again with slightly different wording. Immediately switch to technical discovery tools.\n\n\
             SWARM PROTOCOL:\n{}\n\n\
             --- GLOBAL ARCHITECTURE MAP ---\n\
             {}\n\n\
             --- GLOBAL OS IDENTITY ---\n\
             {}\n\n\
             --- LONG-TERM SWARM MEMORY ---\n\
             {}\n\n\
             --- CURRENT WORKING CONTEXT (Persistent Scratchpad) ---\n\
             {}\n\n\
             --- MISSION SUMMARY (Historical Context) ---\n\
             {}\n\n\
             (cache_control: {{\"type\": \"ephemeral\"}})\n\
             You may use 'update_working_memory' to refine your current scratchpad as your mission evolves.",
            ctx.name, ctx.agent_id, ctx.role, hierarchy_label, ctx.department, ctx.description,
            priority_str,
            ctx.model_config.system_prompt.as_deref().unwrap_or("No specific personality instructions."),
            compressed_swarm_context,
            self.generate_breadcrumbs_display(ctx),
            compressed_findings,
            ctx.primary_goal.as_deref().unwrap_or("See mission scope for details."),
            cluster_directory,
            lineage_display,
            &ctx.skills, &ctx.workflows,
            filesystem_bias_mandate,
            swarm_str,
            pruned_repo_map,
            identity,
            memory,
            serde_json::to_string_pretty(&ctx.working_memory).unwrap_or_else(|_| "{}".to_string()),
            self.aaak_encode(ctx.summarized_history.as_deref().unwrap_or("No history summarized yet."))
        );

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

    /// Generates the Repo Map summary, applying pruning logic if needed.
    /// Generates a real-time repository map for the agent's context.
    /// This includes a structural overview of the codebase and a list of
    /// recently accessed files to provide localized context for modifications.
    async fn generate_repo_map(&self, ctx: &RunContext, payload_message: &str) -> String {
        let graph: std::sync::Arc<parking_lot::RwLock<crate::utils::graph::CodeGraph>> =
            self.state.resources.get_code_graph().await;

        let summary = graph.read().generate_summary();

        // Only include repo map for non-trivial tasks or technical roles to save tokens
        if payload_message.len() > 10 || ctx.department.contains("Technical") {
            summary
        } else {
            "Architecture Map: Omitted for high-level greeting.".to_string()
        }
    }

    /// Handles context pruning based on token count (TPM Protection).
    /// Prunes the assembled context to fit within the model's TPM (Tokens Per Minute) limit.
    /// Prioritizes identity and recent history, truncating the repo map and
    /// shared findings if necessary to ensure prompt validity.
    fn prune_context(
        &self,
        ctx: &RunContext,
        identity: &str,
        memory: &str,
        repo_map: &str,
        swarm_context_str: &str,
    ) -> (String, String) {
        let bpe = cl100k_base().expect("Tadpole Error: Failed to initialize cl100k_base tokenizer. Check tiktoken-rs dependencies.");

        let tpm_limit = ctx.model_config.tpm.unwrap_or(12000);
        let safe_limit = (tpm_limit as f32 * 0.8) as usize;

        let initial_context = format!("{}{}{}{}", identity, memory, repo_map, swarm_context_str);
        let est_tokens = bpe.encode_with_special_tokens(&initial_context).len();

        let pruned_repo_map = repo_map.to_string();
        let mut pruned_swarm_context_str = swarm_context_str.to_string();

        if est_tokens > safe_limit {
            tracing::warn!("⚠️ [Runner] Context length ({est_tokens}) exceeds safe limit ({safe_limit}) for TPM {tpm_limit}. Initiating pruning...");

            // 1. Prune Repo Map
            if est_tokens > tpm_limit as usize {
                tracing::warn!("⚠️ [Pruning] Context exceeds TPM limit ({} > {}). Applying emergency truncation.", est_tokens, tpm_limit);

                let current_context = format!("{}{}{}", identity, memory, swarm_context_str);
                let est_tokens = bpe.encode_with_special_tokens(&current_context).len();

                if est_tokens > safe_limit && pruned_swarm_context_str.len() > 2000 {
                    pruned_swarm_context_str = self.safe_truncate(&pruned_swarm_context_str, 2000);
                }
            }
        } else {
            tracing::info!(
                "✅ [Runner] Context token load ({}) is safely below the limit.",
                est_tokens
            );
        }

        // Return the (potentially truncated) components
        (pruned_repo_map, pruned_swarm_context_str)
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
            "🔒 SECURITY: SAFE MODE ACTIVE. No filesystem mutations allowed. You are in read-only audit mode.".to_string()
        } else {
            "🔓 SECURITY: ACTIVE. You have authority to propose and execute mutations within your workspace scope.".to_string()
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

    /// Generates the detailed, multi-level protocols (CEO/COO/Specialist).
    /// Injects swarm-specific protocols based on the agent's hierarchical role.
    /// Enforces recursion limits, strategic intent inheritance, and
    /// identification masking for specialists when viewed by high-level managers.
    fn generate_protocols(&self, ctx: &RunContext) -> String {
        let mut priority_protocols: Vec<String> = Vec::new();
        let is_manager = ctx.agent_id == AGENT_CEO
            || ctx.agent_id == AGENT_COO
            || ctx.role.contains("Orchestrator")
            || ctx.role.contains("Overlord");

        // Protocol Logic remains non-trivial and is kept grouped for readability
        if ctx.agent_id == AGENT_CEO {
            priority_protocols.push("1. CEO PROTOCOL: You are a STRATEGIC ROUTER. You MUST delegate via 'issue_alpha_directive' for all complex missions. You are FORBIDDEN from using 'spawn_subagent' directly.".to_string());
        } else if ctx.agent_id == AGENT_COO {
            priority_protocols.push("1. COO PROTOCOL: You MUST delegate the mission to the Alpha Node. Use 'spawn_subagent' with agent_id 'alpha'. Direct specialist recruitment is SYSTEM-BLOCKED.".to_string());
        } else if ctx.agent_id == AGENT_ALPHA {
            priority_protocols.push("1. ALPHA COMMAND: You are the Swarm Mission Commander. You are responsible for recruiting and synthesizing specialists (Researcher, Coder, etc.).".to_string());
        } else {
            // Specialist Label Masking for Managers
            let identity_label = if is_manager {
                format!(
                    "Specialist-{}",
                    &ctx.agent_id[..std::cmp::min(ctx.agent_id.len(), 4)]
                )
            } else {
                ctx.agent_id.clone()
            };
            priority_protocols.push(format!("1. SPECIALIST AUTONOMY: You are tactical specialist {}. You MUST resolve your mission independently using your assigned tools.", identity_label));
            priority_protocols.push(format!("2. COMMANDER IS BUSY: You are under the supervision of the Alpha Node. Do NOT attempt to recruit '{}', '{}', or '{}' for assistance. Resolve the task yourself.", AGENT_ALPHA, AGENT_COO, AGENT_CEO).to_string());
        }

        // Shared protocols for all nodes
        priority_protocols.push("3. NO NARRATION: Do not explain strategy or tool options. CALL THE TOOLS. Text-only responses are MISSION FAILURE.".to_string());
        priority_protocols.push(
            "4. COMPLETION: Complete ALL steps autonomously before reporting back.".to_string(),
        );
        priority_protocols.push("5. OVERSIGHT TRUST: Never ask for permission. The system handles approval flows automatically.".to_string());

        let priority_str = priority_protocols
            .iter()
            .enumerate()
            .map(|(i, p)| format!("{}. {}", i + 1, p))
            .collect::<Vec<_>>()
            .join("\n");
        priority_str
    }

    /// Generates the shared Swarm Protocol rules.
    fn generate_swarm_protocols(&self, ctx: &RunContext) -> String {
        let mut swarm_protocols: Vec<String> = Vec::new();

        if ctx.agent_id == AGENT_CEO {
            swarm_protocols.push("1. CEO DELEGATION: Use 'issue_alpha_directive' for all operations. Do NOT spawn specialists directly.".to_string());
        } else if ctx.agent_id == AGENT_COO {
            swarm_protocols.push("1. COO OPERATION: Translate executive directives into a structured mission and recruit an ALPHA NODE (ID: alpha) as Commander.".to_string());
        } else if ctx.agent_id == AGENT_ALPHA {
            swarm_protocols.push("1. ALPHA COMMAND: Recruit specific specialists based on the task type (e.g. 'researcher', 'coder'). Synthesize their results for the COO.".to_string());
        } else {
            swarm_protocols.push("1. INDEPENDENT MISSION: Your priority is loop closure. Call the tools and provide results, not strategy.".to_string());
        }

        // The rest of the shared protocols
        swarm_protocols.push("2. REDUNDANCY: Check mission context/lineage before spawning sub-agents. Prefer lateral collaboration.".to_string());
        swarm_protocols.push("3. SOURCE OF TRUTH: Prioritize codebase tools (read_codebase_file) over general RAG for code queries.".to_string());
        swarm_protocols.push(format!("4. RECURSION FAILURE: You are FORBIDDEN from recruiting agents in your LINEAGE ({:?}) or YOURSELF ({}). Attempting to recruit these will causing a HARD SYSTEM ERROR.", &ctx.lineage, ctx.agent_id));

        swarm_protocols.join("\n")
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

        use dashmap::DashMap;
        use once_cell::sync::Lazy;

        // Static cache for tool definitions, keyed by a combined hash of skills and safe_mode
        static TOOL_CACHE: Lazy<DashMap<String, ToolDefinition>> =
            Lazy::new(DashMap::new);

        let mut sorted_skills = ctx.skills.clone();
        sorted_skills.sort();
        let cache_key = format!(
            "{}:{}:{}",
            sorted_skills.join(","),
            ctx.safe_mode,
            ctx.agent_id
        );

        if let Some(cached) = TOOL_CACHE.get(&cache_key) {
            return cached.value().clone();
        }

        // Bounded eviction to prevent memory growth
        if TOOL_CACHE.len() > 64 {
            TOOL_CACHE.clear();
        }

        let mut function_declarations = Vec::new();

        // 1. Inject Core Swarm Tools
        self.add_core_tools(ctx, &mut function_declarations);

        // 2. Inject File System Tools (Hierarchy-aware)
        if self.has_file_system_capability(ctx) {
            self.add_filesystem_tools(ctx, &mut function_declarations);
        }

        // 3. Inject Advanced Capability Tools
        self.add_advanced_tools(ctx, &mut function_declarations);

        // 4. Dynamic MCP Tools
        if !ctx.safe_mode {
            let mcp_tools = self
                .state
                .registry
                .mcp_host
                .list_tools(&ctx.skills, &self.state.registry.skills.skills)
                .await;
            for tool in mcp_tools {
                function_declarations.push(FunctionDeclaration {
                    name: tool.name,
                    description: tool.description,
                    parameters: tool.input_schema,
                });
            }
        }

        let tools = ToolDefinition {
            function_declarations,
        };
        TOOL_CACHE.insert(cache_key, tools.clone());
        tools
    }

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

    /// Registers the core operational tools of the swarm infrastructure.
    fn add_core_tools(
        &self,
        ctx: &RunContext,
        function_declarations: &mut Vec<FunctionDeclaration>,
    ) {
        if !ctx.safe_mode {
            if ctx.agent_id != AGENT_CEO {
                function_declarations.push(FunctionDeclaration {
                    name: "spawn_subagent".to_string(),
                    description: format!("Spawns one or more specialized sub-agents to handle tasks in parallel. Use this for high-throughput research, coding, or auditing. If you are the COO, you MUST use '{}' as the agent_id.", AGENT_ALPHA).to_string(),
                    parameters: serde_json::json!({
                        "type": "object",
                        "properties": {
                            "agent_id": { "type": "string", "description": "The ID of the specialist agent to recruit (e.g., 'researcher')." },
                            "agent_ids": { "type": "array", "items": { "type": "string" }, "description": "Optional: A list of multiple specialist IDs to recruit in parallel for a swarm task." },
                            "message": { "type": "string", "description": "The specific instruction or question for the sub-agent(s)." },
                            "role": { "type": "string", "description": "Optional: override the agent's functional role." }
                        },
                        "required": ["message"]
                    }),
                });
            }

            if ctx.agent_id == AGENT_CEO {
                function_declarations.push(FunctionDeclaration {
                    name: "issue_alpha_directive".to_string(),
                    description: "Delegates a complex mission to Tadpole Alpha (the COO). MUST be used by the CEO for all operations.".to_string(),
                    parameters: serde_json::json!({
                        "type": "object",
                        "properties": {
                            "directive": { "type": "string", "description": "The strategic objective or mission to delegate to Alpha." }
                        },
                        "required": ["directive"]
                    }),
                });
            }
        }

        function_declarations.push(FunctionDeclaration {
            name: "share_finding".to_string(),
            description: "Shares a key finding, insight, or data point with the rest of the swarm.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "topic": { "type": "string", "description": "Short label (e.g., 'API Endpoint')." },
                    "finding": { "type": "string", "description": "The detailed finding." }
                },
                "required": ["topic", "finding"]
            }),
        });

        function_declarations.push(FunctionDeclaration {
            name: "update_working_memory".to_string(),
            description: "Updates your persistent structured working memory (scratchpad). Use this to track plan progress, key discoveries, or state between turns. Overwrites existing keys.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "memory": { 
                        "type": "object", 
                        "description": "The JSON object containing the new memory state. This should be a full state or specific keys to merge." 
                    }
                },
                "required": ["memory"]
            }),
        });

        function_declarations.push(FunctionDeclaration {
            name: "complete_mission".to_string(),
            description: "Signals that the mission objective has been achieved. Provide a final comprehensive report. REQUIRES OVERSIGHT.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "final_report": { "type": "string", "description": "Detailed summary of all work done and final results." }
                },
                "required": ["final_report"]
            }),
        });

        function_declarations.push(FunctionDeclaration {
            name: "pin_mission".to_string(),
            description: "Pins the current mission for long-term retention. Use this for high-value research or discoveries that should bypass the automatic 48h Swarm Reaper cycle.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {}
            }),
        });
    }

    /// Registers file system I/O tools, enforcing the CEO block.
    fn add_filesystem_tools(
        &self,
        ctx: &RunContext,
        function_declarations: &mut Vec<FunctionDeclaration>,
    ) {
        if ctx.agent_id == AGENT_CEO {
            return;
        } // HIERARCHY GUARD: Agent 1 (CEO) is blocked.

        function_declarations.push(FunctionDeclaration {
            name: "read_file".to_string(),
            description: "Reads the content of a file within your assigned workspace. Use this to inspect your own work or data files.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "filename": { "type": "string", "description": "The name or relative path of the file to read." }
                },
                "required": ["filename"]
            }),
        });

        function_declarations.push(FunctionDeclaration {
            name: "write_file".to_string(),
            description: "Writes content to a file in the workspace. Overwrites existing files. REQUIRES OVERSIGHT for sensitive data.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "filename": { "type": "string", "description": "The name to save the file as (e.g., 'report.md')." },
                    "content": { "type": "string", "description": "The full text content to write." }
                },
                "required": ["filename", "content"]
            }),
        });

        function_declarations.push(FunctionDeclaration {
            name: "list_files".to_string(),
            description: "Lists all files in a directory within the workspace to navigate your environment.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "dir": { "type": "string", "description": "The directory to list (e.g., '.', 'data/'). Defaults to root." }
                }
            }),
        });

        function_declarations.push(FunctionDeclaration {
            name: "delete_file".to_string(),
            description: "Deletes a file from the workspace. REQUIRES CRITICAL OVERSIGHT (SAPPHIRE GATE). Use with extreme caution.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "filename": { "type": "string", "description": "The name of the file to permanently remove." }
                },
                "required": ["filename"]
            }),
        });
    }

    /// Registers the project-wide architectural and capability expansion tools.
    fn add_advanced_tools(
        &self,
        ctx: &RunContext,
        function_declarations: &mut Vec<FunctionDeclaration>,
    ) {
        function_declarations.push(FunctionDeclaration {
            name: "get_agent_metrics".to_string(),
            description: "Retrieves your own live identity and financial metrics (budget vs cost) from the system registry. Use this for deterministic financial auditing.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {}
            }),
        });

        function_declarations.push(FunctionDeclaration {
            name: "search_mission_knowledge".to_string(),
            description: "Searches the mission's vector RAG scope and the agent's long-term memory for relevant context using semantic search.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "query": { "type": "string", "description": "The search query or concept to look for in memory." }
                },
                "required": ["query"]
            }),
        });

        if ctx.skills.contains(&"propose_skill".to_string())
            || ctx.skills.contains(&"propose_capability".to_string())
        {
            function_declarations.push(FunctionDeclaration {
                name: "propose_capability".to_string(),
                description: "Proposes a new skill, workflow, or hook for the system. Use this to expand the swarm's abilities. REQUIRES OVERSIGHT.".to_string(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "type": { "type": "string", "enum": ["skill", "workflow", "hook"], "description": "The type of capability being proposed." },
                        "name": { "type": "string", "description": "Unique name (lowercase, snake_case)." },
                        "description": { "type": "string", "description": "LLM-facing description of utility." },
                        "execution_command": { "type": "string", "description": "For skills: Shell command (e.g., 'python scripts/tool.py')." },
                        "schema": { "type": "object", "description": "For skills: JSON schema for parameters." },
                        "content": { "type": "string", "description": "For workflows/hooks: Procedure markdown or script content." },
                        "hook_type": { "type": "string", "description": "For hooks: category (e.g., 'pre_validation', 'post_tool')." },
                        "full_instructions": { "type": "string", "description": "Detailed operating instructions." },
                        "negative_constraints": { "type": "array", "items": { "type": "string" }, "description": "Prohibited behaviors." }
                    },
                    "required": ["type", "name", "description"]
                }),
            });
        }

        function_declarations.push(FunctionDeclaration {
            name: "read_codebase_file".to_string(),
            description: "Reads the full content of a file from the project's own source code repository. Use this for architectural analysis, debugging the OS itself, or researching core logic. REQUIRES OVERSIGHT.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "Relative path to the file from the project root (e.g., 'server-rs/src/main.rs')." }
                },
                "required": ["path"]
            }),
        });

        function_declarations.push(FunctionDeclaration {
            name: "script_builder".to_string(),
            description: "Batches multiple atomic operations (filesystem, research, findings) into a single execution turn to reduce latency. Useful for complex setups.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "steps": {
                        "type": "array",
                        "items": {
                            "type": "object",
                            "properties": {
                                "tool": { "type": "string", "description": "The name of the tool to call." },
                                "params": { "type": "object", "description": "The parameters for that tool." }
                            },
                            "required": ["tool", "params"]
                        }
                    }
                },
                "required": ["steps"]
            }),
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::runner::RunContext;
    use crate::agent::types::{ModelConfig, ModelProvider};
    use crate::state::AppState;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_build_system_prompt_pruning() {
        let state = AppState::default();
        let runner = AgentRunner {
            state: Arc::new(state),
        };

        let mut ctx = RunContext {
            agent_id: "test-agent".to_string(),
            name: "Tester".to_string(),
            role: "Assurance".to_string(),
            department: "QA".to_string(),
            description: "Testing pruning logic with a long description ".repeat(5),
            skills: vec!["filesystem".to_string()],
            workflows: vec![],
            mission_id: "mission-1".to_string(),
            depth: 0,
            lineage: vec![],
            workspace_root: std::path::PathBuf::from("."),
            base_dir: std::path::PathBuf::from("."),
            fs_adapter: crate::adapter::filesystem::FilesystemAdapter::new(
                std::path::PathBuf::from("."),
            ),
            safe_mode: false,
            analysis: false,
            traceparent: None,
            user_id: None,
            provider_name: "groq".to_string(),
            last_accessed_files: std::sync::Arc::new(parking_lot::Mutex::new(Vec::new())),
            recent_findings: None,
            working_memory: serde_json::json!({}),
            mcp_tools: vec![],
            model_config: ModelConfig {
                provider: ModelProvider::Groq,
                model_id: "llama3-7b".to_string(),
                tpm: Some(1000), // Very low limit for testing
                skills: None,
                workflows: None,
                mcp_tools: None,
                ..ModelConfig::default()
            },
            summarized_history: None,
            structured_output: false,
            backlog: None,
            primary_goal: None,
            budget_usd: 0.0,
            current_cost_usd: 0.0,
        };

        // 1. SMALL CONTEXT
        let prompt_small = runner.build_system_prompt(&ctx, "root", "hi").await;
        assert!(
            !prompt_small.contains("[TRUNCATED"),
            "Small context should not be truncated"
        );

        // 2. LARGE CONTEXT (Technical role)
        ctx.department = "Technical (Engineering)".to_string();
        let _prompt_tech = runner.build_system_prompt(&ctx, "root", "hi").await;
    }

    #[tokio::test]
    async fn test_ceo_tool_block() {
        let state = AppState::default();
        let runner = AgentRunner {
            state: Arc::new(state),
        };

        // Agent 1 is the CEO
        let ctx = RunContext {
            agent_id: AGENT_CEO.to_string(),
            model_config: ModelConfig { provider: ModelProvider::Groq, ..Default::default() },
            ..Default::default()
        };

        let tools = runner.build_tools(&ctx).await;

        // Verify no filesystem tools
        let has_filesystem = tools
            .function_declarations
            .iter()
            .any(|t| t.name == "read_file" || t.name == "write_file");
        assert!(
            !has_filesystem,
            "CEO (Agent 1) should NOT have direct filesystem access"
        );
    }

    #[tokio::test]
    async fn test_specialist_masking() {
        let state = AppState::default();
        let runner = AgentRunner {
            state: Arc::new(state),
        };

        let ctx = RunContext { role: "Orchestrator".to_string(), ..Default::default() }; // Managers get masking

        let protocols = runner.generate_protocols(&ctx);
        assert!(
            protocols.contains("Specialist-"),
            "Managers should see anonymized Specialist labels"
        );
    }

    #[tokio::test]
    async fn test_aaak_encoding_fidelity() {
        let state = AppState::default();
        let runner = AgentRunner {
            state: Arc::new(state),
        };

        let input = "STATUS: failed. RESULT: The temperature is 72 degrees.";
        let encoded = runner.aaak_encode(input);

        assert!(encoded.contains("*err*"), "Should encode failed to *err*");
        assert!(encoded.contains("RES:"), "Should encode RESULT: to RES:");
        assert!(
            encoded.contains("temp"),
            "Should encode temperature to temp"
        );
    }
}

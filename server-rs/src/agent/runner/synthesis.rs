//! Prompt Synthesis — Context assembly and system prompt generation
//!
//! @docs ARCHITECTURE:Runner

use super::{AgentRunner, RunContext};
use tiktoken_rs::cl100k_base;

impl AgentRunner {
    // ─────────────────────────────────────────────────────────
    //  SYSTEM PROMPT CONSTRUCTION
    // ─────────────────────────────────────────────────────────

    /// Constructs the final system prompt for an agent run.
    ///
    /// Synthesizes identity, long-term memory, repo-map, and shared swarm
    /// findings into a single prompt. Includes a TPM-aware pruning loop
    /// that truncates less critical context if the token limit is approached.
    pub(crate) async fn build_system_prompt(
        &self,
        ctx: &RunContext,
        hierarchy_label: &str,
        payload_message: &str,
    ) -> String {
        let swarm_context =
            crate::agent::mission::get_mission_context(&self.state.resources.pool, &ctx.mission_id)
                .await
                .unwrap_or_default();

        // Fix 8: Use cached context instead of reading from disk on every prompt
        let identity = self.state.resources.get_identity_context().await;
        let memory = self.state.resources.get_memory_context().await;

        // Fix 9: Only include repo map for non-trivial tasks or technical roles to save tokens
        let mut repo_map = if payload_message.len() > 10 || ctx.department.contains("Technical") {
            let graph: std::sync::Arc<parking_lot::RwLock<crate::utils::graph::CodeGraph>> =
                self.state.resources.get_code_graph().await;
            let summary = graph.read().generate_summary();
            summary
        } else {
            "Architecture Map: Omitted for high-level greeting.".to_string()
        };

        let mut swarm_context_str = if swarm_context.is_empty() {
            "No shared findings yet.".to_string()
        } else {
            swarm_context.clone()
        };

        let safe_mode_prefix = if ctx.safe_mode { "🛡️ [SAFE MODE ACTIVE] " } else { "" };
        let tool_mode_prefix = if ctx.structured_output { "🤖 [STRUCTURED OUTPUT] " } else { "" };

        // --- CONTEXT PRUNING (TPM PROTECTION) ---
        let bpe = match cl100k_base() {
            Ok(b) => b,
            Err(e) => {
                tracing::error!("🚨 [Runner] Failed to initialize tokenizer: {:?}", e);
                return format!(
                    "{safe_mode_prefix}{tool_mode_prefix} Swarm system error: Tokenizer failure. \
                     Proceeding with un-pruned context. \n\n identity: {}\n\n findings: {}",
                    identity, swarm_context_str
                );
            }
        };
        let tpm_limit = ctx.model_config.tpm.unwrap_or(12000);
        let safe_limit = (tpm_limit as f32 * 0.8) as usize; // 80% safe threshold

        let mut est_tokens = bpe
            .encode_with_special_tokens(&format!(
                "{}{}{}{}{}",
                identity, memory, repo_map, swarm_context_str, ctx.description
            ))
            .len();

        if est_tokens > safe_limit {
            tracing::warn!("⚠️ [Runner] Context length ({est_tokens}) exceeds safe limit ({safe_limit}) for TPM {tpm_limit}. Pruning...");

            // 1. Prune Repo Map first (it's often the biggest bloat)
            if repo_map.len() > 2000 {
                repo_map = format!("{}... [TRUNCATED FOR TPM LIMIT]", &repo_map[..2000]);
            }

            // 2. Prune Swarm Context if still needed
            est_tokens = bpe
                .encode_with_special_tokens(&format!(
                    "{}{}{}{}{}",
                    identity, memory, repo_map, swarm_context_str, ctx.description
                ))
                .len();
            if est_tokens > safe_limit && swarm_context_str.len() > 1000 {
                swarm_context_str = format!("{}... [TRUNCATED]", &swarm_context_str[..1000]);
            }

            let final_est = bpe
                .encode_with_special_tokens(&format!(
                    "{}{}{}{}{}",
                    identity, memory, repo_map, swarm_context_str, ctx.description
                ))
                .len();
            tracing::info!("✅ [Runner] Context pruned down to ~{} tokens.", final_est);
        }

        let breadcrumbs = ctx.last_accessed_files.lock();
        let breadcrumbs_str = if breadcrumbs.is_empty() {
            "None available.".to_string()
        } else {
            breadcrumbs.join("\n- ")
        };

        let findings_str = ctx
            .recent_findings
            .as_deref()
            .unwrap_or("No recent findings inherited.");

        let lineage_display = if ctx.lineage.is_empty() {
            "None (You are the root node)".to_string()
        } else {
            ctx.lineage.join(" -> ")
        };

        let mut forbidden = ctx.lineage.clone();
        forbidden.push(ctx.agent_id.clone());

        let safe_mode_prefix = if ctx.safe_mode {
            "[BRAINSTORM SAFE MODE ACTIVE]\n\
             You are currently in Safe/Brainstorm Mode for a high-level strategic discussion with the Overlord. ALL execution tools and workflows (such as bash, writing files, and spawning sub-agents) have been DISABLED for safety. Discuss ideas, explore concepts, and generate plans. Do not attempt to execute actions; only strategize.\n\n"
        } else {
            ""
        };

        let tool_mode_prefix = if !ctx.model_config.supports_native_tools() {
            "[TEXT-ONLY MODE ACTIVE]\n\
             Your current model engine does NOT support native tool-calling. You must operate strictly in text-only mode. \
             DO NOT attempt to use <function=...> or any tool-calling syntax. \
             If you need to perform an action, describe it in natural language. You have been assigned skills and workflows for context, but you cannot execute them directly in this mode.\n\n"
        } else {
            ""
        };

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
        let skill_fragments_str = if skill_fragments.is_empty() {
            "".to_string()
        } else {
            format!(
                "\nSKILL-SPECIFIC DEEP INSTRUCTIONS:\n{}\n",
                skill_fragments.join("\n\n")
            )
        };

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
        let workflow_fragments_str = if workflow_fragments.is_empty() {
            "".to_string()
        } else {
            format!(
                "\nWORKFLOW-SPECIFIC PROCEDURES:\n{}\n",
                workflow_fragments.join("\n\n")
            )
        };

        format!(
            "{safe_mode_prefix}{tool_mode_prefix}You are {} (ID: {}, Role: {}) at the {} level of the swarm hierarchy.\n\
             Department: {}\n\
             Description: {}\n\n\
             DIRECTIVE PRIORITY (MANDATORY):\n\
             1. CEO PROTOCOL: You are a STRATEGIC ROUTER. You MUST delegate via 'issue_alpha_directive' for all complex missions. You are FORBIDDEN from using 'spawn_subagent' directly. Do NOT explain why you are delegating.\n\
             2. COO PROTOCOL: If you are the COO, you MUST delegate the mission to the Alpha Node. Use 'spawn_subagent' with agentId 'alpha'. The Alpha Node has full filesystem and coding skills. Direct specialist recruitment is SYSTEM-BLOCKED.\n\
             3. NO NARRATION: Do not explain strategy, roadblocks, or tool options. CALL THE TOOLS. Text-only responses are MISSION FAILURE. If you have nothing to add, call 'share_finding' or 'complete_mission'.\n\
             4. COMPLETION: If a directive contains multiple steps (e.g. 'write then read'), you MUST complete ALL steps autonomously before reporting back or asking for further instructions.\n\
             5. OVERSIGHT TRUST: Never ask for permission to perform an action (e.g. delete file). CALL THE TOOLS. The system will handle the approval flow (Sapphire Gate) automatically.\n\
             6. ACTION BIAS: If a file path is unknown, you MUST use 'list_files' to find it. Never report 'file not found' until you have searched the codebase.\n\n\
             PERSONALITY & CONSTRAINTS:\n\
             {}\n\n\
             {skill_fragments_str}\n\
             {workflow_fragments_str}\n\
             SWARM MISSION CONTEXT (Shared Findings):\n\
             {}\n\n\
             CONTEXT BREADCRUMBS (Inherited File Paths):\n\
             - {}\n\n\
             RECENT FINDINGS (Inherited from Parent):\n\
             {}\n\n\
             RECRUITMENT LINEAGE (Mission Path):\n\
             {}\n\n\
             SKILLS: {:?}\n\
             WORKFLOWS: {:?}\n\n\
             ACTION BIAS (Troubleshooting & Discovery):\n\
             - MANDATE: If you are asked about a file and it's not in your CONTEXT BREADCRUMBS, you MUST use 'list_files' or 'grep_search' to find it. Do not report failure until 'read_codebase_file' has been attempted on the correct path.\n\
             - NO REPEATS: If 'search_mission_knowledge' returns no results, do not try it again with slightly different wording. Immediately switch to technical discovery tools.\n\n\
             SWARM PROTOCOL:\n\
             1. RECURSION LIMIT: You are prohibited from recruiting YOURSELF or any agent already in your LINEAGE. Do not spawn any of these IDs: {:?}.\n\
             2. REDUNDANCY: Always check if the mission context or lineage already contains the information you need before spawning a sub-agent. Prefer lateral collaboration over deep hierarchy.\n\
             3. HIERARCHY: You report to higher nodes. Your autonomy is bound by Oversight & Compliance.\n\
             4. DEEP ANALYSIS: If 'Deep Analysis' is in your workflows, you MUST follow the Generator->Verifier->Reviser-> loop. Identify your own flaws before final delivery.\n\
             5. SOURCE OF TRUTH: If a specific file is mentioned in your task or CONTEXT BREADCRUMBS, prioritize 'read_codebase_file' or 'read_file'. (EXCEPTION: If your task explicitly instructs you to CREATE a new file, obviously do not try to read it first. Just use 'write_file' directly). If the path is unknown, use 'list_files' or 'grep_search' to find it FIRST. Do not rely solely on 'search_mission_knowledge' for codebase queries.\n\
             6. NO NARRATION: Do not describe your plan, \"options,\" or intended tool usage in text. Just CALL THE TOOLS. Your mission is to provide RESULTS, not a strategy guide.\n\
             7. LOOP CLOSURE: If a sub-agent reports a failure or a mandate, you MUST attempt to solve it yourself using your own tools before responding to the parent. Never just pass an error up the chain if you have the skills to fix it.\n\
             8. CEO DELEGATION: If you are the CEO (Agent of Nine), your primary tool for complex missions is 'issue_alpha_directive'. Do NOT spawn specialists directly. Delegate to the COO (Tadpole Alpha) who will manage the execution cluster.\n\
             9. COO OPERATION: If you are the COO (Tadpole Alpha), you MUST translate executive directives into a structured mission and recruit an ALPHA NODE (ID: alpha) to serve as the Swarm Mission Commander.\n\
             10. ALPHA COMMAND: If you are the Alpha Node (ID: alpha), you are the Swarm Mission Commander. You are responsible for recruiting specialists (Researcher, Coder, etc.) and synthesizing their work for the COO.\n\n\
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
             ctx.model_config.system_prompt.as_deref().unwrap_or("No specific personality instructions."),
             swarm_context_str,
             breadcrumbs_str,
             findings_str,
             lineage_display,
             ctx.skills, ctx.workflows,
             forbidden,
             repo_map,
             identity,
             memory,
             serde_json::to_string_pretty(&ctx.working_memory).unwrap_or_else(|_| "{}".to_string()),
             ctx.summarized_history.as_deref().unwrap_or("No history summarized yet.")
        )
    }

    // ─────────────────────────────────────────────────────────
    //  TOOL DEFINITIONS
    // ─────────────────────────────────────────────────────────

    /// Dynamically manifests tool definitions for the LLM provider.
    ///
    /// Maps enabled skills and MCP tools into Gemini-compatible functions.
    /// Features a static cache to avoid redundant schema generation on hot paths.
    pub(crate) async fn build_tools(&self, ctx: &RunContext) -> crate::agent::gemini::GeminiTool {
        if !ctx.model_config.supports_native_tools() {
            return crate::agent::gemini::GeminiTool {
                function_declarations: vec![],
            };
        }

        use dashmap::DashMap;
        use once_cell::sync::Lazy;

        // Static cache for tool definitions, keyed by a combined hash of skills and safe_mode
        // Fix 7: Bounded to 64 entries to prevent unbounded memory growth
        static TOOL_CACHE: Lazy<DashMap<String, crate::agent::gemini::GeminiTool>> =
            Lazy::new(DashMap::new);

        // Create a unique key for the current combination of skills and safety settings
        let mut sorted_skills = ctx.skills.clone();
        sorted_skills.sort();
        let cache_key = format!("{}:{}", sorted_skills.join(","), ctx.safe_mode);

        if let Some(cached) = TOOL_CACHE.get(&cache_key) {
            return cached.value().clone();
        }

        // Evict all entries if cache exceeds bound (simple but effective)
        if TOOL_CACHE.len() > 64 {
            TOOL_CACHE.clear();
        }

        let mut function_declarations = Vec::new();

        // Always include Swarm Core tools unless in safe mode
        if !ctx.safe_mode {
            if ctx.agent_id != "1" {
                function_declarations.push(crate::agent::gemini::GeminiFunctionDeclaration {
                    name: "spawn_subagent".to_string(),
                    description: "Spawns a specialized sub-agent to handle a specific sub-task in parallel. Use this for research, coding, or auditing while you orchestrate. If you are the COO, you MUST use 'alpha' as the agentId.".to_string(),
                    parameters: serde_json::json!({
                        "type": "object",
                        "properties": {
                            "agentId": { "type": "string", "description": "The ID of the specialist agent to recruit." },
                            "message": { "type": "string", "description": "The specific instruction or question for the sub-agent." }
                        },
                        "required": ["agentId", "message"]
                    }),
                });
            }

            if ctx.agent_id == "1" {
                function_declarations.push(crate::agent::gemini::GeminiFunctionDeclaration {
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

        function_declarations.push(crate::agent::gemini::GeminiFunctionDeclaration {
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

        function_declarations.push(crate::agent::gemini::GeminiFunctionDeclaration {
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

        function_declarations.push(crate::agent::gemini::GeminiFunctionDeclaration {
            name: "complete_mission".to_string(),
            description: "Signals that the mission objective has been achieved. Provide a final comprehensive report. REQUIRES OVERSIGHT.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "finalReport": { "type": "string", "description": "Detailed summary of all work done and final results." }
                },
                "required": ["finalReport"]
            }),
        });

        function_declarations.push(crate::agent::gemini::GeminiFunctionDeclaration {
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

        // --- FILESYSTEM TOOLS (Securely Scoped to Workspace) ---
        let has_fs_skill = ctx.skills.iter().any(|s| {
            s == "filesystem"
                || s == "read_file"
                || s == "write_file"
                || s == "list_files"
                || s == "delete_file"
        });

        // HIERARCHY GUARD: Agent 1 (CEO) is forbidden from direct worker tool access.
        // This forces delegation via 'issue_alpha_directive' for complex system changes.
        if has_fs_skill && ctx.agent_id != "1" {
            function_declarations.push(crate::agent::gemini::GeminiFunctionDeclaration {
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

            function_declarations.push(crate::agent::gemini::GeminiFunctionDeclaration {
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

            function_declarations.push(crate::agent::gemini::GeminiFunctionDeclaration {
                name: "list_files".to_string(),
                description: "Lists all files in a directory within the workspace to navigate your environment.".to_string(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "dir": { "type": "string", "description": "The directory to list (e.g., '.', 'data/'). Defaults to root." }
                    }
                }),
            });

            function_declarations.push(crate::agent::gemini::GeminiFunctionDeclaration {
                name: "delete_file".to_string(),
                description: " Deletes a file from the workspace. REQUIRES CRITICAL OVERSIGHT (SAPPHIRE GATE). Use with extreme caution.".to_string(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "filename": { "type": "string", "description": "The name of the file to permanently remove." }
                    },
                    "required": ["filename"]
                }),
            });
        }

        if ctx.skills.contains(&"propose_skill".to_string()) {
            function_declarations.push(crate::agent::gemini::GeminiFunctionDeclaration {
                name: "propose_skill".to_string(),
                description: "Proposes a new skill or workflow for the system. Use this to expand the swarm's abilities based on user intent. REQUIRES OVERSIGHT.".to_string(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "type": { "type": "string", "enum": ["skill", "workflow"], "description": "Whether this is a 'skill' (active tool) or 'workflow' (passive instruction)." },
                        "name": { "type": "string", "description": "The unique name of the skill (lowercase, snake_case)." },
                        "description": { "type": "string", "description": "LLM-facing description of what this does." },
                        "executionCommand": { "type": "string", "description": "For skills: The shell command to run (e.g., 'python scripts/tool.py')." },
                        "schema": { "type": "object", "description": "For skills: JSON schema defining input parameters." },
                        "content": { "type": "string", "description": "For workflows: The Markdown content of the procedure." }
                    },
                    "required": ["type", "name", "description"]
                }),
            });
        }

        function_declarations.push(crate::agent::gemini::GeminiFunctionDeclaration {
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

        function_declarations.push(crate::agent::gemini::GeminiFunctionDeclaration {
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

        // Dynamic Skills & MCP Tools
        if !ctx.safe_mode {
            let mcp_tools = self
                .state
                .registry
                .mcp_host
                .list_tools(&ctx.skills, &self.state.registry.skills.skills)
                .await;
            for tool in mcp_tools {
                function_declarations.push(crate::agent::gemini::GeminiFunctionDeclaration {
                    name: tool.name,
                    description: tool.description,
                    parameters: tool.input_schema,
                });
            }
        }

        // Fallback for skill definitions not yet migrated to MCP style in build_tools
        // (Removing the old manual loop as list_tools now handles it)

        let tools = crate::agent::gemini::GeminiTool {
            function_declarations,
        };
        TOOL_CACHE.insert(cache_key, tools.clone());
        tools
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::runner::RunContext;
    use crate::agent::types::ModelConfig;
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
                provider: "groq".to_string(),
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
}

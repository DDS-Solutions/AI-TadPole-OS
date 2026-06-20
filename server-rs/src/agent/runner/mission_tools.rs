//! @docs ARCHITECTURE:Runner
//!
//! ### AI Assist Note
//! **Mission Tools**: High-level task management and global project interaction.
//! Includes **Knowledge Search** (vector-based RAG), **Codebase Navigation**,
//! and **Skill Proposals**. Implements **Sovereignty Guard** (Oversight for
//! codebase writes) and **Breadcrumb Resolution** for ambiguous project paths.
//!
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: Sector RAG (LanceDB) connection error, codebase path
//!   validation failure (traversal block), or sensitive file (e.g. .env) access block.
//! - **Trace Scope**: `server-rs::agent::runner::mission_tools`

use super::{AgentRunner, RunContext};
use crate::agent::runner::tools::error::ToolExecutionError;
use crate::error::AppError;
use std::sync::OnceLock;

const CODEBASE_READ_MAX_CHARS: usize = 10_000;
#[cfg(feature = "vector-memory")]
const RAG_TOP_K: usize = 5;
const BREADCRUMB_CAP: usize = 10;
#[cfg(feature = "vector-memory")]
const IKS_MIN_CONFIDENCE: f32 = 0.3;
const OVERSIGHT_DESC_PREVIEW_BYTES: usize = 120;
const SWARM_REAPER_CYCLE: &str = "48h";
const SENSITIVE_IKS_TOPICS: &[&str] = &["finance", "legal", "payroll", "pii", "medical"];
const MAX_TEMPLATE_LITERAL_DEPTH: usize = 32;

impl AgentRunner {
    /// Handles `share_finding`: persists a finding to the swarm context.
    ///
    /// ### 📢 Global Visibility
    /// Findings are persisted to the database and also broadcasted to the
    /// live telemetry stream. This allows human operators to see
    /// "Intelligence Nuggets" in real-time.
    pub(crate) async fn handle_share_finding(
        &self,
        ctx: &RunContext,
        fc: &crate::agent::types::ToolCall,
    ) -> Result<String, ToolExecutionError> {
        let topic = require_str_opt(ctx, &fc.args, "topic", "share_finding")?.unwrap_or_else(|| "General".to_string());
        let finding = require_str(ctx, &fc.args, "finding", "share_finding")?;

        tracing::info!(
            "📢 [Swarm] Agent {} shared a finding on {}: {}",
            ctx.agent_id,
            topic,
            finding
        );
        self.broadcast_agent(
            ctx,
            &format!("📢 Swarm: context added for {}", topic),
            "success",
        );

        crate::agent::mission::share_finding(
            &self.state.resources.pool,
            &ctx.mission_id,
            &ctx.agent_id,
            &topic,
            &finding,
        )
        .await?;

        // Conversational "Echo" to ensure the agent's contribution is visible in the chat bubble
        let echo = format!("**(Shared finding on topic '{}' successfully recorded.)**", topic);
        Ok(echo)
    }

    /// Handles `complete_mission`: marks the mission as completed after oversight.
    ///
    /// ### 🏁 Finalization Workflow
    /// 1. **Oversight**: Submits the final report for human/governance approval.
    /// 2. **Semantic Archive**: If approved, triggers a RAG archival pass to
    ///    summarize session memories into a dense record.
    /// 3. **Clean Delivery**: Strips previous turn noise to provide a professional
    ///    report to the user.
    pub(crate) async fn handle_complete_mission(
        &self,
        ctx: &RunContext,
        fc: &crate::agent::types::ToolCall,
    ) -> Result<String, ToolExecutionError> {
        let report = require_str_opt(ctx, &fc.args, "final_report", "complete_mission")?.unwrap_or_else(|| "Mission complete.".to_string());

        tracing::info!(
            "🏁 [Mission] Agent {} requesting completion...",
            ctx.agent_id
        );
        self.broadcast_agent(
            ctx,
            "🏁 Oversight: work finished. Reviewing final report...",
            "warning",
        );

        let approved = self
            .submit_oversight(
                crate::agent::types::ToolCallAudit {
                    id: uuid::Uuid::new_v4().to_string(),
                    agent_id: ctx.agent_id.clone(),
                    mission_id: Some(ctx.mission_id.clone()),
                    skill: "complete_mission".to_string(),
                    params: fc.args.clone(),
                    department: ctx.department.clone(),
                    description: "Final mission sign-off and reporting.".to_string(),
                    timestamp: chrono::Utc::now().to_rfc3339(),
                },
                Some(ctx.mission_id.clone()),
            )
            .await?;

        if approved {
            // Semantic Archival before closing mission
            #[cfg(feature = "vector-memory")]
            {
                let api_key = ctx.model_config.api_key.clone().unwrap_or_else(|| {
                    self.state
                        .registry
                        .providers
                        .get(&ctx.model_config.provider.to_string())
                        .and_then(|p| p.api_key.clone())
                        .unwrap_or_default()
                });

                if let Some(mem) = self.connect_mission_memory(ctx).await {
                    if let Err(e) = mem
                        .summarize_and_archive(
                            &ctx.mission_id,
                            &self.state.resources.http_client,
                            &api_key,
                            &ctx.model_config.model_id,
                        )
                        .await
                    {
                        tracing::warn!(
                            "⚠️ [Mission Archival] Failed to summarize and archive mission {}: {}",
                            ctx.mission_id,
                            e
                        );
                    }
                }
            }

            crate::agent::mission::update_mission(
                &self.state.resources.pool,
                &ctx.mission_id,
                crate::agent::types::MissionStatus::Completed,
                None,
            )
            .await?;
            self.broadcast_agent(
                ctx,
                &format!("✅ Mission {} COMPLETED and archived.", ctx.mission_id),
                "success",
            );
            // 🛡️ [Harden Phase 4: Clean Delivery]
            // We strip previous turn noise to provide a clear, professional final report.
            Ok(format!(
                "🏁 **MISSION ARCHIVE REPORT**\n\
                 Mission ID: {}\n\
                 Status: SUCCESS\n\n\
                 The mission has been successfully summarized and archived into long-term vector memory.\n\n\
                 **Summary Highlights**:\n{}",
                ctx.mission_id, report
            ))
        } else {
            Ok("(Mission completion REJECTED)".to_string())
        }
    }

    /// Handles `pin_mission`: protects the mission from the Swarm Reaper.
    ///
    /// Note: The `_fc` parameter is deliberately ignored because pinning is a global toggle
    /// that operates strictly on the current mission ID within the RunContext, requiring no
    /// external tool arguments.
    pub(crate) async fn handle_pin_mission(
        &self,
        ctx: &RunContext,
        _fc: &crate::agent::types::ToolCall,
    ) -> Result<String, ToolExecutionError> {
        tracing::info!(
            "📌 [Governance] Agent {} pinning mission {} for long-term retention.",
            ctx.agent_id,
            ctx.mission_id
        );

        sqlx::query("UPDATE mission_history SET is_pinned = 1 WHERE id = ?")
            .bind(&ctx.mission_id)
            .execute(&self.state.resources.pool)
            .await
            .map_err(AppError::Sqlx)?;

        self.broadcast_agent(
            ctx,
            &format!(
                "📌 Mission {} pinned for long-term retention.",
                ctx.mission_id
            ),
            "success",
        );

        Ok(
            format!("(MISSION PINNED: This mission will now bypass the {} Swarm Reaper cycle.)", SWARM_REAPER_CYCLE)
        )
    }

    /// Handles `search_mission_knowledge`: vector search across LanceDB memory scope.
    ///
    /// ### 🧩 RAG Fallback
    /// If no semantic findings are found, this function provides "Hints" to the
    /// agent to try physical filesystem tools (`list_files`, `grep_search`).
    pub(crate) async fn handle_search_mission_knowledge(
        &self,
        ctx: &RunContext,
        fc: &crate::agent::types::ToolCall,
    ) -> Result<String, ToolExecutionError> {
        let query = require_str(ctx, &fc.args, "query", "search_mission_knowledge")?;
        tracing::info!(
            "🧠 [Memory] Agent {} searching knowledge for: {}",
            ctx.agent_id,
            query
        );

        #[cfg_attr(not(feature = "vector-memory"), allow(unused_mut))]
        let mut results_text = String::new();

        #[cfg(feature = "vector-memory")]
        {
            let api_key = ctx.model_config.api_key.clone().unwrap_or_default();
            let http_client = self.state.resources.http_client.clone();

            if let Ok(vec) =
                crate::agent::memory::get_gemini_embedding(&http_client, &api_key, query).await
            {
                if let Some(mission_mem) = self.connect_mission_memory(ctx).await {
                    if let Ok(results) = mission_mem.search_knowledge(vec, RAG_TOP_K).await {
                        for (i, text) in results.into_iter().enumerate() {
                            results_text.push_str(&format!("[Result {}]: {}\n", i + 1, text));
                        }
                    }
                }
            }
        }

        if results_text.is_empty() {
            let lower_query = query.to_lowercase();
            let is_financial = lower_query.contains("budget")
                || lower_query.contains("cost")
                || lower_query.contains("limit")
                || lower_query.contains("usd");

            let hint = if is_financial {
                "HINT: This query appears to relate to live financial metrics. Vector RAG only contains static shared findings. Use 'get_agent_metrics' to see your own current budget/costs, or 'query_financial_logs' to review overall mission history."
            } else {
                "This query might be reference a physical file or keyword in the workspace. Since you have technical tools, you should now use 'list_files' or 'grep_search' to locate the target and then 'read_file' or 'read_codebase_file' to inspect it directly."
            };

            Ok(format!(
                "(RESOURCE NOT FOUND: No relevant shared findings found for '{}'. {})",
                query, hint
            ))
        } else {
            Ok(format!(
                "(SEARCH RESULTS FOR '{}'):\n{}",
                query, results_text
            ))
        }
    }

    /// Handles `read_codebase_file`: allows reading files from the project root.
    ///
    /// ### 🛡️ Security Filter (Sovereign)
    /// - **Oversight**: Requires manual approval to access files outside the
    ///   mission sandbox.
    /// - **Credential Filter**: Blocks any files containing "key", "token", or
    ///   ".env" to prevent data leakage.
    /// - **Breadcrumb Resolution**: If a relative path is ambiguous, uses
    ///   the `RunContext` history to resolve the absolute path.
    pub(crate) async fn handle_read_codebase_file(
        &self,
        ctx: &RunContext,
        fc: &crate::agent::types::ToolCall,
    ) -> Result<String, ToolExecutionError> {
        let path_str = require_str(ctx, &fc.args, "path", "read_codebase_file")?;

        tracing::info!(
            "🔍 [Sovereignty] Agent {} requesting codebase read: {}",
            ctx.agent_id,
            path_str
        );

        self.broadcast_agent(
            ctx,
            &format!(
                "🔍 Oversight: wants to read codebase file: {}. Review required.",
                path_str
            ),
            "warning",
        );

        let approved = self
            .submit_oversight(
                crate::agent::types::ToolCallAudit {
                    id: uuid::Uuid::new_v4().to_string(),
                    agent_id: ctx.agent_id.clone(),
                    mission_id: Some(ctx.mission_id.clone()),
                    skill: "read_codebase_file".to_string(),
                    params: fc.args.clone(),
                    department: ctx.department.clone(),
                    description: format!(
                        "Reading codebase file for architectural analysis: {}",
                        path_str
                    ),
                    timestamp: chrono::Utc::now().to_rfc3339(),
                },
                Some(ctx.mission_id.clone()),
            )
            .await?;

        if !approved {
            return Ok("(Codebase read REJECTED by Oversight)".to_string());
        }

        let final_path = match self.require_path_safety(ctx, &path_str).await {
            Ok(p) => p,
            Err(e) => return Ok(e),
        };

        match self.read_codebase_file_helper(ctx, &final_path, &path_str).await {
            Ok(content) => {
                let truncated = self.safe_truncate(&content, CODEBASE_READ_MAX_CHARS);
                Ok(format!("(FILE CONTENT OF {}):\n\n{}", path_str, truncated))
            }
            Err(e) => Ok(format!("(CODEBASE READ FAILED for {}: {})", path_str, e)),
        }
    }

    /// Handles `propose_capability`: submits a new skill, workflow, or hook proposal to the oversight system.
    pub(crate) async fn handle_propose_capability(
        &self,
        ctx: &RunContext,
        fc: &crate::agent::types::ToolCall,
    ) -> Result<String, ToolExecutionError> {
        let cap_type_str = require_str_opt(ctx, &fc.args, "type", "propose_capability")?.unwrap_or_else(|| "skill".to_string());
        let name = require_str_opt(ctx, &fc.args, "name", "propose_capability")?.unwrap_or_else(|| "unnamed".to_string());
        let description = require_str_opt(ctx, &fc.args, "description", "propose_capability")?.unwrap_or_default();

        let cap_type = match cap_type_str.as_str() {
            "workflow" => crate::agent::types::SkillType::Workflow,
            "hook" => crate::agent::types::SkillType::Hook,
            _ => crate::agent::types::SkillType::Skill,
        };

        // Validation logic
        match cap_type {
            crate::agent::types::SkillType::Skill => {
                if fc.args.get("execution_command").is_none() || fc.args.get("schema").is_none() {
                    return Ok("(Proposal REJECTED: Skill proposals must include 'execution_command' and 'schema' arguments.)".to_string());
                }
            }
            crate::agent::types::SkillType::Workflow => {
                if fc.args.get("content").is_none() {
                    return Ok("(Proposal REJECTED: Workflow proposals must include a 'content' argument.)".to_string());
                }
            }
            crate::agent::types::SkillType::Hook => {
                if fc.args.get("hook_type").is_none() || fc.args.get("content").is_none() {
                    return Ok("(Proposal REJECTED: Hook proposals must include 'hook_type' and 'content' arguments.)".to_string());
                }
            }
        }

        let proposal_id = uuid::Uuid::new_v4().to_string();
        let payload_json = serde_json::to_string(&fc.args).unwrap_or_default();

        tracing::info!(
            "💡 [Cognitive Autonomy] Agent {} proposing a new capability: {} ({})",
            ctx.agent_id,
            name,
            cap_type_str
        );

        // Persist to the capability_proposals table for human review
        sqlx::query(
            "INSERT INTO capability_proposals (id, mission_id, agent_id, capability_type, name, description, payload, status) VALUES (?, ?, ?, ?, ?, ?, ?, 'pending')"
        )
        .bind(&proposal_id)
        .bind(&ctx.mission_id)
        .bind(&ctx.agent_id)
        .bind(&cap_type_str)
        .bind(&name)
        .bind(&description)
        .bind(&payload_json)
        .execute(&self.state.resources.pool)
        .await
        .map_err(AppError::Sqlx)?;

        self.broadcast_agent(
            ctx,
            &format!(
                "💡 Oversight: new capability proposal '{}' ({}) submitted for review.",
                name, cap_type_str
            ),
            "warning",
        );

        // Non-blocking response: The agent can proceed with other tasks while approval is pending.
        Ok(format!(
            "(CAPABILITY PROPOSAL SUBMITTED): The proposed {} '{}' has been queued for human oversight (Proposal ID: {}). You may continue your mission while the Governance Hub reviews this capability expansion.",
            cap_type_str, name, proposal_id
        ))
    }


    /// Handles `list_file_symbols`: parses a file to list functions, classes, and variables.
    pub(crate) async fn handle_list_file_symbols(
        &self,
        ctx: &RunContext,
        fc: &crate::agent::types::ToolCall,
    ) -> Result<String, ToolExecutionError> {
        let path_str = require_str(ctx, &fc.args, "path", "list_file_symbols")?;

        self.broadcast_agent(
            ctx,
            &format!(
                "🔍 Oversight: wants to list symbols in codebase file: {}. Review required.",
                path_str
            ),
            "warning",
        );

        let approved = self
            .submit_oversight(
                crate::agent::types::ToolCallAudit {
                    id: uuid::Uuid::new_v4().to_string(),
                    agent_id: ctx.agent_id.clone(),
                    mission_id: Some(ctx.mission_id.clone()),
                    skill: "list_file_symbols".to_string(),
                    params: fc.args.clone(),
                    department: ctx.department.clone(),
                    description: format!(
                        "Listing codebase symbols for architectural analysis: {}",
                        path_str
                    ),
                    timestamp: chrono::Utc::now().to_rfc3339(),
                },
                Some(ctx.mission_id.clone()),
            )
            .await?;

        if !approved {
            return Ok("(List codebase symbols REJECTED by Oversight)".to_string());
        }

        let final_path = match self.require_path_safety(ctx, &path_str).await {
            Ok(p) => p,
            Err(e) => return Ok(e),
        };

        match self.read_codebase_file_helper(ctx, &final_path, &path_str).await {
            Ok(content) => {
                let symbols = self.extract_symbols(&content, &path_str);
                if symbols.is_empty() {
                    Ok(format!("(No recognizable symbols found in {})", path_str))
                } else {
                    let symbol_list = symbols.join("\n");
                    Ok(format!("(SYMBOLS IN {}):\n\n{}", path_str, symbol_list))
                }
            }
            Err(e) => Ok(format!("(LIST SYMBOLS FAILED for {}: {})", path_str, e)),
        }
    }

    /// Handles `get_symbol_body`: extracts the implementation of a specific symbol from a file.
    pub(crate) async fn handle_get_symbol_body(
        &self,
        ctx: &RunContext,
        fc: &crate::agent::types::ToolCall,
    ) -> Result<String, ToolExecutionError> {
        let path_str = require_str(ctx, &fc.args, "path", "get_symbol_body")?;
        let symbol_name = require_str(ctx, &fc.args, "symbol", "get_symbol_body")?;

        self.broadcast_agent(
            ctx,
            &format!(
                "🔍 Oversight: wants to retrieve symbol body of '{}' in: {}. Review required.",
                symbol_name, path_str
            ),
            "warning",
        );

        let approved = self
            .submit_oversight(
                crate::agent::types::ToolCallAudit {
                    id: uuid::Uuid::new_v4().to_string(),
                    agent_id: ctx.agent_id.clone(),
                    mission_id: Some(ctx.mission_id.clone()),
                    skill: "get_symbol_body".to_string(),
                    params: fc.args.clone(),
                    department: ctx.department.clone(),
                    description: format!(
                        "Extracting symbol implementation for architectural analysis: {} in {}",
                        symbol_name, path_str
                    ),
                    timestamp: chrono::Utc::now().to_rfc3339(),
                },
                Some(ctx.mission_id.clone()),
            )
            .await?;

        if !approved {
            return Ok("(Get codebase symbol body REJECTED by Oversight)".to_string());
        }

        let final_path = match self.require_path_safety(ctx, &path_str).await {
            Ok(p) => p,
            Err(e) => return Ok(e),
        };

        match self.read_codebase_file_helper(ctx, &final_path, &path_str).await {
            Ok(content) => {
                if let Some(body) = self.extract_symbol_body(&content, &symbol_name, &path_str) {
                    Ok(format!(
                        "(BODY OF SYMBOL '{}' IN {}):\n\n{}",
                        symbol_name, path_str, body
                    ))
                } else {
                    Ok(format!(
                        "(SYMBOL '{}' NOT FOUND in {})",
                        symbol_name, path_str
                    ))
                }
            }
            Err(e) => Ok(format!("(GET SYMBOL FAILED for {}: {})", path_str, e)),
        }
    }

    /// Internal helper: Extracts a list of symbols using regex patterns based on file extension.
    fn extract_symbols(&self, content: &str, path: &str) -> Vec<String> {
        let ext = std::path::Path::new(path)
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("");
        let mut symbols = Vec::new();

        static RUST_RE: OnceLock<regex::Regex> = OnceLock::new();
        static JS_TS_RE: OnceLock<regex::Regex> = OnceLock::new();
        static PYTHON_RE: OnceLock<regex::Regex> = OnceLock::new();
        static FALLBACK_RE: OnceLock<regex::Regex> = OnceLock::new();

        match ext {
            "rs" => {
                let re = RUST_RE.get_or_init(|| regex::Regex::new(r"(?m)^[ \t]*(?:pub(?:\(.*\))?\s+)?(?:async\s+)?(fn|struct|enum|trait|type|const|static)\s+([a-zA-Z_][a-zA-Z0-9_]*)").unwrap());
                for cap in re.captures_iter(content) {
                    symbols.push(format!("[{}] {}", &cap[1], &cap[2]));
                }
            }
            "ts" | "js" | "tsx" | "jsx" => {
                let re = JS_TS_RE.get_or_init(|| regex::Regex::new(r"(?m)^[ \t]*(?:export\s+)?(?:async\s+)?(function|class|type|interface|const|let|var)\s+([a-zA-Z_][a-zA-Z0-9_]*)").unwrap());
                for cap in re.captures_iter(content) {
                    symbols.push(format!("[{}] {}", &cap[1], &cap[2]));
                }
            }
            "py" => {
                let re = PYTHON_RE.get_or_init(|| regex::Regex::new(r"(?m)^[ \t]*(def|class)\s+([a-zA-Z_][a-zA-Z0-9_]*)").unwrap());
                for cap in re.captures_iter(content) {
                    symbols.push(format!("[{}] {}", &cap[1], &cap[2]));
                }
            }
            _ => {
                // Fallback for unknown languages: search for common patterns
                let re = FALLBACK_RE.get_or_init(|| regex::Regex::new(
                    r"(?m)^[ \t]*(?:function|class|def|fn)\s+([a-zA-Z_][a-zA-Z0-9_]*)",
                ).unwrap());
                for cap in re.captures_iter(content) {
                    symbols.push(format!("[symbol] {}", &cap[1]));
                }
            }
        }
        symbols
    }

    /// Internal helper: Extracts the body of a specific symbol.
    ///
    /// ### ⚠️ Python Limitations & Parser Assumptions
    /// - **Tabs vs. Spaces**: Indents are normalized (tabs counted as 4 spaces) to handle mixed spacing.
    /// - **Decorators & Multi-line defs**: Decorators above functions and multi-line parameter definitions
    ///   are not fully parsed, only the function body following the signature is evaluated.
    /// - **Docstrings at Column 0**: Multi-line docstrings starting at column 0 (unindented) inside
    ///   a function can trigger premature termination.
    fn extract_symbol_body(&self, content: &str, symbol: &str, path: &str) -> Option<String> {
        let lines: Vec<&str> = content.lines().collect();
        let ext = std::path::Path::new(path)
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("");

        // Find the line where the symbol is defined
        let start_idx = match ext {
            "py" => lines.iter().position(|l| {
                l.contains(&format!("def {}", symbol)) || l.contains(&format!("class {}", symbol))
            }),
            "rs" => lines.iter().position(|l| {
                l.contains(&format!("fn {}", symbol))
                    || l.contains(&format!("struct {}", symbol))
                    || l.contains(&format!("enum {}", symbol))
                    || l.contains(&format!("trait {}", symbol))
            }),
            _ => lines.iter().position(|l| {
                l.contains(&format!("function {}", symbol))
                    || l.contains(&format!("class {}", symbol))
                    || l.contains(&format!("const {}", symbol))
            }),
        };

        if let Some(start) = start_idx {
            let mut body = Vec::new();
            let mut found_start = false;
            let mut indent_level = None;
            let mut parser_state = BraceCounterState::default();

            for line in &lines[start..] {
                body.push(*line);

                if ext == "py" {
                    // Python indentation-based blocks
                    // Normalize tabs to 4 spaces to prevent tabs/spaces mismatches
                    let current_indent: usize = line.chars()
                        .take_while(|c| c.is_whitespace())
                        .map(|c| if c == '\t' { 4 } else { 1 })
                        .sum();
                    
                    let trimmed = line.trim();
                    if !trimmed.is_empty() 
                        && !trimmed.starts_with('#') 
                        && !trimmed.starts_with("\"\"\"") 
                        && !trimmed.starts_with("'''") 
                    {
                        if let Some(level) = indent_level {
                            if current_indent <= level && body.len() > 1 {
                                // Block ended
                                body.pop();
                                break;
                            }
                        } else {
                            indent_level = Some(current_indent);
                        }
                    }
                } else {
                    // Brace-based blocks (RS, JS, TS)
                    count_braces_robust(line, &mut parser_state);

                    if line.contains('{') {
                        found_start = true;
                    }

                    if found_start && parser_state.current_depth == 0 {
                        break;
                    }
                }
            }
            return Some(body.join("\n"));
        }
        None
    }

    /// Handles `send_mission_directive`: delegates a task to another agent.
    pub(crate) async fn handle_send_mission_directive(
        &self,
        ctx: &RunContext,
        fc: &crate::agent::types::ToolCall,
    ) -> Result<String, ToolExecutionError> {
        let target_agent_id = require_str(ctx, &fc.args, "agent_id", "send_mission_directive")?;
        let instruction = require_str(ctx, &fc.args, "instruction", "send_mission_directive")?;

        tracing::info!(
            "🧬 [Swarm] Agent {} issuing directive to {}: {}",
            ctx.agent_id,
            target_agent_id,
            instruction
        );

        self.broadcast_agent(
            ctx,
            &format!("🧬 Issuing directive to {}...", target_agent_id),
            "info",
        );

        let id = super::swarm_persistence::save_directive(
            &self.state.resources.pool,
            ctx,
            &target_agent_id,
            &instruction,
        )
        .await?;

        Ok(format!("Directive [{}] sent to agent {}. It will be picked up at the start of their next turn.", id, target_agent_id))
    }

    /// Handles `request_peer_audit`: submits content for review.
    pub(crate) async fn handle_request_peer_audit(
        &self,
        ctx: &RunContext,
        fc: &crate::agent::types::ToolCall,
    ) -> Result<String, ToolExecutionError> {
        let reviewer_id = require_str(ctx, &fc.args, "reviewer_id", "request_peer_audit")?;
        let content = require_str(ctx, &fc.args, "content", "request_peer_audit")?;
        let criteria = fc.args.get("criteria").and_then(|v| v.as_str());

        tracing::info!(
            "⚖️ [Swarm] Agent {} requested audit from {}.",
            ctx.agent_id,
            reviewer_id
        );

        self.broadcast_agent(
            ctx,
            &format!("⚖️ Requesting audit from {}...", reviewer_id),
            "info",
        );

        let id = super::swarm_persistence::save_review_request(
            &self.state.resources.pool,
            ctx,
            &reviewer_id,
            &content,
            criteria,
        )
        .await?;

        Ok(format!(
            "Audit request [{}] sent to {}. Check back later for feedback.",
            id, reviewer_id
        ))
    }

    /// Handles `submit_peer_review`: provides feedback on a request.
    pub(crate) async fn handle_submit_peer_review(
        &self,
        ctx: &RunContext,
        fc: &crate::agent::types::ToolCall,
    ) -> Result<String, ToolExecutionError> {
        let request_id = require_str(ctx, &fc.args, "request_id", "submit_peer_review")?;
        let feedback = require_str(ctx, &fc.args, "feedback", "submit_peer_review")?;
        let status = fc
            .args
            .get("status")
            .and_then(|v| v.as_str())
            .unwrap_or("approved");

        tracing::info!(
            "✅ [Swarm] Agent {} submitting peer review for {}.",
            ctx.agent_id,
            request_id
        );

        self.broadcast_agent(
            ctx,
            &format!("✅ Submitting peer review for {}...", request_id),
            "success",
        );

        super::swarm_persistence::submit_review(
            &self.state.resources.pool,
            &request_id,
            &feedback,
            status,
        )
        .await?;

        Ok(format!(
            "Peer review for [{}] submitted. Feedback: {}",
            request_id, feedback
        ))
    }

    /// Handles `archive_to_global_vault`: persists a mission nugget to the global swarm vault.
    pub(crate) async fn handle_archive_to_global_vault(
        &self,
        ctx: &RunContext,
        fc: &crate::agent::types::ToolCall,
    ) -> Result<String, ToolExecutionError> {
        let topic = require_str_opt(ctx, &fc.args, "topic", "archive_to_global_vault")?.unwrap_or_else(|| "General".to_string());
        tracing::info!(
            "🏛️ [Global Vault] Agent {} archiving nugget on {}.",
            ctx.agent_id,
            topic
        );
        #[cfg(feature = "vector-memory")]
        {
            let content = require_str(ctx, &fc.args, "content", "archive_to_global_vault")?;
            if let Some(vec) = self.gemini_embed_or_log(ctx, &content, "archive").await {
                if let Some(vault) = self.connect_global_vault().await {
                    let id = format!("global-{}", uuid::Uuid::new_v4());
                    match vault.add_memory(&id, &content, &ctx.mission_id, vec).await {
                        Ok(_) => {
                            self.broadcast_agent(
                                ctx,
                                &format!("🏛️ Global Vault: nugget archived on {}", topic),
                                "success",
                            );
                            return Ok(format!("(GLOBAL ARCHIVE SUCCESS): Nugget on '{}' added to the swarm intelligence vault.", topic));
                        }
                        Err(e) => {
                            tracing::warn!(
                                "⚠️ [Global Vault] Failed to add memory entry to global vault for mission {}: {}",
                                ctx.mission_id,
                                e
                            );
                        }
                    }
                }
            }
        }

        Ok(
            "(GLOBAL ARCHIVE FAILED): Ensure vector-memory is enabled and API keys are valid."
                .to_string(),
        )
    }

    /// Handles `search_global_vault`: performs a semantic search across all mission histories.
    pub(crate) async fn handle_search_global_vault(
        &self,
        ctx: &RunContext,
        fc: &crate::agent::types::ToolCall,
    ) -> Result<String, ToolExecutionError> {
        let query = require_str(ctx, &fc.args, "query", "search_global_vault")?;
        tracing::info!(
            "🏛️ [Global Vault] Agent {} searching global vault for: {}",
            ctx.agent_id,
            query
        );

        #[cfg(feature = "vector-memory")]
        {
            if let Some(vec) = self.gemini_embed_or_log(ctx, &query, "search").await {
                if let Some(vault) = self.connect_global_vault().await {
                    match vault.search_knowledge(vec, RAG_TOP_K).await {
                        Ok(results) => {
                            if results.is_empty() {
                                return Ok(format!(
                                    "(GLOBAL SEARCH): No relevant intelligence found for '{}'.",
                                    query
                                ));
                            } else {
                                return Ok(format!(
                                    "(GLOBAL INTELLIGENCE RETRIEVED for '{}'):\n\n{}",
                                    query,
                                    results.join("\n\n----- \n\n")
                                ));
                            }
                        }
                        Err(e) => {
                            tracing::warn!(
                                "⚠️ [Global Vault] Failed to search global vault records: {}",
                                e
                            );
                        }
                    }
                }
            }
        }

        Ok(
            "(GLOBAL SEARCH FAILED): Ensure vector-memory is enabled and API keys are valid."
                .to_string(),
        )
    }

    /// Handles `store_knowledge`: writes a new curated knowledge entry to the IKS.
    /// Deduplicates by content hash. Escapes to oversight if topic is sensitive.
    pub(crate) async fn handle_store_knowledge(
        &self,
        ctx: &RunContext,
        fc: &crate::agent::types::ToolCall,
    ) -> Result<String, ToolExecutionError> {
        let text = require_str(ctx, &fc.args, "text", "store_knowledge")?;
        let topic = require_str(ctx, &fc.args, "topic", "store_knowledge")?;

        // Topic-ACL Check
        let topic_lower = topic.to_lowercase();
        let is_sensitive = SENSITIVE_IKS_TOPICS.iter().any(|&s| topic_lower.contains(s));

        if is_sensitive {
            self.broadcast_agent(
                ctx,
                &format!(
                    "🧠 Oversight: wants to store sensitive knowledge under '{}'. Review required.",
                    topic
                ),
                "warning",
            );

            let approved = self
                .submit_oversight(
                    crate::agent::types::ToolCallAudit {
                        id: uuid::Uuid::new_v4().to_string(),
                        agent_id: ctx.agent_id.clone(),
                        mission_id: Some(ctx.mission_id.clone()),
                        skill: "store_knowledge".to_string(),
                        params: fc.args.clone(),
                        department: ctx.department.clone(),
                        description: format!(
                            "Storing sensitive knowledge on topic '{}': {}",
                            topic,
                            {
                                let mut end = OVERSIGHT_DESC_PREVIEW_BYTES;
                                if text.len() > end {
                                    while end > 0 && !text.is_char_boundary(end) {
                                        end -= 1;
                                    }
                                    &text[..end]
                                } else {
                                    &text
                                }
                            }
                        ),
                        timestamp: chrono::Utc::now().to_rfc3339(),
                    },
                    Some(ctx.mission_id.clone()),
                )
                .await?;

            if !approved {
                return Ok("(IKS write REJECTED by Oversight)".to_string());
            }
        }

        #[cfg(feature = "vector-memory")]
        {
            let cluster_id = fc
                .args
                .get("cluster_id")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            let confidence = fc
                .args
                .get("confidence")
                .and_then(|v| v.as_f64())
                .map(|f| f as f32);
            let ttl_days = fc.args.get("ttl_days").and_then(|v| v.as_i64());

            let req = crate::agent::knowledge_store::AddKnowledgeRequest {
                text,
                topic: topic.clone(),
                cluster_id,
                source_node_id: None,
                source_agent_id: Some(ctx.agent_id.clone()),
                confidence,
                ttl_days,
                human_confirmed: if is_sensitive { Some(true) } else { None }, // Auto-confirm if sensitive & approved
            };

            match self.state.resources.get_knowledge_store().await {
                Ok(ks) => {
                    match ks
                        .add_entry(req, self.state.resources.http_client.as_ref().clone())
                        .await
                    {
                        Ok(entry) => {
                            let msg = format!(
                                "(STORE KNOWLEDGE SUCCESS): Entry stored with ID: {}",
                                entry.id
                            );
                            self.broadcast_agent(
                                ctx,
                                &format!("🧠 Curated fact stored in IKS on topic '{}'", topic),
                                "success",
                            );
                            return Ok(msg);
                        }
                        Err(e) => {
                            return Ok(format!("(STORE KNOWLEDGE FAILED: {})", e));
                        }
                    }
                }
                Err(e) => {
                    return Ok(format!(
                        "(STORE KNOWLEDGE FAILED: Could not acquire store: {})",
                        e
                    ));
                }
            }
        }

        #[cfg(not(feature = "vector-memory"))]
        {
            Ok("(STORE KNOWLEDGE FAILED: Vector memory is disabled on this node.)".to_string())
        }
    }

    /// Handles `search_knowledge`: searches across the cross-cluster persistent Institutional Knowledge Store.
    pub(crate) async fn handle_search_knowledge(
        &self,
        ctx: &RunContext,
        fc: &crate::agent::types::ToolCall,
    ) -> Result<String, ToolExecutionError> {
        let query = require_str(ctx, &fc.args, "query", "search_knowledge")?;

        tracing::info!(
            "🧠 [IKS] Agent {} searching Institutional Knowledge Store for: {}",
            ctx.agent_id,
            query
        );

        #[cfg(feature = "vector-memory")]
        {
            let topic = require_str_opt(ctx, &fc.args, "topic", "search_knowledge")?;
            let limit = fc
                .args
                .get("limit")
                .and_then(|v| v.as_u64())
                .map(|n| n as usize);

            let req = crate::agent::knowledge_store::KnowledgeSearchRequest {
                query,
                topic,
                cluster_id: None, // Global + local cluster scoped
                limit,
                min_confidence: Some(IKS_MIN_CONFIDENCE),
            };

            match self.state.resources.get_knowledge_store().await {
                Ok(ks) => {
                    match ks
                        .search(&req, self.state.resources.http_client.as_ref().clone())
                        .await
                    {
                        Ok(results) => {
                            if results.is_empty() {
                                return Ok(format!("(IKS SEARCH): No relevant institutional knowledge found for '{}'.", req.query));
                            } else {
                                let mut lines = Vec::new();
                                for (i, entry) in results.into_iter().enumerate() {
                                    lines.push(format!(
                                        "[Entry {} (ID: {}, Topic: '{}', Confidence: {:.2})]:\n{}",
                                        i + 1,
                                        entry.id,
                                        entry.topic,
                                        entry.confidence,
                                        entry.text
                                    ));
                                }
                                return Ok(format!(
                                    "(INSTITUTIONAL KNOWLEDGE RETRIEVED for '{}'):\n\n{}",
                                    req.query,
                                    lines.join("\n\n----- \n\n")
                                ));
                            }
                        }
                        Err(e) => {
                            return Ok(format!("(SEARCH KNOWLEDGE FAILED: {})", e));
                        }
                    }
                }
                Err(e) => {
                    return Ok(format!(
                        "(SEARCH KNOWLEDGE FAILED: Could not acquire store: {})",
                        e
                    ));
                }
            }
        }

        #[cfg(not(feature = "vector-memory"))]
        {
            Ok("(SEARCH KNOWLEDGE FAILED: Vector memory is disabled on this node.)".to_string())
        }
    }

    /// Centralized safety path validator and resolver helper
    async fn require_path_safety(
        &self,
        ctx: &RunContext,
        path_str: &str,
    ) -> Result<crate::utils::security::SafePath, String> {
        let sensitive_patterns = [".env", "key", "token", "credential", "secret", "private"];
        if sensitive_patterns
            .iter()
            .any(|p| path_str.to_lowercase().contains(p))
        {
            return Err(format!(
                "(SECURITY BLOCKED: Access to sensitive file '{}' is prohibited.)",
                path_str
            ));
        }

        let root = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
        let target_path = match crate::utils::security::validate_path(&root, path_str) {
            Ok(p) => p,
            Err(e) => {
                return Err(format!("(SECURITY BLOCKED: {})", e));
            }
        };

        let mut final_path = target_path.clone();
        if tokio::fs::metadata(&final_path).await.is_err() {
            let breadcrumbs = ctx.last_accessed_files.lock();
            if let Some(resolved) = breadcrumbs.iter().find(|p| {
                let p_path = std::path::Path::new(p);
                let target = std::path::Path::new(path_str);
                p_path.ends_with(target)
            }) {
                tracing::info!(
                    "🧩 [Context] Resolved ambiguous codebase path '{}' to '{}' via breadcrumbs",
                    path_str,
                    resolved
                );
                final_path = crate::utils::security::SafePath::from_trusted(root.join(resolved));
            }
        }

        if tokio::fs::metadata(&final_path).await.is_err()
            && !path_str.contains('/')
            && !path_str.contains('\\')
        {
            let common_dirs = ["src", "src/agent", "server-rs/src", "server-rs/src/agent"];
            for dir in common_dirs {
                let alt_path = root.join(dir).join(path_str);
                if tokio::fs::metadata(&alt_path).await.is_ok() {
                    tracing::info!("🧩 [Context] Resolved ambiguous codebase path '{}' to '{:?}' via common-dirs", path_str, alt_path);
                    final_path = crate::utils::security::SafePath::from_trusted(alt_path);
                    break;
                }
            }
        }

        Ok(final_path)
    }

    /// Helper to connect to mission memory
    #[cfg(feature = "vector-memory")]
    async fn connect_mission_memory(
        &self,
        ctx: &RunContext,
    ) -> Option<crate::agent::memory::VectorMemory> {
        let cluster_name = ctx.workspace_root
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| "default".to_string());
        let mission_scope_dir = format!(
            "data/workspaces/{}/missions/{}/scope.lance",
            cluster_name, ctx.mission_id
        );
        match crate::agent::memory::VectorMemory::connect(&mission_scope_dir, "scope").await {
            Ok(mem) => Some(mem),
            Err(e) => {
                tracing::warn!(
                    "⚠️ [Mission Archival] Failed to connect to mission scope vector memory for mission {}: {}",
                    ctx.mission_id,
                    e
                );
                None
            }
        }
    }

    /// Record path access breadcrumb
    fn record_breadcrumb(&self, ctx: &RunContext, final_path: &std::path::Path, path_str: &str) {
        let mut breadcrumbs = ctx.last_accessed_files.lock();
        let path_to_record = if final_path.is_absolute() {
            final_path.to_string_lossy().to_string()
        } else {
            path_str.to_string()
        };

        if !breadcrumbs.contains(&path_to_record) {
            breadcrumbs.push(path_to_record);
            if breadcrumbs.len() > BREADCRUMB_CAP {
                breadcrumbs.remove(0);
            }
        }
    }

    /// Read file and record breadcrumb
    async fn read_codebase_file_helper(
        &self,
        ctx: &RunContext,
        final_path: &std::path::Path,
        path_str: &str,
    ) -> Result<String, String> {
        match tokio::fs::read_to_string(final_path).await {
            Ok(content) => {
                self.record_breadcrumb(ctx, final_path, path_str);
                Ok(content)
            }
            Err(e) => Err(format!("{}", e)),
        }
    }

    /// Connects to the global swarm vault.
    #[cfg(feature = "vector-memory")]
    async fn connect_global_vault(&self) -> Option<crate::agent::memory::VectorMemory> {
        let global_vault_path = self
            .state
            .base_dir
            .join("data/intelligence/global_vault.lance");
        match crate::agent::memory::VectorMemory::connect(&global_vault_path.to_string_lossy(), "global").await {
            Ok(vault) => Some(vault),
            Err(e) => {
                tracing::warn!(
                    "⚠️ [Global Vault] Failed to connect to global vault at {:?}: {}",
                    global_vault_path,
                    e
                );
                None
            }
        }
    }

    /// Helper to get Gemini embedding for a string, or log warning on failure.
    #[cfg(feature = "vector-memory")]
    async fn gemini_embed_or_log(&self, ctx: &RunContext, text: &str, operation: &str) -> Option<Vec<f32>> {
        let api_key = ctx.model_config.api_key.clone().unwrap_or_else(|| {
            self.state
                .registry
                .providers
                .get(&ctx.model_config.provider.to_string())
                .and_then(|p| p.api_key.clone())
                .unwrap_or_default()
        });
        if api_key.trim().is_empty() {
            tracing::error!(
                "❌ [Global Vault] No API key configured for provider '{}' (mission {}). Aborting {}.",
                ctx.model_config.provider,
                ctx.mission_id,
                operation
            );
            return None;
        }
        let http_client = self.state.resources.http_client.clone();
        match crate::agent::memory::get_gemini_embedding(&http_client, &api_key, text).await {
            Ok(vec) => Some(vec),
            Err(e) => {
                tracing::warn!(
                    "⚠️ [Global Vault] Failed to generate embedding for {} in mission {}: {}",
                    operation,
                    ctx.mission_id,
                    e
                );
                None
            }
        }
    }
}

fn require_str(
    ctx: &RunContext,
    args: &serde_json::Value,
    key: &str,
    tool_name: &str,
) -> Result<String, ToolExecutionError> {
    args.get(key)
        .ok_or_else(|| {
            ToolExecutionError::AppError(AppError::BadRequest(format!(
                "[Agent {} | Mission {}] Tool '{}' missing required argument '{}'",
                ctx.agent_id, ctx.mission_id, tool_name, key
            )))
        })?
        .as_str()
        .filter(|s| !s.trim().is_empty())
        .map(|s| s.to_string())
        .ok_or_else(|| {
            ToolExecutionError::AppError(AppError::BadRequest(format!(
                "[Agent {} | Mission {}] Tool '{}' argument '{}' must be a non-empty string",
                ctx.agent_id, ctx.mission_id, tool_name, key
            )))
        })
}

fn require_str_opt(
    ctx: &RunContext,
    args: &serde_json::Value,
    key: &str,
    tool_name: &str,
) -> Result<Option<String>, ToolExecutionError> {
    match args.get(key) {
        None => Ok(None),
        Some(v) => {
            if v.is_null() {
                Ok(None)
            } else {
                v.as_str()
                    .map(|s| Some(s.to_string()))
                    .ok_or_else(|| {
                        ToolExecutionError::AppError(AppError::BadRequest(format!(
                            "[Agent {} | Mission {}] Tool '{}' argument '{}' must be a valid string",
                            ctx.agent_id, ctx.mission_id, tool_name, key
                        )))
                    })
            }
        }
    }
}

#[derive(Default, Clone)]
struct BraceCounterState {
    in_string: bool,
    in_char: bool,
    in_block_comment: bool,
    escaped: bool,
    raw_string_hashes: Option<usize>,
    in_template_literal: bool,
    template_literal_brace_depths: Vec<i32>,
    current_depth: i32,
}

fn count_braces_robust(line: &str, state: &mut BraceCounterState) {
    let mut chars = line.char_indices().peekable();

    while let Some((_, c)) = chars.next() {
        if state.in_block_comment {
            if c == '*' {
                if let Some((_, '/')) = chars.peek() {
                    chars.next();
                    state.in_block_comment = false;
                }
            }
            continue;
        }

        if state.raw_string_hashes.is_some() {
            if c == '"' {
                let n = state.raw_string_hashes.unwrap();
                let mut hash_count = 0;
                let mut temp_chars = chars.clone();
                while hash_count < n {
                    if let Some((_, '#')) = temp_chars.peek() {
                        temp_chars.next();
                        hash_count += 1;
                    } else {
                        break;
                    }
                }
                if hash_count == n {
                    for _ in 0..n {
                        chars.next();
                    }
                    state.raw_string_hashes = None;
                }
            }
            continue;
        }

        if state.in_string {
            if state.escaped {
                state.escaped = false;
            } else if c == '\\' {
                state.escaped = true;
            } else if c == '"' {
                state.in_string = false;
            }
            continue;
        }

        if state.in_char {
            if state.escaped {
                state.escaped = false;
            } else if c == '\\' {
                state.escaped = true;
            } else if c == '\'' {
                state.in_char = false;
            }
            continue;
        }

        if state.in_template_literal {
            if state.escaped {
                state.escaped = false;
            } else if c == '\\' {
                state.escaped = true;
            } else if c == '`' {
                state.in_template_literal = false;
            } else if c == '$' {
                if let Some((_, '{')) = chars.peek() {
                    chars.next();
                    state.current_depth += 1;
                    if state.template_literal_brace_depths.len() < MAX_TEMPLATE_LITERAL_DEPTH {
                        state.template_literal_brace_depths.push(state.current_depth);
                    }
                    state.in_template_literal = false;
                }
            }
            continue;
        }

        // We are in normal code state
        state.escaped = false;

        // Check for line comments or block comments
        if c == '/' {
            if let Some((_, '/')) = chars.peek() {
                break; // Line comment ends parsing for this line
            } else if let Some((_, '*')) = chars.peek() {
                chars.next();
                state.in_block_comment = true;
                continue;
            }
        }

        // Check for Rust raw string start
        if c == 'r' {
            let mut hash_count = 0;
            let temp_chars = chars.clone();
            let mut found_raw_start = false;
            for (_, next_c) in temp_chars {
                if next_c == '#' {
                    hash_count += 1;
                } else if next_c == '"' {
                    found_raw_start = true;
                    break;
                } else {
                    break;
                }
            }
            if found_raw_start {
                for _ in 0..(hash_count + 1) {
                    chars.next();
                }
                state.raw_string_hashes = Some(hash_count);
                continue;
            }
        }

        // Normal triggers
        if c == '"' {
            state.in_string = true;
        } else if c == '\'' {
            state.in_char = true;
        } else if c == '`' {
            state.in_template_literal = true;
        } else if c == '{' {
            state.current_depth += 1;
        } else if c == '}' {
            state.current_depth -= 1;
            // Check if we just exited a template literal interpolation block
            if let Some(&depth) = state.template_literal_brace_depths.last() {
                if state.current_depth < depth {
                    state.template_literal_brace_depths.pop();
                    state.in_template_literal = true;
                }
            }
        }
    }
}

// ─────────────────────────────────────────────────────────
//  UNIT TESTS
// ─────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::runner::RunContext;
    use crate::state::AppState;
    use std::sync::Arc;

    async fn setup_test_runner() -> (AgentRunner, Arc<AppState>) {
        let state = Arc::new(AppState::new_minimal_mock().await);
        let runner = AgentRunner::new(state.clone());
        (runner, state)
    }

    #[test]
    fn test_count_braces_robust() {
        let mut state = BraceCounterState::default();
        count_braces_robust("fn hello() {}", &mut state);
        assert_eq!(state.current_depth, 0);

        let mut state = BraceCounterState::default();
        count_braces_robust("fn hello() {", &mut state);
        assert_eq!(state.current_depth, 1);

        let mut state = BraceCounterState::default();
        count_braces_robust("}", &mut state);
        assert_eq!(state.current_depth, -1);

        // String literals
        let mut state = BraceCounterState::default();
        count_braces_robust("let x = \"{ escaped brace }\";", &mut state);
        assert_eq!(state.current_depth, 0);

        // Char literals
        let mut state = BraceCounterState::default();
        count_braces_robust("let x = '{';", &mut state);
        assert_eq!(state.current_depth, 0);

        let mut state = BraceCounterState::default();
        count_braces_robust("let x = '}';", &mut state);
        assert_eq!(state.current_depth, 0);

        // Nested braces
        let mut state = BraceCounterState::default();
        count_braces_robust("let x = '{'; fn nested() {", &mut state);
        assert_eq!(state.current_depth, 1);

        // Comments
        let mut state = BraceCounterState::default();
        count_braces_robust("let x = \"}\"; // comment with {", &mut state);
        assert_eq!(state.current_depth, 0);

        // Block comments spanning lines
        let mut state = BraceCounterState::default();
        count_braces_robust("/* { brace in block comment */", &mut state);
        assert_eq!(state.current_depth, 0);
        assert!(!state.in_block_comment);

        let mut state = BraceCounterState::default();
        count_braces_robust("/* start comment", &mut state);
        assert_eq!(state.current_depth, 0);
        assert!(state.in_block_comment);
        count_braces_robust("brace { still in comment */ }", &mut state);
        assert_eq!(state.current_depth, -1); // only the trailing `}` should count
        assert!(!state.in_block_comment);

        // Rust Raw strings
        let mut state = BraceCounterState::default();
        count_braces_robust("let raw = r#\" { raw content } \"#;", &mut state);
        assert_eq!(state.current_depth, 0);

        // JS template literals with interpolation
        let mut state = BraceCounterState::default();
        count_braces_robust("let msg = `hello ${name}`;", &mut state);
        assert_eq!(state.current_depth, 0);

        let mut state = BraceCounterState::default();
        count_braces_robust("let msg = `nested ${ {a: '{'} }`;", &mut state);
        assert_eq!(state.current_depth, 0);

        // Multi-level template literals nesting
        let mut state = BraceCounterState::default();
        count_braces_robust("let s = `${ a ? `${b}` : c }`;", &mut state);
        assert_eq!(state.current_depth, 0);

        let mut state = BraceCounterState::default();
        count_braces_robust("let s = `a ${ `${'}'}` } b`;", &mut state);
        assert_eq!(state.current_depth, 0);
    }

    #[tokio::test]
    async fn test_extract_symbols_rust() {
        let (runner, _) = setup_test_runner().await;
        let content = r#"
            pub fn run() {}
            struct Data {}
            enum Kind {}
        "#;
        let symbols = runner.extract_symbols(content, "src/main.rs");
        assert_eq!(symbols.len(), 3);
        assert_eq!(symbols[0], "[fn] run");
        assert_eq!(symbols[1], "[struct] Data");
        assert_eq!(symbols[2], "[enum] Kind");
    }

    #[tokio::test]
    async fn test_extract_symbol_body_rust() {
        let (runner, _) = setup_test_runner().await;
        let content = r#"
            fn add(a: i32, b: i32) -> i32 {
                let sum = a + b;
                sum
            }
        "#;
        let body = runner.extract_symbol_body(content, "add", "src/main.rs");
        assert!(body.is_some());
        let body_str = body.unwrap();
        assert!(body_str.contains("fn add"));
        assert!(body_str.contains("let sum = a + b;"));
        assert!(body_str.contains("}"));
    }

    #[tokio::test]
    async fn test_extract_symbol_body_python_indentation() {
        let (runner, _) = setup_test_runner().await;
        let content = r#"
def calc(a, b):
    # This is a tab expanded indent
	val = a + b
	return val

def another_func():
    pass
"#;
        let body = runner.extract_symbol_body(content, "calc", "main.py");
        assert!(body.is_some());
        let body_str = body.unwrap();
        assert!(body_str.contains("def calc"));
        assert!(body_str.contains("val = a + b"));
        assert!(body_str.contains("return val"));
        assert!(!body_str.contains("def another_func"));
    }

    #[tokio::test]
    async fn test_require_path_safety() {
        let (runner, _) = setup_test_runner().await;
        let ctx = RunContext::default();
        
        // Prohibited sensitive paths
        let res = runner.require_path_safety(&ctx, ".env").await;
        assert!(res.is_err());
        assert!(res.unwrap_err().contains("SECURITY BLOCKED"));

        let res = runner.require_path_safety(&ctx, "prod.secret").await;
        assert!(res.is_err());
        assert!(res.unwrap_err().contains("SECURITY BLOCKED"));

        // Valid relative paths
        let res = runner.require_path_safety(&ctx, "src/agent/runner/mission_tools.rs").await;
        assert!(res.is_ok());
    }
}

// Metadata: [mission_tools]

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
use crate::error::AppError;
use crate::agent::runner::tools::error::ToolExecutionError;

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
        let topic = fc
            .args
            .get("topic")
            .and_then(|v| v.as_str())
            .unwrap_or("General");
        let finding = fc
            .args
            .get("finding")
            .and_then(|v| v.as_str())
            .unwrap_or("");

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
            topic,
            finding,
        )
        .await?;

        // Conversational "Echo" to ensure the agent's contribution is visible in the chat bubble
        let echo = format!("**Shared finding on {}:**\n\n{}\n\n", topic, finding);
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
        let report = fc
            .args
            .get("final_report")
            .and_then(|v| v.as_str())
            .unwrap_or("Mission complete.");

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
            // Fix 10: Proactive Semantic Archival before closing mission
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
                let cluster_name = ctx
                    .workspace_root
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();
                let mission_scope_dir = format!(
                    "data/workspaces/{}/missions/{}/scope.lance",
                    cluster_name, ctx.mission_id
                );

                if let Ok(mem) =
                    crate::agent::memory::VectorMemory::connect(&mission_scope_dir, "scope").await
                {
                    let _ = mem
                        .summarize_and_archive(
                            &ctx.mission_id,
                            &self.state.resources.http_client,
                            &api_key,
                            &ctx.model_config.model_id,
                        )
                        .await;
                }
            }

            crate::agent::mission::update_mission(
                &self.state.resources.pool,
                &ctx.mission_id,
                crate::agent::types::MissionStatus::Completed,
                0.0,
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
    pub(crate) async fn handle_pin_mission(
        &self,
        ctx: &RunContext,
        _fc: &crate::agent::types::ToolCall,
        _usage: &mut Option<crate::agent::types::TokenUsage>) -> Result<String, ToolExecutionError> {
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
            &format!("📌 Mission {} pinned for long-term retention.", ctx.mission_id),
            "success",
        );
 
        Ok("(MISSION PINNED: This mission will now bypass the 48h Swarm Reaper cycle.)".to_string())
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
        let query = fc.args.get("query").and_then(|v| v.as_str()).unwrap_or("");
        tracing::info!(
            "🧠 [Memory] Agent {} searching knowledge for: {}",
            ctx.agent_id,
            query
        );

        #[allow(unused_mut)]
        let mut results_text = String::new();

        #[cfg(feature = "vector-memory")]
        {
            let cluster_name = ctx
                .workspace_root
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();
            let mission_scope_dir = format!(
                "data/workspaces/{}/missions/{}/scope.lance",
                cluster_name, ctx.mission_id
            );
            let api_key = ctx.model_config.api_key.clone().unwrap_or_default();
            let http_client = self.state.resources.http_client.clone();

            if let Ok(vec) =
                crate::agent::memory::get_gemini_embedding(&http_client, &api_key, query).await
            {
                if let Ok(mission_mem) =
                    crate::agent::memory::VectorMemory::connect(&mission_scope_dir, "scope").await
                {
                    if let Ok(results) = mission_mem.search_knowledge(vec, 5).await {
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
        let path_str = fc.args.get("path").and_then(|v| v.as_str()).unwrap_or("");

        tracing::info!(
            "🔍 [Sovereignty] Agent {} requesting codebase read: {}",
            ctx.agent_id,
            path_str
        );

        // Security Filter: Prohibit sensitive files
        let sensitive_patterns = [".env", "key", "token", "credential", "secret", "private"];
        if sensitive_patterns
            .iter()
            .any(|p| path_str.to_lowercase().contains(p))
        {
            return Ok(format!(
                "(SECURITY BLOCKED: Access to sensitive file '{}' is prohibited.)",
                path_str
            ));
        }

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

        // Resolve path relative to project root (CWD of the server)
        let root = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));

        // SEC: Centralized path validation with traversal protection
        let target_path = match crate::utils::security::validate_path(&root, path_str) {
            Ok(p) => p,
            Err(e) => {
                return Ok(format!("(SECURITY BLOCKED: {})", e));
            }
        };
        // 🧩 Breadcrumb Resolution: If the direct path fails, try to resolve from recent history.
        let mut final_path = target_path.clone();
        if tokio::fs::metadata(&final_path).await.is_err() {
            let breadcrumbs = ctx.last_accessed_files.lock();
            if let Some(resolved) = breadcrumbs.iter().find(|p| p.ends_with(path_str)) {
                tracing::info!(
                    "🧩 [Context] Resolved ambiguous codebase path '{}' to '{}' via breadcrumbs",
                    path_str,
                    resolved
                );
                final_path = crate::utils::security::SafePath::from_trusted(root.join(resolved));
            }
        }

        // 🧩 Deep Resolution: If it's just a filename, try to find it in the src/ directory.
        if tokio::fs::metadata(&final_path).await.is_err()
            && !path_str.contains("/")
            && !path_str.contains("\\")
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

        match tokio::fs::read_to_string(&final_path).await {
            Ok(content) => {
                // 🥖 Drop a breadcrumb for future sub-agents
                let mut breadcrumbs = ctx.last_accessed_files.lock();
                let path_to_record = if final_path.is_absolute() {
                    final_path.to_string_lossy().to_string()
                } else {
                    path_str.to_string()
                };

                if !breadcrumbs.contains(&path_to_record) {
                    breadcrumbs.push(path_to_record);
                    if breadcrumbs.len() > 10 {
                        breadcrumbs.remove(0);
                    }
                }

                let truncated = self.safe_truncate(&content, 10000);
                Ok(format!("(FILE CONTENT OF {}):\n\n{}", path_str, truncated))
            }
            Err(e) => {
                Ok(format!("(CODEBASE READ FAILED for {}: {})", path_str, e))
            }
        }
    }

    /// Handles `propose_capability`: submits a new skill, workflow, or hook proposal to the oversight system.
    pub(crate) async fn handle_propose_capability(
        &self,
        ctx: &RunContext,
        fc: &crate::agent::types::ToolCall,
        ) -> Result<String, ToolExecutionError> {
        let cap_type_str = fc
            .args
            .get("type")
            .and_then(|v| v.as_str())
            .unwrap_or("skill");
        let name = fc
            .args
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("unnamed");
        let description = fc
            .args
            .get("description")
            .and_then(|v| v.as_str())
            .unwrap_or_default();

        let cap_type = match cap_type_str {
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
        .bind(cap_type_str)
        .bind(name)
        .bind(description)
        .bind(payload_json)
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

    /// Executes a verification script for a skill.
    pub(crate) async fn run_verification_script(
        &self,
        script: &str,
        skill_name: &str,
        params: &serde_json::Value,
        output: &str,
        cwd: &std::path::Path,
    ) -> Result<String, AppError> {
        let mut child = tokio::process::Command::new("powershell")
            .arg("-Command")
            .arg(script)
            .env("SKILL_NAME", skill_name)
            .env("SKILL_PARAMS", params.to_string())
            .env("SKILL_OUTPUT", output)
            .current_dir(cwd)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .map_err(|e| AppError::InternalServerError(format!("Failed to spawn verification script: {}", e)))?;

        let status = child.wait().await.map_err(|e: std::io::Error| AppError::Io(e))?;
        let output = child.wait_with_output().await.map_err(|e: std::io::Error| AppError::Io(e))?;

        if status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
        } else {
            let err = String::from_utf8_lossy(&output.stderr).trim().to_string();
            Err(AppError::InternalServerError(format!(
                "Verification script failed ({}): {}",
                status,
                err
            )))
        }
    }
    /// Handles `list_file_symbols`: parses a file to list functions, classes, and variables.
    pub(crate) async fn handle_list_file_symbols(
        &self,
        ctx: &RunContext,
        fc: &crate::agent::types::ToolCall,
        ) -> Result<String, ToolExecutionError> {
        let path_str = fc.args.get("path").and_then(|v| v.as_str()).unwrap_or("");
        if path_str.is_empty() {
            return Ok("(LIST SYMBOLS FAILED: 'path' argument is missing)".to_string());
        }

        let adapter = &ctx.fs_adapter;
        match adapter.read_file(path_str).await {
            Ok(content) => {
                let symbols = self.extract_symbols(&content, path_str);
                if symbols.is_empty() {
                    Ok(format!("(No recognizable symbols found in {})", path_str))
                } else {
                    let symbol_list = symbols.join("\n");
                    Ok(format!("(SYMBOLS IN {}):\n\n{}", path_str, symbol_list))
                }
            }
            Err(e) => {
                Ok(format!("(LIST SYMBOLS FAILED for {}: {})", path_str, e))
            }
        }
    }

    /// Handles `get_symbol_body`: extracts the implementation of a specific symbol from a file.
    pub(crate) async fn handle_get_symbol_body(
        &self,
        ctx: &RunContext,
        fc: &crate::agent::types::ToolCall,
        ) -> Result<String, ToolExecutionError> {
        let path_str = fc.args.get("path").and_then(|v| v.as_str()).unwrap_or("");
        let symbol_name = fc.args.get("symbol").and_then(|v| v.as_str()).unwrap_or("");

        if path_str.is_empty() || symbol_name.is_empty() {
            return Ok("(GET SYMBOL FAILED: 'path' and 'symbol' arguments are required)".to_string());
        }

        let adapter = &ctx.fs_adapter;
        match adapter.read_file(path_str).await {
            Ok(content) => {
                if let Some(body) = self.extract_symbol_body(&content, symbol_name, path_str) {
                    Ok(format!("(BODY OF SYMBOL '{}' IN {}):\n\n{}", symbol_name, path_str, body))
                } else {
                    Ok(format!("(SYMBOL '{}' NOT FOUND in {})", symbol_name, path_str))
                }
            }
            Err(e) => {
                Ok(format!("(GET SYMBOL FAILED for {}: {})", path_str, e))
            }
        }
    }

    /// Internal helper: Extracts a list of symbols using regex patterns based on file extension.
    fn extract_symbols(&self, content: &str, path: &str) -> Vec<String> {
        let ext = std::path::Path::new(path).extension().and_then(|s| s.to_str()).unwrap_or("");
        let mut symbols = Vec::new();

        match ext {
            "rs" => {
                let re = regex::Regex::new(r"(?m)^[ \t]*(?:pub(?:\(.*\))?\s+)?(?:async\s+)?(fn|struct|enum|trait|type|const|static)\s+([a-zA-Z_][a-zA-Z0-9_]*)").unwrap();
                for cap in re.captures_iter(content) {
                    symbols.push(format!("[{}] {}", &cap[1], &cap[2]));
                }
            }
            "ts" | "js" | "tsx" | "jsx" => {
                let re = regex::Regex::new(r"(?m)^[ \t]*(?:export\s+)?(?:async\s+)?(function|class|type|interface|const|let|var)\s+([a-zA-Z_][a-zA-Z0-9_]*)").unwrap();
                for cap in re.captures_iter(content) {
                    symbols.push(format!("[{}] {}", &cap[1], &cap[2]));
                }
            }
            "py" => {
                let re = regex::Regex::new(r"(?m)^[ \t]*(def|class)\s+([a-zA-Z_][a-zA-Z0-9_]*)").unwrap();
                for cap in re.captures_iter(content) {
                    symbols.push(format!("[{}] {}", &cap[1], &cap[2]));
                }
            }
            _ => {
                // Fallback for unknown languages: search for common patterns
                let re = regex::Regex::new(r"(?m)^[ \t]*(?:function|class|def|fn)\s+([a-zA-Z_][a-zA-Z0-9_]*)").unwrap();
                for cap in re.captures_iter(content) {
                    symbols.push(format!("[symbol] {}", &cap[1]));
                }
            }
        }
        symbols
    }

    /// Internal helper: Extracts the body of a specific symbol.
    fn extract_symbol_body(&self, content: &str, symbol: &str, path: &str) -> Option<String> {
        let lines: Vec<&str> = content.lines().collect();
        let ext = std::path::Path::new(path).extension().and_then(|s| s.to_str()).unwrap_or("");

        // Find the line where the symbol is defined
        let start_idx = match ext {
            "py" => lines.iter().position(|l| l.contains(&format!("def {}", symbol)) || l.contains(&format!("class {}", symbol))),
            "rs" => lines.iter().position(|l| l.contains(&format!("fn {}", symbol)) || l.contains(&format!("struct {}", symbol)) || l.contains(&format!("enum {}", symbol)) || l.contains(&format!("trait {}", symbol))),
            _ => lines.iter().position(|l| l.contains(&format!("function {}", symbol)) || l.contains(&format!("class {}", symbol)) || l.contains(&format!("const {}", symbol))),
        };

        if let Some(start) = start_idx {
            let mut body = Vec::new();
            let mut brace_count = 0;
            let mut found_start = false;
            let mut indent_level = None;

            for line in &lines[start..] {
                body.push(*line);
                
                if ext == "py" {
                    // Python indentation-based blocks
                    let current_indent = line.chars().take_while(|c| c.is_whitespace()).count();
                    if !line.trim().is_empty() {
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
                    brace_count += line.matches('{').count();
                    brace_count -= line.matches('}').count();
                    
                    if line.contains('{') { found_start = true; }
                    
                    if found_start && brace_count == 0 {
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
        let target_agent_id = fc
            .args
            .get("agent_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| AppError::BadRequest("Missing agent_id".to_string()))?;
        let instruction = fc
            .args
            .get("instruction")
            .and_then(|v| v.as_str())
            .ok_or_else(|| AppError::BadRequest("Missing instruction".to_string()))?;

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
            target_agent_id,
            instruction,
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
        let reviewer_id = fc
            .args
            .get("reviewer_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| AppError::BadRequest("Missing reviewer_id".to_string()))?;
        let content = fc
            .args
            .get("content")
            .and_then(|v| v.as_str())
            .ok_or_else(|| AppError::BadRequest("Missing content".to_string()))?;
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
            reviewer_id,
            content,
            criteria,
        )
        .await?;

        Ok(format!("Audit request [{}] sent to {}. Check back later for feedback.", id, reviewer_id))
    }

    /// Handles `submit_peer_review`: provides feedback on a request.
    pub(crate) async fn handle_submit_peer_review(
        &self,
        ctx: &RunContext,
        fc: &crate::agent::types::ToolCall,
        ) -> Result<String, ToolExecutionError> {
        let request_id = fc
            .args
            .get("request_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| AppError::BadRequest("Missing request_id".to_string()))?;
        let feedback = fc
            .args
            .get("feedback")
            .and_then(|v| v.as_str())
            .ok_or_else(|| AppError::BadRequest("Missing feedback".to_string()))?;
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
            request_id,
            feedback,
            status,
        )
        .await?;

        Ok(format!("Peer review for [{}] submitted. Feedback: {}", request_id, feedback))
    }

    /// Handles `archive_to_global_vault`: persists a mission nugget to the global swarm vault.
    pub(crate) async fn handle_archive_to_global_vault(
        &self,
        ctx: &RunContext,
        fc: &crate::agent::types::ToolCall,
        ) -> Result<String, ToolExecutionError> {
        let topic = fc.args.get("topic").and_then(|v| v.as_str()).unwrap_or("General");
        tracing::info!("🏛️ [Global Vault] Agent {} archiving nugget on {}.", ctx.agent_id, topic);
        #[cfg(feature = "vector-memory")]
        {
            let content = fc.args.get("content").and_then(|v| v.as_str()).unwrap_or("");
            let api_key = ctx.model_config.api_key.clone().unwrap_or_else(|| {
                self.state.registry.providers.get(&ctx.model_config.provider.to_string())
                    .and_then(|p| p.api_key.clone())
                    .unwrap_or_default()
            });
            let http_client = self.state.resources.http_client.clone();
            let global_vault_path = self.state.base_dir.join("data/intelligence/global_vault.lance");

            if let Ok(vec) = crate::agent::memory::get_gemini_embedding(&http_client, &api_key, content).await {
                if let Ok(vault) = crate::agent::memory::VectorMemory::connect(&global_vault_path.to_string_lossy(), "global").await {
                    let metadata = serde_json::json!({
                        "agent_id": ctx.agent_id,
                        "mission_id": ctx.mission_id,
                        "topic": topic,
                        "timestamp": chrono::Utc::now().to_rfc3339()
                    });
                    if let Ok(_) = vault.add_record(content, vec, Some(metadata)).await {
                        self.broadcast_agent(ctx, &format!("🏛️ Global Vault: nugget archived on {}", topic), "success");
                        return Ok(format!("(GLOBAL ARCHIVE SUCCESS): Nugget on '{}' added to the swarm intelligence vault.", topic));
                    }
                }
            }
        }

        Ok("(GLOBAL ARCHIVE FAILED): Ensure vector-memory is enabled and API keys are valid.".to_string())
    }

    /// Handles `search_global_vault`: performs a semantic search across all mission histories.
    pub(crate) async fn handle_search_global_vault(
        &self,
        ctx: &RunContext,
        fc: &crate::agent::types::ToolCall,
        ) -> Result<String, ToolExecutionError> {
        let query = fc.args.get("query").and_then(|v| v.as_str()).unwrap_or("");
        tracing::info!("🏛️ [Global Vault] Agent {} searching global vault for: {}", ctx.agent_id, query);

        #[cfg(feature = "vector-memory")]
        {
            let api_key = ctx.model_config.api_key.clone().unwrap_or_else(|| {
                self.state.registry.providers.get(&ctx.model_config.provider.to_string())
                    .and_then(|p| p.api_key.clone())
                    .unwrap_or_default()
            });
            let http_client = self.state.resources.http_client.clone();
            let global_vault_path = self.state.base_dir.join("data/intelligence/global_vault.lance");

            if let Ok(vec) = crate::agent::memory::get_gemini_embedding(&http_client, &api_key, query).await {
                if let Ok(vault) = crate::agent::memory::VectorMemory::connect(&global_vault_path.to_string_lossy(), "global").await {
                    if let Ok(results) = vault.search_knowledge(vec, 5).await {
                        if results.is_empty() {
                            return Ok(format!("(GLOBAL SEARCH): No relevant intelligence found for '{}'.", query));
                        } else {
                            return Ok(format!("(GLOBAL INTELLIGENCE RETRIEVED for '{}'):\n\n{}", query, results.join("\n\n----- \n\n")));
                        }
                    }
                }
            }
        }

        Ok("(GLOBAL SEARCH FAILED): Ensure vector-memory is enabled and API keys are valid.".to_string())
    }
}

// Metadata: [mission_tools]

// Metadata: [mission_tools]

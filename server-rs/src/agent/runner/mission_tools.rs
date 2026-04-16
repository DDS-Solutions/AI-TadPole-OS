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

impl AgentRunner {
    /// Handles `share_finding`: persists a finding to the swarm context.
    pub(crate) async fn handle_share_finding(
        &self,
        ctx: &RunContext,
        fc: &crate::agent::types::GeminiFunctionCall,
        output_text: &mut String,
    ) -> anyhow::Result<()> {
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
        *output_text = format!("{}{}", echo, output_text);

        Ok(())
    }

    /// Handles `complete_mission`: marks the mission as completed after oversight.
    pub(crate) async fn handle_complete_mission(
        &self,
        ctx: &RunContext,
        fc: &crate::agent::types::GeminiFunctionCall,
        output_text: &mut String,
    ) -> anyhow::Result<()> {
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
                crate::agent::types::ToolCall {
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
            .await;

        if approved {
            // Fix 10: Proactive Semantic Archival before closing mission
            #[cfg(feature = "vector-memory")]
            {
                let api_key = ctx.model_config.api_key.clone().unwrap_or_else(|| {
                    self.state
                        .registry
                        .providers
                        .get(&ctx.model_config.provider)
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
            *output_text = format!(
                "🏁 **MISSION ARCHIVE REPORT**\n\
                 Mission ID: {}\n\
                 Status: SUCCESS\n\n\
                 **FINAL SUMMARY:**\n\
                 {}\n\n\
                 **END OF MISSION REPORT**",
                ctx.mission_id, report
            );
        } else {
            *output_text = format!("(Mission completion REJECTED) {}", output_text);
        }

        Ok(())
    }

    /// Handles `search_mission_knowledge`: vector search across LanceDB memory scope.
    pub(crate) async fn handle_search_mission_knowledge(
        &self,
        ctx: &RunContext,
        fc: &crate::agent::types::GeminiFunctionCall,
        output_text: &mut String,
    ) -> anyhow::Result<()> {
        let query = fc.args.get("query").and_then(|v| v.as_str()).unwrap_or("");
        tracing::info!(
            "🧠 [Memory] Agent {} searching knowledge for: {}",
            ctx.agent_id,
            query
        );

        #[cfg(feature = "vector-memory")]
        let mut results_text = String::new();

        #[cfg(not(feature = "vector-memory"))]
        let results_text = String::new();

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

            *output_text = format!(
                "(RESOURCE NOT FOUND: No relevant shared findings found for '{}'. {}) {}",
                query, hint, output_text
            );
        } else {
            *output_text = format!(
                "(SEARCH RESULTS FOR '{}'):\n{}\n{}",
                query, results_text, output_text
            );
        }

        Ok(())
    }

    /// Handles `read_codebase_file`: allows reading files from the project root with oversight.
    /// This bypasses the standard workspace sandbox but enforces strict security filters.
    pub(crate) async fn handle_read_codebase_file(
        &self,
        ctx: &RunContext,
        fc: &crate::agent::types::GeminiFunctionCall,
        output_text: &mut String,
    ) -> anyhow::Result<()> {
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
            *output_text = format!(
                "(SECURITY BLOCKED: Access to sensitive file '{}' is prohibited.)",
                path_str
            );
            return Ok(());
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
                crate::agent::types::ToolCall {
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
            .await;

        if !approved {
            *output_text = format!("(Codebase read REJECTED by Oversight) {}", output_text);
            return Ok(());
        }

        // Resolve path relative to project root (CWD of the server)
        let root = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));

        // SEC: Centralized path validation with traversal protection
        let target_path = match crate::utils::security::validate_path(&root, path_str) {
            Ok(p) => p,
            Err(e) => {
                *output_text = format!("(SECURITY BLOCKED: {})", e);
                return Ok(());
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
                final_path = root.join(resolved);
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
                    final_path = alt_path;
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
                *output_text = format!("(FILE CONTENT OF {}):\n\n{}", path_str, truncated);
            }
            Err(e) => {
                *output_text = format!("(CODEBASE READ FAILED for {}: {})", path_str, e);
            }
        }

        Ok(())
    }

    /// Handles `propose_capability`: submits a new skill, workflow, or hook proposal to the oversight system.
    pub(crate) async fn handle_propose_capability(
        &self,
        ctx: &RunContext,
        fc: &crate::agent::types::GeminiFunctionCall,
        output_text: &mut String,
    ) -> anyhow::Result<()> {
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
                    *output_text = "(Proposal REJECTED: Skill proposals must include 'execution_command' and 'schema' arguments.)".to_string();
                    return Ok(());
                }
            }
            crate::agent::types::SkillType::Workflow => {
                if fc.args.get("content").is_none() {
                    *output_text = "(Proposal REJECTED: Workflow proposals must include a 'content' argument.)".to_string();
                    return Ok(());
                }
            }
            crate::agent::types::SkillType::Hook => {
                if fc.args.get("hook_type").is_none() || fc.args.get("content").is_none() {
                    *output_text = "(Proposal REJECTED: Hook proposals must include 'hook_type' and 'content' arguments.)".to_string();
                    return Ok(());
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
        .await?;

        self.broadcast_agent(
            ctx,
            &format!(
                "💡 Oversight: new capability proposal '{}' ({}) submitted for review.",
                name, cap_type_str
            ),
            "warning",
        );

        // Non-blocking response: The agent can proceed with other tasks while approval is pending.
        *output_text = format!(
            "(CAPABILITY PROPOSAL SUBMITTED): The proposed {} '{}' has been queued for human oversight (Proposal ID: {}). You may continue your mission while the Governance Hub reviews this capability expansion.",
            cap_type_str, name, proposal_id
        );

        Ok(())
    }

    /// Executes a verification script for a skill.
    pub(crate) async fn run_verification_script(
        &self,
        script: &str,
        skill_name: &str,
        params: &serde_json::Value,
        output: &str,
        cwd: &std::path::Path,
    ) -> anyhow::Result<String> {
        let mut child = tokio::process::Command::new("powershell")
            .arg("-Command")
            .arg(script)
            .env("SKILL_NAME", skill_name)
            .env("SKILL_PARAMS", params.to_string())
            .env("SKILL_OUTPUT", output)
            .current_dir(cwd)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()?;

        let status = child.wait().await?;
        let output = child.wait_with_output().await?;

        if status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
        } else {
            let err = String::from_utf8_lossy(&output.stderr).trim().to_string();
            Err(anyhow::anyhow!(
                "Verification script failed ({}): {}",
                status,
                err
            ))
        }
    }
}

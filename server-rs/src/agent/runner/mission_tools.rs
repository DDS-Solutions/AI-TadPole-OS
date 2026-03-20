//! Mission-scoped interaction and lifecycle control tools.
//!
//! Provides agents with the ability to share findings, search long-term memory,
//! propose new system capabilities, and signal mission completion.

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
        self.broadcast_sys(
            &format!("📢 Swarm: {} added context for {}", ctx.name, topic),
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
            .get("finalReport")
            .and_then(|v| v.as_str())
            .unwrap_or("Mission complete.");

        tracing::info!(
            "🏁 [Mission] Agent {} requesting completion...",
            ctx.agent_id
        );
        self.broadcast_sys(
            &format!(
                "🏁 Oversight: {} finished work. Reviewing final report...",
                ctx.name
            ),
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
            let api_key = ctx.model_config.api_key.clone().unwrap_or_else(|| {
                self.state.registry
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

            crate::agent::mission::update_mission(
                &self.state.resources.pool,
                &ctx.mission_id,
                crate::agent::types::MissionStatus::Completed,
                0.0,
            )
            .await?;
            self.broadcast_sys(
                &format!("✅ Mission {} COMPLETED and archived.", ctx.mission_id),
                "success",
            );
            *output_text = format!(
                "**MISSION COMPLETED & ARCHIVED:**\n\n{}\n\n{}",
                report, output_text
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

        let mut results_text = String::new();

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

        if results_text.is_empty() {
            *output_text = format!(
                "(RESOURCE NOT FOUND: No relevant shared findings found for '{}'. This query might be reference a physical file or keyword in the workspace. Since you have technical tools, you should now use 'list_files' or 'grep_search' to locate the target and then 'read_file' or 'read_codebase_file' to inspect it directly.) {}",
                query, output_text
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
        if sensitive_patterns.iter().any(|p| path_str.to_lowercase().contains(p)) {
            *output_text = format!("(SECURITY BLOCKED: Access to sensitive file '{}' is prohibited.)", path_str);
            return Ok(());
        }

        self.broadcast_sys(
            &format!("🔍 Oversight: {} wants to read codebase file: {}. Review required.", ctx.name, path_str),
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
                    description: format!("Reading codebase file for architectural analysis: {}", path_str),
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
        let mut target_path = root.clone();
        target_path.push(path_str);
        
        // SEC: Basic path traversal protection (no '..')
        if path_str.contains("..") {
             *output_text = format!("(SECURITY BLOCKED: Path traversal detected in '{}'.)", path_str);
             return Ok(());
        }
        // 🧩 Breadcrumb Resolution: If the direct path fails, try to resolve from recent history.
        let mut final_path = target_path.clone();
        if tokio::fs::metadata(&final_path).await.is_err() {
            let breadcrumbs = ctx.last_accessed_files.lock().unwrap();
            if let Some(resolved) = breadcrumbs.iter().find(|p| p.ends_with(path_str)) {
                tracing::info!("🧩 [Context] Resolved ambiguous codebase path '{}' to '{}' via breadcrumbs", path_str, resolved);
                final_path = root.join(resolved);
            }
        }

        // 🧩 Deep Resolution: If it's just a filename, try to find it in the src/ directory.
        if tokio::fs::metadata(&final_path).await.is_err() && !path_str.contains("/") && !path_str.contains("\\") {
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
                let mut breadcrumbs = ctx.last_accessed_files.lock().unwrap();
                let path_to_record = if final_path.is_absolute() {
                    final_path.to_string_lossy().to_string()
                } else {
                    path_str.to_string()
                };

                if !breadcrumbs.contains(&path_to_record) {
                    breadcrumbs.push(path_to_record);
                    if breadcrumbs.len() > 10 { breadcrumbs.remove(0); }
                }

                let truncated = if content.len() > 8000 {
                    format!("{}... [TRUNCATED]", &content[..8000])
                } else {
                    content
                };
                *output_text = format!("(CODEBASE FILE {}):\n\n{}", path_str, truncated);
            }
            Err(e) => {
                *output_text = format!("(CODEBASE READ FAILED for {}: {})", path_str, e);
            }
        }

        Ok(())
    }

    /// Handles `propose_capability`: submits a new skill or workflow proposal to the Oversight Gate.
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

        let cap_type = if cap_type_str == "workflow" {
            crate::agent::types::CapabilityType::Workflow
        } else {
            crate::agent::types::CapabilityType::Skill
        };

        if cap_type == crate::agent::types::CapabilityType::Skill {
            if fc.args.get("executionCommand").is_none() || fc.args.get("schema").is_none() {
                *output_text = format!("(Proposal REJECTED: Skill proposals must include 'executionCommand' and 'schema' arguments.) {}", output_text);
                return Ok(());
            }
        } else if fc.args.get("content").is_none() {
            *output_text = format!(
                "(Proposal REJECTED: Workflow proposals must include a 'content' argument.) {}",
                output_text
            );
            return Ok(());
        }

        let proposal = crate::agent::types::CapabilityProposal {
            r#type: cap_type.clone(),
            name: name.to_string(),
            description: fc
                .args
                .get("description")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string(),
            execution_command: fc
                .args
                .get("executionCommand")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            schema: fc.args.get("schema").cloned(),
            content: fc
                .args
                .get("content")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            full_instructions: fc
                .args
                .get("fullInstructions")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            negative_constraints: fc
                .args
                .get("negativeConstraints")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(|s| s.to_string()))
                        .collect()
                }),
            verification_script: fc
                .args
                .get("verificationScript")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
        };

        tracing::info!(
            "💡 [Sovereignty] Agent {} proposing a new capability: {} ({})",
            ctx.agent_id,
            name,
            cap_type_str
        );
        self.broadcast_sys(
            &format!(
                "💡 Oversight: {} wants to expand our swarm with a new {}: {}. Review required.",
                ctx.name, cap_type_str, name
            ),
            "warning",
        );

        let approved = self
            .submit_capability_oversight(
                proposal.clone(),
                Some(ctx.mission_id.clone()),
                &ctx.agent_id,
                &ctx.department,
            )
            .await;

        if approved {
            if cap_type == crate::agent::types::CapabilityType::Skill {
                let skill = crate::agent::capabilities::SkillDefinition {
                    id: None,
                    name: proposal.name.clone(),
                    description: proposal.description.clone(),
                    execution_command: proposal.execution_command.unwrap_or_default(),
                    schema: proposal.schema.unwrap_or_else(|| serde_json::json!({})),
                    oversight_required: true, 
                    doc_url: None,
                    tags: Some(vec!["agent-generated".to_string()]),
                    full_instructions: fc
                        .args
                        .get("fullInstructions")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string()),
                    negative_constraints: fc
                        .args
                        .get("negativeConstraints")
                        .and_then(|v| v.as_array())
                        .map(|arr| {
                            arr.iter()
                                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                                .collect()
                        }),
                    verification_script: fc
                        .args
                        .get("verificationScript")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string()),
                };
                if let Err(e) = self.state.registry.capabilities.save_skill(skill).await {
                    tracing::error!("Failed to save proposed skill {}: {}", name, e);
                    *output_text = format!(
                        "(Proposal APPROVED but failed to save skill: {}) {}",
                        e, output_text
                    );
                    return Ok(());
                }
            } else {
                let workflow = crate::agent::capabilities::WorkflowDefinition {
                    id: None,
                    name: proposal.name.clone(),
                    content: proposal.content.unwrap_or_default(),
                    doc_url: None,
                    tags: Some(vec!["agent-generated".to_string()]),
                };
                if let Err(e) = self.state.registry.capabilities.save_workflow(workflow).await {
                    tracing::error!("Failed to save proposed workflow {}: {}", name, e);
                    *output_text = format!(
                        "(Proposal APPROVED but failed to save workflow: {}) {}",
                        e, output_text
                    );
                    return Ok(());
                }
            }

            *output_text = format!(
                "(Successfully PROPOSED and APPROVED new {}: {}) {}",
                cap_type_str, name, output_text
            );
        } else {
            *output_text = format!(
                "(Capability Proposal for {} REJECTED by Oversight) {}",
                name, output_text
            );
        }

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

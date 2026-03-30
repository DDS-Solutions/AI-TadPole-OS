//! Secure tool dispatch and multi-layered execution guardrails.
//!
//! This module acts as the "Intelligence Layer" cross-cutting concern,
//! ensuring every tool call is audited, scanned for malicious patterns,
//! and validated against Skill-Based Security (CBS) policies.

use super::{AgentRunner, RunContext};
use crate::agent::hooks::HookContext;

impl AgentRunner {
    // ─────────────────────────────────────────────────────────
    //  TOOL EXECUTION (The "Intelligence" Layer)
    // ─────────────────────────────────────────────────────────

    /// Dispatches a function call to the appropriate tool handler.
    #[tracing::instrument(
        name = "execute_tool",
        skip(self, ctx, output_text, usage, _user_message),
        fields(
            agent_id = %ctx.agent_id,
            mission_id = %ctx.mission_id,
            tool_name = %fc.name
        )
    )]
    pub(crate) async fn execute_tool(
        &self,
        ctx: &RunContext,
        fc: &crate::agent::types::GeminiFunctionCall,
        output_text: &mut String,
        usage: &mut Option<crate::agent::types::TokenUsage>,
        _user_message: &str,
    ) -> anyhow::Result<Option<String>> {
        let hook_ctx = HookContext {
            agent_id: ctx.agent_id.clone(),
            mission_id: Some(ctx.mission_id.clone()),
            skill: fc.name.clone(),
        };

        // 🌐 [CBS] Skill-Based Security Allowlist
        if let Some(agent) = self.state.registry.agents.get(&ctx.agent_id) {
            let allowed_skills = &agent.value().skills;
            let is_builtin = matches!(
                fc.name.as_str(),
                "spawn_subagent"
                    | "issue_alpha_directive"
                    | "share_finding"
                    | "query_financial_logs"
                    | "complete_mission"
                    | "propose_skill"
                    | "archive_to_vault"
                    | "notify_discord"
                    | "fetch_url"
                    | "read_file"
                    | "write_file"
                    | "list_files"
                    | "delete_file"
                    | "search_mission_knowledge"
                    | "read_codebase_file"
                    | "list_file_symbols"
                    | "get_symbol_body"
                    | "update_working_memory"
                    | "script_builder"
            );

            if !is_builtin && !allowed_skills.is_empty() && !allowed_skills.contains(&fc.name) {
                tracing::warn!(
                    "🛡️ [CBS] Agent {} attempted unauthorized skill: {}",
                    ctx.agent_id,
                    fc.name
                );
                self.broadcast_sys(
                    &format!(
                        "🛡️ CBS: {} attempted unauthorized skill: {}",
                        ctx.name, fc.name
                    ),
                    "error",
                    Some(ctx.mission_id.clone()),
                );
                *output_text = format!(
                    "(TOOL EXECUTION BLOCKED: Skill '{}' not in agent allowlist)",
                    fc.name
                );
                return Ok(Some(output_text.clone()));
            }

            // 🛡️ [Hierarchy Guard] Enforce strategic delegation for CEO/COO
            if matches!(fc.name.as_str(), "spawn_subagent" | "recruit_specialist") {
                if ctx.agent_id == "1" {
                    tracing::warn!("🛡️ [Hierarchy Guard] CEO (ID: 1) blocked from spawning specialists directly.");
                    self.broadcast_sys("🛡️ Hierarchy Guard: CEO blocked from direct worker recruitment. Use 'issue_alpha_directive' instead.", "warning", Some(ctx.mission_id.clone()));
                    *output_text = "(HIERARCHY BLOCKED: As CEO, you are prohibited from direct worker recruitment. You MUST use 'issue_alpha_directive' to delegate complex missions to the COO.)".to_string();
                    return Ok(Some(output_text.clone()));
                }
                if ctx.agent_id == "2" {
                    let target = fc
                        .args
                        .get("agent_id")
                        .or_else(|| fc.args.get("agentId"))
                        .and_then(|v| v.as_str())
                        .unwrap_or("");
                    if target != "alpha" {
                        tracing::warn!("🛡️ [Hierarchy Guard] COO (ID: 2) blocked from spawning specialist '{}' directly.", target);
                        self.broadcast_sys("🛡️ Hierarchy Guard: COO blocked from direct worker recruitment. Use Alpha Node commander instead.", "warning", Some(ctx.mission_id.clone()));
                        *output_text = "(HIERARCHY BLOCKED: As COO, you are prohibited from direct worker recruitment. You MUST recruit an Alpha Node (ID: alpha) to serve as Swarm Mission Commander.)".to_string();
                        return Ok(Some(output_text.clone()));
                    }
                }
            }
        }

        // 🔒 [Security Gate] Skill Manifest Validation & Agent-Level Oversight
        let mut trigger_oversight = false;
        let mut oversight_reason = String::new();

        {
            // 🛡️ [Security Gate] Skill Manifest Validation (Built-in & Dynamic)
            let mut manifest_requires = false;

            // Check built-in manifests
            if let Some(manifest) = self.state.registry.skill_registry.get(&fc.name) {
                if manifest.requires_oversight {
                    manifest_requires = true;
                }
            }
            // Check dynamic script skills
            if !manifest_requires {
                if let Some(skill) = self.state.registry.skills.skills.get(&fc.name) {
                    if skill.oversight_required {
                        manifest_requires = true;
                    }
                }
            }

            if manifest_requires {
                trigger_oversight = true;
                oversight_reason = format!("Security Gate triggered by manifest for: {}", fc.name);
            }

            // [Agent-Level Oversight]
            if let Some(agent) = self.state.registry.agents.get(&ctx.agent_id) {
                if agent.value().requires_oversight {
                    trigger_oversight = true;
                    oversight_reason = format!("Mandatory oversight enabled for agent: {}", ctx.name);
                }
            }
        } // Guards are dropped here

        if trigger_oversight {
            tracing::info!(
                "🔒 [Security Gate] Oversight triggered for tool '{}': {}",
                fc.name,
                oversight_reason
            );
            self.broadcast_sys(
                &format!(
                    "🔒 Security Gate: '{}' requires explicit approval.",
                    fc.name
                ),
                "warning",
                Some(ctx.mission_id.clone()),
            );

            let approved = self
                .submit_oversight(
                    crate::agent::types::ToolCall {
                        id: uuid::Uuid::new_v4().to_string(),
                        agent_id: ctx.agent_id.clone(),
                        mission_id: Some(ctx.mission_id.clone()),
                        skill: fc.name.clone(),
                        params: fc.args.clone(),
                        department: ctx.department.clone(),
                        description: oversight_reason,
                        timestamp: chrono::Utc::now().to_rfc3339(),
                    },
                    Some(ctx.mission_id.clone()),
                )
                .await;

            if !approved {
                *output_text = format!(
                    "(Execution of {} REJECTED by Oversight Security Gate) {}",
                    fc.name, output_text
                );
                return Ok(Some(output_text.clone()));
            }
        }

        // 🛡️ [Guardrail] Pre-tool Lifecycle Hook
        self.state
            .registry
            .hooks
            .trigger_hook("pre-tool", &hook_ctx, &fc.args)
            .await?;

        // 📝 [Audit] Record action attempt in Merkle Hash Chain
        let audit_params = self
            .state
            .security
            .secret_redactor
            .redact(&serde_json::to_string(&fc.args).unwrap_or_default());
        let audit_trail = self.state.security.audit_trail.clone();
        let agent_id = ctx.agent_id.clone();
        let mission_id = ctx.mission_id.clone();
        let user_id = ctx.user_id.clone();
        let action = fc.name.clone();
        tokio::spawn(async move {
            let _ = audit_trail
                .record(
                    &agent_id,
                    Some(&mission_id),
                    user_id.as_deref(),
                    &action,
                    &audit_params,
                )
                .await;
        });

        // 📋 [Proactive Security] Shell Safety Scanner (Script Analysis)
        let args_str = serde_json::to_string(&fc.args).unwrap_or_default();
        if let crate::security::scanner::ScannerResult::Risky(reason) =
            self.state.security.shell_scanner.scan(&args_str)
        {
            tracing::warn!(
                "🛡️ [Security] Blocked risky tool call for '{}': {}",
                fc.name,
                reason
            );
            self.broadcast_sys(
                &format!(
                    "🛡️ Security: Blocked risky tool call for '{}': {}",
                    fc.name, reason
                ),
                "error",
                Some(ctx.mission_id.clone()),
            );
            *output_text = format!("(TOOL EXECUTION BLOCKED FOR SECURITY: {})", reason);
            return Ok(Some(output_text.clone()));
        }

        let result: anyhow::Result<Option<String>> = match fc.name.as_str() {
            "fetch_url" => {
                self.handle_fetch_url(ctx, fc, output_text, usage).await?;
                Ok(None)
            }
            "read_file" => {
                self.handle_read_file(ctx, fc, output_text, usage).await?;
                Ok(None)
            }
            "write_file" => {
                self.handle_write_file(ctx, fc, output_text).await?;
                Ok(None)
            }
            "list_files" => {
                self.handle_list_files(ctx, fc, output_text, usage).await?;
                Ok(None)
            }
            "delete_file" => {
                self.handle_delete_file(ctx, fc, output_text).await?;
                Ok(None)
            }
            "spawn_subagent" => {
                self.handle_spawn_subagent(ctx, fc, output_text, usage)
                    .await?;
                Ok(None)
            }
            "issue_alpha_directive" => {
                let res = self.handle_alpha_directive(ctx, fc).await?;
                Ok(Some(res))
            }
            "share_finding" => {
                self.handle_share_finding(ctx, fc, output_text).await?;
                Ok(None)
            }
            "query_financial_logs" => {
                self.handle_query_financial_logs(ctx, fc, output_text, usage)
                    .await?;
                Ok(None)
            }
            "archive_to_vault" => {
                self.handle_archive_to_vault(ctx, fc, output_text).await?;
                Ok(None)
            }
            "notify_discord" => {
                self.handle_notify_discord(ctx, fc, output_text).await?;
                Ok(None)
            }
            "complete_mission" => {
                self.handle_complete_mission(ctx, fc, output_text).await?;
                Ok(None)
            }
            "propose_skill" => {
                self.handle_propose_skill(ctx, fc, output_text).await?;
                Ok(None)
            }
            "search_mission_knowledge" => {
                self.handle_search_mission_knowledge(ctx, fc, output_text)
                    .await?;
                Ok(None)
            }
            "read_codebase_file" => {
                self.handle_read_codebase_file(ctx, fc, output_text).await?;
                Ok(None)
            }
            "update_working_memory" => {
                self.handle_update_working_memory(ctx, fc, output_text)
                    .await?;
                Ok(None)
            }
            "script_builder" => {
                self.handle_script_builder(ctx, fc, output_text, usage, _user_message)
                    .await?;
                Ok(None)
            }
            _ => {
                // FALLBACK: External dynamic skills loaded from database
                if let Some(dynamic_skill) = self.state.registry.skills.skills.get(&fc.name) {
                    self.handle_dynamic_skill(ctx, fc, output_text, &dynamic_skill, usage)
                        .await?;
                    Ok(None)
                } else {
                    tracing::warn!(
                        "⚠️ [Protocol] Unknown tool execution attempted: {}",
                        fc.name
                    );
                    *output_text = format!("(ERROR: Unknown tool '{}')", fc.name);
                    Ok(None)
                }
            }
        };

        // 🛡️ [Hook] Post-tool Lifecycle Hook
        self.state
            .registry
            .hooks
            .trigger_hook("post-tool", &hook_ctx, &fc.args)
            .await?;

        result
    }

    /// Handles execution of dynamic file-based skills via MCP Host.
    async fn handle_dynamic_skill(
        &self,
        ctx: &RunContext,
        fc: &crate::agent::types::GeminiFunctionCall,
        output_text: &mut String,
        skill: &crate::agent::script_skills::SkillDefinition,
        _usage: &mut Option<crate::agent::types::TokenUsage>,
    ) -> anyhow::Result<()> {
        let result = self
            .state
            .registry
            .mcp_host
            .call_tool(
                &skill.name,
                fc.args.clone(),
                ctx.workspace_root.clone(),
                &self.state.registry.skills.skills,
            )
            .await;

        match result {
            Ok(crate::agent::mcp::McpResult::Raw(output)) => {
                // 🛡️ [Security] Sanitization Hook
                if let crate::agent::sanitizer::SanitizationResult::Alert(msg) =
                    crate::agent::sanitizer::Sanitizer::scan(&output)
                {
                    *output_text = format!("(TOOL EXECUTION HALTED FOR SECURITY: {})", msg);
                    return Ok(());
                }

                let mut final_output = output;
                if let Some(verify_script) = &skill.verification_script {
                    match self
                        .run_verification_script(
                            verify_script,
                            &skill.name,
                            &fc.args,
                            &final_output,
                            &ctx.workspace_root,
                        )
                        .await
                    {
                        Ok(verify_res) => {
                            final_output = format!(
                                "{}\n\n[VERIFICATION STATUS]:\n{}",
                                final_output, verify_res
                            );
                        }
                        Err(e) => {
                            final_output =
                                format!("{}\n\n[VERIFICATION CRITICAL ERROR]: {}", final_output, e);
                        }
                    }
                }

                *output_text = format!(
                    "({} EXECUTED SUCCESSFULLY):\n\n{}",
                    skill.name, final_output
                );
            }
            Ok(crate::agent::mcp::McpResult::SystemDelegate(name, args)) => {
                if name == "recruit_specialist" {
                    let mut mapped_args = serde_json::Map::new();
                    if let Some(aid) = args.get("agent_id") {
                        mapped_args.insert("agentId".to_string(), aid.clone());
                    }
                    if let Some(msg) = args.get("task_description") {
                        mapped_args.insert("message".to_string(), msg.clone());
                    }

                    let mapped_fc = crate::agent::types::GeminiFunctionCall {
                        name: "spawn_subagent".to_string(),
                        args: serde_json::Value::Object(mapped_args),
                    };
                    self.handle_spawn_subagent(ctx, &mapped_fc, output_text, _usage)
                        .await?;
                }
            }
            Err(e) => {
                *output_text = format!("(SKILL EXEC FAILED: {})", e);
            }
        }
        Ok(())
    }

    /// Updates the agent's persistent working memory (scratchpad).
    pub(crate) async fn handle_update_working_memory(
        &self,
        ctx: &RunContext,
        fc: &crate::agent::types::GeminiFunctionCall,
        output_text: &mut String,
    ) -> anyhow::Result<()> {
        let new_memory = fc
            .args
            .get("memory")
            .cloned()
            .unwrap_or(serde_json::json!({}));

        if let Some(mut entry) = self.state.registry.agents.get_mut(&ctx.agent_id) {
            let agent = entry.value_mut();

            // If both are objects, we perform a shallow merge. Otherwise, full overwrite.
            if let (Some(old_obj), Some(new_obj)) =
                (agent.working_memory.as_object_mut(), new_memory.as_object())
            {
                for (k, v) in new_obj {
                    old_obj.insert(k.clone(), v.clone());
                }
            } else {
                agent.working_memory = new_memory;
            }

            let agent_data = agent.clone();
            drop(entry); // Release DashMap lock

            // Sync to DB
            crate::agent::persistence::save_agent_db(&self.state.resources.pool, &agent_data)
                .await?;

            self.state.emit_event(serde_json::json!({
                "type": "agent:update",
                "data": agent_data
            }));

            *output_text = "(WORKING MEMORY UPDATED SUCCESSFULLY)".to_string();
        } else {
            *output_text =
                "(ERROR: Agent not found in registry during working memory update)".to_string();
        }

        Ok(())
    }

    /// Recursively executes a batch of tool calls provided by the LLM.
    /// This "collapses" multiple turns into one, reducing overhead.
    pub(crate) async fn handle_script_builder(
        &self,
        ctx: &RunContext,
        fc: &crate::agent::types::GeminiFunctionCall,
        output_text: &mut String,
        usage: &mut Option<crate::agent::types::TokenUsage>,
        user_message: &str,
    ) -> anyhow::Result<()> {
        let steps = fc
            .args
            .get("steps")
            .and_then(|v| v.as_array())
            .ok_or_else(|| anyhow::anyhow!("'steps' must be an array in script_builder"))?;

        output_text.push_str("\n--- BATCH EXECUTION STARTED ---\n");

        for (i, step) in steps.iter().enumerate() {
            let tool_name = step
                .get("tool")
                .and_then(|v| v.as_str())
                .ok_or_else(|| anyhow::anyhow!("Step {} missing 'tool' name", i))?;
            let params = step
                .get("params")
                .cloned()
                .unwrap_or(serde_json::Value::Object(serde_json::Map::new()));

            let mut step_output = String::new();
            let step_fc = crate::agent::types::GeminiFunctionCall {
                name: tool_name.to_string(),
                args: params,
            };

            tracing::info!("📦 [ScriptBuilder] Executing step {}: {}", i + 1, tool_name);
            output_text.push_str(&format!("\n[Step {}: {}]\n", i + 1, tool_name));

            // Execute the individual tool
            let _ = std::pin::Pin::from(Box::new(
                self.execute_tool(ctx, &step_fc, &mut step_output, usage, user_message),
            ))
            .await?;

            output_text.push_str(&step_output);
        }

        output_text.push_str("\n--- BATCH EXECUTION COMPLETED ---\n");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_propose_skill_validation_rejection() {
        let state = Arc::new(crate::state::AppState::new().await);
        let runner = AgentRunner::new(state);
        let ctx = RunContext {
            agent_id: "test".to_string(),
            name: "test".to_string(),
            role: "test".to_string(),
            department: "test".to_string(),
            description: "test".to_string(),
            mission_id: "test".to_string(),
            model_config: crate::agent::types::ModelConfig {
                provider: "test".to_string(),
                model_id: "test".to_string(),
                api_key: None,
                base_url: None,
                system_prompt: None,
                temperature: None,
                max_tokens: None,
                external_id: None,
                rpm: None,
                rpd: None,
                tpm: None,
                tpd: None,
                skills: None,
                workflows: None,
                mcp_tools: None,
                connector_configs: None,
                extra_parameters: None,
            },
            provider_name: "test".to_string(),
            skills: vec![],
            workflows: vec![],
            mcp_tools: vec![],
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
            last_accessed_files: std::sync::Arc::new(std::sync::Mutex::new(Vec::new())),
            recent_findings: None,
            working_memory: serde_json::json!({}),
            summarized_history: None,
        };

        let mut output = String::new();

        let fc = crate::agent::types::GeminiFunctionCall {
            name: "propose_skill".to_string(),
            args: serde_json::json!({
                "type": "skill",
                "name": "test_skill",
                "description": "test"
            }),
        };

        runner
            .handle_propose_skill(&ctx, &fc, &mut output)
            .await
            .unwrap();
        assert!(output.contains("REJECTED: Skill proposals must include"));

        let mut output_workflow = String::new();
        let fc_workflow = crate::agent::types::GeminiFunctionCall {
            name: "propose_skill".to_string(),
            args: serde_json::json!({
                "type": "workflow",
                "name": "test_workflow",
                "description": "test"
            }),
        };

        runner
            .handle_propose_skill(&ctx, &fc_workflow, &mut output_workflow)
            .await
            .unwrap();
        assert!(output_workflow.contains("REJECTED: Workflow proposals must include"));
    }
}

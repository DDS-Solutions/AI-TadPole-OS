//! @docs ARCHITECTURE:Registry
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Agent Runner / lifecycle
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Type-safe state handling and bounded execution without unhandled panics.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: none declared
//! - **Witness Tests**: none declared

use super::require_str_opt;
use crate::agent::runner::tools::error::ToolExecutionError;
use crate::agent::runner::{AgentRunner, RunContext};
use crate::error::AppError;

const SWARM_REAPER_CYCLE: &str = "48h";

impl AgentRunner {
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
        let report = require_str_opt(ctx, &fc.args, "final_report", "complete_mission")?
            .unwrap_or_else(|| "Mission complete.".to_string());

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
            // Semantic Archival is now handled asynchronously by CognitiveMemoryPipelineService
            // A child may complete its assigned branch, but only the root
            // orchestrator or operator may close the shared mission ledger.
            if ctx.depth == 0 {
                if ctx
                    .swarm_partial_failure
                    .load(std::sync::atomic::Ordering::Acquire)
                {
                    ctx.swarm_review_approved
                        .store(true, std::sync::atomic::Ordering::Release);
                }
                crate::agent::mission::update_mission(
                    &self.state.resources.pool,
                    &ctx.mission_id,
                    crate::agent::types::MissionStatus::Completed,
                    None,
                )
                .await?;
            }
            self.broadcast_agent(
                ctx,
                &if ctx.depth == 0 {
                    format!("✅ Mission {} COMPLETED and archived.", ctx.mission_id)
                } else {
                    format!(
                        "✅ Branch for mission {} completed; parent review remains open.",
                        ctx.mission_id
                    )
                },
                "success",
            );
            // 🛡️ [Harden Phase 4: Clean Delivery]
            // We strip previous turn noise to provide a clear, professional final report.
            Ok(format!(
                "🏁 **MISSION ARCHIVE REPORT**\n\
                 Mission ID: {}\n\
                 Status: {}\n\n\
                 The mission has been successfully summarized and archived into long-term vector memory.\n\n\
                 **Summary Highlights**:\n{}",
                ctx.mission_id,
                if ctx.depth == 0 { "SUCCESS" } else { "BRANCH COMPLETE" },
                report
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

        Ok(format!(
            "(MISSION PINNED: This mission will now bypass the {} Swarm Reaper cycle.)",
            SWARM_REAPER_CYCLE
        ))
    }

    /// Handles `propose_capability`: submits a new skill, workflow, or hook proposal to the oversight system.
    pub(crate) async fn handle_propose_capability(
        &self,
        ctx: &RunContext,
        fc: &crate::agent::types::ToolCall,
    ) -> Result<String, ToolExecutionError> {
        let cap_type_str = require_str_opt(ctx, &fc.args, "type", "propose_capability")?
            .unwrap_or_else(|| "skill".to_string());
        let name = require_str_opt(ctx, &fc.args, "name", "propose_capability")?
            .unwrap_or_else(|| "unnamed".to_string());
        let description = require_str_opt(ctx, &fc.args, "description", "propose_capability")?
            .unwrap_or_default();

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

    /// Handles `ask_user_question`: prompts the operator with a structured question and 0-5 options.
    ///
    /// Implements AI-Tadpole-OS interactive HITL delegation:
    /// - 0-5 mutually exclusive choices.
    /// - Optional "(Recommended)" suffix for favored choice.
    /// - Emits `agent:user_question_requested` and queues into `oversight_queue`.
    /// - Suspends turn until user clicks an option or provides free-text input.
    pub(crate) async fn handle_ask_user_question(
        &self,
        ctx: &RunContext,
        fc: &crate::agent::types::ToolCall,
    ) -> Result<String, ToolExecutionError> {
        let question = super::require_str(ctx, &fc.args, "question", "ask_user_question")?;
        let options: Vec<String> = fc
            .args
            .get("options")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(|s| s.trim().to_string()))
                    .filter(|s| !s.is_empty())
                    .collect()
            })
            .unwrap_or_default();

        if options.len() > 5 {
            return Err(ToolExecutionError::Validation(
                "ask_user_question accepts at most 5 options".to_string(),
            ));
        }

        let context_desc = fc
            .args
            .get("context")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        tracing::info!(
            "❓ [HITL] Agent {} asking user question: '{}' ({} options)",
            ctx.agent_id,
            question,
            options.len()
        );

        let entry_id = uuid::Uuid::new_v4().to_string();

        // Broadcast interactive question event to UI
        let question_event = serde_json::json!({
            "type": "agent:user_question_requested",
            "question_id": entry_id,
            "mission_id": ctx.mission_id,
            "agent_id": ctx.agent_id,
            "question": question,
            "options": options,
            "context": context_desc
        });
        let _ = self.state.comms.telemetry_tx.send(question_event.clone());
        self.state.emit_event(question_event);

        self.broadcast_agent(
            ctx,
            &format!("❓ Awaiting user input: {}", question),
            "info",
        );

        // Submit to oversight resolution pipeline
        let audit = crate::agent::types::ToolCallAudit {
            id: entry_id.clone(),
            agent_id: ctx.agent_id.clone(),
            mission_id: Some(ctx.mission_id.clone()),
            skill: "ask_user_question".to_string(),
            params: serde_json::json!({
                "question": question,
                "options": options,
                "context": context_desc
            }),
            department: ctx.department.clone(),
            description: format!("Question: {} | Context: {}", question, context_desc),
            timestamp: chrono::Utc::now().to_rfc3339(),
        };

        let resolution = self
            .submit_oversight_resolution(audit, Some(ctx.mission_id.clone()))
            .await
            .map_err(ToolExecutionError::AppError)?;

        if let Some(answer) = resolution.user_answer {
            Ok(format!("(USER ANSWER): {}", answer))
        } else if resolution.approved {
            let default_answer = options
                .iter()
                .find(|opt| opt.contains("(Recommended)"))
                .cloned()
                .or_else(|| options.first().cloned())
                .unwrap_or_else(|| "Approved / Confirmed".to_string());
            Ok(format!("(USER CONFIRMED): {}", default_answer))
        } else {
            Ok("(USER REJECTED / DISMISSED): The operator rejected the prompt without answering. Proceed conservatively or choose safe defaults.".to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::types::ToolCall;
    use crate::state::AppState;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_ask_user_question_max_options_validation() {
        let state = Arc::new(AppState::new_minimal_mock().await);
        let runner = AgentRunner::new(state);
        let ctx = RunContext::default();

        let fc = ToolCall {
            name: "ask_user_question".to_string(),
            args: serde_json::json!({
                "question": "Which database engine should we adopt?",
                "options": ["Option 1", "Option 2", "Option 3", "Option 4", "Option 5", "Option 6"]
            }),
        };

        let result = runner.handle_ask_user_question(&ctx, &fc).await;
        assert!(
            matches!(result, Err(ToolExecutionError::Validation(msg)) if msg.contains("at most 5 options"))
        );
    }

    #[tokio::test]
    async fn test_ask_user_question_resolution_flow() {
        let state = Arc::new(AppState::new_minimal_mock().await);
        let runner = AgentRunner::new(state.clone());
        let ctx = RunContext::default();

        let _ = sqlx::query("INSERT INTO agents (id, name, role, department, description, status, metadata) VALUES (?, 'Oversight Test', 'Specialist', 'Standard', 'desc', 'idle', '{}')")
            .bind(&ctx.agent_id).execute(&state.resources.pool).await;
        let _ = sqlx::query("INSERT INTO mission_history (id, agent_id, title, status) VALUES (?, ?, 'Oversight Verification', 'active')")
            .bind(&ctx.mission_id).bind(&ctx.agent_id).execute(&state.resources.pool).await;

        let fc = ToolCall {
            name: "ask_user_question".to_string(),
            args: serde_json::json!({
                "question": "Should we migrate to SQLite WAL mode?",
                "options": ["Yes, enable WAL mode (Recommended)", "No, keep default DELETE mode"],
                "context": "WAL mode significantly improves concurrent read performance."
            }),
        };

        let runner_clone = runner.clone();
        let ctx_clone = ctx.clone();
        let handle =
            tokio::spawn(
                async move { runner_clone.handle_ask_user_question(&ctx_clone, &fc).await },
            );

        // Wait for oversight entry to be queued
        let mut entry_id = String::new();
        for _ in 0..20 {
            if let Some(entry) = state.comms.oversight_resolvers.iter().next() {
                entry_id = entry.key().clone();
                break;
            }
            tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        }
        assert!(!entry_id.is_empty(), "Oversight entry should be registered");

        // Simulate operator providing a user answer
        if let Some((_, tx)) = state.comms.oversight_resolvers.remove(&entry_id) {
            let _ = tx.send(crate::agent::types::OversightResolution {
                approved: true,
                override_slot: None,
                user_answer: Some("Yes, enable WAL mode (Recommended)".to_string()),
            });
        }

        let output = handle
            .await
            .unwrap()
            .expect("handle_ask_user_question failed");
        assert!(output.contains("(USER ANSWER): Yes, enable WAL mode (Recommended)"));
    }
}

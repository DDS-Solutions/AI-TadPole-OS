//! @docs ARCHITECTURE:Registry
//! @docs ARCHITECTURE:Runner
//!
//! ### AI Assist Note
//! **Mission Lifecycle Tools**: Completion oversight protocols, RAG semantic archiving, retention pins, and capability expansions.
//! This module implements high-fidelity logic for the Sovereign Reality layer.
//!
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: Runtime logic error, state desynchronization, or resource exhaustion.
//! - **Telemetry Link**: Search `[lifecycle]` in tracing logs.

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
}

// Metadata: [lifecycle]

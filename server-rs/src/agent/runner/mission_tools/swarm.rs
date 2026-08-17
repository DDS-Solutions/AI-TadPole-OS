//! @docs ARCHITECTURE:Registry
//! @docs ARCHITECTURE:Runner
//!
//! ### AI Assist Note
//! **Swarm Coordination Tools**: Peer messaging (envelopes), delegation (directives), and audit/review tools.
//! This module implements high-fidelity logic for the Sovereign Reality layer.
//!
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: Runtime logic error, state desynchronization, or resource exhaustion.
//! - **Telemetry Link**: Search `[swarm]` in tracing logs.

use super::{require_str, require_str_opt};
use crate::agent::runner::tools::error::ToolExecutionError;
use crate::agent::runner::{AgentRunner, RunContext};

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
        let topic = require_str_opt(ctx, &fc.args, "topic", "share_finding")?
            .unwrap_or_else(|| "General".to_string());
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
        let echo = format!(
            "**(Shared finding on topic '{}' successfully recorded.)**",
            topic
        );
        Ok(echo)
    }

    /// Handles `send_mission_directive`: delegates a task to another agent.
    ///
    /// ### 🛡️ Pre-Flight Guard
    /// Validates that the target agent exists before queuing the directive.
    /// If the agent is missing, attempts auto-registration via `ensure_sub_agent_exists`
    /// (same pattern as `spawn_subagent`). Returns an actionable error to the LLM
    /// if registration fails, rather than silently queuing an undeliverable directive.
    pub(crate) async fn handle_send_mission_directive(
        &self,
        ctx: &RunContext,
        fc: &crate::agent::types::ToolCall,
    ) -> Result<String, ToolExecutionError> {
        let target_agent_id = require_str(ctx, &fc.args, "agent_id", "send_mission_directive")?;
        let instruction = require_str(ctx, &fc.args, "instruction", "send_mission_directive")?;

        // ── Pre-Flight: verify target agent exists ────────────────────────────
        let exists = sqlx::query_scalar::<_, i64>("SELECT 1 FROM agents WHERE id = ?1")
            .bind(&target_agent_id)
            .fetch_optional(&self.state.resources.pool)
            .await
            .map(|r| r.is_some())
            .unwrap_or(false);

        if !exists {
            tracing::info!(
                "🧬 [Swarm] send_mission_directive: target '{}' not found — attempting auto-registration.",
                target_agent_id
            );
            // Attempt auto-registration using parent model config.
            // Note: ensure_sub_agent_exists takes &mut SqliteConnection.
            let registered = if let Ok(mut tx) = self.state.resources.pool.begin().await {
                let result = self
                    .ensure_sub_agent_exists(
                        &mut tx,
                        crate::agent::runner::swarm::SubAgentOptions {
                            agent_id: &target_agent_id,
                            parent_config: &ctx.model_config,
                            extra_skills: None,
                            extra_workflows: None,
                            role_override: None,
                        },
                    )
                    .await;
                match result {
                    Ok(_) => {
                        let _ = tx.commit().await;
                        true
                    }
                    Err(_) => false,
                }
            } else {
                false
            };

            if !registered {
                tracing::warn!(
                    "⚠️ [Swarm] send_mission_directive: target '{}' could not be auto-registered.",
                    target_agent_id
                );
                return Ok(format!(
                    "(DIRECTIVE FAILED: Agent '{}' does not exist and could not be auto-created. \
                     Use spawn_subagent to recruit them first, then retry send_mission_directive.)",
                    target_agent_id
                ));
            }
        }

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

        let id = crate::agent::runner::swarm_persistence::save_directive(
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

        let id = crate::agent::runner::swarm_persistence::save_review_request(
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

        crate::agent::runner::swarm_persistence::submit_review(
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

    /// Handles `send_agent_envelope`: dispatches message to another agent.
    pub(crate) async fn handle_send_agent_envelope(
        &self,
        ctx: &RunContext,
        fc: &crate::agent::types::ToolCall,
    ) -> Result<String, ToolExecutionError> {
        let target_agent_id = require_str(ctx, &fc.args, "target_agent_id", "send_agent_envelope")?;
        let instruction = require_str(ctx, &fc.args, "instruction", "send_agent_envelope")?;

        let artifacts = fc.args.get("artifacts").and_then(|v| {
            v.as_array().map(|arr| {
                arr.iter()
                    .filter_map(|val| val.as_str().map(|s| s.to_string()))
                    .collect::<Vec<String>>()
            })
        });

        let reasoning_trace = ctx
            .recent_findings
            .clone()
            .or_else(|| {
                if ctx.working_memory.is_null() || (ctx.working_memory.is_object() && ctx.working_memory.as_object().unwrap().is_empty()) {
                    None
                } else {
                    Some(ctx.working_memory.to_string())
                }
            })
            .map(|mut trace| {
                if trace.chars().count() > 1000 {
                    let chars: Vec<char> = trace.chars().collect();
                    let first_part: String = chars[..450].iter().collect();
                    let last_part: String = chars[chars.len() - 450..].iter().collect();
                    trace = format!(
                        "{}...\n[COMPRESSED REASONING TRACE: {} characters pruned for token optimization]\n...{}",
                        first_part,
                        chars.len() - 900,
                        last_part
                    );
                }
                trace
            });

        let envelope = crate::agent::runner::a2a_mailbox::MailboxEnvelope {
            id: uuid::Uuid::new_v4().to_string(),
            mission_id: ctx.mission_id.clone(),
            source_agent_id: ctx.agent_id.clone(),
            target_agent_id: target_agent_id.clone(),
            instruction,
            reasoning_trace,
            status: "pending".to_string(),
            result: None,
            artifacts,
        };

        let mailbox =
            crate::agent::runner::a2a_mailbox::A2AMailbox::new(self.state.resources.pool.clone());
        mailbox.send_envelope(&envelope).await?;

        Ok(format!(
            "Envelope [{}] successfully sent to agent {}.",
            envelope.id, target_agent_id
        ))
    }
}

// Metadata: [swarm]

//! @docs ARCHITECTURE:Runner
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Agent Runner / Tool Oversight Gate
//!
//! ### AI Assist Note
//! - Enforce policy approval and record auto-approved tool calls in the oversight ledger.
//!
//! ### 🔍 Debugging & Observability
//! - **Telemetry Targets**: none declared

use super::{
    error::ToolExecutionError,
    resolved_policy::ToolPolicyDescriptor,
    security::{DefaultSecurityManager, SecurityManager},
    AgentRunner, RunContext,
};

impl AgentRunner {
    /// Evaluates and executes the security policy and human-in-the-loop oversight gates.
    pub(super) async fn validate_security_and_oversight(
        &self,
        ctx: &RunContext,
        fc: &crate::agent::types::ToolCall,
        user_message: &str,
        descriptor: &ToolPolicyDescriptor,
    ) -> Result<(), ToolExecutionError> {
        let sec_mgr = DefaultSecurityManager;
        let validation = sec_mgr
            .pre_validate_resolved(self, ctx, fc, descriptor)
            .await?;
        let mission_id_opt = Some(ctx.mission_state.mission_id.clone());

        if validation.oversight_required {
            self.broadcast_sys(
                &format!(
                    "🔒 Security Gate: '{}' requires explicit approval.",
                    fc.name
                ),
                "warning",
                mission_id_opt.clone(),
            );

            let description = if user_message.is_empty() {
                validation.oversight_reason
            } else {
                let raw = format!(
                    "User Context: '{}' | Reason: {}",
                    user_message, validation.oversight_reason
                );
                self.state.security.secret_redactor.redact(&raw)
            };

            let approved = self
                .submit_oversight(
                    crate::agent::types::ToolCallAudit {
                        id: uuid::Uuid::new_v4().to_string(),
                        agent_id: ctx.identity.agent_id.clone(),
                        mission_id: mission_id_opt.clone(),
                        skill: fc.name.clone(),
                        params: fc.args.clone(),
                        department: ctx.identity.department.clone(),
                        description,
                        timestamp: chrono::Utc::now().to_rfc3339(),
                    },
                    mission_id_opt.clone(),
                )
                .await
                .map_err(ToolExecutionError::AppError)?;

            if !approved {
                return Err(ToolExecutionError::SecurityBlocked(format!(
                    "Execution of {} REJECTED by Oversight Security Gate",
                    fc.name
                )));
            }
        } else {
            // Auto-approved by policy: record entry in oversight_log for persistent audit history & UI Action Ledger
            let entry_id = uuid::Uuid::new_v4().to_string();
            let params_json = serde_json::to_string(&fc.args).unwrap_or_default();
            let payload_desc = if user_message.is_empty() {
                format!("Auto-approved by policy for tool: {}", fc.name)
            } else {
                format!("User Context: '{}' | Auto-approved by policy", user_message)
            };
            let _ = sqlx::query(
                "INSERT INTO oversight_log (id, mission_id, agent_id, entry_type, skill, params, status, decision, decided_at, decided_by, payload) VALUES (?, ?, ?, 'tool_call', ?, ?, 'auto_approved', 'auto_approved', datetime('now'), 'auto_policy', ?)"
            )
            .bind(&entry_id)
            .bind(&mission_id_opt)
            .bind(&ctx.identity.agent_id)
            .bind(&fc.name)
            .bind(params_json)
            .bind(payload_desc)
            .execute(&self.state.resources.pool)
            .await;
        }
        Ok(())
    }
}

//! @docs ARCHITECTURE:Registry:Mcp
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Agent Runner / MCP Permission Gate
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Default-deny on absent declarations and unconfigured prompter.
//! - `[Structural]` Fail-closed permission policy evaluation with strict agent identity binding.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: `AppError::Forbidden`
//! - **Telemetry Targets**: none declared

use std::sync::Arc;

use super::authz::{decode_mcp_tool_name, is_mcp_tool_authorized};
use super::context::AgentScope;
use crate::error::AppError;
use crate::security::permissions::{PermissionMode, PermissionPolicy, PermissionPrompter};

/// Sanitize dynamic strings for safe reflection in logs, errors, and events.
pub fn sanitize_reflected(s: &str, max_len: usize) -> String {
    s.chars()
        .filter(|c| !c.is_control())
        .take(max_len)
        .collect()
}

/// Boundary enforcement gate evaluating agent capabilities, policy modes, and human-in-the-loop prompts.
#[derive(Clone)]
pub struct PermissionGate {
    policy: Arc<PermissionPolicy>,
    prompter: Option<Arc<dyn PermissionPrompter>>,
}

impl PermissionGate {
    pub fn new(
        policy: Arc<PermissionPolicy>,
        prompter: Option<Arc<dyn PermissionPrompter>>,
    ) -> Self {
        Self { policy, prompter }
    }

    pub fn with_prompter(mut self, prompter: Arc<dyn PermissionPrompter>) -> Self {
        self.prompter = Some(prompter);
        self
    }

    pub fn policy(&self) -> &Arc<PermissionPolicy> {
        &self.policy
    }

    pub fn prompter(&self) -> Option<&Arc<dyn PermissionPrompter>> {
        self.prompter.as_ref()
    }

    /// Evaluates if the agent scope is authorized to execute the specified tool with arguments.
    pub async fn check_permission(
        &self,
        scope: &AgentScope<'_>,
        tool_name: &str,
        arguments: &serde_json::Value,
    ) -> Result<(), AppError> {
        // 1. Enforce agent capability declarations if tool belongs to an MCP server
        if decode_mcp_tool_name(tool_name).is_some()
            && !is_mcp_tool_authorized(scope.mcp_declarations, tool_name)
        {
            return Err(AppError::Forbidden(format!(
                "Permission denied: MCP tool '{}' is not authorized by agent declarations",
                sanitize_reflected(tool_name, 64)
            )));
        }

        // 2. Query policy binding agent identity and role
        let mode = self
            .policy
            .get_mode(Some(scope.agent_id), scope.role, tool_name)
            .await;

        match mode {
            PermissionMode::Deny => Err(AppError::Forbidden(format!(
                "Permission denied: Tool '{}' is explicitly blocked by policy.",
                sanitize_reflected(tool_name, 64)
            ))),
            PermissionMode::Prompt => {
                if let Some(ref prompter) = self.prompter {
                    let arg_summary = {
                        let full = arguments.to_string();
                        if full.chars().count() > 500 {
                            let truncated: String = full.chars().take(500).collect();
                            format!("{}... [truncated]", sanitize_reflected(&truncated, 500))
                        } else {
                            sanitize_reflected(&full, 500)
                        }
                    };

                    let prompt_msg = format!(
                        "Agent '{}' requests execution of tool '{}' with arguments: {}",
                        sanitize_reflected(scope.agent_id, 64),
                        sanitize_reflected(tool_name, 64),
                        arg_summary
                    );

                    let decision =
                        prompter
                            .prompt_user(tool_name, &prompt_msg)
                            .await
                            .map_err(|e| {
                                AppError::Forbidden(format!(
                                    "Prompt failed for tool '{}': {}",
                                    sanitize_reflected(tool_name, 64),
                                    e
                                ))
                            })?;

                    if decision != PermissionMode::Allow {
                        return Err(AppError::Forbidden(format!(
                            "Permission denied: Execution of tool '{}' was declined by operator.",
                            sanitize_reflected(tool_name, 64)
                        )));
                    }
                    Ok(())
                } else {
                    Err(AppError::Forbidden(format!(
                        "Permission denied: Tool '{}' requires human confirmation via Prompt policy, but no prompter is configured.",
                        sanitize_reflected(tool_name, 64)
                    )))
                }
            }
            PermissionMode::Allow => Ok(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::mcp::authz::encode_mcp_tool_name;

    async fn create_test_pool() -> sqlx::SqlitePool {
        let pool = sqlx::sqlite::SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .unwrap();

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS permission_policies (
                tool_name TEXT PRIMARY KEY,
                mode TEXT NOT NULL
            );
            CREATE TABLE IF NOT EXISTS agent_permission_policies (
                agent_id TEXT NOT NULL,
                tool_name TEXT NOT NULL,
                mode TEXT NOT NULL,
                PRIMARY KEY (agent_id, tool_name)
            );
            CREATE TABLE IF NOT EXISTS role_permission_policies (
                role TEXT NOT NULL,
                tool_name TEXT NOT NULL,
                mode TEXT NOT NULL,
                PRIMARY KEY (role, tool_name)
            );
            "#,
        )
        .execute(&pool)
        .await
        .unwrap();

        pool
    }

    #[tokio::test]
    async fn test_unauthorized_mcp_tool_blocked_by_declarations() {
        let pool = create_test_pool().await;
        let policy = Arc::new(PermissionPolicy::new(pool));
        let gate = PermissionGate::new(policy, None);

        let declarations = vec!["brave-search:*".to_string()];
        let tool_name = encode_mcp_tool_name("github", "create_issue");
        let scope = AgentScope::new("agent-1", &declarations);

        let res = gate
            .check_permission(&scope, &tool_name, &serde_json::json!({}))
            .await;
        assert!(res.is_err());
        assert!(matches!(res.unwrap_err(), AppError::Forbidden(_)));
    }

    #[tokio::test]
    async fn test_prompt_mode_fails_closed_without_prompter() {
        let pool = create_test_pool().await;
        let policy = Arc::new(PermissionPolicy::new(pool));
        policy
            .set_mode("sensitive_tool", PermissionMode::Prompt)
            .await
            .unwrap();

        let gate = PermissionGate::new(policy, None);
        let scope = AgentScope::new("agent-1", &[]);

        let res = gate
            .check_permission(&scope, "sensitive_tool", &serde_json::json!({}))
            .await;
        assert!(res.is_err());
        assert!(matches!(res.unwrap_err(), AppError::Forbidden(_)));
    }

    #[tokio::test]
    async fn test_agent_policy_overrides_global_policy() {
        let pool = create_test_pool().await;
        sqlx::query(
            "INSERT INTO permission_policies (tool_name, mode) VALUES ('restricted_tool', 'deny')",
        )
        .execute(&pool)
        .await
        .unwrap();
        sqlx::query("INSERT INTO agent_permission_policies (agent_id, tool_name, mode) VALUES ('agent-a', 'restricted_tool', 'allow')")
            .execute(&pool)
            .await
            .unwrap();

        let policy = Arc::new(PermissionPolicy::new(pool));
        let gate = PermissionGate::new(policy, None);

        let scope_a = AgentScope::new("agent-a", &[]);
        let res_a = gate
            .check_permission(&scope_a, "restricted_tool", &serde_json::json!({}))
            .await;
        assert!(res_a.is_ok());

        let scope_b = AgentScope::new("agent-b", &[]);
        let res_b = gate
            .check_permission(&scope_b, "restricted_tool", &serde_json::json!({}))
            .await;
        assert!(res_b.is_err());
    }
}

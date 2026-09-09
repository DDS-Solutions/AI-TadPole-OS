//! @docs ARCHITECTURE:Evolution
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Agent Runner / evolution_tools
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Type-safe state handling and bounded execution without unhandled panics.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: none declared
//! - **Witness Tests**: none declared

use super::{AgentRunner, RunContext};
use crate::agent::runner::tools::error::ToolExecutionError;
use crate::error::AppError;

fn validate_skill_name(name: &str) -> Result<String, AppError> {
    let trimmed = name.trim().to_lowercase().replace(' ', "_");
    if trimmed.is_empty() || trimmed.len() > 64 {
        return Err(AppError::BadRequest(
            "Skill name must be between 1 and 64 characters long".to_string(),
        ));
    }
    if !trimmed
        .chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
    {
        return Err(AppError::BadRequest(format!(
            "Skill name '{}' contains invalid characters. Only lowercase alphanumeric and underscores allowed.",
            name
        )));
    }
    Ok(trimmed)
}

fn validate_skill_schema(schema: &serde_json::Value) -> Result<(), AppError> {
    if !schema.is_object() {
        return Err(AppError::BadRequest(
            "Skill schema must be a valid JSON Object matching tool definition schema".to_string(),
        ));
    }
    Ok(())
}

impl AgentRunner {
    /// Handles `synthesize_micro_script`: allows agents to autonomously create new specialized tools.
    pub(crate) async fn handle_synthesize_micro_script(
        &self,
        ctx: &RunContext,
        fc: &crate::agent::types::ToolCall,
    ) -> Result<String, ToolExecutionError> {
        let raw_name = fc
            .args
            .get("skill_name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| AppError::BadRequest("Missing required 'skill_name'".to_string()))?;
        let description = fc
            .args
            .get("description")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let code = fc
            .args
            .get("code")
            .and_then(|v| v.as_str())
            .ok_or_else(|| AppError::BadRequest("Missing required 'code'".to_string()))?;
        let schema = fc
            .args
            .get("schema")
            .cloned()
            .unwrap_or(serde_json::json!({}));

        if code.trim().is_empty() {
            return Err(ToolExecutionError::ExecutionFailed(
                "Skill code cannot be empty".to_string(),
            ));
        }
        if code.len() > 500_000 {
            return Err(ToolExecutionError::ExecutionFailed(
                "Skill code exceeds maximum allowable limit (500 KB)".to_string(),
            ));
        }

        let safe_name = validate_skill_name(raw_name)?;
        validate_skill_schema(&schema)?;

        tracing::info!(
            "🧬 [Evolution] Agent {} synthesizing new micro-script: {}",
            ctx.agent_id,
            safe_name
        );

        let skill_dir = std::path::Path::new("execution/agent_generated/skills");
        tokio::fs::create_dir_all(skill_dir)
            .await
            .map_err(AppError::Io)?;
        let canonical_dir = skill_dir.canonicalize().map_err(AppError::Io)?;

        let skill_file_path = canonical_dir.join(format!("{}.py", safe_name));
        let meta_file_path = canonical_dir.join(format!("{}.json", safe_name));

        // Path containment check
        if !skill_file_path.starts_with(&canonical_dir)
            || !meta_file_path.starts_with(&canonical_dir)
        {
            return Err(ToolExecutionError::ExecutionFailed(
                "Security Gate: Path traversal detected outside skill directory".to_string(),
            ));
        }

        // Security: Prevent overwriting existing manual tools
        if skill_file_path.exists() {
            return Ok(format!(
                "(SYNTHESIS FAILED: Skill '{}' already exists. Use 'refactor_synthesized_skill' to update it.)",
                safe_name
            ));
        }

        // Oversight: New code execution requires approval
        let approved = self
            .submit_oversight(
                crate::agent::types::ToolCallAudit {
                    id: uuid::Uuid::new_v4().to_string(),
                    agent_id: ctx.agent_id.clone(),
                    mission_id: Some(ctx.mission_id.clone()),
                    skill: "synthesize_micro_script".to_string(),
                    params: fc.args.clone(),
                    department: ctx.department.clone(),
                    description: format!("Creating new autonomous skill: {}", safe_name),
                    timestamp: chrono::Utc::now().to_rfc3339(),
                },
                Some(ctx.mission_id.clone()),
            )
            .await?;

        if !approved {
            return Ok(format!(
                "(Synthesis for '{}' REJECTED by Oversight)",
                safe_name
            ));
        }

        // Write Python script
        tokio::fs::write(&skill_file_path, code)
            .await
            .map_err(AppError::Io)?;

        // Write Skill Manifest
        let manifest = serde_json::json!({
            "name": safe_name,
            "description": description,
            "execution_command": format!("python execution/agent_generated/skills/{}.py", safe_name),
            "schema": schema,
            "requires_oversight": true
        });
        let manifest_json = serde_json::to_string_pretty(&manifest)
            .map_err(|e| AppError::InternalServerError(e.to_string()))?;
        tokio::fs::write(&meta_file_path, manifest_json)
            .await
            .map_err(AppError::Io)?;

        // Dynamic Registry Refresh AFTER all file writes succeed
        if let Err(e) = self.state.registry.skills.reload_all().await {
            tracing::error!(
                "🚨 [Evolution] Failed to hot-reload registry after synthesis: {:?}",
                e
            );
        }

        self.emit_evolution_event(
            ctx,
            "synthesis",
            &safe_name,
            "Created new autonomous micro-script.",
        );
        Ok(format!("(SUCCESS): New skill '{}' synthesized and added to the registry. You can now call this tool directly in your next turn.", safe_name))
    }

    /// Handles `refactor_synthesized_skill`: updates an existing agent-generated tool.
    pub(crate) async fn handle_refactor_synthesized_skill(
        &self,
        ctx: &RunContext,
        fc: &crate::agent::types::ToolCall,
    ) -> Result<String, ToolExecutionError> {
        let raw_name = fc
            .args
            .get("skill_name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| AppError::BadRequest("Missing skill_name".to_string()))?;
        let code = fc
            .args
            .get("code")
            .and_then(|v| v.as_str())
            .ok_or_else(|| AppError::BadRequest("Missing code".to_string()))?;
        let description = fc.args.get("description").and_then(|v| v.as_str());

        if code.trim().is_empty() {
            return Err(ToolExecutionError::ExecutionFailed(
                "Skill code cannot be empty".to_string(),
            ));
        }
        if code.len() > 500_000 {
            return Err(ToolExecutionError::ExecutionFailed(
                "Skill code exceeds maximum allowable limit (500 KB)".to_string(),
            ));
        }

        let safe_name = validate_skill_name(raw_name)?;

        let skill_dir = std::path::Path::new("execution/agent_generated/skills");
        tokio::fs::create_dir_all(skill_dir)
            .await
            .map_err(AppError::Io)?;
        let canonical_dir = skill_dir.canonicalize().map_err(AppError::Io)?;

        let skill_file_path = canonical_dir.join(format!("{}.py", safe_name));
        let meta_file_path = canonical_dir.join(format!("{}.json", safe_name));

        // Path containment check
        if !skill_file_path.starts_with(&canonical_dir)
            || !meta_file_path.starts_with(&canonical_dir)
        {
            return Err(ToolExecutionError::ExecutionFailed(
                "Security Gate: Path traversal detected outside skill directory".to_string(),
            ));
        }

        // Security: Ensure the skill exists before refactoring (to prevent random file writes)
        if !skill_file_path.exists() {
            return Ok(format!("(REFACTOR FAILED: Skill '{}' does not exist. Use 'synthesize_micro_script' to create it first.)", safe_name));
        }

        tracing::info!(
            "🧬 [Evolution] Agent {} refactoring skill: {}",
            ctx.agent_id,
            safe_name
        );

        self.broadcast_agent(
            ctx,
            &format!("🧬 Refactoring synthesized skill: {}...", safe_name),
            "info",
        );

        // Oversight: All refactors require approval to prevent drift/hallucination
        let approved = self
            .submit_oversight(
                crate::agent::types::ToolCallAudit {
                    id: uuid::Uuid::new_v4().to_string(),
                    agent_id: ctx.agent_id.clone(),
                    mission_id: Some(ctx.mission_id.clone()),
                    skill: "refactor_synthesized_skill".to_string(),
                    params: fc.args.clone(),
                    department: ctx.department.clone(),
                    description: format!(
                        "Refining synthesized skill '{}' to improve performance/logic.",
                        safe_name
                    ),
                    timestamp: chrono::Utc::now().to_rfc3339(),
                },
                Some(ctx.mission_id.clone()),
            )
            .await?;

        if !approved {
            return Ok(format!(
                "(Refactor for '{}' REJECTED by Oversight)",
                safe_name
            ));
        }

        // 1. Write updated Python script
        tokio::fs::write(&skill_file_path, code)
            .await
            .map_err(AppError::Io)?;

        // 2. Update description if provided
        if let Some(desc) = description {
            if let Ok(content) = tokio::fs::read_to_string(&meta_file_path).await {
                if let Ok(mut json) = serde_json::from_str::<serde_json::Value>(&content) {
                    json["description"] = serde_json::json!(desc);
                    let pretty = serde_json::to_string_pretty(&json)
                        .map_err(|e| AppError::InternalServerError(e.to_string()))?;
                    tokio::fs::write(&meta_file_path, pretty)
                        .await
                        .map_err(AppError::Io)?;
                }
            }
        }

        // 3. Dynamic Registry Refresh AFTER all file updates succeed
        if let Err(e) = self.state.registry.skills.reload_all().await {
            tracing::error!(
                "🚨 [Evolution] Failed to hot-reload registry after refactor: {:?}",
                e
            );
        }

        self.emit_evolution_event(
            ctx,
            "refactor",
            &safe_name,
            "Improved logic and updated tool definition.",
        );
        Ok(format!(
            "Skill '{}' refactored and updated successfully.",
            safe_name
        ))
    }

    pub(crate) fn emit_evolution_event(
        &self,
        ctx: &RunContext,
        evolution_type: &str,
        skill_name: &str,
        details: &str,
    ) {
        let event = serde_json::json!({
            "type": "evolution:event",
            "agentId": ctx.agent_id,
            "missionId": ctx.mission_id,
            "evolutionType": evolution_type,
            "skillName": skill_name,
            "details": details,
            "timestamp": chrono::Utc::now().to_rfc3339(),
        });
        let _ = crate::telemetry::TELEMETRY_TX.send(event);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_skill_name_success() {
        assert_eq!(validate_skill_name("my_tool").unwrap(), "my_tool");
        assert_eq!(validate_skill_name("My Tool").unwrap(), "my_tool");
        assert_eq!(validate_skill_name("tool123").unwrap(), "tool123");
    }

    #[test]
    fn test_validate_skill_name_rejects_traversal_and_invalid_chars() {
        assert!(validate_skill_name("../../cron").is_err());
        assert!(validate_skill_name("tool/sub").is_err());
        assert!(validate_skill_name("tool\\sub").is_err());
        assert!(validate_skill_name("tool.py").is_err());
        assert!(validate_skill_name("").is_err());
        assert!(validate_skill_name(&"a".repeat(65)).is_err());
    }

    #[test]
    fn test_validate_skill_schema() {
        assert!(validate_skill_schema(&serde_json::json!({"type": "object"})).is_ok());
        assert!(validate_skill_schema(&serde_json::json!("not-an-object")).is_err());
        assert!(validate_skill_schema(&serde_json::json!([1, 2, 3])).is_err());
    }
}

//! @docs ARCHITECTURE:Evolution
//!
//! ### AI Assist Note
//! **Evolution Tools**: Allows agents to autonomously refine their specialized 
//! skill set. Implements the **Self-Refactoring** loop for synthesized scripts.
//!
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: File system lock during script update, invalid Python 
//!   syntax in refactored code, or security block on unauthorized paths.
//! - **Trace Scope**: `server-rs::agent::runner::evolution_tools`

use super::{AgentRunner, RunContext};
use crate::error::AppError;
use crate::agent::runner::tools::error::ToolExecutionError;

impl AgentRunner {
    /// Handles `synthesize_micro_script`: allows agents to autonomously create new specialized tools.
    pub(crate) async fn handle_synthesize_micro_script(
        &self,
        ctx: &RunContext,
        fc: &crate::agent::types::ToolCall,
        ) -> Result<String, ToolExecutionError> {
        let skill_name = fc.args.get("skill_name").and_then(|v| v.as_str()).unwrap_or("unnamed_skill");
        let description = fc.args.get("description").and_then(|v| v.as_str()).unwrap_or("");
        let code = fc.args.get("code").and_then(|v| v.as_str()).unwrap_or("");
        let schema = fc.args.get("schema").cloned().unwrap_or(serde_json::json!({}));

        tracing::info!("🧬 [Evolution] Agent {} synthesizing new micro-script: {}", ctx.agent_id, skill_name);

        let safe_name = skill_name.to_lowercase().replace(' ', "_");
        let skill_dir = "execution/agent_generated/skills";
        tokio::fs::create_dir_all(skill_dir).await.map_err(AppError::Io)?;

        let skill_file_path = format!("{}/{}.py", skill_dir, safe_name);
        let meta_file_path = format!("{}/{}.json", skill_dir, safe_name);

        // Security: Prevent overwriting existing manual tools
        if std::path::Path::new(&skill_file_path).exists() {
            return Ok(format!("(SYNTHESIS FAILED: Skill '{}' already exists. Use 'refactor_synthesized_skill' to update it.)", skill_name));
        }

        // Oversight: New code execution requires approval
        let approved = self.submit_oversight(
            crate::agent::types::ToolCallAudit {
                id: uuid::Uuid::new_v4().to_string(),
                agent_id: ctx.agent_id.clone(),
                mission_id: Some(ctx.mission_id.clone()),
                skill: "synthesize_micro_script".to_string(),
                params: fc.args.clone(),
                department: ctx.department.clone(),
                description: format!("Creating new autonomous skill: {}", skill_name),
                timestamp: chrono::Utc::now().to_rfc3339(),
            },
            Some(ctx.mission_id.clone()),
        ).await?;

        if !approved {
            return Ok(format!("(Synthesis for '{}' REJECTED by Oversight)", skill_name));
        }

        // Write Python script
        tokio::fs::write(&skill_file_path, code).await.map_err(AppError::Io)?;

        // Write Skill Manifest
        let manifest = serde_json::json!({
            "name": safe_name,
            "description": description,
            "execution_command": format!("python execution/agent_generated/skills/{}.py", safe_name),
            "schema": schema,
            "requires_oversight": true
        });
        let manifest_json = serde_json::to_string_pretty(&manifest).map_err(|e| AppError::InternalServerError(e.to_string()))?;
        tokio::fs::write(&meta_file_path, manifest_json).await.map_err(AppError::Io)?;

        // Phase 4: Dynamic Registry Refresh
        if let Err(e) = self.state.registry.skills.reload_all().await {
            tracing::error!("🚨 [Evolution] Failed to hot-reload registry after synthesis: {:?}", e);
        }

        self.emit_evolution_event(ctx, "synthesis", &safe_name, "Created new autonomous micro-script.");
        Ok(format!("(SUCCESS): New skill '{}' synthesized and added to the registry. You can now call this tool directly in your next turn.", safe_name))
    }

    /// Handles `refactor_synthesized_skill`: updates an existing agent-generated tool.
    pub(crate) async fn handle_refactor_synthesized_skill(
        &self,
        ctx: &RunContext,
        fc: &crate::agent::types::ToolCall,
        ) -> Result<String, ToolExecutionError> {
        let skill_name = fc
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

        let safe_name = skill_name.to_lowercase().replace(' ', "_");
        let skill_file_path = format!("execution/agent_generated/skills/{}.py", safe_name);
        let meta_file_path = format!("execution/agent_generated/skills/{}.json", safe_name);

        // Security: Ensure the skill exists before refactoring (to prevent random file writes)
        if !std::path::Path::new(&skill_file_path).exists() {
             return Ok(format!("(REFACTOR FAILED: Skill '{}' does not exist. Use 'synthesize_micro_script' to create it first.)", skill_name));
        }

        tracing::info!(
            "🧬 [Evolution] Agent {} refactoring skill: {}",
            ctx.agent_id,
            skill_name
        );
        
        self.broadcast_agent(
            ctx,
            &format!("🧬 Refactoring synthesized skill: {}...", skill_name),
            "info",
        );

        // Oversight: All refactors require approval to prevent drift/hallucination
        let approved = self.submit_oversight(
            crate::agent::types::ToolCallAudit {
                id: uuid::Uuid::new_v4().to_string(),
                agent_id: ctx.agent_id.clone(),
                mission_id: Some(ctx.mission_id.clone()),
                skill: "refactor_synthesized_skill".to_string(),
                params: fc.args.clone(),
                department: ctx.department.clone(),
                description: format!("Refining synthesized skill '{}' to improve performance/logic.", skill_name),
                timestamp: chrono::Utc::now().to_rfc3339(),
            },
            Some(ctx.mission_id.clone()),
        ).await?;


        if !approved {
            return Ok(format!("(Refactor for '{}' REJECTED by Oversight)", skill_name));
        }

        tokio::fs::write(&skill_file_path, code).await.map_err(AppError::Io)?;

        // Phase 4: Dynamic Registry Refresh
        if let Err(e) = self.state.registry.skills.reload_all().await {
            tracing::error!("🚨 [Evolution] Failed to hot-reload registry after refactor: {:?}", e);
        }

        // Update description if provided
        if let Some(desc) = description {
            if let Ok(content) = tokio::fs::read_to_string(&meta_file_path).await {
                if let Ok(mut json) = serde_json::from_str::<serde_json::Value>(&content) {
                    json["description"] = serde_json::json!(desc);
                    let pretty = serde_json::to_string_pretty(&json).map_err(|e| AppError::InternalServerError(e.to_string()))?;
                    tokio::fs::write(&meta_file_path, pretty).await.map_err(|e: std::io::Error| AppError::Io(e))?;
                }
            }
        }

        self.emit_evolution_event(ctx, "refactor", skill_name, "Improved logic and updated tool definition.");
        Ok(format!("Skill '{}' refactored and updated successfully.", skill_name))
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

// Metadata: [evolution_tools]

// Metadata: [evolution_tools]

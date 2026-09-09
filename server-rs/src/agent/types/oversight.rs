//! @docs ARCHITECTURE:Registry
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Agent Runner / oversight
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Type-safe state handling and bounded execution without unhandled panics.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: none declared
//! - **Witness Tests**: none declared

use super::model::Validatable;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow, Default, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct RoleBlueprint {
    pub id: String,
    pub name: String,
    pub department: String,
    pub description: String,
    #[serde(default)]
    pub skills: String,
    #[serde(default)]
    pub workflows: String,
    #[serde(default, alias = "mcpTools")]
    pub mcp_tools: String,
    #[serde(default, alias = "requiresOversight")]
    pub requires_oversight: bool,
    #[serde(default, alias = "modelId")]
    pub model_id: Option<String>,
    #[serde(default, alias = "createdAt")]
    pub created_at: Option<DateTime<Utc>>,
}

impl Validatable for RoleBlueprint {
    fn validate(&self) -> Result<(), String> {
        if self.id.trim().is_empty() {
            return Err("Blueprint ID cannot be empty".to_string());
        }
        if self.name.trim().is_empty() {
            return Err("Blueprint name cannot be empty".to_string());
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, specta::Type)]
pub struct ToolCallAudit {
    pub id: String,
    pub mission_id: Option<String>,
    #[serde(rename = "agent_id")]
    pub agent_id: String,
    pub skill: String,
    pub params: serde_json::Value,
    pub department: String,
    pub description: String,
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, specta::Type)]
#[serde(rename_all = "lowercase")]
pub enum SkillType {
    Skill,
    Workflow,
    Hook,
}

fn default_category() -> String {
    "user".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct SkillProposal {
    pub r#type: SkillType,
    pub name: String,
    pub description: String,
    pub execution_command: Option<String>,
    pub schema: Option<serde_json::Value>,
    pub content: Option<String>,
    pub full_instructions: Option<String>,
    pub negative_constraints: Option<Vec<String>>,
    pub verification_script: Option<String>,
    #[serde(default = "default_category")]
    pub category: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct OversightEntry {
    pub id: String,
    pub mission_id: Option<String>,
    pub tool_call: Option<ToolCallAudit>,
    #[serde(alias = "capability_proposal")]
    pub skill_proposal: Option<SkillProposal>,
    pub status: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OversightDecision {
    pub decision: String,
    pub signature: Option<String>,
    pub verifying_key: Option<String>,
    pub override_slot: Option<String>,
    pub timestamp: Option<i64>,
    pub nonce: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OversightResolution {
    pub approved: bool,
    pub override_slot: Option<String>,
}

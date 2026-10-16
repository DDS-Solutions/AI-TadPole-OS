//! @docs ARCHITECTURE:Registry:Mcp
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Agent Runner / MCP Types
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Type-safe state representations for MCP tools and responses.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: none declared
//! - **Witness Tests**: `types::tests::*`

use crate::agent::script_skills::SkillDefinition;
use serde::{Deserialize, Serialize};

/// Operational statistics for a specific tool.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct McpToolStats {
    pub invocations: u64,
    pub success_count: u64,
    pub failure_count: u64,
    pub avg_latency_ms: u64,
}

/// A structured tool definition registered within the MCP ecosystem.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpToolHub {
    pub name: String,
    pub description: String,
    pub input_schema: serde_json::Value,
    pub source: String,
    pub stats: McpToolStats,
    pub category: String,
}

impl From<SkillDefinition> for McpToolHub {
    fn from(skill: SkillDefinition) -> Self {
        Self {
            name: skill.name,
            description: skill.description,
            input_schema: skill.schema,
            source: "legacy".to_string(),
            stats: McpToolStats::default(),
            category: skill.category,
        }
    }
}

/// The result returned from an MCP tool invocation.
#[derive(Debug, Clone)]
pub enum McpResult {
    Raw(String),
    /// Full validated MCP tool result, including structured content and execution metadata.
    Structured(serde_json::Value),
    SystemDelegate(String, serde_json::Value),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_skill_definition_into_mcp_tool_hub() {
        let skill: SkillDefinition = serde_json::from_value(serde_json::json!({
            "name": "test_skill",
            "description": "A test skill",
            "schema": {"type": "object"},
            "execution_command": "python test.py",
            "category": "automation"
        }))
        .unwrap();

        let hub = McpToolHub::from(skill);
        assert_eq!(hub.name, "test_skill");
        assert_eq!(hub.source, "legacy");
        assert_eq!(hub.category, "automation");
        assert_eq!(hub.stats.invocations, 0);
    }
}

//! @docs ARCHITECTURE:Registry:Mcp
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Agent Runner / MCP Tool Resolution
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Disjoint namespace resolution: MCP prefix tools (`mcp__*`) MUST NOT be shadowed by legacy skills or native tools.
//! - `[Structural]` Deterministic pure resolution mapping to typed ResolvedTool variants.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: `AppError::NotFound`
//! - **Telemetry Targets**: none declared

use super::authz::decode_mcp_tool_name;
use super::gate::sanitize_reflected;
use crate::agent::script_skills::SkillDefinition;
use crate::error::AppError;

/// Strongly-typed resolution outcome for a tool identifier.
#[derive(Debug, Clone)]
pub enum ResolvedTool {
    Native(String),
    Skill(SkillDefinition),
    Mcp {
        server_name: String,
        tool_name: String,
    },
}

/// Resolves a tool identifier into its deterministic execution target.
/// Enforces disjoint namespaces so that no external skill or native tool can hijack an MCP namespace.
pub fn resolve_tool(
    tool_name: &str,
    has_native: bool,
    skill: Option<SkillDefinition>,
) -> Result<ResolvedTool, AppError> {
    if let Some((server, tool)) = decode_mcp_tool_name(tool_name) {
        Ok(ResolvedTool::Mcp {
            server_name: server.to_string(),
            tool_name: tool.to_string(),
        })
    } else if let Some(skill_def) = skill {
        Ok(ResolvedTool::Skill(skill_def))
    } else if has_native {
        Ok(ResolvedTool::Native(tool_name.to_string()))
    } else {
        Err(AppError::NotFound(format!(
            "Tool '{}' not found",
            sanitize_reflected(tool_name, 64)
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dummy_skill(name: &str) -> SkillDefinition {
        SkillDefinition {
            id: None,
            name: name.to_string(),
            description: "test".to_string(),
            execution_command: "echo test".to_string(),
            schema: serde_json::json!({}),
            oversight_required: false,
            doc_url: None,
            tags: None,
            full_instructions: None,
            negative_constraints: None,
            verification_script: None,
            category: "test".to_string(),
            security_score: None,
            security_severity: None,
            security_report: None,
        }
    }

    #[test]
    fn test_resolve_mcp_tool_cannot_be_shadowed_by_skill() {
        let shadow_skill = dummy_skill("mcp__github__create_issue");
        let resolved = resolve_tool(
            "mcp__github__create_issue",
            true, // even if native also matches
            Some(shadow_skill),
        )
        .expect("should resolve");

        match resolved {
            ResolvedTool::Mcp {
                server_name,
                tool_name,
            } => {
                assert_eq!(server_name, "github");
                assert_eq!(tool_name, "create_issue");
            }
            _ => panic!("MCP tool was shadowed! Expected ResolvedTool::Mcp"),
        }
    }

    #[test]
    fn test_resolve_legacy_skill() {
        let skill = dummy_skill("custom_backup");
        let resolved = resolve_tool("custom_backup", false, Some(skill)).expect("should resolve");

        match resolved {
            ResolvedTool::Skill(s) => assert_eq!(s.name, "custom_backup"),
            _ => panic!("Expected ResolvedTool::Skill"),
        }
    }

    #[test]
    fn test_resolve_native_tool() {
        let resolved =
            resolve_tool("recruit_specialist", true, None).expect("should resolve native");

        match resolved {
            ResolvedTool::Native(name) => assert_eq!(name, "recruit_specialist"),
            _ => panic!("Expected ResolvedTool::Native"),
        }
    }

    #[test]
    fn test_resolve_unknown_tool_fails() {
        let res = resolve_tool("non_existent_tool", false, None);
        assert!(res.is_err());
        assert!(matches!(res.unwrap_err(), AppError::NotFound(_)));
    }
}

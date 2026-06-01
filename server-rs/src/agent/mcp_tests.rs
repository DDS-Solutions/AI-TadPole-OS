//! @docs ARCHITECTURE:Agent
//!
//! ### AI Assist Note
//! **MCP Integration Tests**: Orchestrates the verification of the
//! Model Context Protocol (MCP) host and tool connectivity for the
//! Tadpole OS engine. Features **Protocol Verification**: ensures
//! compliance with the MCP 1.0 specification for tool discovery and
//! execution. Implements **System Tool Verification**: validates the
//! availability and behavior of native Hydra-RS tools (e.g.,
//! `recruit_specialist`, `list_file_symbols`). AI agents should run
//! these tests after modifying the tool registry or security policies
//! to ensure no regressions in environment interaction (MCP-03).
//!
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: Tool discovery failures due to incorrect
//!   registry lookup, protocol mismatch when calling external
//!   servers, or permission denial for unauthorized test operations.
//! - **Trace Scope**: `server-rs::agent::mcp_tests`

#[cfg(test)]
mod tests {
    use crate::agent::mcp::{McpHost, McpResult};
    use crate::agent::script_skills::SkillDefinition;
    use crate::security::permissions::PermissionPolicy;
    use dashmap::DashMap;
    use serde_json::json;
    use sqlx::SqlitePool;
    use std::sync::Arc;

    async fn setup_mock_policy() -> Arc<PermissionPolicy> {
        let pool = SqlitePool::connect_lazy("sqlite::memory:").unwrap();
        Arc::new(PermissionPolicy::new(pool))
    }

    #[tokio::test]
    async fn test_list_tools_includes_system_tools() {
        let host = McpHost::new(
            tokio::sync::broadcast::channel(1).0,
            None,
            setup_mock_policy().await,
        );
        let all_skills = DashMap::new();
        let agent_skills = vec![];

        let tools = host.list_tools(&agent_skills, &all_skills).await;

        // Should contain recruit_specialist and native tools
        assert!(tools.iter().any(|t| t.name == "recruit_specialist"));
        assert!(tools.iter().any(|t| t.name == "list_file_symbols"));
        assert!(tools.iter().any(|t| t.name == "get_symbol_body"));
    }

    #[tokio::test]
    async fn test_list_tools_includes_agent_skills() {
        let host = McpHost::new(
            tokio::sync::broadcast::channel(1).0,
            None,
            setup_mock_policy().await,
        );
        let all_skills = DashMap::new();

        let skill = SkillDefinition {
            id: None,
            name: "test_skill".to_string(),
            description: "A test skill".to_string(),
            execution_command: "echo hello".to_string(),
            schema: json!({"type": "object"}),
            oversight_required: true,
            doc_url: None,
            tags: None,
            full_instructions: None,
            negative_constraints: None,
            verification_script: None,
            category: "user".to_string(),
        };
        all_skills.insert("test_skill".to_string(), skill);

        let agent_skills = vec!["test_skill".to_string()];
        let tools = host.list_tools(&agent_skills, &all_skills).await;

        assert!(tools.iter().any(|t| t.name == "test_skill"));
        assert!(tools.iter().any(|t| t.name == "recruit_specialist"));
    }

    #[tokio::test]
    async fn test_call_tool_system_delegate() {
        let host = McpHost::new(
            tokio::sync::broadcast::channel(1).0,
            None,
            setup_mock_policy().await,
        );
        let all_skills = DashMap::new();
        let workspace = std::path::PathBuf::from(".");

        let result = host
            .call_tool(
                "recruit_specialist",
                json!({"agent_id": "tester", "task_description": "test"}),
                workspace,
                &all_skills,
            )
            .await
            .unwrap();

        match result {
            McpResult::SystemDelegate(name, _) => assert_eq!(name, "recruit_specialist"),
            _ => panic!("Expected SystemDelegate"),
        }
    }

    #[tokio::test]
    async fn test_call_tool_native_delegate() {
        let host = McpHost::new(
            tokio::sync::broadcast::channel(1).0,
            None,
            setup_mock_policy().await,
        );
        let all_skills = DashMap::new();
        let workspace = std::path::PathBuf::from(".");

        // Testing list_file_symbols
        let result = host
            .call_tool(
                "list_file_symbols",
                json!({"path": "src/main.rs"}),
                workspace.clone(),
                &all_skills,
            )
            .await
            .unwrap();

        match result {
            McpResult::Raw(_) => (), // Success
            _ => panic!("Expected Raw result for native tool"),
        }
    }

    #[tokio::test]
    async fn test_call_tool_not_found() {
        let host = McpHost::new(
            tokio::sync::broadcast::channel(1).0,
            None,
            setup_mock_policy().await,
        );
        let all_skills = DashMap::new();
        let workspace = std::path::PathBuf::from(".");

        let result = host
            .call_tool("non_existent", json!({}), workspace, &all_skills)
            .await;

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }

    #[tokio::test]
    async fn test_native_tool_schema_accuracy() {
        let host = McpHost::new(
            tokio::sync::broadcast::channel(1).0,
            None,
            setup_mock_policy().await,
        );
        let all_skills = DashMap::new();
        let agent_skills = vec![];
        let tools = host.list_tools(&agent_skills, &all_skills).await;

        // Verify list_file_symbols schema
        let list_tool = tools.iter().find(|t| t.name == "list_file_symbols").unwrap();
        let schema = &list_tool.input_schema;
        assert_eq!(schema["type"], "object");
        assert!(schema["properties"].get("path").is_some());
        // Verify system parameters (workspace_root) are excluded
        assert!(schema["properties"].get("workspace_root").is_none());
        assert!(schema["properties"].get("_workspace_root").is_none());
        assert!(schema["properties"].get("args").is_none());
        
        let required = schema["required"].as_array().unwrap();
        assert!(required.iter().any(|v| v.as_str() == Some("path")));
        assert!(!required.iter().any(|v| v.as_str() == Some("workspace_root")));
    }

    #[tokio::test]
    async fn test_runner_executes_mcp_fallback() {
        use crate::state::AppState;
        use crate::agent::runner::{AgentRunner, RunContext};
        use crate::agent::types::{EngineAgent, ToolCall};

        let state = Arc::new(AppState::new_minimal_mock().await);
        let runner = AgentRunner::new(state.clone());
        let mut ctx = RunContext::default();
        ctx.agent_id = "test-agent".to_string();
        ctx.workspace_root = std::path::PathBuf::from(".");

        let mut agent = EngineAgent::default();
        agent.identity.id = ctx.agent_id.clone();
        agent.capabilities.skills = vec!["list_file_symbols".to_string()];
        state.registry.agents.insert(ctx.agent_id.clone(), agent);

        // We execute list_file_symbols via runner's Zero-Trust pipeline
        let fc = ToolCall {
            name: "list_file_symbols".to_string(),
            args: json!({ "path": "src/main.rs" }),
        };

        let mut output = String::new();
        let mut usage = None;
        let result = runner.execute_tool(&ctx, &fc, &mut output, &mut usage, "").await;

        assert!(result.is_ok());
        assert!(!output.contains("ToolNotFound"));
    }
}

// Metadata: [mcp_tests]

// Metadata: [mcp_tests]

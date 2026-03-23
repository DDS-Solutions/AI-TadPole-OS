#[cfg(test)]
mod tests {
    use crate::agent::script_skills::SkillDefinition;
    use crate::agent::mcp::{McpHost, McpResult};
    use dashmap::DashMap;
    use serde_json::json;

    #[tokio::test]
    async fn test_list_tools_includes_system_tools() {
        let host = McpHost::new(tokio::sync::broadcast::channel(1).0, None);
        let all_skills = DashMap::new();
        let agent_skills = vec![];

        let tools = host.list_tools(&agent_skills, &all_skills);

        // Should contain recruit_specialist and native tools
        assert!(tools.iter().any(|t| t.name == "recruit_specialist"));
        assert!(tools.iter().any(|t| t.name == "list_file_symbols"));
        assert!(tools.iter().any(|t| t.name == "get_symbol_body"));
    }

    #[tokio::test]
    async fn test_list_tools_includes_agent_skills() {
        let host = McpHost::new(tokio::sync::broadcast::channel(1).0, None);
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
        let tools = host.list_tools(&agent_skills, &all_skills);

        assert!(tools.iter().any(|t| t.name == "test_skill"));
        assert!(tools.iter().any(|t| t.name == "recruit_specialist"));
    }

    #[tokio::test]
    async fn test_call_tool_system_delegate() {
        let host = McpHost::new(tokio::sync::broadcast::channel(1).0, None);
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
        let host = McpHost::new(tokio::sync::broadcast::channel(1).0, None);
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
        let host = McpHost::new(tokio::sync::broadcast::channel(1).0, None);
        let all_skills = DashMap::new();
        let workspace = std::path::PathBuf::from(".");

        let result = host
            .call_tool("non_existent", json!({}), workspace, &all_skills)
            .await;

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }
}

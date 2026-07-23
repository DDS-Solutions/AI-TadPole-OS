//! Skill Verification — Sandbox and tool dispatch tests
//!
//! @docs ARCHITECTURE:Agent
//!
//! @state SkillsRegistry: (Initialized | MockedStorage)
//!
//! ### AI Assist Note
//! **Verification Strategy**: Uses `Uuid` based unique identifiers to avoid
//! collision in the physical file system during concurrent test execution.
//! Tests both the in-memory DashMap and the debounced disk sync.
//!
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: IO permission errors, malformed Markdown parsing, or
//!   stale file handles preventing clean deletion.
//! - **Trace Scope**: `server-rs::agent::tests_skills`

use super::script_skills::{ScriptSkillsRegistry, SkillDefinition, WorkflowDefinition};
use std::error::Error;
use tempfile::tempdir;
use uuid::Uuid;

async fn create_test_registry(
    base_path: &std::path::Path,
) -> Result<ScriptSkillsRegistry, Box<dyn Error>> {
    std::fs::create_dir_all(base_path.join("execution/agent_generated/skills"))?;
    std::fs::create_dir_all(base_path.join("directives/agent_generated/workflows"))?;
    std::fs::create_dir_all(base_path.join("hooks/agent_generated/hooks"))?;

    let registry = ScriptSkillsRegistry::mock(base_path.to_path_buf());
    Ok(registry)
}

#[tokio::test]
async fn test_skills_registry_save_and_sanitize() -> Result<(), Box<dyn Error>> {
    let temp_dir = tempdir()?;
    let base_path = temp_dir.path();
    let registry = create_test_registry(base_path).await?;

    // Create a mock skill with problematic characters in the name
    let weird_name = format!("Bad Skill! *Name_{}", Uuid::new_v4());
    let skill = SkillDefinition {
        id: None,
        name: weird_name.clone(),
        description: "Test skill".to_string(),
        execution_command: "echo test".to_string(),
        schema: serde_json::json!({
            "type": "object",
            "properties": {}
        }),
        oversight_required: true,
        doc_url: None,
        tags: None,
        full_instructions: None,
        negative_constraints: None,
        verification_script: None,
        category: "user".to_string(),
        security_score: None,
        security_severity: None,
        security_report: None,
    };

    // Save should sanitize the file name but preserve the internal name
    registry.save_skill(skill.clone()).await?;

    // Verify it is in the in-memory map
    assert!(
        registry.skills.contains_key(&weird_name),
        "Skill must be in memory with exact name"
    );

    // Check if the file was created
    // We don't have direct access to registry.skills_dir, but we can attempt to load it
    // by reloading the registry and ensuring our weird name still parses
    let new_registry = create_test_registry(base_path).await?;
    new_registry.reload_all().await?;
    assert!(
        new_registry.skills.contains_key(&weird_name),
        "Skill must persist and load properly"
    );

    // Clean up
    registry.delete_skill(&weird_name).await?;
    assert!(
        !registry.skills.contains_key(&weird_name),
        "Skill must be removed from memory"
    );

    let cleanup_registry = create_test_registry(base_path).await?;
    cleanup_registry.reload_all().await?;
    assert!(
        !cleanup_registry.skills.contains_key(&weird_name),
        "Skill must be removed from disk"
    );

    Ok(())
}

#[tokio::test]
async fn test_workflows_registry_save_and_delete() -> Result<(), Box<dyn Error>> {
    let temp_dir = tempdir()?;
    let base_path = temp_dir.path();
    let registry = create_test_registry(base_path).await?;

    let workflow_name = format!("test_workflow_{}", Uuid::new_v4());
    let workflow = WorkflowDefinition {
        id: None,
        name: workflow_name.clone(),
        content: "## Test Workflow\nSteps...".to_string(),
        doc_url: None,
        tags: None,
        category: "user".to_string(),
    };

    registry.save_workflow(workflow.clone()).await?;
    assert!(registry.workflows.contains_key(&workflow_name));

    let loaded_registry = create_test_registry(base_path).await?;
    loaded_registry.reload_all().await?;
    assert!(loaded_registry.workflows.contains_key(&workflow_name));
    assert_eq!(
        loaded_registry
            .workflows
            .get(&workflow_name)
            .unwrap()
            .content,
        "## Test Workflow\nSteps..."
    );

    registry.delete_workflow(&workflow_name).await?;
    assert!(!registry.workflows.contains_key(&workflow_name));

    Ok(())
}

// Metadata: [tests_skills]

#[cfg(test)]
mod recruitment_tests {

    #[tokio::test]
    async fn test_dependency_guard_binaries_and_envs() {
        use crate::security::dependency_guard::{check_skill_dependencies, is_binary_available};

        // Test a basic skill that should not require anything
        let res = check_skill_dependencies(&["read_file".to_string()]);
        assert!(res.is_ok());

        // Test a nonexistent binary skill
        let res_err = check_skill_dependencies(&["docker_run".to_string()]);
        if !is_binary_available("docker") {
            assert!(res_err.is_err());
            let errs = res_err.unwrap_err();
            assert!(errs[0].contains("requires system binary"));
        }
    }

    #[tokio::test]
    async fn test_ensure_sub_agent_exists_provider_fallback() {
        use crate::state::AppState;
        use crate::agent::runner::AgentRunner;
        use crate::agent::runner::swarm::SubAgentOptions;

        let state = std::sync::Arc::new(AppState::new_mock().await);
        let runner = AgentRunner::new(state.clone());
        let mut conn = state.resources.pool.acquire().await.unwrap();

        // Create custom options where sub-agent uses a provider with no keys
        let mut parent_config = crate::agent::types::ModelConfig::default();
        parent_config.provider = crate::agent::types::ModelProvider::Ollama;
        parent_config.model_id = "gemma-4-e4b".to_string();

        // Clean up environment variables to make sure Groq is not configured
        let _groq_key = std::env::var("GROQ_API_KEY");
        std::env::remove_var("GROQ_API_KEY");

        // Add a matched agent in registry using Groq (unconfigured)
        let agent_id = "test-groq-agent";
        let mut mock_agent = crate::agent::types::EngineAgent::default();
        mock_agent.identity.id = agent_id.to_string();
        mock_agent.identity.name = agent_id.to_string();
        mock_agent.identity.category = "user".to_string();
        mock_agent.models.model.provider = crate::agent::types::ModelProvider::Groq;
        mock_agent.models.model_id = Some("llama-3.3-70b-versatile".to_string());
        
        state.registry.agents.insert(agent_id.to_string(), mock_agent.clone());

        let opts = SubAgentOptions {
            agent_id,
            parent_config: &parent_config,
            extra_skills: None,
            extra_workflows: None,
            role_override: None,
        };

        // Run ensure_sub_agent_exists
        runner.ensure_sub_agent_exists(&mut conn, opts).await.unwrap();

        // Verify fallback occurred! The resolved agent in registry must now use Ollama (parent's provider)
        let resolved = state.registry.agents.get(agent_id).unwrap();
        assert_eq!(resolved.models.model.provider, crate::agent::types::ModelProvider::Ollama);
        assert_eq!(resolved.models.model.model_id, "gemma-4-e4b");

        // Restore Groq key if it existed
        if let Ok(val) = _groq_key {
            std::env::set_var("GROQ_API_KEY", val);
        }
    }

    #[tokio::test]
    async fn test_hierarchical_budgeting() {
        use crate::state::AppState;
        use crate::agent::runner::AgentRunner;
        use crate::agent::runner::RunContext;

        let state = std::sync::Arc::new(AppState::new_mock().await);
        let runner = AgentRunner::new(state.clone());

        let mut ctx = RunContext::default();
        ctx.budget_usd = 10.0;
        ctx.current_cost_usd = 1.0;
        ctx.sub_budget_usd = Some(0.5); // Allow sub-agent to spend $0.5

        // Check with 0 cost, should pass
        let res = runner.check_budget(&ctx, 0.0, "result").await.unwrap();
        assert!(res.is_none());

        // Check with step cost 0.6 (exceeds $0.5 sub-budget)
        let res_exceeded = runner.check_budget(&ctx, 0.6, "result").await.unwrap();
        assert!(res_exceeded.is_some());
        assert!(res_exceeded.unwrap().contains("Sub-budget Exceeded"));
    }
}

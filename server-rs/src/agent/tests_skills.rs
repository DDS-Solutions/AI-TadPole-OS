// use std::path::PathBuf;
// use std::env;
use uuid::Uuid;
use super::script_skills::{ScriptSkillsRegistry, SkillDefinition, WorkflowDefinition};

#[tokio::test]
async fn test_skills_registry_save_and_sanitize() -> anyhow::Result<()> {
    let registry = ScriptSkillsRegistry::new().await?;
    
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
    };

    // Save should sanitize the file name but preserve the internal name
    registry.save_skill(skill.clone()).await?;

    // Verify it is in the in-memory map
    assert!(registry.skills.contains_key(&weird_name), "Skill must be in memory with exact name");

    // let _sanitized_filename = weird_name.replace(|c: char| !c.is_alphanumeric() && c != '_' && c != '-', "_"); // Unused variable
    
    // Check if the file was created
    // We don't have direct access to registry.skills_dir, but we can attempt to load it
    // by reloading the registry and ensuring our weird name still parses
    let new_registry = ScriptSkillsRegistry::new().await?;
    assert!(new_registry.skills.contains_key(&weird_name), "Skill must persist and load properly");

    // Clean up
    registry.delete_skill(&weird_name).await?;
    assert!(!registry.skills.contains_key(&weird_name), "Skill must be removed from memory");
    
    let cleanup_registry = ScriptSkillsRegistry::new().await?;
    assert!(!cleanup_registry.skills.contains_key(&weird_name), "Skill must be removed from disk");

    Ok(())
}

#[tokio::test]
async fn test_workflows_registry_save_and_delete() -> anyhow::Result<()> {
    let registry = ScriptSkillsRegistry::new().await?;
    
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

    let loaded_registry = ScriptSkillsRegistry::new().await?;
    assert!(loaded_registry.workflows.contains_key(&workflow_name));
    assert_eq!(loaded_registry.workflows.get(&workflow_name).unwrap().content, "## Test Workflow\nSteps...");

    registry.delete_workflow(&workflow_name).await?;
    assert!(!registry.workflows.contains_key(&workflow_name));

    Ok(())
}

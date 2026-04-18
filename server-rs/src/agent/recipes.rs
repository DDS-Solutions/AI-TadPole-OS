//! @docs ARCHITECTURE:Registry
//! 
//! ### AI Assist Note
//! **Core technical module for the Tadpole OS hardened engine.**
//! This module implements high-fidelity logic for the Sovereign Reality layer.
//! 
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: Runtime logic error, state desynchronization, or resource exhaustion.
//! - **Telemetry Link**: Search `[recipes.rs]` in tracing logs.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use tracing::{info, error};
use crate::agent::types::EngineAgent;
use crate::state::AppState;
use std::sync::Arc;

/// A declarative definition of a specialized agent within a swarm recipe.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentRecipe {
    pub id: String,
    pub name: String,
    pub role: String,
    pub department: String,
    pub description: String,
    pub model_id: String,
    #[serde(default)]
    pub skills: Vec<String>,
    #[serde(default)]
    pub workflows: Vec<String>,
}

/// A declarative swarm blueprint for auto-bootstrapping mission specialists.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwarmRecipe {
    pub name: String,
    pub description: String,
    pub agents: Vec<AgentRecipe>,
}

/// Scans the .agent/recipes directory for YAML blueprints and ingests them into the registry.
pub async fn auto_ingest_recipes(state: Arc<AppState>) {
    let recipe_dir = state.base_dir.join(".agent").join("recipes");
    if !recipe_dir.exists() {
        let _ = tokio::fs::create_dir_all(&recipe_dir).await;
        // Seed default recipe if empty? 
        return;
    }

    match tokio::fs::read_dir(&recipe_dir).await {
        Ok(mut entries) => {
            while let Ok(Some(entry)) = entries.next_entry().await {
                let path = entry.path();
                if path.is_file() && (path.extension() == Some("yaml".as_ref()) || path.extension() == Some("yml".as_ref())) {
                    if let Err(e) = ingest_recipe(path, state.clone()).await {
                        error!("🚨 [Recipes] Failed to ingest recipe: {}", e);
                    }
                }
            }
        }
        Err(e) => error!("🚨 [Recipes] Failed to read recipe directory: {}", e),
    }
}

async fn ingest_recipe(path: PathBuf, state: Arc<AppState>) -> anyhow::Result<()> {
    let content = tokio::fs::read_to_string(&path).await?;
    let recipe: SwarmRecipe = serde_yaml::from_str(&content)?;

    info!("🍳 [Recipes] Ingesting Swarm Recipe: {} ({} agents)", recipe.name, recipe.agents.len());

    for a in recipe.agents {
        // Map AgentRecipe to EngineAgent
        let agent = EngineAgent {
            id: a.id.clone(),
            name: a.name,
            role: a.role,
            department: a.department,
            description: a.description,
            model_id: Some(a.model_id.clone()),
            model: crate::agent::types::ModelConfig {
                model_id: a.model_id,
                ..Default::default()
            },
            skills: a.skills,
            workflows: a.workflows,
            status: "idle".to_string(),
            metadata: HashMap::new(),
            ..Default::default()
        };

        // UPSERT into database
        if let Err(e) = crate::agent::persistence::save_agent_db(&state.resources.pool, &agent).await {
            error!("🚨 [Recipes] Failed to persist agent '{}': {}", agent.id, e);
        } else {
            // Update in-memory registry
            state.registry.agents.insert(agent.id.clone(), agent);
        }
    }

    Ok(())
}

// ─────────────────────────────────────────────────────────
//  UNIT TESTS
// ─────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::AppState;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_yaml_recipe_ingestion() -> anyhow::Result<()> {
        let state = Arc::new(AppState::new_mock().await); 
        
        // 1. Create a mock recipe YAML
        let temp_dir = tempfile::tempdir()?;
        let recipe_path = temp_dir.path().join("swarm_alpha.yaml");
        
        let yaml_content = r#"
name: Alpha Swarm
description: A high-tier tactical deployment blueprint.
agents:
  - id: tactical_auditor
    name: Tactical Auditor
    role: Security Specialist
    department: Compliance
    description: Verifies protocol integrity.
    model_id: llama3-70b
    skills: ["grep_search", "read_file"]
"#;
        tokio::fs::write(&recipe_path, yaml_content).await?;

        // 2. Ingest the recipe
        ingest_recipe(recipe_path, state.clone()).await?;

        // 3. Verify registry populate
        assert!(state.registry.agents.contains_key("tactical_auditor"), "Agent should be in registry");
        
        let agent = state.registry.agents.get("tactical_auditor").unwrap();
        assert_eq!(agent.role, "Security Specialist");
        assert!(agent.skills.contains(&"grep_search".to_string()));
        
        Ok(())
    }

    #[tokio::test]
    async fn test_recipe_id_collision_updates() -> anyhow::Result<()> {
        let state = Arc::new(AppState::new_mock().await);
        let temp_dir = tempfile::tempdir()?;
        let recipe_path = temp_dir.path().join("update_test.yaml");

        // Initial ingestion
        let yaml_1 = r#"
name: Test Swarm
description: Initial description.
agents:
  - id: worker_001
    name: Worker One
    role: Runner
    department: Ops
    description: Initial role.
    model_id: gpt-4
"#;
        tokio::fs::write(&recipe_path, yaml_1).await?;
        ingest_recipe(recipe_path.clone(), state.clone()).await?;

        // Update ingestion (same ID, different role)
        let yaml_2 = r#"
name: Test Swarm
description: Updated description.
agents:
  - id: worker_001
    name: Worker One
    role: Specialist
    department: Ops
    description: Updated role.
    model_id: gpt-4
"#;
        tokio::fs::write(&recipe_path, yaml_2).await?;
        ingest_recipe(recipe_path, state.clone()).await?;

        let agent = state.registry.agents.get("worker_001").unwrap();
        assert_eq!(agent.role, "Specialist", "Agent role should have been updated");
        
        Ok(())
    }
}

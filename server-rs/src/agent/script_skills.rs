//! @docs ARCHITECTURE:Agent
//!
//! ### AI Assist Note
//! - **Subsystem**: Script skills / Registry facade
//! - The facade owns published maps; codecs, ordered source loading, security scanning and guarded disk operations have separate owners.
//!
//! ### 🔍 Debugging & Observability
//! - **Telemetry Targets**: none declared

mod codec;
mod repository;
mod sources;
mod validation;

use crate::error::AppError;
pub use codec::parse_skill_md;
use dashmap::DashMap;
use repository::{ArtifactKind, ArtifactOrigin, ArtifactRepository};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::fs;

/// Represents a dynamic skill loaded from `data/skills/*.json`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillDefinition {
    pub id: Option<String>,
    pub name: String,
    pub description: String,
    pub execution_command: String,
    #[serde(alias = "parameters")]
    pub schema: serde_json::Value,
    #[serde(default = "default_oversight")]
    pub oversight_required: bool,
    pub doc_url: Option<String>,
    pub tags: Option<Vec<String>>,
    pub full_instructions: Option<String>,
    pub negative_constraints: Option<Vec<String>>,
    pub verification_script: Option<String>,
    #[serde(default = "default_category")]
    pub category: String,
    #[serde(default)]
    pub security_score: Option<u8>,
    #[serde(default)]
    pub security_severity: Option<String>,
    #[serde(default)]
    pub security_report: Option<serde_json::Value>,
}

fn default_category() -> String {
    "user".to_string()
}

fn default_oversight() -> bool {
    true
}

/// Represents a dynamic workflow loaded from `data/workflows/*.md`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowDefinition {
    pub id: Option<String>,
    pub name: String,
    pub content: String,
    pub doc_url: Option<String>,
    pub tags: Option<Vec<String>>,
    #[serde(default = "default_category")]
    pub category: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookDefinition {
    pub name: String,
    pub description: String,
    pub hook_type: String, // e.g., "pre_validation", "post_analysis"
    pub content: String,
    pub active: bool,
    #[serde(default = "default_category")]
    pub category: String,
}

/// The Skills registry holding in-memory maps of skills and workflows.
pub struct ScriptSkillsRegistry {
    repository: ArtifactRepository,
    pub skills: DashMap<String, SkillDefinition>,
    pub workflows: DashMap<String, WorkflowDefinition>,
    pub hooks: DashMap<String, HookDefinition>,
}

impl ScriptSkillsRegistry {
    pub async fn new() -> Result<Self, AppError> {
        let base_dir = std::env::var("WORKSPACE_ROOT")
            .map(PathBuf::from)
            .unwrap_or_else(|_| {
                if std::env::current_dir()
                    .unwrap_or_default()
                    .ends_with("server-rs")
                {
                    PathBuf::from("..")
                } else {
                    PathBuf::from(".")
                }
            });

        // Resolve workspace_root to its absolute path to prevent traversal/escape issues
        let workspace_root = fs::canonicalize(&base_dir)
            .await
            .unwrap_or_else(|_| base_dir.clone());

        let registry = Self::mock(workspace_root);
        registry.repository.ensure_directories().await?;
        registry.reload_all().await?;
        Ok(registry)
    }

    pub fn mock(workspace_root: PathBuf) -> Self {
        Self {
            repository: ArtifactRepository::new(workspace_root),
            skills: DashMap::new(),
            workflows: DashMap::new(),
            hooks: DashMap::new(),
        }
    }

    pub async fn reload_all(&self) -> Result<(), AppError> {
        sources::RegistrySnapshot::load(&self.repository)
            .await
            .publish(self);
        Ok(())
    }

    async fn save_skill_origin(
        &self,
        skill: SkillDefinition,
        origin: ArtifactOrigin,
    ) -> Result<(), AppError> {
        let mut skill = validation::validate_skill(skill).await?;
        skill.category = origin.category().to_string();
        if origin == ArtifactOrigin::Generated {
            skill.oversight_required = true;
        }
        let content = serde_json::to_vec_pretty(&skill)
            .map_err(|error| AppError::InternalServerError(error.to_string()))?;
        self.repository
            .save(ArtifactKind::Skill, origin, &skill.name, &content)
            .await?;
        self.skills.insert(skill.name.clone(), skill);
        Ok(())
    }

    async fn save_workflow_origin(
        &self,
        mut workflow: WorkflowDefinition,
        origin: ArtifactOrigin,
    ) -> Result<(), AppError> {
        validation::validate_content(&workflow.content, "WORKFLOW.md", "workflow").await?;
        workflow.category = origin.category().to_string();
        self.repository
            .save(
                ArtifactKind::Workflow,
                origin,
                &workflow.name,
                workflow.content.as_bytes(),
            )
            .await?;
        self.workflows.insert(workflow.name.clone(), workflow);
        Ok(())
    }

    async fn save_hook_origin(
        &self,
        mut hook: HookDefinition,
        origin: ArtifactOrigin,
    ) -> Result<(), AppError> {
        validation::validate_content(&hook.content, "HOOK.json", "hook").await?;
        hook.category = origin.category().to_string();
        let content = serde_json::to_vec_pretty(&hook)
            .map_err(|error| AppError::InternalServerError(error.to_string()))?;
        self.repository
            .save(ArtifactKind::Hook, origin, &hook.name, &content)
            .await?;
        self.hooks.insert(hook.name.clone(), hook);
        Ok(())
    }

    pub async fn save_skill(&self, skill: SkillDefinition) -> Result<(), AppError> {
        self.save_skill_origin(skill, ArtifactOrigin::User).await
    }
    pub async fn save_agent_skill(&self, mut skill: SkillDefinition) -> Result<(), AppError> {
        skill.oversight_required = true;
        skill.category = ArtifactOrigin::Generated.category().to_string();
        self.save_skill_origin(skill, ArtifactOrigin::Generated)
            .await
    }
    pub async fn save_workflow(&self, workflow: WorkflowDefinition) -> Result<(), AppError> {
        self.save_workflow_origin(workflow, ArtifactOrigin::User)
            .await
    }
    pub async fn save_agent_workflow(&self, workflow: WorkflowDefinition) -> Result<(), AppError> {
        self.save_workflow_origin(workflow, ArtifactOrigin::Generated)
            .await
    }
    pub async fn save_hook(&self, hook: HookDefinition) -> Result<(), AppError> {
        self.save_hook_origin(hook, ArtifactOrigin::User).await
    }
    pub async fn save_agent_hook(&self, hook: HookDefinition) -> Result<(), AppError> {
        self.save_hook_origin(hook, ArtifactOrigin::Generated).await
    }
    pub async fn delete_skill(&self, name: &str) -> Result<(), AppError> {
        self.repository.delete(ArtifactKind::Skill, name).await;
        self.skills.remove(name);
        Ok(())
    }
    pub async fn delete_workflow(&self, name: &str) -> Result<(), AppError> {
        self.repository.delete(ArtifactKind::Workflow, name).await;
        self.workflows.remove(name);
        Ok(())
    }
    pub async fn delete_hook(&self, name: &str) -> Result<(), AppError> {
        self.repository.delete(ArtifactKind::Hook, name).await;
        self.hooks.remove(name);
        Ok(())
    }

    /// Validates and registers a discovered or imported capability.
    /// Categorizes as "ai" if autonomously discovered, or "user" if manually imported.
    pub async fn register_capability(
        &self,
        cap_type: &str,
        data: serde_json::Value,
        category: &str,
    ) -> Result<String, AppError> {
        let is_agent = category == "agent_generated";
        match cap_type {
            "skill" => {
                let mut skill: SkillDefinition = serde_json::from_value(data)
                    .map_err(|e| AppError::BadRequest(e.to_string()))?;
                let name = skill.name.clone();
                if is_agent {
                    self.save_agent_skill(skill).await?;
                } else {
                    skill.category = category.to_string();
                    self.save_skill(skill).await?;
                }
                Ok(name)
            }
            "workflow" => {
                let mut workflow: WorkflowDefinition = serde_json::from_value(data)
                    .map_err(|e| AppError::BadRequest(e.to_string()))?;
                let name = workflow.name.clone();
                if is_agent {
                    self.save_agent_workflow(workflow).await?;
                } else {
                    workflow.category = category.to_string();
                    self.save_workflow(workflow).await?;
                }
                Ok(name)
            }
            "hook" => {
                let mut hook: HookDefinition = serde_json::from_value(data)
                    .map_err(|e| AppError::BadRequest(e.to_string()))?;
                let name = hook.name.clone();
                if is_agent {
                    self.save_agent_hook(hook).await?;
                } else {
                    hook.category = category.to_string();
                    self.save_hook(hook).await?;
                }
                Ok(name)
            }
            _ => Err(AppError::BadRequest(format!(
                "Unknown capability type: {}",
                cap_type
            ))),
        }
    }
}

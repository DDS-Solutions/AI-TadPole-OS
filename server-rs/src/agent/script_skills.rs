//! @docs ARCHITECTURE:Agent
//!
//! ### AI Assist Note
//! **Dynamic Capabilities**: Orchestrates the discovery and execution of
//! **Custom Skills** (JSON) and **Deterministic Workflows** (Markdown).
//! Features a dual-loader system for standard and agent-specific
//! directories. Supports **Frontmatter Extraction** (YAML) and
//! **Schema Validation** for autonomously discovered toolsets.
//!
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: Invalid YAML frontmatter in `SKILL.md`, duplicate
//!   skill names in the `DashMap`, or `WORKSPACE_ROOT` resolution failure.
//! - **Trace Scope**: `server-rs::agent::script_skills`

use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::path::PathBuf;
use tokio::fs;

/// Represents a dynamic skill loaded from `data/skills/*.json`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillDefinition {
    pub id: Option<String>,
    pub name: String,
    pub description: String,
    pub execution_command: String,
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
    skills_dir: PathBuf,
    workflows_dir: PathBuf,
    hooks_dir: PathBuf,
    agent_skills_dir: PathBuf,
    agent_workflows_dir: PathBuf,
    agent_hooks_dir: PathBuf,
    pub skills: DashMap<String, SkillDefinition>,
    pub workflows: DashMap<String, WorkflowDefinition>,
    pub hooks: DashMap<String, HookDefinition>,
}

impl ScriptSkillsRegistry {
    pub async fn new() -> anyhow::Result<Self> {
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

        let skills_dir = base_dir.join("execution");
        let workflows_dir = base_dir.join("directives");
        let hooks_dir = base_dir.join("hooks");
        let agent_root = skills_dir.join("agent_generated");

        let agent_skills_dir = agent_root.join("skills");
        let agent_workflows_dir = agent_root.join("workflows");
        let agent_hooks_dir = agent_root.join("hooks");

        // Ensure directories exist
        fs::create_dir_all(&skills_dir).await?;
        fs::create_dir_all(&workflows_dir).await?;
        fs::create_dir_all(&hooks_dir).await?;
        fs::create_dir_all(&agent_skills_dir).await?;
        fs::create_dir_all(&agent_workflows_dir).await?;
        fs::create_dir_all(&agent_hooks_dir).await?;

        let registry = Self {
            skills_dir,
            workflows_dir,
            hooks_dir,
            agent_skills_dir,
            agent_workflows_dir,
            agent_hooks_dir,
            skills: DashMap::new(),
            workflows: DashMap::new(),
            hooks: DashMap::new(),
        };

        registry.reload_all().await?;
        Ok(registry)
    }

    /// Create a mock registry for testing
    pub fn mock() -> Self {
        Self {
            skills_dir: PathBuf::from("tmp/skills"),
            workflows_dir: PathBuf::from("tmp/workflows"),
            hooks_dir: PathBuf::from("tmp/hooks"),
            agent_skills_dir: PathBuf::from("tmp/agent_generated/skills"),
            agent_workflows_dir: PathBuf::from("tmp/agent_generated/workflows"),
            agent_hooks_dir: PathBuf::from("tmp/agent_generated/hooks"),
            skills: DashMap::new(),
            workflows: DashMap::new(),
            hooks: DashMap::new(),
        }
    }

    /// Read all defined skills and workflows from disk into memory
    pub async fn reload_all(&self) -> anyhow::Result<()> {
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

        let agent_skills_dir = base_dir.join(".agent").join("skills");
        let agent_workflows_dir = base_dir.join(".agent").join("workflows");

        self.skills.clear();
        self.workflows.clear();
        self.hooks.clear();

        // 1. Load Standard Skills (JSON)
        if let Ok(mut entries) = fs::read_dir(&self.skills_dir).await {
            while let Ok(Some(entry)) = entries.next_entry().await {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("json") {
                    if let Ok(content) = fs::read_to_string(&path).await {
                        if let Ok(skill) = serde_json::from_str::<SkillDefinition>(&content) {
                            self.skills.insert(skill.name.clone(), skill);
                        }
                    }
                }
            }
        }

        // 1b. Load Agent-Generated Skills (JSON)
        if let Ok(mut entries) = fs::read_dir(&self.agent_skills_dir).await {
            while let Ok(Some(entry)) = entries.next_entry().await {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("json") {
                    if let Ok(content) = fs::read_to_string(&path).await {
                        if let Ok(mut skill) = serde_json::from_str::<SkillDefinition>(&content) {
                            skill.category = "agent_generated".to_string(); // Override category
                            self.skills.insert(skill.name.clone(), skill);
                        }
                    }
                }
            }
        }

        // 2. Load Agent Skills (SKILL.md)
        if let Ok(mut entries) = fs::read_dir(&agent_skills_dir).await {
            while let Ok(Some(entry)) = entries.next_entry().await {
                let path = entry.path();
                if path.is_dir() {
                    let skill_md = path.join("SKILL.md");
                    if skill_md.exists() {
                        if let Ok(content) = fs::read_to_string(&skill_md).await {
                            if let Some(skill) = parse_skill_md(&content) {
                                self.skills.insert(skill.name.clone(), skill);
                            }
                        }
                    }
                }
            }
        }

        // 3. Load Standard Workflows (MD)
        if let Ok(mut entries) = fs::read_dir(&self.workflows_dir).await {
            while let Ok(Some(entry)) = entries.next_entry().await {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("md") {
                    if let Ok(content) = fs::read_to_string(&path).await {
                        let name = path.file_stem().unwrap().to_string_lossy().to_string();
                        self.workflows.insert(
                            name.clone(),
                            WorkflowDefinition {
                                id: None,
                                name,
                                content,
                                doc_url: None,
                                tags: None,
                                category: "user".to_string(),
                            },
                        );
                    }
                }
            }
        }

        // 4. Load Agent Workflows (MD)
        if let Ok(mut entries) = fs::read_dir(&agent_workflows_dir).await {
            while let Ok(Some(entry)) = entries.next_entry().await {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("md") {
                    if let Ok(content) = fs::read_to_string(&path).await {
                        let name = path.file_stem().unwrap().to_string_lossy().to_string();
                        self.workflows.insert(
                            name.clone(),
                            WorkflowDefinition {
                                id: None,
                                name,
                                content,
                                doc_url: None,
                                tags: None,
                                category: "user".to_string(),
                            },
                        );
                    }
                }
            }
        }

        // 4b. Load Agent-Generated Workflows (MD)
        if let Ok(mut entries) = fs::read_dir(&self.agent_workflows_dir).await {
            while let Ok(Some(entry)) = entries.next_entry().await {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("md") {
                    if let Ok(content) = fs::read_to_string(&path).await {
                        let name = path.file_stem().unwrap().to_string_lossy().to_string();
                        self.workflows.insert(
                            name.clone(),
                            WorkflowDefinition {
                                id: None,
                                name,
                                content,
                                doc_url: None,
                                tags: None,
                                category: "agent_generated".to_string(),
                            },
                        );
                    }
                }
            }
        }

        // 5. Load Hooks
        if let Ok(mut entries) = fs::read_dir(&self.hooks_dir).await {
            while let Ok(Some(entry)) = entries.next_entry().await {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("json") {
                    if let Ok(content) = fs::read_to_string(&path).await {
                        if let Ok(hook) = serde_json::from_str::<HookDefinition>(&content) {
                            self.hooks.insert(hook.name.clone(), hook);
                        }
                    }
                }
            }
        }

        // 5b. Load Agent-Generated Hooks
        if let Ok(mut entries) = fs::read_dir(&self.agent_hooks_dir).await {
            while let Ok(Some(entry)) = entries.next_entry().await {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("json") {
                    if let Ok(content) = fs::read_to_string(&path).await {
                        if let Ok(mut hook) = serde_json::from_str::<HookDefinition>(&content) {
                            hook.category = "agent_generated".to_string();
                            self.hooks.insert(hook.name.clone(), hook);
                        }
                    }
                }
            }
        }

        Ok(())
    }

    pub async fn save_skill(&self, skill: SkillDefinition) -> anyhow::Result<()> {
        let safe_name = crate::utils::security::sanitize_id(&skill.name);
        let filename = format!("{}.json", safe_name);
        let path = crate::utils::security::validate_path(&self.skills_dir, &filename)?;

        let content = serde_json::to_string_pretty(&skill)?;
        fs::write(&path, content).await?;

        self.skills.insert(skill.name.clone(), skill);
        Ok(())
    }

    /// Saves a skill to the dedicated agent_generated directory.
    pub async fn save_agent_skill(&self, mut skill: SkillDefinition) -> anyhow::Result<()> {
        let safe_name = crate::utils::security::sanitize_id(&skill.name);
        let filename = format!("{}.json", safe_name);
        let path = crate::utils::security::validate_path(&self.agent_skills_dir, &filename)?;

        skill.category = "agent_generated".to_string();
        let content = serde_json::to_string_pretty(&skill)?;
        fs::write(&path, content).await?;

        self.skills.insert(skill.name.clone(), skill);
        Ok(())
    }

    /// Saves a workflow to the dedicated agent_generated directory.
    pub async fn save_agent_workflow(
        &self,
        mut workflow: WorkflowDefinition,
    ) -> anyhow::Result<()> {
        let safe_name = crate::utils::security::sanitize_id(&workflow.name);
        let filename = format!("{}.md", safe_name);
        let path = crate::utils::security::validate_path(&self.agent_workflows_dir, &filename)?;

        workflow.category = "agent_generated".to_string();
        fs::write(&path, &workflow.content).await?;

        self.workflows.insert(workflow.name.clone(), workflow);
        Ok(())
    }

    /// Saves a hook to the dedicated agent_generated directory.
    pub async fn save_agent_hook(&self, mut hook: HookDefinition) -> anyhow::Result<()> {
        let safe_name = crate::utils::security::sanitize_id(&hook.name);
        let filename = format!("{}.json", safe_name);
        let path = crate::utils::security::validate_path(&self.agent_hooks_dir, &filename)?;

        hook.category = "agent_generated".to_string();
        let content = serde_json::to_string_pretty(&hook)?;
        fs::write(&path, content).await?;

        self.hooks.insert(hook.name.clone(), hook);
        Ok(())
    }

    pub async fn delete_skill(&self, name: &str) -> anyhow::Result<()> {
        let safe_name = crate::utils::security::sanitize_id(name);
        let filename = format!("{}.json", safe_name);
        let path = crate::utils::security::validate_path(&self.skills_dir, &filename)?;

        if path.exists() {
            fs::remove_file(path).await?;
        }
        self.skills.remove(name);
        Ok(())
    }

    pub async fn save_workflow(&self, workflow: WorkflowDefinition) -> anyhow::Result<()> {
        let safe_name = crate::utils::security::sanitize_id(&workflow.name);
        let filename = format!("{}.md", safe_name);
        let path = crate::utils::security::validate_path(&self.workflows_dir, &filename)?;

        fs::write(&path, &workflow.content).await?;

        self.workflows.insert(workflow.name.clone(), workflow);
        Ok(())
    }

    pub async fn delete_workflow(&self, name: &str) -> anyhow::Result<()> {
        let safe_name = crate::utils::security::sanitize_id(name);
        let filename = format!("{}.md", safe_name);
        let path = crate::utils::security::validate_path(&self.workflows_dir, &filename)?;

        if path.exists() {
            fs::remove_file(path).await?;
        }
        self.workflows.remove(name);
        Ok(())
    }

    pub async fn save_hook(&self, hook: HookDefinition) -> anyhow::Result<()> {
        let safe_name = crate::utils::security::sanitize_id(&hook.name);
        let filename = format!("{}.json", safe_name);
        let path = crate::utils::security::validate_path(&self.hooks_dir, &filename)?;
        let content = serde_json::to_string_pretty(&hook)?;
        fs::write(&path, content).await?;
        self.hooks.insert(hook.name.clone(), hook);
        Ok(())
    }

    pub async fn delete_hook(&self, name: &str) -> anyhow::Result<()> {
        let safe_name = crate::utils::security::sanitize_id(name);
        let filename = format!("{}.json", safe_name);
        let path = crate::utils::security::validate_path(&self.hooks_dir, &filename)?;
        if path.exists() {
            fs::remove_file(path).await?;
        }
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
    ) -> anyhow::Result<String> {
        let is_agent = category == "agent_generated";
        match cap_type {
            "skill" => {
                let mut skill: SkillDefinition = serde_json::from_value(data)?;
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
                let mut workflow: WorkflowDefinition = serde_json::from_value(data)?;
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
                let mut hook: HookDefinition = serde_json::from_value(data)?;
                let name = hook.name.clone();
                if is_agent {
                    self.save_agent_hook(hook).await?;
                } else {
                    hook.category = category.to_string();
                    self.save_hook(hook).await?;
                }
                Ok(name)
            }
            _ => Err(anyhow::anyhow!("Unknown capability type: {}", cap_type)),
        }
    }
}

pub fn parse_skill_md(content: &str) -> Option<SkillDefinition> {
    if !content.starts_with("---") {
        return None;
    }

    let parts: Vec<&str> = content.split("---").collect();
    if parts.len() < 3 {
        return None;
    }

    let yaml_str = parts[1];
    let body = parts[2..].join("---");

    let metadata: serde_json::Value = serde_yaml::from_str(yaml_str).ok()?;
    let name = metadata
        .get("name")
        .and_then(|v| v.as_str())
        .or_else(|| metadata.get("title").and_then(|v| v.as_str()))
        .ok_or_else(|| anyhow::anyhow!("Missing 'name' or 'title' in frontmatter"))
        .ok()?
        .to_string();
    let description = metadata
        .get("description")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    Some(SkillDefinition {
        id: None,
        name,
        description,
        execution_command: metadata
            .get("command")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        schema: metadata
            .get("schema")
            .cloned()
            .unwrap_or(json!({ "type": "object", "properties": {} })),
        oversight_required: metadata
            .get("oversight")
            .and_then(|v| v.as_bool())
            .unwrap_or(true),
        doc_url: metadata
            .get("doc_url")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()),
        tags: metadata.get("tags").and_then(|v| v.as_array()).map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect()
        }),
        full_instructions: Some(body.trim().to_string()),
        negative_constraints: None,
        verification_script: None,
        category: "user".to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Tests the basic markdown parsing functionality for skills.
    /// This follows industry standards for Arrange-Act-Assert (AAA) pattern.
    #[test]
    fn test_parse_skill_md_basic() {
        // Arrange
        let content = r#"---
name: test_skill
description: A test skill
command: python test.py
oversight: false
tags: ["test", "verify"]
---
This is the body content."#;

        // Act
        let skill = parse_skill_md(content).expect("Should parse valid markdown");

        // Assert
        assert_eq!(skill.name, "test_skill");
        assert_eq!(skill.description, "A test skill");
        assert_eq!(skill.execution_command, "python test.py");
        assert!(!skill.oversight_required);
        assert_eq!(
            skill.tags.unwrap(),
            vec!["test".to_string(), "verify".to_string()]
        );
        assert_eq!(
            skill.full_instructions.unwrap(),
            "This is the body content."
        );
    }

    /// Tests the title fallback mechanism when 'name' is missing in frontmatter.
    #[test]
    fn test_parse_skill_md_title_fallback() {
        // Arrange
        let content = r#"---
title: My Advanced Skill
description: Fallback test
---
Body"#;

        // Act
        let skill = parse_skill_md(content).expect("Should fallback to title");

        // Assert
        assert_eq!(skill.name, "My Advanced Skill");
    }

    /// Tests that invalid markdown (missing frontmatter) returns None.
    #[test]
    fn test_parse_skill_md_invalid() {
        // Arrange
        let content = "Just some random text";

        // Act
        let skill = parse_skill_md(content);

        // Assert
        assert!(
            skill.is_none(),
            "Should return None for invalid markdown structure"
        );
    }

    /// Tests parsing with a complex JSON schema in the frontmatter.
    #[test]
    fn test_parse_skill_md_with_schema() {
        // Arrange
        let content = r#"---
name: schema_skill
schema:
  type: object
  properties:
    query:
      type: string
---
Body"#;

        // Act
        let skill = parse_skill_md(content).expect("Should parse schema");

        // Assert
        assert_eq!(skill.schema["type"], "object");
        assert_eq!(skill.schema["properties"]["query"]["type"], "string");
    }
}

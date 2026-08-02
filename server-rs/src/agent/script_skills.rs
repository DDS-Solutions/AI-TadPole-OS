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

use crate::error::AppError;
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::path::{Path, PathBuf};
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
    workspace_root: PathBuf,
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
    /// ### 🏗️ Core Architecture: Dynamic Capability Registry
    /// Initializes the in-memory registry by scanning specialized workspace
    /// directories for deterministic and autonomous capabilities.
    ///
    /// ### 🧬 Directory Structure: The Neural Lobe
    /// - `execution/`: Standard JSON skills (deterministic tools).
    /// - `directives/`: Markdown workflows (procedural logic).
    /// - `hooks/`: Lifecycle interceptors (post-analysis/pre-validation).
    /// - `agent_generated/`: Autonomous artifacts created by current or past agents.
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

        let skills_dir = workspace_root.join("execution");
        let workflows_dir = workspace_root.join("directives");
        let hooks_dir = workspace_root.join("hooks");
        let agent_root = skills_dir.join("agent_generated");

        let agent_skills_dir = agent_root.join("skills");
        let agent_workflows_dir = agent_root.join("workflows");
        let agent_hooks_dir = agent_root.join("hooks");

        // Ensure directories exist
        fs::create_dir_all(&skills_dir)
            .await
            .map_err(AppError::Io)?;
        fs::create_dir_all(&workflows_dir)
            .await
            .map_err(AppError::Io)?;
        fs::create_dir_all(&hooks_dir).await.map_err(AppError::Io)?;
        fs::create_dir_all(&agent_skills_dir)
            .await
            .map_err(AppError::Io)?;
        fs::create_dir_all(&agent_workflows_dir)
            .await
            .map_err(AppError::Io)?;
        fs::create_dir_all(&agent_hooks_dir)
            .await
            .map_err(AppError::Io)?;

        let registry = Self {
            workspace_root,
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

    /// Create a mock registry for testing with isolated directories.
    pub fn mock(base_dir: PathBuf) -> Self {
        let skills_dir = base_dir.join("execution");
        let workflows_dir = base_dir.join("directives");
        let hooks_dir = base_dir.join("hooks");
        let agent_root = skills_dir.join("agent_generated");

        Self {
            workspace_root: base_dir.clone(),
            skills_dir,
            workflows_dir,
            hooks_dir,
            agent_skills_dir: agent_root.join("skills"),
            agent_workflows_dir: agent_root.join("workflows"),
            agent_hooks_dir: agent_root.join("hooks"),
            skills: DashMap::new(),
            workflows: DashMap::new(),
            hooks: DashMap::new(),
        }
    }

    /// ### 📡 Synchronization: reload_all
    /// Scans all local and sovereign directories to sync the in-memory registry
    /// with the physical disk state.
    ///
    pub async fn reload_all(&self) -> Result<(), AppError> {
        let built_in_agent_skills_dir = self.workspace_root.join(".agent").join("skills");
        let built_in_agent_workflows_dir = self.workspace_root.join(".agent").join("workflows");

        // Ensure all scan directories exist to prevent OS Error 3 warnings
        let _ = fs::create_dir_all(&self.skills_dir).await;
        let _ = fs::create_dir_all(&self.agent_skills_dir).await;
        let _ = fs::create_dir_all(&self.workflows_dir).await;
        let _ = fs::create_dir_all(&self.agent_workflows_dir).await;
        let _ = fs::create_dir_all(&self.hooks_dir).await;
        let _ = fs::create_dir_all(&self.agent_hooks_dir).await;
        let _ = fs::create_dir_all(&built_in_agent_skills_dir).await;
        let _ = fs::create_dir_all(&built_in_agent_workflows_dir).await;

        let mut temp_skills = std::collections::HashMap::new();
        let mut temp_workflows = std::collections::HashMap::new();
        let mut temp_hooks = std::collections::HashMap::new();

        // 1. Load Standard Skills (JSON)
        match fs::read_dir(&self.skills_dir).await {
            Ok(mut entries) => {
                while let Ok(Some(entry)) = entries.next_entry().await {
                    let path = entry.path();
                    if path.extension().and_then(|s| s.to_str()) == Some("json") {
                        match read_file_safe(&path).await {
                            Ok(content) => {
                                match serde_json::from_str::<SkillDefinition>(&content) {
                                    Ok(skill) => {
                                        temp_skills.insert(skill.name.clone(), skill);
                                    }
                                    Err(e) => {
                                        tracing::warn!(
                                            "⚠️ Failed to parse skill JSON at {:?}: {}",
                                            path,
                                            e
                                        );
                                    }
                                }
                            }
                            Err(e) => {
                                tracing::warn!("⚠️ Failed to read skill file {:?}: {}", path, e);
                            }
                        }
                    }
                }
            }
            Err(e) => {
                tracing::warn!(
                    "⚠️ Failed to read skills directory {:?}: {}",
                    self.skills_dir,
                    e
                );
            }
        }

        // 1b. Load Agent-Generated Skills (JSON)
        match fs::read_dir(&self.agent_skills_dir).await {
            Ok(mut entries) => {
                while let Ok(Some(entry)) = entries.next_entry().await {
                    let path = entry.path();
                    if path.extension().and_then(|s| s.to_str()) == Some("json") {
                        match read_file_safe(&path).await {
                            Ok(content) => {
                                match serde_json::from_str::<SkillDefinition>(&content) {
                                    Ok(mut skill) => {
                                        skill.category = "agent_generated".to_string(); // Override category
                                        temp_skills.insert(skill.name.clone(), skill);
                                    }
                                    Err(e) => {
                                        tracing::warn!(
                                            "⚠️ Failed to parse agent skill JSON at {:?}: {}",
                                            path,
                                            e
                                        );
                                    }
                                }
                            }
                            Err(e) => {
                                tracing::warn!(
                                    "⚠️ Failed to read agent skill file {:?}: {}",
                                    path,
                                    e
                                );
                            }
                        }
                    }
                }
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => {
                tracing::warn!(
                    "⚠️ Failed to read agent skills directory {:?}: {}",
                    self.agent_skills_dir,
                    e
                );
            }
        }

        // 2. Load Agent Skills (SKILL.md)
        match fs::read_dir(&built_in_agent_skills_dir).await {
            Ok(mut entries) => {
                while let Ok(Some(entry)) = entries.next_entry().await {
                    let path = entry.path();
                    if path.is_dir() {
                        let skill_md = path.join("SKILL.md");
                        if skill_md.exists() {
                            match read_file_safe(&skill_md).await {
                                Ok(content) => {
                                    if let Some(skill) = parse_skill_md(&content) {
                                        temp_skills.insert(skill.name.clone(), skill);
                                    } else {
                                        tracing::warn!(
                                            "⚠️ Failed to parse built-in skill MD at {:?}",
                                            skill_md
                                        );
                                    }
                                }
                                Err(e) => {
                                    tracing::warn!(
                                        "⚠️ Failed to read built-in skill MD file {:?}: {}",
                                        skill_md,
                                        e
                                    );
                                }
                            }
                        }
                    }
                }
            }
            Err(e) => {
                tracing::warn!(
                    "⚠️ Failed to read built-in agent skills directory {:?}: {}",
                    built_in_agent_skills_dir,
                    e
                );
            }
        }

        // 3. Load Standard Workflows (MD)
        match fs::read_dir(&self.workflows_dir).await {
            Ok(mut entries) => {
                while let Ok(Some(entry)) = entries.next_entry().await {
                    let path = entry.path();
                    if path.extension().and_then(|s| s.to_str()) == Some("md") {
                        match read_file_safe(&path).await {
                            Ok(content) => {
                                let name = match path.file_stem() {
                                    Some(s) => s.to_string_lossy().to_string(),
                                    None => continue,
                                };
                                temp_workflows.insert(
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
                            Err(e) => {
                                tracing::warn!("⚠️ Failed to read workflow file {:?}: {}", path, e);
                            }
                        }
                    }
                }
            }
            Err(e) => {
                tracing::warn!(
                    "⚠️ Failed to read workflows directory {:?}: {}",
                    self.workflows_dir,
                    e
                );
            }
        }

        // 4. Load Agent Workflows (MD)
        match fs::read_dir(&built_in_agent_workflows_dir).await {
            Ok(mut entries) => {
                while let Ok(Some(entry)) = entries.next_entry().await {
                    let path = entry.path();
                    if path.extension().and_then(|s| s.to_str()) == Some("md") {
                        match read_file_safe(&path).await {
                            Ok(content) => {
                                let name = match path.file_stem() {
                                    Some(s) => s.to_string_lossy().to_string(),
                                    None => continue,
                                };
                                temp_workflows.insert(
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
                            Err(e) => {
                                tracing::warn!(
                                    "⚠️ Failed to read agent workflow file {:?}: {}",
                                    path,
                                    e
                                );
                            }
                        }
                    }
                }
            }
            Err(e) => {
                tracing::warn!(
                    "⚠️ Failed to read built-in agent workflows directory {:?}: {}",
                    built_in_agent_workflows_dir,
                    e
                );
            }
        }

        // 4b. Load Agent-Generated Workflows (MD)
        match fs::read_dir(&self.agent_workflows_dir).await {
            Ok(mut entries) => {
                while let Ok(Some(entry)) = entries.next_entry().await {
                    let path = entry.path();
                    if path.extension().and_then(|s| s.to_str()) == Some("md") {
                        match read_file_safe(&path).await {
                            Ok(content) => {
                                let name = match path.file_stem() {
                                    Some(s) => s.to_string_lossy().to_string(),
                                    None => continue,
                                };
                                temp_workflows.insert(
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
                            Err(e) => {
                                tracing::warn!(
                                    "⚠️ Failed to read agent-generated workflow file {:?}: {}",
                                    path,
                                    e
                                );
                            }
                        }
                    }
                }
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => {
                tracing::warn!(
                    "⚠️ Failed to read agent workflows directory {:?}: {}",
                    self.agent_workflows_dir,
                    e
                );
            }
        }

        // 5. Load Hooks
        match fs::read_dir(&self.hooks_dir).await {
            Ok(mut entries) => {
                while let Ok(Some(entry)) = entries.next_entry().await {
                    let path = entry.path();
                    if path.extension().and_then(|s| s.to_str()) == Some("json") {
                        match read_file_safe(&path).await {
                            Ok(content) => match serde_json::from_str::<HookDefinition>(&content) {
                                Ok(hook) => {
                                    temp_hooks.insert(hook.name.clone(), hook);
                                }
                                Err(e) => {
                                    tracing::warn!(
                                        "⚠️ Failed to parse hook JSON at {:?}: {}",
                                        path,
                                        e
                                    );
                                }
                            },
                            Err(e) => {
                                tracing::warn!("⚠️ Failed to read hook file {:?}: {}", path, e);
                            }
                        }
                    }
                }
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => {
                tracing::warn!(
                    "⚠️ Failed to read hooks directory {:?}: {}",
                    self.hooks_dir,
                    e
                );
            }
        }

        // 5b. Load Agent-Generated Hooks
        match fs::read_dir(&self.agent_hooks_dir).await {
            Ok(mut entries) => {
                while let Ok(Some(entry)) = entries.next_entry().await {
                    let path = entry.path();
                    if path.extension().and_then(|s| s.to_str()) == Some("json") {
                        match read_file_safe(&path).await {
                            Ok(content) => match serde_json::from_str::<HookDefinition>(&content) {
                                Ok(mut hook) => {
                                    hook.category = "agent_generated".to_string();
                                    temp_hooks.insert(hook.name.clone(), hook);
                                }
                                Err(e) => {
                                    tracing::warn!(
                                        "⚠️ Failed to parse agent hook JSON at {:?}: {}",
                                        path,
                                        e
                                    );
                                }
                            },
                            Err(e) => {
                                tracing::warn!(
                                    "⚠️ Failed to read agent hook file {:?}: {}",
                                    path,
                                    e
                                );
                            }
                        }
                    }
                }
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => {
                tracing::warn!(
                    "⚠️ Failed to read agent hooks directory {:?}: {}",
                    self.agent_hooks_dir,
                    e
                );
            }
        }

        // Apply loaded maps using synchronized retain and insert (reordered to eliminate race window)
        let temp_skill_keys: std::collections::HashSet<String> =
            temp_skills.keys().cloned().collect();
        for (k, v) in temp_skills {
            self.skills.insert(k, v);
        }
        self.skills.retain(|k, _| temp_skill_keys.contains(k));

        let temp_workflow_keys: std::collections::HashSet<String> =
            temp_workflows.keys().cloned().collect();
        for (k, v) in temp_workflows {
            self.workflows.insert(k, v);
        }
        self.workflows.retain(|k, _| temp_workflow_keys.contains(k));

        let temp_hook_keys: std::collections::HashSet<String> =
            temp_hooks.keys().cloned().collect();
        for (k, v) in temp_hooks {
            self.hooks.insert(k, v);
        }
        self.hooks.retain(|k, _| temp_hook_keys.contains(k));

        Ok(())
    }

    async fn save_skill_internal(
        &self,
        mut skill: SkillDefinition,
        target_dir: &Path,
        category: &str,
    ) -> Result<(), AppError> {
        let mut content_to_scan = String::new();
        if let Some(ref inst) = skill.full_instructions {
            content_to_scan.push_str(inst);
            content_to_scan.push('\n');
        }
        content_to_scan.push_str(&skill.execution_command);
        content_to_scan.push('\n');
        if let Some(ref script) = skill.verification_script {
            content_to_scan.push_str(script);
            content_to_scan.push('\n');
        }
        if let Some(ref constraints) = skill.negative_constraints {
            for c in constraints {
                content_to_scan.push_str(c);
                content_to_scan.push('\n');
            }
        }
        content_to_scan.push_str(&skill.schema.to_string());

        let scan_res = crate::security::skillspector::scan_content(&content_to_scan, "SKILL.md")?;
        skill.security_score = Some(scan_res.risk_score);
        skill.security_severity = Some(scan_res.risk_severity.clone());
        if scan_res.risk_score >= 50 {
            return Err(AppError::Forbidden(format!(
                "Security audit failed: skill has a high-risk security score of {} (Severity: {}). Registration rejected.",
                scan_res.risk_score, scan_res.risk_severity
            )));
        }
        skill.security_report = Some(
            serde_json::to_value(&scan_res)
                .unwrap_or(serde_json::json!({ "filtered_findings": [] })),
        );

        let safe_name = crate::utils::security::sanitize_id(&skill.name);
        let filename = format!("{}.json", safe_name);
        let path = crate::utils::security::validate_path(target_dir, &filename)
            .map_err(|e| AppError::InternalServerError(e.to_string()))?;

        skill.category = category.to_string();
        let content = serde_json::to_string_pretty(&skill)
            .map_err(|e| AppError::InternalServerError(e.to_string()))?;
        write_atomic(&path, content.as_bytes()).await?;

        self.skills.insert(skill.name.clone(), skill);
        Ok(())
    }

    async fn save_workflow_internal(
        &self,
        mut workflow: WorkflowDefinition,
        target_dir: &Path,
        category: &str,
    ) -> Result<(), AppError> {
        let scan_res =
            crate::security::skillspector::scan_content(&workflow.content, "WORKFLOW.md")?;
        if scan_res.risk_score >= 50 {
            return Err(AppError::Forbidden(format!(
                "Security audit failed: workflow has a high-risk security score of {} (Severity: {}). Registration rejected.",
                scan_res.risk_score, scan_res.risk_severity
            )));
        }

        let safe_name = crate::utils::security::sanitize_id(&workflow.name);
        let filename = format!("{}.md", safe_name);
        let path = crate::utils::security::validate_path(target_dir, &filename)
            .map_err(|e| AppError::InternalServerError(e.to_string()))?;

        workflow.category = category.to_string();
        write_atomic(&path, workflow.content.as_bytes()).await?;

        self.workflows.insert(workflow.name.clone(), workflow);
        Ok(())
    }

    async fn save_hook_internal(
        &self,
        mut hook: HookDefinition,
        target_dir: &Path,
        category: &str,
    ) -> Result<(), AppError> {
        let scan_res = crate::security::skillspector::scan_content(&hook.content, "HOOK.json")?;
        if scan_res.risk_score >= 50 {
            return Err(AppError::Forbidden(format!(
                "Security audit failed: hook has a high-risk security score of {} (Severity: {}). Registration rejected.",
                scan_res.risk_score, scan_res.risk_severity
            )));
        }

        let safe_name = crate::utils::security::sanitize_id(&hook.name);
        let filename = format!("{}.json", safe_name);
        let path = crate::utils::security::validate_path(target_dir, &filename)
            .map_err(|e| AppError::InternalServerError(e.to_string()))?;

        hook.category = category.to_string();
        let content = serde_json::to_string_pretty(&hook)
            .map_err(|e| AppError::InternalServerError(e.to_string()))?;
        write_atomic(&path, content.as_bytes()).await?;

        self.hooks.insert(hook.name.clone(), hook);
        Ok(())
    }

    pub async fn save_skill(&self, skill: SkillDefinition) -> Result<(), AppError> {
        self.save_skill_internal(skill, &self.skills_dir, "user").await
    }

    /// Saves a skill to the dedicated agent_generated directory.
    pub async fn save_agent_skill(&self, skill: SkillDefinition) -> Result<(), AppError> {
        self.save_skill_internal(skill, &self.agent_skills_dir, "agent_generated").await
    }

    pub async fn save_workflow(&self, workflow: WorkflowDefinition) -> Result<(), AppError> {
        self.save_workflow_internal(workflow, &self.workflows_dir, "user").await
    }

    /// Saves a workflow to the dedicated agent_generated directory.
    pub async fn save_agent_workflow(
        &self,
        workflow: WorkflowDefinition,
    ) -> Result<(), AppError> {
        self.save_workflow_internal(workflow, &self.agent_workflows_dir, "agent_generated").await
    }

    pub async fn save_hook(&self, hook: HookDefinition) -> Result<(), AppError> {
        self.save_hook_internal(hook, &self.hooks_dir, "user").await
    }

    /// Saves a hook to the dedicated agent_generated directory.
    pub async fn save_agent_hook(&self, hook: HookDefinition) -> Result<(), AppError> {
        self.save_hook_internal(hook, &self.agent_hooks_dir, "agent_generated").await
    }

    pub async fn delete_skill(&self, name: &str) -> Result<(), AppError> {
        let safe_name = crate::utils::security::sanitize_id(name);
        let filename = format!("{}.json", safe_name);
        let path = crate::utils::security::validate_path(&self.skills_dir, &filename)
            .map_err(|e| AppError::InternalServerError(e.to_string()))?;

        delete_safe(&path).await?;
        self.skills.remove(name);
        Ok(())
    }

    pub async fn delete_workflow(&self, name: &str) -> Result<(), AppError> {
        let safe_name = crate::utils::security::sanitize_id(name);
        let filename = format!("{}.md", safe_name);
        let path = crate::utils::security::validate_path(&self.workflows_dir, &filename)
            .map_err(|e| AppError::InternalServerError(e.to_string()))?;

        delete_safe(&path).await?;
        self.workflows.remove(name);
        Ok(())
    }

    pub async fn delete_hook(&self, name: &str) -> Result<(), AppError> {
        let safe_name = crate::utils::security::sanitize_id(name);
        let filename = format!("{}.json", safe_name);
        let path = crate::utils::security::validate_path(&self.hooks_dir, &filename)
            .map_err(|e| AppError::InternalServerError(e.to_string()))?;

        delete_safe(&path).await?;
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

async fn read_file_safe(path: &Path) -> Result<String, AppError> {
    let meta = fs::symlink_metadata(path)
        .await
        .map_err(|e| AppError::Io(e))?;

    if meta.file_type().is_symlink() {
        return Err(AppError::Forbidden(
            "Security boundary: Symlinks are not allowed".into(),
        ));
    }
    if meta.len() > 1_048_576 {
        return Err(AppError::Forbidden(format!(
            "Security boundary: File size exceeds limit of 1 MiB ({} bytes)",
            meta.len()
        )));
    }

    let content = fs::read_to_string(path)
        .await
        .map_err(|e| AppError::Io(e))?;
    Ok(content)
}

async fn write_atomic(path: &Path, content: &[u8]) -> Result<(), AppError> {
    let parent = path.parent().ok_or_else(|| {
        AppError::InternalServerError("Target path has no parent directory".to_string())
    })?;
    let temp_name = format!("tmp_{}.tmp", uuid::Uuid::new_v4());
    let temp_path = parent.join(temp_name);

    if let Err(e) = fs::write(&temp_path, content).await {
        let _ = fs::remove_file(&temp_path).await;
        return Err(AppError::Io(e));
    }

    if let Err(e) = fs::rename(&temp_path, path).await {
        let _ = fs::remove_file(&temp_path).await;
        return Err(AppError::Io(e));
    }
    Ok(())
}

async fn delete_safe(path: &Path) -> Result<(), AppError> {
    match fs::symlink_metadata(path).await {
        Ok(meta) => {
            if meta.file_type().is_symlink() {
                return Err(AppError::Forbidden(
                    "Security boundary: Symlinks are not allowed".into(),
                ));
            }
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            return Ok(());
        }
        Err(e) => return Err(AppError::Io(e)),
    }

    match fs::remove_file(path).await {
        Ok(_) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(AppError::Io(e)),
    }
}

/// ### 🧪 Logic: Semantic Skill Extraction (parse_skill_md)
/// Parses a self-describing `SKILL.md` file using YAML frontmatter extraction.
///
/// ### 🧬 Rationale: Self-Documenting Swarms
/// Allows agents to discover toolsets that include their own documentation,
/// schema, and execution commands in a single human-readable Markdown file.
/// Following the industry standard for SSG (Static Site Generator) metadata.
pub fn parse_skill_md(content: &str) -> Option<SkillDefinition> {
    let content_trimmed = content.trim_start();
    let lines = content_trimmed.lines();

    let mut yaml_lines = Vec::new();
    let mut body_lines = Vec::new();
    let mut found_start = false;
    let mut found_end = false;

    for line in lines {
        if !found_start {
            if line.trim() == "---" {
                found_start = true;
            }
        } else if !found_end {
            if line.trim() == "---" {
                found_end = true;
            } else {
                yaml_lines.push(line);
            }
        } else {
            body_lines.push(line);
        }
    }

    if !found_start || !found_end {
        return None;
    }

    let yaml_str = yaml_lines.join("\n");
    let body = body_lines.join("\n");

    if yaml_str.len() > 65536 {
        tracing::warn!(
            "⚠️ Guard: frontmatter size exceeds limit of 64 KiB ({} bytes)",
            yaml_str.len()
        );
        return None;
    }

    let metadata: serde_json::Value = serde_yaml::from_str(&yaml_str).ok()?;
    let name = metadata
        .get("name")
        .and_then(|v| v.as_str())
        .or_else(|| metadata.get("title").and_then(|v| v.as_str()))
        .or(None)?;

    let description = metadata
        .get("description")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    Some(SkillDefinition {
        id: None,
        name: name.to_string(),
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
        security_score: None,
        security_severity: None,
        security_report: None,
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

    #[test]
    fn test_parse_skill_md_robustness() {
        let content = " \n\n  ---\nname: test_whitespace\n---\nbody";
        let skill = parse_skill_md(content).expect("Should parse with leading whitespace");
        assert_eq!(skill.name, "test_whitespace");
        assert_eq!(skill.full_instructions.unwrap(), "body");
    }

    #[test]
    fn test_parse_skill_md_body_separators() {
        let content = "---\nname: test_separators\n---\nbody\n---\ninner\n---\nmore";
        let skill = parse_skill_md(content).expect("Should parse with separators in body");
        assert_eq!(skill.name, "test_separators");
        assert_eq!(
            skill.full_instructions.unwrap(),
            "body\n---\ninner\n---\nmore"
        );
    }

    #[test]
    fn test_parse_skill_md_size_limit() {
        let large_yaml = "a: ".repeat(40000);
        let content = format!("---\nname: test_large\n{}---\nbody", large_yaml);
        let skill = parse_skill_md(&content);
        assert!(skill.is_none(), "Should reject extremely large YAML block");
    }
}

// Metadata: [script_skills]

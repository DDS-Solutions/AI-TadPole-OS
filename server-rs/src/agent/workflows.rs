//! @docs ARCHITECTURE:Agent
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Agent Runner / workflows
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Type-safe state handling and bounded execution without unhandled panics.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: `AppError::BadRequest`, `AppError::NotFound`, `AppError::Io`
//! - **Telemetry Targets**: none declared
//! - **Witness Tests**: `workflows::tests::*`

use crate::error::AppError;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowStep {
    pub id: String,
    pub title: String,
    pub instruction: String,
    pub tool_required: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowExecutionState {
    pub workflow_name: String,
    pub current_step_index: usize,
    pub steps: Vec<WorkflowStep>,
    pub results: std::collections::HashMap<String, serde_json::Value>,
    pub status: String, // "running", "completed", "failed"
}

impl WorkflowExecutionState {
    pub fn new(name: String, content: &str) -> Result<Self, AppError> {
        let steps = parse_workflow_markdown(content)?;
        Ok(Self {
            workflow_name: name,
            current_step_index: 0,
            steps,
            results: std::collections::HashMap::new(),
            status: "running".to_string(),
        })
    }

    pub fn current_step(&self) -> Option<&WorkflowStep> {
        self.steps.get(self.current_step_index)
    }

    pub fn advance(&mut self) {
        self.current_step_index += 1;
        if self.current_step_index >= self.steps.len() {
            self.status = "completed".to_string();
        }
    }
}

/// Simple parser to extract steps from Markdown SOPs.
/// Looks for H2 or H3 headers as step boundaries.
fn parse_workflow_markdown(content: &str) -> Result<Vec<WorkflowStep>, AppError> {
    let mut steps = Vec::new();
    let mut current_title = String::new();
    let mut current_content = Vec::new();

    for line in content.lines() {
        if line.starts_with("## ") || line.starts_with("### ") {
            if !current_title.is_empty() {
                steps.push(WorkflowStep {
                    id: format!("step-{}", steps.len()),
                    title: current_title.clone(),
                    instruction: current_content.join("\n").trim().to_string(),
                    tool_required: None, // Could parse from tags like [TOOL:search]
                });
            }
            current_title = line.trim_start_matches('#').trim().to_string();
            current_content.clear();
        } else {
            current_content.push(line);
        }
    }

    // Push last step
    if !current_title.is_empty() {
        steps.push(WorkflowStep {
            id: format!("step-{}", steps.len()),
            title: current_title,
            instruction: current_content.join("\n").trim().to_string(),
            tool_required: None,
        });
    }

    if steps.is_empty() {
        return Err(AppError::BadRequest(
            "No steps found in workflow markdown. Ensure you use ## or ### headers for steps."
                .to_string(),
        ));
    }

    Ok(steps)
}

pub async fn load_workflow(
    base_dir: &std::path::Path,
    name: &str,
) -> Result<WorkflowExecutionState, AppError> {
    validate_workflow_name(name)?;

    // `directives/` is the canonical workflow registry used by the installer and
    // ScriptSkillRegistry. Keep the legacy location as a read-only compatibility
    // fallback for workspaces created before the registry was unified.
    let file_name = format!("{}.md", name);
    let canonical_path = base_dir.join("directives").join(&file_name);
    let legacy_path = base_dir.join("data").join("workflows").join(&file_name);
    let path = if tokio::fs::try_exists(&canonical_path)
        .await
        .map_err(AppError::Io)?
    {
        canonical_path
    } else if tokio::fs::try_exists(&legacy_path)
        .await
        .map_err(AppError::Io)?
    {
        legacy_path
    } else {
        return Err(AppError::NotFound(format!(
            "Workflow file '{}' was not found in directives/ or data/workflows/",
            file_name
        )));
    };

    let content = tokio::fs::read_to_string(path)
        .await
        .map_err(AppError::Io)?;
    WorkflowExecutionState::new(name.to_string(), &content)
}

fn validate_workflow_name(name: &str) -> Result<(), AppError> {
    if name.is_empty()
        || !name.chars().all(|character| {
            character.is_ascii_alphanumeric() || character == '-' || character == '_'
        })
    {
        return Err(AppError::BadRequest(
            "Workflow names may only contain ASCII letters, numbers, '-' and '_'".to_string(),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn loads_installed_workflow_from_directives() {
        let temp = tempfile::tempdir().unwrap();
        let directives = temp.path().join("directives");
        tokio::fs::create_dir_all(&directives).await.unwrap();
        tokio::fs::write(
            directives.join("installed_flow.md"),
            "## Inspect\nCheck the installed assets.\n\n## Report\nReturn the receipt.",
        )
        .await
        .unwrap();

        let workflow = load_workflow(temp.path(), "installed_flow").await.unwrap();

        assert_eq!(workflow.workflow_name, "installed_flow");
        assert_eq!(workflow.steps.len(), 2);
        assert_eq!(workflow.steps[0].title, "Inspect");
    }

    #[tokio::test]
    async fn falls_back_to_legacy_workflow_location() {
        let temp = tempfile::tempdir().unwrap();
        let legacy = temp.path().join("data").join("workflows");
        tokio::fs::create_dir_all(&legacy).await.unwrap();
        tokio::fs::write(
            legacy.join("legacy_flow.md"),
            "## Run\nUse the legacy flow.",
        )
        .await
        .unwrap();

        let workflow = load_workflow(temp.path(), "legacy_flow").await.unwrap();

        assert_eq!(workflow.steps.len(), 1);
        assert_eq!(workflow.steps[0].title, "Run");
    }

    #[tokio::test]
    async fn rejects_workflow_path_traversal() {
        let temp = tempfile::tempdir().unwrap();
        let error = load_workflow(temp.path(), "../outside").await.unwrap_err();

        assert!(matches!(error, AppError::BadRequest(_)));
    }
}

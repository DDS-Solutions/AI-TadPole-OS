use anyhow::{anyhow, Result};
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
    pub fn new(name: String, content: &str) -> Result<Self> {
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
fn parse_workflow_markdown(content: &str) -> Result<Vec<WorkflowStep>> {
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
        return Err(anyhow!(
            "No steps found in workflow markdown. Ensure you use ## or ### headers for steps."
        ));
    }

    Ok(steps)
}

pub async fn load_workflow(base_dir: &std::path::Path, name: &str) -> Result<WorkflowExecutionState> {
    let path = base_dir.join("data").join("workflows").join(format!("{}.md", name));
    if !path.exists() {
        return Err(anyhow!("Workflow file not found: {:?}", path));
    }

    let content = tokio::fs::read_to_string(path).await?;
    WorkflowExecutionState::new(name.to_string(), &content)
}

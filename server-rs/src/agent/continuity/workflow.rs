//! @docs ARCHITECTURE:State
//! 
//! ### AI Assist Note
//! **Continuity Workflow Engine**: Orchestrates the deterministic 
//! sequence of agent tasks and long-running state machine 
//! resumption. Features **Multi-Step Orchestration**: enables the 
//! piping of results between agents using template placeholders 
//! (`{{context_keys}}`). Implements **Durable Run Persistence**: 
//! every workflow step is committed to the `workflow_runs` and 
//! `workflow_step_runs` tables, ensuring that the engine can 
//! reconstruct the execution path after a system restart. AI agents 
//! should utilize the `context` object to maintain state across 
//! asynchronous task boundaries (CONT-03).
//! 
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: Placeholder injection failures due to missing 
//!   context keys, step-order conflicts in the database, or agent 
//!   timeouts during high-concurrency workflow bursts.
//! - **Trace Scope**: `server-rs::agent::continuity::workflow`

use crate::agent::runner::AgentRunner;
use crate::agent::types::TaskPayload;
use crate::state::AppState;
use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::Row;
use std::sync::Arc;
use uuid::Uuid;

/// Defines a deterministic sequence of agent tasks.
///
/// Workflows enable multi-step orchestration where the state/output of one agent
/// can be passed to the next using template placeholders (e.g., {{previous_step_result}}).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workflow {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// A single execution unit within a Workflow.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowStep {
    pub id: String,
    pub workflow_id: String,
    pub agent_id: String,
    /// The execution order (lower numbers run first).
    pub step_order: i32,
    pub name: String,
    /// The prompt sent to the agent, supporting {{context_keys}} injection.
    pub prompt_template: String,
    pub config: Option<serde_json::Value>,
}

/// The core engine responsible for executing and persisting workflows.
pub struct WorkflowEngine {
    state: Arc<AppState>,
}

impl WorkflowEngine {
    pub fn new(state: Arc<AppState>) -> Self {
        Self { state }
    }

    /// Lists all workflows
    pub async fn list_workflows(&self) -> Result<Vec<Workflow>> {
        let rows = sqlx::query("SELECT * FROM workflows ORDER BY created_at DESC")
            .fetch_all(&self.state.resources.pool)
            .await?;

        let mut workflows = Vec::new();
        for row in rows {
            workflows.push(Workflow {
                id: row.get::<String, _>("id"),
                name: row.get::<String, _>("name"),
                description: row.get::<Option<String>, _>("description"),
                enabled: row.get::<i64, _>("enabled") != 0,
                created_at: row.get::<DateTime<Utc>, _>("created_at"),
                updated_at: row.get::<DateTime<Utc>, _>("updated_at"),
            });
        }
        Ok(workflows)
    }

    /// Creates a new workflow
    pub async fn create_workflow(
        &self,
        name: String,
        description: Option<String>,
    ) -> Result<Workflow> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now();

        sqlx::query("INSERT INTO workflows (id, name, description, enabled, created_at, updated_at) VALUES (?1, ?2, ?3, 1, ?4, ?5)")
            .bind(&id)
            .bind(&name)
            .bind(&description)
            .bind(now)
            .bind(now)
            .execute(&self.state.resources.pool)
            .await?;

        Ok(Workflow {
            id,
            name,
            description,
            enabled: true,
            created_at: now,
            updated_at: now,
        })
    }

    /// Adds a step to a workflow
    pub async fn add_step(
        &self,
        workflow_id: &str,
        agent_id: &str,
        name: String,
        prompt_template: String,
        step_order: i32,
    ) -> Result<WorkflowStep> {
        let id = Uuid::new_v4().to_string();

        sqlx::query("INSERT INTO workflow_steps (id, workflow_id, agent_id, step_order, name, prompt_template) VALUES (?1, ?2, ?3, ?4, ?5, ?6)")
            .bind(&id)
            .bind(workflow_id)
            .bind(agent_id)
            .bind(step_order)
            .bind(&name)
            .bind(&prompt_template)
            .execute(&self.state.resources.pool)
            .await?;

        Ok(WorkflowStep {
            id,
            workflow_id: workflow_id.to_string(),
            agent_id: agent_id.to_string(),
            step_order,
            name,
            prompt_template,
            config: None,
        })
    }

    /// Deletes a workflow
    pub async fn delete_workflow(&self, id: &str) -> Result<()> {
        sqlx::query("DELETE FROM workflows WHERE id = ?1")
            .bind(id)
            .execute(&self.state.resources.pool)
            .await?;
        Ok(())
    }

    /// Fetches steps for a workflow ordered by step_order
    async fn get_workflow_steps(&self, workflow_id: &str) -> Result<Vec<WorkflowStep>> {
        let rows = sqlx::query(
            "SELECT * FROM workflow_steps WHERE workflow_id = ?1 ORDER BY step_order ASC",
        )
        .bind(workflow_id)
        .fetch_all(&self.state.resources.pool)
        .await?;

        let mut steps = Vec::new();
        for row in rows {
            steps.push(WorkflowStep {
                id: row.get::<String, _>("id"),
                workflow_id: row.get::<String, _>("workflow_id"),
                agent_id: row.get::<String, _>("agent_id"),
                step_order: row.get::<i32, _>("step_order"),
                name: row.get::<String, _>("name"),
                prompt_template: row.get::<String, _>("prompt_template"),
                config: row
                    .get::<Option<String>, _>("config")
                    .and_then(|s| serde_json::from_str(&s).ok()),
            });
        }
        Ok(steps)
    }

    /// Orchestrates a full execution run of a workflow.
    ///
    /// This method:
    /// 1. Validates the workflow and retrieves its steps.
    /// 2. Creates a durable run record in the database.
    /// 3. Iterates through steps, injecting the shared `context` into prompts.
    /// 4. Triggers the AgentRunner for each step and captures output.
    /// 5. Updates the shared context for the next step in the sequence.
    pub async fn run_workflow(
        &self,
        workflow_id: &str,
        initial_context: serde_json::Value,
    ) -> Result<String> {
        let row: sqlx::sqlite::SqliteRow = sqlx::query::<sqlx::Sqlite>("SELECT * FROM workflows WHERE id = ?1")
            .bind(workflow_id)
            .fetch_one(&self.state.resources.pool)
            .await?;

        if row.get::<i64, _>("enabled") == 0 {
            return Err(anyhow!("Workflow is disabled"));
        }

        let steps = self.get_workflow_steps(workflow_id).await?;
        if steps.is_empty() {
            return Err(anyhow!("Workflow has no steps"));
        }

        let run_id = Uuid::new_v4().to_string();
        let now = Utc::now();

        sqlx::query("INSERT INTO workflow_runs (id, workflow_id, started_at, status, current_step, context) VALUES (?1, ?2, ?3, 'running', 0, ?4)")
            .bind(&run_id)
            .bind(workflow_id)
            .bind(now)
            .bind(serde_json::to_string(&initial_context)?)
            .execute(&self.state.resources.pool)
            .await?;

        let mut context = initial_context;
        let mut final_output = String::new();

        for (i, step) in steps.iter().enumerate() {
            tracing::info!(
                "🔄 [Workflow] Running step {}/{} for workflow {}: {}",
                i + 1,
                steps.len(),
                workflow_id,
                step.name
            );

            // Update current step
            sqlx::query("UPDATE workflow_runs SET current_step = ?1 WHERE id = ?2")
                .bind(i as i32)
                .bind(&run_id)
                .execute(&self.state.resources.pool)
                .await?;

            let step_run_id = Uuid::new_v4().to_string();
            sqlx::query("INSERT INTO workflow_step_runs (id, run_id, step_id, started_at, status) VALUES (?1, ?2, ?3, ?4, 'running')")
                .bind(&step_run_id)
                .bind(&run_id)
                .bind(&step.id)
                .bind(Utc::now())
                .execute(&self.state.resources.pool)
                .await?;

            // Prepare prompt by injecting context
            let mut prompt = step.prompt_template.clone();
            if let Some(obj) = context.as_object() {
                for (k, v) in obj {
                    let placeholder = format!("{{{{{}}}}}", k);
                    if let Some(s) = v.as_str() {
                        prompt = prompt.replace(&placeholder, s);
                    } else {
                        prompt = prompt.replace(&placeholder, &v.to_string());
                    }
                }
            }

            let payload = TaskPayload {
                message: prompt,
                department: Some(format!("Workflow: {}", row.get::<String, _>("name"))),
                safe_mode: Some(false),
                analysis: Some(false),
                primary_goal: context.get("primary_goal").and_then(|v| v.as_str()).map(|s| s.to_string()),
                ..Default::default()
            };

            let runner = AgentRunner::new(Arc::clone(&self.state));
            match runner.run(step.agent_id.clone(), payload).await {
                Ok(output) => {
                    final_output = output.clone();

                    // Update context with step output if it's named (simple heuristic: name becomes key)
                    let key = step.name.to_lowercase().replace(" ", "_");
                    if let Some(obj) = context.as_object_mut() {
                        obj.insert(key, serde_json::Value::String(output.clone()));
                    }

                    sqlx::query("UPDATE workflow_step_runs SET completed_at = ?1, status = 'completed', output_text = ?2 WHERE id = ?3")
                        .bind(Utc::now())
                        .bind(&output)
                        .bind(&step_run_id)
                        .execute(&self.state.resources.pool)
                        .await?;

                    sqlx::query("UPDATE workflow_runs SET context = ?1 WHERE id = ?2")
                        .bind(serde_json::to_string(&context)?)
                        .bind(&run_id)
                        .execute(&self.state.resources.pool)
                        .await?;
                }
                Err(e) => {
                    tracing::error!("❌ [Workflow] Step failed: {}", e);

                    sqlx::query("UPDATE workflow_step_runs SET completed_at = ?1, status = 'failed', output_text = ?2 WHERE id = ?3")
                        .bind(Utc::now())
                        .bind(format!("Error: {}", e))
                        .bind(&step_run_id)
                        .execute(&self.state.resources.pool)
                        .await?;

                    sqlx::query("UPDATE workflow_runs SET completed_at = ?1, status = 'failed' WHERE id = ?2")
                        .bind(Utc::now())
                        .bind(&run_id)
                        .execute(&self.state.resources.pool)
                        .await?;

                    return Err(anyhow!("Workflow step '{}' failed: {}", step.name, e));
                }
            }
        }

        sqlx::query(
            "UPDATE workflow_runs SET completed_at = ?1, status = 'completed' WHERE id = ?2",
        )
        .bind(Utc::now())
        .bind(&run_id)
        .execute(&self.state.resources.pool)
        .await?;

        Ok(final_output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::AppState;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_workflow_logic() -> Result<()> {
        let state = Arc::new(AppState::default());
        let _engine = WorkflowEngine::new(Arc::clone(&state));

        // This is a unit test of the structure, as runner.run() is mocked/requires complex setup
        // But we can verify the context injection logic or table setup if needed.
        // For now, verify it can be instantiated.
        // ok

        Ok(())
    }
}

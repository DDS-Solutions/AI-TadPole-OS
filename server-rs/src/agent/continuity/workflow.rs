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
use crate::error::AppError;
use crate::state::AppState;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::Row;
use std::sync::Arc;
use uuid::Uuid;
use futures::StreamExt;

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

fn default_max_retries() -> i32 { 3 }
fn default_backoff_factor_secs() -> i64 { 2 }
fn default_depends_on() -> Vec<String> { vec![] }

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
    #[serde(default = "default_max_retries")]
    pub max_retries: i32,
    #[serde(default = "default_backoff_factor_secs")]
    pub backoff_factor_secs: i64,
    #[serde(default = "default_depends_on")]
    pub depends_on: Vec<String>,
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
    pub async fn list_workflows(&self) -> Result<Vec<Workflow>, AppError> {
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
    ) -> Result<Workflow, AppError> {
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
    #[allow(clippy::too_many_arguments)]
    pub async fn add_step(
        &self,
        workflow_id: &str,
        agent_id: &str,
        name: String,
        prompt_template: String,
        step_order: i32,
        max_retries: Option<i32>,
        backoff_factor_secs: Option<i64>,
        depends_on: Option<Vec<String>>,
    ) -> Result<WorkflowStep, AppError> {
        let id = Uuid::new_v4().to_string();
        let max_r = max_retries.unwrap_or(3);
        let backoff = backoff_factor_secs.unwrap_or(2);
        let deps_vec = depends_on.unwrap_or_default();
        let deps_str = serde_json::to_string(&deps_vec).unwrap_or_else(|_| "[]".to_string());

        sqlx::query("INSERT INTO workflow_steps (id, workflow_id, agent_id, step_order, name, prompt_template, max_retries, backoff_factor_secs, depends_on) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)")
            .bind(&id)
            .bind(workflow_id)
            .bind(agent_id)
            .bind(step_order)
            .bind(&name)
            .bind(&prompt_template)
            .bind(max_r)
            .bind(backoff)
            .bind(&deps_str)
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
            max_retries: max_r,
            backoff_factor_secs: backoff,
            depends_on: deps_vec,
        })
    }

    /// Deletes a workflow
    pub async fn delete_workflow(&self, id: &str) -> Result<(), AppError> {
        sqlx::query("DELETE FROM workflows WHERE id = ?1")
            .bind(id)
            .execute(&self.state.resources.pool)
            .await?;
        Ok(())
    }

    /// Fetches steps for a workflow ordered by step_order
    async fn get_workflow_steps(&self, workflow_id: &str) -> Result<Vec<WorkflowStep>, AppError> {
        let rows = sqlx::query(
            "SELECT * FROM workflow_steps WHERE workflow_id = ?1 ORDER BY step_order ASC, id ASC",
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
                max_retries: row.get::<Option<i32>, _>("max_retries").unwrap_or(3),
                backoff_factor_secs: row.get::<Option<i64>, _>("backoff_factor_secs").unwrap_or(2),
                depends_on: row.get::<Option<String>, _>("depends_on")
                    .and_then(|s| serde_json::from_str(&s).ok())
                    .unwrap_or_default(),
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
    ) -> Result<String, AppError> {
        let row: sqlx::sqlite::SqliteRow =
            sqlx::query::<sqlx::Sqlite>("SELECT * FROM workflows WHERE id = ?1")
                .bind(workflow_id)
                .fetch_one(&self.state.resources.pool)
                .await?;

        if row.get::<i64, _>("enabled") == 0 {
            return Err(AppError::BadRequest("Workflow is disabled".to_string()));
        }

        let steps = self.get_workflow_steps(workflow_id).await?;
        if steps.is_empty() {
            return Err(AppError::BadRequest("Workflow has no steps".to_string()));
        }

        let run_id = Uuid::new_v4().to_string();
        let now = Utc::now();

        sqlx::query("INSERT INTO workflow_runs (id, workflow_id, started_at, status, current_step, context) VALUES (?1, ?2, ?3, 'running', 0, ?4)")
            .bind(&run_id)
            .bind(workflow_id)
            .bind(now)
            .bind(serde_json::to_string(&initial_context).map_err(|e| AppError::InternalServerError(e.to_string()))?)
            .execute(&self.state.resources.pool)
            .await?;

        let context = Arc::new(tokio::sync::Mutex::new(initial_context));
        let workflow_name = row.get::<String, _>("name");

        // Track step status
        use std::collections::HashMap;
        let mut step_status: HashMap<String, StepStatus> = steps
            .iter()
            .map(|s| (s.id.clone(), StepStatus::Pending))
            .collect();

        let mut active_futures = futures::stream::FuturesUnordered::new();
        let mut completed_count = 0;
        let total_steps = steps.len();

        loop {
            // 1. Check if any active futures finished, and update statuses
            while let Ok(Some((step_id, res))) = tokio::time::timeout(std::time::Duration::from_millis(10), active_futures.next()).await {
                let res: Result<String, AppError> = res;
                match res {
                    Ok(out) => {
                        step_status.insert(step_id, StepStatus::Completed(out));
                        completed_count += 1;
                    }
                    Err(e) => {
                        step_status.insert(step_id, StepStatus::Failed(e.to_string()));
                        // Cancel other tasks and mark workflow failed
                        sqlx::query("UPDATE workflow_runs SET completed_at = ?1, status = 'failed' WHERE id = ?2")
                            .bind(Utc::now())
                            .bind(&run_id)
                            .execute(&self.state.resources.pool)
                            .await?;
                        return Err(e);
                    }
                }
            }

            if completed_count == total_steps {
                break;
            }

            // 2. Identify ready steps to execute (dependencies met, and status is Pending)
            let mut ready_steps = Vec::new();
            for step in &steps {
                if step_status.get(&step.id) == Some(&StepStatus::Pending) {
                    // Check dependencies
                    let mut deps_met = true;
                    for dep_id in &step.depends_on {
                        let dep_status = step_status.get(dep_id)
                            .or_else(|| {
                                // Fallback: look up by name in case depends_on specifies name
                                steps.iter().find(|s| &s.name == dep_id).map(|s| &s.id).and_then(|id| step_status.get(id))
                            });

                        match dep_status {
                            Some(StepStatus::Completed(_)) => {}
                            Some(StepStatus::Failed(_err)) => {
                                // If dependency failed, this step fails too
                                return Err(AppError::InternalServerError(format!(
                                    "Dependency '{}' failed, cannot execute step '{}'",
                                    dep_id, step.name
                                )));
                            }
                            _ => {
                                deps_met = false;
                            }
                        }
                    }
                    if deps_met {
                        ready_steps.push(step.clone());
                    }
                }
            }

            // 3. Spawn ready steps up to the concurrency limit (5)
            for step in ready_steps {
                if active_futures.len() >= 5 {
                    break; // Capped at 5 concurrent threads
                }

                step_status.insert(step.id.clone(), StepStatus::Running);

                let state_clone = Arc::clone(&self.state);
                let run_id_clone = run_id.clone();
                let workflow_name_clone = workflow_name.clone();
                let context_clone = Arc::clone(&context);
                let step_id = step.id.clone();

                active_futures.push(async move {
                    let res = execute_single_step(
                        state_clone,
                        run_id_clone,
                        step,
                        workflow_name_clone,
                        context_clone,
                    ).await;
                    (step_id, res)
                });
            }

            // If we have no active tasks and cannot make progress, we might have a deadlock/dependency cycle
            if active_futures.is_empty() && completed_count < total_steps {
                return Err(AppError::InternalServerError(
                    "Workflow execution stalled: Possible dependency cycle detected.".to_string()
                ));
            }

            tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        }

        // Finalize success
        let final_ctx_str = {
            let ctx_lock = context.lock().await;
            serde_json::to_string(&*ctx_lock).unwrap_or_else(|_| "{}".to_string())
        };

        sqlx::query(
            "UPDATE workflow_runs SET completed_at = ?1, status = 'completed', context = ?2 WHERE id = ?3",
        )
        .bind(Utc::now())
        .bind(final_ctx_str)
        .bind(&run_id)
        .execute(&self.state.resources.pool)
        .await?;

        // Find the output of the last step (either by step_order or the one with no dependents)
        // For compatibility, return the last completed step's output
        let mut final_output = String::new();
        if let Some(last_step) = steps.iter().max_by_key(|s| s.step_order) {
            if let Some(StepStatus::Completed(out)) = step_status.get(&last_step.id) {
                final_output = out.clone();
            }
        }

        Ok(final_output)
    }
}

#[derive(Clone, Debug, PartialEq)]
enum StepStatus {
    Pending,
    Running,
    Completed(String),
    Failed(String),
}

async fn execute_single_step(
    state: Arc<AppState>,
    run_id: String,
    step: WorkflowStep,
    workflow_name: String,
    context: Arc<tokio::sync::Mutex<serde_json::Value>>,
) -> Result<String, AppError> {
    let step_run_id = Uuid::new_v4().to_string();
    sqlx::query("INSERT INTO workflow_step_runs (id, run_id, step_id, started_at, status) VALUES (?1, ?2, ?3, ?4, 'running')")
        .bind(&step_run_id)
        .bind(&run_id)
        .bind(&step.id)
        .bind(Utc::now())
        .execute(&state.resources.pool)
        .await?;

    let mut attempt = 0;
    let max_attempts = step.max_retries.max(0) + 1; // 3 retries = 4 attempts
    let mut last_error = String::new();
    let mut output = String::new();
    let runner = AgentRunner::new(Arc::clone(&state));

    while attempt < max_attempts {
        if attempt > 0 {
            let delay_secs = step.backoff_factor_secs.max(1) * (2_i64.pow(attempt as u32 - 1));
            tracing::info!(
                "🔄 [Workflow] Step '{}' failed. Retrying in {} seconds (attempt {}/{})",
                step.name, delay_secs, attempt, max_attempts - 1
            );
            tokio::time::sleep(std::time::Duration::from_secs(delay_secs as u64)).await;
        }

        let prompt = {
            let ctx_lock = context.lock().await;
            let mut p = step.prompt_template.clone();
            if let Some(obj) = ctx_lock.as_object() {
                for (k, v) in obj {
                    let placeholder = format!("{{{{{}}}}}", k);
                    if let Some(s) = v.as_str() {
                        p = p.replace(&placeholder, s);
                    } else {
                        p = p.replace(&placeholder, &v.to_string());
                    }
                }
            }
            p
        };

        let payload = TaskPayload {
            message: prompt,
            department: Some(format!("Workflow: {}", workflow_name)),
            safe_mode: Some(false),
            analysis: Some(false),
            primary_goal: {
                let ctx_lock = context.lock().await;
                ctx_lock.get("primary_goal").and_then(|v| v.as_str()).map(|s| s.to_string())
            },
            ..Default::default()
        };

        match runner.run(step.agent_id.clone(), payload).await {
            Ok(out) => {
                output = out;
                break;
            }
            Err(e) => {
                last_error = e.to_string();
                attempt += 1;
            }
        }
    }

    if attempt >= max_attempts {
        let err_msg = format!(
            "Step '{}' failed after {} attempts. Last error: {}",
            step.name, max_attempts, last_error
        );
        tracing::error!("❌ [Workflow] {}", err_msg);

        sqlx::query("UPDATE workflow_step_runs SET completed_at = ?1, status = 'failed', output_text = ?2 WHERE id = ?3")
            .bind(Utc::now())
            .bind(&err_msg)
            .bind(&step_run_id)
            .execute(&state.resources.pool)
            .await?;

        return Err(AppError::InternalServerError(err_msg));
    }

    let pruned = prune_step_output(&output, &step.config);
    let key = step.name.to_lowercase().replace(" ", "_");
    {
        let mut ctx_lock = context.lock().await;
        if let Some(obj) = ctx_lock.as_object_mut() {
            obj.insert(key, serde_json::Value::String(pruned));
        }
    }

    sqlx::query("UPDATE workflow_step_runs SET completed_at = ?1, status = 'completed', output_text = ?2 WHERE id = ?3")
        .bind(Utc::now())
        .bind(&output)
        .bind(&step_run_id)
        .execute(&state.resources.pool)
        .await?;

    Ok(output)
}

/// Helper function to extract or prune step output stored in workflow context based on configuration.
fn prune_step_output(output: &str, config: &Option<serde_json::Value>) -> String {
    let Some(cfg) = config else {
        return output.to_string();
    };

    let mut result = output.to_string();

    // 1. JSON keys extraction (if output is JSON and context_keys array is specified)
    if let Some(keys) = cfg.get("context_keys").and_then(|v| v.as_array()) {
        if let Ok(json_val) = serde_json::from_str::<serde_json::Value>(&result) {
            let mut extracted = serde_json::Map::new();
            for key_val in keys {
                if let Some(key_str) = key_val.as_str() {
                    if let Some(v) = json_val.get(key_str) {
                        extracted.insert(key_str.to_string(), v.clone());
                    }
                }
            }
            if !extracted.is_empty() {
                result = serde_json::Value::Object(extracted).to_string();
            }
        }
    }
    // 2. JSON path extraction (dot-notation, e.g. "data.summary")
    else if let Some(path) = cfg.get("context_json_path").and_then(|v| v.as_str()) {
        if let Ok(json_val) = serde_json::from_str::<serde_json::Value>(&result) {
            let mut current = &json_val;
            let mut found = true;
            for part in path.split('.') {
                if let Some(next) = current.get(part) {
                    current = next;
                } else {
                    found = false;
                    break;
                }
            }
            if found {
                if let Some(s) = current.as_str() {
                    result = s.to_string();
                } else {
                    result = current.to_string();
                }
            }
        }
    }

    // 3. String truncation limit
    if let Some(limit) = cfg.get("context_max_chars").and_then(|v| v.as_i64()) {
        if limit > 0 && result.chars().count() > limit as usize {
            let truncated: String = result.chars().take(limit as usize).collect();
            result = format!(
                "{}... [Truncated: exceeded context limit of {} chars]",
                truncated, limit
            );
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::init_db;

    #[tokio::test]
    async fn test_workflow_crud() -> Result<(), Box<dyn std::error::Error>> {
        let pool = init_db("sqlite::memory:").await?;
        let state = Arc::new(AppState::with_pool(pool).await);
        let engine = WorkflowEngine::new(Arc::clone(&state));

        // 1. Create
        let wf = engine
            .create_workflow("Test Workflow".to_string(), Some("Desc".to_string()))
            .await?;
        assert_eq!(wf.name, "Test Workflow");

        // 2. Add Steps
        engine
            .add_step(
                &wf.id,
                "agent-1",
                "Step 1".to_string(),
                "Do A".to_string(),
                1,
                None,
                None,
                None,
            )
            .await?;
        engine
            .add_step(
                &wf.id,
                "agent-2",
                "Step 2".to_string(),
                "Do B".to_string(),
                2,
                None,
                None,
                None,
            )
            .await?;

        // 3. List & Verify Steps
        let steps = engine.get_workflow_steps(&wf.id).await?;
        assert_eq!(steps.len(), 2);
        assert_eq!(steps[0].step_order, 1);
        assert_eq!(steps[1].step_order, 2);

        // 4. Delete
        engine.delete_workflow(&wf.id).await?;
        let list = engine.list_workflows().await?;
        assert!(list.is_empty());

        Ok(())
    }

    #[test]
    fn test_prune_step_output_no_config() {
        let output = "original output";
        assert_eq!(prune_step_output(output, &None), "original output");
    }

    #[test]
    fn test_prune_step_output_max_chars() {
        let output = "1234567890";
        let config = Some(serde_json::json!({ "context_max_chars": 5 }));
        let pruned = prune_step_output(output, &config);
        assert!(pruned.starts_with("12345"));
        assert!(pruned.contains("Truncated"));

        let config_under = Some(serde_json::json!({ "context_max_chars": 15 }));
        assert_eq!(prune_step_output(output, &config_under), "1234567890");
    }

    #[test]
    fn test_prune_step_output_json_keys() {
        let output = serde_json::json!({
            "key1": "value1",
            "key2": "value2",
            "key3": "value3"
        })
        .to_string();

        let config = Some(serde_json::json!({
            "context_keys": ["key1", "key3"]
        }));

        let pruned = prune_step_output(&output, &config);
        let parsed: serde_json::Value = serde_json::from_str(&pruned).unwrap();
        assert_eq!(parsed["key1"], "value1");
        assert_eq!(parsed["key3"], "value3");
        assert!(parsed.get("key2").is_none());
    }

    #[test]
    fn test_prune_step_output_json_path() {
        let output = serde_json::json!({
            "nested": {
                "target": "target_value",
                "other": 123
            }
        })
        .to_string();

        let config = Some(serde_json::json!({
            "context_json_path": "nested.target"
        }));

        let pruned = prune_step_output(&output, &config);
        assert_eq!(pruned, "target_value");

        let config_nonexistent = Some(serde_json::json!({
            "context_json_path": "nested.invalid"
        }));
        assert_eq!(prune_step_output(&output, &config_nonexistent), output);
    }
}

// Metadata: [workflow]

// Metadata: [workflow]

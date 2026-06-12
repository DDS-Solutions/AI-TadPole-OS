//! @docs ARCHITECTURE:Registry
//! 
//! ### AI Assist Note
//! **Core technical resource for the Tadpole OS Sovereign infrastructure.**
//! This module implements high-fidelity logic for the Sovereign Reality layer.
//! 
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: Runtime logic error, state desynchronization, or resource exhaustion.
//! - **Telemetry Link**: Search `[engine]` in tracing logs.

use crate::error::AppError;
use crate::state::AppState;
use super::types::{
    Workflow, WorkflowStep, StepConfig, StepStatus, ActiveRunGuard, WorkflowRunGuard
};
use super::helpers::{
    detect_dependency_cycle, sanitize_context_key, evaluate_routing_rules, get_context_as_string,
    get_concurrency_limit
};
use super::executor::execute_single_step;
use std::sync::Arc;
use uuid::Uuid;
use dashmap::DashMap;
use std::collections::{HashMap, HashSet};
use futures::StreamExt;

pub struct WorkflowEngine {
    pub(crate) state: Arc<AppState>,
    pub(crate) repo: crate::agent::continuity::repository::WorkflowRepository,
    pub(crate) active_runs: Arc<DashMap<String, tokio_util::sync::CancellationToken>>,
}

impl WorkflowEngine {
    pub fn new(state: Arc<AppState>) -> Self {
        let repo = crate::agent::continuity::repository::WorkflowRepository::new(
            state.resources.pool.clone(),
        );
        Self {
            state,
            repo,
            active_runs: Arc::new(DashMap::new()),
        }
    }

    pub async fn list_workflows(&self, tenant_id: &str) -> Result<Vec<Workflow>, AppError> {
        self.repo.list_workflows(tenant_id).await
    }

    pub async fn create_workflow(
        &self,
        tenant_id: &str,
        name: String,
        description: Option<String>,
    ) -> Result<Workflow, AppError> {
        self.repo
            .create_workflow(tenant_id, name, description)
            .await
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn add_step(
        &self,
        tenant_id: &str,
        workflow_id: &str,
        agent_id: &str,
        name: String,
        prompt_template: String,
        step_order: i32,
        max_retries: Option<i32>,
        backoff_factor_secs: Option<i64>,
        depends_on: Option<Vec<String>>,
    ) -> Result<WorkflowStep, AppError> {
        if name.len() > 100 {
            return Err(AppError::BadRequest(
                "Step name cannot exceed 100 characters".to_string(),
            ));
        }
        if prompt_template.len() > 50000 {
            return Err(AppError::BadRequest(
                "Prompt template cannot exceed 50000 characters".to_string(),
            ));
        }
        self.repo
            .add_step(
                tenant_id,
                workflow_id,
                agent_id,
                name,
                prompt_template,
                step_order,
                max_retries,
                backoff_factor_secs,
                depends_on,
            )
            .await
    }

    pub async fn delete_workflow(&self, tenant_id: &str, id: &str) -> Result<(), AppError> {
        self.repo.delete_workflow(id, tenant_id).await
    }

    pub async fn get_workflow_steps(
        &self,
        tenant_id: &str,
        workflow_id: &str,
    ) -> Result<Vec<WorkflowStep>, AppError> {
        self.repo.get_workflow_steps(workflow_id, tenant_id).await
    }

    pub async fn list_workflow_runs(
        &self,
        tenant_id: &str,
        workflow_id: &str,
        limit: i64,
    ) -> Result<Vec<serde_json::Value>, AppError> {
        self.repo.list_workflow_runs(workflow_id, tenant_id, limit).await
    }

    pub async fn cancel_run(&self, tenant_id: &str, run_id: &str) -> Result<(), AppError> {
        let row = self.repo.get_workflow_run_tenant_and_status(run_id).await?;

        let (run_tenant, status) =
            row.ok_or_else(|| AppError::NotFound(format!("Workflow run '{}' not found", run_id)))?;

        if run_tenant != tenant_id {
            return Err(AppError::NotFound(format!(
                "Workflow run '{}' not found",
                run_id
            )));
        }
        if status != "running" {
            return Err(AppError::BadRequest(format!(
                "Workflow run '{}' is not running (status: {})",
                run_id, status
            )));
        }

        if let Some(token) = self.active_runs.get(run_id) {
            token.cancel();
            tracing::info!("🛑 [Workflow] Cancellation requested for run '{}'", run_id);
        } else {
            tracing::warn!("⚠️ [Workflow] Cancellation token not found for run '{}' — run may have already completed", run_id);
        }

        self.repo.set_run_status_cancelling(run_id).await?;

        if let Err(e) = self.state.security.audit_trail.record(
            "system/workflow_run_cancel",
            Some(run_id),
            None,
            "workflow_run_cancelled",
            &serde_json::json!({
                "run_id": run_id,
                "tenant_id": tenant_id,
                "status": "cancelling",
                "reason": "User requested run cancellation"
            }).to_string()
        ).await {
            tracing::error!("Failed to write audit trail for workflow run cancellation: {}", e);
        }

        self.repo
            .cleanup_step_runs_on_cancel(run_id, tenant_id)
            .await?;

        Ok(())
    }

    pub async fn run_workflow(
        &self,
        tenant_id: &str,
        workflow_id: &str,
        initial_context: serde_json::Value,
    ) -> Result<(String, String), AppError> {
        let repo = crate::agent::continuity::repository::WorkflowRepository::new(
            self.state.resources.pool.clone(),
        );
        let workflow = repo.get_workflow(workflow_id, tenant_id).await?;

        if !workflow.enabled {
            return Err(AppError::BadRequest("Workflow is disabled".to_string()));
        }

        let steps = repo.get_workflow_steps(workflow_id, tenant_id).await?;
        if steps.is_empty() {
            return Err(AppError::BadRequest("Workflow has no steps".to_string()));
        }

        detect_dependency_cycle(&steps)?;

        let mut sanitized_keys = HashSet::new();
        for step in &steps {
            let key = sanitize_context_key(&step.name);
            if !sanitized_keys.insert(key.clone()) {
                return Err(AppError::BadRequest(format!(
                    "Duplicate step name / context key collision detected for key '{}'. Step names must map to unique sanitized keys.",
                    key
                )));
            }
        }

        let run_id = Uuid::new_v4().to_string();
        let initial_ctx_str = serde_json::to_string(&initial_context)
            .map_err(|e| AppError::InternalServerError(e.to_string()))?;
        repo.create_workflow_run(&run_id, workflow_id, tenant_id, initial_ctx_str)
            .await?;

        let context = Arc::new(parking_lot::Mutex::new(initial_context));
        let workflow_name = workflow.name;
        let total_steps = steps.len();
        let initial_version = workflow.version;

        let token = tokio_util::sync::CancellationToken::new();
        self.active_runs.insert(run_id.clone(), token.clone());
        let _active_run_guard = ActiveRunGuard {
            active_runs: Arc::clone(&self.active_runs),
            run_id: run_id.clone(),
        };

        tracing::info!(
            run_id = %run_id,
            workflow_id = %workflow_id,
            workflow_name = %workflow_name,
            tenant_id = %tenant_id,
            total_steps = %total_steps,
            "🚀 [Workflow] Starting workflow run"
        );

        self.state.emit_event(serde_json::json!({
            "type": "engine:workflow_run_started",
            "data": {
                "run_id": run_id,
                "workflow_id": workflow_id,
                "workflow_name": workflow_name,
                "total_steps": total_steps,
            }
        }));

        let mut step_status: HashMap<String, StepStatus> = steps
            .iter()
            .map(|s| (s.id.clone(), StepStatus::Pending))
            .collect();

        let mut active_futures = futures::stream::FuturesUnordered::<
            futures::future::BoxFuture<'static, (String, Result<String, AppError>)>,
        >::new();
        let mut completed_count: usize = 0;

        self.spawn_ready_steps(
            &steps,
            &mut step_status,
            &mut active_futures,
            &run_id,
            &workflow_name,
            &context,
            token.clone(),
        )?;

        let run_guard = WorkflowRunGuard::new(
            run_id.clone(),
            tenant_id.to_string(),
            Arc::clone(&self.state),
        );

        let mut routing_reset_count: usize = 0;
        const MAX_ROUTING_RESETS: usize = 10;

        loop {
            tokio::select! {
                _ = token.cancelled() => {
                    return Err(AppError::InternalServerError("Workflow execution cancelled".to_string()));
                }
                opt = active_futures.next() => {
                    let (step_id, res) = match opt {
                        Some(val) => val,
                        None => {
                            if completed_count < total_steps {
                                return Err(AppError::InternalServerError(
                                    "Workflow execution stalled: Possible dependency cycle detected.".to_string()
                                ));
                            }
                            break;
                        }
                    };

                    match res {
                        Ok(out) => {
                            step_status.insert(step_id.clone(), StepStatus::Completed(out));
                            completed_count += 1;

                            if let Some(completed_step) = steps.iter().find(|s| s.id == step_id) {
                                if let Some(cfg_val) = &completed_step.config {
                                    let cfg = serde_json::from_value::<StepConfig>(cfg_val.clone()).map_err(|e| {
                                        AppError::InternalServerError(format!("Invalid StepConfig: {}", e))
                                    })?;
                                    if let Some(routing) = cfg.routing {
                                        if let Some(reset_steps) = evaluate_routing_rules(&context, &routing.rules) {
                                            routing_reset_count += 1;
                                            if routing_reset_count > MAX_ROUTING_RESETS {
                                                return Err(AppError::InternalServerError(
                                                    format!("Workflow exceeded maximum routing resets ({}). Possible infinite loop.", MAX_ROUTING_RESETS)
                                                ));
                                            }
                                            tracing::info!("🔀 [Workflow] Routing rule matched! Resetting target steps: {:?} (reset #{}/{})", reset_steps, routing_reset_count, MAX_ROUTING_RESETS);
                                            let mut reset_targets = Vec::new();
                                            for reset_dep in &reset_steps {
                                                if let Some(target_step) = steps.iter().find(|s| s.id == *reset_dep) {
                                                    reset_targets.push(target_step.id.clone());
                                                }
                                            }

                                            let downstream_to_reset = crate::agent::continuity::workflow::helpers::get_all_downstream(&reset_targets, &steps);
                                            let mut all_to_reset = HashSet::new();
                                            for target in reset_targets {
                                                all_to_reset.insert(target);
                                            }
                                            for ds in downstream_to_reset {
                                                all_to_reset.insert(ds);
                                            }

                                            for reset_id in all_to_reset {
                                                if let Some(old_status) = step_status.get(&reset_id) {
                                                    if matches!(old_status, StepStatus::Completed(_)) {
                                                        completed_count = completed_count.saturating_sub(1);
                                                    }
                                                }
                                                step_status.insert(reset_id, StepStatus::Pending);
                                            }
                                        }
                                    }
                                }
                            }

                            self.spawn_ready_steps(&steps, &mut step_status, &mut active_futures, &run_id, &workflow_name, &context, token.clone())?;
                        }
                        Err(e) => {
                            step_status.insert(step_id, StepStatus::Failed(e.to_string()));
                            return Err(e);
                        }
                    }

                    if completed_count == total_steps {
                        break;
                    }
                }
            }
        }

        let current_workflow = repo.get_workflow(workflow_id, tenant_id).await?;
        if current_workflow.version != initial_version {
            return Err(AppError::InternalServerError(
                "Workflow step configuration changed during execution. Aborting.".to_string(),
            ));
        }

        run_guard.finalize();

        let final_ctx_str = get_context_as_string(&context)?;
        repo.update_workflow_run_success(&run_id, tenant_id, final_ctx_str)
            .await?;

        let mut final_output = String::new();
        if let Some(last_step) = steps.iter().max_by_key(|s| s.step_order) {
            if let Some(StepStatus::Completed(out)) = step_status.get(&last_step.id) {
                final_output = out.clone();
            }
        }

        Ok((final_output, run_id))
    }

    fn spawn_ready_steps(
        &self,
        steps: &[WorkflowStep],
        step_status: &mut HashMap<String, StepStatus>,
        active_futures: &mut futures::stream::FuturesUnordered<
            futures::future::BoxFuture<'static, (String, Result<String, AppError>)>,
        >,
        run_id: &str,
        workflow_name: &str,
        context: &Arc<parking_lot::Mutex<serde_json::Value>>,
        cancel_token: tokio_util::sync::CancellationToken,
    ) -> Result<(), AppError> {
        let mut ready_steps = Vec::new();
        for step in steps {
            if step_status.get(&step.id) == Some(&StepStatus::Pending) {
                let mut deps_met = true;
                for dep_id in &step.depends_on {
                    let dep_status = step_status.get(dep_id);

                    match dep_status {
                        Some(StepStatus::Completed(_)) => {}
                        Some(StepStatus::Failed(_err)) => {
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

        let limit = get_concurrency_limit();
        for step in ready_steps {
            if active_futures.len() >= limit {
                break;
            }

            step_status.insert(step.id.clone(), StepStatus::Running);

            let state_clone = Arc::clone(&self.state);
            let run_id_clone = run_id.to_string();
            let workflow_name_clone = workflow_name.to_string();
            let context_clone = Arc::clone(context);
            let step_id = step.id.clone();

            let cancel_token_clone = cancel_token.clone();
            active_futures.push(Box::pin(async move {
                let token_for_select = cancel_token_clone.clone();
                tokio::select! {
                    _ = token_for_select.cancelled() => {
                        (step_id, Err(AppError::InternalServerError("Step execution cancelled".to_string())))
                    }
                    res = execute_single_step(
                        state_clone,
                        run_id_clone,
                        step,
                        workflow_name_clone,
                        context_clone,
                        cancel_token_clone,
                    ) => {
                        (step_id, res)
                    }
                }
            }));
        }
        Ok(())
    }
}

// Metadata: [engine]

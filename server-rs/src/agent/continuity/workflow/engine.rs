//! @docs ARCHITECTURE:Continuity:Workflow
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Agent Runner / engine
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Dependency cycle validation is performed before spawning any step futures.
//! - `[Behavioral]` Cyclic dependencies immediately fail workflow initialization.
//!   - enforced_by: `test_dag_cycle_detection`
//! - `[Behavioral]` Active run guards prevent duplicate concurrent executions of the same workflow run.
//!   - enforced_by: `test_workflow_run_guard`
//! - `[Behavioral]` Stale workflow versions reject execution via optimistic locking.
//!   - enforced_by: `test_workflow_version_optimistic_locking`
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: `[Workflow]`
//! - **Witness Tests**: `test_dag_cycle_detection`, `test_workflow_run_guard`, `test_workflow_version_optimistic_locking`

use super::executor::execute_single_step;
use super::helpers::{
    detect_dependency_cycle, evaluate_routing_rules, get_concurrency_limit, get_context_as_string,
    get_terminal_steps, resolve_step_target, sanitize_context_key,
};
use super::types::{
    ActiveRunGuard, StepConfig, StepStatus, Workflow, WorkflowRunGuard, WorkflowStep,
};
use crate::error::AppError;
use crate::state::AppState;
use dashmap::DashMap;
use futures::StreamExt;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use uuid::Uuid;

pub struct WorkflowEngine {
    pub(crate) state: Arc<AppState>,
    pub(crate) repo: crate::agent::continuity::repository::WorkflowRepository,
    pub(crate) active_runs: Arc<DashMap<String, tokio_util::sync::CancellationToken>>,
    pub(crate) concurrency_semaphore: Arc<tokio::sync::Semaphore>,
}

impl WorkflowEngine {
    pub fn new(state: Arc<AppState>) -> Self {
        let repo = crate::agent::continuity::repository::WorkflowRepository::new(
            state.resources.pool.clone(),
        );
        Self {
            active_runs: Arc::clone(&state.resources.workflow_active_runs),
            concurrency_semaphore: Arc::clone(&state.resources.workflow_concurrency_semaphore),
            state,
            repo,
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
        if name.chars().count() > 100 {
            return Err(AppError::BadRequest(
                "Step name cannot exceed 100 characters".to_string(),
            ));
        }
        if prompt_template.chars().count() > 50000 {
            return Err(AppError::BadRequest(
                "Prompt template cannot exceed 50000 characters".to_string(),
            ));
        }

        let existing_steps = self.repo.get_workflow_steps(workflow_id, tenant_id).await?;
        let sanitized_new_key = sanitize_context_key(&name);

        for existing in &existing_steps {
            if existing.name == name || sanitize_context_key(&existing.name) == sanitized_new_key {
                return Err(AppError::BadRequest(format!(
                    "Duplicate step name / context key collision detected for key '{}'. Step names must map to unique sanitized keys.",
                    sanitized_new_key
                )));
            }
        }

        if let Some(deps) = &depends_on {
            for dep in deps {
                if dep == &name || sanitize_context_key(dep) == sanitized_new_key {
                    return Err(AppError::BadRequest(format!(
                        "Step '{}' cannot depend on itself",
                        name
                    )));
                }
                let exists = existing_steps.iter().any(|s| {
                    s.id == *dep
                        || s.name == *dep
                        || sanitize_context_key(&s.name) == sanitize_context_key(dep)
                });
                if !exists {
                    return Err(AppError::BadRequest(format!(
                        "Step '{}' specifies dependency '{}' which does not exist in workflow '{}'",
                        name, dep, workflow_id
                    )));
                }
            }
        }

        // Pre-validate cycle detection with candidate step before DB write
        let candidate_step = WorkflowStep {
            id: "__new_step__".to_string(),
            workflow_id: workflow_id.to_string(),
            agent_id: agent_id.to_string(),
            step_order,
            name: name.clone(),
            prompt_template: prompt_template.clone(),
            config: None,
            max_retries: max_retries.unwrap_or(3),
            backoff_factor_secs: backoff_factor_secs.unwrap_or(2),
            depends_on: depends_on.clone().unwrap_or_default(),
        };
        let mut candidate_steps = existing_steps;
        candidate_steps.push(candidate_step);
        detect_dependency_cycle(&candidate_steps)?;

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

    #[allow(dead_code)]
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
        self.repo
            .list_workflow_runs(workflow_id, tenant_id, limit)
            .await
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

        let updated = self
            .repo
            .set_run_status_cancelling(run_id, tenant_id)
            .await?;
        if !updated {
            tracing::warn!("⚠️ [Workflow] Run '{}' was no longer in 'running' status during cancellation update", run_id);
        }

        if let Err(e) = self
            .state
            .security
            .audit_trail
            .record(
                "system/workflow_run_cancel",
                Some(run_id),
                None,
                "workflow_run_cancelled",
                &serde_json::json!({
                    "run_id": run_id,
                    "tenant_id": tenant_id,
                    "status": "cancelling",
                    "reason": "User requested run cancellation"
                })
                .to_string(),
            )
            .await
        {
            tracing::error!(
                "Failed to write audit trail for workflow run cancellation: {}",
                e
            );
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
        let run_id = Uuid::new_v4().to_string();
        self.run_workflow_with_id(tenant_id, workflow_id, &run_id, initial_context)
            .await
    }

    pub async fn run_workflow_with_id(
        &self,
        tenant_id: &str,
        workflow_id: &str,
        run_id: &str,
        initial_context: serde_json::Value,
    ) -> Result<(String, String), AppError> {
        let workflow = self.repo.get_workflow(workflow_id, tenant_id).await?;

        if !workflow.enabled {
            return Err(AppError::BadRequest("Workflow is disabled".to_string()));
        }

        let steps = self.repo.get_workflow_steps(workflow_id, tenant_id).await?;
        if steps.is_empty() {
            return Err(AppError::BadRequest("Workflow has no steps".to_string()));
        }

        detect_dependency_cycle(&steps)?;

        // Pre-parse and validate all StepConfig objects upfront
        let mut step_configs: HashMap<String, Option<StepConfig>> = HashMap::new();
        let mut sanitized_keys = HashSet::new();
        for step in &steps {
            let key = sanitize_context_key(&step.name);
            if !sanitized_keys.insert(key.clone()) {
                return Err(AppError::BadRequest(format!(
                    "Duplicate step name / context key collision detected for key '{}'. Step names must map to unique sanitized keys.",
                    key
                )));
            }

            if let Some(cfg_val) = &step.config {
                let cfg = serde_json::from_value::<StepConfig>(cfg_val.clone()).map_err(|e| {
                    AppError::BadRequest(format!(
                        "Invalid StepConfig for step '{}': {}",
                        step.name, e
                    ))
                })?;
                cfg.validate()?;
                step_configs.insert(step.id.clone(), Some(cfg));
            } else {
                step_configs.insert(step.id.clone(), None);
            }
        }

        let run_id = run_id.to_string();
        let initial_ctx_str = serde_json::to_string(&initial_context)
            .map_err(|e| AppError::InternalServerError(e.to_string()))?;
        self.repo
            .create_workflow_run(&run_id, workflow_id, tenant_id, initial_ctx_str)
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
                "tenant_id": tenant_id,
                "total_steps": total_steps,
            }
        }));

        let mut step_status: HashMap<String, StepStatus> = steps
            .iter()
            .map(|s| (s.id.clone(), StepStatus::Pending))
            .collect();

        let mut epoch_tracker: HashMap<String, u64> =
            steps.iter().map(|s| (s.id.clone(), 0u64)).collect();

        let mut active_futures = futures::stream::FuturesUnordered::<
            futures::future::BoxFuture<'static, (String, u64, Result<String, AppError>)>,
        >::new();
        let mut completed_count: usize = 0;

        self.spawn_ready_steps(
            tenant_id,
            &steps,
            &mut step_status,
            &epoch_tracker,
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
        const DEFAULT_MAX_ROUTING_RESETS: usize = 10;

        loop {
            tokio::select! {
                _ = token.cancelled() => {
                    return Err(AppError::Conflict("Workflow execution cancelled".to_string()));
                }
                opt = active_futures.next() => {
                    let (step_id, spawned_epoch, res) = match opt {
                        Some(val) => val,
                        None => {
                            if completed_count < total_steps {
                                return Err(AppError::BadRequest(
                                    "Workflow execution stalled: Possible dependency cycle detected.".to_string()
                                ));
                            }
                            break;
                        }
                    };

                    // Check for stale execution from a superseded epoch
                    let current_epoch = epoch_tracker.get(&step_id).copied().unwrap_or(0);
                    if spawned_epoch != current_epoch {
                        tracing::warn!(
                            "⚠️ [Workflow] Discarding stale completion for step '{}' (spawned epoch {}, current epoch {})",
                            step_id, spawned_epoch, current_epoch
                        );
                        continue;
                    }

                    match res {
                        Ok(out) => {
                            step_status.insert(step_id.clone(), StepStatus::Completed(out));
                            completed_count += 1;

                            if let Some(Some(cfg)) = step_configs.get(&step_id) {
                                if let Some(routing) = &cfg.routing {
                                    if let Some(reset_steps) = evaluate_routing_rules(&context, &routing.rules) {
                                        let max_resets = routing.max_resets.unwrap_or(DEFAULT_MAX_ROUTING_RESETS);
                                        routing_reset_count += 1;
                                        if routing_reset_count > max_resets {
                                            return Err(AppError::Conflict(
                                                format!("Workflow exceeded maximum routing resets ({}). Possible infinite loop.", max_resets)
                                            ));
                                        }
                                        tracing::info!("🔀 [Workflow] Routing rule matched! Resetting target steps: {:?} (reset #{}/{})", reset_steps, routing_reset_count, max_resets);

                                        let mut reset_targets = Vec::new();
                                        for reset_dep in &reset_steps {
                                            if let Some(target_id) = resolve_step_target(reset_dep, &steps) {
                                                reset_targets.push(target_id);
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
                                            // Advance epoch to immediately invalidate any in-flight running task for this step
                                            if let Some(epoch) = epoch_tracker.get_mut(&reset_id) {
                                                *epoch += 1;
                                            }
                                            step_status.insert(reset_id, StepStatus::Pending);
                                        }
                                    }
                                }
                            }

                            self.spawn_ready_steps(tenant_id, &steps, &mut step_status, &epoch_tracker, &mut active_futures, &run_id, &workflow_name, &context, token.clone())?;
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

        let current_workflow = self.repo.get_workflow(workflow_id, tenant_id).await?;
        if current_workflow.version != initial_version {
            return Err(AppError::Conflict(
                "Workflow step configuration changed during execution. Aborting.".to_string(),
            ));
        }

        run_guard.finalize();

        let final_ctx_str = get_context_as_string(&context)?;
        let updated = self
            .repo
            .update_workflow_run_success(&run_id, tenant_id, final_ctx_str)
            .await?;
        if !updated {
            tracing::warn!("⚠️ [Workflow] Run '{}' completion could not be written (run was no longer in 'running' status)", run_id);
        }

        let mut final_output = String::new();
        let terminals = get_terminal_steps(&steps);
        if let Some(terminal_step) = terminals.iter().max_by_key(|s| s.step_order) {
            if let Some(StepStatus::Completed(out)) = step_status.get(&terminal_step.id) {
                final_output = out.clone();
            }
        } else if let Some(last_step) = steps.iter().max_by_key(|s| s.step_order) {
            if let Some(StepStatus::Completed(out)) = step_status.get(&last_step.id) {
                final_output = out.clone();
            }
        }

        Ok((final_output, run_id))
    }

    #[allow(clippy::too_many_arguments)]
    fn spawn_ready_steps(
        &self,
        tenant_id: &str,
        steps: &[WorkflowStep],
        step_status: &mut HashMap<String, StepStatus>,
        epoch_tracker: &HashMap<String, u64>,
        active_futures: &mut futures::stream::FuturesUnordered<
            futures::future::BoxFuture<'static, (String, u64, Result<String, AppError>)>,
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
                            return Err(AppError::BadRequest(format!(
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

            let epoch = epoch_tracker.get(&step.id).copied().unwrap_or(0);
            step_status.insert(step.id.clone(), StepStatus::Running { epoch });

            let state_clone = Arc::clone(&self.state);
            let tenant_id_clone = tenant_id.to_string();
            let run_id_clone = run_id.to_string();
            let workflow_name_clone = workflow_name.to_string();
            let context_clone = Arc::clone(context);
            let step_id = step.id.clone();
            let cancel_token_clone = cancel_token.clone();
            let sem_clone = Arc::clone(&self.concurrency_semaphore);

            active_futures.push(Box::pin(async move {
                let token_for_select = cancel_token_clone.clone();
                let _permit = sem_clone.acquire().await.ok();
                tokio::select! {
                    _ = token_for_select.cancelled() => {
                        (step_id, epoch, Err(AppError::Conflict("Step execution cancelled".to_string())))
                    }
                    res = execute_single_step(
                        state_clone,
                        tenant_id_clone,
                        run_id_clone,
                        step,
                        workflow_name_clone,
                        context_clone,
                        cancel_token_clone,
                    ) => {
                        (step_id, epoch, res)
                    }
                }
            }));
        }
        Ok(())
    }
}

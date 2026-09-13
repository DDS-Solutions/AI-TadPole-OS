//! @docs ARCHITECTURE:Continuity:Workflow
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Agent Runner / executor
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` `cancel_token.is_cancelled()` is evaluated before dispatch and within standard retry loops.
//! - `[Behavioral]` `FanOutFailStrategy::HaltOnAnyFailure` halts parallel stream on first encountered error.
//!   - enforced_by: `test_concurrent_fan_out`
//! - `[Behavioral]` Step failures preserve full attempts history and do not overwrite prior error metadata.
//!   - enforced_by: `test_failure_metadata_preservation`
//! - `[Behavioral]` Agent execution timeouts return `AppError::InternalServerError`.
//!   - enforced_by: `test_step_run_failed_on_config_error`
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: `[Workflow]`
//! - **Witness Tests**: `test_concurrent_fan_out`, `test_failure_metadata_preservation`, `test_step_run_failed_on_config_error`, `test_workflow_step_failure_event`, `test_workflow_cancellation`, `test_multi_model_tournament`

use super::helpers::{
    get_agent_timeout_duration, get_concurrency_limit, get_fan_out_items, insert_context_key,
    prune_step_output, resolve_fan_out_prompt, resolve_step_prompts, resolve_tournament_prompt,
    sanitize_context_key, truncate_candidate_output,
};
use super::types::{
    FanOutConfig, FanOutFailStrategy, StepConfig, TournamentConfig, WorkflowStep, MAX_FAN_OUT_ITEMS,
};
use crate::agent::runner::AgentRunner;
use crate::agent::types::TaskPayload;
use crate::error::AppError;
use crate::state::AppState;
use chrono::Utc;
use futures::StreamExt;
use std::sync::Arc;
use uuid::Uuid;

/// Unified guarded runner for agent execution with timeout, cancellation, and elapsed timing.
async fn run_agent_guarded(
    state: &Arc<AppState>,
    agent_id: String,
    payload: TaskPayload,
    cancel_token: &tokio_util::sync::CancellationToken,
) -> Result<(String, i64), (String, i64)> {
    if cancel_token.is_cancelled() {
        return Err(("Step execution cancelled".to_string(), 0));
    }
    let runner = AgentRunner::new(Arc::clone(state));
    let start_time = Utc::now();
    let timeout_dur = get_agent_timeout_duration();
    let run_fut = runner.run(agent_id, payload);

    let res = tokio::select! {
        _ = cancel_token.cancelled() => {
            Err(AppError::Conflict("Step execution cancelled".to_string()))
        }
        res = tokio::time::timeout(timeout_dur, run_fut) => {
            match res {
                Ok(Ok(out)) => Ok(out),
                Ok(Err(e)) => Err(e),
                Err(_) => Err(AppError::InternalServerError("Agent execution timed out".to_string())),
            }
        }
    };
    let elapsed_ms = Utc::now()
        .signed_duration_since(start_time)
        .num_milliseconds();

    match res {
        Ok(out) => Ok((out, elapsed_ms)),
        Err(e) => Err((e.to_string(), elapsed_ms)),
    }
}

pub(crate) async fn execute_fan_out_step(
    state: Arc<AppState>,
    _step: &WorkflowStep,
    workflow_name: &str,
    context: &Arc<parking_lot::Mutex<serde_json::Value>>,
    fan_out: &FanOutConfig,
    step_config: Option<&StepConfig>,
    step_run_id: &str,
    cancel_token: tokio_util::sync::CancellationToken,
) -> Result<(String, Option<serde_json::Value>), AppError> {
    let items = get_fan_out_items(context, &fan_out.array_path);
    let items_len = items.len();

    if items.is_empty() {
        return Ok((
            "[]".to_string(),
            Some(serde_json::json!({
                "type": "fan_out",
                "status": "empty",
                "items_count": 0,
                "runs": []
            })),
        ));
    }

    if items_len > MAX_FAN_OUT_ITEMS {
        let err_msg = format!(
            "Fan-out array size {} exceeds limit of {}",
            items_len, MAX_FAN_OUT_ITEMS
        );
        return Err(AppError::BadRequest(err_msg));
    }

    let safe_mode = step_config.and_then(|c| c.safe_mode).unwrap_or(true);
    let analysis = step_config.and_then(|c| c.analysis).unwrap_or(false);

    let state_clone = Arc::clone(&state);
    let workflow_name_clone = workflow_name.to_string();
    let context_clone = Arc::clone(context);
    let fan_out_clone = fan_out.clone();
    let step_run_id_str = step_run_id.to_string();
    let cancel_token_clone = cancel_token.clone();

    let streams = futures::stream::iter(items.into_iter().enumerate().map(|(idx, item)| {
        let state = Arc::clone(&state_clone);
        let agent_id = fan_out_clone.agent_id.clone();
        let item_placeholder = fan_out_clone.item_placeholder.clone();
        let item_str = if let Some(s) = item.as_str() {
            s.to_string()
        } else {
            item.to_string()
        };
        let workflow_name = workflow_name_clone.clone();
        let step_run_id_clone = step_run_id_str.clone();
        let cancel_token_item = cancel_token_clone.clone();

        let resolved_prompt = resolve_fan_out_prompt(
            &context_clone,
            &fan_out_clone.prompt_template,
            &item_placeholder,
            &item_str,
        );

        async move {
            tracing::debug!("Starting fan-out item {} ({})", idx, item_str);
            let payload = TaskPayload {
                message: resolved_prompt,
                department: Some(format!("Workflow (Fanout): {}", workflow_name)),
                safe_mode: Some(safe_mode),
                analysis: Some(analysis),
                cluster_id: Some(format!("{}-fanout-{}", step_run_id_clone, idx)),
                ..Default::default()
            };

            let res = run_agent_guarded(&state, agent_id, payload, &cancel_token_item).await;

            match res {
                Ok((out, elapsed_ms)) => Ok(serde_json::json!({
                    "index": idx,
                    "item": item,
                    "status": "completed",
                    "output": truncate_candidate_output(&out),
                    "full_output": out,
                    "elapsed_ms": elapsed_ms
                })),
                Err((err, elapsed_ms)) => Err((idx, item, err, elapsed_ms)),
            }
        }
    }));

    let results = streams
        .buffer_unordered(get_concurrency_limit())
        .collect::<Vec<_>>()
        .await;
    let mut runs = Vec::new();
    let mut errors = Vec::new();
    for res in results {
        match res {
            Ok(run_json) => {
                runs.push(run_json);
            }
            Err((idx, item, err, elapsed_ms)) => {
                errors.push(serde_json::json!({
                    "index": idx,
                    "item": item,
                    "status": "failed",
                    "error": err,
                    "elapsed_ms": elapsed_ms
                }));
            }
        }
    }

    runs.sort_by_key(|r| r["index"].as_u64().unwrap_or(0));
    errors.sort_by_key(|e| e["index"].as_u64().unwrap_or(0));

    // Three-way status distinction: completed / partial_success / failed
    let status = if errors.is_empty() {
        "completed"
    } else if runs.is_empty() {
        "failed"
    } else {
        "partial_success"
    };

    if !errors.is_empty() && matches!(fan_out.fail_strategy, FanOutFailStrategy::FailFast) {
        let detail = format!(
            "Fan-out step failed. {} of {} items errored",
            errors.len(),
            items_len
        );
        let metadata = serde_json::json!({
            "type": "fan_out",
            "status": "failed",
            "items_count": items_len,
            "runs": runs,
            "errors": errors
        });

        return Err(AppError::WorkflowStepFailed {
            step_name: "fan_out".to_string(),
            detail,
            metadata: Some(metadata),
        });
    }

    if runs.is_empty() && items_len > 0 {
        let detail = format!(
            "Fan-out step completely failed. All {} items errored",
            items_len
        );
        let metadata = serde_json::json!({
            "type": "fan_out",
            "status": "failed",
            "items_count": items_len,
            "runs": runs,
            "errors": errors
        });

        return Err(AppError::WorkflowStepFailed {
            step_name: "fan_out".to_string(),
            detail,
            metadata: Some(metadata),
        });
    }

    // Clean JSON array with null for failed items (avoiding ambiguous string markers)
    let outputs_array: Vec<serde_json::Value> = {
        let mut arr = vec![serde_json::Value::Null; items_len];
        for r in &runs {
            if let Some(idx) = r["index"].as_u64() {
                if let Some(out_str) = r["full_output"].as_str() {
                    arr[idx as usize] = serde_json::Value::String(out_str.to_string());
                }
            }
        }
        arr
    };
    let output = serde_json::to_string(&outputs_array)
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;

    // Clean metadata runs (stripping full_output to avoid DB bloat)
    let cleaned_runs: Vec<serde_json::Value> = runs
        .into_iter()
        .map(|mut r| {
            if let Some(obj) = r.as_object_mut() {
                obj.remove("full_output");
            }
            r
        })
        .collect();

    let metadata = Some(serde_json::json!({
        "type": "fan_out",
        "status": status,
        "items_count": items_len,
        "runs": cleaned_runs,
        "errors": errors
    }));

    Ok((output, metadata))
}

pub(crate) async fn execute_tournament_step(
    state: Arc<AppState>,
    _step: &WorkflowStep,
    workflow_name: &str,
    context: &Arc<parking_lot::Mutex<serde_json::Value>>,
    tournament: &TournamentConfig,
    step_config: Option<&StepConfig>,
    step_run_id: &str,
    cancel_token: tokio_util::sync::CancellationToken,
) -> Result<(String, Option<serde_json::Value>), AppError> {
    let safe_mode = step_config.and_then(|c| c.safe_mode).unwrap_or(true);
    let analysis = step_config.and_then(|c| c.analysis).unwrap_or(false);

    let state_clone = Arc::clone(&state);
    let workflow_name_clone = workflow_name.to_string();
    let context_clone = Arc::clone(context);
    let step_run_id_str = step_run_id.to_string();
    let cancel_token_clone = cancel_token.clone();

    let streams = futures::stream::iter(tournament.candidates.clone().into_iter().enumerate().map(
        |(idx, cand)| {
            let state = Arc::clone(&state_clone);
            let agent_id = cand.agent_id.clone();
            let workflow_name = workflow_name_clone.clone();
            let step_run_id_clone = step_run_id_str.clone();
            let cancel_token_cand = cancel_token_clone.clone();

            let resolved_prompt = resolve_tournament_prompt(&context_clone, &cand.prompt_template);

            async move {
                let payload = TaskPayload {
                    message: resolved_prompt,
                    department: Some(format!("Workflow (Tournament): {}", workflow_name)),
                    safe_mode: Some(safe_mode),
                    analysis: Some(analysis),
                    cluster_id: Some(format!(
                        "{}-tournament-candidate-{}",
                        step_run_id_clone, idx
                    )),
                    ..Default::default()
                };

                let res = run_agent_guarded(&state, agent_id, payload, &cancel_token_cand).await;

                match res {
                    Ok((out, elapsed_ms)) => Ok(serde_json::json!({
                        "index": idx,
                        "agent_id": cand.agent_id,
                        "status": "completed",
                        "output": truncate_candidate_output(&out),
                        "full_output": out,
                        "elapsed_ms": elapsed_ms
                    })),
                    Err((err, elapsed_ms)) => Err((idx, cand.agent_id.clone(), err, elapsed_ms)),
                }
            }
        },
    ));

    let results = streams
        .buffer_unordered(get_concurrency_limit())
        .collect::<Vec<_>>()
        .await;
    let mut runs = Vec::new();
    let mut errors = Vec::new();
    for res in results {
        match res {
            Ok(run_json) => {
                runs.push(run_json);
            }
            Err((idx, agent_id, err, elapsed_ms)) => {
                errors.push(serde_json::json!({
                    "index": idx,
                    "agent_id": agent_id,
                    "status": "failed",
                    "error": err,
                    "elapsed_ms": elapsed_ms
                }));
            }
        }
    }

    runs.sort_by_key(|r| r["index"].as_u64().unwrap_or(0));
    errors.sort_by_key(|e| e["index"].as_u64().unwrap_or(0));

    let total_candidates = tournament.candidates.len();
    let min_successful = tournament.min_successful.unwrap_or(total_candidates);

    if runs.len() < min_successful {
        let detail = format!(
            "Tournament step failed: only {} of {} required candidates succeeded ({} errors)",
            runs.len(),
            min_successful,
            errors.len()
        );
        let metadata = serde_json::json!({
            "type": "tournament",
            "status": "failed",
            "candidates": runs,
            "errors": errors
        });

        return Err(AppError::WorkflowStepFailed {
            step_name: "tournament".to_string(),
            detail,
            metadata: Some(metadata),
        });
    }

    let resolved_judge_prompt =
        resolve_tournament_prompt(context, &tournament.judge_prompt_template);
    let mut final_judge_prompt = resolved_judge_prompt;
    for run in &runs {
        let idx = run["index"].as_u64().unwrap_or(0);
        let raw_output = run["full_output"].as_str().unwrap_or("");
        let cand_output = truncate_candidate_output(raw_output);
        final_judge_prompt =
            final_judge_prompt.replace(&format!("{{{{candidate_{}}}}}", idx), &cand_output);
    }

    let judge_payload = TaskPayload {
        message: final_judge_prompt,
        department: Some(format!("Workflow (Judge): {}", workflow_name)),
        safe_mode: Some(safe_mode),
        analysis: Some(analysis),
        cluster_id: Some(format!("{}-tournament-judge", step_run_id)),
        ..Default::default()
    };

    let judge_res = run_agent_guarded(
        &state,
        tournament.judge_agent_id.clone(),
        judge_payload,
        &cancel_token,
    )
    .await;

    // Clean metadata runs (stripping full_output)
    let cleaned_runs: Vec<serde_json::Value> = runs
        .into_iter()
        .map(|mut r| {
            if let Some(obj) = r.as_object_mut() {
                obj.remove("full_output");
            }
            r
        })
        .collect();

    match judge_res {
        Ok((judge_out, judge_elapsed)) => {
            let metadata = Some(serde_json::json!({
                "type": "tournament",
                "status": "completed",
                "candidates": cleaned_runs,
                "errors": errors,
                "judge": {
                    "agent_id": tournament.judge_agent_id,
                    "output": truncate_candidate_output(&judge_out),
                    "elapsed_ms": judge_elapsed
                }
            }));
            Ok((judge_out, metadata))
        }
        Err((err, _)) => {
            let detail = format!("Tournament step judge failed: {}", err);
            let metadata = serde_json::json!({
                "type": "tournament",
                "status": "failed",
                "candidates": cleaned_runs,
                "errors": errors,
                "judge_error": err
            });

            Err(AppError::WorkflowStepFailed {
                step_name: "tournament_judge".to_string(),
                detail,
                metadata: Some(metadata),
            })
        }
    }
}

pub(crate) async fn execute_standard_step(
    state: Arc<AppState>,
    step: &WorkflowStep,
    workflow_name: &str,
    context: &Arc<parking_lot::Mutex<serde_json::Value>>,
    step_config: Option<&StepConfig>,
    step_run_id: &str,
    cancel_token: tokio_util::sync::CancellationToken,
) -> Result<(String, Option<serde_json::Value>), AppError> {
    let mut attempt = 0;
    let max_attempts = step.max_retries.max(0) + 1; // 3 retries = 4 attempts
    let mut last_error = String::new();
    let mut attempts_history = Vec::new();

    let safe_mode = step_config.and_then(|c| c.safe_mode).unwrap_or(true);
    let analysis = step_config.and_then(|c| c.analysis).unwrap_or(false);

    while attempt < max_attempts {
        if attempt > 0 {
            let capped_attempt = attempt.min(10);
            let base_delay =
                step.backoff_factor_secs.max(1) * 2_i64.saturating_pow(capped_attempt as u32 - 1);
            // Add ±20% randomized/subsecond jitter to prevent synchronized retry storms
            let jitter_ms = (Utc::now().timestamp_subsec_millis() as i64 % 400) - 200;
            let delay_ms = (base_delay * 1000 + jitter_ms).max(100);
            tracing::info!(
                "🔄 [Workflow] Step '{}' failed. Retrying in {} ms (attempt {}/{})",
                step.name,
                delay_ms,
                attempt,
                max_attempts - 1
            );
            tokio::select! {
                _ = cancel_token.cancelled() => {
                    return Err(AppError::Conflict("Step execution cancelled".to_string()));
                }
                _ = tokio::time::sleep(std::time::Duration::from_millis(delay_ms as u64)) => {}
            }
        }

        if cancel_token.is_cancelled() {
            return Err(AppError::Conflict("Step execution cancelled".to_string()));
        }

        let (prompt, primary_goal) = resolve_step_prompts(context, &step.prompt_template);

        let payload = TaskPayload {
            message: prompt,
            department: Some(format!("Workflow: {}", workflow_name)),
            safe_mode: Some(safe_mode),
            analysis: Some(analysis),
            primary_goal,
            cluster_id: Some(format!("{}-{}", step_run_id, attempt)),
            ..Default::default()
        };

        let res = run_agent_guarded(&state, step.agent_id.clone(), payload, &cancel_token).await;

        match res {
            Ok((out, elapsed_ms)) => {
                attempts_history.push(serde_json::json!({
                    "attempt": attempt,
                    "status": "completed",
                    "elapsed_ms": elapsed_ms
                }));
                let metadata = Some(serde_json::json!({
                    "type": "standard",
                    "status": "completed",
                    "attempts": attempts_history
                }));
                return Ok((out, metadata));
            }
            Err((err, elapsed_ms)) => {
                if cancel_token.is_cancelled() {
                    return Err(AppError::Conflict("Step execution cancelled".to_string()));
                }
                last_error = err.clone();
                attempts_history.push(serde_json::json!({
                    "attempt": attempt,
                    "status": "failed",
                    "error": err,
                    "elapsed_ms": elapsed_ms
                }));
                attempt += 1;
            }
        }
    }

    let err_msg = format!(
        "Step '{}' failed after {} attempts. Last error: {}",
        step.name, max_attempts, last_error
    );
    tracing::error!("❌ [Workflow] {}", err_msg);

    let metadata = serde_json::json!({
        "type": "standard",
        "status": "failed",
        "attempts": attempts_history,
        "last_error": last_error
    });

    Err(AppError::WorkflowStepFailed {
        step_name: step.name.clone(),
        detail: err_msg,
        metadata: Some(metadata),
    })
}

// ---------------------------------------------------------------------------
// execute_single_step — Thin orchestrator decomposed into 3 phases
// ---------------------------------------------------------------------------

/// Phase 1: Create step_run in DB, parse & validate StepConfig, emit step_started event.
async fn prepare_step_run(
    state: &Arc<AppState>,
    tenant_id: &str,
    run_id: &str,
    step: &WorkflowStep,
) -> Result<
    (
        String,
        crate::agent::continuity::repository::WorkflowRepository,
        Option<StepConfig>,
    ),
    AppError,
> {
    let step_run_id = Uuid::new_v4().to_string();
    let repo =
        crate::agent::continuity::repository::WorkflowRepository::new(state.resources.pool.clone());

    // Create step_run row BEFORE emitting event to guarantee event integrity
    repo.create_step_run(&step_run_id, run_id, &step.id).await?;

    state.emit_event(serde_json::json!({
        "type": "engine:workflow_step_started",
        "data": {
            "tenant_id": tenant_id,
            "run_id": run_id,
            "step_id": step.id,
            "step_name": step.name,
            "step_run_id": step_run_id
        }
    }));

    let step_config = match &step.config {
        Some(c) => match serde_json::from_value::<StepConfig>(c.clone()) {
            Ok(cfg) => Some(cfg),
            Err(e) => {
                let err_msg = format!("Invalid StepConfig: {}", e);
                return Err(AppError::BadRequest(err_msg));
            }
        },
        None => None,
    };

    // Validate mutually-exclusive execution modes and tournament caps
    if let Some(cfg) = step_config.as_ref() {
        cfg.validate()?;
    }

    Ok((step_run_id, repo, step_config))
}

/// Phase 2: Route to fan-out / tournament / standard execution mode.
async fn dispatch_execution(
    state: &Arc<AppState>,
    step: &WorkflowStep,
    workflow_name: &str,
    context: &Arc<parking_lot::Mutex<serde_json::Value>>,
    step_run_id: &str,
    step_config: Option<&StepConfig>,
    cancel_token: tokio_util::sync::CancellationToken,
) -> Result<(String, Option<serde_json::Value>), AppError> {
    if let Some(cfg) = step_config {
        if let Some(fan_out) = &cfg.fan_out {
            return execute_fan_out_step(
                Arc::clone(state),
                step,
                workflow_name,
                context,
                fan_out,
                step_config,
                step_run_id,
                cancel_token,
            )
            .await;
        } else if let Some(tournament) = &cfg.tournament {
            return execute_tournament_step(
                Arc::clone(state),
                step,
                workflow_name,
                context,
                tournament,
                step_config,
                step_run_id,
                cancel_token,
            )
            .await;
        }
    }

    execute_standard_step(
        Arc::clone(state),
        step,
        workflow_name,
        context,
        step_config,
        step_run_id,
        cancel_token,
    )
    .await
}

/// Phase 3: Prune output, insert context key, persist success, emit step_completed event.
async fn finalize_step_run(
    state: &Arc<AppState>,
    tenant_id: &str,
    run_id: &str,
    step: &WorkflowStep,
    context: &Arc<parking_lot::Mutex<serde_json::Value>>,
    step_run_id: &str,
    repo: &crate::agent::continuity::repository::WorkflowRepository,
    output: &str,
    metadata_val: Option<serde_json::Value>,
) -> Result<(), AppError> {
    let pruned = prune_step_output(output, &step.config);
    let key = sanitize_context_key(&step.name);

    if let Err(e) = insert_context_key(context, key, pruned) {
        tracing::error!(
            run_id = %run_id,
            step_id = %step.id,
            step_name = %step.name,
            "⚠️ [Workflow] Context update failed (step execution succeeded): {}",
            e
        );
        state.emit_event(serde_json::json!({
            "type": "engine:workflow_step_context_error",
            "data": {
                "tenant_id": tenant_id,
                "run_id": run_id,
                "step_id": step.id,
                "step_name": step.name,
                "error": e.to_string()
            }
        }));
    }

    let metadata_str = metadata_val.and_then(|v| serde_json::to_string(&v).ok());

    let cost = match repo.get_step_run_cost(step_run_id).await {
        Ok(c) => c,
        Err(e) => {
            tracing::warn!(
                "⚠️ [Workflow] Failed to fetch cost for step_run '{}': {}",
                step_run_id,
                e
            );
            0.0
        }
    };

    repo.update_step_run_success(
        step_run_id,
        output,
        metadata_str.unwrap_or_else(|| "{}".to_string()),
        cost,
    )
    .await?;

    state.emit_event(serde_json::json!({
        "type": "engine:workflow_step_completed",
        "data": {
            "tenant_id": tenant_id,
            "run_id": run_id,
            "step_id": step.id,
            "step_name": step.name,
            "step_run_id": step_run_id
        }
    }));

    Ok(())
}

/// Thin orchestrator: prepare → dispatch → finalize.
pub(crate) fn execute_single_step(
    state: Arc<AppState>,
    tenant_id: String,
    run_id: String,
    step: WorkflowStep,
    workflow_name: String,
    context: Arc<parking_lot::Mutex<serde_json::Value>>,
    cancel_token: tokio_util::sync::CancellationToken,
) -> futures::future::BoxFuture<'static, Result<String, AppError>> {
    Box::pin(async move {
        tracing::info!(
            tenant_id = %tenant_id,
            run_id = %run_id,
            step_id = %step.id,
            step_name = %step.name,
            "🔧 [Workflow] Executing step"
        );

        // Phase 1: Prepare
        let (step_run_id, repo, step_config) =
            match prepare_step_run(&state, &tenant_id, &run_id, &step).await {
                Ok(tuple) => tuple,
                Err(e) => {
                    let err_msg = e.to_string();
                    state.emit_event(serde_json::json!({
                        "type": "engine:workflow_step_failed",
                        "data": {
                            "tenant_id": tenant_id,
                            "run_id": run_id,
                            "step_id": step.id,
                            "step_name": step.name,
                            "error": err_msg
                        }
                    }));
                    return Err(e);
                }
            };

        // Phase 2: Dispatch (Single-writer failure persistence)
        let (output, metadata_val) = match dispatch_execution(
            &state,
            &step,
            &workflow_name,
            &context,
            &step_run_id,
            step_config.as_ref(),
            cancel_token.clone(),
        )
        .await
        {
            Ok(res) => res,
            Err(e) => {
                let (err_msg, failure_meta) = match &e {
                    AppError::WorkflowStepFailed {
                        detail, metadata, ..
                    } => (
                        detail.clone(),
                        metadata.clone().unwrap_or_else(|| serde_json::json!({})),
                    ),
                    _ => (e.to_string(), serde_json::json!({})),
                };
                let meta_str =
                    serde_json::to_string(&failure_meta).unwrap_or_else(|_| "{}".to_string());
                let _ = repo
                    .update_step_run_failed(&step_run_id, &err_msg, meta_str)
                    .await;
                state.emit_event(serde_json::json!({
                    "type": "engine:workflow_step_failed",
                    "data": {
                        "tenant_id": tenant_id,
                        "run_id": run_id,
                        "step_id": step.id,
                        "step_name": step.name,
                        "error": err_msg,
                        "metadata": failure_meta
                    }
                }));
                return Err(e);
            }
        };

        // Phase 3: Finalize
        finalize_step_run(
            &state,
            &tenant_id,
            &run_id,
            &step,
            &context,
            &step_run_id,
            &repo,
            &output,
            metadata_val,
        )
        .await?;

        Ok(output)
    })
}

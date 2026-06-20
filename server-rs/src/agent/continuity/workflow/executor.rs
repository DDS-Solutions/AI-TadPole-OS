//! @docs ARCHITECTURE:Registry
//! 
//! ### AI Assist Note
//! **Core technical resource for the Tadpole OS Sovereign infrastructure.**
//! This module implements high-fidelity logic for the Sovereign Reality layer.
//! 
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: Runtime logic error, state desynchronization, or resource exhaustion.
//! - **Telemetry Link**: Search `[executor]` in tracing logs.

use crate::error::AppError;
use crate::state::AppState;
use crate::agent::runner::AgentRunner;
use crate::agent::types::TaskPayload;
use super::types::{WorkflowStep, StepConfig, FanOutConfig, TournamentConfig, FanOutFailStrategy};
use super::helpers::{
    get_agent_timeout_duration, get_concurrency_limit, resolve_fan_out_prompt,
    resolve_tournament_prompt, resolve_step_prompts, prune_step_output,
    sanitize_context_key, insert_context_key, get_fan_out_items
};
use std::sync::Arc;
use chrono::Utc;
use uuid::Uuid;
use futures::StreamExt;

pub(crate) async fn execute_fan_out_step(
    state: Arc<AppState>,
    _step: &WorkflowStep,
    workflow_name: &str,
    context: &Arc<parking_lot::Mutex<serde_json::Value>>,
    fan_out: &FanOutConfig,
    step_run_id: &str,
    repo: &crate::agent::continuity::repository::WorkflowRepository,
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

    if items_len > 50 {
        let err_msg = format!("Fan-out array size {} exceeds limit of 50", items_len);
        return Err(AppError::BadRequest(err_msg));
    }

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
            if cancel_token_item.is_cancelled() {
                return Err((idx, item, "Step execution cancelled".to_string(), 0));
            }
            tracing::debug!("Starting fan-out item {} ({})", idx, item_str);
            let payload = TaskPayload {
                message: resolved_prompt,
                department: Some(format!("Workflow (Fanout): {}", workflow_name)),
                safe_mode: Some(false),
                analysis: Some(false),
                cluster_id: Some(format!("{}-fanout-{}", step_run_id_clone, idx)),
                ..Default::default()
            };

            let runner = AgentRunner::new(state);
            let start_time = Utc::now();
            let timeout_dur = get_agent_timeout_duration();
            let run_fut = runner.run(agent_id, payload);
            
            let res = tokio::select! {
                _ = cancel_token_item.cancelled() => {
                    Err(AppError::InternalServerError("Step execution cancelled".to_string()))
                }
                res = tokio::time::timeout(timeout_dur, run_fut) => {
                    match res {
                        Ok(inner) => inner,
                        Err(_) => Err(AppError::InternalServerError("Agent execution timed out".to_string())),
                    }
                }
            };
            let elapsed_ms = Utc::now()
                .signed_duration_since(start_time)
                .num_milliseconds();

            match res {
                Ok(out) => Ok(serde_json::json!({
                    "index": idx,
                    "item": item,
                    "status": "completed",
                    "output": out,
                    "elapsed_ms": elapsed_ms
                })),
                Err(e) => Err((idx, item, e.to_string(), elapsed_ms)),
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

    if !errors.is_empty() && matches!(fan_out.fail_strategy, FanOutFailStrategy::FailFast) {
        let err_msg = format!("Fan-out step failed. Errors: {:?}", errors);
        let meta_str = serde_json::to_string(&serde_json::json!({
            "type": "fan_out",
            "status": "failed",
            "items_count": items_len,
            "runs": runs,
            "errors": errors
        }))
        .unwrap();

        repo.update_step_run_failed(step_run_id, &err_msg, meta_str)
            .await?;
        return Err(AppError::InternalServerError(err_msg));
    }

    let outputs_array: Vec<String> = {
        let mut arr = vec![String::new(); items_len];
        for r in &runs {
            if let Some(idx) = r["index"].as_u64() {
                if let Some(out_str) = r["output"].as_str() {
                    arr[idx as usize] = out_str.to_string();
                }
            }
        }
        for err in &errors {
            if let Some(idx) = err["index"].as_u64() {
                if let Some(err_str) = err["error"].as_str() {
                    arr[idx as usize] = format!("ERROR: {}", err_str);
                }
            }
        }
        arr
    };
    let output = serde_json::to_string(&outputs_array).unwrap();
    let metadata = Some(serde_json::json!({
        "type": "fan_out",
        "status": if errors.is_empty() { "completed" } else { "partial_success" },
        "items_count": items_len,
        "runs": runs,
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
    step_run_id: &str,
    repo: &crate::agent::continuity::repository::WorkflowRepository,
    cancel_token: tokio_util::sync::CancellationToken,
) -> Result<(String, Option<serde_json::Value>), AppError> {
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
                if cancel_token_cand.is_cancelled() {
                    return Err((idx, cand.agent_id.clone(), "Step execution cancelled".to_string(), 0));
                }
                let payload = TaskPayload {
                    message: resolved_prompt,
                    department: Some(format!("Workflow (Tournament): {}", workflow_name)),
                    safe_mode: Some(false),
                    analysis: Some(false),
                    cluster_id: Some(format!(
                        "{}-tournament-candidate-{}",
                        step_run_id_clone, idx
                    )),
                    ..Default::default()
                };

                let runner = AgentRunner::new(state);
                let start_time = Utc::now();
                let timeout_dur = get_agent_timeout_duration();
                let run_fut = runner.run(agent_id, payload);
                
                let res = tokio::select! {
                    _ = cancel_token_cand.cancelled() => {
                        Err(AppError::InternalServerError("Step execution cancelled".to_string()))
                    }
                    res = tokio::time::timeout(timeout_dur, run_fut) => {
                        match res {
                            Ok(inner) => inner,
                            Err(_) => Err(AppError::InternalServerError("Agent execution timed out".to_string())),
                        }
                    }
                };
                let elapsed_ms = Utc::now()
                    .signed_duration_since(start_time)
                    .num_milliseconds();

                match res {
                    Ok(out) => Ok(serde_json::json!({
                        "index": idx,
                        "agent_id": cand.agent_id,
                        "status": "completed",
                        "output": out,
                        "elapsed_ms": elapsed_ms
                    })),
                    Err(e) => Err((idx, cand.agent_id.clone(), e.to_string(), elapsed_ms)),
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

    if !errors.is_empty() {
        let err_msg = format!(
            "Tournament step failed because some candidates failed. Errors: {:?}",
            errors
        );
        let meta_str = serde_json::to_string(&serde_json::json!({
            "type": "tournament",
            "status": "failed",
            "candidates": runs,
            "errors": errors
        }))
        .unwrap();

        repo.update_step_run_failed(step_run_id, &err_msg, meta_str)
            .await?;
        return Err(AppError::InternalServerError(err_msg));
    }

    runs.sort_by_key(|r| r["index"].as_u64().unwrap_or(0));

    let resolved_judge_prompt = resolve_tournament_prompt(context, &tournament.judge_prompt_template);
    let mut final_judge_prompt = resolved_judge_prompt;
    for run in &runs {
        let idx = run["index"].as_u64().unwrap_or(0);
        let cand_output = run["output"].as_str().unwrap_or("");
        final_judge_prompt = final_judge_prompt.replace(&format!("{{{{candidate_{}}}}}", idx), cand_output);
    }

    let judge_payload = TaskPayload {
        message: final_judge_prompt,
        department: Some(format!("Workflow (Judge): {}", workflow_name)),
        safe_mode: Some(false),
        analysis: Some(false),
        cluster_id: Some(format!("{}-tournament-judge", step_run_id)),
        ..Default::default()
    };

    let judge_runner = AgentRunner::new(Arc::clone(&state));
    let judge_start_time = Utc::now();
    let timeout_dur = get_agent_timeout_duration();
    let run_fut = judge_runner.run(tournament.judge_agent_id.clone(), judge_payload);
    
    let judge_res = tokio::select! {
        _ = cancel_token.cancelled() => {
            Err(AppError::InternalServerError("Step execution cancelled".to_string()))
        }
        res = tokio::time::timeout(timeout_dur, run_fut) => {
            match res {
                Ok(inner) => inner,
                Err(_) => Err(AppError::InternalServerError("Agent execution timed out".to_string())),
            }
        }
    };
    let judge_elapsed = Utc::now()
        .signed_duration_since(judge_start_time)
        .num_milliseconds();

    match judge_res {
        Ok(judge_out) => {
            let metadata = Some(serde_json::json!({
                "type": "tournament",
                "status": "completed",
                "candidates": runs,
                "judge": {
                    "agent_id": tournament.judge_agent_id,
                    "output": judge_out,
                    "elapsed_ms": judge_elapsed
                }
            }));
            Ok((judge_out, metadata))
        }
        Err(e) => {
            let err_msg = format!("Tournament step judge failed: {}", e);
            let meta_str = serde_json::to_string(&serde_json::json!({
                "type": "tournament",
                "status": "failed",
                "candidates": runs,
                "judge_error": e.to_string()
            }))
            .unwrap();

            repo.update_step_run_failed(step_run_id, &err_msg, meta_str)
                .await?;
            Err(AppError::InternalServerError(err_msg))
        }
    }
}

pub(crate) async fn execute_standard_step(
    state: Arc<AppState>,
    step: &WorkflowStep,
    workflow_name: &str,
    context: &Arc<parking_lot::Mutex<serde_json::Value>>,
    step_run_id: &str,
    repo: &crate::agent::continuity::repository::WorkflowRepository,
    cancel_token: tokio_util::sync::CancellationToken,
) -> Result<(String, Option<serde_json::Value>), AppError> {
    let mut attempt = 0;
    let max_attempts = step.max_retries.max(0) + 1; // 3 retries = 4 attempts
    let mut last_error = String::new();
    let runner = AgentRunner::new(Arc::clone(&state));

    while attempt < max_attempts {
        if attempt > 0 {
            let capped_attempt = attempt.min(10);
            let delay_secs =
                step.backoff_factor_secs.max(1) * 2_i64.saturating_pow(capped_attempt as u32 - 1);
            tracing::info!(
                "🔄 [Workflow] Step '{}' failed. Retrying in {} seconds (attempt {}/{})",
                step.name,
                delay_secs,
                attempt,
                max_attempts - 1
            );
            tokio::select! {
                _ = cancel_token.cancelled() => {
                    return Err(AppError::InternalServerError("Step execution cancelled".to_string()));
                }
                _ = tokio::time::sleep(std::time::Duration::from_secs(delay_secs as u64)) => {}
            }
        }

        if cancel_token.is_cancelled() {
            return Err(AppError::InternalServerError("Step execution cancelled".to_string()));
        }

        let (prompt, primary_goal) = resolve_step_prompts(context, &step.prompt_template);

        let payload = TaskPayload {
            message: prompt,
            department: Some(format!("Workflow: {}", workflow_name)),
            safe_mode: Some(false),
            analysis: Some(false),
            primary_goal,
            cluster_id: Some(format!("{}-{}", step_run_id, attempt)),
            ..Default::default()
        };

        let timeout_dur = get_agent_timeout_duration();
        let run_fut = runner.run(step.agent_id.clone(), payload);
        
        let res = tokio::select! {
            _ = cancel_token.cancelled() => {
                Err(AppError::InternalServerError("Step execution cancelled".to_string()))
            }
            res = tokio::time::timeout(timeout_dur, run_fut) => {
                match res {
                    Ok(Ok(out)) => Ok(out),
                    Ok(Err(e)) => Err(e),
                    Err(_) => Err(AppError::InternalServerError("Agent execution timed out".to_string())),
                }
            }
        };

        match res {
            Ok(out) => {
                return Ok((out, None));
            }
            Err(e) => {
                last_error = e.to_string();
                attempt += 1;
            }
        }
    }

    let err_msg = format!(
        "Step '{}' failed after {} attempts. Last error: {}",
        step.name, max_attempts, last_error
    );
    tracing::error!("❌ [Workflow] {}", err_msg);

    repo.update_step_run_failed(step_run_id, &err_msg, "{}".to_string())
        .await?;
    Err(AppError::InternalServerError(err_msg))
}

pub(crate) async fn execute_single_step(
    state: Arc<AppState>,
    run_id: String,
    step: WorkflowStep,
    workflow_name: String,
    context: Arc<parking_lot::Mutex<serde_json::Value>>,
    cancel_token: tokio_util::sync::CancellationToken,
) -> Result<String, AppError> {
    // Phase 2: Structured tracing for step execution
    tracing::info!(
        run_id = %run_id,
        step_id = %step.id,
        step_name = %step.name,
        "🔧 [Workflow] Executing step"
    );

    // Phase 2: WebSocket event — step started
    state.emit_event(serde_json::json!({
        "type": "engine:workflow_step_started",
        "data": { "run_id": run_id, "step_id": step.id, "step_name": step.name }
    }));

    let step_run_id = Uuid::new_v4().to_string();
    let repo =
        crate::agent::continuity::repository::WorkflowRepository::new(state.resources.pool.clone());
    repo.create_step_run(&step_run_id, &run_id, &step.id)
        .await?;

    let step_config = match &step.config {
        Some(c) => match serde_json::from_value::<StepConfig>(c.clone()) {
            Ok(cfg) => Some(cfg),
            Err(e) => {
                let err_msg = format!("Invalid StepConfig: {}", e);
                let _ = repo.update_step_run_failed(&step_run_id, &err_msg, "{}".to_string()).await;
                state.emit_event(serde_json::json!({
                    "type": "engine:workflow_step_failed",
                    "data": { "run_id": run_id, "step_id": step.id, "step_name": step.name, "error": err_msg }
                }));
                return Err(AppError::InternalServerError(err_msg));
            }
        },
        None => None,
    };

    // Validate mutually-exclusive execution modes
    if let Some(cfg) = step_config.as_ref() {
        if let Err(e) = cfg.validate() {
            let err_msg = e.to_string();
            let _ = repo.update_step_run_failed(&step_run_id, &err_msg, "{}".to_string()).await;
            state.emit_event(serde_json::json!({
                "type": "engine:workflow_step_failed",
                "data": { "run_id": run_id, "step_id": step.id, "step_name": step.name, "error": err_msg }
            }));
            return Err(e);
        }
    }

    // Route to the appropriate execution mode
    let execution_result = if let Some(cfg) = step_config.as_ref() {
        if let Some(fan_out) = &cfg.fan_out {
            execute_fan_out_step(
                Arc::clone(&state),
                &step,
                &workflow_name,
                &context,
                fan_out,
                &step_run_id,
                &repo,
                cancel_token.clone(),
            )
            .await
        } else if let Some(tournament) = &cfg.tournament {
            execute_tournament_step(
                Arc::clone(&state),
                &step,
                &workflow_name,
                &context,
                tournament,
                &step_run_id,
                &repo,
                cancel_token.clone(),
            )
            .await
        } else {
            execute_standard_step(
                Arc::clone(&state),
                &step,
                &workflow_name,
                &context,
                &step_run_id,
                &repo,
                cancel_token.clone(),
            )
            .await
        }
    } else {
        execute_standard_step(
            Arc::clone(&state),
            &step,
            &workflow_name,
            &context,
            &step_run_id,
            &repo,
            cancel_token.clone(),
        )
        .await
    };

    let (output, metadata_val) = match execution_result {
        Ok(res) => res,
        Err(e) => {
            let err_msg = e.to_string();
            let _ = repo.update_step_run_failed(&step_run_id, &err_msg, "{}".to_string()).await;
            // Phase 2: WebSocket event — step failed
            state.emit_event(serde_json::json!({
                "type": "engine:workflow_step_failed",
                "data": { "run_id": run_id, "step_id": step.id, "step_name": step.name, "error": err_msg }
            }));
            return Err(e);
        }
    };

    // Shared finalization: prune output, insert into context, persist step run success
    let pruned = prune_step_output(&output, &step.config);
    let key = sanitize_context_key(&step.name);
    if let Err(e) = insert_context_key(&context, key, pruned) {
        let err_msg = format!("Context update failed: {}", e);
        repo.update_step_run_failed(&step_run_id, &err_msg, "{}".to_string())
            .await?;
        // Phase 2: WebSocket event — step failed
        state.emit_event(serde_json::json!({
            "type": "engine:workflow_step_failed",
            "data": { "run_id": run_id, "step_id": step.id, "step_name": step.name, "error": err_msg }
        }));
        return Err(e);
    }

    let metadata_str = metadata_val.and_then(|v| serde_json::to_string(&v).ok());

    let cost = repo.get_step_run_cost(&step_run_id).await.unwrap_or(0.0);

    repo.update_step_run_success(
        &step_run_id,
        &output,
        metadata_str.unwrap_or_else(|| "{}".to_string()),
        cost,
    )
    .await?;

    // Phase 2: WebSocket event — step completed
    state.emit_event(serde_json::json!({
        "type": "engine:workflow_step_completed",
        "data": { "run_id": run_id, "step_id": step.id, "step_name": step.name }
    }));

    Ok(output)
}

// Metadata: [executor]

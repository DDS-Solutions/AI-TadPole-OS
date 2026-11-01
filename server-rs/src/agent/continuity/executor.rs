//! @docs ARCHITECTURE:Continuity
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Agent Runner / executor
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: `[Continuity]`
//!
//! ### AI Assist Note
//! - Keep cancellation ownership and usage persistence aligned with the existing execution paths.

use chrono::Utc;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Semaphore;
use uuid::Uuid;

use super::scheduler::{
    complete_job_run, create_job_run, get_due_jobs, get_job_financial_spend, record_job_tick,
    recover_interrupted_jobs,
};
use super::types::JobRunStatus;
use crate::agent::runner::AgentRunner;
use crate::agent::types::TaskPayload;
use crate::error::AppError;
use crate::state::AppState;

pub const CONTINUITY_DEFAULT_CONCURRENCY: usize = 10;

/// RAII Guard that guarantees agent release on drop if the task panics or aborts prematurely.
struct AgentClaimGuard {
    pool: sqlx::SqlitePool,
    agent_id: String,
    finalized: bool,
    comms: Arc<crate::state::hubs::comm::CommunicationHub>,
}

impl AgentClaimGuard {
    fn new(
        pool: sqlx::SqlitePool,
        agent_id: String,
        comms: Arc<crate::state::hubs::comm::CommunicationHub>,
    ) -> Self {
        Self {
            pool,
            agent_id,
            finalized: false,
            comms,
        }
    }

    fn finalize(&mut self) {
        self.finalized = true;
    }
}

impl Drop for AgentClaimGuard {
    fn drop(&mut self) {
        if !self.finalized {
            let pool = self.pool.clone();
            let agent_id = self.agent_id.clone();
            let teardown = self.comms.begin_teardown(&agent_id);
            tokio::spawn(async move {
                let _teardown = teardown;
                if let Err(e) = crate::agent::persistence::release_agent(&pool, &agent_id).await {
                    tracing::error!(
                        "❌ [Continuity] Failed to release agent claim in Drop guard for agent '{}': {}",
                        agent_id,
                        e
                    );
                }
            });
        }
    }
}

/// Starts the continuity scheduler background task.
///
/// Runs every 60 seconds, looking for "scheduled_jobs" rows where `next_run_at <= now`.
/// For each due job, it spawns a mission via `AgentRunner::run()` and records the run.
///
/// Safety guarantees baked in:
/// - Each agent is limited to 1 concurrent scheduled job (skip if agent is "busy").
/// - Each job has a `budget_usd` cap enforced by the existing `AgentRunner` budget logic.
/// - Consecutive failures auto-disable a job after `max_failures` runs.
/// - Token burn: controlled by the agent's model rate limits.
pub async fn start_scheduler(state: Arc<AppState>) {
    tracing::info!("🕐 [Continuity] Scheduler started — checking for due jobs every 60 seconds.");

    // Run recovery on startup to clean up any stuck running tasks from past sessions
    if let Err(e) = recover_interrupted_jobs(&state.resources.pool).await {
        tracing::error!("❌ [Continuity] Failed to recover interrupted jobs: {}", e);
    }

    // Recover orphaned workflow runs from past sessions
    if let Err(e) =
        crate::agent::continuity::repository::WorkflowRepository::recover_interrupted_workflow_runs(
            &state.resources.pool,
        )
        .await
    {
        tracing::error!(
            "❌ [Continuity] Failed to recover interrupted workflow runs: {}",
            e
        );
    }

    let concurrency_limit = std::env::var("CONTINUITY_CONCURRENCY_LIMIT")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(CONTINUITY_DEFAULT_CONCURRENCY);

    let semaphore = Arc::new(Semaphore::new(concurrency_limit));
    let mut interval = tokio::time::interval(Duration::from_secs(60));
    interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

    loop {
        interval.tick().await;

        let mem_threshold = std::env::var("CONTINUITY_MEM_THRESHOLD")
            .ok()
            .and_then(|v| v.parse::<f64>().ok())
            .unwrap_or(0.85);

        let stats = state.security.system_monitor.get_system_defense_stats();
        if stats.memory_pressure > mem_threshold {
            tracing::warn!(
                "⚠️ [Resource Guardian] High memory pressure ({:.1}%) detected (threshold: {:.1}%). Postponing scheduled job execution tick.",
                stats.memory_pressure * 100.0,
                mem_threshold * 100.0
            );
        } else {
            if let Err(e) = tick(&state, &semaphore, concurrency_limit).await {
                tracing::error!("❌ [Continuity] Scheduler tick error: {}", e);
            }
        }

        // Reaper: Clean up orphans every tick (60s)
        if let Err(e) =
            crate::agent::persistence::reap_stale_agents(&state.resources.pool, 300).await
        {
            tracing::error!("❌ [Continuity] Failed to reap stale agents: {}", e);
        }

        // A2A Lock Sweeper: Roll back expired transaction locks
        if let Err(e) =
            crate::agent::runner::a2a_ledger::A2ATransactionCoordinator::sweep_expired_locks(
                &state.resources.pool,
            )
            .await
        {
            tracing::error!("❌ [A2A Sweep] Failed to sweep expired locks: {:?}", e);
        }

        // A2A Mailbox Sweeper: Deliver pending directives to idle recipient agents
        if let Err(e) =
            crate::agent::runner::a2a_mailbox::A2AMailbox::sweep_and_deliver_pending(&state).await
        {
            tracing::error!(
                "❌ [A2A Mailbox Sweep] Failed to sweep pending mailbox envelopes: {:?}",
                e
            );
        }
    }
}

/// A single scheduler tick: fetch due jobs and execute them.
async fn tick(
    state: &Arc<AppState>,
    semaphore: &Arc<Semaphore>,
    concurrency_limit: usize,
) -> Result<(), AppError> {
    if state
        .comms
        .engine_stopped
        .load(std::sync::atomic::Ordering::Acquire)
    {
        return Ok(());
    }
    let due_jobs = get_due_jobs(&state.resources.pool).await?;

    if due_jobs.is_empty() {
        return Ok(());
    }

    tracing::info!(
        "🕐 [Continuity] {} job(s) due for execution.",
        due_jobs.len()
    );

    for job in due_jobs {
        let permit = match semaphore.clone().try_acquire_owned() {
            Ok(p) => p,
            Err(_) => {
                tracing::warn!(
                    "⚠️ [Continuity] Scheduled job concurrency limit of {} reached. Postponing remaining due jobs to next tick.",
                    concurrency_limit
                );
                break;
            }
        };
        let state_clone = Arc::clone(state);
        let job_clone = job.clone();

        tokio::spawn(async move {
            let _permit = permit;
            execute_job(state_clone, job_clone).await;
        });
    }

    Ok(())
}

/// Executes a single scheduled job:
/// 1. Atomically claims the agent → skip if busy (prevents concurrency conflicts).
/// 2. Creates a run record.
/// 3. Executes workflow or calls `AgentRunner::run()`.
/// 4. Finalises the run record with financial spend and advances cron.
pub fn execute_job(
    state: Arc<AppState>,
    job: super::types::ScheduledJob,
) -> futures::future::BoxFuture<'static, ()> {
    execute_job_with_run(state, job, None)
}

/// Executes a single scheduled job with optional pre-created run record:
/// 1. Atomically claims the agent → skip if busy (prevents concurrency conflicts).
/// 2. Creates a run record if not already provided.
/// 3. Executes workflow or calls `AgentRunner::run()`.
/// 4. Finalises the run record with financial spend and advances cron.
pub fn execute_job_with_run(
    state: Arc<AppState>,
    job: super::types::ScheduledJob,
    existing_run: Option<super::types::ScheduledJobRun>,
) -> futures::future::BoxFuture<'static, ()> {
    Box::pin(async move {
        // Register before claims or run creation; emergency stop also blocks later ticks.
        let Ok((_execution_guard, cancel)) = state.comms.begin_execution(&job.agent_id) else {
            return;
        };
        let job_id = job.id.clone();
        let cancelled = tokio::select! {
            biased;
            _ = cancel.cancelled() => true,
            _ = execute_job_with_run_inner(state.clone(), job, existing_run) => false,
        };
        if cancelled {
            if let Err(error) = sqlx::query("UPDATE scheduled_job_runs SET status = 'failed', completed_at = CURRENT_TIMESTAMP, output_summary = 'Execution cancelled' WHERE job_id = ? AND status = 'running'")
                .bind(&job_id).execute(&state.resources.pool).await {
                tracing::error!("[Continuity] Failed to persist cancelled job {}: {}", job_id, error);
            }
        }
    })
}

fn execute_job_with_run_inner(
    state: Arc<AppState>,
    job: super::types::ScheduledJob,
    existing_run: Option<super::types::ScheduledJobRun>,
) -> futures::future::BoxFuture<'static, ()> {
    Box::pin(async move {
        tracing::info!(
            "🚀 [Continuity] Starting job '{}' for agent '{}' (budget: ${:.3})",
            job.name,
            job.agent_id,
            job.budget_usd
        );

        let tenant_id = job
            .metadata
            .as_ref()
            .and_then(|m| m.get("tenant_id"))
            .and_then(|v| v.as_str())
            .unwrap_or("default");

        // 1. Skip if agent is busy - ATOMIC CLAIM (only applicable if agent_id is specified)
        let claim_guard = if !job.agent_id.is_empty() {
            match crate::agent::persistence::claim_agent(&state.resources.pool, &job.agent_id).await
            {
                Ok(true) => {}
                Ok(false) => {
                    tracing::warn!(
                        "⏭ [Continuity] Skipping job '{}' — agent '{}' is currently busy.",
                        job.name,
                        job.agent_id
                    );
                    let run_res = match existing_run {
                        Some(r) => Ok(r),
                        None => create_job_run(&state.resources.pool, &job.id).await,
                    };
                    if let Ok(run) = run_res {
                        let _ = complete_job_run(
                            &state.resources.pool,
                            &run.id,
                            None,
                            JobRunStatus::Skipped,
                            0.0,
                            Some("Agent was busy when job was scheduled."),
                        )
                        .await;
                    }
                    return;
                }
                Err(e) => {
                    tracing::error!(
                        "❌ [Continuity] Database error checking/claiming agent '{}' for job '{}': {}",
                        job.agent_id,
                        job.name,
                        e
                    );
                    return;
                }
            }
            Some(AgentClaimGuard::new(
                state.resources.pool.clone(),
                job.agent_id.clone(),
                state.comms.clone(),
            ))
        } else {
            // Workflow-only job (agent_id is empty): ensure no concurrent execution of the same job
            if existing_run.is_none() {
                let is_running: bool = sqlx::query_scalar(
                    "SELECT EXISTS(SELECT 1 FROM scheduled_job_runs WHERE job_id = ?1 AND status = 'running')",
                )
                .bind(&job.id)
                .fetch_one(&state.resources.pool)
                .await
                .unwrap_or(false);

                if is_running {
                    tracing::warn!(
                        "⏭ [Continuity] Skipping workflow job '{}' — previous execution is still running.",
                        job.name
                    );
                    return;
                }
            }
            None
        };

        // 2. Create or reuse run record
        let run = match existing_run {
            Some(r) => r,
            None => match create_job_run(&state.resources.pool, &job.id).await {
                Ok(r) => r,
                Err(e) => {
                    tracing::error!(
                        "❌ [Continuity] Failed to create run record for job '{}': {}",
                        job.id,
                        e
                    );
                    return;
                }
            },
        };

        // 3. Spawn keep-alive task to maintain agent heartbeat during execution (if agent claimed)
        let (heartbeat_cancel_tx, heartbeat_task) = if !job.agent_id.is_empty() {
            let heartbeat_agent_id = job.agent_id.clone();
            let heartbeat_pool = state.resources.pool.clone();
            let (tx, mut rx) = tokio::sync::oneshot::channel::<()>();

            let task = tokio::spawn(async move {
                let mut interval = tokio::time::interval(Duration::from_secs(10));
                let mut ticks = 0u64;
                loop {
                    tokio::select! {
                        _ = interval.tick() => {
                            ticks += 1;
                            if let Err(e) = crate::agent::persistence::update_agent_heartbeat(&heartbeat_pool, &heartbeat_agent_id).await {
                                tracing::error!("❌ [Continuity] Failed to update agent heartbeat for active job: {}", e);
                            } else if ticks.is_multiple_of(6) {
                                tracing::debug!("💓 [Continuity] Heartbeat active for agent '{}' (elapsed: ~{}s)", heartbeat_agent_id, ticks * 10);
                            }
                        }
                        _ = &mut rx => {
                            break;
                        }
                    }
                }
            });
            (Some(tx), Some(task))
        } else {
            (None, None)
        };

        // Configurable timeouts
        let mission_timeout = Duration::from_secs(
            std::env::var("CONTINUITY_MISSION_TIMEOUT_SECS")
                .or_else(|_| std::env::var("CONTINUITY_JOB_TIMEOUT_SECS"))
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(300),
        );

        let workflow_timeout = Duration::from_secs(
            std::env::var("CONTINUITY_WORKFLOW_TIMEOUT_SECS")
                .or_else(|_| std::env::var("CONTINUITY_JOB_TIMEOUT_SECS"))
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(900),
        );

        let safe_mode = job
            .metadata
            .as_ref()
            .and_then(|m| m.get("safe_mode"))
            .and_then(|v| v.as_bool())
            .unwrap_or_else(|| {
                std::env::var("CONTINUITY_SAFE_MODE")
                    .map(|v| v.to_lowercase() == "true" || v == "1")
                    .unwrap_or(true)
            });

        let analysis = job
            .metadata
            .as_ref()
            .and_then(|m| m.get("analysis"))
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        // 4. Branch: Workflow vs Atomic Agent Mission
        let (result_output, resolved_mission_id, run_workflow_id) = if let Some(workflow_id) =
            &job.workflow_id
        {
            let wf_run_id = Uuid::new_v4().to_string();
            tracing::info!(
                "🔄 [Continuity] Executing workflow '{}' (wf_id: {}, run_id: {})",
                job.name,
                workflow_id,
                wf_run_id
            );
            let workflow_engine = super::workflow::WorkflowEngine::new(Arc::clone(&state));
            let wf_fut = workflow_engine.run_workflow_with_id(
                tenant_id,
                workflow_id,
                &wf_run_id,
                serde_json::json!({
                    "budget_usd": job.budget_usd
                }),
            );
            match tokio::time::timeout(workflow_timeout, wf_fut).await {
                Ok(Ok((out, _))) => (Ok(out), Some(wf_run_id.clone()), Some(wf_run_id)),
                Ok(Err(e)) => (Err(e), Some(wf_run_id.clone()), Some(wf_run_id)),
                Err(_) => (
                    Err(AppError::InternalServerError(
                        "Continuity workflow execution timed out".to_string(),
                    )),
                    Some(wf_run_id.clone()),
                    Some(wf_run_id),
                ),
            }
        } else {
            if job.agent_id.is_empty() {
                (
                    Err(AppError::BadRequest(
                        "Continuity atomic mission requires a non-empty agent_id".to_string(),
                    )),
                    None,
                    None,
                )
            } else {
                let atomic_mission_id = format!("continuity-{}", run.id);
                let payload = TaskPayload {
                    message: job.prompt.clone(),
                    department: Some(format!("Continuity: {}", job.name)),
                    budget_usd: Some(job.budget_usd),
                    safe_mode: Some(safe_mode),
                    analysis: Some(analysis),
                    cluster_id: Some(atomic_mission_id.clone()),
                    ..Default::default()
                };

                let runner = AgentRunner::new(Arc::clone(&state));
                let runner_fut = runner.run(job.agent_id.clone(), payload);
                match tokio::time::timeout(mission_timeout, runner_fut).await {
                    Ok(Ok(out)) => (Ok(out), Some(atomic_mission_id.clone()), None),
                    Ok(Err(e)) => (Err(e), Some(atomic_mission_id.clone()), None),
                    Err(_) => {
                        let pool = state.resources.pool.clone();
                        let aid = job.agent_id.clone();
                        let mid = atomic_mission_id.clone();
                        let _ = sqlx::query(
                                "UPDATE mission_history SET status = 'failed', updated_at = CURRENT_TIMESTAMP WHERE (id = ? OR agent_id = ?) AND status IN ('pending', 'active')",
                            )
                            .bind(&mid)
                            .bind(&aid)
                            .execute(&pool)
                            .await;

                        if let Some(mut entry) = state.registry.agents.get_mut(&aid) {
                            let a = entry.value_mut();
                            a.health.status = "idle".to_string();
                            a.state.current_task = None;
                            a.state.active_mission = None;
                        }

                        if let Ok(Some(mut db_agent)) =
                            crate::agent::persistence::load_agent_by_id_db(&pool, &aid).await
                        {
                            db_agent.health.status = "idle".to_string();
                            db_agent.state.current_task = None;
                            db_agent.state.active_mission = None;
                            let _ = crate::agent::persistence::save_agent_db(&pool, &mut db_agent)
                                .await;
                        }

                        (
                            Err(AppError::InternalServerError(
                                "Continuity agent mission timed out".to_string(),
                            )),
                            Some(atomic_mission_id.clone()),
                            None,
                        )
                    }
                }
            }
        };

        // Stop the heartbeat keep-alive task if active
        if let Some(tx) = heartbeat_cancel_tx {
            let _ = tx.send(());
        }
        if let Some(task) = heartbeat_task {
            let _ = task.await;
        }

        // Query financial spend from mission_history ledger on BOTH success and failure
        let cost_usd = if let Some(ref m_id) = resolved_mission_id {
            get_job_financial_spend(&state.resources.pool, m_id).await
        } else {
            0.0
        };

        // 5. Finalise run record and compute final status
        let (success, status, output_summary) = match result_output {
            Ok(output) => {
                let summary = if output.chars().count() > 500 {
                    format!("{}...", output.chars().take(500).collect::<String>())
                } else {
                    output
                };

                let final_status = if job.budget_usd > 0.0 && cost_usd > job.budget_usd {
                    tracing::warn!(
                        "⚠️ [Continuity] Job '{}' exceeded budget: spend=${:.4}, cap=${:.4}",
                        job.name,
                        cost_usd,
                        job.budget_usd
                    );
                    JobRunStatus::BudgetExceeded
                } else {
                    JobRunStatus::Completed
                };

                (true, final_status, Some(summary))
            }
            Err(e) => {
                tracing::error!("❌ [Continuity] Job '{}' failed: {}", job.name, e);
                let msg = format!("Error: {}", e);
                (false, JobRunStatus::Failed, Some(msg))
            }
        };

        if let Err(e) = complete_job_run(
            &state.resources.pool,
            &run.id,
            resolved_mission_id.as_deref(),
            status.clone(),
            cost_usd,
            output_summary.as_deref(),
        )
        .await
        {
            tracing::error!(
                "❌ [Continuity] Failed to finalise run record for job '{}': {}",
                job.name,
                e
            );
        }

        // Retry recording tick up to 3 times to prevent duplicate job triggers on transient SQLite locks
        for tick_attempt in 1..=3 {
            match record_job_tick(
                &state.resources.pool,
                &job.id,
                &job.cron_expr,
                success,
                job.max_failures,
            )
            .await
            {
                Ok(_) => break,
                Err(e) => {
                    if tick_attempt == 3 {
                        tracing::error!(
                            "❌ [Continuity] Failed to record job tick for job '{}' after 3 attempts: {}",
                            job.name,
                            e
                        );
                    } else {
                        tokio::time::sleep(Duration::from_millis(50 * tick_attempt)).await;
                    }
                }
            }
        }

        // 6. Release agent claim and finalize guard
        if let Some(mut guard) = claim_guard {
            if let Err(e) =
                crate::agent::persistence::release_agent(&state.resources.pool, &job.agent_id).await
            {
                tracing::error!(
                    "❌ [Continuity] Failed to release agent claim for job '{}': {}",
                    job.name,
                    e
                );
            }
            guard.finalize();
        }

        // Emit WebSocket event with comprehensive outcome linkage
        state.emit_event(serde_json::json!({
            "type": "engine:scheduled_job_complete",
            "data": {
                "tenant_id": tenant_id,
                "job_id": job.id,
                "job_name": job.name,
                "agent_id": job.agent_id,
                "run_id": run.id,
                "mission_id": resolved_mission_id,
                "workflow_run_id": run_workflow_id,
                "cost_usd": cost_usd,
                "status": status.as_str(),
                "success": success,
                "timestamp": Utc::now().to_rfc3339(),
            }
        }));

        tracing::info!(
            "✅ [Continuity] Job '{}' finished (status={}, cost=${:.4})",
            job.name,
            status.as_str(),
            cost_usd
        );
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::continuity::scheduler::create_job;
    use crate::agent::continuity::types::CreateJobRequest;
    use crate::db::init_db;
    use std::sync::atomic::Ordering;

    #[tokio::test]
    async fn kill_cancels_inflight_provider_for_scheduled_and_workflow_entrypoints() {
        use crate::agent::continuity::workflow::WorkflowEngine;
        use crate::agent::types::{EngineAgent, ModelConfig, ModelProvider};
        use axum::{
            extract::State,
            response::IntoResponse,
            routing::{get, post},
            Json, Router,
        };

        for workflow in [false, true] {
            let requested = Arc::new(tokio::sync::Notify::new());
            let release = Arc::new(tokio::sync::Notify::new());
            let response_sent = Arc::new(tokio::sync::Notify::new());
            let provider_requested = requested.clone();
            let provider_release = release.clone();
            let provider_response = response_sent.clone();
            let app = Router::new()
                .route("/v1/models", get(|| async {
                    Json(serde_json::json!({"data": [{"id": "audit-blocked-provider"}]}))
                }))
                .route("/v1/chat/completions", post(move || {
                    let requested = provider_requested.clone();
                    let release = provider_release.clone();
                    let response_sent = provider_response.clone();
                    async move {
                        let (reply, response) = tokio::sync::oneshot::channel();
                        // Remote provider work can finish after its HTTP caller disconnects.
                        tokio::spawn(async move {
                            release.notified().await;
                            response_sent.notify_one();
                            let _ = reply.send(Json(serde_json::json!({
                                "choices": [{"message": {"role": "assistant", "content": "late result"}, "finish_reason": "stop"}],
                                "usage": {"prompt_tokens": 1, "completion_tokens": 1, "total_tokens": 2}
                            })));
                        });
                        requested.notify_one();
                        response.await.unwrap()
                    }
                }));
            let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
            let address = listener.local_addr().unwrap();
            let server = tokio::spawn(async move {
                axum::serve(listener, app).await.unwrap();
            });
            let state = Arc::new(AppState::new_minimal_mock().await);
            let mut agent = EngineAgent::default();
            agent.identity.id = "inflight-stop-agent".to_string();
            agent.identity.role = "Specialist".to_string();
            agent.health.status = "idle".to_string();
            agent.economics.budget_usd = 0.5;
            agent.models.model = ModelConfig {
                provider: ModelProvider::Openai,
                model_id: "audit-blocked-provider".to_string(),
                api_key: Some("local-test-only".to_string()),
                base_url: Some(format!("http://{address}/v1")),
                input_cost_per_m: Some(0.0),
                output_cost_per_m: Some(0.0),
                max_tokens: Some(1),
                ..Default::default()
            };
            crate::agent::persistence::save_agent_db(&state.resources.pool, &mut agent)
                .await
                .unwrap();
            state
                .registry
                .agents
                .insert(agent.identity.id.clone(), agent);
            let work_state = state.clone();
            let mut work = if workflow {
                let engine = WorkflowEngine::new(work_state);
                let wf = engine
                    .create_workflow("default", "Blocked workflow".to_string(), None)
                    .await
                    .unwrap();
                engine
                    .add_step(
                        "default",
                        &wf.id,
                        "inflight-stop-agent",
                        "Blocked step".to_string(),
                        "Return a brief result".to_string(),
                        1,
                        Some(0),
                        None,
                        None,
                    )
                    .await
                    .unwrap();
                tokio::spawn(async move {
                    engine
                        .run_workflow("default", &wf.id, serde_json::json!({"budget_usd": 0.5}))
                        .await
                        .map(|_| ())
                })
            } else {
                let job = create_job(
                    &state.resources.pool,
                    CreateJobRequest {
                        agent_id: "inflight-stop-agent".to_string(),
                        workflow_id: None,
                        name: "Blocked schedule".to_string(),
                        prompt: "Return a brief result".to_string(),
                        cron_expr: "* * * * *".to_string(),
                        budget_usd: Some(0.5),
                        max_failures: None,
                        metadata: Some(serde_json::json!({"safe_mode": true, "analysis": false})),
                    },
                )
                .await
                .unwrap();
                tokio::spawn(async move {
                    execute_job(work_state, job).await;
                    Ok::<(), AppError>(())
                })
            };
            tokio::time::timeout(Duration::from_secs(15), async {
                tokio::select! {
                    _ = requested.notified() => {},
                    result = &mut work => panic!("Entry point exited before reaching the blocked provider: {result:?}"),
                }
            }).await.expect("Entry point must reach a real loopback provider request");

            let response = crate::routes::engine_control::kill_agents(State(state.clone()))
                .await
                .into_response();
            assert_eq!(response.status(), axum::http::StatusCode::OK);
            let result = tokio::time::timeout(Duration::from_secs(5), work)
                .await
                .expect("Kill must finish the real entry point before the provider is released")
                .unwrap();
            assert_eq!(result.is_err(), workflow);
            assert!(state.comms.active_executions.is_empty());
            assert!(state.resources.workflow_active_runs.is_empty());
            assert_eq!(state.governance.active_agents.load(Ordering::Acquire), 0);
            assert!(state.comms.begin_execution("later-scheduler-tick").is_err());
            let run_status: String = sqlx::query_scalar(if workflow {
                "SELECT status FROM workflow_runs"
            } else {
                "SELECT status FROM scheduled_job_runs"
            })
            .fetch_one(&state.resources.pool)
            .await
            .unwrap();
            assert!(matches!(run_status.as_str(), "failed" | "cancelled"));

            // The real entry point and owned cleanup have finished before this late response.
            release.notify_one();
            tokio::time::timeout(Duration::from_secs(5), response_sent.notified())
                .await
                .unwrap();
            let saved = crate::agent::persistence::load_agent_by_id_db(
                &state.resources.pool,
                "inflight-stop-agent",
            )
            .await
            .unwrap()
            .unwrap();
            let live = state.registry.agents.get("inflight-stop-agent").unwrap();
            assert_eq!(live.health.status, "idle");
            assert_eq!(saved.health.status, "idle");
            assert!(live.state.current_task.is_none() && saved.state.current_task.is_none());
            assert!(live.state.active_mission.is_none() && saved.state.active_mission.is_none());
            drop(live);
            server.abort();
            let _ = server.await;
        }
    }

    #[tokio::test]
    async fn stopped_scheduler_does_not_claim_agent_or_create_run() {
        let state = Arc::new(AppState::new_minimal_mock().await);
        let mut agent = crate::agent::types::EngineAgent::default();
        agent.identity.id = "stopped-schedule-agent".to_string();
        crate::agent::persistence::save_agent_db(&state.resources.pool, &mut agent)
            .await
            .unwrap();
        state
            .registry
            .agents
            .insert(agent.identity.id.clone(), agent);
        let job = create_job(
            &state.resources.pool,
            CreateJobRequest {
                agent_id: "stopped-schedule-agent".to_string(),
                workflow_id: None,
                name: "Stopped schedule".to_string(),
                prompt: "safe task".to_string(),
                cron_expr: "* * * * *".to_string(),
                budget_usd: Some(0.1),
                max_failures: None,
                metadata: None,
            },
        )
        .await
        .unwrap();
        state.comms.engine_stopped.store(true, Ordering::Release);
        execute_job(state.clone(), job).await;
        let runs: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM scheduled_job_runs")
            .fetch_one(&state.resources.pool)
            .await
            .unwrap();
        assert_eq!(runs, 0);
        assert!(crate::agent::persistence::claim_agent(
            &state.resources.pool,
            "stopped-schedule-agent"
        )
        .await
        .unwrap());
        crate::agent::persistence::release_agent(&state.resources.pool, "stopped-schedule-agent")
            .await
            .unwrap();
        assert!(state.comms.active_executions.is_empty());
    }

    #[tokio::test]
    async fn test_execute_job_busy_skip() -> Result<(), Box<dyn std::error::Error>> {
        let pool = init_db("sqlite::memory:").await?;
        let state = Arc::new(AppState::with_pool(pool.clone()).await);
        state
            .governance
            .null_providers_test_mode
            .store(true, Ordering::Relaxed);

        // 1. Create agent and make it busy
        sqlx::query("INSERT INTO agents (id, name, role, department, description, status, skills, workflows, mcp_tools, metadata) \
                     VALUES ('agent-1', 'Agent 1', 'Specialist', 'Core', 'Desc', 'busy', '[]', '[]', '[]', '{}')")
            .execute(&pool)
            .await?;

        let req = CreateJobRequest {
            agent_id: "agent-1".to_string(),
            workflow_id: None,
            name: "Test Job".to_string(),
            prompt: "Say hello".to_string(),
            cron_expr: "* * * * *".to_string(),
            budget_usd: None,
            max_failures: None,
            metadata: None,
        };
        let job = create_job(&pool, req).await?;

        // 2. Execute job - should skip
        execute_job(Arc::clone(&state), job.clone()).await;

        // 3. Verify run record is 'skipped'
        let run_status: String =
            sqlx::query_scalar("SELECT status FROM scheduled_job_runs WHERE job_id = ?1")
                .bind(&job.id)
                .fetch_one(&pool)
                .await?;
        assert_eq!(run_status, "skipped");

        Ok(())
    }

    #[tokio::test]
    async fn test_execute_job_releases_agent() -> Result<(), Box<dyn std::error::Error>> {
        let pool = init_db("sqlite::memory:").await?;
        let state = Arc::new(AppState::with_pool(pool.clone()).await);
        state
            .governance
            .null_providers_test_mode
            .store(true, Ordering::Relaxed);

        // 1. Insert agent as idle
        sqlx::query("INSERT INTO agents (id, name, role, department, description, status, skills, workflows, mcp_tools, metadata) \
                     VALUES ('agent-2', 'Agent 2', 'Specialist', 'Core', 'Desc', 'idle', '[]', '[]', '[]', '{}')")
            .execute(&pool)
            .await?;

        let req = CreateJobRequest {
            agent_id: "agent-2".to_string(),
            workflow_id: None,
            name: "Idle Test Job".to_string(),
            prompt: "Say hello".to_string(),
            cron_expr: "* * * * *".to_string(),
            budget_usd: None,
            max_failures: None,
            metadata: None,
        };
        let job = create_job(&pool, req).await?;

        // 2. Execute job
        execute_job(Arc::clone(&state), job.clone()).await;

        // 3. Verify agent status was released back to 'idle'
        let agent_status: String = sqlx::query_scalar("SELECT status FROM agents WHERE id = ?1")
            .bind(&job.agent_id)
            .fetch_one(&pool)
            .await?;
        assert_eq!(agent_status, "idle");

        Ok(())
    }

    #[tokio::test]
    async fn test_execute_job_failure_preserves_spend_and_releases_agent(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let pool = init_db("sqlite::memory:").await?;
        let state = Arc::new(AppState::with_pool(pool.clone()).await);
        // Do NOT enable null_providers_test_mode so the mission runner fails deterministically

        sqlx::query("INSERT INTO agents (id, name, role, department, description, status, skills, workflows, mcp_tools, metadata) \
                     VALUES ('agent-fail', 'Failing Agent', 'Specialist', 'Core', 'Desc', 'idle', '[]', '[]', '[]', '{}')")
            .execute(&pool)
            .await?;

        let req = CreateJobRequest {
            agent_id: "agent-fail".to_string(),
            workflow_id: None,
            name: "Failing Job".to_string(),
            prompt: "Prompt".to_string(),
            cron_expr: "* * * * *".to_string(),
            budget_usd: Some(0.50),
            max_failures: None,
            metadata: None,
        };
        let job = create_job(&pool, req).await?;

        // Execute job - fails due to no model provider configured
        execute_job(Arc::clone(&state), job.clone()).await;

        // Verify run record has failed status and non-null mission_id
        let run =
            sqlx::query("SELECT status, mission_id FROM scheduled_job_runs WHERE job_id = ?1")
                .bind(&job.id)
                .fetch_one(&pool)
                .await?;
        use sqlx::Row;
        let status: String = run.get("status");
        let mission_id: Option<String> = run.get("mission_id");
        assert_eq!(status, "failed");
        assert!(mission_id.is_some());
        assert!(mission_id.unwrap().starts_with("continuity-"));

        // Verify agent is still released back to idle
        let agent_status: String = sqlx::query_scalar("SELECT status FROM agents WHERE id = ?1")
            .bind(&job.agent_id)
            .fetch_one(&pool)
            .await?;
        assert_eq!(agent_status, "idle");

        Ok(())
    }
}

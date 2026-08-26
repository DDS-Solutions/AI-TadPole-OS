//! @docs ARCHITECTURE:Continuity
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Agent Runner / executor
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Type-safe state handling and bounded execution without unhandled panics.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: `[Continuity]`
//! - **Witness Tests**: none declared

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
}

impl AgentClaimGuard {
    fn new(pool: sqlx::SqlitePool, agent_id: String) -> Self {
        Self {
            pool,
            agent_id,
            finalized: false,
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
            tokio::spawn(async move {
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
/// Runs every 60 seconds, looking for `scheduled_jobs` rows where `next_run_at <= now`.
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
    }
}

/// A single scheduler tick: fetch due jobs and execute them.
async fn tick(
    state: &Arc<AppState>,
    semaphore: &Arc<Semaphore>,
    concurrency_limit: usize,
) -> Result<(), AppError> {
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

        // 1. Skip if agent is busy - ATOMIC CLAIM
        match crate::agent::persistence::claim_agent(&state.resources.pool, &job.agent_id).await {
            Ok(true) => {}
            Ok(false) => {
                tracing::warn!(
                    "⏭ [Continuity] Skipping job '{}' — agent '{}' is currently busy.",
                    job.name,
                    job.agent_id
                );
                if let Ok(run) = create_job_run(&state.resources.pool, &job.id).await {
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

        // Establish RAII guard for the agent claim
        let mut claim_guard =
            AgentClaimGuard::new(state.resources.pool.clone(), job.agent_id.clone());

        // 2. Create run record
        let run = match create_job_run(&state.resources.pool, &job.id).await {
            Ok(r) => r,
            Err(e) => {
                tracing::error!(
                    "❌ [Continuity] Failed to create run record for job '{}': {}",
                    job.id,
                    e
                );
                return;
            }
        };

        // 3. Spawn keep-alive task to maintain agent heartbeat during execution
        let heartbeat_agent_id = job.agent_id.clone();
        let heartbeat_pool = state.resources.pool.clone();
        let (heartbeat_cancel_tx, mut heartbeat_cancel_rx) = tokio::sync::oneshot::channel::<()>();

        let heartbeat_task = tokio::spawn(async move {
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
                    _ = &mut heartbeat_cancel_rx => {
                        break;
                    }
                }
            }
        });

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
        let (result_output, resolved_mission_id, run_workflow_id) =
            if let Some(workflow_id) = &job.workflow_id {
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
                    Err(_) => (
                        Err(AppError::InternalServerError(
                            "Continuity agent mission timed out".to_string(),
                        )),
                        Some(atomic_mission_id.clone()),
                        None,
                    ),
                }
            };

        // Stop the heartbeat keep-alive task
        let _ = heartbeat_cancel_tx.send(());
        let _ = heartbeat_task.await;

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
        claim_guard.finalize();
        if let Err(e) =
            crate::agent::persistence::release_agent(&state.resources.pool, &job.agent_id).await
        {
            tracing::error!(
                "❌ [Continuity] Failed to release agent claim for job '{}': {}",
                job.name,
                e
            );
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

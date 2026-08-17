//! Background execution engine for scheduled agent missions and workflows.
//!
//! @docs ARCHITECTURE:Continuity
//!
//! ### AI Assist Note
//! **The Continuity Engine**: Drives the autonomous lifecycle of the swarm.
//! Polling every 60s, it awakens dormant agents to perform scheduled missions.
//!
//! ### Sovereign Skipping (Saturating Cron)
//! If the system experiences downtime, scheduled jobs will execute exactly once
//! upon restart rather than catching up on missed intervals. This prevents
//! resource spikes and recursive token-spend cascades.
//!
//! ### 🔍 Debugging & Observability
//! - **Trace Scope**: `server-rs::agent::continuity::executor`

use chrono::Utc;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Semaphore;

use super::scheduler::{
    complete_job_run, create_job_run, get_due_jobs, record_job_tick, recover_interrupted_jobs,
};
use super::types::JobRunStatus;
use crate::agent::runner::AgentRunner;
use crate::agent::types::TaskPayload;
use crate::error::AppError;
use crate::state::AppState;

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

    let semaphore = Arc::new(Semaphore::new(10));
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
            if let Err(e) = tick(&state, &semaphore).await {
                tracing::error!("❌ [Continuity] Scheduler tick error: {}", e);
            }
        }

        // Reaper: Clean up orphans every tick (60s)
        let _ = crate::agent::persistence::reap_stale_agents(&state.resources.pool, 300).await;

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
///
/// This function performs the following steps:
/// 1. Queries the database for all enabled jobs that are due for execution.
/// 2. Skips execution if no jobs are due.
/// 3. Spawns an asynchronous tokio task for each due job to prevent blocking the ticker,
///    respecting the concurrency limit via try_acquire_owned.
async fn tick(state: &Arc<AppState>, semaphore: &Arc<Semaphore>) -> Result<(), AppError> {
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
                    "⚠️ [Continuity] Scheduled job concurrency limit of 10 reached. Postponing remaining due jobs to next tick."
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
/// 3. Calls `AgentRunner::run()` with the job's prompt and budget.
/// 4. Finalises the run record and advances cron to next window.
pub async fn execute_job(state: Arc<AppState>, job: super::types::ScheduledJob) {
    let now = Utc::now();
    tracing::info!(
        "🚀 [Continuity] Starting job '{}' for agent '{}' (budget: ${:.3})",
        job.name,
        job.agent_id,
        job.budget_usd
    );

    // 1. Skip if agent is busy (prevents overlapping missions) - ATOMIC CLAIM
    let claimed = matches!(
        crate::agent::persistence::claim_agent(&state.resources.pool, &job.agent_id).await,
        Ok(true)
    );

    if !claimed {
        tracing::warn!(
            "⏭ [Continuity] Skipping job '{}' — agent '{}' is currently busy or claim failed.",
            job.name,
            job.agent_id
        );

        // Record skip
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
        // Don't advance cron — retry on next tick
        return;
    }

    // 2. Create run record
    let run = match create_job_run(&state.resources.pool, &job.id).await {
        Ok(r) => r,
        Err(e) => {
            tracing::error!(
                "❌ [Continuity] Failed to create run record for job '{}': {}",
                job.id,
                e
            );
            let _ = crate::agent::persistence::release_agent(&state.resources.pool, &job.agent_id)
                .await;
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

    // 4. Branch: Workflow vs Atomic Agent Mission
    let (result_output, run_workflow_id) = if let Some(workflow_id) = &job.workflow_id {
        tracing::info!(
            "🔄 [Continuity] Executing workflow '{}' (id: {})",
            job.name,
            workflow_id
        );
        let workflow_engine = super::workflow::WorkflowEngine::new(Arc::clone(&state));
        match workflow_engine
            .run_workflow("default", workflow_id, serde_json::json!({}))
            .await
        {
            Ok((out, wf_run_id)) => (Ok(out), Some(wf_run_id)),
            Err(e) => (Err(e), None),
        }
    } else {
        // Build payload and invoke AgentRunner for atomic agent mission
        let payload = TaskPayload {
            message: job.prompt.clone(),
            department: Some(format!("Continuity: {}", job.name)),
            budget_usd: Some(job.budget_usd),
            safe_mode: Some(false),
            analysis: Some(false),
            ..Default::default()
        };

        let runner = AgentRunner::new(Arc::clone(&state));
        match runner.run(job.agent_id.clone(), payload).await {
            Ok(out) => (Ok(out), None),
            Err(e) => (Err(e), None),
        }
    };

    // Stop the heartbeat keep-alive task
    let _ = heartbeat_cancel_tx.send(());
    let _ = heartbeat_task.await;

    // 4. Finalise run record and advance cron
    let (success, mission_id, cost_usd, output_summary, status) = match result_output {
        Ok(output) => {
            let summary = if output.chars().count() > 500 {
                format!("{}...", output.chars().take(500).collect::<String>())
            } else {
                output.clone()
            };

            // Query actual financial spend from mission history ledger
            let cost = if let Some(ref wf_id) = run_workflow_id {
                sqlx::query_scalar::<_, f64>(
                    "SELECT COALESCE(SUM(cost_usd), 0.0) FROM mission_history WHERE id LIKE ?1",
                )
                .bind(format!("{}%", wf_id))
                .fetch_one(&state.resources.pool)
                .await
                .unwrap_or(0.0)
            } else {
                sqlx::query_scalar::<_, f64>(
                    "SELECT COALESCE(cost_usd, 0.0) FROM mission_history WHERE agent_id = ?1 ORDER BY created_at DESC LIMIT 1",
                )
                .bind(&job.agent_id)
                .fetch_one(&state.resources.pool)
                .await
                .unwrap_or(0.0)
            };

            (
                true,
                run_workflow_id,
                cost,
                Some(summary),
                JobRunStatus::Completed,
            )
        }
        Err(e) => {
            tracing::error!("❌ [Continuity] Job '{}' failed: {}", job.name, e);
            let msg = format!("Error: {}", e);
            (false, None, 0.0, Some(msg), JobRunStatus::Failed)
        }
    };

    if let Err(e) = complete_job_run(
        &state.resources.pool,
        &run.id,
        mission_id.as_deref(),
        status,
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

    if let Err(e) = record_job_tick(
        &state.resources.pool,
        &job.id,
        &job.cron_expr,
        success,
        job.max_failures,
    )
    .await
    {
        tracing::error!(
            "❌ [Continuity] Failed to record job tick for job '{}': {}",
            job.name,
            e
        );
    }

    // 5. Release agent claim so subsequent ticks/missions can claim the agent
    if let Err(e) =
        crate::agent::persistence::release_agent(&state.resources.pool, &job.agent_id).await
    {
        tracing::error!(
            "❌ [Continuity] Failed to release agent claim for job '{}': {}",
            job.name,
            e
        );
    }

    // Emit WebSocket event
    state.emit_event(serde_json::json!({
        "type": "engine:scheduled_job_complete",
        "data": {
            "job_id": job.id,
            "job_name": job.name,
            "agent_id": job.agent_id,
            "success": success,
            "timestamp": now.to_rfc3339(),
        }
    }));

    tracing::info!(
        "✅ [Continuity] Job '{}' finished (success={})",
        job.name,
        success
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::continuity::scheduler::create_job;
    use crate::agent::continuity::types::CreateJobRequest;
    use crate::db::init_db;

    #[tokio::test]
    async fn test_execute_job_busy_skip() -> Result<(), Box<dyn std::error::Error>> {
        let pool = init_db("sqlite::memory:").await?;
        let state = Arc::new(AppState::with_pool(pool.clone()).await);

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

        // 1. Manually claim agent (make it busy)
        sqlx::query("INSERT INTO agents (id, name, role, department, description, status, skills, workflows, mcp_tools, metadata) \
                     VALUES (?1, 'Agent 1', 'Specialist', 'Core', 'Desc', 'busy', '[]', '[]', '[]', '{}')")
            .bind(&job.agent_id)
            .execute(&pool)
            .await?;

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

        // 1. Insert agent as idle
        sqlx::query("INSERT INTO agents (id, name, role, department, description, status, skills, workflows, mcp_tools, metadata) \
                     VALUES (?1, 'Agent 2', 'Specialist', 'Core', 'Desc', 'idle', '[]', '[]', '[]', '{}')")
            .bind(&job.agent_id)
            .execute(&pool)
            .await?;

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
}

// Metadata: [executor]

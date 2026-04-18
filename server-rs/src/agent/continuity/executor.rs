//! Background execution engine for scheduled agent missions and workflows.
//!
//! @docs ARCHITECTURE:Continuity
//!
//! ### AI Assist Note
//! **The Continuity Engine**: Drives the autonomous lifecycle of the swarm.
//! Polling every 60s, it awakens dormant agents to perform scheduled missions.
//!
//! ### 🔍 Debugging & Observability
//! - **Trace Scope**: `server-rs::agent::continuity::executor`

use chrono::Utc;
use std::sync::Arc;
use std::time::Duration;

use super::scheduler::{complete_job_run, create_job_run, get_due_jobs, record_job_tick};
use super::types::JobRunStatus;
use crate::agent::runner::AgentRunner;
use crate::agent::types::TaskPayload;
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

    let mut interval = tokio::time::interval(Duration::from_secs(60));
    interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

    loop {
        interval.tick().await;

        if let Err(e) = tick(&state).await {
            tracing::error!("❌ [Continuity] Scheduler tick error: {}", e);
        }

        // Reaper: Clean up orphans every tick (60s)
        let _ = crate::agent::persistence::reap_stale_agents(&state.resources.pool, 300).await;
    }
}

/// A single scheduler tick: fetch due jobs and execute them.
///
/// This function performs the following steps:
/// 1. Queries the database for all enabled jobs that are due for execution.
/// 2. Skips execution if no jobs are due.
/// 3. Spawns an asynchronous tokio task for each due job to prevent blocking the ticker.
async fn tick(state: &Arc<AppState>) -> anyhow::Result<()> {
    let due_jobs = get_due_jobs(&state.resources.pool).await?;

    if due_jobs.is_empty() {
        return Ok(());
    }

    tracing::info!(
        "🕐 [Continuity] {} job(s) due for execution.",
        due_jobs.len()
    );

    for job in due_jobs {
        let state_clone = Arc::clone(state);
        let job_clone = job.clone();

        tokio::spawn(async move {
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
async fn execute_job(state: Arc<AppState>, job: super::types::ScheduledJob) {
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
            return;
        }
    };

    // 3. Branch: Workflow vs Atomic Agent Mission
    let result = if let Some(workflow_id) = &job.workflow_id {
        tracing::info!(
            "🔄 [Continuity] Executing workflow '{}' (id: {})",
            job.name,
            workflow_id
        );
        let workflow_engine = super::workflow::WorkflowEngine::new(Arc::clone(&state));
        // Use an empty initial context for now, could be passed in metadata later
        workflow_engine
            .run_workflow(workflow_id, serde_json::json!({}))
            .await
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
        runner.run(job.agent_id.clone(), payload).await
    };

    // 4. Finalise run record and advance cron
    let (success, mission_id, cost_usd, output_summary, status) = match result {
        Ok(output) => {
            let summary = output.chars().take(500).collect::<String>();
            (
                true,
                None::<String>,
                0.0f64,
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

    let _ = complete_job_run(
        &state.resources.pool,
        &run.id,
        mission_id.as_deref(),
        status,
        cost_usd,
        output_summary.as_deref(),
    )
    .await;

    let _ = record_job_tick(
        &state.resources.pool,
        &job.id,
        &job.cron_expr,
        success,
        job.max_failures,
    )
    .await;

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

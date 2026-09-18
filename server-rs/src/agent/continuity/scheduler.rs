//! @docs ARCHITECTURE:Continuity
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Agent Runner / scheduler
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Type-safe state handling and bounded execution without unhandled panics.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: none declared
//! - **Witness Tests**: none declared

use crate::error::AppError;

use chrono::Utc;
use sqlx::SqlitePool;
use uuid::Uuid;

use super::types::{
    CreateJobRequest, JobRunStatus, ScheduledJob, ScheduledJobRun, UpdateJobRequest,
    DEFAULT_BUDGET_USD,
};

// ─────────────────────────────────────────────────────────
//  DATABASE HELPERS
// ─────────────────────────────────────────────────────────

/// Creates a new scheduled job.
pub async fn create_job(
    pool: &SqlitePool,
    req: CreateJobRequest,
) -> Result<ScheduledJob, AppError> {
    if req.name.chars().count() > 100 {
        return Err(AppError::BadRequest(
            "Job name cannot exceed 100 characters".to_string(),
        ));
    }
    if req.prompt.chars().count() > 50000 {
        return Err(AppError::BadRequest(
            "Job prompt cannot exceed 50000 characters".to_string(),
        ));
    }

    let budget = req.budget_usd.unwrap_or(DEFAULT_BUDGET_USD);
    if budget < 0.0 {
        return Err(AppError::BadRequest(
            "Budget must be non-negative".to_string(),
        ));
    }

    let max_failures = req.max_failures.unwrap_or(3);
    if max_failures < 1 {
        return Err(AppError::BadRequest(
            "max_failures must be at least 1".to_string(),
        ));
    }

    // Validate foreign entity existence
    let agent_exists: bool =
        sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM agents WHERE id = ?1)")
            .bind(&req.agent_id)
            .fetch_one(pool)
            .await
            .map_err(AppError::Sqlx)?;
    if !agent_exists {
        return Err(AppError::NotFound(format!(
            "Agent '{}' does not exist",
            req.agent_id
        )));
    }

    if let Some(ref wf_id) = req.workflow_id {
        let wf_exists: bool =
            sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM workflows WHERE id = ?1)")
                .bind(wf_id)
                .fetch_one(pool)
                .await
                .map_err(AppError::Sqlx)?;
        if !wf_exists {
            return Err(AppError::NotFound(format!(
                "Workflow '{}' does not exist",
                wf_id
            )));
        }
    }

    let job_id = Uuid::new_v4().to_string();
    let now = Utc::now();
    let metadata_json = req
        .metadata
        .as_ref()
        .and_then(|m| serde_json::to_string(m).ok());

    // Compute next_run_at from cron expression (evaluated in UTC)
    let next_run = compute_next_run(&req.cron_expr, &now)?;

    sqlx::query(
        "INSERT INTO scheduled_jobs \
         (id, agent_id, workflow_id, name, prompt, cron_expr, budget_usd, enabled, \
          next_run_at, consecutive_failures, max_failures, created_at, metadata) \
         VALUES (?1,?2,?3,?4,?5,?6,?7,1,?8,0,?9,?10,?11)",
    )
    .bind(&job_id)
    .bind(&req.agent_id)
    .bind(&req.workflow_id)
    .bind(&req.name)
    .bind(&req.prompt)
    .bind(&req.cron_expr)
    .bind(budget)
    .bind(next_run)
    .bind(max_failures)
    .bind(now)
    .bind(metadata_json)
    .execute(pool)
    .await?;

    get_job_by_id(pool, &job_id).await?.ok_or_else(|| {
        AppError::InternalServerError("Failed to retrieve newly created job".to_string())
    })
}

/// Updates fields on a scheduled job atomically within a single transaction.
pub async fn update_job(
    pool: &SqlitePool,
    job_id: &str,
    req: UpdateJobRequest,
) -> Result<ScheduledJob, AppError> {
    let now = Utc::now();

    // Check existence first
    let _existing = get_job_by_id(pool, job_id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Job '{}' not found", job_id)))?;

    // Validations
    if let Some(ref name) = req.name {
        if name.chars().count() > 100 {
            return Err(AppError::BadRequest(
                "Job name cannot exceed 100 characters".to_string(),
            ));
        }
    }
    if let Some(ref prompt) = req.prompt {
        if prompt.chars().count() > 50000 {
            return Err(AppError::BadRequest(
                "Job prompt cannot exceed 50000 characters".to_string(),
            ));
        }
    }
    if let Some(budget) = req.budget_usd {
        if budget < 0.0 {
            return Err(AppError::BadRequest(
                "Budget must be non-negative".to_string(),
            ));
        }
    }
    if let Some(max_f) = req.max_failures {
        if max_f < 1 {
            return Err(AppError::BadRequest(
                "max_failures must be at least 1".to_string(),
            ));
        }
    }
    if let Some(ref wf_id) = req.workflow_id {
        let wf_exists: bool =
            sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM workflows WHERE id = ?1)")
                .bind(wf_id)
                .fetch_one(pool)
                .await
                .unwrap_or(false);
        if !wf_exists {
            return Err(AppError::NotFound(format!(
                "Workflow '{}' does not exist",
                wf_id
            )));
        }
    }

    // Atomic transaction execution
    let mut tx = pool.begin().await?;

    if let Some(ref cron_expr) = req.cron_expr {
        let next_run = compute_next_run(cron_expr, &now)?;
        sqlx::query("UPDATE scheduled_jobs SET cron_expr = ?1, next_run_at = ?2 WHERE id = ?3")
            .bind(cron_expr)
            .bind(next_run)
            .bind(job_id)
            .execute(&mut *tx)
            .await?;
    }

    if let Some(ref name) = req.name {
        sqlx::query("UPDATE scheduled_jobs SET name = ?1 WHERE id = ?2")
            .bind(name)
            .bind(job_id)
            .execute(&mut *tx)
            .await?;
    }
    if let Some(ref prompt) = req.prompt {
        sqlx::query("UPDATE scheduled_jobs SET prompt = ?1 WHERE id = ?2")
            .bind(prompt)
            .bind(job_id)
            .execute(&mut *tx)
            .await?;
    }
    if let Some(budget) = req.budget_usd {
        sqlx::query("UPDATE scheduled_jobs SET budget_usd = ?1 WHERE id = ?2")
            .bind(budget)
            .bind(job_id)
            .execute(&mut *tx)
            .await?;
    }
    if let Some(enabled) = req.enabled {
        sqlx::query("UPDATE scheduled_jobs SET enabled = ?1 WHERE id = ?2")
            .bind(enabled as i64)
            .bind(job_id)
            .execute(&mut *tx)
            .await?;
    }
    if let Some(ref workflow_id) = req.workflow_id {
        sqlx::query("UPDATE scheduled_jobs SET workflow_id = ?1 WHERE id = ?2")
            .bind(workflow_id)
            .bind(job_id)
            .execute(&mut *tx)
            .await?;
    }
    if let Some(max_f) = req.max_failures {
        sqlx::query("UPDATE scheduled_jobs SET max_failures = ?1 WHERE id = ?2")
            .bind(max_f)
            .bind(job_id)
            .execute(&mut *tx)
            .await?;
    }
    if let Some(ref meta) = req.metadata {
        let meta_str = serde_json::to_string(meta).ok();
        sqlx::query("UPDATE scheduled_jobs SET metadata = ?1 WHERE id = ?2")
            .bind(meta_str)
            .bind(job_id)
            .execute(&mut *tx)
            .await?;
    }

    tx.commit().await?;

    get_job_by_id(pool, job_id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Job not found: {}", job_id)))
}

/// Deletes a scheduled job and all its historical run logs.
///
/// # Warning
/// This operation is destructive and cannot be undone. All audit records for
/// the job's runs will be deleted via the database's ON DELETE CASCADE constraint.
pub async fn delete_job(pool: &SqlitePool, job_id: &str) -> Result<(), AppError> {
    let rows = sqlx::query("DELETE FROM scheduled_jobs WHERE id = ?1")
        .bind(job_id)
        .execute(pool)
        .await?
        .rows_affected();

    if rows == 0 {
        return Err(AppError::NotFound(format!("Job '{}' not found", job_id)));
    }
    Ok(())
}

pub async fn get_job_by_id(
    pool: &SqlitePool,
    job_id: &str,
) -> Result<Option<ScheduledJob>, AppError> {
    let row = sqlx::query(
        "SELECT id, agent_id, workflow_id, name, prompt, cron_expr, budget_usd, enabled, \
                last_run_at, next_run_at, consecutive_failures, max_failures, created_at, metadata \
         FROM scheduled_jobs WHERE id = ?1",
    )
    .bind(job_id)
    .fetch_optional(pool)
    .await?;
    Ok(row.map(|r| row_to_job(&r)))
}

pub async fn list_jobs(pool: &SqlitePool) -> Result<Vec<ScheduledJob>, AppError> {
    let rows = sqlx::query(
        "SELECT id, agent_id, workflow_id, name, prompt, cron_expr, budget_usd, enabled, \
                last_run_at, next_run_at, consecutive_failures, max_failures, created_at, metadata \
         FROM scheduled_jobs ORDER BY created_at DESC",
    )
    .fetch_all(pool)
    .await?;
    Ok(rows.iter().map(row_to_job).collect())
}

pub async fn list_runs_for_job(
    pool: &SqlitePool,
    job_id: &str,
    limit: i64,
) -> Result<Vec<ScheduledJobRun>, AppError> {
    let clamped_limit = limit.clamp(1, 1000);
    let rows = sqlx::query(
        "SELECT id, job_id, mission_id, started_at, completed_at, status, cost_usd, output_summary \
         FROM scheduled_job_runs WHERE job_id = ?1 ORDER BY started_at DESC LIMIT ?2",
    )
    .bind(job_id)
    .bind(clamped_limit)
    .fetch_all(pool)
    .await?;
    Ok(rows.iter().map(row_to_run).collect())
}

/// Fetches all enabled jobs whose next_run_at <= now, ordered by next_run_at ASC for scheduling fairness.
pub async fn get_due_jobs(pool: &SqlitePool) -> Result<Vec<ScheduledJob>, AppError> {
    let now = Utc::now();
    let rows = sqlx::query(
        "SELECT id, agent_id, workflow_id, name, prompt, cron_expr, budget_usd, enabled, \
                last_run_at, next_run_at, consecutive_failures, max_failures, created_at, metadata \
         FROM scheduled_jobs WHERE enabled = 1 AND next_run_at <= ?1 \
         ORDER BY next_run_at ASC",
    )
    .bind(now)
    .fetch_all(pool)
    .await?;
    Ok(rows.iter().map(row_to_job).collect())
}

/// Queries cumulative financial spend from mission_history ledger for a given mission ID prefix.
pub async fn get_job_financial_spend(pool: &SqlitePool, mission_id_prefix: &str) -> f64 {
    match sqlx::query_scalar::<_, f64>(
        "SELECT COALESCE(SUM(cost_usd), 0.0) FROM mission_history WHERE id LIKE ?1",
    )
    .bind(format!("{}%", mission_id_prefix))
    .fetch_one(pool)
    .await
    {
        Ok(c) => c,
        Err(e) => {
            tracing::warn!(
                "⚠️ [Continuity] Failed to fetch financial spend for '{}': {}",
                mission_id_prefix,
                e
            );
            0.0
        }
    }
}

/// Records a successful or failed "tick" of a job execution.
///
/// Note: SQLite evaluates both expressions against the pre-update row, ensuring
/// consecutive_failures increment and threshold check are executed atomically without race conditions.
pub async fn record_job_tick(
    pool: &SqlitePool,
    job_id: &str,
    cron_expr: &str,
    success: bool,
    max_failures: i64,
) -> Result<(), AppError> {
    let now = Utc::now();
    let next_run = compute_next_run(cron_expr, &now)?;

    if success {
        sqlx::query(
            "UPDATE scheduled_jobs SET last_run_at=?1, next_run_at=?2, consecutive_failures=0 WHERE id=?3")
            .bind(now).bind(next_run).bind(job_id)
            .execute(pool).await?;
    } else {
        // Increment failure counter; if >= max_failures, auto-disable
        sqlx::query(
            "UPDATE scheduled_jobs \
             SET last_run_at=?1, next_run_at=?2, \
                 consecutive_failures=consecutive_failures+1, \
                 enabled = CASE WHEN consecutive_failures+1 >= ?3 THEN 0 ELSE enabled END \
             WHERE id=?4",
        )
        .bind(now)
        .bind(next_run)
        .bind(max_failures)
        .bind(job_id)
        .execute(pool)
        .await?;

        // Log if auto-disabled
        let job = get_job_by_id(pool, job_id).await?;
        if let Some(j) = job {
            if !j.enabled {
                tracing::warn!(
                    "🚫 [Continuity] Job '{}' (agent: {}) auto-disabled after {} consecutive failures.",
                    j.name, j.agent_id, j.consecutive_failures
                );
            }
        }
    }
    Ok(())
}

/// Creates a run record for a job.
pub async fn create_job_run(pool: &SqlitePool, job_id: &str) -> Result<ScheduledJobRun, AppError> {
    let run_id = Uuid::new_v4().to_string();
    let now = Utc::now();

    sqlx::query(
        "INSERT INTO scheduled_job_runs (id, job_id, started_at, status, cost_usd) \
         VALUES (?1, ?2, ?3, 'running', 0.0)",
    )
    .bind(&run_id)
    .bind(job_id)
    .bind(now)
    .execute(pool)
    .await?;

    Ok(ScheduledJobRun {
        id: run_id,
        job_id: job_id.to_string(),
        mission_id: None,
        started_at: now,
        completed_at: None,
        status: JobRunStatus::Running,
        cost_usd: 0.0,
        output_summary: None,
    })
}

/// Finalises a run record conditionally on status='running'.
pub async fn complete_job_run(
    pool: &SqlitePool,
    run_id: &str,
    mission_id: Option<&str>,
    status: JobRunStatus,
    cost_usd: f64,
    output_summary: Option<&str>,
) -> Result<(), AppError> {
    let now = Utc::now();
    let rows = sqlx::query(
        "UPDATE scheduled_job_runs \
         SET completed_at=?1, mission_id=?2, status=?3, cost_usd=?4, output_summary=?5 \
         WHERE id=?6 AND status='running'",
    )
    .bind(now)
    .bind(mission_id)
    .bind(status.as_str())
    .bind(cost_usd)
    .bind(output_summary)
    .bind(run_id)
    .execute(pool)
    .await?
    .rows_affected();

    if rows == 0 {
        tracing::warn!(
            "⚠️ [Continuity] Run '{}' was already finalized or not in running state",
            run_id
        );
    }

    Ok(())
}

/// Finds any `running` job and workflow runs in the database from prior startup sessions and marks them as failed.
/// Also releases any lingering agent claims back to `idle`.
pub async fn recover_interrupted_jobs(pool: &SqlitePool) -> Result<(), AppError> {
    let now = Utc::now();
    let mut tx = pool.begin().await?;

    // Recover scheduled job runs
    let jobs_recovered = sqlx::query(
        "UPDATE scheduled_job_runs \
         SET completed_at = ?1, status = 'failed', output_summary = 'Interrupted by system restart' \
         WHERE status = 'running'",
    )
    .bind(now)
    .execute(&mut *tx)
    .await?
    .rows_affected();

    // Recover workflow runs
    let workflows_recovered = sqlx::query(
        "UPDATE workflow_runs \
         SET completed_at = ?1, status = 'failed' \
         WHERE status = 'running'",
    )
    .bind(now)
    .execute(&mut *tx)
    .await?
    .rows_affected();

    // Recover workflow step runs
    let steps_recovered = sqlx::query(
        "UPDATE workflow_step_runs \
         SET completed_at = ?1, status = 'failed', output_text = 'Interrupted by system restart' \
         WHERE status = 'running'",
    )
    .bind(now)
    .execute(&mut *tx)
    .await?
    .rows_affected();

    // Reconcile stuck agents on restart
    let agents_released = sqlx::query("UPDATE agents SET status = 'idle' WHERE status = 'busy'")
        .execute(&mut *tx)
        .await?
        .rows_affected();

    tx.commit().await?;

    if jobs_recovered > 0 || workflows_recovered > 0 || steps_recovered > 0 || agents_released > 0 {
        tracing::warn!(
            "⚠️ [Continuity] Startup recovery reconciled: {} job run(s), {} workflow run(s), {} step run(s), {} agent claim(s).",
            jobs_recovered,
            workflows_recovered,
            steps_recovered,
            agents_released
        );
    }

    Ok(())
}

/// Advances the given `now` by the cron period to compute `next_run_at` in UTC.
/// Uses the `cron` crate for full standard-compliant support.
pub fn compute_next_run(
    cron_expr: &str,
    from: &chrono::DateTime<Utc>,
) -> Result<chrono::DateTime<Utc>, AppError> {
    use cron::Schedule;
    use std::str::FromStr;

    // Support standard shorthand aliases
    let expr = match cron_expr.trim() {
        "@hourly" => "0 0 * * * *",
        "@daily" => "0 0 0 * * *",
        "@weekly" => "0 0 0 * * 0",
        "@monthly" => "0 0 0 1 * *",
        shorthand if shorthand.starts_with('@') => {
            return Err(AppError::BadRequest(format!(
                "Unsupported shorthand: {}",
                shorthand
            )));
        }
        _ => cron_expr,
    };

    // Standard 5-field cron to 6-field (prepend 0 seconds)
    let expr = if expr.split_whitespace().count() == 5 {
        format!("0 {}", expr)
    } else {
        expr.to_string()
    };

    let schedule = Schedule::from_str(&expr)
        .map_err(|e| AppError::BadRequest(format!("Invalid cron expression '{}': {}", expr, e)))?;

    let next = schedule.after(from).next().ok_or_else(|| {
        AppError::BadRequest(format!(
            "No future run time found for cron expression '{}'",
            expr
        ))
    })?;

    Ok(next)
}

// ─────────────────────────────────────────────────────────
//  ROW MAPPERS
// ─────────────────────────────────────────────────────────

fn row_to_job(row: &sqlx::sqlite::SqliteRow) -> ScheduledJob {
    use sqlx::Row;
    let enabled: i64 = row.get("enabled");
    let budget_usd: f64 = row.try_get("budget_usd").unwrap_or(DEFAULT_BUDGET_USD);
    let metadata_str: Option<String> = row.get("metadata");
    let metadata = metadata_str.and_then(|s| match serde_json::from_str(&s) {
        Ok(v) => Some(v),
        Err(e) => {
            tracing::warn!(
                "⚠️ [Continuity] Failed to deserialize job metadata JSON: {}",
                e
            );
            None
        }
    });

    ScheduledJob {
        id: row.get("id"),
        agent_id: row.get("agent_id"),
        workflow_id: row.get("workflow_id"),
        name: row.get("name"),
        prompt: row.get("prompt"),
        cron_expr: row.get("cron_expr"),
        budget_usd,
        enabled: enabled != 0,
        last_run_at: row.get("last_run_at"),
        next_run_at: row.get("next_run_at"),
        consecutive_failures: row.get("consecutive_failures"),
        max_failures: row.get("max_failures"),
        created_at: row.get("created_at"),
        metadata,
    }
}

fn row_to_run(row: &sqlx::sqlite::SqliteRow) -> ScheduledJobRun {
    use sqlx::Row;
    let status_str: String = row.get("status");
    let cost_usd: f64 = row.try_get("cost_usd").unwrap_or(0.0);

    ScheduledJobRun {
        id: row.get("id"),
        job_id: row.get("job_id"),
        mission_id: row.get("mission_id"),
        started_at: row.get("started_at"),
        completed_at: row.get("completed_at"),
        status: JobRunStatus::from_str_lossless(&status_str),
        cost_usd,
        output_summary: row.get("output_summary"),
    }
}

// ─────────────────────────────────────────────────────────
//  TESTS
// ─────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Timelike, Utc};

    #[test]
    fn test_compute_next_run_daily_cron() {
        let from = Utc::now();
        let result = compute_next_run("0 9 * * *", &from);
        assert!(
            result.is_ok(),
            "Should parse daily cron: {:?}",
            result.err()
        );
        let next = result.unwrap();
        assert!(next > from, "next_run should be in the future");
    }

    #[test]
    fn test_compute_next_run_shorthand_hourly() {
        use chrono::TimeZone;
        let from = Utc.with_ymd_and_hms(2024, 1, 1, 10, 30, 0).unwrap();
        let next = compute_next_run("@hourly", &from).unwrap();
        assert_eq!(next.hour(), 11);
        assert_eq!(next.minute(), 0);
    }

    #[test]
    fn test_compute_next_run_invalid_cron() {
        let from = Utc::now();
        let result = compute_next_run("not a cron", &from);
        assert!(result.is_err(), "Invalid cron should return Err");
    }

    #[tokio::test]
    async fn test_db_job_lifecycle() {
        use crate::db::init_db;
        let pool = init_db("sqlite::memory:").await.unwrap();

        // Ensure agent exists
        sqlx::query("INSERT INTO agents (id, name, role, department, description, status, skills, workflows, mcp_tools, metadata) \
                     VALUES ('agent-1', 'Agent 1', 'Specialist', 'Core', 'Desc', 'idle', '[]', '[]', '[]', '{}')")
            .execute(&pool)
            .await
            .unwrap();

        let req = CreateJobRequest {
            agent_id: "agent-1".to_string(),
            workflow_id: None,
            name: "Test Job".to_string(),
            prompt: "Say hello".to_string(),
            cron_expr: "* * * * *".to_string(),
            budget_usd: Some(0.5),
            max_failures: Some(3),
            metadata: None,
        };

        // 1. Create
        let job = create_job(&pool, req).await.expect("Failed to create job");
        assert_eq!(job.name, "Test Job");
        assert!(job.enabled);

        // 2. Fetch due - Manually set next_run_at to the past to ensure it's due
        let past = Utc::now() - chrono::Duration::minutes(1);
        sqlx::query("UPDATE scheduled_jobs SET next_run_at = ?1 WHERE id = ?2")
            .bind(past)
            .bind(&job.id)
            .execute(&pool)
            .await
            .unwrap();

        let due = get_due_jobs(&pool).await.unwrap();
        assert_eq!(due.len(), 1);
        assert_eq!(due[0].id, job.id);

        // 3. Record success tick
        record_job_tick(&pool, &job.id, &job.cron_expr, true, 3)
            .await
            .unwrap();
        let updated = get_job_by_id(&pool, &job.id).await.unwrap().unwrap();
        assert_eq!(updated.consecutive_failures, 0);
        assert!(updated.last_run_at.is_some());

        // 4. Record failure until auto-disabled
        record_job_tick(&pool, &job.id, &job.cron_expr, false, 2)
            .await
            .unwrap();
        let updated = get_job_by_id(&pool, &job.id).await.unwrap().unwrap();
        assert_eq!(updated.consecutive_failures, 1);
        assert!(updated.enabled);

        record_job_tick(&pool, &job.id, &job.cron_expr, false, 2)
            .await
            .unwrap();
        let updated = get_job_by_id(&pool, &job.id).await.unwrap().unwrap();
        assert_eq!(updated.consecutive_failures, 2);
        assert!(
            !updated.enabled,
            "Job should be auto-disabled after 2 failures"
        );
    }

    #[tokio::test]
    async fn test_create_job_run_lifecycle() {
        use crate::db::init_db;
        let pool = init_db("sqlite::memory:").await.unwrap();

        // Ensure agent exists
        sqlx::query("INSERT INTO agents (id, name, role, department, description, status, skills, workflows, mcp_tools, metadata) \
                     VALUES ('agent-1', 'Agent 1', 'Specialist', 'Core', 'Desc', 'idle', '[]', '[]', '[]', '{}')")
            .execute(&pool)
            .await
            .unwrap();

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
        let job = create_job(&pool, req).await.unwrap();

        let run = create_job_run(&pool, &job.id)
            .await
            .expect("Failed to create run");
        assert_eq!(run.status, JobRunStatus::Running);

        complete_job_run(
            &pool,
            &run.id,
            Some("mission-123"),
            JobRunStatus::Completed,
            0.05,
            Some("Done"),
        )
        .await
        .unwrap();

        let runs = list_runs_for_job(&pool, &job.id, 10).await.unwrap();
        assert_eq!(runs.len(), 1);
        assert_eq!(runs[0].status, JobRunStatus::Completed);
        assert_eq!(runs[0].mission_id, Some("mission-123".to_string()));
    }

    #[tokio::test]
    async fn test_recover_interrupted_jobs_lifecycle() -> Result<(), Box<dyn std::error::Error>> {
        use crate::db::init_db;
        let pool = init_db("sqlite::memory:").await?;

        // Ensure agent exists
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

        // 2. Create a running run
        let run = create_job_run(&pool, &job.id).await?;
        assert_eq!(run.status, JobRunStatus::Running);

        // 3. Trigger recovery
        recover_interrupted_jobs(&pool).await?;

        // 4. Verify run was recovered and marked as failed
        let runs = list_runs_for_job(&pool, &job.id, 10).await?;
        assert_eq!(runs.len(), 1);
        assert_eq!(runs[0].status, JobRunStatus::Failed);
        assert_eq!(
            runs[0].output_summary,
            Some("Interrupted by system restart".to_string())
        );
        assert!(runs[0].completed_at.is_some());

        // 5. Verify agent was released from busy to idle
        let agent_status: String =
            sqlx::query_scalar("SELECT status FROM agents WHERE id = 'agent-1'")
                .fetch_one(&pool)
                .await?;
        assert_eq!(agent_status, "idle");

        Ok(())
    }

    #[tokio::test]
    async fn test_update_job_atomic_transaction() -> Result<(), Box<dyn std::error::Error>> {
        use crate::db::init_db;
        let pool = init_db("sqlite::memory:").await?;

        sqlx::query("INSERT INTO agents (id, name, role, department, description, status, skills, workflows, mcp_tools, metadata) \
                     VALUES ('agent-1', 'Agent 1', 'Specialist', 'Core', 'Desc', 'idle', '[]', '[]', '[]', '{}')")
            .execute(&pool)
            .await?;

        let req = CreateJobRequest {
            agent_id: "agent-1".to_string(),
            workflow_id: None,
            name: "Original Name".to_string(),
            prompt: "Original Prompt".to_string(),
            cron_expr: "0 0 * * *".to_string(),
            budget_usd: Some(1.0),
            max_failures: Some(3),
            metadata: None,
        };
        let job = create_job(&pool, req).await?;

        // Updating with invalid cron fails and rolls back completely
        let bad_update = UpdateJobRequest {
            name: Some("Corrupted Name".to_string()),
            prompt: None,
            workflow_id: None,
            cron_expr: Some("invalid cron string".to_string()),
            budget_usd: None,
            enabled: None,
            max_failures: None,
            metadata: None,
        };
        let res = update_job(&pool, &job.id, bad_update).await;
        assert!(res.is_err());

        // Name was NOT modified
        let current = get_job_by_id(&pool, &job.id).await?.unwrap();
        assert_eq!(current.name, "Original Name");

        // Valid update succeeds
        let good_update = UpdateJobRequest {
            name: Some("New Name".to_string()),
            prompt: Some("New Prompt".to_string()),
            workflow_id: None,
            cron_expr: Some("@hourly".to_string()),
            budget_usd: Some(2.5),
            enabled: Some(true),
            max_failures: Some(5),
            metadata: None,
        };
        let updated = update_job(&pool, &job.id, good_update).await?;
        assert_eq!(updated.name, "New Name");
        assert_eq!(updated.budget_usd, 2.5);
        assert_eq!(updated.cron_expr, "@hourly");
        assert_eq!(updated.max_failures, 5);

        Ok(())
    }

    #[tokio::test]
    async fn test_create_job_validates_entities_and_bounds(
    ) -> Result<(), Box<dyn std::error::Error>> {
        use crate::db::init_db;
        let pool = init_db("sqlite::memory:").await?;

        // Non-existent agent rejected
        let bad_agent_req = CreateJobRequest {
            agent_id: "ghost-agent".to_string(),
            workflow_id: None,
            name: "Job".to_string(),
            prompt: "Prompt".to_string(),
            cron_expr: "* * * * *".to_string(),
            budget_usd: None,
            max_failures: None,
            metadata: None,
        };
        assert!(create_job(&pool, bad_agent_req).await.is_err());

        // Negative budget rejected
        sqlx::query("INSERT INTO agents (id, name, role, department, description, status, skills, workflows, mcp_tools, metadata) \
                     VALUES ('agent-1', 'Agent 1', 'Specialist', 'Core', 'Desc', 'idle', '[]', '[]', '[]', '{}')")
            .execute(&pool)
            .await?;

        let neg_budget_req = CreateJobRequest {
            agent_id: "agent-1".to_string(),
            workflow_id: None,
            name: "Job".to_string(),
            prompt: "Prompt".to_string(),
            cron_expr: "* * * * *".to_string(),
            budget_usd: Some(-1.0),
            max_failures: None,
            metadata: None,
        };
        assert!(create_job(&pool, neg_budget_req).await.is_err());

        // Zero max_failures rejected
        let zero_failures_req = CreateJobRequest {
            agent_id: "agent-1".to_string(),
            workflow_id: None,
            name: "Job".to_string(),
            prompt: "Prompt".to_string(),
            cron_expr: "* * * * *".to_string(),
            budget_usd: None,
            max_failures: Some(0),
            metadata: None,
        };
        assert!(create_job(&pool, zero_failures_req).await.is_err());

        Ok(())
    }

    #[tokio::test]
    async fn test_delete_job_not_found() -> Result<(), Box<dyn std::error::Error>> {
        use crate::db::init_db;
        let pool = init_db("sqlite::memory:").await?;

        let res = delete_job(&pool, "non-existent-job").await;
        assert!(res.is_err());
        assert!(matches!(res.unwrap_err(), AppError::NotFound(_)));
        Ok(())
    }
}

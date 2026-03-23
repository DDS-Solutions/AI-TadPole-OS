//! Persistent scheduling logic and database orchestration for periodic missions.
//!
//! Handles the CRUD operations for `ScheduledJob` records and provides utilities 
//! for cron-to-timestamp calculations using standard cron expressions.

use anyhow::Result;
use chrono::Utc;
use sqlx::SqlitePool;
use uuid::Uuid;

use super::types::{
    CreateJobRequest, JobRunStatus, ScheduledJob, ScheduledJobRun, UpdateJobRequest,
};

// ─────────────────────────────────────────────────────────
//  DATABASE HELPERS
// ─────────────────────────────────────────────────────────

/// Creates a new scheduled job.
pub async fn create_job(pool: &SqlitePool, req: CreateJobRequest) -> Result<ScheduledJob> {
    let job_id = Uuid::new_v4().to_string();
    let now = Utc::now();
    let budget = req.budget_usd.unwrap_or(0.10);
    let max_failures = req.max_failures.unwrap_or(3);
    let metadata_json = req
        .metadata
        .as_ref()
        .and_then(|m| serde_json::to_string(m).ok());

    // Compute next_run_at from cron expression
    // Validates the cron string and calculates the first execution time relative to now.
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

    get_job_by_id(pool, &job_id)
        .await?
        .ok_or_else(|| anyhow::anyhow!("Failed to retrieve newly created job"))
}

/// Updates fields on a scheduled job.
pub async fn update_job(
    pool: &SqlitePool,
    job_id: &str,
    req: UpdateJobRequest,
) -> Result<ScheduledJob> {
    let now = Utc::now();

    if let Some(ref cron_expr) = req.cron_expr {
        let next_run = compute_next_run(cron_expr, &now)?;
        sqlx::query("UPDATE scheduled_jobs SET next_run_at = ?1 WHERE id = ?2")
            .bind(next_run)
            .bind(job_id)
            .execute(pool)
            .await?;
    }

    if let Some(name) = &req.name {
        sqlx::query("UPDATE scheduled_jobs SET name = ?1 WHERE id = ?2")
            .bind(name)
            .bind(job_id)
            .execute(pool)
            .await?;
    }
    if let Some(prompt) = &req.prompt {
        sqlx::query("UPDATE scheduled_jobs SET prompt = ?1 WHERE id = ?2")
            .bind(prompt)
            .bind(job_id)
            .execute(pool)
            .await?;
    }
    if let Some(cron_expr) = &req.cron_expr {
        sqlx::query("UPDATE scheduled_jobs SET cron_expr = ?1 WHERE id = ?2")
            .bind(cron_expr)
            .bind(job_id)
            .execute(pool)
            .await?;
    }
    if let Some(budget) = req.budget_usd {
        sqlx::query("UPDATE scheduled_jobs SET budget_usd = ?1 WHERE id = ?2")
            .bind(budget)
            .bind(job_id)
            .execute(pool)
            .await?;
    }
    if let Some(enabled) = req.enabled {
        sqlx::query("UPDATE scheduled_jobs SET enabled = ?1 WHERE id = ?2")
            .bind(enabled as i64)
            .bind(job_id)
            .execute(pool)
            .await?;
    }
    if let Some(workflow_id) = &req.workflow_id {
        sqlx::query("UPDATE scheduled_jobs SET workflow_id = ?1 WHERE id = ?2")
            .bind(workflow_id)
            .bind(job_id)
            .execute(pool)
            .await?;
    }
    if let Some(max_f) = req.max_failures {
        sqlx::query("UPDATE scheduled_jobs SET max_failures = ?1 WHERE id = ?2")
            .bind(max_f)
            .bind(job_id)
            .execute(pool)
            .await?;
    }

    get_job_by_id(pool, job_id)
        .await?
        .ok_or_else(|| anyhow::anyhow!("Job not found: {}", job_id))
}

/// Deletes a scheduled job and all its historical run logs.
/// 
/// # Warning
/// This operation is destructive and cannot be undone. All audit records for 
/// the job's runs will be deleted via the database's ON DELETE CASCADE constraint.
pub async fn delete_job(pool: &SqlitePool, job_id: &str) -> Result<()> {
    sqlx::query("DELETE FROM scheduled_jobs WHERE id = ?1")
        .bind(job_id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn get_job_by_id(pool: &SqlitePool, job_id: &str) -> Result<Option<ScheduledJob>> {
    let row = sqlx::query("SELECT * FROM scheduled_jobs WHERE id = ?1")
        .bind(job_id)
        .fetch_optional(pool)
        .await?;
    Ok(row.map(|r| row_to_job(&r)))
}

pub async fn list_jobs(pool: &SqlitePool) -> Result<Vec<ScheduledJob>> {
    let rows = sqlx::query("SELECT * FROM scheduled_jobs ORDER BY created_at DESC")
        .fetch_all(pool)
        .await?;
    Ok(rows.iter().map(row_to_job).collect())
}

pub async fn list_runs_for_job(
    pool: &SqlitePool,
    job_id: &str,
    limit: i64,
) -> Result<Vec<ScheduledJobRun>> {
    let rows = sqlx::query(
        "SELECT * FROM scheduled_job_runs WHERE job_id = ?1 ORDER BY started_at DESC LIMIT ?2",
    )
    .bind(job_id)
    .bind(limit)
    .fetch_all(pool)
    .await?;
    Ok(rows.iter().map(row_to_run).collect())
}

/// Fetches all enabled jobs whose next_run_at <= now. Called by the scheduler loop.
pub async fn get_due_jobs(pool: &SqlitePool) -> Result<Vec<ScheduledJob>> {
    let now = Utc::now();
    let rows = sqlx::query("SELECT * FROM scheduled_jobs WHERE enabled = 1 AND next_run_at <= ?1")
        .bind(now)
        .fetch_all(pool)
        .await?;
    Ok(rows.iter().map(row_to_job).collect())
}

/// Records a successful or failed "tick" of a job execution.
///
/// This maintains the scheduling continuity by calculating the next `next_run_at`
/// and handles the "auto-disable" lifecycle if `max_failures` is reached.
pub async fn record_job_tick(
    pool: &SqlitePool,
    job_id: &str,
    cron_expr: &str,
    success: bool,
    max_failures: i64,
) -> Result<()> {
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
pub async fn create_job_run(pool: &SqlitePool, job_id: &str) -> Result<ScheduledJobRun> {
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

/// Finalises a run record.
pub async fn complete_job_run(
    pool: &SqlitePool,
    run_id: &str,
    mission_id: Option<&str>,
    status: JobRunStatus,
    cost_usd: f64,
    output_summary: Option<&str>,
) -> Result<()> {
    let now = Utc::now();
    sqlx::query(
        "UPDATE scheduled_job_runs \
         SET completed_at=?1, mission_id=?2, status=?3, cost_usd=?4, output_summary=?5 \
         WHERE id=?6",
    )
    .bind(now)
    .bind(mission_id)
    .bind(status.as_str())
    .bind(cost_usd)
    .bind(output_summary)
    .bind(run_id)
    .execute(pool)
    .await?;
    Ok(())
}

/// Advances the given `now` by the cron period to compute `next_run_at`.
/// Uses the `cron` crate for full standard-compliant support.
pub fn compute_next_run(
    cron_expr: &str,
    from: &chrono::DateTime<Utc>,
) -> Result<chrono::DateTime<Utc>> {
    use cron::Schedule;
    use std::str::FromStr;

    // Support shorthand aliases
    let expr = match cron_expr.trim() {
        "@hourly" => "0 0 * * * *",
        "@daily" => "0 0 0 * * *",
        "@weekly" => "0 0 0 * * 0",
        "@monthly" => "0 0 0 1 * *",
        shorthand if shorthand.starts_with('@') => {
            return Err(anyhow::anyhow!("Unsupported shorthand: {}", shorthand));
        }
        _ => cron_expr,
    };

    // Standard 5-field cron to 6-field (prepend 0 seconds)
    let expr = if expr.split_whitespace().count() == 5 {
        format!("0 {}", expr)
    } else {
        expr.to_string()
    };

    let schedule = Schedule::from_str(&expr).map_err(|e| {
        anyhow::anyhow!("Invalid cron expression '{}': {}", expr, e)
    })?;

    let next = schedule.after(from).next().ok_or_else(|| {
        anyhow::anyhow!("No future run time found for cron expression '{}'", expr)
    })?;

    Ok(next)
}

// ─────────────────────────────────────────────────────────
//  ROW MAPPERS
// ─────────────────────────────────────────────────────────

fn row_to_job(row: &sqlx::sqlite::SqliteRow) -> ScheduledJob {
    use sqlx::Row;
    let enabled: i64 = row.get("enabled");
    let metadata_str: Option<String> = row.get("metadata");
    ScheduledJob {
        id: row.get("id"),
        agent_id: row.get("agent_id"),
        workflow_id: row.get("workflow_id"),
        name: row.get("name"),
        prompt: row.get("prompt"),
        cron_expr: row.get("cron_expr"),
        budget_usd: row.get("budget_usd"),
        enabled: enabled != 0,
        last_run_at: row.get("last_run_at"),
        next_run_at: row.get("next_run_at"),
        consecutive_failures: row.get("consecutive_failures"),
        max_failures: row.get("max_failures"),
        created_at: row.get("created_at"),
        metadata: metadata_str.and_then(|s| serde_json::from_str(&s).ok()),
    }
}

fn row_to_run(row: &sqlx::sqlite::SqliteRow) -> ScheduledJobRun {
    use sqlx::Row;
    let status_str: String = row.get("status");
    ScheduledJobRun {
        id: row.get("id"),
        job_id: row.get("job_id"),
        mission_id: row.get("mission_id"),
        started_at: row.get("started_at"),
        completed_at: row.get("completed_at"),
        status: match status_str.as_str() {
            "running" => JobRunStatus::Running,
            "completed" => JobRunStatus::Completed,
            "failed" => JobRunStatus::Failed,
            "budget_exceeded" => JobRunStatus::BudgetExceeded,
            _ => JobRunStatus::Skipped,
        },
        cost_usd: row.get("cost_usd"),
        output_summary: row.get("output_summary"),
    }
}

// ─────────────────────────────────────────────────────────
//  TESTS
// ─────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Utc, Timelike};

    #[test]
    fn test_compute_next_run_daily_cron() {
        let from = Utc::now();
        let result = compute_next_run("0 9 * * *", &from);
        assert!(result.is_ok(), "Should parse daily cron: {:?}", result.err());
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
}


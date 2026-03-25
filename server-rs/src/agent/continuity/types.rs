//! Data models for the continuity and workflow scheduling system.
//!
//! Defines the shared structures used for defining `ScheduledJob` schedules
//! and tracking the status of `ScheduledJobRun` execution histories.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// A scheduled autonomous mission job.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduledJob {
    pub id: String,
    pub agent_id: String,
    pub workflow_id: Option<String>,
    pub name: String,
    pub prompt: String,
    pub cron_expr: String,
    pub budget_usd: f64,
    pub enabled: bool,
    pub last_run_at: Option<DateTime<Utc>>,
    pub next_run_at: DateTime<Utc>,
    pub consecutive_failures: i64,
    pub max_failures: i64,
    pub created_at: DateTime<Utc>,
    pub metadata: Option<serde_json::Value>,
}

/// A single execution record for a `ScheduledJob`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduledJobRun {
    pub id: String,
    pub job_id: String,
    pub mission_id: Option<String>,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub status: JobRunStatus,
    pub cost_usd: f64,
    pub output_summary: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum JobRunStatus {
    Running,
    Completed,
    Failed,
    BudgetExceeded,
    Skipped,
}

impl JobRunStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            JobRunStatus::Running => "running",
            JobRunStatus::Completed => "completed",
            JobRunStatus::Failed => "failed",
            JobRunStatus::BudgetExceeded => "budget_exceeded",
            JobRunStatus::Skipped => "skipped",
        }
    }
}

// ─── Request / Response types for the REST API ───────────────────

#[derive(Debug, Deserialize)]
pub struct CreateJobRequest {
    pub agent_id: String,
    pub workflow_id: Option<String>,
    pub name: String,
    pub prompt: String,
    /// Standard 5-field cron expression, e.g. "0 9 * * *" (9 AM daily UTC)
    pub cron_expr: String,
    /// Max USD spend allowed per single run. Default: 0.10
    pub budget_usd: Option<f64>,
    /// Auto-disable after this many consecutive failures. Default: 3.
    pub max_failures: Option<i64>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateJobRequest {
    pub name: Option<String>,
    pub prompt: Option<String>,
    pub workflow_id: Option<String>,
    pub cron_expr: Option<String>,
    pub budget_usd: Option<f64>,
    pub enabled: Option<bool>,
    pub max_failures: Option<i64>,
}

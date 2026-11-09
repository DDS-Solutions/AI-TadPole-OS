//! @docs ARCHITECTURE:State
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Agent Runner / types
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Type-safe state handling and bounded execution without unhandled panics.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: none declared
//! - **Witness Tests**: none declared

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

pub const DEFAULT_BUDGET_USD: f64 = 0.10;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum JobRunStatus {
    Running,
    Completed,
    Failed,
    BudgetExceeded,
    Skipped,
    Unknown(String),
}

impl JobRunStatus {
    pub fn as_str(&self) -> &str {
        match self {
            JobRunStatus::Running => "running",
            JobRunStatus::Completed => "completed",
            JobRunStatus::Failed => "failed",
            JobRunStatus::BudgetExceeded => "budget_exceeded",
            JobRunStatus::Skipped => "skipped",
            JobRunStatus::Unknown(s) => s.as_str(),
        }
    }

    pub fn from_str_lossless(s: &str) -> Self {
        match s {
            "running" => JobRunStatus::Running,
            "completed" => JobRunStatus::Completed,
            "failed" => JobRunStatus::Failed,
            "budget_exceeded" => JobRunStatus::BudgetExceeded,
            "skipped" => JobRunStatus::Skipped,
            other => JobRunStatus::Unknown(other.to_string()),
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

#[derive(Debug, Default, Deserialize)]
pub struct UpdateJobRequest {
    pub name: Option<String>,
    pub prompt: Option<String>,
    pub workflow_id: Option<String>,
    pub cron_expr: Option<String>,
    pub budget_usd: Option<f64>,
    pub enabled: Option<bool>,
    pub max_failures: Option<i64>,
    pub metadata: Option<serde_json::Value>,
}

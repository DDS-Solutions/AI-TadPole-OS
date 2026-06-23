//! @docs ARCHITECTURE:Registry
//! 
//! ### AI Assist Note
//! **Core technical resource for the Tadpole OS Sovereign infrastructure.**
//! This module implements high-fidelity logic for the Sovereign Reality layer.
//! 
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: Runtime logic error, state desynchronization, or resource exhaustion.
//! - **Telemetry Link**: Search `[types]` in tracing logs.

use crate::error::AppError;
use crate::state::AppState;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use once_cell::sync::Lazy;

/// Defines a deterministic sequence of agent tasks.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workflow {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub version: i32,
}

fn default_max_retries() -> i32 {
    3
}
fn default_backoff_factor_secs() -> i64 {
    2
}
fn default_depends_on() -> Vec<String> {
    vec![]
}

/// A single execution unit within a Workflow.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowStep {
    pub id: String,
    pub workflow_id: String,
    pub agent_id: String,
    pub step_order: i32,
    pub name: String,
    pub prompt_template: String,
    pub config: Option<serde_json::Value>,
    #[serde(default = "default_max_retries")]
    pub max_retries: i32,
    #[serde(default = "default_backoff_factor_secs")]
    pub backoff_factor_secs: i64,
    #[serde(default = "default_depends_on")]
    pub depends_on: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepConfig {
    pub context_keys: Option<Vec<String>>,
    pub context_json_path: Option<String>,
    pub context_max_chars: Option<i64>,
    pub routing: Option<RoutingConfig>,
    pub fan_out: Option<FanOutConfig>,
    pub tournament: Option<TournamentConfig>,
}

impl StepConfig {
    pub fn validate(&self) -> Result<(), AppError> {
        let mode_count = [self.routing.is_some(), self.fan_out.is_some(), self.tournament.is_some()]
            .iter()
            .filter(|&&x| x)
            .count();
        if mode_count > 1 {
            return Err(AppError::BadRequest(
                "Step config cannot mix routing, fan_out, or tournament modes".to_string(),
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingConfig {
    pub default_next: Option<String>,
    pub rules: Vec<RoutingRule>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingRule {
    pub condition: RuleCondition,
    pub next_step: String,
    pub reset_steps: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConditionOperator {
    Equals,
    NotEquals,
    Contains,
    GreaterThan,
    LessThan,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleCondition {
    pub path: String,
    pub operator: ConditionOperator,
    pub value: serde_json::Value,
}

fn default_fail_strategy() -> FanOutFailStrategy {
    FanOutFailStrategy::FailFast
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FanOutFailStrategy {
    FailFast,
    BestEffort,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FanOutConfig {
    pub array_path: String,
    pub item_placeholder: String,
    pub agent_id: String,
    pub prompt_template: String,
    #[serde(default = "default_fail_strategy")]
    pub fail_strategy: FanOutFailStrategy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TournamentConfig {
    pub candidates: Vec<TournamentCandidate>,
    pub judge_agent_id: String,
    pub judge_prompt_template: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TournamentCandidate {
    pub agent_id: String,
    pub prompt_template: String,
}

#[derive(Clone, Debug, PartialEq)]
pub enum StepStatus {
    Pending,
    Running,
    Completed(String),
    Failed(String),
}

pub struct ActiveRunGuard {
    pub active_runs: Arc<dashmap::DashMap<String, tokio_util::sync::CancellationToken>>,
    pub run_id: String,
}

impl Drop for ActiveRunGuard {
    fn drop(&mut self) {
        self.active_runs.remove(&self.run_id);
    }
}

pub static FALLBACK_RUNTIME: Lazy<Result<tokio::runtime::Runtime, std::io::Error>> = Lazy::new(|| {
    tokio::runtime::Builder::new_multi_thread()
        .worker_threads(1)
        .enable_all()
        .build()
});

/// RAII Guard to clean up running states in the database if workflow execution is dropped/aborted mid-flight.
pub struct WorkflowRunGuard {
    pub run_id: String,
    pub tenant_id: String,
    pub state: Arc<AppState>,
    pub completed: std::sync::atomic::AtomicBool,
}

impl WorkflowRunGuard {
    pub fn new(run_id: String, tenant_id: String, state: Arc<AppState>) -> Self {
        Self {
            run_id,
            tenant_id,
            state,
            completed: std::sync::atomic::AtomicBool::new(false),
        }
    }

    pub fn finalize(&self) {
        self.completed
            .store(true, std::sync::atomic::Ordering::Relaxed);
    }
}

impl Drop for WorkflowRunGuard {
    fn drop(&mut self) {
        if !self.completed.load(std::sync::atomic::Ordering::Relaxed) {
            tracing::warn!("⚠️ [Workflow] WorkflowRunGuard dropped without explicit finalization! Triggering fallback cleanup.");
            let run_id = self.run_id.clone();
            let tenant_id = self.tenant_id.clone();
            let state = Arc::clone(&self.state);
            let pool = self.state.resources.pool.clone();

            let cleanup_future = async move {
                let now = chrono::Utc::now();

                // Write cryptographically chained audit record
                if let Err(e) = state
                    .security
                    .audit_trail
                    .record(
                        "system/workflow_run_guard",
                        Some(&run_id),
                        None,
                        "workflow_run_cleanup",
                        &serde_json::json!({
                            "run_id": run_id,
                            "status": "failed",
                            "reason": "WorkflowRunGuard dropped without explicit finalization"
                        })
                        .to_string(),
                    )
                    .await
                {
                    tracing::error!(
                        "Failed to write audit trail for workflow run cleanup: {}",
                        e
                    );
                }

                if let Err(e) = sqlx::query(
                    "UPDATE workflow_runs SET completed_at = ?1, \
                     status = CASE WHEN status = 'cancelling' THEN 'cancelled' ELSE 'failed' END \
                     WHERE id = ?2 AND tenant_id = ?3 AND status IN ('running', 'cancelling')"
                )
                .bind(now)
                .bind(&run_id)
                .bind(&tenant_id)
                .execute(&pool)
                .await
                {
                    tracing::error!("Failed to clean up workflow run state on drop: {}", e);
                }

                if let Err(e) = sqlx::query("UPDATE workflow_step_runs SET completed_at = ?1, status = 'failed', output_text = 'Cancelled due to execution termination' WHERE run_id = ?2 AND status = 'running' AND EXISTS (SELECT 1 FROM workflow_runs WHERE id = ?2 AND tenant_id = ?3)")
                    .bind(now)
                    .bind(&run_id)
                    .bind(&tenant_id)
                    .execute(&pool)
                    .await
                {
                    tracing::error!("Failed to clean up workflow step runs state on drop: {}", e);
                }
            };

            if let Ok(handle) = tokio::runtime::Handle::try_current() {
                handle.spawn(cleanup_future);
            } else if let Ok(rt) = &*FALLBACK_RUNTIME {
                rt.spawn(cleanup_future);
            } else {
                tracing::warn!("⚠️ [Workflow] No active Tokio runtime handle found and fallback runtime failed to initialize. Skipping fallback async database cleanup.");
            }
        }
    }
}

// Metadata: [types]

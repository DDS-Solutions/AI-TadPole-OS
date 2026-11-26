//! @docs ARCHITECTURE:Runner
//!
//! ### AI Assist Note
//! - Grouped execution context owns identity, mission state and environment; retain scope across callers.
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Agent Runner / metrics_tools
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: none declared

use super::{AgentRunner, RunContext};
use crate::agent::runner::tools::error::ToolExecutionError;
use chrono::Datelike;
use sqlx::{QueryBuilder, Row, Sqlite};

pub const BUDGET_STATUS_BREACHED: &str = "BREACHED";
pub const BUDGET_STATUS_OK: &str = "OK";

/// Computes the start timestamp in RFC3339 for a given timeframe, or returns None for "all".
/// Rejects unsupported timeframe strings with a validation error to trigger agent self-correction.
pub(crate) fn parse_timeframe(
    timeframe: &str,
    now: chrono::DateTime<chrono::Utc>,
) -> Result<Option<String>, ToolExecutionError> {
    match timeframe.trim().to_lowercase().as_str() {
        "all" | "" => Ok(None),
        "year_to_date" | "ytd" => {
            let start = now
                .date_naive()
                .with_month(1)
                .and_then(|d| d.with_day(1))
                .and_then(|d| d.and_hms_opt(0, 0, 0))
                .map(|dt| {
                    chrono::DateTime::<chrono::Utc>::from_naive_utc_and_offset(dt, chrono::Utc)
                })
                .ok_or_else(|| {
                    ToolExecutionError::Validation("Failed to compute UTC start of year".to_string())
                })?;
            Ok(Some(start.to_rfc3339()))
        }
        "month_to_date" | "mtd" => {
            let start = now
                .date_naive()
                .with_day(1)
                .and_then(|d| d.and_hms_opt(0, 0, 0))
                .map(|dt| {
                    chrono::DateTime::<chrono::Utc>::from_naive_utc_and_offset(dt, chrono::Utc)
                })
                .ok_or_else(|| {
                    ToolExecutionError::Validation(
                        "Failed to compute UTC start of month".to_string(),
                    )
                })?;
            Ok(Some(start.to_rfc3339()))
        }
        "today" => {
            let start = now
                .date_naive()
                .and_hms_opt(0, 0, 0)
                .map(|dt| {
                    chrono::DateTime::<chrono::Utc>::from_naive_utc_and_offset(dt, chrono::Utc)
                })
                .ok_or_else(|| {
                    ToolExecutionError::Validation("Failed to compute UTC start of day".to_string())
                })?;
            Ok(Some(start.to_rfc3339()))
        }
        unknown => Err(ToolExecutionError::Validation(format!(
            "Invalid timeframe '{}'. Supported values are: 'all', 'year_to_date' (or 'ytd'), 'month_to_date' (or 'mtd'), 'today'.",
            unknown
        ))),
    }
}

impl AgentRunner {
    /// Handles `handle_get_agent_metrics`: retrieves live financial and identity data from the registry.
    #[tracing::instrument(skip(self, ctx, _fc, _usage), fields(agent_id = %ctx.identity.agent_id))]
    pub(crate) async fn handle_get_agent_metrics(
        &self,
        ctx: &RunContext,
        _fc: &crate::agent::types::ToolCall,
        _usage: &mut Option<crate::agent::types::TokenUsage>,
    ) -> Result<String, ToolExecutionError> {
        let agent_id = &ctx.identity.agent_id;

        tracing::info!(
            "📊 [Governance] Agent {} fetching live metrics...",
            agent_id
        );

        // Fetch live data from the in-memory registry for maximum accuracy
        let (budget, cost, name, role, department) =
            if let Some(a) = self.state.registry.agents.get(agent_id) {
                let v = a.value();
                (
                    v.economics.budget_usd,
                    v.economics.cost_usd,
                    v.identity.name.clone(),
                    v.identity.role.clone(),
                    v.identity.department.clone(),
                )
            } else {
                // Fallback to RunContext if registry lookup fails (though this shouldn't happen during a valid run)
                (
                    0.0,
                    0.0,
                    ctx.identity.name.clone(),
                    ctx.identity.role.clone(),
                    ctx.identity.department.clone(),
                )
            };

        let exec_snap = ctx
            .execution_metrics
            .snapshot(&ctx.mission_state.mission_id, agent_id);
        let status = if cost >= budget {
            BUDGET_STATUS_BREACHED
        } else {
            BUDGET_STATUS_OK
        };

        let metrics = serde_json::json!({
            "agent_id": agent_id,
            "name": name,
            "role": role,
            "department": department,
            "budget_limit_usd": budget,
            "current_cost_usd": cost,
            "remaining_budget_usd": budget - cost,
            "status": status,
            "mission_id": ctx.mission_state.mission_id,
            "execution_metrics": exec_snap
        });

        let metrics_json_str = serde_json::to_string_pretty(&metrics).map_err(|e| {
            ToolExecutionError::ExecutionFailed(format!("Failed to serialize metrics: {}", e))
        })?;

        let metrics_str = format!("(AGENT METRICS RETRIEVED):\n\n{}", metrics_json_str);

        self.broadcast_agent(
            ctx,
            &format!(
                "📊 Governance: reviewed live metrics (${:.4} / ${:.2})",
                cost, budget
            ),
            "info",
        );

        Ok(metrics_str)
    }

    /// Handles `handle_get_system_metrics`: retrieves system-wide operational metrics and telemetry from the database.
    #[tracing::instrument(skip(self, ctx, fc, _usage), fields(agent_id = %ctx.identity.agent_id))]
    pub(crate) async fn handle_get_system_metrics(
        &self,
        ctx: &RunContext,
        fc: &crate::agent::types::ToolCall,
        _usage: &mut Option<crate::agent::types::TokenUsage>,
    ) -> Result<String, ToolExecutionError> {
        let timeframe_str = fc
            .args
            .get("timeframe")
            .and_then(|v| v.as_str())
            .unwrap_or("all");

        let agent_id_filter = fc
            .args
            .get("agent_id")
            .and_then(|v| v.as_str())
            .map(|s| s.trim())
            .filter(|s| !s.is_empty());

        let now = chrono::Utc::now();
        let start_date = parse_timeframe(timeframe_str, now)?;

        tracing::info!(
            "📊 [System Metrics] Agent {} requesting system metrics (timeframe: {}, agent: {:?})",
            ctx.identity.agent_id,
            timeframe_str,
            agent_id_filter
        );

        let pool = &self.state.resources.pool;

        // 1. Consolidated single-query scan for mission totals, cost, and categorical statuses
        let mut qb = QueryBuilder::<Sqlite>::new(
            "SELECT \
                COUNT(*), \
                COALESCE(SUM(cost_usd), 0.0), \
                COALESCE(SUM(CASE WHEN LOWER(status) = 'completed' THEN 1 ELSE 0 END), 0), \
                COALESCE(SUM(CASE WHEN LOWER(status) = 'failed' THEN 1 ELSE 0 END), 0), \
                COALESCE(SUM(CASE WHEN LOWER(status) IN ('active', 'running') THEN 1 ELSE 0 END), 0), \
                COALESCE(SUM(CASE WHEN LOWER(status) NOT IN ('completed', 'failed', 'active', 'running') THEN 1 ELSE 0 END), 0) \
            FROM mission_history"
        );

        let mut has_where = false;
        if let Some(ref start) = start_date {
            qb.push(" WHERE created_at >= ");
            qb.push_bind(start);
            has_where = true;
        }
        if let Some(agent) = agent_id_filter {
            if has_where {
                qb.push(" AND agent_id = ");
            } else {
                qb.push(" WHERE agent_id = ");
            }
            qb.push_bind(agent);
        }

        let mission_row = qb.build().fetch_one(pool).await?;

        let total_missions: i64 = mission_row.try_get(0).unwrap_or(0);
        let total_cost_usd: f64 = mission_row.try_get(1).unwrap_or(0.0);
        let completed_missions: i64 = mission_row.try_get(2).unwrap_or(0);
        let failed_missions: i64 = mission_row.try_get(3).unwrap_or(0);
        let active_missions: i64 = mission_row.try_get(4).unwrap_or(0);
        let other_missions: i64 = mission_row.try_get(5).unwrap_or(0);

        let finalized = completed_missions + failed_missions;
        let success_rate = if finalized > 0 {
            (completed_missions as f64 / finalized as f64) * 100.0
        } else {
            0.0
        };

        // 2. Query continuity scheduled jobs
        let (total_scheduled_jobs, enabled_scheduled_jobs): (i64, i64) = match sqlx::query(
            "SELECT COUNT(*), COALESCE(SUM(CASE WHEN enabled = 1 THEN 1 ELSE 0 END), 0) FROM scheduled_jobs",
        )
        .fetch_optional(pool)
        .await?
        {
            Some(row) => (row.try_get(0).unwrap_or(0), row.try_get(1).unwrap_or(0)),
            None => (0, 0),
        };

        // 3. Query continuity scheduled job runs
        let run_rows =
            sqlx::query("SELECT status, COUNT(*) FROM scheduled_job_runs GROUP BY status")
                .fetch_all(pool)
                .await?;

        let mut completed_runs = 0i64;
        let mut failed_runs = 0i64;
        let mut skipped_runs = 0i64;
        let mut total_runs = 0i64;

        for r in run_rows {
            let status: String = r.try_get(0).unwrap_or_default();
            let count: i64 = r.try_get(1).unwrap_or(0);
            total_runs += count;
            match status.to_lowercase().as_str() {
                "completed" => completed_runs += count,
                "failed" => failed_runs += count,
                "skipped" => skipped_runs += count,
                _ => {}
            }
        }

        // 4. Registry agent stats
        let total_registered_agents = self.state.registry.agents.len();
        let active_agents_count = self
            .state
            .registry
            .agents
            .iter()
            .filter(|a| {
                let s = a.value().health.status.as_str();
                s != "idle" && s != "offline" && s != "suspended"
            })
            .count();

        let metrics_json = serde_json::json!({
            "timeframe": timeframe_str,
            "filter_agent_id": agent_id_filter,
            "start_date": start_date,
            "generated_at": now.to_rfc3339(),
            "missions": {
                "total": total_missions,
                "completed": completed_missions,
                "failed": failed_missions,
                "active": active_missions,
                "other": other_missions,
                "success_rate_percent": format!("{:.2}%", success_rate),
                "total_cost_usd": total_cost_usd
            },
            "scheduled_continuity_jobs": {
                "total_jobs": total_scheduled_jobs,
                "enabled_jobs": enabled_scheduled_jobs,
                "total_runs": total_runs,
                "completed_runs": completed_runs,
                "failed_runs": failed_runs,
                "skipped_runs": skipped_runs
            },
            "swarm_agents": {
                "total_registered": total_registered_agents,
                "currently_active": active_agents_count
            }
        });

        let metrics_json_str = serde_json::to_string_pretty(&metrics_json).map_err(|e| {
            ToolExecutionError::ExecutionFailed(format!(
                "Failed to serialize system metrics JSON: {}",
                e
            ))
        })?;

        let summary = format!(
            "(SYSTEM METRICS RETRIEVED):\n\n\
            ### 📊 Sovereign System Operational Metrics\n\
            - **Timeframe**: {} (since {})\n\
            - **Total Missions**: {}\n\
              - ✅ Completed: {}\n\
              - ❌ Failed: {}\n\
              - 🔄 Active: {}\n\
              - 📈 Success Rate: {:.2}%\n\
            - **Financial Spend**: ${:.4} USD\n\
            - **Scheduled Continuity Jobs**: {} total ({} enabled), {} total runs ({} completed, {} failed, {} skipped)\n\
            - **Swarm Agents**: {} registered ({} active)\n\n\
            ```json\n{}\n```",
            timeframe_str,
            start_date.as_deref().unwrap_or("beginning of records"),
            total_missions,
            completed_missions,
            failed_missions,
            active_missions,
            success_rate,
            total_cost_usd,
            total_scheduled_jobs,
            enabled_scheduled_jobs,
            total_runs,
            completed_runs,
            failed_runs,
            skipped_runs,
            total_registered_agents,
            active_agents_count,
            metrics_json_str
        );

        self.broadcast_agent(
            ctx,
            &format!(
                "📊 System Metrics: reviewed {} missions (${:.4}) for {}",
                total_missions, total_cost_usd, timeframe_str
            ),
            "info",
        );

        Ok(summary)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Verifies the budget status boundary: cost >= budget => BREACHED, else OK.
    #[test]
    fn test_metrics_budget_status_boundary() {
        let check_status = |cost: f64, budget: f64| -> &'static str {
            if cost >= budget {
                BUDGET_STATUS_BREACHED
            } else {
                BUDGET_STATUS_OK
            }
        };

        assert_eq!(check_status(0.0, 100.0), BUDGET_STATUS_OK);
        assert_eq!(check_status(99.99, 100.0), BUDGET_STATUS_OK);
        assert_eq!(check_status(100.0, 100.0), BUDGET_STATUS_BREACHED);
        assert_eq!(check_status(100.01, 100.0), BUDGET_STATUS_BREACHED);
        assert_eq!(check_status(0.0, 0.0), BUDGET_STATUS_BREACHED);
    }

    /// Verifies success rate calculation with zero and non-zero division safety.
    #[test]
    fn test_system_metrics_success_rate() {
        let calc_rate = |completed: i64, failed: i64| -> f64 {
            let finalized = completed + failed;
            if finalized > 0 {
                (completed as f64 / finalized as f64) * 100.0
            } else {
                0.0
            }
        };

        assert_eq!(calc_rate(0, 0), 0.0);
        assert_eq!(calc_rate(25, 0), 100.0);
        assert_eq!(calc_rate(20, 20), 50.0);

        let expected_rate = (25.0 / 39.0) * 100.0;
        assert!((calc_rate(25, 14) - expected_rate).abs() < f64::EPSILON * 100.0);
    }

    /// Verifies timeframe parsing for all valid options and case variations.
    #[test]
    fn test_parse_timeframe_valid() {
        let test_now = chrono::DateTime::parse_from_rfc3339("2026-10-17T15:30:00Z")
            .unwrap()
            .with_timezone(&chrono::Utc);

        assert_eq!(parse_timeframe("all", test_now).unwrap(), None);
        assert_eq!(parse_timeframe("", test_now).unwrap(), None);
        assert_eq!(parse_timeframe("  ALL  ", test_now).unwrap(), None);

        assert_eq!(
            parse_timeframe("year_to_date", test_now).unwrap(),
            Some("2026-01-01T00:00:00+00:00".to_string())
        );
        assert_eq!(
            parse_timeframe("YTD", test_now).unwrap(),
            Some("2026-01-01T00:00:00+00:00".to_string())
        );

        assert_eq!(
            parse_timeframe("month_to_date", test_now).unwrap(),
            Some("2026-10-01T00:00:00+00:00".to_string())
        );
        assert_eq!(
            parse_timeframe("MTD", test_now).unwrap(),
            Some("2026-10-01T00:00:00+00:00".to_string())
        );

        assert_eq!(
            parse_timeframe("today", test_now).unwrap(),
            Some("2026-10-17T00:00:00+00:00".to_string())
        );
    }

    /// Verifies timeframe parsing returns a Validation error on unknown timeframes.
    #[test]
    fn test_parse_timeframe_invalid() {
        let test_now = chrono::Utc::now();
        let result = parse_timeframe("last_week", test_now);

        match result {
            Err(ToolExecutionError::Validation(msg)) => {
                assert!(msg.contains("Invalid timeframe 'last_week'"));
                assert!(msg.contains("year_to_date"));
            }
            other => panic!("Expected Validation error, got {:?}", other),
        }
    }
}

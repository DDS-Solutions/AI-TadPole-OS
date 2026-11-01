//! @docs ARCHITECTURE:ResourceMetering
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Security & Governance / metering
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: none declared
//!
//! ### AI Assist Note
//! - Keep cancellation ownership and usage persistence aligned with the existing execution paths.

use anyhow::{anyhow, Result};
use chrono::{DateTime, Datelike, Utc};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use std::collections::VecDeque;
use std::sync::Arc;

/// Micro-dollars per 1.00 USD (1 USD = 1,000,000 micro-dollars).
pub const MICROS_PER_USD: f64 = 1_000_000.0;

/// Default initial budget for newly encountered agents ($0.50).
pub const DEFAULT_AGENT_BUDGET_USD: f64 = 0.50;

/// Default initial budget for newly encountered missions/clusters ($5.00).
pub const DEFAULT_MISSION_BUDGET_USD: f64 = 5.00;

/// Memory pressure threshold (85%) above which cost estimation multiplier is applied.
pub const HIGH_MEMORY_PRESSURE_THRESHOLD: f64 = 0.85;

/// Cost multiplier applied during high memory pressure.
pub const HIGH_MEMORY_PRESSURE_MULTIPLIER: f64 = 2.0;

/// TTL duration for in-flight budget reservations (60 seconds).
pub const RESERVATION_TTL_SECS: i64 = 60;

/// Convert USD (`f64`) to integer micro-dollars (`i64`) with strict boundary validation.
///
/// Rejects NaN, Infinity, negative values, and values that would overflow i64.
pub fn usd_to_micros(cost_usd: f64) -> Result<i64> {
    if !cost_usd.is_finite() {
        return Err(anyhow!("cost_usd must be finite (got {})", cost_usd));
    }
    if cost_usd < 0.0 {
        return Err(anyhow!("cost_usd cannot be negative (got {})", cost_usd));
    }
    let scaled = cost_usd * MICROS_PER_USD;
    if scaled > (i64::MAX as f64) {
        return Err(anyhow!("cost_usd exceeds maximum representable budget"));
    }
    Ok(scaled.round() as i64)
}

/// Convert integer micro-dollars (`i64`) to USD (`f64`).
pub fn micros_to_usd(micros: i64) -> f64 {
    (micros as f64) / MICROS_PER_USD
}

/// Computes the next scheduled reset timestamp based on the reset period.
pub fn compute_next_reset(now: DateTime<Utc>, period: ResetPeriod) -> DateTime<Utc> {
    match period {
        ResetPeriod::Daily => now + chrono::Duration::days(1),
        ResetPeriod::Monthly => {
            let mut year = now.year();
            let mut month = now.month() + 1;
            if month > 12 {
                month = 1;
                year += 1;
            }
            let day = now.day().min(28);
            chrono::NaiveDate::from_ymd_opt(year, month, day)
                .and_then(|d| d.and_time(now.time()).and_local_timezone(Utc).single())
                .unwrap_or_else(|| now + chrono::Duration::days(30))
        }
        ResetPeriod::Never => now + chrono::Duration::days(365 * 100),
    }
}

/// Defines the resource allocation and consumption limits for an entity (agent or user).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Quota {
    /// Unique database primary key for the quota entry.
    pub id: String,
    /// The unique identifier for the agent or user being metered.
    pub entity_id: String,
    /// Total allowed budget in USD for the current period.
    pub budget_usd: f64,
    /// Cumulative cost consumed in the current period.
    pub used_usd: f64,
    /// How often the budget resets to zero (e.g., Daily, Monthly, Never).
    pub reset_period: ResetPeriod,
    /// Timestamp of the last successful reset.
    pub last_reset_at: DateTime<Utc>,
    /// Scheduled timestamp for the next automatic reset.
    pub next_reset_at: DateTime<Utc>,
}

/// Specifies the frequency for budget replenishment.
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ResetPeriod {
    Daily,
    Monthly,
    /// Budget never resets; essentially a lifetime cap.
    Never,
}

#[derive(Debug, Clone, Copy)]
enum QuotaTarget {
    Agent,
    Mission,
}

type Reservations = DashMap<String, VecDeque<(i64, DateTime<Utc>, Option<String>)>>;

/// One provider attempt owns its reservations until response, error, or cancellation.
pub(crate) struct CallBudgetReservation {
    guard: Arc<BudgetGuard>,
    owner: String,
    agent_id: String,
    mission_id: String,
    settled: bool,
}

/// A security guard responsible for enforcing LLM cost constraints.
///
/// BudgetGuard prevents model extraction attacks or runaway loops by checking
/// and reserving quotas before permitting LLM calls. Actual costs are recorded
/// post-execution into the SQLite persistence layer to ensure financial governance.
pub struct BudgetGuard {
    /// Database pool for persistence of usage metrics.
    pool: SqlitePool,
    /// Thread-safe buffer for debounced agent usage updates in i64 micro-dollars (entity_id -> accumulated_micros).
    buffer: DashMap<String, i64>,
    /// Thread-safe buffer for debounced mission usage updates in i64 micro-dollars (cluster_id -> accumulated_micros).
    mission_buffer: DashMap<String, i64>,
    /// Budget reads and flushes share a boundary; buffered charges stay visible until commit.
    flush_lock: tokio::sync::Mutex<()>,
    accounting_gate: parking_lot::Mutex<()>,
    #[cfg(test)]
    flush_commit_pause:
        parking_lot::Mutex<Option<(Arc<tokio::sync::Notify>, Arc<tokio::sync::Notify>)>>,
    /// In-flight budget reservations for agents (entity_id -> queue of (reserved_micros, timestamp)).
    reservations: Reservations,
    /// In-flight budget reservations for missions (cluster_id -> queue of (reserved_micros, timestamp)).
    mission_reservations: Reservations,
    /// Security monitor reference for cached memory pressure checks.
    system_monitor: Arc<crate::security::monitoring::SecurityMonitor>,
}

impl BudgetGuard {
    pub fn new(
        pool: SqlitePool,
        system_monitor: Arc<crate::security::monitoring::SecurityMonitor>,
    ) -> Self {
        Self {
            pool,
            buffer: DashMap::new(),
            mission_buffer: DashMap::new(),
            flush_lock: tokio::sync::Mutex::new(()),
            accounting_gate: parking_lot::Mutex::new(()),
            #[cfg(test)]
            flush_commit_pause: parking_lot::Mutex::new(None),
            reservations: DashMap::new(),
            mission_reservations: DashMap::new(),
            system_monitor,
        }
    }
}

mod persistence;
mod reservations;
mod usage;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::security::monitoring::SecurityMonitor;
    use sqlx::SqlitePool;

    async fn setup_test_db() -> SqlitePool {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS agent_quotas (
                id TEXT PRIMARY KEY NOT NULL,
                entity_id TEXT NOT NULL UNIQUE,
                used_usd INTEGER NOT NULL DEFAULT 0,
                budget_usd INTEGER NOT NULL DEFAULT 500000,
                last_reset_at DATETIME NOT NULL,
                next_reset_at DATETIME NOT NULL,
                reset_period TEXT NOT NULL CHECK(reset_period IN ('daily', 'monthly', 'never'))
            )",
        )
        .execute(&pool)
        .await
        .unwrap();

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS mission_quotas (
                id TEXT PRIMARY KEY NOT NULL,
                cluster_id TEXT NOT NULL UNIQUE,
                used_usd INTEGER NOT NULL DEFAULT 0,
                budget_usd INTEGER NOT NULL DEFAULT 5000000,
                last_reset_at DATETIME NOT NULL,
                next_reset_at DATETIME NOT NULL,
                reset_period TEXT NOT NULL CHECK(reset_period IN ('daily', 'monthly', 'never'))
            )",
        )
        .execute(&pool)
        .await
        .unwrap();

        sqlx::query("CREATE TABLE IF NOT EXISTS mission_history (id TEXT PRIMARY KEY, cost_usd REAL NOT NULL DEFAULT 0)")
            .execute(&pool).await.unwrap();
        pool
    }

    #[tokio::test]
    async fn test_quota_enforcement() -> Result<()> {
        let pool = setup_test_db().await;
        let monitor = Arc::new(SecurityMonitor::for_test(0.0));
        let guard = Arc::new(BudgetGuard::new(pool, monitor));

        // 1. Add agent with 1.0 budget manually (1_000_000 micros)
        sqlx::query(
            "INSERT INTO agent_quotas (id, entity_id, used_usd, budget_usd, reset_period, last_reset_at, next_reset_at) \
             VALUES ('1', 'agent_1', 0, 1000000, 'never', '2024-01-01 00:00:00', '2124-01-01 00:00:00')"
        )
        .execute(&guard.pool).await?;

        assert!(guard.check_budget("agent_1", 0.01).await?);

        // 2. Record usage nearly hitting budget
        guard.record_usage("agent_1", 0.90).await?;
        assert!(guard.check_budget("agent_1", 0.05).await?);

        // 3. Hit budget
        guard.record_usage("agent_1", 0.10).await?; // Total used: 1.0 (1_000_000 micros)
        assert!(!guard.check_budget("agent_1", 0.01).await?); // 1.01 > 1.0

        Ok(())
    }

    #[tokio::test]
    async fn test_debounced_flushing() -> Result<()> {
        let pool = setup_test_db().await;
        let monitor = Arc::new(SecurityMonitor::for_test(0.0));
        let guard = Arc::new(BudgetGuard::new(pool, monitor));

        // 1. Setup agent & mission
        sqlx::query(
            "INSERT INTO agent_quotas (id, entity_id, used_usd, budget_usd, reset_period, last_reset_at, next_reset_at) \
             VALUES ('1', 'agent_flush', 0, 10000000, 'never', '2024-01-01 00:00:00', '2124-01-01 00:00:00')"
        )
        .execute(&guard.pool).await?;
        sqlx::query(
            "INSERT INTO mission_quotas (id, cluster_id, used_usd, budget_usd, reset_period, last_reset_at, next_reset_at) \
             VALUES ('2', 'cluster_flush', 0, 10000000, 'never', '2024-01-01 00:00:00', '2124-01-01 00:00:00')"
        )
        .execute(&guard.pool).await?;

        // 2. Record usage (should only hit buffers)
        guard.record_usage("agent_flush", 1.5).await?;
        guard.record_usage("agent_flush", 2.5).await?;
        guard.record_mission_usage("cluster_flush", 5.0).await?;

        // Verify DB hasn't changed yet
        let agent_used: i64 =
            sqlx::query_scalar("SELECT used_usd FROM agent_quotas WHERE entity_id = 'agent_flush'")
                .fetch_one(&guard.pool)
                .await?;
        let mission_used: i64 = sqlx::query_scalar(
            "SELECT used_usd FROM mission_quotas WHERE cluster_id = 'cluster_flush'",
        )
        .fetch_one(&guard.pool)
        .await?;

        assert_eq!(agent_used, 0);
        assert_eq!(mission_used, 0);

        // 3. Flush
        guard.flush_to_db().await?;

        // Verify DB updated
        let agent_used_after: i64 =
            sqlx::query_scalar("SELECT used_usd FROM agent_quotas WHERE entity_id = 'agent_flush'")
                .fetch_one(&guard.pool)
                .await?;
        let mission_used_after: i64 = sqlx::query_scalar(
            "SELECT used_usd FROM mission_quotas WHERE cluster_id = 'cluster_flush'",
        )
        .fetch_one(&guard.pool)
        .await?;

        assert_eq!(agent_used_after, 4000000);
        assert_eq!(mission_used_after, 5000000);

        // Verify buffers are reset/removed
        assert!(guard.buffer.get("agent_flush").is_none());
        assert!(guard.mission_buffer.get("cluster_flush").is_none());

        Ok(())
    }

    #[tokio::test]
    async fn failed_statement_retains_both_buffers_for_exact_retry() -> Result<()> {
        let pool = setup_test_db().await;
        let guard = Arc::new(BudgetGuard::new(
            pool,
            Arc::new(SecurityMonitor::for_test(0.0)),
        ));
        sqlx::query("INSERT INTO mission_history (id) VALUES ('mission-retry')")
            .execute(&guard.pool)
            .await?;
        guard.record_usage("agent-retry", 0.1).await?;
        guard.record_mission_usage("mission-retry", 0.1).await?;
        sqlx::query("CREATE TRIGGER reject_mission BEFORE INSERT ON mission_quotas BEGIN SELECT RAISE(ABORT, 'test rejection'); END")
            .execute(&guard.pool).await?;
        assert!(guard.flush_to_db().await.is_err());
        assert_eq!(*guard.buffer.get("agent-retry").unwrap(), 100_000);
        assert_eq!(*guard.mission_buffer.get("mission-retry").unwrap(), 100_000);
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM agent_quotas")
            .fetch_one(&guard.pool)
            .await?;
        assert_eq!(
            count, 0,
            "Agent write must roll back when mission write fails"
        );
        sqlx::query("DROP TRIGGER reject_mission")
            .execute(&guard.pool)
            .await?;
        guard.flush_to_db().await?;
        guard.flush_to_db().await?;
        assert_eq!(
            guard.get_or_create_quota("agent-retry").await?.used_usd,
            0.1
        );
        assert_eq!(
            guard
                .get_or_create_mission_quota("mission-retry")
                .await?
                .used_usd,
            0.1
        );
        let cost: f64 =
            sqlx::query_scalar("SELECT cost_usd FROM mission_history WHERE id = 'mission-retry'")
                .fetch_one(&guard.pool)
                .await?;
        assert_eq!(cost, 0.1);
        assert!(!guard.has_pending_usage());
        Ok(())
    }

    #[tokio::test]
    async fn cancelled_flush_after_commit_keeps_new_usage_and_settles_once() -> Result<()> {
        let pool = setup_test_db().await;
        let guard = Arc::new(BudgetGuard::new(
            pool,
            Arc::new(SecurityMonitor::for_test(0.0)),
        ));
        guard
            .update_quota("agent-commit", 0.3, Some(ResetPeriod::Never))
            .await?;
        guard.record_usage("agent-commit", 0.1).await?;
        let committed = Arc::new(tokio::sync::Notify::new());
        let release = Arc::new(tokio::sync::Notify::new());
        *guard.flush_commit_pause.lock() = Some((committed.clone(), release.clone()));
        let caller_guard = guard.clone();
        let caller = tokio::spawn(async move { caller_guard.flush_to_db().await });
        tokio::time::timeout(std::time::Duration::from_secs(5), committed.notified()).await?;
        caller.abort();
        let _ = caller.await;
        guard.record_usage("agent-commit", 0.1).await?;
        let read_guard = guard.clone();
        let budget_read =
            tokio::spawn(async move { read_guard.check_budget("agent-commit", 0.15).await });
        tokio::task::yield_now().await;
        assert!(
            !budget_read.is_finished(),
            "Quota checks wait for commit reconciliation"
        );
        release.notify_one();
        assert!(!budget_read.await??);
        guard.flush_to_db().await?;
        assert_eq!(
            guard.get_or_create_quota("agent-commit").await?.used_usd,
            0.2
        );
        assert!(!guard.has_pending_usage());
        Ok(())
    }

    #[tokio::test]
    async fn owned_reservations_cancel_out_of_order_and_settle_once() -> Result<()> {
        let pool = setup_test_db().await;
        let guard = Arc::new(BudgetGuard::new(
            pool,
            Arc::new(SecurityMonitor::for_test(0.0)),
        ));
        guard
            .update_quota("agent-owned", 1.0, Some(ResetPeriod::Never))
            .await?;
        guard
            .update_mission_quota("mission-owned", 1.0, Some(ResetPeriod::Never))
            .await?;
        let first = guard
            .reserve_call("agent-owned", "mission-owned", 0.6, 1.0)
            .await?
            .unwrap();
        let mut second = guard
            .reserve_call("agent-owned", "mission-owned", 0.3, 1.0)
            .await?
            .unwrap();
        drop(first);
        assert_eq!(
            BudgetGuard::get_active_reservation(&guard.reservations, "agent-owned"),
            300_000
        );
        second.settle(0.2)?;
        assert!(second.settle(0.2).is_err());
        drop(second);
        assert_eq!(
            BudgetGuard::get_active_reservation(&guard.reservations, "agent-owned"),
            0
        );
        assert_eq!(*guard.buffer.get("agent-owned").unwrap(), 200_000);
        assert_eq!(*guard.mission_buffer.get("mission-owned").unwrap(), 200_000);
        guard
            .update_mission_quota("mission-owned", 0.0, Some(ResetPeriod::Never))
            .await?;
        assert!(guard
            .reserve_call("agent-owned", "mission-owned", 0.01, 0.0)
            .await?
            .is_none());
        Ok(())
    }

    #[tokio::test]
    async fn high_memory_pressure_reserves_conservatively_without_overcharging() -> Result<()> {
        let pool = setup_test_db().await;
        let guard = Arc::new(BudgetGuard::new(
            pool,
            Arc::new(SecurityMonitor::for_test(0.9)),
        ));
        guard
            .update_quota("pressure-agent", 1.0, Some(ResetPeriod::Never))
            .await?;
        guard
            .update_mission_quota("pressure-mission", 1.0, Some(ResetPeriod::Never))
            .await?;
        let mut reservation = guard
            .reserve_call("pressure-agent", "pressure-mission", 0.4, 1.0)
            .await?
            .unwrap();
        assert_eq!(
            BudgetGuard::get_active_reservation(&guard.reservations, "pressure-agent"),
            800_000
        );
        assert!(guard
            .reserve_call("pressure-agent", "pressure-mission", 0.2, 1.0)
            .await?
            .is_none());
        reservation.settle(0.3)?;
        assert_eq!(*guard.buffer.get("pressure-agent").unwrap(), 300_000);
        assert_eq!(
            *guard.mission_buffer.get("pressure-mission").unwrap(),
            300_000
        );
        assert_eq!(
            BudgetGuard::get_active_reservation(&guard.reservations, "pressure-agent"),
            0
        );
        Ok(())
    }

    #[tokio::test]
    async fn owned_reservations_do_not_expire_during_long_provider_calls() -> Result<()> {
        let pool = setup_test_db().await;
        let guard = Arc::new(BudgetGuard::new(
            pool,
            Arc::new(SecurityMonitor::for_test(0.0)),
        ));
        guard
            .update_quota("agent-long", 0.5, Some(ResetPeriod::Never))
            .await?;
        let lease = guard
            .reserve_call("agent-long", "mission-long", 0.4, 0.0)
            .await?
            .unwrap();
        guard.reservations.get_mut("agent-long").unwrap()[0].1 =
            Utc::now() - chrono::Duration::seconds(120);
        assert!(guard
            .reserve_call("agent-long", "mission-long", 0.2, 0.0)
            .await?
            .is_none());
        drop(lease);
        assert!(guard
            .reserve_call("agent-long", "mission-long", 0.2, 0.0)
            .await?
            .is_some());
        Ok(())
    }

    #[tokio::test]
    async fn begin_failure_retains_usage() -> Result<()> {
        let pool = setup_test_db().await;
        let guard = Arc::new(BudgetGuard::new(
            pool,
            Arc::new(SecurityMonitor::for_test(0.0)),
        ));
        guard.record_usage("closed-agent", 0.1).await?;
        guard.record_mission_usage("closed-mission", 0.1).await?;
        guard.pool.close().await;
        assert!(guard.flush_to_db().await.is_err());
        assert_eq!(*guard.buffer.get("closed-agent").unwrap(), 100_000);
        assert_eq!(
            *guard.mission_buffer.get("closed-mission").unwrap(),
            100_000
        );
        Ok(())
    }

    #[tokio::test]
    async fn test_check_then_act_reservation_bounds_concurrency() -> Result<()> {
        let pool = setup_test_db().await;
        let monitor = Arc::new(SecurityMonitor::for_test(0.0));
        let guard = Arc::new(BudgetGuard::new(pool, monitor));

        // Provision agent with $1.00 budget
        guard
            .update_quota("ag_race", 1.00, Some(ResetPeriod::Never))
            .await?;

        // Task 1 checks $0.60 -> should pass and hold reservation
        assert!(guard.check_budget("ag_race", 0.60).await?);

        // Task 2 checks $0.60 concurrently before Task 1 records usage -> MUST BE BLOCKED
        assert!(!guard.check_budget("ag_race", 0.60).await?);

        // Task 3 checks $0.30 -> ($0.60 reserved + $0.30 = $0.90 <= $1.00) -> should pass
        assert!(guard.check_budget("ag_race", 0.30).await?);

        // Settle task 1 with actual $0.50 spend -> pops Task 1's $0.60 reservation, buffer += $0.50
        guard.record_usage("ag_race", 0.50).await?;

        // Now used in buffer = $0.50, reservations = $0.30 (Task 3). Total = $0.80.
        // Checking $0.30 exceeds ($0.80 + $0.30 = $1.10 > $1.00)
        assert!(!guard.check_budget("ag_race", 0.30).await?);

        // Checking $0.15 passes ($0.80 + $0.15 = $0.95 <= $1.00)
        assert!(guard.check_budget("ag_race", 0.15).await?);

        // Settle task 3 with actual $0.30 spend -> pops Task 3's $0.30 reservation, buffer += $0.30
        guard.record_usage("ag_race", 0.30).await?;

        // Now used in buffer = $0.80, reservations = $0.15 (Task 4). Total = $0.95.
        // Checking $0.10 exceeds ($0.95 + $0.10 = $1.05 > $1.00)
        assert!(!guard.check_budget("ag_race", 0.10).await?);

        Ok(())
    }

    #[tokio::test]
    async fn test_flush_upsert_unknown_entity() -> Result<()> {
        let pool = setup_test_db().await;
        let monitor = Arc::new(SecurityMonitor::for_test(0.0));
        let guard = Arc::new(BudgetGuard::new(pool, monitor));

        // Record usage for an entity without prior check_budget or DB row
        guard.record_usage("novel_entity", 0.75).await?;

        // Flush to DB
        guard.flush_to_db().await?;

        // Verify entity was upserted into DB and recorded spend was preserved
        let row: (i64, i64) = sqlx::query_as(
            "SELECT used_usd, budget_usd FROM agent_quotas WHERE entity_id = 'novel_entity'",
        )
        .fetch_one(&guard.pool)
        .await?;

        assert_eq!(row.0, 750_000);
        assert_eq!(row.1, 500_000); // Default $0.50 (500_000 micros)

        Ok(())
    }

    #[test]
    fn test_float_validation_rejects_negative_nan_infinity() {
        assert!(usd_to_micros(-0.01).is_err());
        assert!(usd_to_micros(f64::NAN).is_err());
        assert!(usd_to_micros(f64::INFINITY).is_err());
        assert!(usd_to_micros(f64::NEG_INFINITY).is_err());
        assert_eq!(usd_to_micros(1.50).unwrap(), 1_500_000);
        assert_eq!(usd_to_micros(0.0).unwrap(), 0);
    }

    #[tokio::test]
    async fn test_update_quota_recomputes_reset_period() -> Result<()> {
        let pool = setup_test_db().await;
        let monitor = Arc::new(SecurityMonitor::for_test(0.0));
        let guard = Arc::new(BudgetGuard::new(pool, monitor));

        guard
            .update_quota("entity_reset", 2.00, Some(ResetPeriod::Daily))
            .await?;
        let q1 = guard.get_or_create_quota("entity_reset").await?;
        assert_eq!(q1.reset_period, ResetPeriod::Daily);

        // Change to Monthly
        guard
            .update_quota("entity_reset", 5.00, Some(ResetPeriod::Monthly))
            .await?;
        let q2 = guard.get_or_create_quota("entity_reset").await?;
        assert_eq!(q2.reset_period, ResetPeriod::Monthly);
        assert_eq!(q2.budget_usd, 5.00);

        Ok(())
    }

    #[tokio::test]
    async fn test_mission_budget_check_and_flush() -> Result<()> {
        let pool = setup_test_db().await;
        let monitor = Arc::new(SecurityMonitor::for_test(0.0));
        let guard = Arc::new(BudgetGuard::new(pool, monitor));

        // Mission check and record
        assert!(guard.check_mission_budget("cluster_1", 2.00).await?);
        guard.record_mission_usage("cluster_1", 1.80).await?;

        guard.flush_to_db().await?;

        let row: (i64, i64) = sqlx::query_as(
            "SELECT used_usd, budget_usd FROM mission_quotas WHERE cluster_id = 'cluster_1'",
        )
        .fetch_one(&guard.pool)
        .await?;

        assert_eq!(row.0, 1_800_000);
        assert_eq!(row.1, 5_000_000); // Default $5.00

        Ok(())
    }
}

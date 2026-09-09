//! @docs ARCHITECTURE:ResourceMetering
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Security & Governance / metering
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Type-safe state handling and bounded execution without unhandled panics.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: none declared
//! - **Witness Tests**: none declared

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
    /// In-flight budget reservations for agents (entity_id -> queue of (reserved_micros, timestamp)).
    reservations: DashMap<String, VecDeque<(i64, DateTime<Utc>)>>,
    /// In-flight budget reservations for missions (cluster_id -> queue of (reserved_micros, timestamp)).
    mission_reservations: DashMap<String, VecDeque<(i64, DateTime<Utc>)>>,
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
            reservations: DashMap::new(),
            mission_reservations: DashMap::new(),
            system_monitor,
        }
    }

    fn get_active_reservation(
        map: &DashMap<String, VecDeque<(i64, DateTime<Utc>)>>,
        id: &str,
    ) -> i64 {
        let now = Utc::now();
        if let Some(entry) = map.get(id) {
            entry
                .value()
                .iter()
                .filter(|(_, created_at)| (now - *created_at).num_seconds() <= RESERVATION_TTL_SECS)
                .map(|(amount, _)| *amount)
                .fold(0i64, |acc, x| acc.saturating_add(x))
        } else {
            0
        }
    }

    fn add_reservation(
        map: &DashMap<String, VecDeque<(i64, DateTime<Utc>)>>,
        id: &str,
        amount: i64,
    ) {
        let now = Utc::now();
        let mut entry = map.entry(id.to_string()).or_default();
        entry.retain(|(_, created_at)| (now - *created_at).num_seconds() <= RESERVATION_TTL_SECS);
        entry.push_back((amount, now));
    }

    fn release_reservation(map: &DashMap<String, VecDeque<(i64, DateTime<Utc>)>>, id: &str) {
        let now = Utc::now();
        if let Some(mut entry) = map.get_mut(id) {
            while let Some((_, created_at)) = entry.front() {
                if (now - *created_at).num_seconds() > RESERVATION_TTL_SECS {
                    entry.pop_front();
                } else {
                    entry.pop_front();
                    break;
                }
            }
            if entry.is_empty() {
                drop(entry);
                map.remove(id);
            }
        }
    }

    /// Returns true if there is any pending cost > 0 inside the buffers.
    pub fn has_pending_usage(&self) -> bool {
        self.buffer.iter().any(|entry| *entry.value() > 0)
            || self.mission_buffer.iter().any(|entry| *entry.value() > 0)
    }

    /// Verifies if an entity has sufficient remaining budget and holds an in-flight reservation.
    ///
    /// If the reset time has passed, this method triggers an automatic reset
    /// before performing the check. Returns `true` if (used + buffered + reserved + estimated) <= budget.
    #[tracing::instrument(skip(self), fields(entity_id = %entity_id, cost = estimated_cost), name = "security::budget_check")]
    pub async fn check_budget(&self, entity_id: &str, estimated_cost: f64) -> Result<bool> {
        let raw_estimated_micros = usd_to_micros(estimated_cost)?;
        let memory_pressure = self.system_monitor.get_cached_memory_pressure();
        let multiplier = if memory_pressure > HIGH_MEMORY_PRESSURE_THRESHOLD {
            tracing::warn!(
                "⚠️ [Resource Guardian] High memory pressure ({:.1}%) detected during budget check. Enforcing strict budget multiplier ({:.1}x).",
                memory_pressure * 100.0,
                HIGH_MEMORY_PRESSURE_MULTIPLIER
            );
            HIGH_MEMORY_PRESSURE_MULTIPLIER
        } else {
            1.0
        };

        let estimated_micros = ((raw_estimated_micros as f64) * multiplier).round() as i64;
        let mut quota = self.get_or_create_quota(entity_id).await?;

        // Auto-reset if needed before scoring
        if Utc::now() >= quota.next_reset_at && quota.reset_period != ResetPeriod::Never {
            self.reset_quota(&quota.id, quota.reset_period).await?;
            quota = self.get_or_create_quota(entity_id).await?;
        }

        let used_micros = usd_to_micros(quota.used_usd)?;
        let budget_micros = usd_to_micros(quota.budget_usd)?;
        let buffered_micros = self.buffer.get(entity_id).map(|v| *v.value()).unwrap_or(0);
        let reserved_micros = Self::get_active_reservation(&self.reservations, entity_id);

        let total_anticipated = used_micros
            .saturating_add(buffered_micros)
            .saturating_add(reserved_micros)
            .saturating_add(estimated_micros);

        if total_anticipated <= budget_micros {
            Self::add_reservation(&self.reservations, entity_id, estimated_micros);
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Records actual cost after an operation completes and settles in-flight reservations.
    #[tracing::instrument(skip(self), fields(entity_id = %entity_id, cost = cost_usd), name = "security::budget_record")]
    pub async fn record_usage(&self, entity_id: &str, cost_usd: f64) -> Result<()> {
        let cost_micros = usd_to_micros(cost_usd)?;
        if cost_micros > 0 {
            *self.buffer.entry(entity_id.to_string()).or_insert(0) += cost_micros;
        }
        Self::release_reservation(&self.reservations, entity_id);
        Ok(())
    }

    /// Checks if a mission (cluster) has sufficient remaining budget and holds an in-flight reservation.
    #[allow(dead_code)]
    pub async fn check_mission_budget(
        &self,
        cluster_id: &str,
        estimated_cost: f64,
    ) -> Result<bool> {
        let estimated_micros = usd_to_micros(estimated_cost)?;
        let mut quota = self.get_or_create_mission_quota(cluster_id).await?;

        // Auto-reset if needed before scoring
        if Utc::now() >= quota.next_reset_at && quota.reset_period != ResetPeriod::Never {
            self.reset_mission_quota(&quota.id, quota.reset_period)
                .await?;
            quota = self.get_or_create_mission_quota(cluster_id).await?;
        }

        let used_micros = usd_to_micros(quota.used_usd)?;
        let budget_micros = usd_to_micros(quota.budget_usd)?;
        let buffered_micros = self
            .mission_buffer
            .get(cluster_id)
            .map(|v| *v.value())
            .unwrap_or(0);
        let reserved_micros = Self::get_active_reservation(&self.mission_reservations, cluster_id);

        let total_anticipated = used_micros
            .saturating_add(buffered_micros)
            .saturating_add(reserved_micros)
            .saturating_add(estimated_micros);

        if total_anticipated <= budget_micros {
            Self::add_reservation(&self.mission_reservations, cluster_id, estimated_micros);
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Records usage for a specific mission and settles in-flight reservations.
    #[allow(dead_code)]
    pub async fn record_mission_usage(&self, cluster_id: &str, cost_usd: f64) -> Result<()> {
        let cost_micros = usd_to_micros(cost_usd)?;
        if cost_micros > 0 {
            *self
                .mission_buffer
                .entry(cluster_id.to_string())
                .or_insert(0) += cost_micros;
        }
        Self::release_reservation(&self.mission_reservations, cluster_id);
        Ok(())
    }

    /// Flushes all buffered usage metrics to the database using lossless atomic UPSERTs.
    #[tracing::instrument(skip(self), name = "security::budget_flush")]
    pub async fn flush_to_db(&self) -> Result<()> {
        let default_agent_usd = std::env::var("DEFAULT_AGENT_BUDGET_USD")
            .ok()
            .and_then(|v| v.parse::<f64>().ok())
            .unwrap_or(DEFAULT_AGENT_BUDGET_USD);
        let default_agent_micros = usd_to_micros(default_agent_usd).unwrap_or(500_000);

        let default_mission_usd = std::env::var("DEFAULT_MISSION_BUDGET_USD")
            .ok()
            .and_then(|v| v.parse::<f64>().ok())
            .unwrap_or(DEFAULT_MISSION_BUDGET_USD);
        let default_mission_micros = usd_to_micros(default_mission_usd).unwrap_or(5_000_000);

        let now = Utc::now();
        let next_reset = compute_next_reset(now, ResetPeriod::Daily);

        // 1. Batch Sync Agent Quotas using atomic swap + UPSERT
        let mut agent_updates = Vec::new();
        self.buffer.retain(|entity_id, cost_ref| {
            let cost = *cost_ref;
            if cost > 0 {
                agent_updates.push((entity_id.clone(), cost));
            }
            false // Evict processed and zero/negative entries
        });

        if !agent_updates.is_empty() {
            let mut tx = self.pool.begin().await?;
            for (entity_id, cost) in &agent_updates {
                let row_id = uuid::Uuid::new_v4().to_string();
                sqlx::query(
                    "INSERT INTO agent_quotas (id, entity_id, budget_usd, used_usd, reset_period, last_reset_at, next_reset_at) \
                     VALUES (?1, ?2, ?3, ?4, 'daily', ?5, ?6) \
                     ON CONFLICT(entity_id) DO UPDATE SET used_usd = agent_quotas.used_usd + excluded.used_usd",
                )
                .bind(&row_id)
                .bind(entity_id)
                .bind(default_agent_micros)
                .bind(*cost)
                .bind(now)
                .bind(next_reset)
                .execute(&mut *tx)
                .await?;
            }
            if let Err(e) = tx.commit().await {
                for (entity_id, cost) in agent_updates {
                    *self.buffer.entry(entity_id).or_insert(0) += cost;
                }
                return Err(e.into());
            }
        }

        // 2. Batch Sync Mission Quotas using atomic swap + UPSERT
        let mut mission_updates = Vec::new();
        self.mission_buffer.retain(|cluster_id, cost_ref| {
            let cost = *cost_ref;
            if cost > 0 {
                mission_updates.push((cluster_id.clone(), cost));
            }
            false // Evict processed and zero/negative entries
        });

        if !mission_updates.is_empty() {
            let mut tx = self.pool.begin().await?;
            for (cluster_id, cost) in &mission_updates {
                let row_id = uuid::Uuid::new_v4().to_string();
                sqlx::query(
                    "INSERT INTO mission_quotas (id, cluster_id, budget_usd, used_usd, reset_period, last_reset_at, next_reset_at) \
                     VALUES (?1, ?2, ?3, ?4, 'daily', ?5, ?6) \
                     ON CONFLICT(cluster_id) DO UPDATE SET used_usd = mission_quotas.used_usd + excluded.used_usd",
                )
                .bind(&row_id)
                .bind(cluster_id)
                .bind(default_mission_micros)
                .bind(*cost)
                .bind(now)
                .bind(next_reset)
                .execute(&mut *tx)
                .await?;
            }
            if let Err(e) = tx.commit().await {
                for (cluster_id, cost) in mission_updates {
                    *self.mission_buffer.entry(cluster_id).or_insert(0) += cost;
                }
                return Err(e.into());
            }
        }

        Ok(())
    }

    /// Fetches all registered agent quotas.
    pub async fn get_all_quotas(&self) -> Result<Vec<Quota>> {
        let rows = sqlx::query("SELECT * FROM agent_quotas ORDER BY entity_id ASC")
            .fetch_all(&self.pool)
            .await?;

        let mut results = Vec::new();
        for r in rows {
            use sqlx::Row;
            let period_str: String = r.get("reset_period");
            let period = match period_str.as_str() {
                "daily" => ResetPeriod::Daily,
                "monthly" => ResetPeriod::Monthly,
                _ => ResetPeriod::Never,
            };

            let budget_micros: i64 = r.get("budget_usd");
            let used_micros: i64 = r.get("used_usd");

            results.push(Quota {
                id: r.get("id"),
                entity_id: r.get("entity_id"),
                budget_usd: micros_to_usd(budget_micros),
                used_usd: micros_to_usd(used_micros),
                reset_period: period,
                last_reset_at: r.get("last_reset_at"),
                next_reset_at: r.get("next_reset_at"),
            });
        }
        Ok(results)
    }

    /// Updates an entity's quota and recomputes the reset deadline if period changes.
    #[tracing::instrument(skip(self), fields(entity_id = %entity_id, budget = budget_usd), name = "security::budget_update")]
    pub async fn update_quota(
        &self,
        entity_id: &str,
        budget_usd: f64,
        reset_period: Option<ResetPeriod>,
    ) -> Result<()> {
        let budget_micros = usd_to_micros(budget_usd)?;
        let now = Utc::now();

        if let Some(period) = reset_period {
            let period_str = match period {
                ResetPeriod::Daily => "daily",
                ResetPeriod::Monthly => "monthly",
                ResetPeriod::Never => "never",
            };
            let next_reset = compute_next_reset(now, period);
            let row_id = uuid::Uuid::new_v4().to_string();

            sqlx::query(
                "INSERT INTO agent_quotas (id, entity_id, budget_usd, used_usd, reset_period, last_reset_at, next_reset_at) \
                 VALUES (?1, ?2, ?3, 0, ?4, ?5, ?6) \
                 ON CONFLICT(entity_id) DO UPDATE SET budget_usd = ?3, reset_period = ?4, next_reset_at = ?6",
            )
            .bind(&row_id)
            .bind(entity_id)
            .bind(budget_micros)
            .bind(period_str)
            .bind(now)
            .bind(next_reset)
            .execute(&self.pool)
            .await?;
        } else {
            sqlx::query("UPDATE agent_quotas SET budget_usd = ?1 WHERE entity_id = ?2")
                .bind(budget_micros)
                .bind(entity_id)
                .execute(&self.pool)
                .await?;
        }
        Ok(())
    }

    /// Updates a mission's quota and recomputes the reset deadline.
    pub async fn update_mission_quota(
        &self,
        cluster_id: &str,
        budget_usd: f64,
        reset_period: Option<ResetPeriod>,
    ) -> Result<()> {
        let budget_micros = usd_to_micros(budget_usd)?;
        let period = reset_period.unwrap_or(ResetPeriod::Daily);
        let period_str = match period {
            ResetPeriod::Daily => "daily",
            ResetPeriod::Monthly => "monthly",
            ResetPeriod::Never => "never",
        };
        let now = Utc::now();
        let next_reset = compute_next_reset(now, period);

        sqlx::query(
            "INSERT INTO mission_quotas (id, cluster_id, budget_usd, used_usd, reset_period, last_reset_at, next_reset_at) \
             VALUES (?1, ?2, ?3, 0, ?4, ?5, ?6) \
             ON CONFLICT(cluster_id) DO UPDATE SET budget_usd = ?3, reset_period = ?4, next_reset_at = ?6",
        )
        .bind(uuid::Uuid::new_v4().to_string())
        .bind(cluster_id)
        .bind(budget_micros)
        .bind(period_str)
        .bind(now)
        .bind(next_reset)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// Mock budget guard for tests
    pub fn mock() -> Self {
        Self {
            pool: sqlx::SqlitePool::connect_lazy("sqlite::memory:").expect("Tadpole OS Metering: Failed to connect to in-memory database for mock BudgetGuard."),
            buffer: DashMap::new(),
            mission_buffer: DashMap::new(),
            reservations: DashMap::new(),
            mission_reservations: DashMap::new(),
            system_monitor: Arc::new(crate::security::monitoring::SecurityMonitor::new()),
        }
    }

    /// Shared helper to fetch a quota row from either `agent_quotas` or `mission_quotas`.
    async fn fetch_quota(pool: &SqlitePool, target: QuotaTarget, id_val: &str) -> Result<Quota> {
        let (query_str, id_col) = match target {
            QuotaTarget::Agent => (
                "SELECT id, entity_id, budget_usd, used_usd, reset_period, last_reset_at, next_reset_at FROM agent_quotas WHERE entity_id = ?1",
                "entity_id",
            ),
            QuotaTarget::Mission => (
                "SELECT id, cluster_id, budget_usd, used_usd, reset_period, last_reset_at, next_reset_at FROM mission_quotas WHERE cluster_id = ?1",
                "cluster_id",
            ),
        };

        let r = sqlx::query(query_str).bind(id_val).fetch_one(pool).await?;

        use sqlx::Row;
        let period_str: String = r.get("reset_period");
        let period = match period_str.as_str() {
            "daily" => ResetPeriod::Daily,
            "monthly" => ResetPeriod::Monthly,
            _ => ResetPeriod::Never,
        };

        let budget_micros: i64 = r.get("budget_usd");
        let used_micros: i64 = r.get("used_usd");

        Ok(Quota {
            id: r.get("id"),
            entity_id: r.get(id_col),
            budget_usd: micros_to_usd(budget_micros),
            used_usd: micros_to_usd(used_micros),
            reset_period: period,
            last_reset_at: r.get("last_reset_at"),
            next_reset_at: r.get("next_reset_at"),
        })
    }

    async fn get_or_create_quota(&self, entity_id: &str) -> Result<Quota> {
        let id = uuid::Uuid::new_v4().to_string();
        let now = Utc::now();
        let next_reset = compute_next_reset(now, ResetPeriod::Daily);

        let default_agent_usd = std::env::var("DEFAULT_AGENT_BUDGET_USD")
            .ok()
            .and_then(|v| v.parse::<f64>().ok())
            .unwrap_or(DEFAULT_AGENT_BUDGET_USD);
        let default_agent_micros = usd_to_micros(default_agent_usd).unwrap_or(500_000);

        sqlx::query(
            "INSERT OR IGNORE INTO agent_quotas (id, entity_id, budget_usd, used_usd, reset_period, last_reset_at, next_reset_at) \
             VALUES (?1, ?2, ?3, 0, 'daily', ?4, ?5)",
        )
        .bind(&id)
        .bind(entity_id)
        .bind(default_agent_micros)
        .bind(now)
        .bind(next_reset)
        .execute(&self.pool)
        .await?;

        Self::fetch_quota(&self.pool, QuotaTarget::Agent, entity_id).await
    }

    async fn reset_quota(&self, id: &str, period: ResetPeriod) -> Result<()> {
        let now = Utc::now();
        let next_reset = compute_next_reset(now, period);

        sqlx::query(
            "UPDATE agent_quotas SET used_usd = 0, last_reset_at = ?1, next_reset_at = ?2 WHERE id = ?3 AND next_reset_at <= ?4",
        )
        .bind(now)
        .bind(next_reset)
        .bind(id)
        .bind(now)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn get_or_create_mission_quota(&self, cluster_id: &str) -> Result<Quota> {
        let id = uuid::Uuid::new_v4().to_string();
        let now = Utc::now();
        let next_reset = compute_next_reset(now, ResetPeriod::Daily);

        let default_mission_usd = std::env::var("DEFAULT_MISSION_BUDGET_USD")
            .ok()
            .and_then(|v| v.parse::<f64>().ok())
            .unwrap_or(DEFAULT_MISSION_BUDGET_USD);
        let default_mission_micros = usd_to_micros(default_mission_usd).unwrap_or(5_000_000);

        sqlx::query(
            "INSERT OR IGNORE INTO mission_quotas (id, cluster_id, budget_usd, used_usd, reset_period, last_reset_at, next_reset_at) \
             VALUES (?1, ?2, ?3, 0, 'daily', ?4, ?5)",
        )
        .bind(&id)
        .bind(cluster_id)
        .bind(default_mission_micros)
        .bind(now)
        .bind(next_reset)
        .execute(&self.pool)
        .await?;

        Self::fetch_quota(&self.pool, QuotaTarget::Mission, cluster_id).await
    }

    async fn reset_mission_quota(&self, id: &str, period: ResetPeriod) -> Result<()> {
        let now = Utc::now();
        let next_reset = compute_next_reset(now, period);

        sqlx::query(
            "UPDATE mission_quotas SET used_usd = 0, last_reset_at = ?1, next_reset_at = ?2 WHERE id = ?3 AND next_reset_at <= ?4",
        )
        .bind(now)
        .bind(next_reset)
        .bind(id)
        .bind(now)
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}

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

        pool
    }

    #[tokio::test]
    async fn test_quota_enforcement() -> Result<()> {
        let pool = setup_test_db().await;
        let monitor = Arc::new(SecurityMonitor::new());
        let guard = BudgetGuard::new(pool, monitor);

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
        let monitor = Arc::new(SecurityMonitor::new());
        let guard = BudgetGuard::new(pool, monitor);

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
    async fn test_check_then_act_reservation_bounds_concurrency() -> Result<()> {
        let pool = setup_test_db().await;
        let monitor = Arc::new(SecurityMonitor::new());
        let guard = BudgetGuard::new(pool, monitor);

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
        let monitor = Arc::new(SecurityMonitor::new());
        let guard = BudgetGuard::new(pool, monitor);

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
        let monitor = Arc::new(SecurityMonitor::new());
        let guard = BudgetGuard::new(pool, monitor);

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
        let monitor = Arc::new(SecurityMonitor::new());
        let guard = BudgetGuard::new(pool, monitor);

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

//! Resource Consumption Metering & Quota Enforcement
//!
//! Orchestrates the high-fidelity tracking of USD consumption across agents
//! and missions, enforcing hard-budget boundaries to prevent runaway costs.
//!
//! @docs ARCHITECTURE:ResourceMetering
//!
//! ### AI Assist Note
//! **Resource Metering & Quota Enforcement**: Orchestrates the
//! high-fidelity tracking of **USD Consumption** across agents and
//! missions. Enforces **Sovereign Budget Boundaries** by checking
//! available credits before permitting LLM inference. features
//! **Debounced Persistence**: usage is recorded in high-speed
//! thread-safe buffers (`DashMap`) and flushed asynchronously to
//! SQLite (`agent_quotas`, `mission_quotas`) via the `flush_to_db`
//! background loop to minimize write contention (MET-01).
//!
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: Budget exhaustion causing 429 errors,
//!   flush-to-db latency causing temporary metric discrepancies,
//!   or incorrect USD-to-token conversions for local models.
//! - **Telemetry Link**: Search for `[Security]` or `[BudgetGuard]` in tracing logs.
//! - **Trace Scope**: `server-rs::security::metering`

use anyhow::Result;
use chrono::{DateTime, Utc};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use std::sync::Arc;

/// Defines the resource allocation and consumption limits for an entity (agent or user).
///
/// Tadpole OS uses a prepaid-style credit system where `budget_usd` represents the
/// maximum allowed cost before the entity is throttled or paused.
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

/// A security guard responsible for enforcing LLM cost constraints.
///
/// BudgetGuard prevents model extraction attacks or runaway loops by checking
/// quotas before permitting LLM calls. Actual costs are recorded post-execution
/// into the SQLite persistence layer to ensure high-fidelity financial governance.
/// A security guard responsible for enforcing LLM cost constraints.
///
/// BudgetGuard prevents model extraction attacks or runaway loops by checking
/// quotas before permitting LLM calls. Actual costs are recorded post-execution
/// into the SQLite persistence layer to ensure high-fidelity financial governance.
pub struct BudgetGuard {
    /// Database pool for persistence of usage metrics.
    pool: SqlitePool,
    /// Thread-safe buffer for debounced agent usage updates in i64 micro-dollars (entity_id -> accumulated_micros).
    buffer: DashMap<String, i64>,
    /// Thread-safe buffer for debounced mission usage updates in i64 micro-dollars (cluster_id -> accumulated_micros).
    mission_buffer: DashMap<String, i64>,
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
            system_monitor,
        }
    }

    /// Returns true if there is any pending cost > 0 inside the buffers.
    pub fn has_pending_usage(&self) -> bool {
        self.buffer.iter().any(|entry| *entry.value() > 0)
            || self.mission_buffer.iter().any(|entry| *entry.value() > 0)
    }

    /// Verifies if an entity has sufficient remaining budget to perform a task.
    ///
    /// If the reset time has passed, this method triggers an automatic reset
    /// before performing the check. Returns `true` if (used + estimated) <= budget.
    #[tracing::instrument(skip(self), fields(entity_id = %entity_id, cost = estimated_cost), name = "security::budget_check")]
    pub async fn check_budget(&self, entity_id: &str, estimated_cost: f64) -> Result<bool> {
        let memory_pressure = self.system_monitor.get_cached_memory_pressure();
        let multiplier = if memory_pressure > 0.85 {
            tracing::warn!(
                "⚠️ [Resource Guardian] High memory pressure ({:.1}%) detected during budget check. Enforcing strict budget multiplier (2.0x).",
                memory_pressure * 100.0
            );
            2.0
        } else {
            1.0
        };

        let estimated_micros = (estimated_cost * multiplier * 1_000_000.0).round() as i64;
        let quota = self.get_or_create_quota(entity_id).await?;
        let buffered_micros = self.buffer.get(entity_id).map(|v| *v.value()).unwrap_or(0);

        let used_micros = (quota.used_usd * 1_000_000.0).round() as i64;
        let budget_micros = (quota.budget_usd * 1_000_000.0).round() as i64;

        // Auto-reset if needed
        if Utc::now() >= quota.next_reset_at && quota.reset_period != ResetPeriod::Never {
            self.reset_quota(&quota.id, quota.reset_period).await?;
            // Refresh quota after reset
            let refreshed = self.get_or_create_quota(entity_id).await?;
            let refreshed_used_micros = (refreshed.used_usd * 1_000_000.0).round() as i64;
            let refreshed_budget_micros = (refreshed.budget_usd * 1_000_000.0).round() as i64;
            return Ok(refreshed_used_micros + buffered_micros + estimated_micros
                <= refreshed_budget_micros);
        }

        Ok(used_micros + buffered_micros + estimated_micros <= budget_micros)
    }

    /// Records actual cost after an operation completes (Debounced).
    #[tracing::instrument(skip(self), fields(entity_id = %entity_id, cost = cost_usd), name = "security::budget_record")]
    pub async fn record_usage(&self, entity_id: &str, cost_usd: f64) -> Result<()> {
        let cost_micros = (cost_usd * 1_000_000.0).round() as i64;
        *self.buffer.entry(entity_id.to_string()).or_insert(0) += cost_micros;
        Ok(())
    }

    /// Checks if a mission (cluster) has sufficient remaining budget.
    #[allow(dead_code)]
    pub async fn check_mission_budget(
        &self,
        cluster_id: &str,
        estimated_cost: f64,
    ) -> Result<bool> {
        let estimated_micros = (estimated_cost * 1_000_000.0).round() as i64;
        let quota = self.get_or_create_mission_quota(cluster_id).await?;
        let buffered_micros = self
            .mission_buffer
            .get(cluster_id)
            .map(|v| *v.value())
            .unwrap_or(0);

        let used_micros = (quota.used_usd * 1_000_000.0).round() as i64;
        let budget_micros = (quota.budget_usd * 1_000_000.0).round() as i64;

        // Auto-reset if needed
        if Utc::now() >= quota.next_reset_at && quota.reset_period != ResetPeriod::Never {
            self.reset_mission_quota(&quota.id, quota.reset_period)
                .await?;
            let refreshed = self.get_or_create_mission_quota(cluster_id).await?;
            let refreshed_used_micros = (refreshed.used_usd * 1_000_000.0).round() as i64;
            let refreshed_budget_micros = (refreshed.budget_usd * 1_000_000.0).round() as i64;
            return Ok(refreshed_used_micros + buffered_micros + estimated_micros
                <= refreshed_budget_micros);
        }

        Ok(used_micros + buffered_micros + estimated_micros <= budget_micros)
    }

    /// Records usage for a specific mission (Debounced).
    #[allow(dead_code)]
    pub async fn record_mission_usage(&self, cluster_id: &str, cost_usd: f64) -> Result<()> {
        let cost_micros = (cost_usd * 1_000_000.0).round() as i64;
        *self
            .mission_buffer
            .entry(cluster_id.to_string())
            .or_insert(0) += cost_micros;
        Ok(())
    }

    /// Flushes all buffered usage metrics to the database.
    ///
    /// This method should be called by a background loop (e.g., every 5-10 seconds)
    /// to ensure eventual consistency while minimizing DB write contention.
    #[tracing::instrument(skip(self), name = "security::budget_flush")]
    pub async fn flush_to_db(&self) -> Result<()> {
        // 1. Batch Sync Agent Quotas using an atomic swap pattern
        let mut agent_updates = Vec::new();
        self.buffer.retain(|entity_id, cost_ref| {
            let cost = *cost_ref;
            if cost > 0 {
                agent_updates.push((entity_id.clone(), cost));
                false // Atomically remove from buffer
            } else {
                true
            }
        });

        if !agent_updates.is_empty() {
            let mut tx = self.pool.begin().await?;
            for (entity_id, cost) in &agent_updates {
                sqlx::query(
                    "UPDATE agent_quotas SET used_usd = used_usd + ?1 WHERE entity_id = ?2",
                )
                .bind(*cost)
                .bind(entity_id)
                .execute(&mut *tx)
                .await?;
            }
            if let Err(e) = tx.commit().await {
                // Transaction failed, restore the accumulated costs back into the buffer
                for (entity_id, cost) in agent_updates {
                    *self.buffer.entry(entity_id).or_insert(0) += cost;
                }
                return Err(e.into());
            }
        }

        // 2. Batch Sync Mission Quotas using an atomic swap pattern
        let mut mission_updates = Vec::new();
        self.mission_buffer.retain(|cluster_id, cost_ref| {
            let cost = *cost_ref;
            if cost > 0 {
                mission_updates.push((cluster_id.clone(), cost));
                false // Atomically remove from buffer
            } else {
                true
            }
        });

        if !mission_updates.is_empty() {
            let mut tx = self.pool.begin().await?;
            for (cluster_id, cost) in &mission_updates {
                sqlx::query(
                    "UPDATE mission_quotas SET used_usd = used_usd + ?1 WHERE cluster_id = ?2",
                )
                .bind(*cost)
                .bind(cluster_id)
                .execute(&mut *tx)
                .await?;
            }
            if let Err(e) = tx.commit().await {
                // Transaction failed, restore the accumulated costs back into the buffer
                for (cluster_id, cost) in mission_updates {
                    *self.mission_buffer.entry(cluster_id).or_insert(0) += cost;
                }
                return Err(e.into());
            }
        }

        Ok(())
    }

    /// Fetches all registered quotas.
    #[allow(dead_code)]
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
                budget_usd: (budget_micros as f64) / 1_000_000.0,
                used_usd: (used_micros as f64) / 1_000_000.0,
                reset_period: period,
                last_reset_at: r.get("last_reset_at"),
                next_reset_at: r.get("next_reset_at"),
            });
        }
        Ok(results)
    }

    /// Updates an entity's quota.
    #[allow(dead_code)]
    #[tracing::instrument(skip(self), fields(entity_id = %entity_id, budget = budget_usd), name = "security::budget_update")]
    pub async fn update_quota(
        &self,
        entity_id: &str,
        budget_usd: f64,
        reset_period: Option<ResetPeriod>,
    ) -> Result<()> {
        let budget_micros = (budget_usd * 1_000_000.0).round() as i64;
        if let Some(period) = reset_period {
            let period_str = match period {
                ResetPeriod::Daily => "daily",
                ResetPeriod::Monthly => "monthly",
                ResetPeriod::Never => "never",
            };

            sqlx::query(
                "UPDATE agent_quotas SET budget_usd = ?1, reset_period = ?2 WHERE entity_id = ?3",
            )
            .bind(budget_micros)
            .bind(period_str)
            .bind(entity_id)
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

    /// Updates a mission's quota.
    #[allow(dead_code)]
    pub async fn update_mission_quota(
        &self,
        cluster_id: &str,
        budget_usd: f64,
        reset_period: Option<ResetPeriod>,
    ) -> Result<()> {
        let period_str = match reset_period.unwrap_or(ResetPeriod::Daily) {
            ResetPeriod::Daily => "daily",
            ResetPeriod::Monthly => "monthly",
            ResetPeriod::Never => "never",
        };

        let budget_micros = (budget_usd * 1_000_000.0).round() as i64;

        sqlx::query(
            "INSERT INTO mission_quotas (id, cluster_id, budget_usd, used_usd, reset_period, last_reset_at, next_reset_at) \
             VALUES (?1, ?2, ?3, 0, ?4, ?5, ?6) \
             ON CONFLICT(cluster_id) DO UPDATE SET budget_usd = ?3, reset_period = ?4"
        )
        .bind(uuid::Uuid::new_v4().to_string())
        .bind(cluster_id)
        .bind(budget_micros)
        .bind(period_str)
        .bind(Utc::now())
        .bind(Utc::now() + chrono::Duration::days(1))
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
            system_monitor: Arc::new(crate::security::monitoring::SecurityMonitor::new()),
        }
    }

    /// Shared helper to fetch a quota row from either `agent_quotas` or `mission_quotas`.
    /// The `id_col` parameter specifies the lookup column name (e.g. "entity_id" or "cluster_id").
    async fn fetch_quota(
        pool: &SqlitePool,
        table: &str,
        id_col: &str,
        id_val: &str,
    ) -> Result<Quota> {
        let r = match (table, id_col) {
            ("agent_quotas", "entity_id") => {
                sqlx::query("SELECT * FROM agent_quotas WHERE entity_id = ?1")
                    .bind(id_val)
                    .fetch_one(pool)
                    .await?
            }
            ("mission_quotas", "cluster_id") => {
                sqlx::query("SELECT * FROM mission_quotas WHERE cluster_id = ?1")
                    .bind(id_val)
                    .fetch_one(pool)
                    .await?
            }
            _ => return Err(anyhow::anyhow!("Unknown quota table: {}.{}", table, id_col)),
        };

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
            budget_usd: (budget_micros as f64) / 1_000_000.0,
            used_usd: (used_micros as f64) / 1_000_000.0,
            reset_period: period,
            last_reset_at: r.get("last_reset_at"),
            next_reset_at: r.get("next_reset_at"),
        })
    }

    async fn get_or_create_quota(&self, entity_id: &str) -> Result<Quota> {
        let id = uuid::Uuid::new_v4().to_string();
        let now = Utc::now();
        let next_reset = now + chrono::Duration::days(1);

        let default_agent_usd = std::env::var("DEFAULT_AGENT_BUDGET_USD")
            .ok()
            .and_then(|v| v.parse::<f64>().ok())
            .unwrap_or(0.50);
        let default_agent_micros = (default_agent_usd * 1_000_000.0).round() as i64;

        sqlx::query(
            "INSERT OR IGNORE INTO agent_quotas (id, entity_id, budget_usd, used_usd, reset_period, last_reset_at, next_reset_at) \
             VALUES (?1, ?2, ?3, 0, 'daily', ?4, ?5)"
        )
        .bind(&id)
        .bind(entity_id)
        .bind(default_agent_micros)
        .bind(now)
        .bind(next_reset)
        .execute(&self.pool)
        .await?;

        Self::fetch_quota(&self.pool, "agent_quotas", "entity_id", entity_id).await
    }

    async fn reset_quota(&self, id: &str, period: ResetPeriod) -> Result<()> {
        let now = Utc::now();
        let next_reset = match period {
            ResetPeriod::Daily => now + chrono::Duration::days(1),
            ResetPeriod::Monthly => now + chrono::Duration::days(30),
            ResetPeriod::Never => now + chrono::Duration::days(365 * 100),
        };

        sqlx::query(
            "UPDATE agent_quotas SET used_usd = 0, last_reset_at = ?1, next_reset_at = ?2 WHERE id = ?3 AND next_reset_at <= ?4"
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
        let next_reset = now + chrono::Duration::days(1);

        let default_mission_usd = std::env::var("DEFAULT_MISSION_BUDGET_USD")
            .ok()
            .and_then(|v| v.parse::<f64>().ok())
            .unwrap_or(5.00);
        let default_mission_micros = (default_mission_usd * 1_000_000.0).round() as i64;

        sqlx::query(
            "INSERT OR IGNORE INTO mission_quotas (id, cluster_id, budget_usd, used_usd, reset_period, last_reset_at, next_reset_at) \
             VALUES (?1, ?2, ?3, 0, 'daily', ?4, ?5)"
        )
        .bind(&id)
        .bind(cluster_id)
        .bind(default_mission_micros)
        .bind(now)
        .bind(next_reset)
        .execute(&self.pool)
        .await?;

        Self::fetch_quota(&self.pool, "mission_quotas", "cluster_id", cluster_id).await
    }

    async fn reset_mission_quota(&self, id: &str, period: ResetPeriod) -> Result<()> {
        let now = Utc::now();
        let next_reset = match period {
            ResetPeriod::Daily => now + chrono::Duration::days(1),
            ResetPeriod::Monthly => now + chrono::Duration::days(30),
            ResetPeriod::Never => now + chrono::Duration::days(365 * 100),
        };

        sqlx::query(
            "UPDATE mission_quotas SET used_usd = 0, last_reset_at = ?1, next_reset_at = ?2 WHERE id = ?3 AND next_reset_at <= ?4"
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

    #[tokio::test]
    async fn test_quota_enforcement() -> Result<()> {
        let pool = SqlitePool::connect("sqlite::memory:").await?;
        sqlx::query("CREATE TABLE agent_quotas (id TEXT PRIMARY KEY, entity_id TEXT, used_usd INTEGER, budget_usd INTEGER, last_reset_at TEXT, next_reset_at TEXT, reset_period TEXT)")
            .execute(&pool).await?;

        let monitor = Arc::new(SecurityMonitor::new());
        let guard = BudgetGuard::new(pool, monitor);

        // 1. Add agent with 1.0 budget manually (1_000_000 micros)
        sqlx::query("INSERT INTO agent_quotas (id, entity_id, used_usd, budget_usd, reset_period, last_reset_at, next_reset_at) VALUES ('1', 'agent_1', 0, 1000000, 'Never', '2024-01-01 00:00:00', '2124-01-01 00:00:00')")
            .execute(&guard.pool).await?;

        assert!(guard.check_budget("agent_1", 0.01).await?);

        // 3. Record usage nearly hitting budget
        guard.record_usage("agent_1", 0.90).await?;
        assert!(guard.check_budget("agent_1", 0.05).await?);

        // 4. Hit budget
        guard.record_usage("agent_1", 0.10).await?; // Total used: 1.0 (1_000_000 micros)
        assert!(!guard.check_budget("agent_1", 0.01).await?); // 1.01 > 1.0

        Ok(())
    }

    #[tokio::test]
    async fn test_debounced_flushing() -> Result<()> {
        let pool = SqlitePool::connect("sqlite::memory:").await?;
        sqlx::query("CREATE TABLE agent_quotas (id TEXT PRIMARY KEY, entity_id TEXT, used_usd INTEGER, budget_usd INTEGER, last_reset_at TEXT, next_reset_at TEXT, reset_period TEXT)")
            .execute(&pool).await?;
        sqlx::query("CREATE TABLE mission_quotas (id TEXT PRIMARY KEY, cluster_id TEXT, used_usd INTEGER, budget_usd INTEGER, last_reset_at TEXT, next_reset_at TEXT, reset_period TEXT)")
            .execute(&pool).await?;

        let monitor = Arc::new(SecurityMonitor::new());
        let guard = BudgetGuard::new(pool, monitor);

        // 1. Setup agent & mission
        sqlx::query("INSERT INTO agent_quotas (id, entity_id, used_usd, budget_usd, reset_period, last_reset_at, next_reset_at) VALUES ('1', 'agent_flush', 0, 10000000, 'Never', '2024-01-01 00:00:00', '2124-01-01 00:00:00')")
            .execute(&guard.pool).await?;
        sqlx::query("INSERT INTO mission_quotas (id, cluster_id, used_usd, budget_usd, reset_period, last_reset_at, next_reset_at) VALUES ('2', 'cluster_flush', 0, 10000000, 'Never', '2024-01-01 00:00:00', '2124-01-01 00:00:00')")
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
}

// Metadata: [metering]

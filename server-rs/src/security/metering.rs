use anyhow::Result;
use chrono::{DateTime, Utc};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

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
pub struct BudgetGuard {
    /// Database pool for persistence of usage metrics.
    pool: SqlitePool,
    /// Thread-safe buffer for debounced agent usage updates (entity_id -> accumulated_cost).
    buffer: DashMap<String, f64>,
    /// Thread-safe buffer for debounced mission usage updates (cluster_id -> accumulated_cost).
    mission_buffer: DashMap<String, f64>,
}

impl BudgetGuard {
    pub fn new(pool: SqlitePool) -> Self {
        Self {
            pool,
            buffer: DashMap::new(),
            mission_buffer: DashMap::new(),
        }
    }

    /// Verifies if an entity has sufficient remaining budget to perform a task.
    ///
    /// If the reset time has passed, this method triggers an automatic reset
    /// before performing the check. Returns `true` if (used + estimated) <= budget.
    pub async fn check_budget(&self, entity_id: &str, estimated_cost: f64) -> Result<bool> {
        let quota = self.get_or_create_quota(entity_id).await?;
        let buffered = self
            .buffer
            .get(entity_id)
            .map(|v| *v.value())
            .unwrap_or(0.0);

        // Auto-reset if needed
        if Utc::now() >= quota.next_reset_at && quota.reset_period != ResetPeriod::Never {
            self.reset_quota(&quota.id, quota.reset_period).await?;
            // Refresh quota after reset
            let refreshed = self.get_or_create_quota(entity_id).await?;
            return Ok(refreshed.used_usd + buffered + estimated_cost <= refreshed.budget_usd);
        }

        Ok(quota.used_usd + buffered + estimated_cost <= quota.budget_usd)
    }

    /// Records actual cost after an operation completes (Debounced).
    pub async fn record_usage(&self, entity_id: &str, cost_usd: f64) -> Result<()> {
        *self.buffer.entry(entity_id.to_string()).or_insert(0.0) += cost_usd;
        Ok(())
    }

    /// Checks if a mission (cluster) has sufficient remaining budget.
    #[allow(dead_code)]
    pub async fn check_mission_budget(
        &self,
        cluster_id: &str,
        estimated_cost: f64,
    ) -> Result<bool> {
        let quota = self.get_or_create_mission_quota(cluster_id).await?;
        let buffered = self
            .mission_buffer
            .get(cluster_id)
            .map(|v| *v.value())
            .unwrap_or(0.0);

        // Auto-reset if needed
        if Utc::now() >= quota.next_reset_at && quota.reset_period != ResetPeriod::Never {
            self.reset_mission_quota(&quota.id, quota.reset_period)
                .await?;
            let refreshed = self.get_or_create_mission_quota(cluster_id).await?;
            return Ok(refreshed.used_usd + buffered + estimated_cost <= refreshed.budget_usd);
        }

        Ok(quota.used_usd + buffered + estimated_cost <= quota.budget_usd)
    }

    /// Records usage for a specific mission (Debounced).
    #[allow(dead_code)]
    pub async fn record_mission_usage(&self, cluster_id: &str, cost_usd: f64) -> Result<()> {
        *self
            .mission_buffer
            .entry(cluster_id.to_string())
            .or_insert(0.0) += cost_usd;
        Ok(())
    }

    /// Flushes all buffered usage metrics to the database.
    ///
    /// This method should be called by a background loop (e.g., every 5-10 seconds)
    /// to ensure eventual consistency while minimizing DB write contention.
    pub async fn flush_to_db(&self) -> Result<()> {
        // 1. Batch Sync Agent Quotas
        let mut agent_updates = Vec::new();
        // Use a loop to drain or reset entries to avoid double-processing
        for mut entry in self.buffer.iter_mut() {
            let cost = *entry.value();
            if cost > 0.0 {
                agent_updates.push((entry.key().clone(), cost));
                *entry.value_mut() = 0.0;
            }
        }

        if !agent_updates.is_empty() {
            let mut tx = self.pool.begin().await?;
            for (entity_id, cost) in agent_updates {
                sqlx::query(
                    "UPDATE agent_quotas SET used_usd = used_usd + ?1 WHERE entity_id = ?2",
                )
                .bind(cost)
                .bind(entity_id)
                .execute(&mut *tx)
                .await?;
            }
            tx.commit().await?;
        }

        // 2. Batch Sync Mission Quotas
        let mut mission_updates = Vec::new();
        for mut entry in self.mission_buffer.iter_mut() {
            let cost = *entry.value();
            if cost > 0.0 {
                mission_updates.push((entry.key().clone(), cost));
                *entry.value_mut() = 0.0;
            }
        }

        if !mission_updates.is_empty() {
            let mut tx = self.pool.begin().await?;
            for (cluster_id, cost) in mission_updates {
                sqlx::query(
                    "UPDATE mission_quotas SET used_usd = used_usd + ?1 WHERE cluster_id = ?2",
                )
                .bind(cost)
                .bind(cluster_id)
                .execute(&mut *tx)
                .await?;
            }
            tx.commit().await?;
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

            results.push(Quota {
                id: r.get("id"),
                entity_id: r.get("entity_id"),
                budget_usd: r.get("budget_usd"),
                used_usd: r.get("used_usd"),
                reset_period: period,
                last_reset_at: r.get("last_reset_at"),
                next_reset_at: r.get("next_reset_at"),
            });
        }
        Ok(results)
    }

    /// Updates an entity's quota.
    #[allow(dead_code)]
    pub async fn update_quota(
        &self,
        entity_id: &str,
        budget_usd: f64,
        reset_period: Option<ResetPeriod>,
    ) -> Result<()> {
        if let Some(period) = reset_period {
            let period_str = match period {
                ResetPeriod::Daily => "daily",
                ResetPeriod::Monthly => "monthly",
                ResetPeriod::Never => "never",
            };

            sqlx::query(
                "UPDATE agent_quotas SET budget_usd = ?1, reset_period = ?2 WHERE entity_id = ?3",
            )
            .bind(budget_usd)
            .bind(period_str)
            .bind(entity_id)
            .execute(&self.pool)
            .await?;
        } else {
            sqlx::query("UPDATE agent_quotas SET budget_usd = ?1 WHERE entity_id = ?2")
                .bind(budget_usd)
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

        sqlx::query(
            "INSERT INTO mission_quotas (id, cluster_id, budget_usd, used_usd, reset_period, last_reset_at, next_reset_at) \
             VALUES (?1, ?2, ?3, 0.0, ?4, ?5, ?6) \
             ON CONFLICT(cluster_id) DO UPDATE SET budget_usd = ?3, reset_period = ?4"
        )
        .bind(uuid::Uuid::new_v4().to_string())
        .bind(cluster_id)
        .bind(budget_usd)
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
            pool: sqlx::SqlitePool::connect_lazy("sqlite::memory:").unwrap(),
            buffer: DashMap::new(),
            mission_buffer: DashMap::new(),
        }
    }

    async fn get_or_create_quota(&self, entity_id: &str) -> Result<Quota> {
        let row = sqlx::query("SELECT * FROM agent_quotas WHERE entity_id = ?1")
            .bind(entity_id)
            .fetch_optional(&self.pool)
            .await?;

        if let Some(r) = row {
            use sqlx::Row;
            let period_str: String = r.get("reset_period");
            let period = match period_str.as_str() {
                "daily" => ResetPeriod::Daily,
                "monthly" => ResetPeriod::Monthly,
                _ => ResetPeriod::Never,
            };

            Ok(Quota {
                id: r.get("id"),
                entity_id: r.get("entity_id"),
                budget_usd: r.get("budget_usd"),
                used_usd: r.get("used_usd"),
                reset_period: period,
                last_reset_at: r.get("last_reset_at"),
                next_reset_at: r.get("next_reset_at"),
            })
        } else {
            // Create default daily quota ($0.50)
            let id = uuid::Uuid::new_v4().to_string();
            let now = Utc::now();
            let next_reset = now + chrono::Duration::days(1);

            sqlx::query(
                "INSERT INTO agent_quotas (id, entity_id, budget_usd, used_usd, reset_period, last_reset_at, next_reset_at) \
                 VALUES (?1, ?2, 0.50, 0.0, 'daily', ?3, ?4)"
            )
            .bind(&id)
            .bind(entity_id)
            .bind(now)
            .bind(next_reset)
            .execute(&self.pool)
            .await?;

            Ok(Quota {
                id,
                entity_id: entity_id.to_string(),
                budget_usd: 0.50,
                used_usd: 0.0,
                reset_period: ResetPeriod::Daily,
                last_reset_at: now,
                next_reset_at: next_reset,
            })
        }
    }

    async fn reset_quota(&self, id: &str, period: ResetPeriod) -> Result<()> {
        let now = Utc::now();
        let next_reset = match period {
            ResetPeriod::Daily => now + chrono::Duration::days(1),
            ResetPeriod::Monthly => now + chrono::Duration::days(30),
            ResetPeriod::Never => now + chrono::Duration::days(365 * 100),
        };

        sqlx::query(
            "UPDATE agent_quotas SET used_usd = 0.0, last_reset_at = ?1, next_reset_at = ?2 WHERE id = ?3"
        )
        .bind(now)
        .bind(next_reset)
        .bind(id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn get_or_create_mission_quota(&self, cluster_id: &str) -> Result<Quota> {
        let row = sqlx::query("SELECT * FROM mission_quotas WHERE cluster_id = ?1")
            .bind(cluster_id)
            .fetch_optional(&self.pool)
            .await?;

        if let Some(r) = row {
            use sqlx::Row;
            let period_str: String = r.get("reset_period");
            let period = match period_str.as_str() {
                "daily" => ResetPeriod::Daily,
                "monthly" => ResetPeriod::Monthly,
                _ => ResetPeriod::Never,
            };

            Ok(Quota {
                id: r.get("id"),
                entity_id: r.get("cluster_id"),
                budget_usd: r.get("budget_usd"),
                used_usd: r.get("used_usd"),
                reset_period: period,
                last_reset_at: r.get("last_reset_at"),
                next_reset_at: r.get("next_reset_at"),
            })
        } else {
            let id = uuid::Uuid::new_v4().to_string();
            let now = Utc::now();
            let next_reset = now + chrono::Duration::days(1);

            sqlx::query(
                "INSERT INTO mission_quotas (id, cluster_id, budget_usd, used_usd, reset_period, last_reset_at, next_reset_at) \
                 VALUES (?1, ?2, 5.0, 0.0, 'daily', ?3, ?4)"
            )
            .bind(&id)
            .bind(cluster_id)
            .bind(now)
            .bind(next_reset)
            .execute(&self.pool)
            .await?;

            Ok(Quota {
                id,
                entity_id: cluster_id.to_string(),
                budget_usd: 5.0,
                used_usd: 0.0,
                reset_period: ResetPeriod::Daily,
                last_reset_at: now,
                next_reset_at: next_reset,
            })
        }
    }

    async fn reset_mission_quota(&self, id: &str, period: ResetPeriod) -> Result<()> {
        let now = Utc::now();
        let next_reset = match period {
            ResetPeriod::Daily => now + chrono::Duration::days(1),
            ResetPeriod::Monthly => now + chrono::Duration::days(30),
            ResetPeriod::Never => now + chrono::Duration::days(365 * 100),
        };

        sqlx::query(
            "UPDATE mission_quotas SET used_usd = 0.0, last_reset_at = ?1, next_reset_at = ?2 WHERE id = ?3"
        )
        .bind(now)
        .bind(next_reset)
        .bind(id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::SqlitePool;

    #[tokio::test]
    async fn test_quota_enforcement() -> Result<()> {
        let pool = SqlitePool::connect("sqlite::memory:").await?;
        sqlx::query("CREATE TABLE agent_quotas (id TEXT PRIMARY KEY, entity_id TEXT, used_usd REAL, budget_usd REAL, last_reset_at TEXT, next_reset_at TEXT, reset_period TEXT)")
            .execute(&pool).await?;

        let guard = BudgetGuard::new(pool);

        // 1. Add agent with 1.0 budget manually
        sqlx::query("INSERT INTO agent_quotas (id, entity_id, used_usd, budget_usd, reset_period, last_reset_at, next_reset_at) VALUES ('1', 'agent_1', 0.0, 1.0, 'Never', '2024-01-01 00:00:00', '2124-01-01 00:00:00')")
            .execute(&guard.pool).await?;

        assert!(guard.check_budget("agent_1", 0.01).await?);

        // 3. Record usage nearly hitting budget
        guard.record_usage("agent_1", 0.90).await?;
        assert!(guard.check_budget("agent_1", 0.05).await?);

        // 4. Hit budget
        guard.record_usage("agent_1", 0.10).await?; // Total used: 1.0
        assert!(!guard.check_budget("agent_1", 0.01).await?); // 1.01 > 1.0

        Ok(())
    }

    #[tokio::test]
    async fn test_debounced_flushing() -> Result<()> {
        let pool = SqlitePool::connect("sqlite::memory:").await?;
        sqlx::query("CREATE TABLE agent_quotas (id TEXT PRIMARY KEY, entity_id TEXT, used_usd REAL, budget_usd REAL, last_reset_at TEXT, next_reset_at TEXT, reset_period TEXT)")
            .execute(&pool).await?;
        sqlx::query("CREATE TABLE mission_quotas (id TEXT PRIMARY KEY, cluster_id TEXT, used_usd REAL, budget_usd REAL, last_reset_at TEXT, next_reset_at TEXT, reset_period TEXT)")
            .execute(&pool).await?;

        let guard = BudgetGuard::new(pool);

        // 1. Setup agent & mission
        sqlx::query("INSERT INTO agent_quotas (id, entity_id, used_usd, budget_usd, reset_period, last_reset_at, next_reset_at) VALUES ('1', 'agent_flush', 0.0, 10.0, 'Never', '2024-01-01 00:00:00', '2124-01-01 00:00:00')")
            .execute(&guard.pool).await?;
        sqlx::query("INSERT INTO mission_quotas (id, cluster_id, used_usd, budget_usd, reset_period, last_reset_at, next_reset_at) VALUES ('2', 'cluster_flush', 0.0, 10.0, 'Never', '2024-01-01 00:00:00', '2124-01-01 00:00:00')")
            .execute(&guard.pool).await?;

        // 2. Record usage (should only hit buffers)
        guard.record_usage("agent_flush", 1.5).await?;
        guard.record_usage("agent_flush", 2.5).await?;
        guard.record_mission_usage("cluster_flush", 5.0).await?;

        // Verify DB hasn't changed yet
        let agent_used: f64 =
            sqlx::query_scalar("SELECT used_usd FROM agent_quotas WHERE entity_id = 'agent_flush'")
                .fetch_one(&guard.pool)
                .await?;
        let mission_used: f64 = sqlx::query_scalar(
            "SELECT used_usd FROM mission_quotas WHERE cluster_id = 'cluster_flush'",
        )
        .fetch_one(&guard.pool)
        .await?;
        assert_eq!(agent_used, 0.0);
        assert_eq!(mission_used, 0.0);

        // 3. Flush
        guard.flush_to_db().await?;

        // Verify DB updated
        let agent_used_after: f64 =
            sqlx::query_scalar("SELECT used_usd FROM agent_quotas WHERE entity_id = 'agent_flush'")
                .fetch_one(&guard.pool)
                .await?;
        let mission_used_after: f64 = sqlx::query_scalar(
            "SELECT used_usd FROM mission_quotas WHERE cluster_id = 'cluster_flush'",
        )
        .fetch_one(&guard.pool)
        .await?;

        assert_eq!(agent_used_after, 4.0);
        assert_eq!(mission_used_after, 5.0);

        // Verify buffers are reset
        assert_eq!(*guard.buffer.get("agent_flush").unwrap(), 0.0);
        assert_eq!(*guard.mission_buffer.get("cluster_flush").unwrap(), 0.0);

        Ok(())
    }
}

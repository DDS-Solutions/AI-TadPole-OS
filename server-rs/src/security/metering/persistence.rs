//! @docs ARCHITECTURE:ResourceMetering
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Security & Governance / budget accounting
//!
//! ### AI Assist Note
//! - Keep budget reservations, usage accounting, and durable quota updates separately reviewable.
//!
//! ### 🔍 Debugging & Observability
//! - **Telemetry Targets**: none declared

use super::*;

impl BudgetGuard {
    /// Flushes all buffered usage metrics to the database using lossless atomic UPSERTs.
    #[tracing::instrument(skip(self), name = "security::budget_flush")]
    pub async fn flush_to_db(self: &Arc<Self>) -> Result<()> {
        let owned = Arc::clone(self);
        tokio::spawn(async move { owned.flush_owned().await })
            .await
            .map_err(|error| anyhow!("Budget flush task failed: {}", error))?
    }

    async fn flush_owned(&self) -> Result<()> {
        let _flush = self.flush_lock.lock().await;
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

        // Snapshot without eviction: every error leaves all charges available to retry.
        let agent_updates: Vec<_> = self
            .buffer
            .iter()
            .filter(|entry| *entry.value() > 0)
            .map(|entry| (entry.key().clone(), *entry.value()))
            .collect();
        let mission_updates: Vec<_> = self
            .mission_buffer
            .iter()
            .filter(|entry| *entry.value() > 0)
            .map(|entry| (entry.key().clone(), *entry.value()))
            .collect();
        if agent_updates.is_empty() && mission_updates.is_empty() {
            return Ok(());
        }
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
        // Dispatch settlement uses this same transaction for the mission spend ledger.
        for (mission_id, cost) in &mission_updates {
            sqlx::query("UPDATE mission_history SET cost_usd = cost_usd + ?1 WHERE id = ?2")
                .bind(micros_to_usd(*cost))
                .bind(mission_id)
                .execute(&mut *tx)
                .await?;
        }
        tx.commit().await?;
        #[cfg(test)]
        {
            let pause = self.flush_commit_pause.lock().take();
            if let Some((committed, release)) = pause {
                committed.notify_one();
                release.notified().await;
            }
        }
        let _accounting = self.accounting_gate.lock();
        for (id, amount) in agent_updates {
            self.buffer
                .entry(id.clone())
                .and_modify(|cost| *cost = cost.saturating_sub(amount));
            self.buffer.remove_if(&id, |_, cost| *cost == 0);
        }
        for (id, amount) in mission_updates {
            self.mission_buffer
                .entry(id.clone())
                .and_modify(|cost| *cost = cost.saturating_sub(amount));
            self.mission_buffer.remove_if(&id, |_, cost| *cost == 0);
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
            flush_lock: tokio::sync::Mutex::new(()),
            accounting_gate: parking_lot::Mutex::new(()),
            #[cfg(test)]
            flush_commit_pause: parking_lot::Mutex::new(None),
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

    pub(super) async fn get_or_create_quota(&self, entity_id: &str) -> Result<Quota> {
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

    pub(super) async fn reset_quota(&self, id: &str, period: ResetPeriod) -> Result<()> {
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

    pub(super) async fn get_or_create_mission_quota(&self, cluster_id: &str) -> Result<Quota> {
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

    pub(super) async fn reset_mission_quota(&self, id: &str, period: ResetPeriod) -> Result<()> {
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

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
        let _flush = self.flush_lock.lock().await;
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
        let _accounting = self.accounting_gate.lock();
        let buffered_micros = self.buffer.get(entity_id).map(|v| *v.value()).unwrap_or(0);
        let used_and_buffered = used_micros.saturating_add(buffered_micros);

        let allowed = Self::check_and_add_reservation(
            &self.reservations,
            entity_id,
            estimated_micros,
            used_and_buffered,
            budget_micros,
            None,
        );

        Ok(allowed)
    }

    /// Records actual cost after an operation completes and settles in-flight reservations.
    #[tracing::instrument(skip(self), fields(entity_id = %entity_id, cost = cost_usd), name = "security::budget_record")]
    pub async fn record_usage(&self, entity_id: &str, cost_usd: f64) -> Result<()> {
        let cost_micros = usd_to_micros(cost_usd)?;
        let _accounting = self.accounting_gate.lock();
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
        let _flush = self.flush_lock.lock().await;
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
        let _accounting = self.accounting_gate.lock();
        let buffered_micros = self
            .mission_buffer
            .get(cluster_id)
            .map(|v| *v.value())
            .unwrap_or(0);
        let used_and_buffered = used_micros.saturating_add(buffered_micros);

        let allowed = Self::check_and_add_reservation(
            &self.mission_reservations,
            cluster_id,
            estimated_micros,
            used_and_buffered,
            budget_micros,
            None,
        );

        Ok(allowed)
    }

    /// Records usage for a specific mission and settles in-flight reservations.
    #[allow(dead_code)]
    pub async fn record_mission_usage(&self, cluster_id: &str, cost_usd: f64) -> Result<()> {
        let cost_micros = usd_to_micros(cost_usd)?;
        let _accounting = self.accounting_gate.lock();
        if cost_micros > 0 {
            *self
                .mission_buffer
                .entry(cluster_id.to_string())
                .or_insert(0) += cost_micros;
        }
        Self::release_reservation(&self.mission_reservations, cluster_id);
        Ok(())
    }
}

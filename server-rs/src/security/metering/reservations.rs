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

impl CallBudgetReservation {
    pub(crate) fn settle(&mut self, cost_usd: f64) -> Result<()> {
        let micros = usd_to_micros(cost_usd)?;
        let _accounting = self.guard.accounting_gate.lock();
        if self.settled {
            return Err(anyhow!("Provider usage was already settled"));
        }
        let agent_total = self
            .guard
            .buffer
            .get(&self.agent_id)
            .map(|entry| *entry)
            .unwrap_or(0)
            .checked_add(micros)
            .ok_or_else(|| anyhow!("Agent usage buffer overflow"))?;
        let mission_total = self
            .guard
            .mission_buffer
            .get(&self.mission_id)
            .map(|entry| *entry)
            .unwrap_or(0)
            .checked_add(micros)
            .ok_or_else(|| anyhow!("Mission usage buffer overflow"))?;
        self.guard.buffer.insert(self.agent_id.clone(), agent_total);
        self.guard
            .mission_buffer
            .insert(self.mission_id.clone(), mission_total);
        BudgetGuard::release_owned_reservation(
            &self.guard.reservations,
            &self.agent_id,
            &self.owner,
        );
        BudgetGuard::release_owned_reservation(
            &self.guard.mission_reservations,
            &self.mission_id,
            &self.owner,
        );
        self.settled = true;
        Ok(())
    }
}

impl Drop for CallBudgetReservation {
    fn drop(&mut self) {
        if !self.settled {
            let _accounting = self.guard.accounting_gate.lock();
            BudgetGuard::release_owned_reservation(
                &self.guard.reservations,
                &self.agent_id,
                &self.owner,
            );
            BudgetGuard::release_owned_reservation(
                &self.guard.mission_reservations,
                &self.mission_id,
                &self.owner,
            );
        }
    }
}

impl BudgetGuard {
    #[allow(dead_code)]
    pub(super) fn get_active_reservation(map: &Reservations, id: &str) -> i64 {
        let now = Utc::now();
        if let Some(entry) = map.get(id) {
            entry
                .value()
                .iter()
                .filter(|(_, created_at, owner)| {
                    owner.is_some() || (now - *created_at).num_seconds() <= RESERVATION_TTL_SECS
                })
                .map(|(amount, _, _)| *amount)
                .fold(0i64, |acc, x| acc.saturating_add(x))
        } else {
            0
        }
    }

    #[allow(dead_code)]
    fn add_reservation(map: &Reservations, id: &str, amount: i64) {
        let now = Utc::now();
        let mut entry = map.entry(id.to_string()).or_default();
        entry.retain(|(_, created_at, owner)| {
            owner.is_some() || (now - *created_at).num_seconds() <= RESERVATION_TTL_SECS
        });
        entry.push_back((amount, now, None));
    }

    /// Atomically evaluates remaining budget and acquires an in-flight reservation within a single entry lock.
    pub(super) fn check_and_add_reservation(
        map: &Reservations,
        id: &str,
        estimated_micros: i64,
        used_and_buffered_micros: i64,
        budget_micros: i64,
        owner: Option<&str>,
    ) -> bool {
        let now = Utc::now();
        let mut entry = map.entry(id.to_string()).or_default();
        entry.retain(|(_, created_at, owner)| {
            owner.is_some() || (now - *created_at).num_seconds() <= RESERVATION_TTL_SECS
        });
        let active_reserved = entry
            .iter()
            .map(|(amount, _, _)| *amount)
            .fold(0i64, |acc, x| acc.saturating_add(x));

        let total_anticipated = used_and_buffered_micros
            .saturating_add(active_reserved)
            .saturating_add(estimated_micros);

        if total_anticipated <= budget_micros {
            if estimated_micros > 0 || owner.is_some() {
                entry.push_back((estimated_micros, now, owner.map(str::to_string)));
            }
            true
        } else {
            false
        }
    }

    pub(super) fn release_reservation(map: &Reservations, id: &str) {
        let now = Utc::now();
        if let Some(mut entry) = map.get_mut(id) {
            entry.retain(|(_, created_at, owner)| {
                owner.is_some() || (now - *created_at).num_seconds() <= RESERVATION_TTL_SECS
            });
            if let Some(index) = entry.iter().position(|(_, _, owner)| owner.is_none()) {
                entry.remove(index);
            }
            if entry.is_empty() {
                drop(entry);
                map.remove_if(id, |_, entry| entry.is_empty());
            }
        }
    }

    fn release_owned_reservation(map: &Reservations, id: &str, owner: &str) {
        if let Some(mut entry) = map.get_mut(id) {
            entry.retain(|(_, _, entry_owner)| entry_owner.as_deref() != Some(owner));
        }
        map.remove_if(id, |_, entry| entry.is_empty());
    }

    pub(crate) async fn reserve_call(
        self: &Arc<Self>,
        agent_id: &str,
        mission_id: &str,
        estimated_cost: f64,
        mission_limit: f64,
    ) -> Result<Option<CallBudgetReservation>> {
        let _flush = self.flush_lock.lock().await;
        let multiplier =
            if self.system_monitor.get_cached_memory_pressure() > HIGH_MEMORY_PRESSURE_THRESHOLD {
                HIGH_MEMORY_PRESSURE_MULTIPLIER
            } else {
                1.0
            };
        let estimate = usd_to_micros(estimated_cost * multiplier)?;
        let mut agent = self.get_or_create_quota(agent_id).await?;
        let mut mission = self.get_or_create_mission_quota(mission_id).await?;
        if Utc::now() >= agent.next_reset_at && agent.reset_period != ResetPeriod::Never {
            self.reset_quota(&agent.id, agent.reset_period).await?;
            agent = self.get_or_create_quota(agent_id).await?;
        }
        if Utc::now() >= mission.next_reset_at && mission.reset_period != ResetPeriod::Never {
            self.reset_mission_quota(&mission.id, mission.reset_period)
                .await?;
            mission = self.get_or_create_mission_quota(mission_id).await?;
        }
        let agent_budget = usd_to_micros(agent.budget_usd)?;
        let declared_limit = usd_to_micros(mission_limit)?;
        let mission_budget = if declared_limit > 0 {
            usd_to_micros(mission.budget_usd)?.min(declared_limit)
        } else {
            usd_to_micros(mission.budget_usd)?
        };
        let historical_cost: Option<f64> = if mission.reset_period == ResetPeriod::Never {
            sqlx::query_scalar("SELECT cost_usd FROM mission_history WHERE id = ?")
                .bind(mission_id)
                .fetch_optional(&self.pool)
                .await?
        } else {
            None
        };
        let _accounting = self.accounting_gate.lock();
        let agent_used = usd_to_micros(agent.used_usd)?
            .saturating_add(self.buffer.get(agent_id).map(|entry| *entry).unwrap_or(0));
        let mission_used = usd_to_micros(mission.used_usd)?
            .max(usd_to_micros(historical_cost.unwrap_or(0.0))?)
            .saturating_add(
                self.mission_buffer
                    .get(mission_id)
                    .map(|entry| *entry)
                    .unwrap_or(0),
            );
        let owner = uuid::Uuid::new_v4().to_string();
        if !Self::check_and_add_reservation(
            &self.reservations,
            agent_id,
            estimate,
            agent_used,
            agent_budget,
            Some(&owner),
        ) {
            return Ok(None);
        }
        if !Self::check_and_add_reservation(
            &self.mission_reservations,
            mission_id,
            estimate,
            mission_used,
            mission_budget,
            Some(&owner),
        ) {
            Self::release_owned_reservation(&self.reservations, agent_id, &owner);
            return Ok(None);
        }
        Ok(Some(CallBudgetReservation {
            guard: Arc::clone(self),
            owner,
            agent_id: agent_id.to_string(),
            mission_id: mission_id.to_string(),
            settled: false,
        }))
    }
}

//! @docs ARCHITECTURE:Registry
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Agent Runner / a2a_router
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Type-safe state handling and bounded execution without unhandled panics.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: none declared
//! - **Witness Tests**: none declared

#![allow(dead_code)]

use super::a2a_ledger::A2ATransactionCoordinator;
use super::a2a_types::{validate_amount, Address, Amount, EconomicZone};
use crate::error::AppError;
use sqlx::SqlitePool;
use std::sync::Arc;
use std::time::Duration;

#[async_trait::async_trait]
pub trait A2APaymentAdapter: Send + Sync {
    async fn transfer_funds(
        &self,
        pool: &SqlitePool,
        from: &Address,
        to: &Address,
        amount: Amount,
        signing_key: Option<&ed25519_dalek::SigningKey>,
    ) -> Result<String, AppError>;
}

/// LocalMockAdapter: Settles payments instantly in the local database.
pub struct LocalMockAdapter;

#[async_trait::async_trait]
impl A2APaymentAdapter for LocalMockAdapter {
    async fn transfer_funds(
        &self,
        pool: &SqlitePool,
        from: &Address,
        to: &Address,
        amount: Amount,
        signing_key: Option<&ed25519_dalek::SigningKey>,
    ) -> Result<String, AppError> {
        validate_amount(amount)?;
        let mut tx = pool.begin().await?;

        // 1. Prepare (Locks funds/assets)
        let lock_id = A2ATransactionCoordinator::prepare_transaction(
            &mut tx, from, to, amount, None, None, None,
        )
        .await?;

        // 2. Commit (Instantly credits recipient)
        let tx_id =
            A2ATransactionCoordinator::commit_transaction(&mut tx, &lock_id, signing_key).await?;

        tx.commit().await?;
        Ok(tx_id)
    }
}

/// L3HybridAdapter: Settle payments locally, but execute real transfers for external exits.
pub struct L3HybridAdapter {
    rpc_url: String,
    vault_address: String,
    http_client: reqwest::Client,
}

impl L3HybridAdapter {
    pub fn new(rpc_url: String, vault_address: String) -> Self {
        let http_client = reqwest::Client::builder()
            .timeout(Duration::from_secs(15))
            .connect_timeout(Duration::from_secs(5))
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());

        Self {
            rpc_url,
            vault_address,
            http_client,
        }
    }
}

#[async_trait::async_trait]
impl A2APaymentAdapter for L3HybridAdapter {
    async fn transfer_funds(
        &self,
        pool: &SqlitePool,
        from: &Address,
        to: &Address,
        amount: Amount,
        signing_key: Option<&ed25519_dalek::SigningKey>,
    ) -> Result<String, AppError> {
        validate_amount(amount)?;
        let mut tx = pool.begin().await?;

        // 1. Prepare
        let lock_id = A2ATransactionCoordinator::prepare_transaction(
            &mut tx, from, to, amount, None, None, None,
        )
        .await?;

        // 2. Determine if recipient is an external Web3 Address
        if let Address::Web3(ref wallet_addr) = to {
            let is_simulated = self.rpc_url.is_empty()
                || self.rpc_url.starts_with("mock://")
                || self.rpc_url.starts_with("sim://");

            if is_simulated {
                tracing::info!(
                    "🌐 [Web3 Exit Simulation] Simulated on-chain USDC transfer from vault {} to {} for amount: {} micros (RPC: {})",
                    self.vault_address,
                    wallet_addr,
                    amount,
                    if self.rpc_url.is_empty() { "sim://local" } else { &self.rpc_url }
                );

                // Commit local ledger transfer and return deterministic simulation tx hash
                let tx_id =
                    A2ATransactionCoordinator::commit_transaction(&mut tx, &lock_id, signing_key)
                        .await?;
                tx.commit().await?;
                return Ok(format!("0xsim_{}", tx_id));
            }

            // Live external RPC: fail closed if no real EVM raw transaction signer bridge is connected
            tracing::warn!(
                "🌐 [Web3 Exit] Live on-chain transfer requested for vault {} to {} via RPC: {} but automated raw transaction signing is unconfigured",
                self.vault_address,
                wallet_addr,
                self.rpc_url
            );

            let _ = A2ATransactionCoordinator::rollback_transaction(&mut tx, &lock_id).await;
            let _ = tx.commit().await;
            return Err(AppError::BadRequest(
                "Live on-chain Web3 exit requires an authentic signed EVM raw transaction. Configure a simulation RPC ('sim://' or 'mock://') or execute through a verified EVM signer bridge.".to_string(),
            ));
        }

        // 3. Commit
        let tx_id =
            A2ATransactionCoordinator::commit_transaction(&mut tx, &lock_id, signing_key).await?;

        tx.commit().await?;
        Ok(tx_id)
    }
}

/// Default daily spend limit for unconfigured agents (10 USDC = 10,000,000 micros).
pub const DEFAULT_UNCONFIGURED_DAILY_SPEND_LIMIT: i64 = 10_000_000;

/// PaymentRouter: Handles Dev, Staging, and Prod zones and checks daily spend caps.
pub struct PaymentRouter {
    pool: SqlitePool,
    dev_adapter: Arc<dyn A2APaymentAdapter>,
    staging_adapter: Arc<dyn A2APaymentAdapter>,
    prod_adapter: Arc<dyn A2APaymentAdapter>,
}

impl PaymentRouter {
    pub fn new(
        pool: SqlitePool,
        dev_adapter: Arc<dyn A2APaymentAdapter>,
        staging_adapter: Arc<dyn A2APaymentAdapter>,
        prod_adapter: Arc<dyn A2APaymentAdapter>,
    ) -> Self {
        Self {
            pool,
            dev_adapter,
            staging_adapter,
            prod_adapter,
        }
    }

    /// Checks if a proposed transfer from an agent respects their daily spend limit.
    pub async fn check_limit(
        &self,
        agent_id: &str,
        amount: Amount,
    ) -> Result<EconomicZone, AppError> {
        validate_amount(amount)?;
        let meta_opt: Option<(String, i64, i64, i64)> = sqlx::query_as(
            "SELECT economic_zone, daily_spend_limit, daily_spent_accumulated, last_reset_at \
             FROM agent_economics_meta WHERE agent_id = ?1",
        )
        .bind(agent_id)
        .fetch_optional(&self.pool)
        .await?;

        let (zone_str, limit, mut spent, last_reset) = meta_opt.unwrap_or_else(|| {
            (
                "DEV".to_string(),
                DEFAULT_UNCONFIGURED_DAILY_SPEND_LIMIT,
                0,
                0,
            )
        });

        let zone = EconomicZone::parse(&zone_str);

        // Perform daily limit reset if 24 hours have passed
        let now = chrono::Utc::now().timestamp();
        if now > last_reset + 86400 {
            spent = 0;
            sqlx::query(
                "UPDATE agent_economics_meta \
                 SET daily_spent_accumulated = 0, last_reset_at = ?1 \
                 WHERE agent_id = ?2",
            )
            .bind(now)
            .bind(agent_id)
            .execute(&self.pool)
            .await?;
        }

        // Query sum of all locked transaction amounts for this buyer (local:buyer)
        let locked_sum: (i64,) = sqlx::query_as(
            "SELECT COALESCE(SUM(locked_amount), 0) FROM transaction_locks WHERE buyer_id = ?1",
        )
        .bind(format!("local:{}", agent_id))
        .fetch_one(&self.pool)
        .await?;

        let total_projected_spent = spent + locked_sum.0 + amount as i64;

        // Verify budget caps
        if limit > 0 && total_projected_spent > limit {
            return Err(AppError::BadRequest(format!(
                "Transaction rejected: agent {} has exceeded its daily limit cap of {} micro-USDC (spent today: {}, pending locks: {}).",
                agent_id, limit, spent, locked_sum.0
            )));
        }

        Ok(zone)
    }

    /// Increments the daily spent accumulation for an agent atomically in SQL.
    pub async fn accumulate_spend(&self, agent_id: &str, amount: Amount) -> Result<(), AppError> {
        validate_amount(amount)?;

        let result = sqlx::query(
            "UPDATE agent_economics_meta \
             SET daily_spent_accumulated = daily_spent_accumulated + ?1 \
             WHERE agent_id = ?2 AND (daily_spend_limit = 0 OR daily_spent_accumulated + ?1 <= daily_spend_limit)",
        )
        .bind(amount as i64)
        .bind(agent_id)
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            // Check if agent exists or limit was exceeded
            let exists: Option<(i64, i64)> = sqlx::query_as(
                "SELECT daily_spend_limit, daily_spent_accumulated FROM agent_economics_meta WHERE agent_id = ?1",
            )
            .bind(agent_id)
            .fetch_optional(&self.pool)
            .await?;

            if let Some((limit, spent)) = exists {
                if limit > 0 && (spent + amount as i64) > limit {
                    return Err(AppError::BadRequest(format!(
                        "Spend accumulation failed: agent {} would exceed daily limit cap of {} (current: {}, attempted: {})",
                        agent_id, limit, spent, amount
                    )));
                }
            } else {
                if (amount as i64) > DEFAULT_UNCONFIGURED_DAILY_SPEND_LIMIT {
                    return Err(AppError::BadRequest(format!(
                        "Spend accumulation failed: agent {} would exceed default daily limit cap of {} (attempted: {})",
                        agent_id, DEFAULT_UNCONFIGURED_DAILY_SPEND_LIMIT, amount
                    )));
                }

                // Initialize default record for agent
                let now = chrono::Utc::now().timestamp();
                sqlx::query(
                    "INSERT INTO agent_economics_meta (agent_id, economic_zone, daily_spend_limit, daily_spent_accumulated, last_reset_at) \
                     VALUES (?1, 'DEV', ?2, ?3, ?4) \
                     ON CONFLICT(agent_id) DO UPDATE SET daily_spent_accumulated = daily_spent_accumulated + excluded.daily_spent_accumulated",
                )
                .bind(agent_id)
                .bind(DEFAULT_UNCONFIGURED_DAILY_SPEND_LIMIT)
                .bind(amount as i64)
                .bind(now)
                .execute(&self.pool)
                .await?;
            }
        }

        Ok(())
    }

    /// Primary entrypoint to dispatch transactions, checking daily spend caps and atomically accumulating spend upfront.
    pub async fn transfer(
        &self,
        agent_id: &str,
        to: &Address,
        amount: Amount,
        signing_key: Option<&ed25519_dalek::SigningKey>,
    ) -> Result<String, AppError> {
        validate_amount(amount)?;
        let from_addr = Address::Local(agent_id.to_string());

        // 1. Check Economic Zone and Daily Limit Caps (including pending locks)
        let zone = self.check_limit(agent_id, amount).await?;

        // 2. Atomically accumulate daily spend upfront prior to dispatch
        self.accumulate_spend(agent_id, amount).await?;

        // 3. Route to appropriate adapter with rollback on failure
        let tx_res = match zone {
            EconomicZone::Dev => {
                self.dev_adapter
                    .transfer_funds(&self.pool, &from_addr, to, amount, signing_key)
                    .await
            }
            EconomicZone::Staging => {
                self.staging_adapter
                    .transfer_funds(&self.pool, &from_addr, to, amount, signing_key)
                    .await
            }
            EconomicZone::Prod => {
                self.prod_adapter
                    .transfer_funds(&self.pool, &from_addr, to, amount, signing_key)
                    .await
            }
        };

        match tx_res {
            Ok(tx_id) => Ok(tx_id),
            Err(e) => {
                // Roll back the accumulated spend on adapter failure
                let _ = sqlx::query(
                    "UPDATE agent_economics_meta \
                     SET daily_spent_accumulated = MAX(0, daily_spent_accumulated - ?1) \
                     WHERE agent_id = ?2",
                )
                .bind(amount as i64)
                .bind(agent_id)
                .execute(&self.pool)
                .await;
                Err(e)
            }
        }
    }
}

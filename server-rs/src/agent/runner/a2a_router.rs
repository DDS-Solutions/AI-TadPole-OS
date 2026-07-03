//! @docs ARCHITECTURE:Registry
//! @docs ARCHITECTURE:Runner
//! 
//! ### AI Assist Note
//! **A2A Payment Router & Adaptability Bridge**
//! This module implements high-fidelity logic for the Sovereign Reality layer.
//! 
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: Runtime logic error, state desynchronization, or resource exhaustion.
//! - **Telemetry Link**: Search `[a2a_router]` in tracing logs.

#![allow(dead_code)]

use crate::error::AppError;
use super::a2a_types::{Address, Amount, EconomicZone};
use super::a2a_ledger::A2ATransactionCoordinator;
use sqlx::SqlitePool;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

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
        let mut tx = pool.begin().await?;

        // 1. Prepare (Locks funds/assets)
        let lock_id = A2ATransactionCoordinator::prepare_transaction(
            &mut tx,
            from,
            to,
            amount,
            None,
            None,
            None,
        )
        .await?;

        // 2. Commit (Instantly credits recipient)
        let tx_id = A2ATransactionCoordinator::commit_transaction(
            &mut tx,
            &lock_id,
            signing_key,
        )
        .await?;

        tx.commit().await?;
        Ok(tx_id)
    }
}

/// L3HybridAdapter: Settle payments locally, but execute real transfers for external exits.
pub struct L3HybridAdapter {
    rpc_url: String,
    vault_address: String,
}

impl L3HybridAdapter {
    pub fn new(rpc_url: String, vault_address: String) -> Self {
        Self {
            rpc_url,
            vault_address,
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
        let mut tx = pool.begin().await?;

        // 1. Prepare
        let lock_id = A2ATransactionCoordinator::prepare_transaction(
            &mut tx,
            from,
            to,
            amount,
            None,
            None,
            None,
        )
        .await?;

        // 2. Determine if recipient is an external Web3 Address
        if let Address::Web3(ref wallet_addr) = to {
            // Trigger blockchain exit flow (mocking Web3 CDP SDK invocation)
            tracing::info!(
                "🌐 [Web3 Exit] Broadcasting on-chain USDC transfer from vault {} to {} via RPC: {} for amount: {} micros",
                self.vault_address,
                wallet_addr,
                self.rpc_url,
                amount
            );
            // In production, this initiates CDP Wallet transfers or smart contract execution.
        }

        // 3. Commit
        let tx_id = A2ATransactionCoordinator::commit_transaction(
            &mut tx,
            &lock_id,
            signing_key,
        )
        .await?;

        tx.commit().await?;
        Ok(tx_id)
    }
}

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

    /// Primary entrypoint to dispatch transactions, checking daily spend caps first.
    pub async fn transfer(
        &self,
        agent_id: &str,
        to: &Address,
        amount: Amount,
        signing_key: Option<&ed25519_dalek::SigningKey>,
    ) -> Result<String, AppError> {
        let from_addr = Address::Local(agent_id.to_string());

        // 1. Check Economic Zone and Daily Limit Caps
        let meta_opt: Option<(String, i64, i64, i64)> = sqlx::query_as(
            "SELECT economic_zone, daily_spend_limit, daily_spent_accumulated, last_reset_at \
             FROM agent_economics_meta WHERE agent_id = ?1",
        )
        .bind(agent_id)
        .fetch_optional(&self.pool)
        .await?;

        let (zone_str, limit, mut spent, last_reset) = meta_opt.unwrap_or_else(|| {
            ("DEV".to_string(), 0, 0, 0)
        });

        let zone = EconomicZone::parse(&zone_str);

        // 2. Perform daily limit reset if 24 hours have passed
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;
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

        // 3. Verify budget caps
        if limit > 0 && (spent + amount as i64) > limit {
            return Err(AppError::BadRequest(format!(
                "Transaction rejected: agent {} has exceeded its daily limit cap of {} micro-USDC (spent today: {}).",
                agent_id, limit, spent
            )));
        }

        // 4. Route to appropriate adapter first
        let tx_id = match zone {
            EconomicZone::Dev => {
                self.dev_adapter
                    .transfer_funds(&self.pool, &from_addr, to, amount, signing_key)
                    .await?
            }
            EconomicZone::Staging => {
                self.staging_adapter
                    .transfer_funds(&self.pool, &from_addr, to, amount, signing_key)
                    .await?
            }
            EconomicZone::Prod => {
                self.prod_adapter
                    .transfer_funds(&self.pool, &from_addr, to, amount, signing_key)
                    .await?
            }
        };

        // 5. Update daily spent accumulation only if transfer was successful
        if limit > 0 {
            sqlx::query(
                "UPDATE agent_economics_meta \
                 SET daily_spent_accumulated = ?1 \
                 WHERE agent_id = ?2",
            )
            .bind(spent + amount as i64)
            .bind(agent_id)
            .execute(&self.pool)
            .await?;
        }

        Ok(tx_id)
    }
}

// Metadata: [a2a_router]

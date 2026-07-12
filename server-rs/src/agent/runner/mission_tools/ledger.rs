//! @docs ARCHITECTURE:Registry
//! @docs ARCHITECTURE:Runner
//!
//! ### AI Assist Note
//! **Economic Ledger Tools**: Double-entry agent-to-agent transactions (2PC Saga), asset wallet actions, and x402 verification challenge signing.
//! This module implements high-fidelity logic for the Sovereign Reality layer.
//!
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: Runtime logic error, state desynchronization, or resource exhaustion.
//! - **Telemetry Link**: Search `[ledger]` in tracing logs.

use super::{require_str, require_str_opt, require_u64};
use crate::agent::runner::tools::error::ToolExecutionError;
use crate::agent::runner::{AgentRunner, RunContext};
use crate::error::AppError;

impl AgentRunner {
    /// Handles `propose_asset_transaction`: initiates 2PC prepare phase.
    pub(crate) async fn handle_propose_asset_transaction(
        &self,
        ctx: &RunContext,
        fc: &crate::agent::types::ToolCall,
    ) -> Result<String, ToolExecutionError> {
        let seller_id = require_str(ctx, &fc.args, "seller_id", "propose_asset_transaction")?;
        let amount = require_u64(ctx, &fc.args, "amount", "propose_asset_transaction")?;
        let challenge_data =
            require_str_opt(ctx, &fc.args, "challenge_data", "propose_asset_transaction")?;
        let challenge_signature = require_str_opt(
            ctx,
            &fc.args,
            "challenge_signature",
            "propose_asset_transaction",
        )?;

        // 1. Enforce Economic Zone & Daily Limit Caps with Oversight Override
        if let Err(e) = self
            .state
            .resources
            .payment_router
            .check_limit(&ctx.agent_id, amount)
            .await
        {
            if std::env::var("A2A_TEST_MODE").is_ok() {
                return Err(ToolExecutionError::ExecutionFailed(format!(
                    "Daily budget limit cap exceeded (agent exceeded its daily limit cap). Original error: {}", e
                )));
            }

            // Trigger interactive oversight budget extension request
            let audit = crate::agent::types::ToolCallAudit {
                id: uuid::Uuid::new_v4().to_string(),
                mission_id: Some(ctx.mission_id.clone()),
                agent_id: ctx.agent_id.clone(),
                skill: "budget:spend_limit_override".to_string(),
                params: serde_json::json!({
                    "requested_amount": amount,
                    "reason": format!("A2A Transaction limit check failed: {}", e)
                }),
                department: "Finance".to_string(),
                description: format!(
                    "Agent {} requests daily budget extension of {} micro-USDC",
                    ctx.agent_id, amount
                ),
                timestamp: chrono::Utc::now().to_rfc3339(),
            };

            let resolution = self
                .submit_oversight_resolution(audit, Some(ctx.mission_id.clone()))
                .await?;
            if !resolution.approved {
                return Err(ToolExecutionError::ExecutionFailed(format!(
                    "Daily budget limit extension request denied (agent exceeded its daily limit cap). Original error: {}", e
                )));
            }

            // Increase daily spend limit in DB dynamically to allow the transaction to proceed
            sqlx::query(
                "INSERT INTO agent_economics_meta (agent_id, economic_zone, daily_spend_limit, daily_spent_accumulated, last_reset_at) \
                 VALUES (?1, 'DEV', ?2, 0, 0) \
                 ON CONFLICT(agent_id) DO UPDATE SET daily_spend_limit = daily_spend_limit + excluded.daily_spend_limit"
            )
            .bind(&ctx.agent_id)
            .bind(amount as i64)
            .execute(&self.state.resources.pool)
            .await
            .map_err(|db_err| ToolExecutionError::ExecutionFailed(db_err.to_string()))?;
        }

        let buyer_addr = crate::agent::runner::a2a_types::Address::Local(ctx.agent_id.clone());
        let seller_addr = crate::agent::runner::a2a_types::Address::parse(&seller_id)?;

        let mut verifying_key = None;
        let vk_key;
        if challenge_signature.is_some() {
            if let Some(signing_key) =
                crate::security::audit::load_signing_key_from_env().unwrap_or(None)
            {
                use secrecy::ExposeSecret;
                vk_key = signing_key.expose_secret().0.verifying_key();
                verifying_key = Some(vk_key);
            }
        }

        let mut tx = self.state.resources.pool.begin().await?;

        let lock_id =
            crate::agent::runner::a2a_ledger::A2ATransactionCoordinator::prepare_transaction(
                &mut tx,
                &buyer_addr,
                &seller_addr,
                amount,
                challenge_data.as_deref(),
                challenge_signature.as_deref(),
                verifying_key.as_ref(),
            )
            .await?;

        tx.commit().await?;

        Ok(format!(
            "Transaction proposal successfully prepared. Lock ID: {}. Please call 'confirm_asset_transaction' with this lock ID to finalize the transaction.",
            lock_id
        ))
    }

    /// Handles `confirm_asset_transaction`: finalizes 2PC commit.
    pub(crate) async fn handle_confirm_asset_transaction(
        &self,
        ctx: &RunContext,
        fc: &crate::agent::types::ToolCall,
    ) -> Result<String, ToolExecutionError> {
        let lock_id = require_str(ctx, &fc.args, "lock_id", "confirm_asset_transaction")?;

        // 1. Look up the lock details before committing to get the buyer ID and amount for limit tracking
        let mut tx = self.state.resources.pool.begin().await?;
        let lock_opt: Option<(String, i64)> = sqlx::query_as(
            "SELECT buyer_id, locked_amount FROM transaction_locks WHERE lock_id = ?1",
        )
        .bind(&lock_id)
        .fetch_optional(&mut *tx)
        .await?;

        let (buyer_id, locked_amount) = lock_opt
            .ok_or_else(|| AppError::NotFound(format!("Transaction lock {} not found", lock_id)))?;

        let signing_key = crate::security::audit::load_signing_key_from_env().unwrap_or(None);
        use secrecy::ExposeSecret;
        let sk_ref = signing_key.as_ref().map(|k| &k.expose_secret().0);

        let tx_id =
            crate::agent::runner::a2a_ledger::A2ATransactionCoordinator::commit_transaction(
                &mut tx, &lock_id, sk_ref,
            )
            .await?;

        tx.commit().await?;

        // 2. Accumulate the spend on success
        if let Ok(crate::agent::runner::a2a_types::Address::Local(raw_buyer_id)) =
            crate::agent::runner::a2a_types::Address::parse(&buyer_id)
        {
            self.state
                .resources
                .payment_router
                .accumulate_spend(&raw_buyer_id, locked_amount as u64)
                .await?;
        }

        Ok(format!(
            "Transaction successfully committed and signed. Transaction ID: {}.",
            tx_id
        ))
    }

    /// Handles `cancel_asset_transaction`: rolls back locked transaction.
    pub(crate) async fn handle_cancel_asset_transaction(
        &self,
        ctx: &RunContext,
        fc: &crate::agent::types::ToolCall,
    ) -> Result<String, ToolExecutionError> {
        let lock_id = require_str(ctx, &fc.args, "lock_id", "cancel_asset_transaction")?;

        let mut tx = self.state.resources.pool.begin().await?;

        crate::agent::runner::a2a_ledger::A2ATransactionCoordinator::rollback_transaction(
            &mut tx, &lock_id,
        )
        .await?;

        tx.commit().await?;

        Ok(format!(
            "Transaction lock {} successfully rolled back. Locked funds have been returned to your wallet.",
            lock_id
        ))
    }

    /// Handles `resolve_x402_challenge`: signs challenge data.
    pub(crate) async fn handle_resolve_x402_challenge(
        &self,
        ctx: &RunContext,
        fc: &crate::agent::types::ToolCall,
    ) -> Result<String, ToolExecutionError> {
        let challenge_data =
            require_str(ctx, &fc.args, "challenge_data", "resolve_x402_challenge")?;

        let signing_key =
            crate::security::audit::load_signing_key_from_env()?.ok_or_else(|| {
                AppError::BadRequest("No private key loaded for A2A x402 signing".to_string())
            })?;

        use ed25519_dalek::Signer;
        use secrecy::ExposeSecret;
        let sig = signing_key
            .expose_secret()
            .0
            .sign(challenge_data.as_bytes());
        let signature_hex = hex::encode(sig.to_bytes());

        Ok(format!(
            "challenge_data:{}\nsignature:{}",
            challenge_data, signature_hex
        ))
    }
}

// Metadata: [ledger]

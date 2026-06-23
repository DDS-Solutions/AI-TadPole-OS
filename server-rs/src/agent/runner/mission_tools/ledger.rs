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

use crate::agent::runner::{AgentRunner, RunContext};
use crate::agent::runner::tools::error::ToolExecutionError;
use crate::error::AppError;
use super::{require_str, require_str_opt, require_u64};

impl AgentRunner {
    /// Handles `propose_asset_transaction`: initiates 2PC prepare phase.
    pub(crate) async fn handle_propose_asset_transaction(
        &self,
        ctx: &RunContext,
        fc: &crate::agent::types::ToolCall,
    ) -> Result<String, ToolExecutionError> {
        let seller_id = require_str(ctx, &fc.args, "seller_id", "propose_asset_transaction")?;
        let amount = require_u64(ctx, &fc.args, "amount", "propose_asset_transaction")?;
        let challenge_data = require_str_opt(ctx, &fc.args, "challenge_data", "propose_asset_transaction")?;
        let challenge_signature = require_str_opt(ctx, &fc.args, "challenge_signature", "propose_asset_transaction")?;

        let buyer_addr = crate::agent::runner::a2a_types::Address::Local(ctx.agent_id.clone());
        let seller_addr = crate::agent::runner::a2a_types::Address::parse(&seller_id)?;

        let mut verifying_key = None;
        let vk_key;
        if challenge_signature.is_some() {
            if let Some(signing_key) = crate::security::audit::load_signing_key_from_env().unwrap_or(None) {
                use secrecy::ExposeSecret;
                vk_key = signing_key.expose_secret().0.verifying_key();
                verifying_key = Some(vk_key);
            }
        }

        let mut tx = self.state.resources.pool.begin().await?;

        let lock_id = crate::agent::runner::a2a_ledger::A2ATransactionCoordinator::prepare_transaction(
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

        let signing_key = crate::security::audit::load_signing_key_from_env().unwrap_or(None);
        use secrecy::ExposeSecret;
        let sk_ref = signing_key.as_ref().map(|k| &k.expose_secret().0);

        let mut tx = self.state.resources.pool.begin().await?;

        let tx_id = crate::agent::runner::a2a_ledger::A2ATransactionCoordinator::commit_transaction(
            &mut tx,
            &lock_id,
            sk_ref,
        )
        .await?;

        tx.commit().await?;

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
            &mut tx,
            &lock_id,
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
        let challenge_data = require_str(ctx, &fc.args, "challenge_data", "resolve_x402_challenge")?;

        let signing_key = crate::security::audit::load_signing_key_from_env()?
            .ok_or_else(|| AppError::BadRequest("No private key loaded for A2A x402 signing".to_string()))?;

        use secrecy::ExposeSecret;
        use ed25519_dalek::Signer;
        let sig = signing_key.expose_secret().0.sign(challenge_data.as_bytes());
        let signature_hex = hex::encode(sig.to_bytes());

        Ok(format!(
            "challenge_data:{}\nsignature:{}",
            challenge_data, signature_hex
        ))
    }
}

// Metadata: [ledger]

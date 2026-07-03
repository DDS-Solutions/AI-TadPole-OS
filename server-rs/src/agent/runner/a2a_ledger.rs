//! @docs ARCHITECTURE:Registry
//! @docs ARCHITECTURE:Runner
//! 
//! ### AI Assist Note
//! **A2A Ledger & Challenge Protocol Coordinator**
//! This module implements high-fidelity logic for the Sovereign Reality layer.
//! 
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: Runtime logic error, state desynchronization, or resource exhaustion.
//! - **Telemetry Link**: Search `[a2a_ledger]` in tracing logs.

use crate::error::AppError;
use super::a2a_types::{Address, Amount};
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use sqlx::{Sqlite, Transaction};
use std::time::{SystemTime, UNIX_EPOCH};

#[allow(dead_code)]
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct ChallengeHeader {
    pub invoice_id: String,
    pub amount: Amount,
    pub recipient: String,
    pub challenge_data: String,
}

pub struct A2ATransactionCoordinator;

impl A2ATransactionCoordinator {
    /// Generates a x402-compliant HTTP 402 challenge header structure.
    #[allow(dead_code)]
    pub fn generate_x402_challenge(
        recipient: &Address,
        resource_id: &str,
        amount: Amount,
    ) -> ChallengeHeader {
        let invoice_id = uuid::Uuid::new_v4().to_string();
        let challenge_data = format!(
            "invoice:{}:resource:{}:amount:{}:recipient:{}",
            invoice_id,
            resource_id,
            amount,
            recipient.to_string_repr()
        );
        ChallengeHeader {
            invoice_id,
            amount,
            recipient: recipient.to_string_repr(),
            challenge_data,
        }
    }

    /// Verifies the signature of a challenge payment.
    pub fn verify_challenge_signature(
        challenge_data: &str,
        signature_hex: &str,
        verifying_key: &VerifyingKey,
    ) -> Result<bool, AppError> {
        let signature_bytes = hex::decode(signature_hex).map_err(|e| {
            AppError::BadRequest(format!("Invalid hex signature: {}", e))
        })?;
        let signature_arr: [u8; 64] = signature_bytes.as_slice().try_into().map_err(|_| {
            AppError::BadRequest("Signature must be exactly 64 bytes".to_string())
        })?;
        let sig = Signature::from_bytes(&signature_arr);
        Ok(verifying_key.verify(challenge_data.as_bytes(), &sig).is_ok())
    }

    /// Prepare Phase: verifies buyer balance, deducts balance, locks transaction assets.
    pub async fn prepare_transaction(
        tx: &mut Transaction<'_, Sqlite>,
        buyer: &Address,
        seller: &Address,
        amount: Amount,
        challenge_data: Option<&str>,
        challenge_signature_hex: Option<&str>,
        verifying_key: Option<&VerifyingKey>,
    ) -> Result<String, AppError> {
        let buyer_str = buyer.to_string_repr();
        let seller_str = seller.to_string_repr();

        // 1. Verify signatures if challenge is provided
        if let (Some(data), Some(sig_hex), Some(vk)) = (challenge_data, challenge_signature_hex, verifying_key) {
            if !Self::verify_challenge_signature(data, sig_hex, vk)? {
                return Err(AppError::BadRequest(
                    "x402 Challenge signature verification failed".to_string(),
                ));
            }
        }

        // 2. Deduct balance from buyer atomically
        let result = sqlx::query(
            "UPDATE agent_balances SET balance = balance - ?1 \
             WHERE agent_id = ?2 AND asset_type = 'USDC' AND balance >= ?1",
        )
        .bind(amount as i64)
        .bind(&buyer_str)
        .execute(&mut **tx)
        .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::BadRequest(format!(
                "Insufficient funds or wallet does not exist for agent: {}",
                buyer_str
            )));
        }

        // 4. Create record in transaction ledger (PENDING)
        let transaction_id = uuid::Uuid::new_v4().to_string();
        sqlx::query(
            "INSERT INTO transaction_ledger (id, transaction_type, buyer_address, seller_address, amount, status) \
             VALUES (?1, 'TRANSFER', ?2, ?3, ?4, 'PENDING')",
        )
        .bind(&transaction_id)
        .bind(&buyer_str)
        .bind(&seller_str)
        .bind(amount as i64)
        .execute(&mut **tx)
        .await?;

        // 5. Create transaction lock with 30s TTL
        let lock_id = uuid::Uuid::new_v4().to_string();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let expires_at = now + 30;

        sqlx::query(
            "INSERT INTO transaction_locks (lock_id, transaction_id, buyer_id, seller_id, locked_amount, expires_at) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        )
        .bind(&lock_id)
        .bind(&transaction_id)
        .bind(&buyer_str)
        .bind(&seller_str)
        .bind(amount as i64)
        .bind(expires_at as i64)
        .execute(&mut **tx)
        .await?;

        Ok(lock_id)
    }

    /// Commit Phase: credits seller and marks ledger as COMMITTED.
    pub async fn commit_transaction(
        tx: &mut Transaction<'_, Sqlite>,
        lock_id: &str,
        signing_key: Option<&SigningKey>,
    ) -> Result<String, AppError> {
        // 1. Fetch lock
        let lock_opt: Option<(String, String, String, i64)> = sqlx::query_as(
            "SELECT transaction_id, seller_id, buyer_id, locked_amount FROM transaction_locks WHERE lock_id = ?1",
        )
        .bind(lock_id)
        .fetch_optional(&mut **tx)
        .await?;

        let (transaction_id, seller_str, _buyer_str, amount) = match lock_opt {
            Some(l) => l,
            None => return Err(AppError::NotFound(format!("Transaction lock {} not found", lock_id))),
        };

        // 2. Credit seller balance atomically using UPSERT
        sqlx::query(
            "INSERT INTO agent_balances (agent_id, asset_type, balance) \
             VALUES (?1, 'USDC', ?2) \
             ON CONFLICT(agent_id, asset_type) DO UPDATE SET balance = balance + excluded.balance",
        )
        .bind(&seller_str)
        .bind(amount)
        .execute(&mut **tx)
        .await?;

        // 3. Cryptographic Signature
        let mut audit_signature = None;
        if let Some(sk) = signing_key {
            let message = format!("{}:{}:{}", transaction_id, seller_str, amount);
            let sig = sk.sign(message.as_bytes());
            audit_signature = Some(hex::encode(sig.to_bytes()));
        }

        // 4. Update transaction status in ledger (COMMITTED)
        sqlx::query(
            "UPDATE transaction_ledger SET status = 'COMMITTED', audit_signature = ?1 WHERE id = ?2",
        )
        .bind(&audit_signature)
        .bind(&transaction_id)
        .execute(&mut **tx)
        .await?;

        // 5. Delete lock
        sqlx::query("DELETE FROM transaction_locks WHERE lock_id = ?1")
            .bind(lock_id)
            .execute(&mut **tx)
            .await?;

        Ok(transaction_id)
    }

    /// Rollback Phase: returns locked amount to buyer.
    pub async fn rollback_transaction(
        tx: &mut Transaction<'_, Sqlite>,
        lock_id: &str,
    ) -> Result<(), AppError> {
        // 1. Fetch lock
        let lock_opt: Option<(String, String, i64)> = sqlx::query_as(
            "SELECT transaction_id, buyer_id, locked_amount FROM transaction_locks WHERE lock_id = ?1",
        )
        .bind(lock_id)
        .fetch_optional(&mut **tx)
        .await?;

        let (transaction_id, buyer_str, amount) = match lock_opt {
            Some(l) => l,
            None => return Err(AppError::NotFound(format!("Transaction lock {} not found", lock_id))),
        };

        // 2. Return buyer balance atomically using UPSERT
        sqlx::query(
            "INSERT INTO agent_balances (agent_id, asset_type, balance) \
             VALUES (?1, 'USDC', ?2) \
             ON CONFLICT(agent_id, asset_type) DO UPDATE SET balance = balance + excluded.balance",
        )
        .bind(&buyer_str)
        .bind(amount)
        .execute(&mut **tx)
        .await?;

        // 3. Update transaction status in ledger (ROLLED_BACK)
        sqlx::query(
            "UPDATE transaction_ledger SET status = 'ROLLED_BACK' WHERE id = ?1",
        )
        .bind(&transaction_id)
        .execute(&mut **tx)
        .await?;

        // 4. Delete lock
        sqlx::query("DELETE FROM transaction_locks WHERE lock_id = ?1")
            .bind(lock_id)
            .execute(&mut **tx)
            .await?;

        Ok(())
    }
}

// Metadata: [a2a_ledger]

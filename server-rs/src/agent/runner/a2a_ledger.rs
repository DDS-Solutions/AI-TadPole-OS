//! @docs ARCHITECTURE:Registry
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Agent Runner / a2a_ledger
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Type-safe state handling and bounded execution without unhandled panics.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: none declared
//! - **Witness Tests**: none declared

use super::a2a_types::{validate_amount, Address, Amount};
use crate::error::AppError;
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use sqlx::{Sqlite, Transaction};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ChallengeHeader {
    pub invoice_id: String,
    pub amount: Amount,
    pub recipient: String,
    pub challenge_data: String,
}

pub struct A2ATransactionCoordinator;

impl A2ATransactionCoordinator {
    /// Generates a x402-compliant HTTP 402 challenge header structure.
    pub fn generate_x402_challenge(
        recipient: &Address,
        resource_id: &str,
        amount: Amount,
    ) -> Result<ChallengeHeader, AppError> {
        validate_amount(amount)?;
        let invoice_id = uuid::Uuid::new_v4().to_string();
        let recipient_str = recipient.to_string_repr();
        let challenge_data = format!(
            "invoice:{}:resource:{}:amount:{}:recipient:{}",
            invoice_id, resource_id, amount, recipient_str
        );
        Ok(ChallengeHeader {
            invoice_id,
            amount,
            recipient: recipient_str,
            challenge_data,
        })
    }

    /// Parses structured x402 challenge data and extracts required binding parameters.
    pub fn parse_challenge_data(data: &str) -> Result<ChallengeHeader, AppError> {
        let parts: Vec<&str> = data.split(':').collect();
        // Expected format: "invoice:<id>:resource:<res>:amount:<amt>:recipient:<recipient_str>"
        if parts.len() < 8
            || parts[0] != "invoice"
            || parts[2] != "resource"
            || parts[4] != "amount"
            || parts[6] != "recipient"
        {
            return Err(AppError::BadRequest(
                "Malformed x402 challenge data format. Expected 'invoice:<id>:resource:<res>:amount:<amt>:recipient:<rec>'".to_string(),
            ));
        }

        let invoice_id = parts[1].trim().to_string();
        if invoice_id.is_empty() {
            return Err(AppError::BadRequest(
                "Challenge data contains empty invoice ID".to_string(),
            ));
        }

        let amount: Amount = parts[5].parse().map_err(|_| {
            AppError::BadRequest("Invalid amount integer in x402 challenge data".to_string())
        })?;
        validate_amount(amount)?;

        let recipient = parts[7..].join(":");
        if recipient.is_empty() {
            return Err(AppError::BadRequest(
                "Challenge data contains empty recipient".to_string(),
            ));
        }

        Ok(ChallengeHeader {
            invoice_id,
            amount,
            recipient,
            challenge_data: data.to_string(),
        })
    }

    /// Verifies the signature of a challenge payment.
    pub fn verify_challenge_signature(
        challenge_data: &str,
        signature_hex: &str,
        verifying_key: &VerifyingKey,
    ) -> Result<bool, AppError> {
        let signature_bytes = hex::decode(signature_hex)
            .map_err(|e| AppError::BadRequest(format!("Invalid hex signature: {}", e)))?;
        let signature_arr: [u8; 64] = signature_bytes
            .as_slice()
            .try_into()
            .map_err(|_| AppError::BadRequest("Signature must be exactly 64 bytes".to_string()))?;
        let sig = Signature::from_bytes(&signature_arr);

        // Domain-separated signature verification with raw fallback for backward compatibility
        let domain_separated = format!("TADPOLE_X402_V1:{}", challenge_data);
        let valid_separated = verifying_key
            .verify(domain_separated.as_bytes(), &sig)
            .is_ok();
        let valid_raw = verifying_key
            .verify(challenge_data.as_bytes(), &sig)
            .is_ok();

        Ok(valid_separated || valid_raw)
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
        // 1. Sanity and boundary checks
        validate_amount(amount)?;
        if buyer == seller {
            return Err(AppError::BadRequest(
                "Self-transfers between identical addresses are forbidden".to_string(),
            ));
        }

        let buyer_str = buyer.to_string_repr();
        let seller_str = seller.to_string_repr();

        // 2. All-or-Error Challenge Verification with Strict Field Binding and Replay Protection
        let has_any_challenge = challenge_data.is_some()
            || challenge_signature_hex.is_some()
            || verifying_key.is_some();

        if has_any_challenge {
            let (data, sig_hex, vk) = match (challenge_data, challenge_signature_hex, verifying_key)
            {
                (Some(d), Some(s), Some(v)) => (d, s, v),
                _ => {
                    return Err(AppError::BadRequest(
                        "Incomplete x402 challenge parameters: challenge_data, challenge_signature, and verifying_key must all be provided together".to_string(),
                    ));
                }
            };

            if !Self::verify_challenge_signature(data, sig_hex, vk)? {
                return Err(AppError::BadRequest(
                    "x402 Challenge signature verification failed".to_string(),
                ));
            }

            let parsed_challenge = Self::parse_challenge_data(data)?;

            // Bind amount and recipient to this specific transaction
            if parsed_challenge.amount != amount {
                return Err(AppError::BadRequest(format!(
                    "Challenge amount mismatch: challenge authorizes {} micro-USDC, but transaction requests {}",
                    parsed_challenge.amount, amount
                )));
            }
            if parsed_challenge.recipient != seller_str {
                return Err(AppError::BadRequest(format!(
                    "Challenge recipient mismatch: challenge authorizes recipient '{}', but transaction recipient is '{}'",
                    parsed_challenge.recipient, seller_str
                )));
            }

            // Replay protection: enforce unique consumption of invoice_id in the same DB transaction
            let now_ts = chrono::Utc::now().timestamp();
            let insert_invoice = sqlx::query(
                "INSERT INTO consumed_invoices (invoice_id, consumed_at) VALUES (?1, ?2)",
            )
            .bind(&parsed_challenge.invoice_id)
            .bind(now_ts)
            .execute(&mut **tx)
            .await;

            if let Err(e) = insert_invoice {
                return Err(AppError::BadRequest(format!(
                    "Challenge replay detected: invoice '{}' has already been consumed ({})",
                    parsed_challenge.invoice_id, e
                )));
            }
        }

        // 3. Deduct balance from buyer atomically
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
        let now = chrono::Utc::now().timestamp();
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
        .bind(expires_at)
        .execute(&mut **tx)
        .await?;

        tracing::info!(
            "🔒 [a2a_ledger] Prepared lock {} for {} micro-USDC (buyer: {}, seller: {}, expires_at: {})",
            lock_id, amount, buyer_str, seller_str, expires_at
        );

        Ok(lock_id)
    }

    /// Commit Phase: credits seller and marks ledger as COMMITTED.
    pub async fn commit_transaction(
        tx: &mut Transaction<'_, Sqlite>,
        lock_id: &str,
        signing_key: Option<&SigningKey>,
    ) -> Result<String, AppError> {
        let now = chrono::Utc::now().timestamp();

        // 1. Fetch lock details
        let lock_opt: Option<(String, String, String, i64, i64)> = sqlx::query_as(
            "SELECT transaction_id, seller_id, buyer_id, locked_amount, expires_at \
             FROM transaction_locks WHERE lock_id = ?1",
        )
        .bind(lock_id)
        .fetch_optional(&mut **tx)
        .await?;

        let (transaction_id, seller_str, _buyer_str, amount, expires_at) = match lock_opt {
            Some(l) => l,
            None => {
                return Err(AppError::NotFound(format!(
                    "Transaction lock {} not found",
                    lock_id
                )))
            }
        };

        if now > expires_at {
            return Err(AppError::Conflict(format!(
                "Transaction lock {} expired at {} (current time: {})",
                lock_id, expires_at, now
            )));
        }

        // 2. Consume lock atomically BEFORE crediting seller (prevents commit vs sweep race)
        let del_res =
            sqlx::query("DELETE FROM transaction_locks WHERE lock_id = ?1 AND expires_at >= ?2")
                .bind(lock_id)
                .bind(now)
                .execute(&mut **tx)
                .await?;

        if del_res.rows_affected() == 0 {
            return Err(AppError::Conflict(format!(
                "Transaction lock {} was concurrently settled, rolled back, or expired",
                lock_id
            )));
        }

        // 3. Credit seller balance atomically using UPSERT
        sqlx::query(
            "INSERT INTO agent_balances (agent_id, asset_type, balance) \
             VALUES (?1, 'USDC', ?2) \
             ON CONFLICT(agent_id, asset_type) DO UPDATE SET balance = balance + excluded.balance",
        )
        .bind(&seller_str)
        .bind(amount)
        .execute(&mut **tx)
        .await?;

        // 4. Cryptographic Audit Signature
        let mut audit_signature = None;
        if let Some(sk) = signing_key {
            let message = format!("{}:{}:{}", transaction_id, seller_str, amount);
            let sig = sk.sign(message.as_bytes());
            audit_signature = Some(hex::encode(sig.to_bytes()));
        }

        // 5. Update transaction status in ledger (COMMITTED)
        sqlx::query(
            "UPDATE transaction_ledger SET status = 'COMMITTED', audit_signature = ?1 WHERE id = ?2",
        )
        .bind(&audit_signature)
        .bind(&transaction_id)
        .execute(&mut **tx)
        .await?;

        tracing::info!(
            "💳 [a2a_ledger] Committed transaction {} for lock {} (amount: {} micro-USDC -> {})",
            transaction_id,
            lock_id,
            amount,
            seller_str
        );

        Ok(transaction_id)
    }

    /// Rollback Phase: returns locked amount to buyer.
    pub async fn rollback_transaction(
        tx: &mut Transaction<'_, Sqlite>,
        lock_id: &str,
    ) -> Result<(), AppError> {
        // 1. Fetch lock details
        let lock_opt: Option<(String, String, i64)> = sqlx::query_as(
            "SELECT transaction_id, buyer_id, locked_amount FROM transaction_locks WHERE lock_id = ?1",
        )
        .bind(lock_id)
        .fetch_optional(&mut **tx)
        .await?;

        let (transaction_id, buyer_str, amount) = match lock_opt {
            Some(l) => l,
            None => {
                return Err(AppError::NotFound(format!(
                    "Transaction lock {} not found",
                    lock_id
                )))
            }
        };

        // 2. Consume lock atomically BEFORE refunding buyer
        let del_res = sqlx::query("DELETE FROM transaction_locks WHERE lock_id = ?1")
            .bind(lock_id)
            .execute(&mut **tx)
            .await?;

        if del_res.rows_affected() == 0 {
            return Err(AppError::Conflict(format!(
                "Transaction lock {} was concurrently settled or rolled back",
                lock_id
            )));
        }

        // 3. Return buyer balance atomically using UPSERT
        sqlx::query(
            "INSERT INTO agent_balances (agent_id, asset_type, balance) \
             VALUES (?1, 'USDC', ?2) \
             ON CONFLICT(agent_id, asset_type) DO UPDATE SET balance = balance + excluded.balance",
        )
        .bind(&buyer_str)
        .bind(amount)
        .execute(&mut **tx)
        .await?;

        // 4. Update transaction status in ledger (ROLLED_BACK)
        sqlx::query("UPDATE transaction_ledger SET status = 'ROLLED_BACK' WHERE id = ?1")
            .bind(&transaction_id)
            .execute(&mut **tx)
            .await?;

        tracing::info!(
            "🔄 [a2a_ledger] Rolled back transaction {} for lock {} (refunded: {} micro-USDC -> {})",
            transaction_id, lock_id, amount, buyer_str
        );

        Ok(())
    }

    /// Automatically sweeps and rolls back expired locks.
    pub async fn sweep_expired_locks(pool: &sqlx::SqlitePool) -> Result<u32, AppError> {
        let now = chrono::Utc::now().timestamp();

        // Fetch expired lock IDs with limit to avoid unbounded batch sizes
        let expired_locks: Vec<(String,)> =
            sqlx::query_as("SELECT lock_id FROM transaction_locks WHERE expires_at < ?1 LIMIT 100")
                .bind(now)
                .fetch_all(pool)
                .await?;

        let mut swept_count = 0;
        for (lock_id,) in expired_locks {
            tracing::info!(
                "🧹 [a2a_ledger] Sweeping expired transaction lock: {}",
                lock_id
            );
            let mut db_tx = pool.begin().await?;
            match Self::rollback_transaction(&mut db_tx, &lock_id).await {
                Ok(_) => {
                    db_tx.commit().await?;
                    swept_count += 1;
                }
                Err(e) => {
                    tracing::debug!("ℹ️ [a2a_ledger] Sweeper skipped lock {}: {:?}", lock_id, e);
                    let _ = db_tx.rollback().await;
                }
            }
        }

        Ok(swept_count)
    }
}

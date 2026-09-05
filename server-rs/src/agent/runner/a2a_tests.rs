//! @docs ARCHITECTURE:Registry
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Agent Runner / a2a_tests
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Type-safe state handling and bounded execution without unhandled panics.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: none declared
//! - **Witness Tests**: none declared

#[cfg(test)]
mod tests {
    use crate::agent::runner::a2a_ledger::A2ATransactionCoordinator;
    use crate::agent::runner::a2a_mailbox::{A2AMailbox, MailboxEnvelope};
    use crate::agent::runner::a2a_router::{
        A2APaymentAdapter, L3HybridAdapter, LocalMockAdapter, PaymentRouter,
    };
    use crate::agent::runner::a2a_types::Address;
    use ed25519_dalek::{Signer, SigningKey};
    use sqlx::SqlitePool;
    use std::sync::Arc;

    async fn setup_test_db() -> SqlitePool {
        let pool = sqlx::sqlite::SqlitePoolOptions::new()
            .max_connections(5)
            .connect("sqlite::memory:?cache=shared")
            .await
            .unwrap();

        // Run our A2A schema tables
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS agent_balances (
                agent_id TEXT NOT NULL,
                asset_type TEXT NOT NULL,
                balance INTEGER NOT NULL DEFAULT 0,
                PRIMARY KEY (agent_id, asset_type)
            );",
        )
        .execute(&pool)
        .await
        .unwrap();

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS agent_asset_registry (
                asset_id TEXT PRIMARY KEY,
                owner_agent_id TEXT NOT NULL,
                asset_name TEXT NOT NULL,
                asset_data TEXT
            );",
        )
        .execute(&pool)
        .await
        .unwrap();

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS transaction_ledger (
                id TEXT PRIMARY KEY,
                transaction_type TEXT NOT NULL,
                buyer_address TEXT,
                seller_address TEXT,
                asset_id TEXT,
                amount INTEGER NOT NULL,
                status TEXT NOT NULL,
                audit_signature TEXT,
                created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
            );",
        )
        .execute(&pool)
        .await
        .unwrap();

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS transaction_locks (
                lock_id TEXT PRIMARY KEY,
                transaction_id TEXT NOT NULL,
                buyer_id TEXT NOT NULL,
                seller_id TEXT NOT NULL,
                locked_amount INTEGER NOT NULL,
                expires_at INTEGER NOT NULL
            );",
        )
        .execute(&pool)
        .await
        .unwrap();

        sqlx::query(
            "CREATE INDEX IF NOT EXISTS idx_transaction_locks_buyer ON transaction_locks(buyer_id);",
        )
        .execute(&pool)
        .await
        .unwrap();

        sqlx::query(
            "CREATE INDEX IF NOT EXISTS idx_transaction_locks_expires ON transaction_locks(expires_at);",
        )
        .execute(&pool)
        .await
        .unwrap();

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS consumed_invoices (
                invoice_id TEXT PRIMARY KEY,
                consumed_at INTEGER NOT NULL
            );",
        )
        .execute(&pool)
        .await
        .unwrap();

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS agent_economics_meta (
                agent_id TEXT PRIMARY KEY,
                economic_zone TEXT NOT NULL DEFAULT 'DEV',
                daily_spend_limit INTEGER NOT NULL DEFAULT 0,
                daily_spent_accumulated INTEGER NOT NULL DEFAULT 0,
                last_reset_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
            );",
        )
        .execute(&pool)
        .await
        .unwrap();

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS agent_directives (
                id TEXT PRIMARY KEY,
                mission_id TEXT NOT NULL,
                source_agent_id TEXT NOT NULL,
                target_agent_id TEXT NOT NULL,
                instruction TEXT NOT NULL,
                status TEXT NOT NULL DEFAULT 'pending',
                result TEXT,
                reasoning_trace TEXT,
                artifacts TEXT,
                created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
            );",
        )
        .execute(&pool)
        .await
        .unwrap();

        pool
    }

    #[tokio::test]
    async fn test_mailbox_flow() {
        let pool = setup_test_db().await;
        let mailbox = A2AMailbox::new(pool);

        let env = MailboxEnvelope {
            id: "msg-1".to_string(),
            mission_id: "mission-123".to_string(),
            source_agent_id: "agent-a".to_string(),
            target_agent_id: "agent-b".to_string(),
            instruction: "Find files".to_string(),
            reasoning_trace: None,
            status: "pending".to_string(),
            result: None,
            artifacts: None,
            timestamp: None,
            nonce: None,
        };

        mailbox.send_envelope(&env).await.unwrap();

        let mail = mailbox
            .fetch_mailbox("agent-b", None, None, None)
            .await
            .unwrap();
        assert_eq!(mail.len(), 1);
        assert_eq!(mail[0].id, "msg-1");
        assert_eq!(mail[0].instruction, "Find files");
    }

    #[tokio::test]
    async fn test_mailbox_pagination_and_filtering() {
        let pool = setup_test_db().await;
        let mailbox = A2AMailbox::new(pool);

        // Send 3 envelopes
        for i in 1..=3 {
            let status = if i == 2 { "completed" } else { "pending" };
            let env = MailboxEnvelope {
                id: format!("msg-{}", i),
                mission_id: "mission-123".to_string(),
                source_agent_id: "agent-a".to_string(),
                target_agent_id: "agent-b".to_string(),
                instruction: format!("Find files {}", i),
                reasoning_trace: None,
                status: status.to_string(),
                result: None,
                artifacts: None,
                timestamp: None,
                nonce: None,
            };
            mailbox.send_envelope(&env).await.unwrap();
        }

        // Fetch all (no filtering) - sorted by created_at DESC, id DESC -> msg-3, msg-2, msg-1
        let mail_all = mailbox
            .fetch_mailbox("agent-b", None, None, None)
            .await
            .unwrap();
        assert_eq!(mail_all.len(), 3);
        assert_eq!(mail_all[0].id, "msg-3");
        assert_eq!(mail_all[1].id, "msg-2");
        assert_eq!(mail_all[2].id, "msg-1");

        // Filter by status "completed" -> should only return msg-2
        let mail_completed = mailbox
            .fetch_mailbox("agent-b", Some("completed"), None, None)
            .await
            .unwrap();
        assert_eq!(mail_completed.len(), 1);
        assert_eq!(mail_completed[0].id, "msg-2");

        // Filter by status "pending" -> should return msg-3 and msg-1
        let mail_pending = mailbox
            .fetch_mailbox("agent-b", Some("pending"), None, None)
            .await
            .unwrap();
        assert_eq!(mail_pending.len(), 2);
        assert_eq!(mail_pending[0].id, "msg-3");
        assert_eq!(mail_pending[1].id, "msg-1");

        // Limit = 2 -> should return msg-3 and msg-2
        let mail_limit = mailbox
            .fetch_mailbox("agent-b", None, Some(2), None)
            .await
            .unwrap();
        assert_eq!(mail_limit.len(), 2);
        assert_eq!(mail_limit[0].id, "msg-3");
        assert_eq!(mail_limit[1].id, "msg-2");

        // Limit = 2, Offset = 1 -> should skip msg-3, returning msg-2 and msg-1
        let mail_offset = mailbox
            .fetch_mailbox("agent-b", None, Some(2), Some(1))
            .await
            .unwrap();
        assert_eq!(mail_offset.len(), 2);
        assert_eq!(mail_offset[0].id, "msg-2");
        assert_eq!(mail_offset[1].id, "msg-1");
    }

    #[tokio::test]
    async fn test_mailbox_artifacts_with_commas() {
        let pool = setup_test_db().await;
        let mailbox = A2AMailbox::new(pool);

        let artifacts = vec![
            "report,draft,v1.pdf".to_string(),
            "data,2026,final.csv".to_string(),
        ];

        let env = MailboxEnvelope {
            id: "msg-comma".to_string(),
            mission_id: "mission-comma".to_string(),
            source_agent_id: "agent-a".to_string(),
            target_agent_id: "agent-b".to_string(),
            instruction: "Process complex names".to_string(),
            reasoning_trace: None,
            status: "pending".to_string(),
            result: None,
            artifacts: Some(artifacts.clone()),
            timestamp: None,
            nonce: None,
        };

        mailbox.send_envelope(&env).await.unwrap();

        let mail = mailbox
            .fetch_mailbox("agent-b", None, None, None)
            .await
            .unwrap();
        assert_eq!(mail.len(), 1);
        assert_eq!(mail[0].artifacts.as_ref().unwrap(), &artifacts);
    }

    #[tokio::test]
    async fn test_mailbox_ssrf_metadata_rejected() {
        let pool = setup_test_db().await;
        let mailbox = A2AMailbox::new(pool);

        let env = MailboxEnvelope {
            id: "msg-ssrf".to_string(),
            mission_id: "mission-ssrf".to_string(),
            source_agent_id: "agent-a".to_string(),
            target_agent_id: "http://10.0.0.1/latest/meta-data".to_string(),
            instruction: "Steal credentials".to_string(),
            reasoning_trace: None,
            status: "pending".to_string(),
            result: None,
            artifacts: None,
            timestamp: None,
            nonce: None,
        };

        let res = mailbox.send_envelope(&env).await;
        assert!(res.is_err());
        assert!(res.unwrap_err().to_string().contains("SSRF Security Gate"));
    }

    #[tokio::test]
    async fn test_ledger_prepare_commit_flow() {
        let pool = setup_test_db().await;

        let buyer = Address::Local("buyer".to_string());
        let seller = Address::Local("seller".to_string());

        // Fund buyer
        sqlx::query("INSERT INTO agent_balances (agent_id, asset_type, balance) VALUES ('local:buyer', 'USDC', 1000)")
            .execute(&pool)
            .await
            .unwrap();

        let mut tx = pool.begin().await.unwrap();
        let lock_id = A2ATransactionCoordinator::prepare_transaction(
            &mut tx, &buyer, &seller, 200, None, None, None,
        )
        .await
        .unwrap();
        tx.commit().await.unwrap();

        // Check lock & deducted buyer balance
        let buyer_bal: (i64,) =
            sqlx::query_as("SELECT balance FROM agent_balances WHERE agent_id = 'local:buyer'")
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(buyer_bal.0, 800);

        let mut tx = pool.begin().await.unwrap();
        let tx_id = A2ATransactionCoordinator::commit_transaction(&mut tx, &lock_id, None)
            .await
            .unwrap();
        tx.commit().await.unwrap();

        // Check seller balance
        let seller_bal: (i64,) =
            sqlx::query_as("SELECT balance FROM agent_balances WHERE agent_id = 'local:seller'")
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(seller_bal.0, 200);

        // Check ledger entry status
        let status: (String,) =
            sqlx::query_as("SELECT status FROM transaction_ledger WHERE id = ?1")
                .bind(&tx_id)
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(status.0, "COMMITTED");
    }

    #[tokio::test]
    async fn test_ledger_prepare_rollback_flow() {
        let pool = setup_test_db().await;

        let buyer = Address::Local("buyer".to_string());
        let seller = Address::Local("seller".to_string());

        // Fund buyer
        sqlx::query("INSERT INTO agent_balances (agent_id, asset_type, balance) VALUES ('local:buyer', 'USDC', 1000)")
            .execute(&pool)
            .await
            .unwrap();

        let mut tx = pool.begin().await.unwrap();
        let lock_id = A2ATransactionCoordinator::prepare_transaction(
            &mut tx, &buyer, &seller, 200, None, None, None,
        )
        .await
        .unwrap();
        tx.commit().await.unwrap();

        let mut tx = pool.begin().await.unwrap();
        A2ATransactionCoordinator::rollback_transaction(&mut tx, &lock_id)
            .await
            .unwrap();
        tx.commit().await.unwrap();

        // Buyer balance should be restored
        let buyer_bal: (i64,) =
            sqlx::query_as("SELECT balance FROM agent_balances WHERE agent_id = 'local:buyer'")
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(buyer_bal.0, 1000);

        // Seller balance should be 0
        let seller_bal_opt: Option<(i64,)> =
            sqlx::query_as("SELECT balance FROM agent_balances WHERE agent_id = 'local:seller'")
                .fetch_optional(&pool)
                .await
                .unwrap();
        assert!(seller_bal_opt.is_none() || seller_bal_opt.unwrap().0 == 0);
    }

    #[tokio::test]
    async fn test_self_transfer_and_zero_amount_rejected() {
        let pool = setup_test_db().await;
        let buyer = Address::Local("agent_same".to_string());
        let seller = Address::Local("agent_different".to_string());

        let mut tx = pool.begin().await.unwrap();

        // Self transfer rejected
        let self_res = A2ATransactionCoordinator::prepare_transaction(
            &mut tx, &buyer, &buyer, 100, None, None, None,
        )
        .await;
        assert!(self_res.is_err());
        assert!(self_res.unwrap_err().to_string().contains("Self-transfers"));

        // Zero amount rejected
        let zero_res = A2ATransactionCoordinator::prepare_transaction(
            &mut tx, &buyer, &seller, 0, None, None, None,
        )
        .await;
        assert!(zero_res.is_err());
    }

    #[tokio::test]
    async fn test_challenge_signature_binding_and_replay_protection() {
        let pool = setup_test_db().await;

        let buyer = Address::Local("buyer_x402".to_string());
        let seller = Address::Local("seller_x402".to_string());

        // Fund buyer
        sqlx::query("INSERT INTO agent_balances (agent_id, asset_type, balance) VALUES ('local:buyer_x402', 'USDC', 5000)")
            .execute(&pool)
            .await
            .unwrap();

        // Generate challenge header for 200 micro-USDC to seller_x402
        let challenge =
            A2ATransactionCoordinator::generate_x402_challenge(&seller, "resource_gpu_hours", 200)
                .unwrap();

        // Sign challenge with test key
        let signing_key = SigningKey::from_bytes(&[42u8; 32]);
        let verifying_key = signing_key.verifying_key();

        let sig = signing_key.sign(challenge.challenge_data.as_bytes());
        let sig_hex = hex::encode(sig.to_bytes());

        // 1. Valid preparation (succeeds)
        let mut tx = pool.begin().await.unwrap();
        let lock_id = A2ATransactionCoordinator::prepare_transaction(
            &mut tx,
            &buyer,
            &seller,
            200,
            Some(&challenge.challenge_data),
            Some(&sig_hex),
            Some(&verifying_key),
        )
        .await
        .unwrap();
        tx.commit().await.unwrap();
        assert!(!lock_id.is_empty());

        // 2. Replay attack: using the EXACT SAME challenge data and signature a second time (fails)
        let mut tx2 = pool.begin().await.unwrap();
        let replay_res = A2ATransactionCoordinator::prepare_transaction(
            &mut tx2,
            &buyer,
            &seller,
            200,
            Some(&challenge.challenge_data),
            Some(&sig_hex),
            Some(&verifying_key),
        )
        .await;
        let _ = tx2.rollback().await;
        assert!(replay_res.is_err());
        assert!(replay_res
            .unwrap_err()
            .to_string()
            .contains("Challenge replay detected"));

        // 3. Amount mismatch: using challenge with different amount requested (fails)
        let new_challenge =
            A2ATransactionCoordinator::generate_x402_challenge(&seller, "resource_storage", 50)
                .unwrap();
        let sig2 = signing_key.sign(new_challenge.challenge_data.as_bytes());
        let sig2_hex = hex::encode(sig2.to_bytes());

        let mut tx3 = pool.begin().await.unwrap();
        let amount_mismatch_res = A2ATransactionCoordinator::prepare_transaction(
            &mut tx3,
            &buyer,
            &seller,
            100, // requested 100 but challenge authorizes 50
            Some(&new_challenge.challenge_data),
            Some(&sig2_hex),
            Some(&verifying_key),
        )
        .await;
        let _ = tx3.rollback().await;
        assert!(amount_mismatch_res.is_err());
        assert!(amount_mismatch_res
            .unwrap_err()
            .to_string()
            .contains("Challenge amount mismatch"));

        // 4. Domain-separated signature: signing with "TADPOLE_X402_V1:" prefix (succeeds)
        let domain_challenge =
            A2ATransactionCoordinator::generate_x402_challenge(&seller, "resource_compute", 50)
                .unwrap();
        let domain_msg = format!("TADPOLE_X402_V1:{}", domain_challenge.challenge_data);
        let domain_sig = signing_key.sign(domain_msg.as_bytes());
        let domain_sig_hex = hex::encode(domain_sig.to_bytes());

        let mut tx4 = pool.begin().await.unwrap();
        let domain_lock_id = A2ATransactionCoordinator::prepare_transaction(
            &mut tx4,
            &buyer,
            &seller,
            50,
            Some(&domain_challenge.challenge_data),
            Some(&domain_sig_hex),
            Some(&verifying_key),
        )
        .await
        .unwrap();
        tx4.commit().await.unwrap();
        assert!(!domain_lock_id.is_empty());

        // 5. Incomplete challenge parameters: providing challenge_data without signature (fails)
        let mut tx5 = pool.begin().await.unwrap();
        let incomplete_res = A2ATransactionCoordinator::prepare_transaction(
            &mut tx5,
            &buyer,
            &seller,
            50,
            Some(&domain_challenge.challenge_data),
            None,
            Some(&verifying_key),
        )
        .await;
        let _ = tx5.rollback().await;
        assert!(incomplete_res.is_err());
        assert!(incomplete_res
            .unwrap_err()
            .to_string()
            .contains("Incomplete x402 challenge parameters"));

        // 6. Malformed / arbitrary challenge string (fails parsing)
        let malformed_err = A2ATransactionCoordinator::parse_challenge_data(
            "arbitrary-signing-payload-attacker-data",
        );
        assert!(malformed_err.is_err());
        assert!(malformed_err
            .unwrap_err()
            .to_string()
            .contains("Malformed x402 challenge data format"));
    }

    #[tokio::test]
    async fn test_commit_expired_lock_rejected() {
        let pool = setup_test_db().await;

        let buyer = Address::Local("buyer_exp".to_string());
        let seller = Address::Local("seller_exp".to_string());

        sqlx::query("INSERT INTO agent_balances (agent_id, asset_type, balance) VALUES ('local:buyer_exp', 'USDC', 1000)")
            .execute(&pool)
            .await
            .unwrap();

        let mut tx = pool.begin().await.unwrap();
        let lock_id = A2ATransactionCoordinator::prepare_transaction(
            &mut tx, &buyer, &seller, 200, None, None, None,
        )
        .await
        .unwrap();
        tx.commit().await.unwrap();

        // Expire lock manually in DB
        sqlx::query(
            "UPDATE transaction_locks SET expires_at = expires_at - 100 WHERE lock_id = ?1",
        )
        .bind(&lock_id)
        .execute(&pool)
        .await
        .unwrap();

        // Attempting to commit an expired lock must be rejected with Conflict
        let mut tx_commit = pool.begin().await.unwrap();
        let commit_res =
            A2ATransactionCoordinator::commit_transaction(&mut tx_commit, &lock_id, None).await;
        assert!(commit_res.is_err());
        assert!(commit_res.unwrap_err().to_string().contains("expired"));
    }

    #[tokio::test]
    async fn test_router_economic_zone_and_limits() {
        let pool = setup_test_db().await;

        let dev_adapter = Arc::new(LocalMockAdapter);
        let staging_adapter = Arc::new(LocalMockAdapter);
        let prod_adapter = Arc::new(LocalMockAdapter);
        let router = PaymentRouter::new(pool.clone(), dev_adapter, staging_adapter, prod_adapter);

        // Fund agent
        sqlx::query("INSERT INTO agent_balances (agent_id, asset_type, balance) VALUES ('local:agent_1', 'USDC', 10000)")
            .execute(&pool)
            .await
            .unwrap();

        // Create meta with 500 micro USDC limit
        sqlx::query("INSERT INTO agent_economics_meta (agent_id, economic_zone, daily_spend_limit, daily_spent_accumulated) VALUES ('agent_1', 'DEV', 500, 0)")
            .execute(&pool)
            .await
            .unwrap();

        let to_addr = Address::Local("agent_2".to_string());

        // Transfer 200 (should succeed)
        let tx_id = router
            .transfer("agent_1", &to_addr, 200, None)
            .await
            .unwrap();
        assert!(!tx_id.is_empty());

        // Transfer 400 (should exceed remaining limit of 300)
        let err_res = router.transfer("agent_1", &to_addr, 400, None).await;
        assert!(err_res.is_err());
        assert!(err_res
            .unwrap_err()
            .to_string()
            .contains("exceeded its daily limit cap"));
    }

    #[tokio::test]
    async fn test_unconfigured_agent_default_spend_limit() {
        let pool = setup_test_db().await;

        let dev_adapter = Arc::new(LocalMockAdapter);
        let staging_adapter = Arc::new(LocalMockAdapter);
        let prod_adapter = Arc::new(LocalMockAdapter);
        let router = PaymentRouter::new(pool.clone(), dev_adapter, staging_adapter, prod_adapter);

        // Fund an unconfigured agent (no prior row in agent_economics_meta)
        sqlx::query("INSERT INTO agent_balances (agent_id, asset_type, balance) VALUES ('local:unconfigured_agent', 'USDC', 50000000)")
            .execute(&pool)
            .await
            .unwrap();

        let to_addr = Address::Local("target_agent".to_string());

        // Transfer 1_000_000 micro-USDC (within 10_000_000 default limit) -> succeeds
        let tx_id = router
            .transfer("unconfigured_agent", &to_addr, 1_000_000, None)
            .await
            .unwrap();
        assert!(!tx_id.is_empty());

        // Attempt transfer of 15_000_000 micro-USDC -> exceeds remaining default limit
        let err_res = router
            .transfer("unconfigured_agent", &to_addr, 15_000_000, None)
            .await;
        assert!(err_res.is_err());
        let err_msg = err_res.unwrap_err().to_string();
        assert!(
            err_msg.contains("exceeded its daily limit cap")
                || err_msg.contains("default daily limit cap"),
            "Expected daily limit error, got: {}",
            err_msg
        );
    }

    #[tokio::test]
    async fn test_automated_lock_sweeper() {
        let pool = setup_test_db().await;

        let buyer = Address::Local("buyer_sweep".to_string());
        let seller = Address::Local("seller_sweep".to_string());

        // Fund buyer
        sqlx::query("INSERT INTO agent_balances (agent_id, asset_type, balance) VALUES ('local:buyer_sweep', 'USDC', 1000)")
            .execute(&pool)
            .await
            .unwrap();

        let mut tx = pool.begin().await.unwrap();
        let lock_id = A2ATransactionCoordinator::prepare_transaction(
            &mut tx, &buyer, &seller, 200, None, None, None,
        )
        .await
        .unwrap();
        tx.commit().await.unwrap();

        // Artificially modify expires_at to be in the past
        sqlx::query(
            "UPDATE transaction_locks SET expires_at = expires_at - 100 WHERE lock_id = ?1",
        )
        .bind(&lock_id)
        .execute(&pool)
        .await
        .unwrap();

        // Perform sweep
        let swept = A2ATransactionCoordinator::sweep_expired_locks(&pool)
            .await
            .unwrap();
        assert_eq!(swept, 1);

        // Check lock is deleted
        let lock_opt: Option<(String,)> =
            sqlx::query_as("SELECT lock_id FROM transaction_locks WHERE lock_id = ?1")
                .bind(&lock_id)
                .fetch_optional(&pool)
                .await
                .unwrap();
        assert!(lock_opt.is_none());

        // Check buyer balance is restored to 1000
        let buyer_bal: (i64,) = sqlx::query_as(
            "SELECT balance FROM agent_balances WHERE agent_id = 'local:buyer_sweep'",
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(buyer_bal.0, 1000);
    }

    #[tokio::test]
    async fn test_a2a_mailbox_replay_on_completed_directive_is_noop_and_warns() {
        let pool = setup_test_db().await;
        let mailbox = A2AMailbox::new(pool.clone());

        let initial_env = MailboxEnvelope {
            id: "dir-completed-1".to_string(),
            mission_id: "m-1".to_string(),
            source_agent_id: "agent-a".to_string(),
            target_agent_id: "agent-b".to_string(),
            instruction: "Initial pending instruction".to_string(),
            reasoning_trace: Some("Initial trace".to_string()),
            status: "pending".to_string(),
            result: None,
            artifacts: Some(vec!["artifact1.txt".to_string()]),
            timestamp: None,
            nonce: None,
        };

        // 1. Initial insert as pending
        mailbox.send_envelope(&initial_env).await.unwrap();

        // 2. Simulate worker completing the directive
        sqlx::query(
            "UPDATE agent_directives SET status = 'completed', result = 'Finished report' WHERE id = 'dir-completed-1'"
        )
        .execute(&pool)
        .await
        .unwrap();

        // 3. Replay an envelope with the same ID but modified instruction & status='pending'
        let replay_env = MailboxEnvelope {
            id: "dir-completed-1".to_string(),
            mission_id: "m-1".to_string(),
            source_agent_id: "agent-a".to_string(),
            target_agent_id: "agent-b".to_string(),
            instruction: "Overwritten malicious instruction".to_string(),
            reasoning_trace: Some("Overwritten trace".to_string()),
            status: "pending".to_string(),
            result: None,
            artifacts: None,
            timestamp: None,
            nonce: None,
        };

        // Should not fail, but should be a no-op due to WHERE status = 'pending'
        mailbox.send_envelope(&replay_env).await.unwrap();

        // 4. Verify DB record: status must still be 'completed', result preserved, instruction unchanged
        let (status, result, instruction): (String, Option<String>, String) = sqlx::query_as(
            "SELECT status, result, instruction FROM agent_directives WHERE id = 'dir-completed-1'",
        )
        .fetch_one(&pool)
        .await
        .unwrap();

        assert_eq!(status, "completed");
        assert_eq!(result.as_deref(), Some("Finished report"));
        assert_eq!(instruction, "Initial pending instruction");
    }

    #[tokio::test]
    async fn test_a2a_mailbox_replay_on_pending_directive_is_idempotent() {
        let pool = setup_test_db().await;
        let mailbox = A2AMailbox::new(pool.clone());

        let initial_env = MailboxEnvelope {
            id: "dir-pending-1".to_string(),
            mission_id: "m-1".to_string(),
            source_agent_id: "agent-a".to_string(),
            target_agent_id: "agent-b".to_string(),
            instruction: "Initial instruction".to_string(),
            reasoning_trace: Some("Initial trace".to_string()),
            status: "pending".to_string(),
            result: None,
            artifacts: None,
            timestamp: None,
            nonce: None,
        };

        mailbox.send_envelope(&initial_env).await.unwrap();

        // Replay on pending directive: updates instruction/trace idempotently while staying pending
        let updated_env = MailboxEnvelope {
            id: "dir-pending-1".to_string(),
            mission_id: "m-1".to_string(),
            source_agent_id: "agent-a".to_string(),
            target_agent_id: "agent-b".to_string(),
            instruction: "Refined instruction".to_string(),
            reasoning_trace: Some("Refined trace".to_string()),
            status: "pending".to_string(),
            result: None,
            artifacts: Some(vec!["patch.diff".to_string()]),
            timestamp: None,
            nonce: None,
        };

        mailbox.send_envelope(&updated_env).await.unwrap();

        let (status, instruction, trace): (String, String, Option<String>) = sqlx::query_as(
            "SELECT status, instruction, reasoning_trace FROM agent_directives WHERE id = 'dir-pending-1'",
        )
        .fetch_one(&pool)
        .await
        .unwrap();

        assert_eq!(status, "pending");
        assert_eq!(instruction, "Refined instruction");
        assert_eq!(trace.as_deref(), Some("Refined trace"));
    }

    #[tokio::test]
    async fn test_a2a_mailbox_chokepoint_bounds_and_ssrf_validation() {
        let pool = setup_test_db().await;
        let mailbox = A2AMailbox::new(pool);

        // 1. Oversized instruction (> 128 KB)
        let oversized_instruction = MailboxEnvelope {
            id: "dir-oversized-inst".to_string(),
            mission_id: "m-1".to_string(),
            source_agent_id: "agent-a".to_string(),
            target_agent_id: "agent-b".to_string(),
            instruction: "x".repeat(128 * 1024 + 1),
            reasoning_trace: None,
            status: "pending".to_string(),
            result: None,
            artifacts: None,
            timestamp: None,
            nonce: None,
        };
        let res = mailbox.send_envelope(&oversized_instruction).await;
        assert!(res.is_err());
        assert!(res
            .unwrap_err()
            .to_string()
            .contains("Instruction exceeds maximum"));

        // 2. Oversized reasoning trace (> 128 KB)
        let oversized_trace = MailboxEnvelope {
            id: "dir-oversized-trace".to_string(),
            mission_id: "m-1".to_string(),
            source_agent_id: "agent-a".to_string(),
            target_agent_id: "agent-b".to_string(),
            instruction: "normal instruction".to_string(),
            reasoning_trace: Some("t".repeat(128 * 1024 + 1)),
            status: "pending".to_string(),
            result: None,
            artifacts: None,
            timestamp: None,
            nonce: None,
        };
        let res = mailbox.send_envelope(&oversized_trace).await;
        assert!(res.is_err());
        assert!(res
            .unwrap_err()
            .to_string()
            .contains("Reasoning trace exceeds maximum"));

        // 3. Artifact count exceeded (> 50 items)
        let too_many_artifacts = MailboxEnvelope {
            id: "dir-artifacts-count".to_string(),
            mission_id: "m-1".to_string(),
            source_agent_id: "agent-a".to_string(),
            target_agent_id: "agent-b".to_string(),
            instruction: "normal instruction".to_string(),
            reasoning_trace: None,
            status: "pending".to_string(),
            result: None,
            artifacts: Some((0..51).map(|i| format!("art_{}", i)).collect()),
            timestamp: None,
            nonce: None,
        };
        let res = mailbox.send_envelope(&too_many_artifacts).await;
        assert!(res.is_err());
        assert!(res
            .unwrap_err()
            .to_string()
            .contains("Artifacts count exceeds maximum"));

        // 4. Artifact item size exceeded (> 4 KB)
        let oversized_artifact_item = MailboxEnvelope {
            id: "dir-artifact-item".to_string(),
            mission_id: "m-1".to_string(),
            source_agent_id: "agent-a".to_string(),
            target_agent_id: "agent-b".to_string(),
            instruction: "normal instruction".to_string(),
            reasoning_trace: None,
            status: "pending".to_string(),
            result: None,
            artifacts: Some(vec!["a".repeat(4 * 1024 + 1)]),
            timestamp: None,
            nonce: None,
        };
        let res = mailbox.send_envelope(&oversized_artifact_item).await;
        assert!(res.is_err());
        assert!(res
            .unwrap_err()
            .to_string()
            .contains("exceeds maximum allowed size"));
    }

    #[tokio::test]
    async fn test_l3_hybrid_adapter_simulation_exit() {
        let pool = setup_test_db().await;

        // Fund the local vault / from address
        sqlx::query("INSERT INTO agent_balances (agent_id, asset_type, balance) VALUES ('local:vault_agent', 'USDC', 10000000)")
            .execute(&pool)
            .await
            .unwrap();

        let from_addr = Address::Local("vault_agent".to_string());
        let to_web3 = Address::Web3("0x71C...Recipient".to_string());

        // Simulated RPC endpoint (mock://)
        let sim_adapter = L3HybridAdapter::new(
            "mock://polygon-rpc".to_string(),
            "0xVaultAddress".to_string(),
        );
        let tx_hash = sim_adapter
            .transfer_funds(&pool, &from_addr, &to_web3, 500_000, None)
            .await
            .unwrap();

        assert!(
            tx_hash.starts_with("0xsim_"),
            "Expected simulation tx hash prefix, got: {}",
            tx_hash
        );
    }

    #[tokio::test]
    async fn test_l3_hybrid_adapter_live_rpc_guard() {
        let pool = setup_test_db().await;

        // Fund the local vault / from address
        sqlx::query("INSERT INTO agent_balances (agent_id, asset_type, balance) VALUES ('local:vault_live_agent', 'USDC', 10000000)")
            .execute(&pool)
            .await
            .unwrap();

        let from_addr = Address::Local("vault_live_agent".to_string());
        let to_web3 = Address::Web3("0x999...LiveWallet".to_string());

        // Live HTTPS RPC endpoint without EVM raw transaction signer
        let live_adapter = L3HybridAdapter::new(
            "https://polygon-mainnet.infura.io/v3/fake".to_string(),
            "0xVaultAddress".to_string(),
        );
        let res = live_adapter
            .transfer_funds(&pool, &from_addr, &to_web3, 500_000, None)
            .await;

        assert!(res.is_err());
        let err_msg = res.unwrap_err().to_string();
        assert!(
            err_msg.contains(
                "Live on-chain Web3 exit requires an authentic signed EVM raw transaction"
            ),
            "Expected live RPC guard error, got: {}",
            err_msg
        );
    }
}

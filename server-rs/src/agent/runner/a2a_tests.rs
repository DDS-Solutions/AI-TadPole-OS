//! @docs ARCHITECTURE:Registry
//!
//! ### AI Assist Note
//! **! A2A Integration Tests**
//! This module implements high-fidelity logic for the Sovereign Reality layer.
//!
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: Runtime logic error, state desynchronization, or resource exhaustion.
//! - **Telemetry Link**: Search `[a2a_tests]` in tracing logs.


#[cfg(test)]
mod tests {
    use crate::agent::runner::a2a_ledger::A2ATransactionCoordinator;
    use crate::agent::runner::a2a_mailbox::{A2AMailbox, MailboxEnvelope};
    use crate::agent::runner::a2a_router::{
        A2APaymentAdapter, L3HybridAdapter, LocalMockAdapter, PaymentRouter,
    };
    use crate::agent::runner::a2a_types::Address;
    use sqlx::SqlitePool;
    use std::sync::Arc;

    async fn setup_test_db() -> SqlitePool {
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
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
            };
            mailbox.send_envelope(&env).await.unwrap();
        }

        // Fetch all (no filtering) - sorted by id DESC, so msg-3, msg-2, msg-1
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
    async fn test_mailbox_reasoning_and_artifacts() {
        let pool = setup_test_db().await;
        let mailbox = A2AMailbox::new(pool);

        let env = MailboxEnvelope {
            id: "msg-2".to_string(),
            mission_id: "mission-123".to_string(),
            source_agent_id: "agent-a".to_string(),
            target_agent_id: "agent-b".to_string(),
            instruction: "Find files".to_string(),
            reasoning_trace: Some("Reasoning thought traces...".to_string()),
            status: "pending".to_string(),
            result: None,
            artifacts: Some(vec!["file1.txt".to_string(), "file2.txt".to_string()]),
        };

        mailbox.send_envelope(&env).await.unwrap();

        let mail = mailbox
            .fetch_mailbox("agent-b", None, None, None)
            .await
            .unwrap();
        assert_eq!(mail.len(), 1);
        assert_eq!(mail[0].id, "msg-2");
        assert_eq!(
            mail[0].reasoning_trace.as_deref(),
            Some("Reasoning thought traces...")
        );
        assert_eq!(
            mail[0].artifacts.as_ref().unwrap(),
            &vec!["file1.txt".to_string(), "file2.txt".to_string()]
        );
    }

    #[tokio::test]
    async fn test_ledger_spend_limits_integration() {
        std::env::set_var("A2A_TEST_MODE", "true");
        let state = Arc::new(crate::state::AppState::new_minimal_mock().await);
        let pool = state.resources.pool.clone();

        let runner = crate::agent::runner::AgentRunner::new(state.clone());

        // Setup buyer and seller in DB
        sqlx::query("INSERT INTO agents (id, name, role, department, description, status, metadata) VALUES ('buyer', 'buyer', 'buyer', 'buyer', 'buyer', 'idle', '{}')")
            .execute(&pool)
            .await
            .unwrap();
        sqlx::query("INSERT INTO mission_history (id, agent_id, title, status) VALUES ('mission-456', 'buyer', 'title', 'active')")
            .execute(&pool)
            .await
            .unwrap();

        sqlx::query("INSERT INTO agent_balances (agent_id, asset_type, balance) VALUES ('local:buyer', 'USDC', 10000)")
            .execute(&pool)
            .await
            .unwrap();

        sqlx::query("INSERT INTO agent_economics_meta (agent_id, economic_zone, daily_spend_limit, daily_spent_accumulated) VALUES ('buyer', 'DEV', 500, 0)")
            .execute(&pool)
            .await
            .unwrap();

        let ctx = crate::agent::runner::RunContext {
            agent_id: "buyer".to_string(),
            mission_id: "mission-456".to_string(),
            ..crate::agent::runner::RunContext::default()
        };

        // Propose transaction of 200 (within limit)
        let propose_fc = crate::agent::types::ToolCall {
            name: "propose_asset_transaction".to_string(),
            args: serde_json::json!({
                "seller_id": "local:seller",
                "amount": 200
            }),
        };
        let res = runner
            .handle_propose_asset_transaction(&ctx, &propose_fc)
            .await
            .unwrap();
        assert!(res.contains("prepared"));

        // Extract Lock ID from response
        let lock_id = res
            .split("Lock ID: ")
            .nth(1)
            .unwrap()
            .split('.')
            .next()
            .unwrap()
            .to_string();

        // Propose transaction of 400 (should fail as it exceeds remaining limit of 300)
        let propose_fail_fc = crate::agent::types::ToolCall {
            name: "propose_asset_transaction".to_string(),
            args: serde_json::json!({
                "seller_id": "local:seller",
                "amount": 400
            }),
        };
        let err_res = runner
            .handle_propose_asset_transaction(&ctx, &propose_fail_fc)
            .await;
        assert!(err_res.is_err());
        assert!(err_res
            .unwrap_err()
            .to_string()
            .contains("exceeded its daily limit cap"));

        // Confirm the first transaction (which should accumulate spent limit)
        let confirm_fc = crate::agent::types::ToolCall {
            name: "confirm_asset_transaction".to_string(),
            args: serde_json::json!({
                "lock_id": lock_id
            }),
        };
        let res_confirm = runner
            .handle_confirm_asset_transaction(&ctx, &confirm_fc)
            .await
            .unwrap();
        assert!(res_confirm.contains("committed"));

        // Check accumulated spend in DB is now 200
        let spent: (i64,) = sqlx::query_as(
            "SELECT daily_spent_accumulated FROM agent_economics_meta WHERE agent_id = 'buyer'",
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(spent.0, 200);
        std::env::remove_var("A2A_TEST_MODE");
    }

    #[tokio::test]
    async fn test_automated_lock_sweeper() {
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

        // Artificially modify expires_at to be in the past
        sqlx::query(
            "UPDATE transaction_locks SET expires_at = expires_at - 100 WHERE lock_id = ?1",
        )
        .bind(&lock_id)
        .execute(&pool)
        .await
        .unwrap();

        // Perform sweep
        A2ATransactionCoordinator::sweep_expired_locks(&pool)
            .await
            .unwrap();

        // Check lock is deleted
        let lock_opt: Option<(String,)> =
            sqlx::query_as("SELECT lock_id FROM transaction_locks WHERE lock_id = ?1")
                .bind(&lock_id)
                .fetch_optional(&pool)
                .await
                .unwrap();
        assert!(lock_opt.is_none());

        // Check buyer balance is restored to 1000
        let buyer_bal: (i64,) =
            sqlx::query_as("SELECT balance FROM agent_balances WHERE agent_id = 'local:buyer'")
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(buyer_bal.0, 1000);
    }

    #[tokio::test]
    async fn test_web3_rpc_exits() {
        let pool = setup_test_db().await;
        let adapter = L3HybridAdapter::new(
            "http://localhost:9999/rpc".to_string(),
            "0xvault".to_string(),
        );

        let buyer = Address::Local("buyer".to_string());
        let seller = Address::Web3("0xseller".to_string());

        sqlx::query("INSERT INTO agent_balances (agent_id, asset_type, balance) VALUES ('local:buyer', 'USDC', 1000)")
            .execute(&pool)
            .await
            .unwrap();

        // Transfer funds should trigger Web3 RPC (mocked and caught gracefully)
        let tx_id = adapter
            .transfer_funds(&pool, &buyer, &seller, 200, None)
            .await
            .unwrap();
        assert!(!tx_id.is_empty());
    }

    #[tokio::test]
    async fn test_ledger_spend_limits_oversight_override() {
        let state = Arc::new(crate::state::AppState::new_minimal_mock().await);
        let pool = state.resources.pool.clone();
        let runner = crate::agent::runner::AgentRunner::new(state.clone());

        // Setup buyer and seller in DB
        sqlx::query("INSERT INTO agents (id, name, role, department, description, status, metadata) VALUES ('buyer_over', 'buyer_over', 'buyer', 'buyer', 'buyer', 'idle', '{}')")
            .execute(&pool)
            .await
            .unwrap();
        sqlx::query("INSERT INTO mission_history (id, agent_id, title, status) VALUES ('mission-over-123', 'buyer_over', 'title', 'active')")
            .execute(&pool)
            .await
            .unwrap();

        sqlx::query("INSERT INTO agent_balances (agent_id, asset_type, balance) VALUES ('local:buyer_over', 'USDC', 10000)")
            .execute(&pool)
            .await
            .unwrap();

        sqlx::query("INSERT INTO agent_economics_meta (agent_id, economic_zone, daily_spend_limit, daily_spent_accumulated) VALUES ('buyer_over', 'DEV', 100, 0)")
            .execute(&pool)
            .await
            .unwrap();

        let ctx = crate::agent::runner::RunContext {
            agent_id: "buyer_over".to_string(),
            mission_id: "mission-over-123".to_string(),
            ..crate::agent::runner::RunContext::default()
        };

        // Propose transaction of 200 (exceeds limit of 100)
        let propose_fc = crate::agent::types::ToolCall {
            name: "propose_asset_transaction".to_string(),
            args: serde_json::json!({
                "seller_id": "local:seller",
                "amount": 200
            }),
        };

        // We can spawn a background task to approve the oversight request
        let state_clone = state.clone();
        tokio::spawn(async move {
            // Wait for entry to appear
            loop {
                if !state_clone.comms.oversight_queue.is_empty() {
                    break;
                }
                tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
            }

            // Find our entry and resolve it
            let entry_id = state_clone
                .comms
                .oversight_queue
                .iter()
                .next()
                .unwrap()
                .key()
                .clone();
            let resolver = state_clone
                .comms
                .oversight_resolvers
                .remove(&entry_id)
                .unwrap()
                .1;
            let _ = resolver.send(crate::agent::types::OversightResolution {
                approved: true,
                override_slot: None,
            });
            state_clone.comms.oversight_queue.remove(&entry_id);
        });

        let res = runner
            .handle_propose_asset_transaction(&ctx, &propose_fc)
            .await
            .unwrap();
        assert!(res.contains("prepared"));
    }
}

// Metadata: [a2a_tests]

//! @docs ARCHITECTURE:Registry
//! 
//! ### AI Assist Note
//! **! A2A Integration Tests**
//! This module implements high-fidelity logic for the Sovereign Reality layer.
//! 
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: Runtime logic error, state desynchronization, or resource exhaustion.
//! - **Telemetry Link**: Search `[a2a_tests]` in tracing logs.

//! A2A Integration Tests
//! @docs ARCHITECTURE:Runner

#[cfg(test)]
mod tests {
    use crate::agent::runner::a2a_types::Address;
    use crate::agent::runner::a2a_ledger::A2ATransactionCoordinator;
    use crate::agent::runner::a2a_router::{LocalMockAdapter, PaymentRouter};
    use crate::agent::runner::a2a_mailbox::{A2AMailbox, MailboxEnvelope};
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
            );"
        ).execute(&pool).await.unwrap();

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS agent_asset_registry (
                asset_id TEXT PRIMARY KEY,
                owner_agent_id TEXT NOT NULL,
                asset_name TEXT NOT NULL,
                asset_data TEXT
            );"
        ).execute(&pool).await.unwrap();

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
            );"
        ).execute(&pool).await.unwrap();

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS transaction_locks (
                lock_id TEXT PRIMARY KEY,
                transaction_id TEXT NOT NULL,
                buyer_id TEXT NOT NULL,
                seller_id TEXT NOT NULL,
                locked_amount INTEGER NOT NULL,
                expires_at INTEGER NOT NULL
            );"
        ).execute(&pool).await.unwrap();

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS agent_economics_meta (
                agent_id TEXT PRIMARY KEY,
                economic_zone TEXT NOT NULL DEFAULT 'DEV',
                daily_spend_limit INTEGER NOT NULL DEFAULT 0,
                daily_spent_accumulated INTEGER NOT NULL DEFAULT 0,
                last_reset_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
            );"
        ).execute(&pool).await.unwrap();

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS agent_directives (
                id TEXT PRIMARY KEY,
                mission_id TEXT NOT NULL,
                source_agent_id TEXT NOT NULL,
                target_agent_id TEXT NOT NULL,
                instruction TEXT NOT NULL,
                status TEXT NOT NULL DEFAULT 'pending',
                result TEXT,
                created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
            );"
        ).execute(&pool).await.unwrap();

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
            status: "pending".to_string(),
            result: None,
        };

        mailbox.send_envelope(&env).await.unwrap();

        let mail = mailbox.fetch_mailbox("agent-b", None, None, None).await.unwrap();
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
                status: status.to_string(),
                result: None,
            };
            mailbox.send_envelope(&env).await.unwrap();
        }

        // Fetch all (no filtering) - sorted by id DESC, so msg-3, msg-2, msg-1
        let mail_all = mailbox.fetch_mailbox("agent-b", None, None, None).await.unwrap();
        assert_eq!(mail_all.len(), 3);
        assert_eq!(mail_all[0].id, "msg-3");
        assert_eq!(mail_all[1].id, "msg-2");
        assert_eq!(mail_all[2].id, "msg-1");

        // Filter by status "completed" -> should only return msg-2
        let mail_completed = mailbox.fetch_mailbox("agent-b", Some("completed"), None, None).await.unwrap();
        assert_eq!(mail_completed.len(), 1);
        assert_eq!(mail_completed[0].id, "msg-2");

        // Filter by status "pending" -> should return msg-3 and msg-1
        let mail_pending = mailbox.fetch_mailbox("agent-b", Some("pending"), None, None).await.unwrap();
        assert_eq!(mail_pending.len(), 2);
        assert_eq!(mail_pending[0].id, "msg-3");
        assert_eq!(mail_pending[1].id, "msg-1");

        // Limit = 2 -> should return msg-3 and msg-2
        let mail_limit = mailbox.fetch_mailbox("agent-b", None, Some(2), None).await.unwrap();
        assert_eq!(mail_limit.len(), 2);
        assert_eq!(mail_limit[0].id, "msg-3");
        assert_eq!(mail_limit[1].id, "msg-2");

        // Limit = 2, Offset = 1 -> should skip msg-3, returning msg-2 and msg-1
        let mail_offset = mailbox.fetch_mailbox("agent-b", None, Some(2), Some(1)).await.unwrap();
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
            &mut tx,
            &buyer,
            &seller,
            200,
            None,
            None,
            None,
        )
        .await
        .unwrap();
        tx.commit().await.unwrap();

        // Check lock & deducted buyer balance
        let buyer_bal: (i64,) = sqlx::query_as("SELECT balance FROM agent_balances WHERE agent_id = 'local:buyer'")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(buyer_bal.0, 800);

        let mut tx = pool.begin().await.unwrap();
        let tx_id = A2ATransactionCoordinator::commit_transaction(
            &mut tx,
            &lock_id,
            None,
        )
        .await
        .unwrap();
        tx.commit().await.unwrap();

        // Check seller balance
        let seller_bal: (i64,) = sqlx::query_as("SELECT balance FROM agent_balances WHERE agent_id = 'local:seller'")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(seller_bal.0, 200);

        // Check ledger entry status
        let status: (String,) = sqlx::query_as("SELECT status FROM transaction_ledger WHERE id = ?1")
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
            &mut tx,
            &buyer,
            &seller,
            200,
            None,
            None,
            None,
        )
        .await
        .unwrap();
        tx.commit().await.unwrap();

        let mut tx = pool.begin().await.unwrap();
        A2ATransactionCoordinator::rollback_transaction(
            &mut tx,
            &lock_id,
        )
        .await
        .unwrap();
        tx.commit().await.unwrap();

        // Buyer balance should be restored
        let buyer_bal: (i64,) = sqlx::query_as("SELECT balance FROM agent_balances WHERE agent_id = 'local:buyer'")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(buyer_bal.0, 1000);

        // Seller balance should be 0
        let seller_bal_opt: Option<(i64,)> = sqlx::query_as("SELECT balance FROM agent_balances WHERE agent_id = 'local:seller'")
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
        let tx_id = router.transfer("agent_1", &to_addr, 200, None).await.unwrap();
        assert!(!tx_id.is_empty());

        // Transfer 400 (should exceed remaining limit of 300)
        let err_res = router.transfer("agent_1", &to_addr, 400, None).await;
        assert!(err_res.is_err());
        assert!(err_res.unwrap_err().to_string().contains("exceeded its daily limit cap"));
    }
}

// Metadata: [a2a_tests]

//! @docs ARCHITECTURE:Persistence
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Database / Contract Tests
//! - **Primary Entrypoints**: `test_contract_dashmap_backend`, `test_contract_sqlite_backend`
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Every test must pass against BOTH DashMap and SQLite backends.
//! - `[Structural]` These tests define the behavioral contract — new backends must satisfy all invariants.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: none declared
//! - **Witness Tests**: `test_contract_dashmap_backend`, `test_contract_sqlite_backend`

#[cfg(test)]
mod tests {
    use dashmap::DashMap;
    use sqlx::SqlitePool;
    use std::sync::Arc;

    // ── Trait: AgentStore ─────────────────────────────────────

    /// Minimal behavioral contract for agent storage backends.
    /// Any implementation (DashMap, SQLite, Redis, etc.) must satisfy these invariants.
    #[async_trait::async_trait]
    trait AgentStore: Send + Sync {
        async fn insert(&self, id: &str, name: &str, role: &str) -> Result<(), String>;
        async fn get(&self, id: &str) -> Result<Option<(String, String, String)>, String>;
        async fn update_role(&self, id: &str, new_role: &str) -> Result<bool, String>;
        async fn delete(&self, id: &str) -> Result<bool, String>;
        async fn list_all(&self) -> Result<Vec<(String, String, String)>, String>;
        async fn count(&self) -> Result<usize, String>;
    }

    // ── DashMap Backend ──────────────────────────────────────

    struct DashMapAgentStore {
        map: DashMap<String, (String, String)>, // id -> (name, role)
    }

    impl DashMapAgentStore {
        fn new() -> Self {
            Self {
                map: DashMap::new(),
            }
        }
    }

    #[async_trait::async_trait]
    impl AgentStore for DashMapAgentStore {
        async fn insert(&self, id: &str, name: &str, role: &str) -> Result<(), String> {
            self.map
                .insert(id.to_string(), (name.to_string(), role.to_string()));
            Ok(())
        }

        async fn get(&self, id: &str) -> Result<Option<(String, String, String)>, String> {
            Ok(self
                .map
                .get(id)
                .map(|entry| (id.to_string(), entry.0.clone(), entry.1.clone())))
        }

        async fn update_role(&self, id: &str, new_role: &str) -> Result<bool, String> {
            if let Some(mut entry) = self.map.get_mut(id) {
                entry.1 = new_role.to_string();
                Ok(true)
            } else {
                Ok(false)
            }
        }

        async fn delete(&self, id: &str) -> Result<bool, String> {
            Ok(self.map.remove(id).is_some())
        }

        async fn list_all(&self) -> Result<Vec<(String, String, String)>, String> {
            Ok(self
                .map
                .iter()
                .map(|entry| {
                    (
                        entry.key().clone(),
                        entry.value().0.clone(),
                        entry.value().1.clone(),
                    )
                })
                .collect())
        }

        async fn count(&self) -> Result<usize, String> {
            Ok(self.map.len())
        }
    }

    // ── SQLite Backend ───────────────────────────────────────

    struct SqliteAgentStore {
        pool: SqlitePool,
    }

    impl SqliteAgentStore {
        async fn new() -> Self {
            let pool = SqlitePool::connect("sqlite::memory:")
                .await
                .expect("Failed to connect to in-memory SQLite");

            sqlx::query(
                "CREATE TABLE contract_agents (id TEXT PRIMARY KEY, name TEXT NOT NULL, role TEXT NOT NULL)",
            )
            .execute(&pool)
            .await
            .expect("Failed to create contract_agents table");

            Self { pool }
        }
    }

    #[async_trait::async_trait]
    impl AgentStore for SqliteAgentStore {
        async fn insert(&self, id: &str, name: &str, role: &str) -> Result<(), String> {
            sqlx::query("INSERT OR REPLACE INTO contract_agents (id, name, role) VALUES (?, ?, ?)")
                .bind(id)
                .bind(name)
                .bind(role)
                .execute(&self.pool)
                .await
                .map_err(|e| e.to_string())?;
            Ok(())
        }

        async fn get(&self, id: &str) -> Result<Option<(String, String, String)>, String> {
            let row: Option<(String, String, String)> =
                sqlx::query_as("SELECT id, name, role FROM contract_agents WHERE id = ?")
                    .bind(id)
                    .fetch_optional(&self.pool)
                    .await
                    .map_err(|e| e.to_string())?;
            Ok(row)
        }

        async fn update_role(&self, id: &str, new_role: &str) -> Result<bool, String> {
            let result = sqlx::query("UPDATE contract_agents SET role = ? WHERE id = ?")
                .bind(new_role)
                .bind(id)
                .execute(&self.pool)
                .await
                .map_err(|e| e.to_string())?;
            Ok(result.rows_affected() > 0)
        }

        async fn delete(&self, id: &str) -> Result<bool, String> {
            let result = sqlx::query("DELETE FROM contract_agents WHERE id = ?")
                .bind(id)
                .execute(&self.pool)
                .await
                .map_err(|e| e.to_string())?;
            Ok(result.rows_affected() > 0)
        }

        async fn list_all(&self) -> Result<Vec<(String, String, String)>, String> {
            let rows: Vec<(String, String, String)> =
                sqlx::query_as("SELECT id, name, role FROM contract_agents ORDER BY id")
                    .fetch_all(&self.pool)
                    .await
                    .map_err(|e| e.to_string())?;
            Ok(rows)
        }

        async fn count(&self) -> Result<usize, String> {
            let (count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM contract_agents")
                .fetch_one(&self.pool)
                .await
                .map_err(|e| e.to_string())?;
            Ok(count as usize)
        }
    }

    // ── Shared Behavioral Contract Tests ─────────────────────

    /// Run the complete behavioral contract against any AgentStore implementation.
    async fn run_contract_suite(store: &dyn AgentStore, backend_name: &str) {
        // 1. Empty store invariant
        assert_eq!(
            store.count().await.unwrap(),
            0,
            "[{}] Fresh store must be empty",
            backend_name
        );
        assert_eq!(
            store.list_all().await.unwrap().len(),
            0,
            "[{}] list_all on empty store must return empty vec",
            backend_name
        );

        // 2. Insert
        store
            .insert("agent-1", "Alpha", "orchestrator")
            .await
            .unwrap();
        store
            .insert("agent-2", "Bravo", "specialist")
            .await
            .unwrap();
        store.insert("agent-3", "Charlie", "auditor").await.unwrap();
        assert_eq!(
            store.count().await.unwrap(),
            3,
            "[{}] Count after 3 inserts",
            backend_name
        );

        // 3. Get existing
        let agent = store.get("agent-1").await.unwrap();
        assert!(agent.is_some(), "[{}] Get existing agent", backend_name);
        let (id, name, role) = agent.unwrap();
        assert_eq!(id, "agent-1", "[{}] Agent ID match", backend_name);
        assert_eq!(name, "Alpha", "[{}] Agent name match", backend_name);
        assert_eq!(role, "orchestrator", "[{}] Agent role match", backend_name);

        // 4. Get non-existing returns None (not error)
        let missing = store.get("agent-999").await.unwrap();
        assert!(
            missing.is_none(),
            "[{}] Get missing agent returns None",
            backend_name
        );

        // 5. Update existing
        let updated = store.update_role("agent-2", "lead").await.unwrap();
        assert!(updated, "[{}] Update existing returns true", backend_name);
        let agent2 = store.get("agent-2").await.unwrap().unwrap();
        assert_eq!(
            agent2.2, "lead",
            "[{}] Role updated correctly",
            backend_name
        );

        // 6. Update non-existing returns false
        let not_updated = store.update_role("agent-999", "ghost").await.unwrap();
        assert!(
            !not_updated,
            "[{}] Update missing returns false",
            backend_name
        );

        // 7. Delete existing
        let deleted = store.delete("agent-3").await.unwrap();
        assert!(deleted, "[{}] Delete existing returns true", backend_name);
        assert_eq!(
            store.count().await.unwrap(),
            2,
            "[{}] Count after delete",
            backend_name
        );

        // 8. Delete non-existing returns false
        let not_deleted = store.delete("agent-3").await.unwrap();
        assert!(
            !not_deleted,
            "[{}] Delete already-deleted returns false",
            backend_name
        );

        // 9. List all returns consistent results
        let all = store.list_all().await.unwrap();
        assert_eq!(all.len(), 2, "[{}] List all after operations", backend_name);

        // 10. Upsert / idempotent insert
        store
            .insert("agent-1", "Alpha-v2", "commander")
            .await
            .unwrap();
        let updated_agent = store.get("agent-1").await.unwrap().unwrap();
        assert_eq!(
            updated_agent.1, "Alpha-v2",
            "[{}] Upsert overwrites name",
            backend_name
        );
        assert_eq!(
            updated_agent.2, "commander",
            "[{}] Upsert overwrites role",
            backend_name
        );
        assert_eq!(
            store.count().await.unwrap(),
            2,
            "[{}] Upsert doesn't duplicate",
            backend_name
        );
    }

    // ── Contract Test Runners ────────────────────────────────

    #[tokio::test]
    async fn test_contract_dashmap_backend() {
        let store = DashMapAgentStore::new();
        run_contract_suite(&store, "DashMap").await;
    }

    #[tokio::test]
    async fn test_contract_sqlite_backend() {
        let store = SqliteAgentStore::new().await;
        run_contract_suite(&store, "SQLite").await;
    }

    // ── Concurrency Contract (DashMap-specific but validates safety) ──

    #[tokio::test]
    async fn test_concurrent_insert_no_data_loss() {
        let store = Arc::new(DashMapAgentStore::new());
        let mut handles = vec![];

        for i in 0..100 {
            let s = Arc::clone(&store);
            handles.push(tokio::spawn(async move {
                s.insert(&format!("agent-{}", i), &format!("Name-{}", i), "worker")
                    .await
                    .unwrap();
            }));
        }

        for h in handles {
            h.await.unwrap();
        }

        assert_eq!(
            store.count().await.unwrap(),
            100,
            "All 100 concurrent inserts must be preserved"
        );
    }

    #[tokio::test]
    async fn test_concurrent_read_write_safety() {
        let store = Arc::new(DashMapAgentStore::new());

        // Pre-populate
        for i in 0..50 {
            store
                .insert(&format!("agent-{}", i), &format!("Name-{}", i), "worker")
                .await
                .unwrap();
        }

        let mut handles = vec![];

        // Concurrent readers
        for i in 0..50 {
            let s = Arc::clone(&store);
            handles.push(tokio::spawn(async move {
                let result = s.get(&format!("agent-{}", i)).await.unwrap();
                assert!(
                    result.is_some(),
                    "Agent {} must be readable during writes",
                    i
                );
            }));
        }

        // Concurrent writers
        for i in 50..100 {
            let s = Arc::clone(&store);
            handles.push(tokio::spawn(async move {
                s.insert(&format!("agent-{}", i), &format!("Name-{}", i), "worker")
                    .await
                    .unwrap();
            }));
        }

        for h in handles {
            h.await.unwrap();
        }

        assert_eq!(
            store.count().await.unwrap(),
            100,
            "All agents present after concurrent read/write"
        );
    }
}

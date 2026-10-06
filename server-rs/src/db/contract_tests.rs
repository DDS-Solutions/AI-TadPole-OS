//! @docs ARCHITECTURE:Persistence
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Database / Contract Tests
//! - **Primary Entrypoints**: `test_contract_dashmap_backend`, `test_contract_sqlite_backend`
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: none declared

#[cfg(test)]
mod tests {
    use dashmap::DashMap;
    use sqlx::SqlitePool;
    use std::sync::Arc;
    use tokio::task::JoinSet;

    // ── Closed Vocabulary: AgentRole ──────────────────────────

    /// Type-safe, closed vocabulary for agent roles across all storage backends.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
    pub enum AgentRole {
        Orchestrator,
        Specialist,
        Auditor,
        Lead,
        Worker,
    }

    impl AgentRole {
        pub const VARIANTS: [Self; 5] = [
            Self::Orchestrator,
            Self::Specialist,
            Self::Auditor,
            Self::Lead,
            Self::Worker,
        ];

        pub fn as_str(&self) -> &'static str {
            match self {
                Self::Orchestrator => "orchestrator",
                Self::Specialist => "specialist",
                Self::Auditor => "auditor",
                Self::Lead => "lead",
                Self::Worker => "worker",
            }
        }
    }

    impl std::fmt::Display for AgentRole {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.as_str())
        }
    }

    impl TryFrom<&str> for AgentRole {
        type Error = String;

        fn try_from(s: &str) -> Result<Self, Self::Error> {
            match s.trim().to_ascii_lowercase().as_str() {
                "orchestrator" => Ok(Self::Orchestrator),
                "specialist" => Ok(Self::Specialist),
                "auditor" => Ok(Self::Auditor),
                "lead" => Ok(Self::Lead),
                "worker" => Ok(Self::Worker),
                other => Err(format!("unknown agent role '{}'", other)),
            }
        }
    }

    // ── Trait: AgentStore ─────────────────────────────────────

    /// Minimal behavioral contract for agent storage backends.
    /// Any implementation (DashMap, SQLite, Redis, etc.) must satisfy these invariants.
    #[async_trait::async_trait]
    trait AgentStore: Send + Sync {
        async fn insert(&self, id: &str, name: &str, role: AgentRole) -> Result<(), String>;
        async fn get(&self, id: &str) -> Result<Option<(String, String, AgentRole)>, String>;
        async fn update_role(&self, id: &str, new_role: AgentRole) -> Result<bool, String>;
        async fn delete(&self, id: &str) -> Result<bool, String>;
        async fn list_all(&self) -> Result<Vec<(String, String, AgentRole)>, String>;
        async fn count(&self) -> Result<usize, String>;
    }

    // ── DashMap Backend ──────────────────────────────────────

    struct DashMapAgentStore {
        map: DashMap<String, (String, AgentRole)>, // id -> (name, role)
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
        async fn insert(&self, id: &str, name: &str, role: AgentRole) -> Result<(), String> {
            self.map.insert(id.to_string(), (name.to_string(), role));
            Ok(())
        }

        async fn get(&self, id: &str) -> Result<Option<(String, String, AgentRole)>, String> {
            Ok(self.map.get(id).map(|entry| {
                (
                    entry.key().clone(),
                    entry.value().0.clone(),
                    entry.value().1,
                )
            }))
        }

        async fn update_role(&self, id: &str, new_role: AgentRole) -> Result<bool, String> {
            if let Some(mut entry) = self.map.get_mut(id) {
                entry.1 = new_role;
                Ok(true)
            } else {
                Ok(false)
            }
        }

        async fn delete(&self, id: &str) -> Result<bool, String> {
            Ok(self.map.remove(id).is_some())
        }

        async fn list_all(&self) -> Result<Vec<(String, String, AgentRole)>, String> {
            let mut rows: Vec<(String, String, AgentRole)> = self
                .map
                .iter()
                .map(|entry| {
                    (
                        entry.key().clone(),
                        entry.value().0.clone(),
                        entry.value().1,
                    )
                })
                .collect();
            // Contract invariant: list_all MUST be ordered ascending by ID
            rows.sort_by(|a, b| a.0.cmp(&b.0));
            Ok(rows)
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
            let unique_id = uuid::Uuid::new_v4();
            let uri = format!("file:contract_test_{}?mode=memory&cache=private", unique_id);
            let opts = crate::db::init::production_options(&uri)
                .expect("Failed to build SQLite production options for contract test");

            let pool = sqlx::sqlite::SqlitePoolOptions::new()
                .max_connections(1)
                .connect_with(opts)
                .await
                .expect("Failed to connect to isolated in-memory SQLite");

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
        async fn insert(&self, id: &str, name: &str, role: AgentRole) -> Result<(), String> {
            sqlx::query("INSERT OR REPLACE INTO contract_agents (id, name, role) VALUES (?, ?, ?)")
                .bind(id)
                .bind(name)
                .bind(role.as_str())
                .execute(&self.pool)
                .await
                .map_err(|e| e.to_string())?;
            Ok(())
        }

        async fn get(&self, id: &str) -> Result<Option<(String, String, AgentRole)>, String> {
            let row: Option<(String, String, String)> =
                sqlx::query_as("SELECT id, name, role FROM contract_agents WHERE id = ?")
                    .bind(id)
                    .fetch_optional(&self.pool)
                    .await
                    .map_err(|e| e.to_string())?;

            match row {
                Some((id, name, role_str)) => {
                    let role = AgentRole::try_from(role_str.as_str())?;
                    Ok(Some((id, name, role)))
                }
                None => Ok(None),
            }
        }

        async fn update_role(&self, id: &str, new_role: AgentRole) -> Result<bool, String> {
            let result = sqlx::query("UPDATE contract_agents SET role = ? WHERE id = ?")
                .bind(new_role.as_str())
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

        async fn list_all(&self) -> Result<Vec<(String, String, AgentRole)>, String> {
            let rows: Vec<(String, String, String)> =
                sqlx::query_as("SELECT id, name, role FROM contract_agents ORDER BY id")
                    .fetch_all(&self.pool)
                    .await
                    .map_err(|e| e.to_string())?;

            let mut out = Vec::with_capacity(rows.len());
            for (id, name, role_str) in rows {
                let role = AgentRole::try_from(role_str.as_str())?;
                out.push((id, name, role));
            }
            Ok(out)
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
            .insert("agent-1", "Alpha", AgentRole::Orchestrator)
            .await
            .unwrap();
        store
            .insert("agent-2", "Bravo", AgentRole::Specialist)
            .await
            .unwrap();
        store
            .insert("agent-3", "Charlie", AgentRole::Auditor)
            .await
            .unwrap();
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
        assert_eq!(
            role,
            AgentRole::Orchestrator,
            "[{}] Agent role match",
            backend_name
        );

        // 4. Get non-existing returns None (not error)
        let missing = store.get("agent-999").await.unwrap();
        assert!(
            missing.is_none(),
            "[{}] Get missing agent returns None",
            backend_name
        );

        // 5. Update existing
        let updated = store.update_role("agent-2", AgentRole::Lead).await.unwrap();
        assert!(updated, "[{}] Update existing returns true", backend_name);
        let agent2 = store.get("agent-2").await.unwrap().unwrap();
        assert_eq!(
            agent2.2,
            AgentRole::Lead,
            "[{}] Role updated correctly",
            backend_name
        );

        // 6. Update non-existing returns false
        let not_updated = store
            .update_role("agent-999", AgentRole::Worker)
            .await
            .unwrap();
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

        // 9. List all returns consistent, ordered results
        let all = store.list_all().await.unwrap();
        assert_eq!(all.len(), 2, "[{}] List all after operations", backend_name);
        assert!(
            all.windows(2).all(|w| w[0].0 <= w[1].0),
            "[{}] list_all rows must be deterministically ordered by id ascending",
            backend_name
        );

        // 10. Upsert / idempotent insert
        store
            .insert("agent-1", "Alpha-v2", AgentRole::Lead)
            .await
            .unwrap();
        let updated_agent = store.get("agent-1").await.unwrap().unwrap();
        assert_eq!(
            updated_agent.1, "Alpha-v2",
            "[{}] Upsert overwrites name",
            backend_name
        );
        assert_eq!(
            updated_agent.2,
            AgentRole::Lead,
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

    // ── Concurrency Contract ─────────────────────────────────

    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn test_concurrent_insert_no_data_loss() {
        let store = Arc::new(DashMapAgentStore::new());
        let mut join_set = JoinSet::new();

        for i in 0..100 {
            let s = Arc::clone(&store);
            join_set.spawn(async move {
                s.insert(
                    &format!("agent-{:03}", i),
                    &format!("Name-{}", i),
                    AgentRole::Worker,
                )
                .await
                .unwrap();
            });
        }

        while let Some(res) = join_set.join_next().await {
            res.unwrap();
        }

        assert_eq!(
            store.count().await.unwrap(),
            100,
            "All 100 concurrent inserts must be preserved"
        );
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn test_concurrent_read_write_safety_on_shared_keys() {
        let store = Arc::new(DashMapAgentStore::new());

        // Pre-populate 64 agents
        for i in 0..64 {
            store
                .insert(
                    &format!("agent-{:02}", i),
                    &format!("Name-{}", i),
                    AgentRole::Worker,
                )
                .await
                .unwrap();
        }

        let mut join_set = JoinSet::new();

        // Spawn concurrent readers reading all 64 keys
        for i in 0..64 {
            let s = Arc::clone(&store);
            join_set.spawn(async move {
                for _ in 0..200 {
                    let res = s.get(&format!("agent-{:02}", i)).await.unwrap();
                    assert!(
                        res.is_some(),
                        "Agent {} must remain readable under concurrent writes",
                        i
                    );
                }
            });
        }

        // Spawn concurrent writers modifying the SAME 64 keys
        for i in 0..64 {
            let s = Arc::clone(&store);
            join_set.spawn(async move {
                for _ in 0..200 {
                    let _ = s
                        .update_role(&format!("agent-{:02}", i), AgentRole::Lead)
                        .await
                        .unwrap();
                }
            });
        }

        while let Some(res) = join_set.join_next().await {
            res.unwrap();
        }

        assert_eq!(
            store.count().await.unwrap(),
            64,
            "All 64 agents must remain present and intact under active read/write contention"
        );
    }

    #[test]
    fn test_agent_role_vocabulary_validation() {
        let expected = ["orchestrator", "specialist", "auditor", "lead", "worker"];
        let actual: Vec<&str> = AgentRole::VARIANTS.iter().map(|r| r.as_str()).collect();
        assert_eq!(
            actual, expected,
            "AgentRole variants must match closed vocabulary"
        );

        for role_str in expected {
            assert!(AgentRole::try_from(role_str).is_ok());
        }

        assert!(
            AgentRole::try_from("ADMIN").is_err(),
            "arbitrary role strings must fail closed"
        );
        assert!(AgentRole::try_from("ghost").is_err());
    }
}

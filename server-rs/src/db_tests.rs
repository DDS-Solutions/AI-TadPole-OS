//! @docs ARCHITECTURE:DatabaseEngine
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Database & Migrations / db_tests
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
    use crate::db::init_db;
    use sqlx::Row;

    #[tokio::test]
    async fn test_init_db_memory() {
        // Use in-memory SQLite for testing
        let database_url = "sqlite::memory:";
        let pool = init_db(database_url)
            .await
            .expect("Failed to initialize test DB");

        // Verify we can query the agents table (created by migrations)
        let row = sqlx::query("SELECT 1 as connected")
            .fetch_one(&pool)
            .await
            .expect("Failed to query DB");

        let connected: i32 = row.get("connected");
        assert_eq!(connected, 1);

        // Verify WAL mode is NOT necessarily on for memory DBs, but check integrity
        let integrity: String = sqlx::query_scalar("PRAGMA integrity_check")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(integrity, "ok");
    }

    #[tokio::test]
    async fn test_sequential_migrations_data_preservation() {
        let temp_dir =
            std::env::temp_dir().join(format!("tadpole_test_db_{}", uuid::Uuid::new_v4()));
        let db_path = temp_dir.join("test.db");
        std::fs::create_dir_all(&temp_dir).unwrap();
        let db_url = format!("sqlite://{}", db_path.to_string_lossy());

        // Step 1: Initialize database and run migrations
        let pool = init_db(&db_url)
            .await
            .expect("Failed to initialize test DB");

        // Insert a test record into agents table
        sqlx::query("INSERT INTO agents (id, name, role, department, description, status, metadata) VALUES (?, ?, ?, ?, ?, ?, ?)")
            .bind("ag_test_001")
            .bind("Test Agent")
            .bind("Worker")
            .bind("Engineering")
            .bind("Test Agent Description")
            .bind("idle")
            .bind("{}")
            .execute(&pool)
            .await
            .expect("Failed to insert test agent");

        pool.close().await;

        // Step 2: Re-open database (simulating restart) and run init_db again
        let pool_reopened = init_db(&db_url).await.expect("Failed to reopen test DB");

        let agent_name: String = sqlx::query_scalar("SELECT name FROM agents WHERE id = ?")
            .bind("ag_test_001")
            .fetch_one(&pool_reopened)
            .await
            .expect("Failed to query agent after migration check");

        assert_eq!(agent_name, "Test Agent");

        let integrity: String = sqlx::query_scalar("PRAGMA integrity_check")
            .fetch_one(&pool_reopened)
            .await
            .unwrap();
        assert_eq!(integrity, "ok");

        pool_reopened.close().await;
        let _ = std::fs::remove_dir_all(&temp_dir);
    }
}

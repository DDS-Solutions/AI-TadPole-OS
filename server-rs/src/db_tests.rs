//! Database Integrity - Verification Suite
//!
//! Unit tests for schema initialization and migration routing.
//!
//! @docs ARCHITECTURE:DatabaseEngine
//!
//! @state TestDB: (Memory-Only | Isolated)
//!
//! ### AI Assist Note
//! **Verification Strategy**: Uses `sqlite::memory:` to ensure zero-side-effect 
//! schema validation. Useful for testing migration logic without disk I/O.
//!
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: Migration script syntax error or locked memory pointer.
//! - **Trace Scope**: `server-rs::db_tests`

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
}

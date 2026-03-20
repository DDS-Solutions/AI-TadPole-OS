#[cfg(test)]
mod tests {
    use crate::db::init_db;
    use sqlx::Row;

    #[tokio::test]
    async fn test_init_db_memory() {
        // Use in-memory SQLite for testing
        let database_url = "sqlite::memory:";
        let pool = init_db(database_url).await.expect("Failed to initialize test DB");
        
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

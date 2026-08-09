//! @docs ARCHITECTURE:Persistence
//!
//! ### AI Assist Note
//! **Engine Persistence (Database Layer)**: Orchestrates the lifecycle
//! of the **SQLite** connection pool and schema management for the
//! Tadpole OS engine. Decomposed into modular submodules (`init`, `migrations`, `seed`).
//!
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: Database locked (SQLITE_BUSY) during
//!   bursty write operations, migration checksum mismatches due to
//!   manual tampering, or path permission errors on the
//!   `DATABASE_URL` target.
//! - **Telemetry Link**: Search for `[Database]` or `[SQLx]` in
//!   `tracing` logs for query performance and migration status.
//! - **Trace Scope**: `server-rs::db`

pub mod init;
pub mod migrations;
pub mod seed;

#[allow(unused_imports)]
pub use init::{checkpoint_wal, init_db};
#[allow(unused_imports)]
pub use migrations::{
    CONNECTOR_COLUMN_FIX_MIGRATION_VERSION, CREATED_AT_FIX_MIGRATION_VERSION,
    CURRENT_TASK_FIX_MIGRATION_VERSION, INSTITUTIONAL_KNOWLEDGE_STORE_MIGRATION_VERSION,
    IKS_ADD_TEXT_COLUMN_MIGRATION_VERSION, MERKLE_AUDIT_TRAIL_MIGRATION_VERSION,
    SWARM_GRAPH_MIGRATION_VERSION,
};

#[cfg(test)]
mod tests {
    use super::migrations::run_migrations;
    use sqlx::SqlitePool;

    #[tokio::test]
    async fn test_db_pool_init() {
        let pool = SqlitePool::connect("sqlite::memory:")
            .await
            .expect("failed to open sqlite memory database");
        run_migrations(&pool)
            .await
            .expect("failed to run migrations in test");
    }
}

// Metadata: [db]

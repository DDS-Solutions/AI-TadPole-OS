//! @docs ARCHITECTURE:Persistence
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Database & Migrations / mod
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Type-safe state handling and bounded execution without unhandled panics.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: none declared
//! - **Witness Tests**: none declared

pub mod init;
pub mod migrations;
pub mod seed;

#[allow(unused_imports)]
pub use init::{checkpoint_wal, init_db};
#[allow(unused_imports)]
pub use migrations::{
    CONNECTOR_COLUMN_FIX_MIGRATION_VERSION, CREATED_AT_FIX_MIGRATION_VERSION,
    CURRENT_TASK_FIX_MIGRATION_VERSION, IKS_ADD_TEXT_COLUMN_MIGRATION_VERSION,
    INSTITUTIONAL_KNOWLEDGE_STORE_MIGRATION_VERSION, MERKLE_AUDIT_TRAIL_MIGRATION_VERSION,
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

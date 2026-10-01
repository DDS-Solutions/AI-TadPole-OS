//! @docs ARCHITECTURE:Persistence
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Database & Migrations / mod
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: none declared

pub mod init;
pub mod migrations;
pub mod seed;

#[cfg(test)]
mod contract_tests;

#[allow(unused_imports)]
pub use init::{
    checkpoint_wal, init_db, parse_database_url, production_options, strip_skip_seed_param,
    SeedPolicy,
};
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

    #[tokio::test]
    async fn test_db_pool_init() {
        let unique_id = uuid::Uuid::new_v4();
        let uri = format!(
            "file:test_db_pool_init_{}?mode=memory&cache=private",
            unique_id
        );
        let opts =
            crate::db::init::production_options(&uri).expect("valid production options for test");
        let pool = sqlx::sqlite::SqlitePoolOptions::new()
            .max_connections(1)
            .connect_with(opts)
            .await
            .expect("failed to open isolated sqlite memory database");
        run_migrations(&pool)
            .await
            .expect("failed to run migrations in test");
    }
}

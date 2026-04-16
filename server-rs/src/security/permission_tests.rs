//! @docs ARCHITECTURE:Core
//!
//! ### AI Assist Note
//! **Core technical module for the Tadpole OS hardened engine.**
//! This module implements high-fidelity logic for the Sovereign Reality layer.
//!
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: Runtime logic error, state desynchronization, or resource exhaustion.
//! - **Telemetry Link**: Search `[permission_tests.rs]` in tracing logs.
//!
//! @docs ARCHITECTURE:Security Governance Testing
//!
//! ### AI Assist Note
//! **Governance Logic Validation**: Ensures that the `PermissionPolicy` correctly
//! handles SQLite persistence, cache synchronization, and default safety modes (PERM-TEST-01).

use crate::security::permissions::{PermissionMode, PermissionPolicy};
use sqlx::sqlite::SqlitePoolOptions;

async fn setup_test_db() -> sqlx::SqlitePool {
    let pool = SqlitePoolOptions::new()
        .connect("sqlite::memory:")
        .await
        .expect("Failed to create in-memory DB");

    // Initialize the permission_policies table
    sqlx::query(
        "CREATE TABLE permission_policies (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            tool_name TEXT NOT NULL UNIQUE,
            mode TEXT NOT NULL CHECK(mode IN ('allow', 'deny', 'prompt')),
            updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
        )",
    )
    .execute(&pool)
    .await
    .expect("Failed to create table");

    pool
}

#[tokio::test]
async fn test_permission_policy_persistence() {
    let pool = setup_test_db().await;
    let policy = PermissionPolicy::new(pool.clone());

    // 1. Initially, unknown tool should default to Prompt (Sovereign Safety)
    assert_eq!(
        policy.get_mode("dangerous_tool").await,
        PermissionMode::Prompt
    );

    // 2. Insert a policy into the DB
    sqlx::query("INSERT INTO permission_policies (tool_name, mode) VALUES (?, ?)")
        .bind("read_file")
        .bind("allow")
        .execute(&pool)
        .await
        .expect("Failed to insert policy");

    // 3. get_mode should now return Allow (fetches from DB and caches)
    assert_eq!(policy.get_mode("read_file").await, PermissionMode::Allow);
}

#[tokio::test]
async fn test_permission_policy_cache_refresh() {
    let pool = setup_test_db().await;
    let policy = PermissionPolicy::new(pool.clone());

    // 1. Seed the DB
    sqlx::query("INSERT INTO permission_policies (tool_name, mode) VALUES (?, ?)")
        .bind("execute_shell")
        .bind("prompt")
        .execute(&pool)
        .await
        .unwrap();

    // 2. Warm the cache
    assert_eq!(
        policy.get_mode("execute_shell").await,
        PermissionMode::Prompt
    );

    // 3. Update the DB directly (simulating an external tool or OOB edit)
    sqlx::query("UPDATE permission_policies SET mode = 'deny' WHERE tool_name = 'execute_shell'")
        .execute(&pool)
        .await
        .unwrap();

    // 4. Mode should still be Prompt (from cache)
    assert_eq!(
        policy.get_mode("execute_shell").await,
        PermissionMode::Prompt
    );

    // 5. Refresh cache
    policy
        .refresh_cache()
        .await
        .expect("Failed to refresh cache");

    // 6. Mode should now be Deny
    assert_eq!(policy.get_mode("execute_shell").await, PermissionMode::Deny);
}

#[tokio::test]
async fn test_permission_policy_unknown_tool_safety() {
    let pool = setup_test_db().await;
    let policy = PermissionPolicy::new(pool.clone());

    // Default behavior for any unregistered tool must be Prompt
    assert_eq!(
        policy.get_mode("zero_day_exploit_tool").await,
        PermissionMode::Prompt
    );
}

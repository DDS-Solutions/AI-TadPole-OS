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

    sqlx::query(
        "CREATE TABLE agent_permission_policies (
            agent_id TEXT NOT NULL,
            tool_name TEXT NOT NULL,
            mode TEXT NOT NULL,
            PRIMARY KEY (agent_id, tool_name)
        )",
    )
    .execute(&pool)
    .await
    .expect("Failed to create agent policies table");

    sqlx::query(
        "CREATE TABLE role_permission_policies (
            role TEXT NOT NULL,
            tool_name TEXT NOT NULL,
            mode TEXT NOT NULL,
            PRIMARY KEY (role, tool_name)
        )",
    )
    .execute(&pool)
    .await
    .expect("Failed to create role policies table");

    pool
}

#[tokio::test]
async fn test_permission_policy_persistence() {
    let pool = setup_test_db().await;
    let policy = PermissionPolicy::new(pool.clone());

    // 1. Initially, unknown tool should default to Prompt (Sovereign Safety)
    assert_eq!(
        policy.get_mode(None, None, "dangerous_tool").await,
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
    assert_eq!(
        policy.get_mode(None, None, "read_file").await,
        PermissionMode::Allow
    );
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
        policy.get_mode(None, None, "execute_shell").await,
        PermissionMode::Prompt
    );

    // 3. Update the DB directly (simulating an external tool or OOB edit)
    sqlx::query("UPDATE permission_policies SET mode = 'deny' WHERE tool_name = 'execute_shell'")
        .execute(&pool)
        .await
        .unwrap();

    // 4. Mode should still be Prompt (from cache)
    assert_eq!(
        policy.get_mode(None, None, "execute_shell").await,
        PermissionMode::Prompt
    );

    // 5. Refresh cache
    policy
        .refresh_cache()
        .await
        .expect("Failed to refresh cache");

    // 6. Mode should now be Deny
    assert_eq!(
        policy.get_mode(None, None, "execute_shell").await,
        PermissionMode::Deny
    );
}

#[tokio::test]
async fn test_permission_policy_unknown_tool_safety() {
    let pool = setup_test_db().await;
    let policy = PermissionPolicy::new(pool.clone());

    // Default behavior for any unregistered tool must be Prompt
    assert_eq!(
        policy.get_mode(None, None, "zero_day_exploit_tool").await,
        PermissionMode::Prompt
    );
}

#[tokio::test]
async fn test_agent_and_role_permissions() {
    let pool = setup_test_db().await;
    let policy = PermissionPolicy::new(pool.clone());

    // Seed agent policy: agent-A can read_file (allow)
    sqlx::query(
        "INSERT INTO agent_permission_policies (agent_id, tool_name, mode) VALUES (?, ?, ?)",
    )
    .bind("agent-A")
    .bind("read_file")
    .bind("allow")
    .execute(&pool)
    .await
    .unwrap();

    // Seed role policy: role 'developer' has write_file (prompt)
    sqlx::query("INSERT INTO role_permission_policies (role, tool_name, mode) VALUES (?, ?, ?)")
        .bind("developer")
        .bind("write_file")
        .bind("prompt")
        .execute(&pool)
        .await
        .unwrap();

    // Global policy: read_file is prompt, write_file is deny
    sqlx::query("INSERT INTO permission_policies (tool_name, mode) VALUES (?, ?)")
        .bind("read_file")
        .bind("prompt")
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("INSERT INTO permission_policies (tool_name, mode) VALUES (?, ?)")
        .bind("write_file")
        .bind("deny")
        .execute(&pool)
        .await
        .unwrap();

    // Test agent-specific override
    assert_eq!(
        policy
            .get_mode(Some("agent-A"), Some("developer"), "read_file")
            .await,
        PermissionMode::Allow
    );
    // Other agent defaults to global policy (prompt)
    assert_eq!(
        policy
            .get_mode(Some("agent-B"), Some("developer"), "read_file")
            .await,
        PermissionMode::Prompt
    );

    // Test role-based policy
    assert_eq!(
        policy
            .get_mode(Some("agent-B"), Some("developer"), "write_file")
            .await,
        PermissionMode::Prompt
    );
    // Other role defaults to global policy (deny)
    assert_eq!(
        policy
            .get_mode(Some("agent-B"), Some("tester"), "write_file")
            .await,
        PermissionMode::Deny
    );
}

// Metadata: [permission_tests]

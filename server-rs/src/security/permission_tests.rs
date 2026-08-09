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

    sqlx::query(
        "CREATE TABLE capability_policies (
            capability_class TEXT NOT NULL,
            resource_pattern TEXT NOT NULL,
            mode TEXT NOT NULL,
            PRIMARY KEY (capability_class, resource_pattern)
        )",
    )
    .execute(&pool)
    .await
    .expect("Failed to create capability policies table");

    sqlx::query(
        "CREATE TABLE agent_capability_policies (
            agent_id TEXT NOT NULL,
            capability_class TEXT NOT NULL,
            resource_pattern TEXT NOT NULL,
            mode TEXT NOT NULL,
            PRIMARY KEY (agent_id, capability_class, resource_pattern)
        )",
    )
    .execute(&pool)
    .await
    .expect("Failed to create agent capability policies table");

    sqlx::query(
        "CREATE TABLE role_capability_policies (
            role TEXT NOT NULL,
            capability_class TEXT NOT NULL,
            resource_pattern TEXT NOT NULL,
            mode TEXT NOT NULL,
            PRIMARY KEY (role, capability_class, resource_pattern)
        )",
    )
    .execute(&pool)
    .await
    .expect("Failed to create role capability policies table");

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

#[tokio::test]
async fn test_capability_class_hierarchical_resolution() {
    use crate::security::permissions::CapabilityClass;

    let pool = setup_test_db().await;
    let policy = PermissionPolicy::new(pool.clone());

    // 1. Unregistered capability defaults to Prompt
    let mode = policy
        .check_capability(
            Some("agent-1"),
            Some("worker"),
            CapabilityClass::Install,
            "path:.agent/skills/web-scraper/index.js",
        )
        .await;
    assert_eq!(mode, PermissionMode::Prompt);

    // 2. Insert domain policy for 'domain:skills' -> Allow (via signed capability activation)
    policy
        .set_capability_mode_signed(CapabilityClass::Install, "domain:skills", PermissionMode::Allow)
        .await
        .unwrap();

    // 3. Resource in skills directory should resolve domain:skills -> Allow (clamped by SEC-06 floor to Prompt)
    let mode_domain = policy
        .check_capability(
            Some("agent-1"),
            Some("worker"),
            CapabilityClass::Install,
            "path:.agent/skills/web-scraper/index.js",
        )
        .await;
    assert_eq!(mode_domain, PermissionMode::Prompt);

    // 4. Specific path override for sensitive skill -> Deny
    policy
        .set_capability_mode_signed(
            CapabilityClass::Install,
            "path:.agent/skills/security-admin/index.js",
            PermissionMode::Deny,
        )
        .await
        .unwrap();

    let mode_path_deny = policy
        .check_capability(
            Some("agent-1"),
            Some("worker"),
            CapabilityClass::Install,
            "path:.agent/skills/security-admin/index.js",
        )
        .await;
    assert_eq!(mode_path_deny, PermissionMode::Deny);
}

#[tokio::test]
async fn test_legacy_update_invalidates_execute_capability_cache() {
    use crate::security::permissions::CapabilityClass;

    let pool = setup_test_db().await;
    let policy = PermissionPolicy::new(pool.clone());

    // 1. Insert legacy tool policy: "legacy_tool" -> Allow
    policy.set_mode("legacy_tool", PermissionMode::Allow).await.unwrap();

    // 2. Query capability Execute for "legacy_tool" (falls back to get_mode Allow, clamped to Prompt under SEC-06)
    let initial_mode = policy
        .check_capability(None, None, CapabilityClass::Execute, "legacy_tool")
        .await;
    assert_eq!(initial_mode, PermissionMode::Prompt);

    // 3. Admin updates legacy tool policy to Deny via set_mode
    policy.set_mode("legacy_tool", PermissionMode::Deny).await.unwrap();

    // 4. Query capability Execute again -- MUST return Deny (proves capability Execute cache was invalidated)
    let updated_mode = policy
        .check_capability(None, None, CapabilityClass::Execute, "legacy_tool")
        .await;
    assert_eq!(updated_mode, PermissionMode::Deny);
}

#[tokio::test]
async fn test_agent_capability_precedence() {
    use crate::security::permissions::{CapabilityClass, PermissionDecisionSource};

    let pool = setup_test_db().await;
    let policy = PermissionPolicy::new(pool.clone());

    // Seed agent policy: Agent-Alpha -> Allow
    sqlx::query(
        "INSERT INTO agent_capability_policies (agent_id, capability_class, resource_pattern, mode) VALUES (?, ?, ?, ?)",
    )
    .bind("Agent-Alpha")
    .bind("Execute")
    .bind("domain:execution")
    .bind("allow")
    .execute(&pool)
    .await
    .unwrap();

    // Seed role policy: Role 'analyst' -> Deny
    sqlx::query(
        "INSERT INTO role_capability_policies (role, capability_class, resource_pattern, mode) VALUES (?, ?, ?, ?)",
    )
    .bind("analyst")
    .bind("Execute")
    .bind("domain:execution")
    .bind("deny")
    .execute(&pool)
    .await
    .unwrap();

    // Agent-Alpha (with role analyst) gets Agent-specific Allow
    let dec_alpha = policy
        .check_capability_decision(Some("Agent-Alpha"), Some("analyst"), CapabilityClass::Execute, "domain:execution")
        .await;
    assert_eq!(dec_alpha.mode, PermissionMode::Prompt);
    assert_eq!(dec_alpha.source, PermissionDecisionSource::SecurityFloor);

    // Agent-Beta (without agent policy, with role analyst) gets Role-specific Deny
    let dec_beta = policy
        .check_capability_decision(Some("Agent-Beta"), Some("analyst"), CapabilityClass::Execute, "domain:execution")
        .await;
    assert_eq!(dec_beta.mode, PermissionMode::Deny);
}

#[tokio::test]
async fn test_permission_decision_metadata_attribution() {
    use crate::security::permissions::{CapabilityClass, PermissionDecisionSource};

    let pool = setup_test_db().await;
    let policy = PermissionPolicy::new(pool.clone());

    // Signed global capability policy override
    policy
        .set_capability_mode_signed(CapabilityClass::Install, "domain:skills", PermissionMode::Allow)
        .await
        .unwrap();

    let decision = policy
        .check_capability_decision(None, None, CapabilityClass::Install, "path:.agent/skills/test")
        .await;

    // Evaluated Allow for Install capability is clamped by SEC-06 mandatory floor to Prompt
    assert_eq!(decision.mode, PermissionMode::Prompt);
    assert_eq!(decision.source, PermissionDecisionSource::SecurityFloor);
    assert!(decision.reason.is_some());
}

#[tokio::test]
async fn test_mandatory_security_floor_rejects_weaker_writes() {
    use crate::security::permissions::CapabilityClass;

    let pool = setup_test_db().await;
    let policy = PermissionPolicy::new(pool.clone());

    // Attempting to set Allow for Execute capability fails SEC-06 floor validation
    let res_exec = policy
        .set_capability_mode(CapabilityClass::Execute, "domain:shell", PermissionMode::Allow)
        .await;
    assert!(res_exec.is_err());
    assert!(res_exec.unwrap_err().to_string().contains("violates SEC-06 mandatory security floor"));

    // Attempting to set Allow or Prompt for Approve capability fails SEC-06 floor validation
    let res_approve = policy
        .set_capability_mode(CapabilityClass::Approve, "domain:vault", PermissionMode::Prompt)
        .await;
    assert!(res_approve.is_err());
    assert!(res_approve.unwrap_err().to_string().contains("violates SEC-06 mandatory security floor"));

    // Tightening to Deny succeeds
    let res_deny = policy
        .set_capability_mode(CapabilityClass::Execute, "domain:shell", PermissionMode::Deny)
        .await;
    assert!(res_deny.is_ok());
}

// Metadata: [permission_tests]

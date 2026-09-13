//! @docs ARCHITECTURE:Core
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Security & Governance / permission_tests
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Type-safe state handling and bounded execution without unhandled panics.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: none declared
//! - **Witness Tests**: none declared

use crate::security::permissions::{
    CapabilityClass, PermissionDecisionSource, PermissionMode, PermissionPolicy,
};
use crate::security::signed_capability::SignedCapabilityManifest;
use chrono::Utc;
use ed25519_dalek::SigningKey;
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

    sqlx::query(
        "CREATE TABLE signed_capability_manifests (
            capability_id TEXT PRIMARY KEY,
            version TEXT NOT NULL,
            owner TEXT NOT NULL,
            creator TEXT NOT NULL,
            created_at DATETIME NOT NULL,
            content_hash TEXT NOT NULL,
            signature TEXT NOT NULL,
            approval_id TEXT,
            expiration DATETIME,
            risk_class TEXT NOT NULL,
            resource_pattern TEXT NOT NULL,
            status TEXT NOT NULL
        )",
    )
    .execute(&pool)
    .await
    .expect("Failed to create signed_capability_manifests table");

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

    // 2. Insert domain policy for 'domain:skills' -> Prompt (floor compliant)
    policy
        .set_capability_mode(
            CapabilityClass::Install,
            "domain:skills",
            PermissionMode::Prompt,
        )
        .await
        .unwrap();

    // 3. Resource in skills directory resolves domain:skills -> Prompt
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
        .set_capability_mode(
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
    let pool = setup_test_db().await;
    let policy = PermissionPolicy::new(pool.clone());

    // 1. Insert legacy tool policy: "legacy_tool" -> Allow
    policy
        .set_mode("legacy_tool", PermissionMode::Allow)
        .await
        .unwrap();

    // 2. Query capability Execute for "legacy_tool" (falls back to get_mode Allow, clamped to Prompt under SEC-06)
    let initial_mode = policy
        .check_capability(None, None, CapabilityClass::Execute, "legacy_tool")
        .await;
    assert_eq!(initial_mode, PermissionMode::Prompt);

    // 3. Admin updates legacy tool policy to Deny via set_mode
    policy
        .set_mode("legacy_tool", PermissionMode::Deny)
        .await
        .unwrap();

    // 4. Query capability Execute again -- MUST return Deny (proves capability Execute cache was invalidated)
    let updated_mode = policy
        .check_capability(None, None, CapabilityClass::Execute, "legacy_tool")
        .await;
    assert_eq!(updated_mode, PermissionMode::Deny);
}

#[tokio::test]
async fn test_agent_capability_precedence() {
    let pool = setup_test_db().await;
    let policy = PermissionPolicy::new(pool.clone());

    // Seed agent policy: Agent-Alpha -> Allow
    sqlx::query(
        "INSERT INTO agent_capability_policies (agent_id, capability_class, resource_pattern, mode) VALUES (?, ?, ?, ?)",
    )
    .bind("Agent-Alpha")
    .bind("execute")
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
    .bind("execute")
    .bind("domain:execution")
    .bind("deny")
    .execute(&pool)
    .await
    .unwrap();

    // Agent-Alpha (with role analyst) gets Agent-specific Allow, clamped to Prompt by SEC-06
    let dec_alpha = policy
        .check_capability_decision(
            Some("Agent-Alpha"),
            Some("analyst"),
            CapabilityClass::Execute,
            "domain:execution",
        )
        .await;
    assert_eq!(dec_alpha.mode, PermissionMode::Prompt);
    assert_eq!(dec_alpha.source, PermissionDecisionSource::SecurityFloor);

    // Agent-Beta (without agent policy, with role analyst) gets Role-specific Deny
    let dec_beta = policy
        .check_capability_decision(
            Some("Agent-Beta"),
            Some("analyst"),
            CapabilityClass::Execute,
            "domain:execution",
        )
        .await;
    assert_eq!(dec_beta.mode, PermissionMode::Deny);
    assert_eq!(dec_beta.source, PermissionDecisionSource::RolePolicy);
}

#[tokio::test]
async fn test_permission_decision_metadata_attribution() {
    let pool = setup_test_db().await;
    let policy = PermissionPolicy::new(pool.clone());

    // Global capability policy override
    policy
        .set_capability_mode(
            CapabilityClass::Install,
            "domain:skills",
            PermissionMode::Prompt,
        )
        .await
        .unwrap();

    let decision = policy
        .check_capability_decision(
            None,
            None,
            CapabilityClass::Install,
            "path:.agent/skills/test",
        )
        .await;

    assert_eq!(decision.mode, PermissionMode::Prompt);
    assert_eq!(decision.source, PermissionDecisionSource::DomainPolicy);
    assert!(decision.reason.is_some());
}

#[tokio::test]
async fn test_mandatory_security_floor_rejects_weaker_writes() {
    let pool = setup_test_db().await;
    let policy = PermissionPolicy::new(pool.clone());

    // Attempting to set Allow for Execute capability fails SEC-06 floor validation
    let res_exec = policy
        .set_capability_mode(
            CapabilityClass::Execute,
            "domain:shell",
            PermissionMode::Allow,
        )
        .await;
    assert!(res_exec.is_err());
    assert!(res_exec
        .unwrap_err()
        .to_string()
        .contains("violates SEC-06 mandatory security floor"));

    // Attempting to set Allow or Prompt for Approve capability fails SEC-06 floor validation
    let res_approve = policy
        .set_capability_mode(
            CapabilityClass::Approve,
            "domain:vault",
            PermissionMode::Prompt,
        )
        .await;
    assert!(res_approve.is_err());
    assert!(res_approve
        .unwrap_err()
        .to_string()
        .contains("violates SEC-06 mandatory security floor"));

    // Tightening to Deny succeeds
    let res_deny = policy
        .set_capability_mode(
            CapabilityClass::Execute,
            "domain:shell",
            PermissionMode::Deny,
        )
        .await;
    assert!(res_deny.is_ok());
}

#[tokio::test]
async fn test_authorization_precedence_hierarchy_matrix() {
    let pool = setup_test_db().await;
    let policy = PermissionPolicy::new(pool.clone());

    // 1. Unmapped capability defaults to Prompt (DefaultPrompt source)
    let dec_default = policy
        .check_capability_decision(None, None, CapabilityClass::Execute, "custom_unmapped_tool")
        .await;
    assert_eq!(dec_default.mode, PermissionMode::Prompt);
    assert_eq!(dec_default.source, PermissionDecisionSource::DefaultPrompt);

    // 2. Legacy policy override applies to Execute fallback
    policy
        .set_mode("custom_unmapped_tool", PermissionMode::Deny)
        .await
        .unwrap();
    let dec_legacy = policy
        .check_capability_decision(None, None, CapabilityClass::Execute, "custom_unmapped_tool")
        .await;
    assert_eq!(dec_legacy.mode, PermissionMode::Deny);
    assert_eq!(
        dec_legacy.source,
        PermissionDecisionSource::GlobalLegacyPolicy
    );

    // 3. Global Capability Policy takes precedence over Legacy Policy
    policy
        .set_capability_mode(
            CapabilityClass::Execute,
            "custom_unmapped_tool",
            PermissionMode::Prompt,
        )
        .await
        .unwrap();
    let dec_global = policy
        .check_capability_decision(None, None, CapabilityClass::Execute, "custom_unmapped_tool")
        .await;
    assert_eq!(
        dec_global.source,
        PermissionDecisionSource::GlobalCapabilityPolicy
    );

    // 4. Role-specific Capability Policy takes precedence over Global Capability Policy
    policy
        .set_role_capability_mode(
            "admin_role",
            CapabilityClass::Execute,
            "custom_unmapped_tool",
            PermissionMode::Deny,
        )
        .await
        .unwrap();
    let dec_role = policy
        .check_capability_decision(
            None,
            Some("admin_role"),
            CapabilityClass::Execute,
            "custom_unmapped_tool",
        )
        .await;
    assert_eq!(dec_role.mode, PermissionMode::Deny);
    assert_eq!(dec_role.source, PermissionDecisionSource::RolePolicy);

    // 5. Agent-specific Capability Policy takes precedence over Role Policy
    policy
        .set_agent_capability_mode(
            "special_agent",
            CapabilityClass::Execute,
            "custom_unmapped_tool",
            PermissionMode::Prompt,
        )
        .await
        .unwrap();
    let dec_agent = policy
        .check_capability_decision(
            Some("special_agent"),
            Some("admin_role"),
            CapabilityClass::Execute,
            "custom_unmapped_tool",
        )
        .await;
    assert_eq!(dec_agent.mode, PermissionMode::Prompt);
    assert_eq!(dec_agent.source, PermissionDecisionSource::AgentPolicy);
}

#[tokio::test]
async fn test_signed_manifest_floor_override_pipeline() {
    let pool = setup_test_db().await;
    let policy = PermissionPolicy::new(pool.clone());

    let signing_key = SigningKey::generate(&mut rand::rng());
    let verifying_key = signing_key.verifying_key();
    let target_bytes = b"console.log('Safe');";
    let content_hash = SignedCapabilityManifest::compute_hash(target_bytes);

    let mut manifest = SignedCapabilityManifest::sign(
        &signing_key,
        "cap-override-1".to_string(),
        "1.0.0".to_string(),
        "agent-admin".to_string(),
        "creator".to_string(),
        Utc::now(),
        content_hash,
        Some("appr-123".to_string()),
        Some(Utc::now() + chrono::Duration::hours(1)),
        CapabilityClass::Execute,
        "path:scripts/deploy_tool.js".to_string(),
        PermissionMode::Allow,
    );

    // Activate through verified cryptographic pipeline
    manifest
        .verify_and_activate(&verifying_key, target_bytes, &policy)
        .await
        .expect("Verification must succeed");

    // Floor (Prompt) is overridden to Allow because manifest is active & verified
    let mode = policy
        .check_capability(
            None,
            None,
            CapabilityClass::Execute,
            "path:scripts/deploy_tool.js",
        )
        .await;
    assert_eq!(mode, PermissionMode::Allow);
}

#[tokio::test]
async fn test_sql_like_wildcard_matching_in_manifests() {
    let pool = setup_test_db().await;
    let policy = PermissionPolicy::new(pool.clone());

    // Insert active manifest with SQL wildcard pattern: path:tools/special_%/run
    sqlx::query(
        "INSERT INTO signed_capability_manifests (capability_id, version, owner, creator, created_at, content_hash, signature, approval_id, expiration, risk_class, resource_pattern, status) \
         VALUES ('cap-wild-1', '1.0', 'admin', 'admin', CURRENT_TIMESTAMP, 'hash', 'sig', 'appr', NULL, 'execute', 'path:tools/special_%/run', 'active')"
    )
    .execute(&pool)
    .await
    .unwrap();

    // Resource with underscores: path:tools/special_test_1/run matches pattern
    assert!(
        policy
            .is_signed_capability_active(CapabilityClass::Execute, "path:tools/special_test_1/run")
            .await
    );

    // Resource not matching prefix does not match
    assert!(
        !policy
            .is_signed_capability_active(CapabilityClass::Execute, "path:tools/other/run")
            .await
    );
}

#[test]
fn test_domain_inference_strict_path_anchors() {
    assert_eq!(
        PermissionPolicy::infer_domain(".agent/skills/git-sync/SKILL.md"),
        Some("domain:skills".to_string())
    );
    assert_eq!(
        PermissionPolicy::infer_domain("skills/search/SKILL.md"),
        Some("domain:skills".to_string())
    );
    assert_eq!(
        PermissionPolicy::infer_domain("directives/deploy.md"),
        Some("domain:directives".to_string())
    );
    assert_eq!(
        PermissionPolicy::infer_domain("execution/run.py"),
        Some("domain:execution".to_string())
    );
    assert_eq!(
        PermissionPolicy::infer_domain("server-rs/src/main.rs"),
        Some("domain:system".to_string())
    );
    // Path containing 'server-rs' or 'skills' deep in subdirectories does not falsely claim root system
    assert_eq!(
        PermissionPolicy::infer_domain("user_project/nested/server-rs/foo.txt"),
        None
    );
}

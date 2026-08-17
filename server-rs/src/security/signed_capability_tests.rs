//! @docs SEC-05: Signed Capability Manifest Unit Tests
//!
//! ### AI Assist Note
//! **Signed Capability Tests**: Verifies Ed25519 cryptographic signing and verification of SEC-05 manifests.
//!
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: Cryptographic verification failure or SQLite memory pool error.
//! - **Telemetry Link**: Search `[signed_capability]` in tracing logs.

use crate::security::permissions::{CapabilityClass, PermissionMode, PermissionPolicy};
use crate::security::signed_capability::{ManifestStatus, SignedCapabilityManifest};
use chrono::{Duration, Utc};
use ed25519_dalek::SigningKey;
use sqlx::sqlite::SqlitePoolOptions;

async fn setup_test_env() -> (sqlx::SqlitePool, PermissionPolicy) {
    // [signed_capability] test environment setup
    let pool = SqlitePoolOptions::new()
        .connect("sqlite::memory:")
        .await
        .expect("Failed to create in-memory DB");

    sqlx::query(
        "CREATE TABLE capability_policies (
            capability_class TEXT NOT NULL,
            resource_pattern TEXT NOT NULL,
            mode TEXT NOT NULL CHECK(mode IN ('allow', 'deny', 'prompt')),
            PRIMARY KEY (capability_class, resource_pattern)
        )",
    )
    .execute(&pool)
    .await
    .expect("Failed to create capability_policies table");

    sqlx::query(
        "CREATE TABLE agent_capability_policies (
            agent_id TEXT NOT NULL,
            capability_class TEXT NOT NULL,
            resource_pattern TEXT NOT NULL,
            mode TEXT NOT NULL CHECK(mode IN ('allow', 'deny', 'prompt')),
            PRIMARY KEY (agent_id, capability_class, resource_pattern)
        )",
    )
    .execute(&pool)
    .await
    .expect("Failed to create agent_capability_policies table");

    sqlx::query(
        "CREATE TABLE role_capability_policies (
            role TEXT NOT NULL,
            capability_class TEXT NOT NULL,
            resource_pattern TEXT NOT NULL,
            mode TEXT NOT NULL CHECK(mode IN ('allow', 'deny', 'prompt')),
            PRIMARY KEY (role, capability_class, resource_pattern)
        )",
    )
    .execute(&pool)
    .await
    .expect("Failed to create role_capability_policies table");

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

    let policy = PermissionPolicy::new(pool.clone());
    (pool, policy)
}

#[tokio::test]
async fn test_signed_capability_valid_activation() {
    let (_pool, policy) = setup_test_env().await;

    // Generate Ed25519 keypair
    let signing_key = SigningKey::generate(&mut rand::rng());
    let verifying_key = signing_key.verifying_key();

    let target_script = b"console.log('Secure execution');";
    let content_hash = SignedCapabilityManifest::compute_hash(target_script);

    let mut manifest = SignedCapabilityManifest::sign(
        &signing_key,
        "cap-uuid-1".to_string(),
        "1.0.0".to_string(),
        "agent-alpha".to_string(),
        "human-admin".to_string(),
        Utc::now(),
        content_hash,
        Some("ledger-approval-99".to_string()),
        Some(Utc::now() + Duration::hours(1)),
        CapabilityClass::Execute,
        "path:.agent/skills/safe-script/index.js".to_string(),
        PermissionMode::Allow,
    );

    // Initial state is Pending
    assert_eq!(manifest.status, ManifestStatus::Pending);

    // Verify & Activate
    let res = manifest
        .verify_and_activate(&verifying_key, target_script, &policy)
        .await;
    assert!(res.is_ok());
    assert_eq!(manifest.status, ManifestStatus::Active);

    // Check policy mode in PermissionPolicy
    let mode = policy
        .check_capability(
            Some("agent-alpha"),
            None,
            CapabilityClass::Execute,
            "path:.agent/skills/safe-script/index.js",
        )
        .await;
    assert_eq!(mode, PermissionMode::Allow);
}

#[tokio::test]
async fn test_signed_capability_tampered_content_fails() {
    let (_pool, policy) = setup_test_env().await;

    let signing_key = SigningKey::generate(&mut rand::rng());
    let verifying_key = signing_key.verifying_key();

    let original_script = b"console.log('Original approved script');";
    let content_hash = SignedCapabilityManifest::compute_hash(original_script);

    let mut manifest = SignedCapabilityManifest::sign(
        &signing_key,
        "cap-uuid-2".to_string(),
        "1.0.0".to_string(),
        "agent-alpha".to_string(),
        "human-admin".to_string(),
        Utc::now(),
        content_hash,
        Some("ledger-approval-100".to_string()),
        None,
        CapabilityClass::Install,
        "domain:skills".to_string(),
        PermissionMode::Allow,
    );

    // Tampered script bytes
    let tampered_script = b"console.log('Tampered script payload!');";

    let res = manifest
        .verify_and_activate(&verifying_key, tampered_script, &policy)
        .await;
    assert!(res.is_err());
    let err_msg = res.unwrap_err().to_string();
    assert!(err_msg.contains("Content hash mismatch"));
}

#[tokio::test]
async fn test_signed_capability_bad_signature_fails() {
    let (_pool, policy) = setup_test_env().await;

    let signing_key = SigningKey::generate(&mut rand::rng());
    let wrong_key = SigningKey::generate(&mut rand::rng()).verifying_key();

    let target_script = b"function test() {}";
    let content_hash = SignedCapabilityManifest::compute_hash(target_script);

    let mut manifest = SignedCapabilityManifest::sign(
        &signing_key,
        "cap-uuid-3".to_string(),
        "1.0.0".to_string(),
        "agent-alpha".to_string(),
        "human-admin".to_string(),
        Utc::now(),
        content_hash,
        Some("ledger-approval-101".to_string()),
        None,
        CapabilityClass::Modify,
        "domain:directives".to_string(),
        PermissionMode::Allow,
    );

    let res = manifest
        .verify_and_activate(&wrong_key, target_script, &policy)
        .await;
    assert!(res.is_err());
    let err_msg = res.unwrap_err().to_string();
    assert!(err_msg.contains("Ed25519 signature verification failed"));
}

#[tokio::test]
async fn test_signed_capability_expired_fails() {
    let (_pool, policy) = setup_test_env().await;

    let signing_key = SigningKey::generate(&mut rand::rng());
    let verifying_key = signing_key.verifying_key();

    let target_script = b"echo 1";
    let content_hash = SignedCapabilityManifest::compute_hash(target_script);
    let past_time = Utc::now() - Duration::hours(2);

    let mut manifest = SignedCapabilityManifest::sign(
        &signing_key,
        "cap-uuid-4".to_string(),
        "1.0.0".to_string(),
        "agent-alpha".to_string(),
        "human-admin".to_string(),
        past_time - Duration::hours(1),
        content_hash,
        Some("ledger-approval-102".to_string()),
        Some(past_time),
        CapabilityClass::Delete,
        "domain:system".to_string(),
        PermissionMode::Allow,
    );

    let res = manifest
        .verify_and_activate(&verifying_key, target_script, &policy)
        .await;
    assert!(res.is_err());
    assert_eq!(manifest.status, ManifestStatus::Expired);
    let err_msg = res.unwrap_err().to_string();
    assert!(err_msg.contains("expired"));
}

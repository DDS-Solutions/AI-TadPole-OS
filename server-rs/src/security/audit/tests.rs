//! @docs SECURITY:Core
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Security & Governance / audit / tests
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Type-safe state handling and bounded execution without unhandled panics.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: none declared

use super::writer::{load_signing_key_inner, migrate_schema};
use super::*;
use ed25519_dalek::SigningKey;

use super::*;
use rand::RngExt;
use std::sync::Arc;

async fn fresh_pool() -> SqlitePool {
    let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
    migrate_schema(&pool).await.unwrap();
    pool
}

fn random_key() -> SecretBox<SecretSigningKey> {
    let mut bytes = [0u8; 32];
    let mut rng = rand::rng();
    rng.fill(&mut bytes);
    SecretBox::new(Box::new(SecretSigningKey(SigningKey::from_bytes(&bytes))))
}

#[tokio::test]
async fn test_audit_trail_chaining() {
    let pool = fresh_pool().await;
    let audit = MerkleAuditTrail::with_signing_key(pool, None).unwrap();

    let e1 = audit
        .record(
            "agent-1",
            Some("mission-1"),
            Some("user-1"),
            "tool_call",
            "{\"tool\":\"ls\"}",
        )
        .await
        .unwrap();
    assert_eq!(e1.prev_hash, GENESIS_PREV_HASH);
    assert_eq!(e1.seq, Some(1));

    let e2 = audit
        .record(
            "agent-1",
            Some("mission-1"),
            Some("user-1"),
            "tool_call",
            "{\"tool\":\"cat\"}",
        )
        .await
        .unwrap();
    assert_eq!(e2.prev_hash, e1.current_hash);
    assert_eq!(e2.seq, Some(2));

    assert!(audit.verify_chain(None).await.unwrap());
}

#[tokio::test]
async fn test_audit_tamper_detection() {
    let pool = fresh_pool().await;
    let audit = MerkleAuditTrail::with_signing_key(pool, None).unwrap();

    audit
        .record("agent-1", None, None, "action-1", "{}")
        .await
        .unwrap();
    let e2 = audit
        .record("agent-1", None, None, "action-2", "{}")
        .await
        .unwrap();
    assert!(audit.verify_chain(None).await.unwrap());

    // Tamper with action.
    sqlx::query("UPDATE audit_trail SET action = 'tampered' WHERE id = ?")
        .bind(&e2.id)
        .execute(&audit.pool)
        .await
        .unwrap();

    assert!(!audit.verify_chain(None).await.unwrap());
}

#[tokio::test]
async fn test_signature_verification() {
    let pool = fresh_pool().await;
    let key = random_key();
    let audit = MerkleAuditTrail::with_signing_key(pool, Some(key.clone())).unwrap();

    let entry = audit
        .record("agent-1", None, None, "action", "{}")
        .await
        .unwrap();
    assert!(entry.signature.is_some());
    assert!(audit.verify_chain(None).await.unwrap());
    assert!(audit.verify_record(&entry));

    let sig_key = &key.expose_secret().0;
    let pk_hex = hex::encode(sig_key.verifying_key().to_bytes());
    assert!(audit.verify_chain(Some(&pk_hex)).await.unwrap());
}

#[tokio::test]
async fn test_optional_field_presence_detected() {
    let pool = fresh_pool().await;
    let audit = MerkleAuditTrail::with_signing_key(pool, None).unwrap();

    let entry = audit
        .record("agent-1", Some("mission-x"), None, "action", "{}")
        .await
        .unwrap();

    // Tamper optional mission_id
    sqlx::query("UPDATE audit_trail SET mission_id = NULL WHERE id = ?")
        .bind(&entry.id)
        .execute(&audit.pool)
        .await
        .unwrap();

    assert!(!audit.verify_chain(None).await.unwrap());
}

#[tokio::test]
async fn test_concurrent_writers_no_fork() {
    let pool = fresh_pool().await;
    let audit = Arc::new(MerkleAuditTrail::with_signing_key(pool, None).unwrap());

    let mut handles = Vec::new();
    for i in 0..1000 {
        let audit = Arc::clone(&audit);
        handles.push(tokio::spawn(async move {
            audit
                .record("agent", None, None, "noop", &format!("{{\"i\":{}}}", i))
                .await
                .unwrap();
        }));
    }
    for h in handles {
        h.await.unwrap();
    }

    let (verified, total) = audit.verify_last_n(1000, None).await.unwrap();
    assert_eq!(verified, total);
    assert_eq!(total, 1000);
    assert!(audit.verify_chain(None).await.unwrap());
}

#[tokio::test]
async fn test_empty_and_oversized_fields_rejected() {
    let pool = fresh_pool().await;
    let audit = MerkleAuditTrail::with_signing_key(pool, None).unwrap();
    assert!(audit.record("", None, None, "action", "{}").await.is_err());
    assert!(audit.record("agent", None, None, "", "{}").await.is_err());

    // Oversized params
    let huge_params = "x".repeat(MAX_PARAMS_BYTES + 1);
    assert!(audit
        .record("agent", None, None, "action", &huge_params)
        .await
        .is_err());
}

#[tokio::test]
async fn test_prefix_truncation_detected() {
    let pool = fresh_pool().await;
    let audit = MerkleAuditTrail::with_signing_key(pool, None).unwrap();

    let e1 = audit
        .record("agent-1", None, None, "action-1", "{}")
        .await
        .unwrap();
    let _e2 = audit
        .record("agent-1", None, None, "action-2", "{}")
        .await
        .unwrap();
    let _e3 = audit
        .record("agent-1", None, None, "action-3", "{}")
        .await
        .unwrap();

    assert!(audit.verify_chain(None).await.unwrap());

    // Delete genesis entry (prefix truncation)
    sqlx::query("DELETE FROM audit_trail WHERE id = ?")
        .bind(&e1.id)
        .execute(&audit.pool)
        .await
        .unwrap();

    // verify_chain must detect missing genesis anchor
    assert!(!audit.verify_chain(None).await.unwrap());
}

#[test]
fn test_canonical_hash_deterministic_across_threads() {
    let h1 = std::thread::spawn(|| {
        compute_canonical_hash(
            GENESIS_PREV_HASH,
            "a",
            Some("m"),
            Some("u"),
            "act",
            "{}",
            "2024-01-01T00:00:00Z",
        )
    });
    let h2 = std::thread::spawn(|| {
        compute_canonical_hash(
            GENESIS_PREV_HASH,
            "a",
            Some("m"),
            Some("u"),
            "act",
            "{}",
            "2024-01-01T00:00:00Z",
        )
    });
    assert_eq!(h1.join().unwrap(), h2.join().unwrap());
}

#[tokio::test]
async fn test_audit_require_signed_flag_matrix() {
    // Scenario 1: Key unset, REQUIRE_SIGNED=0 -> unsigned mode
    let res1 = load_signing_key_inner(None, false).unwrap();
    assert!(res1.is_none());

    // Scenario 2: Key unset, REQUIRE_SIGNED=1 -> fail closed
    let res2 = load_signing_key_inner(None, true);
    assert!(res2.is_err());

    // Scenario 3: Key malformed, REQUIRE_SIGNED=0 -> unsigned mode
    let res3 = load_signing_key_inner(Some("not-valid-hex"), false).unwrap();
    assert!(res3.is_none());

    // Scenario 4: Key malformed, REQUIRE_SIGNED=1 -> fail closed
    let res4 = load_signing_key_inner(Some("not-valid-hex"), true);
    assert!(res4.is_err());

    // Scenario 5: Valid key
    let valid_key_hex = hex::encode([7u8; 32]);
    let res5 = load_signing_key_inner(Some(&valid_key_hex), true).unwrap();
    assert!(res5.is_some());
}

#[tokio::test]
async fn test_is_alive_probe() {
    let pool = fresh_pool().await;
    let audit = MerkleAuditTrail::with_signing_key(pool, None).unwrap();
    assert!(audit.is_alive());
}

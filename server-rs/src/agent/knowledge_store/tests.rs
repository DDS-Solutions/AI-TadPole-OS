//! @docs ARCHITECTURE:Registry
//! 
//! ### AI Assist Note
//! **Verification and quality assurance for the Tadpole OS engine.**
//! This module implements high-fidelity logic for the Sovereign Reality layer.
//! 
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: Runtime logic error, state desynchronization, or resource exhaustion.
//! - **Telemetry Link**: Search `[tests]` in tracing logs.

use super::store::KnowledgeStore;
use super::types::DEFAULT_TTL_DAYS;

/// SHA-256 hash must be stable across identical inputs.
#[test]
fn test_sha256_hash_stable() {
    let h1 = KnowledgeStore::sha256_hash("hello world");
    let h2 = KnowledgeStore::sha256_hash("hello world");
    assert_eq!(h1, h2);
    assert!(!h1.is_empty());
}

/// Different text must produce different hashes.
#[test]
fn test_sha256_hash_distinct() {
    let h1 = KnowledgeStore::sha256_hash("apple");
    let h2 = KnowledgeStore::sha256_hash("orange");
    assert_ne!(h1, h2);
}

/// SHA-256 output must be a valid 64-char hex string.
#[test]
fn test_sha256_hash_format() {
    let h = KnowledgeStore::sha256_hash("test");
    assert_eq!(h.len(), 64, "SHA-256 hex must be 64 chars");
    assert!(h.chars().all(|c| c.is_ascii_hexdigit()));
}

/// Agent-written entries default to 90-day TTL.
#[test]
fn test_agent_entry_gets_90_day_ttl() {
    let now = 1_000_000_i64;
    let ttl = KnowledgeStore::compute_ttl(false, None, now);
    assert_eq!(ttl, Some(now + DEFAULT_TTL_DAYS * 86_400));
}

/// Human-confirmed entries never expire (ttl = None).
#[test]
fn test_confirmed_entry_has_no_ttl() {
    let now = 1_000_000_i64;
    let ttl = KnowledgeStore::compute_ttl(true, None, now);
    assert_eq!(ttl, None);
}

/// Caller-supplied ttl_days overrides the default.
#[test]
fn test_caller_supplied_ttl() {
    let now = 1_000_000_i64;
    let ttl = KnowledgeStore::compute_ttl(false, Some(7), now);
    assert_eq!(ttl, Some(now + 7 * 86_400));
}

/// Human-confirmed flag overrides caller-supplied ttl_days.
#[test]
fn test_confirmed_overrides_ttl_days() {
    let now = 1_000_000_i64;
    // Even if ttl_days is supplied, human_confirmed wins → None
    let ttl = KnowledgeStore::compute_ttl(true, Some(30), now);
    assert_eq!(ttl, None);
}

/// Full round-trip: add → evict with ttl=0 → confirm → evict again.
#[tokio::test]
async fn test_confirmed_entry_survives_eviction() {
    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .connect("sqlite::memory:")
        .await
        .unwrap();

    // Inline DDL mirrors the real migrations (including the text column).
    sqlx::query(
        r#"CREATE TABLE IF NOT EXISTS knowledge_store_meta (
            id TEXT PRIMARY KEY,
            text TEXT NOT NULL DEFAULT '',
            content_hash TEXT NOT NULL UNIQUE,
            topic TEXT NOT NULL DEFAULT 'general',
            cluster_id TEXT,
            source_node_id TEXT,
            source_agent_id TEXT,
            confidence REAL NOT NULL DEFAULT 1.0,
            access_count INTEGER NOT NULL DEFAULT 0,
            last_accessed_at INTEGER,
            ttl INTEGER,
            created_at INTEGER NOT NULL DEFAULT (unixepoch()),
            updated_at INTEGER NOT NULL DEFAULT (unixepoch()),
            human_confirmed INTEGER NOT NULL DEFAULT 0,
            concept_type TEXT NOT NULL DEFAULT 'general',
            title TEXT,
            description TEXT,
            resource_uri TEXT,
            tags TEXT
        )"#,
    )
    .execute(&pool)
    .await
    .unwrap();

    let store = KnowledgeStore::new(pool.clone());

    // Insert a row with ttl already expired (past)
    let id = "test-entry-1".to_string();
    let past_ttl = chrono::Utc::now().timestamp() - 3600; // 1 hour ago
    sqlx::query(
        r#"INSERT INTO knowledge_store_meta (id, text, content_hash, topic, ttl, human_confirmed, created_at, updated_at, concept_type)
           VALUES (?, 'hello', ?, 'general', ?, 0, unixepoch(), unixepoch(), 'general')"#)
    .bind(&id)
    .bind("abc123hash")
    .bind(past_ttl)
    .execute(&pool)
    .await
    .unwrap();

    // Before confirm: eviction should delete it
    let evicted = store.evict_expired().await.unwrap();
    assert_eq!(evicted, 1, "Expired unconfirmed entry should be evicted");

    // Insert again and confirm it
    sqlx::query(
        r#"INSERT INTO knowledge_store_meta (id, text, content_hash, topic, ttl, human_confirmed, created_at, updated_at, concept_type)
           VALUES (?, 'world', ?, 'general', ?, 0, unixepoch(), unixepoch(), 'general')"#)
    .bind("test-entry-2")
    .bind("def456hash")
    .bind(past_ttl)
    .execute(&pool)
    .await
    .unwrap();

    // Confirm it (sets human_confirmed=1, ttl=NULL)
    store.confirm("test-entry-2").await.unwrap();

    // Eviction should NOT delete confirmed entry
    let evicted_after_confirm = store.evict_expired().await.unwrap();
    assert_eq!(
        evicted_after_confirm, 0,
        "Human-confirmed entry must survive eviction"
    );
}

/// get_by_id must return the stored text (not an empty string).
#[tokio::test]
async fn test_get_by_id_returns_text() {
    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .connect("sqlite::memory:")
        .await
        .unwrap();

    sqlx::query(
        r#"CREATE TABLE IF NOT EXISTS knowledge_store_meta (
            id TEXT PRIMARY KEY,
            text TEXT NOT NULL DEFAULT '',
            content_hash TEXT NOT NULL UNIQUE,
            topic TEXT NOT NULL DEFAULT 'general',
            cluster_id TEXT,
            source_node_id TEXT,
            source_agent_id TEXT,
            confidence REAL NOT NULL DEFAULT 1.0,
            access_count INTEGER NOT NULL DEFAULT 0,
            last_accessed_at INTEGER,
            ttl INTEGER,
            created_at INTEGER NOT NULL DEFAULT (unixepoch()),
            updated_at INTEGER NOT NULL DEFAULT (unixepoch()),
            human_confirmed INTEGER NOT NULL DEFAULT 0,
            concept_type TEXT NOT NULL DEFAULT 'general',
            title TEXT,
            description TEXT,
            resource_uri TEXT,
            tags TEXT
        )"#,
    )
    .execute(&pool)
    .await
    .unwrap();

    sqlx::query(
        "INSERT INTO knowledge_store_meta (id, text, content_hash, topic, created_at, updated_at, concept_type) \
         VALUES ('id-1', 'The quick brown fox', 'hashxyz', 'general', unixepoch(), unixepoch(), 'general')",
    )
    .execute(&pool)
    .await
    .unwrap();

    let store = KnowledgeStore::new(pool);
    let entry = store.get_by_id("id-1").await.unwrap().unwrap();
    assert_eq!(entry.text, "The quick brown fox");
    assert_eq!(entry.access_count, 1); // incremented by get_by_id
}

/// decay_confidence must be time-aware: decay = 0.01 * days_since_update.
#[tokio::test]
async fn test_decay_is_time_aware() {
    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .connect("sqlite::memory:")
        .await
        .unwrap();

    sqlx::query(
        r#"CREATE TABLE IF NOT EXISTS knowledge_store_meta (
            id TEXT PRIMARY KEY,
            text TEXT NOT NULL DEFAULT '',
            content_hash TEXT NOT NULL UNIQUE,
            topic TEXT NOT NULL DEFAULT 'general',
            cluster_id TEXT,
            source_node_id TEXT,
            source_agent_id TEXT,
            confidence REAL NOT NULL DEFAULT 1.0,
            access_count INTEGER NOT NULL DEFAULT 0,
            last_accessed_at INTEGER,
            ttl INTEGER,
            created_at INTEGER NOT NULL DEFAULT (unixepoch()),
            updated_at INTEGER NOT NULL DEFAULT (unixepoch()),
            human_confirmed INTEGER NOT NULL DEFAULT 0,
            concept_type TEXT NOT NULL DEFAULT 'general',
            title TEXT,
            description TEXT,
            resource_uri TEXT,
            tags TEXT
        )"#,
    )
    .execute(&pool)
    .await
    .unwrap();

    // Insert an entry whose `updated_at` is 10 days ago.
    let ten_days_ago = chrono::Utc::now().timestamp() - 10 * 86_400;
    sqlx::query(
        "INSERT INTO knowledge_store_meta (id, text, content_hash, topic, confidence, updated_at, created_at, concept_type) \
         VALUES ('id-decay', 'fact', 'decayhash', 'general', 1.0, ?, unixepoch(), 'general')",
    )
    .bind(ten_days_ago)
    .execute(&pool)
    .await
    .unwrap();

    let store = KnowledgeStore::new(pool.clone());
    store.decay_confidence().await.unwrap();

    let row: (f64,) =
        sqlx::query_as("SELECT confidence FROM knowledge_store_meta WHERE id = 'id-decay'")
            .fetch_one(&pool)
            .await
            .unwrap();

    // Expected: 1.0 - (0.01 * 10) = 0.90, allow ±0.01 for integer truncation in SQLite.
    assert!(
        (row.0 - 0.90).abs() < 0.02,
        "Expected confidence ~0.90 after 10-day decay, got {}",
        row.0
    );
}

// Metadata: [tests]

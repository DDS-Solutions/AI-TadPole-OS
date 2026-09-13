//! @docs ARCHITECTURE:IKS
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Agent Runner / tests
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Type-safe state handling and bounded execution without unhandled panics.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: none declared
//! - **Witness Tests**: none declared

use super::store::KnowledgeStore;
use super::types::{KnowledgeSearchRequest, DEFAULT_TTL_DAYS};
use crate::error::AppError;

pub const IKS_TEST_SCHEMA: &str = r#"
CREATE TABLE IF NOT EXISTS knowledge_store_meta (
    id TEXT PRIMARY KEY,
    text TEXT NOT NULL DEFAULT '',
    content_hash TEXT NOT NULL UNIQUE,
    topic TEXT NOT NULL DEFAULT 'general',
    cluster_id TEXT,
    source_node_id TEXT,
    source_agent_id TEXT,
    confidence REAL NOT NULL DEFAULT 0.70,
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
    tags TEXT,
    security_tier TEXT NOT NULL DEFAULT 'BRONZE_ADHOC',
    parent_id TEXT
);
"#;

async fn init_test_db() -> sqlx::SqlitePool {
    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .connect("sqlite::memory:")
        .await
        .unwrap();

    sqlx::query(IKS_TEST_SCHEMA).execute(&pool).await.unwrap();

    pool
}

/// SHA-256 hash must be stable across identical scoped inputs.
#[test]
fn test_sha256_hash_stable() {
    let h1 = KnowledgeStore::sha256_hash("devops", Some("cluster-a"), "hello world");
    let h2 = KnowledgeStore::sha256_hash("devops", Some("cluster-a"), "hello world");
    assert_eq!(h1, h2);
    assert!(!h1.is_empty());
}

/// Scoped content hashing: different topics or clusters must produce distinct hashes for identical text.
#[test]
fn test_sha256_hash_scoped_distinct() {
    let h_global = KnowledgeStore::sha256_hash("general", None, "standard operating procedure");
    let h_cluster_a =
        KnowledgeStore::sha256_hash("general", Some("cluster-a"), "standard operating procedure");
    let h_cluster_b =
        KnowledgeStore::sha256_hash("general", Some("cluster-b"), "standard operating procedure");
    let h_topic = KnowledgeStore::sha256_hash("security", None, "standard operating procedure");

    assert_ne!(h_global, h_cluster_a);
    assert_ne!(h_cluster_a, h_cluster_b);
    assert_ne!(h_global, h_topic);
}

/// SHA-256 output must be a valid 64-char hex string.
#[test]
fn test_sha256_hash_format() {
    let h = KnowledgeStore::sha256_hash("topic", None, "test");
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
    let ttl = KnowledgeStore::compute_ttl(true, Some(30), now);
    assert_eq!(ttl, None);
}

/// Full round-trip: add → evict with expired TTL → confirm → evict again.
#[tokio::test]
async fn test_confirmed_entry_survives_eviction() {
    let pool = init_test_db().await;
    let store = KnowledgeStore::new(pool.clone());

    let past_ttl = chrono::Utc::now().timestamp() - 3600; // 1 hour ago
    sqlx::query(
        r#"INSERT INTO knowledge_store_meta (id, text, content_hash, topic, ttl, human_confirmed, created_at, updated_at, concept_type)
           VALUES ('test-entry-1', 'hello', 'hash1', 'general', ?, 0, unixepoch(), unixepoch(), 'general')"#)
    .bind(past_ttl)
    .execute(&pool)
    .await
    .unwrap();

    let evicted = store.evict_expired().await.unwrap();
    assert_eq!(evicted, 1, "Expired unconfirmed entry should be evicted");

    // Insert another expired entry and confirm it
    sqlx::query(
        r#"INSERT INTO knowledge_store_meta (id, text, content_hash, topic, ttl, human_confirmed, created_at, updated_at, concept_type)
           VALUES ('test-entry-2', 'world', 'hash2', 'general', ?, 0, unixepoch(), unixepoch(), 'general')"#)
    .bind(past_ttl)
    .execute(&pool)
    .await
    .unwrap();

    store.confirm("test-entry-2").await.unwrap();

    let evicted_after_confirm = store.evict_expired().await.unwrap();
    assert_eq!(
        evicted_after_confirm, 0,
        "Human-confirmed entry must survive eviction"
    );
}

/// Zero-confidence unconfirmed entries should be evicted.
#[tokio::test]
async fn test_zero_confidence_unconfirmed_is_evicted() {
    let pool = init_test_db().await;
    let store = KnowledgeStore::new(pool.clone());

    sqlx::query(
        r#"INSERT INTO knowledge_store_meta (id, text, content_hash, topic, confidence, ttl, human_confirmed, created_at, updated_at, concept_type)
           VALUES ('test-zero-conf', 'low quality fact', 'hash-zero', 'general', 0.0, NULL, 0, unixepoch(), unixepoch(), 'general')"#)
    .execute(&pool)
    .await
    .unwrap();

    let evicted = store.evict_expired().await.unwrap();
    assert_eq!(
        evicted, 1,
        "Zero-confidence unconfirmed entry should be evicted"
    );
}

/// get_by_id must return the stored text and track access.
#[tokio::test]
async fn test_get_by_id_returns_text_and_tracks_access() {
    let pool = init_test_db().await;

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
    assert_eq!(entry.access_count, 1);

    // Non-tracking lookup does not increment access_count
    let entry_no_track = store
        .get_by_id_internal("id-1", false)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(entry_no_track.access_count, 1);
}

/// decay_confidence must be time-aware: decay = 0.01 * days_since_update.
#[tokio::test]
async fn test_decay_is_time_aware() {
    let pool = init_test_db().await;

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

    assert!(
        (row.0 - 0.90).abs() < 0.02,
        "Expected confidence ~0.90 after 10-day decay, got {}",
        row.0
    );
}

/// remove() refuses to delete human-confirmed entries without force=true.
#[tokio::test]
async fn test_remove_refuses_human_confirmed_without_force() {
    let pool = init_test_db().await;
    let store = KnowledgeStore::new(pool.clone());

    sqlx::query(
        "INSERT INTO knowledge_store_meta (id, text, content_hash, topic, human_confirmed, created_at, updated_at, concept_type) \
         VALUES ('id-confirmed', 'Sovereign Law', 'hash-law', 'governance', 1, unixepoch(), unixepoch(), 'policy')",
    )
    .execute(&pool)
    .await
    .unwrap();

    // Attempting delete without force returns 409 Conflict
    let res = store.remove("id-confirmed", false).await;
    assert!(res.is_err());
    assert!(matches!(res.unwrap_err(), AppError::Conflict(_)));

    // Deleting with force succeeds
    let force_res = store.remove("id-confirmed", true).await;
    assert!(force_res.is_ok());

    let lookup = store.get_by_id("id-confirmed").await.unwrap();
    assert!(lookup.is_none());
}

/// confirm() is idempotent.
#[tokio::test]
async fn test_confirm_is_idempotent() {
    let pool = init_test_db().await;
    let store = KnowledgeStore::new(pool.clone());

    sqlx::query(
        "INSERT INTO knowledge_store_meta (id, text, content_hash, topic, human_confirmed, confidence, ttl, created_at, updated_at, concept_type) \
         VALUES ('id-confirm-idem', 'Fact', 'hash-idem', 'general', 0, 0.70, unixepoch() + 1000, unixepoch(), unixepoch(), 'fact')",
    )
    .execute(&pool)
    .await
    .unwrap();

    let e1 = store.confirm("id-confirm-idem").await.unwrap();
    assert!(e1.human_confirmed);
    assert_eq!(e1.confidence, 1.0);
    assert_eq!(e1.ttl, None);

    let e2 = store.confirm("id-confirm-idem").await.unwrap();
    assert!(e2.human_confirmed);
    assert_eq!(e2.confidence, 1.0);
    assert_eq!(e2.ttl, None);
}

/// "global" cluster sentinel in search only returns global (NULL cluster) entries.
#[tokio::test]
async fn test_global_cluster_sentinel_filtering() {
    let pool = init_test_db().await;
    let store = KnowledgeStore::new(pool);

    sqlx::query(
        "INSERT INTO knowledge_store_meta (id, text, content_hash, topic, cluster_id, created_at, updated_at, concept_type) \
         VALUES ('id-global', 'Global Rule', 'hash-g', 'ops', NULL, unixepoch(), unixepoch(), 'playbook'), \
                ('id-tenant', 'Tenant Specific Rule', 'hash-t', 'ops', 'cluster-123', unixepoch(), unixepoch(), 'playbook')",
    )
    .execute(&store.pool)
    .await
    .unwrap();

    let search_req = KnowledgeSearchRequest {
        query: "Rule".to_string(),
        topic: None,
        cluster_id: Some("global".to_string()),
        concept_type: None,
        security_tier: None,
        limit: Some(10),
        min_confidence: None,
    };

    let hits = store
        .search_sqlite_fallback(&search_req, 10, 0.3)
        .await
        .unwrap();

    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0].id, "id-global");
}

/// Fallback SQLite search must match query strings across text and title.
#[tokio::test]
async fn test_sqlite_fallback_search_filters_by_query() {
    let pool = init_test_db().await;
    let store = KnowledgeStore::new(pool);

    sqlx::query(
        "INSERT INTO knowledge_store_meta (id, text, title, content_hash, topic, created_at, updated_at, concept_type) \
         VALUES ('id-docker', 'Docker container guidelines', 'Docker Setup', 'hash1', 'devops', unixepoch(), unixepoch(), 'playbook'), \
                ('id-k8s', 'Kubernetes cluster deployment', 'K8s Cluster', 'hash2', 'devops', unixepoch(), unixepoch(), 'playbook')",
    )
    .execute(&store.pool)
    .await
    .unwrap();

    let search_req = KnowledgeSearchRequest {
        query: "Docker".to_string(),
        topic: None,
        cluster_id: None,
        concept_type: None,
        security_tier: None,
        limit: Some(10),
        min_confidence: None,
    };

    let hits = store
        .search_sqlite_fallback(&search_req, 10, 0.3)
        .await
        .unwrap();

    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0].id, "id-docker");
}

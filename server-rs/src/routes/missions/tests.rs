//! @docs ARCHITECTURE:Networking
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / HTTP Routes / missions / tests
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - [Structural] Deterministic internal state integrity and strict interface contract compliance.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: none declared

use super::catalog::escape_like_pattern;
use super::dossier::compute_line_diff_summary;
use super::*;

#[test]
fn test_compute_line_diff_summary() {
    let pre = "line 1\nline 2\nline 3";
    let cur = "line 1\nline 2 modified\nline 3\nline 4";
    let summary = compute_line_diff_summary(pre, cur);

    assert_eq!(summary.additions, 2);
    assert_eq!(summary.deletions, 1);
    assert_eq!(summary.changes, 1);
}

#[test]
fn test_compute_line_diff_summary_duplicate_lines() {
    let pre = "line\nline";
    let cur = "line\nline\nline\nline";
    let summary = compute_line_diff_summary(pre, cur);

    assert_eq!(summary.additions, 2);
    assert_eq!(summary.deletions, 0);
    assert_eq!(summary.changes, 0);
}

#[test]
fn test_compute_line_diff_summary_identical() {
    let pre = "identical content\nsecond line";
    let cur = "identical content\nsecond line";
    let summary = compute_line_diff_summary(pre, cur);

    assert_eq!(summary.additions, 0);
    assert_eq!(summary.deletions, 0);
    assert_eq!(summary.changes, 0);
}

#[test]
fn test_merkle_log_hash_verification() {
    let mission_id = "test-mission-123";
    let agent_id = "agent-456";
    let source = "System";
    let text = "Mission initialized";
    let timestamp = "2026-10-26T15:00:00Z";

    let hash1 = crate::agent::mission::compute_log_hash(
        None, mission_id, agent_id, source, text, timestamp,
    );
    assert!(!hash1.is_empty());

    let hash2 = crate::agent::mission::compute_log_hash(
        Some(&hash1),
        mission_id,
        agent_id,
        "Agent",
        "Next step executed",
        "2026-10-26T15:00:01Z",
    );
    assert_ne!(hash1, hash2);

    // Verification matches
    let verify_hash = crate::agent::mission::compute_log_hash(
        Some(&hash1),
        mission_id,
        agent_id,
        "Agent",
        "Next step executed",
        "2026-10-26T15:00:01Z",
    );
    assert_eq!(hash2, verify_hash);
}

#[test]
fn test_escape_like_pattern() {
    assert_eq!(escape_like_pattern("test%search"), "%test\\%search%");
    assert_eq!(escape_like_pattern("test_search"), "%test\\_search%");
    assert_eq!(escape_like_pattern("test\\search"), "%test\\\\search%");
    assert_eq!(
        escape_like_pattern("100%_done\\here"),
        "%100\\%\\_done\\\\here%"
    );
    assert_eq!(escape_like_pattern("normal"), "%normal%");
}

#[tokio::test]
async fn test_catalog_sorts_by_updated_at_desc() {
    use crate::state::AppState;
    use std::sync::Arc;

    let state = Arc::new(AppState::new_minimal_mock().await);
    let pool = &state.resources.pool;

    sqlx::query(
        "INSERT INTO agents (id, name, role, department, description, status, metadata) \
         VALUES ('agent-1', 'Agent One', 'assistant', 'engineering', 'desc', 'active', '{}')",
    )
    .execute(pool)
    .await
    .unwrap();

    // Seed Mission A: Created earlier (10 days ago), updated recently (1 min ago)
    sqlx::query(
        "INSERT INTO mission_history (id, agent_id, title, status, created_at, updated_at) \
         VALUES ('mission-rerun', 'agent-1', 'Rerun Mission', 'active', datetime('now', '-10 days'), datetime('now', '-1 minute'))"
    )
    .execute(pool)
    .await
    .unwrap();

    // Seed Mission B: Created 2 days ago, updated 2 days ago
    sqlx::query(
        "INSERT INTO mission_history (id, agent_id, title, status, created_at, updated_at) \
         VALUES ('mission-older', 'agent-1', 'Older Mission', 'completed', datetime('now', '-2 days'), datetime('now', '-2 days'))"
    )
    .execute(pool)
    .await
    .unwrap();

    let query = CatalogQuery {
        page: Some(1),
        per_page: Some(10),
        status: None,
        search: None,
        is_pinned: None,
    };

    use axum::response::IntoResponse;

    let response = get_missions_catalog(axum::extract::State(state), axum::extract::Query(query))
        .await
        .unwrap()
        .into_response();
    let body_bytes = axum::body::to_bytes(response.into_body(), 100_000)
        .await
        .unwrap();
    let res: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
    let data = res["data"].as_array().expect("data array");

    assert_eq!(data.len(), 2);
    assert_eq!(
        data[0]["id"].as_str().unwrap(),
        "mission-rerun",
        "Catalog must sort by updated_at DESC so rerun missions appear first"
    );
}

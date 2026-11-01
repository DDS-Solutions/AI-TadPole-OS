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

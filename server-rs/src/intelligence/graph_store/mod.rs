//! @docs ARCHITECTURE:Core:Intelligence
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / graph_store
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Type-safe state handling and bounded execution without unhandled panics.
//!
//! ### Subsystem Modules
//! - **`model`**: Entity rows, strongly-typed `EdgeKind`, and graph snapshots.
//! - **`heuristics`**: Pure classification rules, path matching, and tokenized security detection.
//! - **`extract`**: AST and regex language processors.
//! - **`metrics`**: Community clustering, risk scoring, and priority-ranked flow tracing.
//! - **`pipeline`**: Workspace scanning, caching, boundary enforcement, and background synthesis.
//! - **`db`**: SQLite persistence, WAL mode connection management, and full-text search.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: `crate::error::AppError`
//! - **Telemetry Targets**: none declared
//! - **Witness Tests**: `intelligence::graph_store::tests`

pub mod db;
pub mod extract;
pub mod heuristics;
pub mod metrics;
pub mod model;
pub mod pipeline;

#[allow(unused_imports)]
pub use model::{CommunityRule, EdgeKind, GraphConfig, GraphDbRefreshSummary};
pub use pipeline::refresh_code_review_graph_db;

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::Row;
    use tempfile::tempdir;

    #[tokio::test]
    async fn refresh_creates_idempotent_graph_db() {
        let dir = tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join("src")).unwrap();
        std::fs::write(
            dir.path().join("src/lib.rs"),
            "fn helper() {}\nfn main() { helper(); }\n",
        )
        .unwrap();
        let db = dir.path().join(".code-review-graph/graph.db");
        let test_salt = uuid::Uuid::new_v4().to_string();
        let first =
            refresh_code_review_graph_db(dir.path().to_path_buf(), db.clone(), test_salt.clone())
                .await
                .unwrap();
        let second = refresh_code_review_graph_db(dir.path().to_path_buf(), db.clone(), test_salt)
            .await
            .unwrap();
        assert_eq!(first.node_count, second.node_count);
        assert_eq!(first.edge_count, second.edge_count);
    }

    #[tokio::test]
    async fn refresh_removes_deleted_symbols() {
        let dir = tempdir().unwrap();
        std::fs::write(dir.path().join("a.rs"), "fn alpha() {}\n").unwrap();
        std::fs::write(dir.path().join("b.rs"), "fn beta() { alpha(); }\n").unwrap();
        let db = dir.path().join(".code-review-graph/graph.db");
        let test_salt = uuid::Uuid::new_v4().to_string();
        refresh_code_review_graph_db(dir.path().to_path_buf(), db.clone(), test_salt.clone())
            .await
            .unwrap();
        std::fs::remove_file(dir.path().join("b.rs")).unwrap();
        refresh_code_review_graph_db(dir.path().to_path_buf(), db.clone(), test_salt)
            .await
            .unwrap();
        let pool = db::open_graph_pool(&db).await.unwrap();
        let count: i64 = sqlx::query("SELECT COUNT(*) FROM nodes WHERE name = 'beta'")
            .fetch_one(&pool)
            .await
            .unwrap()
            .get(0);
        assert_eq!(count, 0);
    }

    #[tokio::test]
    async fn refresh_handles_duplicate_symbol_names_in_same_file() {
        let dir = tempdir().unwrap();
        std::fs::write(
            dir.path().join("lib.rs"),
            "struct AppError;\nimpl AppError { fn one() {} }\nimpl AppError { fn two() {} }\n",
        )
        .unwrap();
        let db = dir.path().join(".code-review-graph/graph.db");
        let test_salt = uuid::Uuid::new_v4().to_string();
        refresh_code_review_graph_db(dir.path().to_path_buf(), db.clone(), test_salt)
            .await
            .unwrap();

        let pool = db::open_graph_pool(&db).await.unwrap();
        let count: i64 =
            sqlx::query("SELECT COUNT(DISTINCT qualified_name) FROM nodes WHERE name = 'AppError'")
                .fetch_one(&pool)
                .await
                .unwrap()
                .get(0);
        assert_eq!(count, 3);
    }

    #[tokio::test]
    async fn risk_index_marks_security_and_fts_rebuilds() {
        let dir = tempdir().unwrap();
        std::fs::write(
            dir.path().join("auth.rs"),
            "fn validate_token() {}\nfn caller() { validate_token(); }\n",
        )
        .unwrap();
        let db = dir.path().join(".code-review-graph/graph.db");
        let test_salt = uuid::Uuid::new_v4().to_string();
        refresh_code_review_graph_db(dir.path().to_path_buf(), db.clone(), test_salt)
            .await
            .unwrap();
        let pool = db::open_graph_pool(&db).await.unwrap();
        let security: i64 = sqlx::query(
            "SELECT security_relevant FROM risk_index WHERE qualified_name LIKE '%validate_token%' LIMIT 1",
        )
        .fetch_one(&pool)
        .await
        .unwrap()
        .get(0);
        let fts_count: i64 =
            sqlx::query("SELECT COUNT(*) FROM nodes_fts WHERE nodes_fts MATCH 'validate_token'")
                .fetch_one(&pool)
                .await
                .unwrap()
                .get(0);
        assert_eq!(security, 1);
        assert!(fts_count > 0);
    }
}

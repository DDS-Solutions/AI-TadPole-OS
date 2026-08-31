//! @docs ARCHITECTURE:Infrastructure
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / okf_gate
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Type-safe state handling and bounded execution without unhandled panics.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: none declared
//! - **Witness Tests**: none declared

use crate::error::AppError;
use dashmap::DashMap;
use once_cell::sync::Lazy;
use serde::Serialize;
use sqlx::{FromRow, SqlitePool};
use std::time::{Duration, Instant};

static PLAYBOOK_REQS_CACHE: Lazy<DashMap<String, (Instant, (Vec<String>, bool))>> =
    Lazy::new(DashMap::new);
const CACHE_TTL: Duration = Duration::from_secs(5);
const MAX_CACHE_ENTRIES: usize = 256;

/// Prunes expired cache entries if total capacity exceeds maximum thresholds.
fn prune_cache_if_needed() {
    if PLAYBOOK_REQS_CACHE.len() > MAX_CACHE_ENTRIES {
        PLAYBOOK_REQS_CACHE.retain(|_, (timestamp, _)| timestamp.elapsed() < CACHE_TTL);
    }
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct OkfNodeInfo {
    pub id: String,
    pub title: String,
    pub concept_type: String,
    pub security_tier: Option<String>,
    pub parent_id: Option<String>,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct OkfValidationResult {
    pub status: String, // "nominal" | "warning" | "critical"
    pub message: Option<String>,
}

#[derive(FromRow)]
struct PlaybookDbRow {
    id: String,
    title: Option<String>,
    concept_type: String,
    security_tier: Option<String>,
    parent_id: Option<String>,
}

/// Query active OKF playbooks associated with a cluster.
pub async fn get_mounted_playbooks(
    pool: &SqlitePool,
    cluster_id: &str,
) -> Result<Vec<OkfNodeInfo>, AppError> {
    let rows = sqlx::query_as::<_, PlaybookDbRow>(
        "SELECT id, title, concept_type, security_tier, parent_id FROM knowledge_store_meta WHERE cluster_id = ? AND concept_type = 'playbook'"
    )
    .bind(cluster_id)
    .fetch_all(pool)
    .await
    .map_err(|e| AppError::InternalServerError(format!("[OKF Gate] Failed to query playbooks: {}", e)))?;

    let playbooks = rows
        .into_iter()
        .map(|row| {
            let title = row.title.unwrap_or_else(|| row.id.clone());
            OkfNodeInfo {
                id: row.id,
                title,
                concept_type: row.concept_type,
                security_tier: row.security_tier,
                parent_id: row.parent_id,
            }
        })
        .collect();

    Ok(playbooks)
}

struct ReqDef {
    key: &'static str,
    status: &'static str,
    message: &'static str,
    pattern_colon: &'static str,
    pattern_space: &'static str,
}

const REQUIREMENTS: &[ReqDef] = &[
    ReqDef {
        key: "docker",
        status: "warning",
        message: "Active OKF playbooks require a Docker container sandbox, but no Docker daemon was detected on the host.",
        pattern_colon: "requires: docker",
        pattern_space: "requires docker",
    },
    ReqDef {
        key: "k8s_node",
        status: "critical",
        message: "Critical: OKF playbooks require a Kubernetes cluster environment, but the application is running outside Kubernetes.",
        pattern_colon: "requires: k8s_node",
        pattern_space: "requires k8s_node",
    },
    ReqDef {
        key: "jupyter_lab",
        status: "warning",
        message: "Active OKF playbooks require Jupyter Lab environment for data modeling, but jupyter was not found in PATH.",
        pattern_colon: "requires: jupyter_lab",
        pattern_space: "requires jupyter_lab",
    },
    ReqDef {
        key: "wasm_sandbox",
        status: "warning",
        message: "Active OKF playbooks require WebAssembly sandbox isolation, but wasm-codec was not detected.",
        pattern_colon: "requires: wasm_sandbox",
        pattern_space: "requires wasm_sandbox",
    },
];

/// Helper to strictly extract required keys from playbook text, handling frontmatter and negation checks.
fn extract_playbook_reqs(text: &str) -> std::collections::HashSet<String> {
    let mut required_keys = std::collections::HashSet::new();

    // 1. Try strict YAML frontmatter parsing first
    if let Some(stripped) = text.strip_prefix("---") {
        if let Some(end_idx) = stripped.find("---") {
            let yaml_part = &stripped[..end_idx];
            #[derive(serde::Deserialize)]
            struct PlaybookHeader {
                requires: Option<Vec<String>>,
            }
            if let Ok(header) = serde_yaml::from_str::<PlaybookHeader>(yaml_part) {
                if let Some(reqs) = header.requires {
                    for r in reqs {
                        required_keys.insert(r.to_lowercase().trim().to_string());
                    }
                }
            }
        }
    }

    // 2. Safe Line-by-Line requirement extraction fallback (window-scoped negation search)
    if required_keys.is_empty() {
        for line in text.lines() {
            let line_lower = line.to_lowercase();
            for req in REQUIREMENTS {
                let mut matched_pos = None;
                if let Some(pos) = line_lower.find(req.pattern_colon) {
                    matched_pos = Some(pos);
                } else if let Some(pos) = line_lower.find(req.pattern_space) {
                    matched_pos = Some(pos);
                }

                if let Some(pos) = matched_pos {
                    // Check for negations immediately preceding the requirement pattern on the same line
                    let prefix = &line_lower[..pos];
                    let window_start = prefix.len().saturating_sub(25);
                    let window = &prefix[window_start..];
                    let has_negation =
                        ["no ", "not ", "don't ", "does not ", "without ", "free of "]
                            .iter()
                            .any(|neg| window.contains(neg));
                    if !has_negation {
                        required_keys.insert(req.key.to_string());
                    }
                }
            }
        }
    }

    required_keys
}

#[derive(FromRow)]
struct PlaybookTextDbRow {
    text: String,
    security_tier: Option<String>,
}

/// Validate the host environments against the requirements of the mounted playbooks.
pub async fn validate_environments(
    pool: &SqlitePool,
    cluster_id: &str,
    detected_envs: &[String],
) -> Result<OkfValidationResult, AppError> {
    prune_cache_if_needed();

    // 1. Check in-memory cache first to avoid database query hotspots
    let cached_entry = if let Some(entry) = PLAYBOOK_REQS_CACHE.get(cluster_id) {
        let (timestamp, data) = entry.value();
        if timestamp.elapsed() < CACHE_TTL {
            Some(data.clone())
        } else {
            // Remove expired cache entry directly
            drop(entry);
            PLAYBOOK_REQS_CACHE.remove(cluster_id);
            None
        }
    } else {
        None
    };

    let (required_keys, has_gold_critical) = match cached_entry {
        Some((reqs, is_gold)) => (reqs, is_gold),
        None => {
            // Cache miss: Query SQLite database for playbook texts and security tiers using sqlx FromRow
            let rows = sqlx::query_as::<_, PlaybookTextDbRow>(
                "SELECT text, security_tier FROM knowledge_store_meta WHERE cluster_id = ? AND concept_type = 'playbook'",
            )
            .bind(cluster_id)
            .fetch_all(pool)
            .await
            .map_err(|e| {
                AppError::InternalServerError(format!("[OKF Gate] Failed to query playbook text: {}", e))
            })?;

            let mut reqs = std::collections::HashSet::new();
            let mut is_gold = false;
            for row in rows {
                if let Some(ref t) = row.security_tier {
                    if t.to_uppercase() == "GOLD_CRITICAL" {
                        is_gold = true;
                    }
                }
                let playbook_reqs = extract_playbook_reqs(&row.text);
                reqs.extend(playbook_reqs);
            }

            let reqs_vec: Vec<String> = reqs.into_iter().collect();
            // Cache the requirements and gold status together for this cluster
            PLAYBOOK_REQS_CACHE.insert(
                cluster_id.to_string(),
                (Instant::now(), (reqs_vec.clone(), is_gold)),
            );
            (reqs_vec, is_gold)
        }
    };

    // 2. Validate environment requirements
    if required_keys.is_empty() {
        return Ok(OkfValidationResult {
            status: "nominal".to_string(),
            message: None,
        });
    }

    for req in REQUIREMENTS {
        if required_keys.contains(&req.key.to_string())
            && !detected_envs.contains(&req.key.to_string())
        {
            let status = if has_gold_critical {
                "critical".to_string()
            } else {
                req.status.to_string()
            };
            return Ok(OkfValidationResult {
                status,
                message: Some(req.message.to_string()),
            });
        }
    }

    Ok(OkfValidationResult {
        status: "nominal".to_string(),
        message: None,
    })
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PlaybookCachedDetail {
    pub id: String,
    pub title: String,
    pub concept_type: String,
    pub text: String,
}

#[derive(FromRow)]
struct PlaybookDetailDbRow {
    id: String,
    title: Option<String>,
    concept_type: String,
    text: String,
}

pub async fn mount_playbooks_to_workspace(
    pool: &SqlitePool,
    cluster_id: &str,
    workspace_dir: &std::path::Path,
) -> Result<(), AppError> {
    // Invalidate requirements cache for this cluster since playbooks are being updated
    PLAYBOOK_REQS_CACHE.remove(cluster_id);

    let rows = sqlx::query_as::<_, PlaybookDetailDbRow>(
        "SELECT id, title, concept_type, text FROM knowledge_store_meta WHERE cluster_id = ? AND concept_type = 'playbook'"
    )
    .bind(cluster_id)
    .fetch_all(pool)
    .await
    .map_err(|e| AppError::InternalServerError(format!("[OKF Gate] Failed to query playbooks for mounting: {}", e)))?;

    let playbooks = rows
        .into_iter()
        .map(|row| {
            let title = row.title.unwrap_or_else(|| row.id.clone());
            PlaybookCachedDetail {
                id: row.id,
                title,
                concept_type: row.concept_type,
                text: row.text,
            }
        })
        .collect::<Vec<_>>();

    if !workspace_dir.exists() {
        tokio::fs::create_dir_all(workspace_dir)
            .await
            .map_err(|e| {
                AppError::InternalServerError(format!(
                    "[OKF Gate] Failed to create workspace directory {:?}: {}",
                    workspace_dir, e
                ))
            })?;
    }

    let json_path = workspace_dir.join("playbooks.json");
    let content = serde_json::to_string_pretty(&playbooks).map_err(|e| {
        AppError::InternalServerError(format!("[OKF Gate] Failed to serialize playbooks: {}", e))
    })?;

    tokio::fs::write(&json_path, content).await.map_err(|e| {
        AppError::InternalServerError(format!("[OKF Gate] Failed to write playbooks.json: {}", e))
    })?;
    tracing::info!(
        "✅ [OKF Gate] Mounted {} playbooks to {}",
        playbooks.len(),
        json_path.display()
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::sqlite::SqlitePoolOptions;

    async fn setup_test_db() -> SqlitePool {
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .unwrap();

        sqlx::query(
            r#"CREATE TABLE knowledge_store_meta (
                id TEXT PRIMARY KEY,
                text TEXT NOT NULL DEFAULT '',
                concept_type TEXT NOT NULL DEFAULT 'general',
                cluster_id TEXT,
                title TEXT,
                description TEXT,
                resource_uri TEXT,
                tags TEXT,
                security_tier TEXT NOT NULL DEFAULT 'BRONZE_ADHOC',
                parent_id TEXT
            )"#,
        )
        .execute(&pool)
        .await
        .unwrap();

        pool
    }

    #[tokio::test]
    async fn test_okf_validation_nominal_empty() {
        let pool = setup_test_db().await;
        let res = validate_environments(&pool, "cl-empty", &[]).await.unwrap();
        assert_eq!(res.status, "nominal");
        assert!(res.message.is_none());
    }

    #[tokio::test]
    async fn test_okf_validation_missing_docker() {
        let pool = setup_test_db().await;

        sqlx::query(
            r#"INSERT INTO knowledge_store_meta (id, text, concept_type, cluster_id, title)
               VALUES ('pb-1', 'Workflow note: requires docker container isolation.', 'playbook', 'cl-docker', 'Docker Playbook')"#,
        )
        .execute(&pool)
        .await
        .unwrap();

        // 1. Missing docker -> warning
        let res = validate_environments(&pool, "cl-docker", &[])
            .await
            .unwrap();
        assert_eq!(res.status, "warning");
        assert!(res.message.unwrap().contains("Docker container sandbox"));

        // 2. Present docker -> nominal
        let res = validate_environments(&pool, "cl-docker", &["docker".to_string()])
            .await
            .unwrap();
        assert_eq!(res.status, "nominal");
        assert!(res.message.is_none());
    }

    #[tokio::test]
    async fn test_okf_validation_missing_k8s() {
        let pool = setup_test_db().await;

        sqlx::query(
            r#"INSERT INTO knowledge_store_meta (id, text, concept_type, cluster_id, title)
               VALUES ('pb-2', 'Production deployment note: requires: k8s_node for scaling.', 'playbook', 'cl-k8s', 'K8s Playbook')"#,
        )
        .execute(&pool)
        .await
        .unwrap();

        // 1. Missing k8s -> critical
        let res = validate_environments(&pool, "cl-k8s", &[]).await.unwrap();
        assert_eq!(res.status, "critical");
        assert!(res.message.unwrap().contains("Kubernetes cluster"));

        // 2. Present k8s -> nominal
        let res = validate_environments(&pool, "cl-k8s", &["k8s_node".to_string()])
            .await
            .unwrap();
        assert_eq!(res.status, "nominal");
        assert!(res.message.is_none());
    }

    #[tokio::test]
    async fn test_okf_validation_yaml_frontmatter() {
        let pool = setup_test_db().await;

        let playbook_yaml = r#"---
title: "Complex ML Playbook"
requires:
  - docker
  - jupyter_lab
---
# Main Procedure
Step 1: Ingest dataset.
"#;

        sqlx::query(
            r#"INSERT INTO knowledge_store_meta (id, text, concept_type, cluster_id, title)
               VALUES ('pb-3', ?, 'playbook', 'cl-test', 'YAML Playbook')"#,
        )
        .bind(playbook_yaml)
        .execute(&pool)
        .await
        .unwrap();

        // 1. Missing both -> warns on docker first
        let res = validate_environments(&pool, "cl-test", &[]).await.unwrap();
        assert_eq!(res.status, "warning");
        assert!(res.message.unwrap().contains("Docker daemon"));

        // 2. Missing jupyter_lab
        let res = validate_environments(&pool, "cl-test", &["docker".to_string()])
            .await
            .unwrap();
        assert_eq!(res.status, "warning");
        assert!(res.message.unwrap().contains("Jupyter Lab"));

        // 3. Nominal (both present)
        let res = validate_environments(
            &pool,
            "cl-test",
            &["docker".to_string(), "jupyter_lab".to_string()],
        )
        .await
        .unwrap();
        assert_eq!(res.status, "nominal");
    }

    #[tokio::test]
    async fn test_okf_validation_negation_avoidance() {
        let pool = setup_test_db().await;

        sqlx::query(
            r#"INSERT INTO knowledge_store_meta (id, text, concept_type, cluster_id, title)
               VALUES ('pb-4', 'We do not require docker or k8s_node for this workflow.', 'playbook', 'cl-neg', 'Negation Playbook')"#,
        )
        .execute(&pool)
        .await
        .unwrap();

        // Should parse as nominal even with empty environments because the requirement is negated
        let res = validate_environments(&pool, "cl-neg", &[]).await.unwrap();
        assert_eq!(res.status, "nominal");
    }

    #[tokio::test]
    async fn test_okf_validation_gold_critical_tier() {
        let pool = setup_test_db().await;

        sqlx::query(
            r#"INSERT INTO knowledge_store_meta (id, text, concept_type, cluster_id, title, security_tier)
               VALUES ('pb-5', 'Security directive: requires docker sandbox.', 'playbook', 'cl-gold', 'Gold Directive', 'GOLD_CRITICAL')"#,
        )
        .execute(&pool)
        .await
        .unwrap();

        // Missing docker on a GOLD_CRITICAL playbook should elevate status from "warning" to "critical"
        let res = validate_environments(&pool, "cl-gold", &[]).await.unwrap();
        assert_eq!(res.status, "critical");

        // Second call (cache hit) must preserve "critical" status
        let res_cached = validate_environments(&pool, "cl-gold", &[]).await.unwrap();
        assert_eq!(
            res_cached.status, "critical",
            "Cached validation must retain gold-critical severity"
        );
    }
}

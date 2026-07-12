//! @docs ARCHITECTURE:Infrastructure
//!
//! ### AI Assist Note
//! **Core technical resource for the Tadpole OS Sovereign infrastructure.**
//! This module implements high-fidelity logic for the Sovereign Reality layer.
//!
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: Runtime logic error, state desynchronization, or resource exhaustion.
//! - **Telemetry Link**: Search `[okf_gate]` in tracing logs.

use crate::error::AppError;
use serde::Serialize;
use sqlx::SqlitePool;

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct OkfNodeInfo {
    pub id: String,
    pub title: String,
    pub concept_type: String,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct OkfValidationResult {
    pub status: String, // "nominal" | "warning" | "critical"
    pub message: Option<String>,
}

/// Query active OKF playbooks associated with a cluster.
pub async fn get_mounted_playbooks(
    pool: &SqlitePool,
    cluster_id: &str,
) -> Result<Vec<OkfNodeInfo>, AppError> {
    let rows = sqlx::query(
        "SELECT id, title, concept_type FROM knowledge_store_meta WHERE cluster_id = ? AND concept_type = 'playbook'"
    )
    .bind(cluster_id)
    .fetch_all(pool)
    .await
    .map_err(|e| AppError::InternalServerError(format!("[OKF Gate] Failed to query playbooks: {}", e)))?;

    let mut playbooks = Vec::new();
    for row in rows {
        use sqlx::Row;
        let id: String = row.try_get("id")?;
        let title_opt: Option<String> = row.try_get("title")?;
        let concept_type: String = row.try_get("concept_type")?;
        let title = title_opt.unwrap_or_else(|| id.clone());
        playbooks.push(OkfNodeInfo {
            id,
            title,
            concept_type,
        });
    }

    Ok(playbooks)
}

struct ReqDef {
    key: &'static str,
    status: &'static str,
    message: &'static str,
}

const REQUIREMENTS: &[ReqDef] = &[
    ReqDef {
        key: "docker",
        status: "warning",
        message: "Active OKF playbooks require a Docker container sandbox, but no Docker daemon was detected on the host.",
    },
    ReqDef {
        key: "k8s_node",
        status: "critical",
        message: "Critical: OKF playbooks require a Kubernetes cluster environment, but the application is running outside Kubernetes.",
    },
    ReqDef {
        key: "jupyter_lab",
        status: "warning",
        message: "Active OKF playbooks require Jupyter Lab environment for data modeling, but jupyter was not found in PATH.",
    },
    ReqDef {
        key: "wasm_sandbox",
        status: "warning",
        message: "Active OKF playbooks require WebAssembly sandbox isolation, but wasm-codec was not detected.",
    },
];

/// Validate the host environments against the requirements of the mounted playbooks.
pub async fn validate_environments(
    pool: &SqlitePool,
    cluster_id: &str,
    detected_envs: &[String],
) -> Result<OkfValidationResult, AppError> {
    // Single query to get all playbook texts
    let rows = sqlx::query(
        "SELECT text FROM knowledge_store_meta WHERE cluster_id = ? AND concept_type = 'playbook'",
    )
    .bind(cluster_id)
    .fetch_all(pool)
    .await
    .map_err(|e| {
        AppError::InternalServerError(format!("[OKF Gate] Failed to query playbook text: {}", e))
    })?;

    if rows.is_empty() {
        return Ok(OkfValidationResult {
            status: "nominal".to_string(),
            message: None,
        });
    }

    for row in rows {
        use sqlx::Row;
        let text: String = row.try_get("text")?;
        let mut required_keys = std::collections::HashSet::new();

        // --- EXTRACTION PHASE ---
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
                            required_keys.insert(r.to_lowercase());
                        }
                    }
                }
            }
        }

        // Fallback to substring matching if no YAML requirements were found
        if required_keys.is_empty() {
            let text_lower = text.to_lowercase();
            for req in REQUIREMENTS {
                if text_lower.contains(&format!("requires: {}", req.key))
                    || text_lower.contains(&format!("requires {}", req.key))
                {
                    required_keys.insert(req.key.to_string());
                }
            }
        }

        // --- VALIDATION PHASE ---
        for req in REQUIREMENTS {
            if required_keys.contains(req.key) && !detected_envs.contains(&req.key.to_string()) {
                return Ok(OkfValidationResult {
                    status: req.status.to_string(),
                    message: Some(req.message.to_string()),
                });
            }
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

pub async fn mount_playbooks_to_workspace(
    pool: &SqlitePool,
    cluster_id: &str,
    workspace_dir: &std::path::Path,
) -> Result<(), AppError> {
    let rows = sqlx::query(
        "SELECT id, title, concept_type, text FROM knowledge_store_meta WHERE cluster_id = ? AND concept_type = 'playbook'"
    )
    .bind(cluster_id)
    .fetch_all(pool)
    .await
    .map_err(|e| AppError::InternalServerError(format!("[OKF Gate] Failed to query playbooks for mounting: {}", e)))?;

    let mut playbooks = Vec::new();
    for row in rows {
        use sqlx::Row;
        let id: String = row.try_get("id")?;
        let title_opt: Option<String> = row.try_get("title")?;
        let concept_type: String = row.try_get("concept_type")?;
        let text: String = row.try_get("text")?;
        let title = title_opt.unwrap_or_else(|| id.clone());
        playbooks.push(PlaybookCachedDetail {
            id,
            title,
            concept_type,
            text,
        });
    }

    if workspace_dir.exists() {
        let json_path = workspace_dir.join("playbooks.json");
        let content = serde_json::to_string_pretty(&playbooks).map_err(|e| {
            AppError::InternalServerError(format!(
                "[OKF Gate] Failed to serialize playbooks: {}",
                e
            ))
        })?;

        tokio::fs::write(&json_path, content).await.map_err(|e| {
            AppError::InternalServerError(format!(
                "[OKF Gate] Failed to write playbooks.json: {}",
                e
            ))
        })?;
        tracing::info!(
            "✅ [OKF Gate] Mounted {} playbooks to {}",
            playbooks.len(),
            json_path.display()
        );
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::sqlite::SqlitePoolOptions;

    async fn setup_test_db() -> SqlitePool {
        let pool = SqlitePoolOptions::new()
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
                tags TEXT
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
        let res = validate_environments(&pool, "cl-test", &[]).await.unwrap();
        assert_eq!(res.status, "nominal");
        assert!(res.message.is_none());
    }

    #[tokio::test]
    async fn test_okf_validation_missing_docker() {
        let pool = setup_test_db().await;

        sqlx::query(
            r#"INSERT INTO knowledge_store_meta (id, text, concept_type, cluster_id, title)
               VALUES ('pb-1', 'This playbook requires docker for isolation.', 'playbook', 'cl-test', 'Docker Playbook')"#
        )
        .execute(&pool)
        .await
        .unwrap();

        let res = validate_environments(&pool, "cl-test", &["vs_code".to_string()])
            .await
            .unwrap();
        assert_eq!(res.status, "warning");
        assert!(res.message.unwrap().contains("Docker daemon"));

        let res = validate_environments(&pool, "cl-test", &["docker".to_string()])
            .await
            .unwrap();
        assert_eq!(res.status, "nominal");
    }

    #[tokio::test]
    async fn test_okf_validation_missing_k8s() {
        let pool = setup_test_db().await;

        sqlx::query(
            r#"INSERT INTO knowledge_store_meta (id, text, concept_type, cluster_id, title)
               VALUES ('pb-2', 'Operational step: requires k8s_node for orchestrations.', 'playbook', 'cl-test', 'K8s Playbook')"#
        )
        .execute(&pool)
        .await
        .unwrap();

        let res = validate_environments(&pool, "cl-test", &["docker".to_string()])
            .await
            .unwrap();
        assert_eq!(res.status, "critical");
        assert!(res.message.unwrap().contains("Kubernetes"));

        let res = validate_environments(&pool, "cl-test", &["k8s_node".to_string()])
            .await
            .unwrap();
        assert_eq!(res.status, "nominal");
    }

    #[tokio::test]
    async fn test_okf_validation_yaml_frontmatter() {
        let pool = setup_test_db().await;

        sqlx::query(
            r#"INSERT INTO knowledge_store_meta (id, text, concept_type, cluster_id, title)
               VALUES ('pb-3', '---
requires:
  - docker
  - jupyter_lab
---
Some instructions.', 'playbook', 'cl-test', 'YAML Playbook')"#,
        )
        .execute(&pool)
        .await
        .unwrap();

        // 1. Missing docker
        let res = validate_environments(&pool, "cl-test", &["jupyter_lab".to_string()])
            .await
            .unwrap();
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
}

// Metadata: [okf_gate]

//! @docs ARCHITECTURE:Gateways
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / HTTP Routes / system
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Type-safe state handling and bounded execution without unhandled panics.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: none declared
//! - **Witness Tests**: `system::tests::*`

use crate::error::AppError;
use crate::state::AppState;
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// Exposes the hardware profile of the Tadpole OS engine for sovereign compute telemetry.
#[tracing::instrument(skip(state), name = "system::compute_profile")]
pub async fn get_compute_profile(
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, AppError> {
    let profile = state.resources.hardware_profiler.get_profile();
    Ok((StatusCode::OK, Json(profile)))
}

#[derive(serde::Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct EnrichedWorkspaceStatus {
    pub id: String,
    pub agent_id: String,
    pub source_type: String,
    pub source_uri: String,
    pub last_sync_at: Option<chrono::DateTime<chrono::Utc>>,
    pub checksum: Option<String>,
    pub status: String,
    pub metadata: Option<String>,
    pub file_count: i32,
    pub total_bytes: i64,
    pub detected_environments: Vec<String>,
    pub mounted_okf_nodes: Vec<crate::system::okf_gate::OkfNodeInfo>,
    pub okf_validation: crate::system::okf_gate::OkfValidationResult,
}

pub fn resolve_cluster_id(source_uri: &str) -> String {
    if source_uri.contains("strategic-command") {
        "cl-command".to_string()
    } else if source_uri.contains("strategic-ops") {
        "cl-chain-a".to_string()
    } else if source_uri.contains("core-intelligence") {
        "cl-chain-b".to_string()
    } else if source_uri.contains("applied-growth") {
        "cl-chain-c".to_string()
    } else {
        for segment in source_uri.split('/') {
            if segment.starts_with("cl-") {
                return segment.to_string();
            }
        }
        if let Some(last) = source_uri.split('/').next_back() {
            if !last.is_empty() {
                return format!("cl-{}", last);
            }
        }
        "default".to_string()
    }
}

pub fn resolve_workspace_dir(base_dir: &Path, source_uri: &str) -> Result<PathBuf, AppError> {
    let clean_uri = source_uri.trim_start_matches('/').trim_start_matches('\\');
    let rel_path =
        if clean_uri.starts_with("data/workspaces") || clean_uri.starts_with("data\\workspaces") {
            PathBuf::from(clean_uri)
        } else if clean_uri.starts_with("workspaces/") || clean_uri.starts_with("workspaces\\") {
            PathBuf::from("data").join(clean_uri)
        } else {
            PathBuf::from("data/workspaces").join(clean_uri)
        };

    let safe_path = crate::utils::security::validate_path(base_dir, &rel_path.to_string_lossy())?;
    Ok(safe_path.to_path_buf())
}

/// Retrieves the synchronization status and metrics for all workspaces/connectors.
/// Pure GET query without file-writing side effects.
#[tracing::instrument(skip(state), name = "system::workspaces_status")]
pub async fn get_workspaces_status(
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, AppError> {
    let manifests =
        crate::agent::persistence::get_all_sync_manifests(&state.resources.pool).await?;

    let mut enriched = Vec::new();
    for m in manifests {
        let cluster_id = resolve_cluster_id(&m.source_uri);
        let detected_envs = crate::system::environment::detect_environments(&m.source_uri);
        let mounted_okf_nodes =
            crate::system::okf_gate::get_mounted_playbooks(&state.resources.pool, &cluster_id)
                .await?;
        let okf_validation = crate::system::okf_gate::validate_environments(
            &state.resources.pool,
            &cluster_id,
            &detected_envs,
        )
        .await?;

        enriched.push(EnrichedWorkspaceStatus {
            id: m.id,
            agent_id: m.agent_id,
            source_type: m.source_type,
            source_uri: m.source_uri,
            last_sync_at: m.last_sync_at,
            checksum: m.checksum,
            status: m.status,
            metadata: m.metadata,
            file_count: m.file_count,
            total_bytes: m.total_bytes,
            detected_environments: detected_envs,
            mounted_okf_nodes,
            okf_validation,
        });
    }

    Ok((StatusCode::OK, Json(enriched)))
}

pub const MAX_WORKSPACE_FILES_COUNT: usize = 5000;
pub const MAX_WORKSPACE_SCAN_DEPTH: usize = 32;

pub fn is_sensitive_file(rel_path: &str) -> bool {
    let lower = rel_path.to_lowercase();
    lower.starts_with(".env")
        || lower.contains("/.env")
        || lower.contains("\\.env")
        || lower.starts_with("data/")
        || lower.starts_with("data\\")
        || lower.contains("/data/")
        || lower.contains("\\data\\")
        || lower.ends_with(".key")
        || lower.ends_with(".pem")
        || lower.ends_with("credentials.json")
        || lower.ends_with("token.json")
        || lower.starts_with(".git/")
        || lower.starts_with(".git\\")
}

/// Helper to scan workspace files from a directory path.
pub async fn scan_workspace_files(
    base_dir: std::path::PathBuf,
) -> Result<(Vec<String>, bool), AppError> {
    tokio::task::spawn_blocking(move || {
        let mut results = Vec::new();
        let mut is_truncated = false;
        let mut visited_dirs = std::collections::HashSet::new();

        let canonical_base = match std::fs::canonicalize(&base_dir) {
            Ok(c) => c,
            Err(e) => {
                tracing::error!("Workspace base directory cannot be canonicalized: {}", e);
                return Err(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    format!("Invalid workspace directory: {}", e),
                ));
            }
        };

        visited_dirs.insert(canonical_base.clone());

        fn visit_dirs(
            dir: &Path,
            base: &Path,
            canonical_base: &Path,
            visited_dirs: &mut std::collections::HashSet<std::path::PathBuf>,
            depth: usize,
            results: &mut Vec<String>,
            truncated: &mut bool,
        ) -> std::io::Result<()> {
            if depth > MAX_WORKSPACE_SCAN_DEPTH {
                *truncated = true;
                return Ok(());
            }

            let entries = match std::fs::read_dir(dir) {
                Ok(entries) => entries,
                Err(e) => {
                    tracing::warn!("Failed to read directory '{}': {}", dir.display(), e);
                    *truncated = true;
                    return Ok(());
                }
            };

            for entry_res in entries {
                if results.len() >= MAX_WORKSPACE_FILES_COUNT {
                    *truncated = true;
                    return Ok(());
                }

                let entry = match entry_res {
                    Ok(e) => e,
                    Err(e) => {
                        tracing::warn!(
                            "Failed to read workspace entry in '{}': {}",
                            dir.display(),
                            e
                        );
                        *truncated = true;
                        continue;
                    }
                };

                let path = entry.path();
                let file_type = match entry.file_type() {
                    Ok(ft) => ft,
                    Err(e) => {
                        tracing::warn!(
                            "Failed to determine file type for '{}': {}",
                            path.display(),
                            e
                        );
                        continue;
                    }
                };

                // Defend against symlink escape & cyclical traversal (SEC-02)
                if file_type.is_symlink() {
                    match std::fs::canonicalize(&path) {
                        Ok(target) => {
                            if !target.starts_with(canonical_base) {
                                tracing::debug!(
                                    "Skipping symlink escaping workspace: {}",
                                    path.display()
                                );
                                continue;
                            }
                            if target.is_dir() {
                                tracing::debug!("Skipping directory symlink: {}", path.display());
                                continue;
                            }
                            if let Ok(target_rel) = target.strip_prefix(canonical_base) {
                                let target_rel_str =
                                    target_rel.to_string_lossy().replace('\\', "/");
                                if is_sensitive_file(&target_rel_str) {
                                    tracing::debug!(
                                        "Skipping symlink pointing to sensitive file: {}",
                                        path.display()
                                    );
                                    continue;
                                }
                            }
                        }
                        Err(_) => continue,
                    }
                }

                if let Ok(rel) = path.strip_prefix(base) {
                    let rel_str = rel.to_string_lossy().replace('\\', "/");
                    if is_sensitive_file(&rel_str) {
                        continue;
                    }

                    if file_type.is_dir() {
                        if let Ok(canonical_dir) = std::fs::canonicalize(&path) {
                            if !visited_dirs.insert(canonical_dir) {
                                continue;
                            }
                        }
                        visit_dirs(
                            &path,
                            base,
                            canonical_base,
                            visited_dirs,
                            depth + 1,
                            results,
                            truncated,
                        )?;
                    } else {
                        if let Ok(canon) = std::fs::canonicalize(&path) {
                            if let Ok(canon_rel) = canon.strip_prefix(canonical_base) {
                                let canon_rel_str = canon_rel.to_string_lossy().replace('\\', "/");
                                if is_sensitive_file(&canon_rel_str) {
                                    continue;
                                }
                            }
                        }
                        results.push(rel_str);
                    }
                }
            }

            Ok(())
        }

        // Verify root directory is accessible; propagate error if root is missing/inaccessible
        let _ = std::fs::read_dir(&base_dir)?;

        visit_dirs(
            &base_dir,
            &base_dir,
            &canonical_base,
            &mut visited_dirs,
            0,
            &mut results,
            &mut is_truncated,
        )?;

        Ok((results, is_truncated))
    })
    .await
    .map_err(|e| AppError::InternalServerError(format!("Workspace scan failed: {}", e)))?
    .map_err(AppError::Io)
}

/// Retrieves the list of files in the workspace (relative paths), excluding sensitive files.
#[tracing::instrument(skip(state), name = "system::workspace_files")]
pub async fn get_workspace_files(
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, AppError> {
    let (files, truncated) = scan_workspace_files(state.resources.base_dir.clone()).await?;

    Ok((
        StatusCode::OK,
        Json(serde_json::json!({
            "files": files,
            "total": files.len(),
            "truncated": truncated
        })),
    ))
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct UpdateEnvironmentRequest {
    pub variables: std::collections::HashMap<String, String>,
}

#[derive(Debug, serde::Serialize)]
pub struct UpdateEnvironmentResponse {
    pub status: String,
    pub updated_keys: Vec<String>,
    pub requires_restart: bool,
}

/// POST /v1/system/environment
/// Securely saves environment variables (e.g. MCP API keys) into .env and runtime state.
#[tracing::instrument(skip(state, payload), name = "system::update_environment")]
pub async fn update_environment_variables(
    State(state): State<Arc<AppState>>,
    _admin: crate::middleware::auth::RequireAdmin,
    Json(payload): Json<UpdateEnvironmentRequest>,
) -> Result<impl IntoResponse, AppError> {
    let mut updated_keys = Vec::new();
    let env_path = if state.base_dir.join(".env").exists() {
        state.base_dir.join(".env")
    } else if std::path::Path::new(".env").exists() {
        std::path::PathBuf::from(".env")
    } else {
        state.base_dir.join(".env")
    };

    let mut existing_env_map = std::collections::HashMap::new();
    if tokio::fs::try_exists(&env_path)
        .await
        .map_err(AppError::Io)?
    {
        let content = tokio::fs::read_to_string(&env_path)
            .await
            .map_err(AppError::Io)?;
        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            if let Some((k, v)) = line.split_once('=') {
                existing_env_map.insert(k.trim().to_string(), v.trim().to_string());
            }
        }
    }

    for (k, v) in &payload.variables {
        let clean_k = k.trim();
        if !clean_k
            .chars()
            .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '_')
        {
            return Err(AppError::BadRequest(format!(
                "Invalid environment variable name '{}'",
                clean_k
            )));
        }
        std::env::set_var(clean_k, v);
        existing_env_map.insert(clean_k.to_string(), v.clone());
        updated_keys.push(clean_k.to_string());
    }

    let mut new_content = String::new();
    let mut sorted_keys: Vec<_> = existing_env_map.keys().collect();
    sorted_keys.sort();
    for k in sorted_keys {
        if let Some(v) = existing_env_map.get(k) {
            new_content.push_str(&format!("{}={}\n", k, v));
        }
    }

    if let Some(parent) = env_path.parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .map_err(AppError::Io)?;
    }

    tokio::fs::write(&env_path, new_content.as_bytes())
        .await
        .map_err(AppError::Io)?;

    Ok((
        StatusCode::OK,
        Json(UpdateEnvironmentResponse {
            status: "success".to_string(),
            updated_keys,
            requires_restart: true,
        }),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_cluster_id() {
        assert_eq!(
            resolve_cluster_id("workspaces/strategic-command"),
            "cl-command"
        );
        assert_eq!(
            resolve_cluster_id("workspaces/cl-my-cluster"),
            "cl-my-cluster"
        );
        assert_eq!(
            resolve_cluster_id("workspaces/core-intelligence"),
            "cl-chain-b"
        );
    }

    #[test]
    fn test_is_sensitive_file() {
        assert!(is_sensitive_file(".env"));
        assert!(is_sensitive_file("subfolder/.env.local"));
        assert!(is_sensitive_file("data/workspaces/app.py"));
        assert!(is_sensitive_file("certs/server.key"));
        assert!(is_sensitive_file("credentials.json"));
        assert!(!is_sensitive_file("src/main.rs"));
        assert!(!is_sensitive_file("README.md"));
    }

    #[tokio::test]
    async fn test_get_workspace_files_success() {
        let state = Arc::new(AppState::new_mock().await);
        let file_path = state.resources.base_dir.join("hello.txt");
        std::fs::write(&file_path, "hello world").unwrap();

        let sub_dir = state.resources.base_dir.join("nested");
        std::fs::create_dir_all(&sub_dir).unwrap();
        let sub_file = sub_dir.join("inner.rs");
        std::fs::write(&sub_file, "fn main() {}").unwrap();

        let res = get_workspace_files(State(state)).await;
        assert!(res.is_ok());
    }

    #[tokio::test]
    async fn test_scan_workspace_files_inaccessible_root_returns_err() {
        let res =
            scan_workspace_files(std::path::PathBuf::from("Z:/nonexistent_test_root_12345")).await;
        assert!(res.is_err());
    }

    #[tokio::test]
    async fn test_scan_workspace_files_omits_sensitive_symlinks() {
        let temp = tempfile::tempdir().unwrap();
        let base_dir = temp.path().to_path_buf();

        // Create a sensitive file
        let env_path = base_dir.join(".env");
        std::fs::write(&env_path, "SECRET=12345").unwrap();

        // Create an innocuous file
        let regular_path = base_dir.join("regular.txt");
        std::fs::write(&regular_path, "normal content").unwrap();

        // Create an innocuous-looking symlink to .env
        let symlink_path = base_dir.join("innocuous.txt");
        #[cfg(target_os = "windows")]
        let link_created = std::os::windows::fs::symlink_file(&env_path, &symlink_path).is_ok();
        #[cfg(not(target_os = "windows"))]
        let link_created = std::os::unix::fs::symlink(&env_path, &symlink_path).is_ok();

        let (files, _) = scan_workspace_files(base_dir).await.unwrap();

        assert!(files.contains(&"regular.txt".to_string()));
        assert!(!files.contains(&".env".to_string()));
        if link_created {
            assert!(!files.contains(&"innocuous.txt".to_string()));
        }
    }
}

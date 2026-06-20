//! @docs ARCHITECTURE:Gateways
//!
//! ### AI Assist Note
//! **Core technical module for the Tadpole OS hardened engine.**
//! This module implements high-fidelity logic for the Sovereign Reality layer.
//!
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: Runtime logic error, state desynchronization, or resource exhaustion.
//! - **Telemetry Link**: Search `[system.rs]` in tracing logs.

use crate::error::AppError;
use crate::state::AppState;
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
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

fn resolve_cluster_id(source_uri: &str) -> String {
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

fn resolve_workspace_dir(base_dir: &std::path::Path, source_uri: &str) -> std::path::PathBuf {
    let clean_uri = source_uri.trim_start_matches('/').trim_start_matches('\\');
    if clean_uri.contains("data/workspaces") {
        base_dir.join(clean_uri)
    } else if clean_uri.contains("workspaces") {
        let relative = clean_uri.replace("workspaces/", "data/workspaces/");
        base_dir.join(relative)
    } else {
        base_dir.join("data").join("workspaces").join(clean_uri)
    }
}

/// Retrieves the synchronization status and metrics for all workspaces/connectors.
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
        let mounted_okf_nodes = crate::system::okf_gate::get_mounted_playbooks(&state.resources.pool, &cluster_id).await?;
        let okf_validation = crate::system::okf_gate::validate_environments(&state.resources.pool, &cluster_id, &detected_envs).await?;

        // Mount the playbooks JSON to the workspace cluster folder on disk
        let workspace_dir = resolve_workspace_dir(&state.base_dir, &m.source_uri);
        let _ = crate::system::okf_gate::mount_playbooks_to_workspace(
            &state.resources.pool,
            &cluster_id,
            &workspace_dir,
        ).await;

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

/// Retrieves the list of files in the workspace (relative paths).
#[tracing::instrument(skip(state), name = "system::workspace_files")]
pub async fn get_workspace_files(
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, AppError> {
    let base_dir = &state.resources.base_dir;
    let base_dir_clone = base_dir.clone();

    let files = tokio::task::spawn_blocking(move || {
        let mut list = Vec::new();
        for entry in walkdir::WalkDir::new(&base_dir_clone)
            .into_iter()
            .filter_entry(|e| {
                let name = e.file_name().to_string_lossy();
                name != "target"
                    && name != "node_modules"
                    && name != ".git"
                    && name != "dist"
                    && name != ".tmp"
                    && name != ".gemini"
            })
            .filter_map(|e| e.ok())
        {
            if entry.path().is_file() {
                if let Ok(rel_path) = entry.path().strip_prefix(&base_dir_clone) {
                    let rel_path_str = rel_path.to_string_lossy().replace('\\', "/");
                    list.push(rel_path_str);
                }
            }
            if list.len() >= 5000 {
                break;
            }
        }
        list
    })
    .await
    .map_err(|e| {
        AppError::InternalServerError(format!("Workspace files listing panicked: {}", e))
    })?;

    Ok((StatusCode::OK, Json(files)))
}

// Metadata: [system]

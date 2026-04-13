//! @docs ARCHITECTURE:Networking
//! 
//! ### AI Assist Note
//! **System Deployment (Update Manager)**: Orchestrates the live 
//! updating and deployment of engine components for the Tadpole OS 
//! ecosystem. Features **Multi-Bunker Support**: allows 
//! targeting specific deployment environments (Bunker 1/2) via the 
//! `target` query parameter. Implements **Secure Pipeline 
//! Execution**: triggers authenticated PowerShell deployment 
//! scripts (`deploy-bunker-X.ps1`) with real-time stdout/stderr 
//! capture. AI agents should use this endpoint to initiate 
//! production releases or environment-wide state migrations after 
//! passing all pre-flight audits (DEP-01).
//! 
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: 401 Unauthorized for invalid `NEURAL_TOKEN`, 
//!   500 Internal Server Error for missing deployment scripts, or 
//!   script timeouts during resource-intensive build processes.
//! - **Telemetry Link**: Search for `🚀 Authenticated deploy` or 
//!   `✅ Deployment succeeded` in `tracing` logs for audit history.
//! - **Trace Scope**: `server-rs::routes::deploy`

use crate::routes::error::AppError;
use crate::state::AppState;

use axum::extract::Query;
use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Standardized response for engine deployment operations.
#[derive(Serialize)]
pub struct DeployResponse {
    /// Operational status (e.g., "success", "error", "unauthorized").
    pub status: String,
    /// Captured stdout from the deployment script.
    pub output: Option<String>,
    /// Captured stderr or internal error message.
    pub error: Option<String>,
}

/// Request parameters for targeting specific deployment bunkers.
#[derive(Deserialize, Debug)]
pub struct DeployParams {
    /// Deployment target index (1 or 2).
    pub target: Option<u8>,
}

/// POST /engine/deploy — Triggers the deployment pipeline.
///
/// **Query Params**: `target` (1 or 2). Defaults to 1.
///
/// **Security**: Requires a valid `Authorization: Bearer <NEURAL_TOKEN>` header.
#[tracing::instrument(skip(state, headers), name = "system::deploy")]
pub async fn trigger_deploy(
    State(state): State<Arc<AppState>>,
    Query(params): Query<DeployParams>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, AppError> {
    // --- Authentication Gate ---
    let expected_token = &state.security.deploy_token;

    let provided = headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "));

    match provided {
        Some(token)
            if crate::middleware::auth::constant_time_eq(
                token.as_bytes(),
                expected_token.as_bytes(),
            ) => {}
        _ => {
            tracing::warn!("🚫 Unauthorized deploy attempt blocked.");
            return Ok((
                StatusCode::UNAUTHORIZED,
                Json(DeployResponse {
                    status: "unauthorized".to_string(),
                    output: None,
                    error: Some("Missing or invalid Authorization header.".to_string()),
                }),
            ));
        }
    }

    let target = params.target.unwrap_or(1);
    let script_file = match target {
        1 => "deploy-bunker-1.ps1",
        2 => "deploy-bunker-2.ps1",
        _ => {
            return Ok((
                StatusCode::BAD_REQUEST,
                Json(DeployResponse {
                    status: "error".to_string(),
                    output: None,
                    error: Some(format!(
                        "Invalid deployment target: {}. Must be 1 or 2.",
                        target
                    )),
                }),
            ));
        }
    };

    tracing::info!(
        "🚀 Authenticated deploy triggered for Bunker {} ({})...",
        target,
        script_file
    );

    // --- Path Resolution & Context Switching ---
    // If the server is started from the 'server-rs' directory, the scripts are in the root (..).
    // We must execute the command in the root so 'npm run build' inside the script works.
    let mut cmd = tokio::process::Command::new("powershell.exe");
    cmd.args(["-ExecutionPolicy", "Bypass", "-File", script_file]);

    if !std::path::Path::new(script_file).exists() {
        if std::path::Path::new("..").join(script_file).exists() {
            tracing::info!("📂 Script not found in CWD. Switching to project root (..) for deployment execution.");
            cmd.current_dir("..");
        } else {
            tracing::error!("❌ Deployment script not found in . or ..: {}", script_file);
            return Ok((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(DeployResponse {
                    status: "error".to_string(),
                    output: None,
                    error: Some(format!("Deployment script not found: {}", script_file)),
                }),
            ));
        }
    }

    // --- Async Process Execution ---
    let result = cmd.output().await;

    match result {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();

            if output.status.success() {
                tracing::info!("✅ Deployment succeeded.");
                if !stderr.is_empty() {
                    tracing::warn!("⚠️ Deployment stderr:\n{}", stderr);
                }
                Ok((
                    StatusCode::OK,
                    Json(DeployResponse {
                        status: "success".to_string(),
                        output: Some(stdout),
                        error: None,
                    }),
                ))
            } else {
                let combined = format!("{}\n{}", stdout, stderr).trim().to_string();
                let error_msg = if combined.is_empty() {
                    format!(
                        "{} exited with code {:?}",
                        script_file,
                        output.status.code()
                    )
                } else {
                    combined
                };
                Ok((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(DeployResponse {
                        status: "error".to_string(),
                        output: Some(stdout),
                        error: Some(error_msg),
                    }),
                ))
            }
        }
        Err(e) => {
            tracing::error!("❌ Failed to spawn PowerShell process: {}", e);
            Ok((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(DeployResponse {
                    status: "error".to_string(),
                    output: None,
                    error: Some(e.to_string()),
                }),
            ))
        }
    }
}

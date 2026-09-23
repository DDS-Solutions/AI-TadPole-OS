//! @docs ARCHITECTURE:Networking
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / HTTP Routes / deploy
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Type-safe state handling and bounded execution without unhandled panics.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: `AppError::BadRequest`, `AppError::Unauthorized`, `AppError::Conflict`, `AppError::InternalServerError`
//! - **Telemetry Targets**: `[Deploy]`
//! - **Witness Tests**: `deploy::tests::test_deploy_params_parsing`, `deploy::tests::test_concurrent_deploy_rejected`

use crate::error::AppError;
use crate::state::AppState;
use axum::extract::{Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::IntoResponse;
use axum::Json;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;
use tokio::sync::Mutex;

pub const DEPLOY_TIMEOUT_SECS: u64 = 300;
pub const MAX_DEPLOY_OUTPUT_BYTES: usize = 1024 * 1024; // 1 MB cap

static DEPLOY_LOCK: Mutex<()> = Mutex::const_new(());

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
            if !token.is_empty()
                && !expected_token.is_empty()
                && crate::middleware::auth::constant_time_eq(
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

    // --- Concurrency Gate ---
    let _deploy_guard = match DEPLOY_LOCK.try_lock() {
        Ok(guard) => guard,
        Err(_) => {
            tracing::warn!("⚠️ [Deploy] Concurrent deploy attempt rejected.");
            return Err(AppError::Conflict(
                "Deployment already in progress".to_string(),
            ));
        }
    };

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
    let mut cmd = tokio::process::Command::new("powershell.exe");
    cmd.args(["-ExecutionPolicy", "Bypass", "-File", script_file]);
    cmd.kill_on_drop(true); // 🛡️ Ensure child process terminates if timeout triggers or request is dropped

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

    // --- Async Process Execution with Timeout ---
    let result = tokio::time::timeout(
        std::time::Duration::from_secs(DEPLOY_TIMEOUT_SECS),
        cmd.output(),
    )
    .await;

    match result {
        Ok(Ok(output)) => {
            let mut stdout = String::from_utf8_lossy(&output.stdout).to_string();
            let mut stderr = String::from_utf8_lossy(&output.stderr).to_string();

            if stdout.len() > MAX_DEPLOY_OUTPUT_BYTES {
                stdout.truncate(MAX_DEPLOY_OUTPUT_BYTES);
                stdout.push_str("\n... [output truncated at 1MB ceiling]");
            }
            if stderr.len() > MAX_DEPLOY_OUTPUT_BYTES {
                stderr.truncate(MAX_DEPLOY_OUTPUT_BYTES);
                stderr.push_str("\n... [stderr truncated at 1MB ceiling]");
            }

            let redacted_stdout = state.security.secret_redactor.redact(&stdout);
            let redacted_stderr = state.security.secret_redactor.redact(&stderr);

            if output.status.success() {
                tracing::info!("✅ Deployment succeeded for Bunker {}", target);
                state.emit_event(json!({
                    "type": "deploy:completed",
                    "target": target,
                    "status": "success",
                    "timestamp": chrono::Utc::now().to_rfc3339()
                }));

                Ok((
                    StatusCode::OK,
                    Json(DeployResponse {
                        status: "success".to_string(),
                        output: Some(redacted_stdout),
                        error: None,
                    }),
                ))
            } else {
                let combined = format!("{}\n{}", redacted_stdout, redacted_stderr)
                    .trim()
                    .to_string();
                let error_msg = if combined.is_empty() {
                    format!(
                        "{} exited with code {:?}",
                        script_file,
                        output.status.code()
                    )
                } else {
                    combined
                };

                state.emit_event(json!({
                    "type": "deploy:completed",
                    "target": target,
                    "status": "failed",
                    "error": &error_msg,
                    "timestamp": chrono::Utc::now().to_rfc3339()
                }));

                Ok((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(DeployResponse {
                        status: "error".to_string(),
                        output: Some(redacted_stdout),
                        error: Some(error_msg),
                    }),
                ))
            }
        }
        Ok(Err(e)) => {
            tracing::error!("❌ Failed to spawn PowerShell process: {}", e);
            let safe_err = state.security.secret_redactor.redact(&e.to_string());
            Ok((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(DeployResponse {
                    status: "error".to_string(),
                    output: None,
                    error: Some(safe_err),
                }),
            ))
        }
        Err(_) => {
            tracing::error!(
                "❌ Deployment script execution timed out after {} seconds",
                DEPLOY_TIMEOUT_SECS
            );
            Ok((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(DeployResponse {
                    status: "error".to_string(),
                    output: None,
                    error: Some(format!(
                        "Deployment script execution timed out after {} seconds.",
                        DEPLOY_TIMEOUT_SECS
                    )),
                }),
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deploy_params_parsing() {
        let params_json = serde_json::json!({ "target": 2 });
        let parsed: DeployParams = serde_json::from_value(params_json).unwrap();
        assert_eq!(parsed.target, Some(2));
    }

    #[tokio::test]
    async fn test_concurrent_deploy_rejected() {
        let guard = DEPLOY_LOCK.try_lock();
        assert!(guard.is_ok());

        let second_guard = DEPLOY_LOCK.try_lock();
        assert!(second_guard.is_err());
    }
}

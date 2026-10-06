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

fn pinned_deploy_script(script_name: &str) -> Result<std::path::PathBuf, String> {
    let scripts_dir = std::path::Path::new("scripts");
    let canon_dir = scripts_dir.canonicalize().map_err(|_| {
        "Deployment scripts directory is missing. Expected ./scripts/deploy-bunker-N.ps1"
            .to_string()
    })?;
    let candidate = scripts_dir.join(script_name);
    let canon = candidate
        .canonicalize()
        .map_err(|_| format!("Deployment script not found: scripts/{script_name}"))?;
    if !canon.starts_with(&canon_dir) {
        return Err("Deployment script escaped the scripts directory".to_string());
    }
    Ok(canon)
}

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
/// **Security**: Requires an admin token when `NEURAL_ADMIN_TOKEN` is set, otherwise the deploy token. The script path is pinned under `scripts/`.
#[tracing::instrument(skip(state, headers), name = "system::deploy")]
pub async fn trigger_deploy(
    State(state): State<Arc<AppState>>,
    Query(params): Query<DeployParams>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, AppError> {
    // --- Authentication Gate ---
    // Prefer the admin token so a deploy credential cannot launch host scripts.
    // Fall back to the deploy token only when no admin token is configured.
    let expected_token = if !state.security.admin_token.is_empty() {
        &state.security.admin_token
    } else {
        &state.security.deploy_token
    };

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
    let script_name = match target {
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

    let script_path = match pinned_deploy_script(script_name) {
        Ok(path) => path,
        Err(error) => {
            tracing::error!("❌ {}", error);
            return Ok((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(DeployResponse {
                    status: "error".to_string(),
                    output: None,
                    error: Some(error),
                }),
            ));
        }
    };
    let script_file = script_path.display().to_string();

    tracing::info!(
        "🚀 Authenticated deploy triggered for Bunker {} ({})...",
        target,
        script_file
    );

    let mut cmd = crate::utils::security::create_isolated_command("powershell.exe");
    cmd.args([
        "-NoProfile",
        "-NonInteractive",
        "-ExecutionPolicy",
        "RemoteSigned",
        "-File",
        &script_file,
    ]);
    cmd.stdout(std::process::Stdio::piped());
    cmd.stderr(std::process::Stdio::piped());

    let mut child = match cmd.spawn() {
        Ok(child) => child,
        Err(e) => {
            tracing::error!("❌ Failed to spawn PowerShell process: {}", e);
            let safe_err = state.security.secret_redactor.redact(&e.to_string());
            return Ok((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(DeployResponse {
                    status: "error".to_string(),
                    output: None,
                    error: Some(safe_err),
                }),
            ));
        }
    };

    let mut child_stdout = child.stdout.take();
    let mut child_stderr = child.stderr.take();

    let run_future = async {
        let mut stdout_buf = Vec::new();
        let mut stderr_buf = Vec::new();
        tokio::join!(
            async {
                if let Some(pipe) = child_stdout.as_mut() {
                    let _ = tokio::io::AsyncReadExt::read_to_end(pipe, &mut stdout_buf).await;
                }
            },
            async {
                if let Some(pipe) = child_stderr.as_mut() {
                    let _ = tokio::io::AsyncReadExt::read_to_end(pipe, &mut stderr_buf).await;
                }
            }
        );
        let status = child.wait().await;
        (status, stdout_buf, stderr_buf)
    };

    // --- Async Process Execution with Timeout ---
    let (status, stdout_bytes, stderr_bytes) = match tokio::time::timeout(
        std::time::Duration::from_secs(DEPLOY_TIMEOUT_SECS),
        run_future,
    )
    .await
    {
        Ok((Ok(status), out, err)) => (status, out, err),
        Ok((Err(e), _, _)) => {
            tracing::error!("❌ PowerShell process wait failed: {}", e);
            let safe_err = state.security.secret_redactor.redact(&e.to_string());
            return Ok((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(DeployResponse {
                    status: "error".to_string(),
                    output: None,
                    error: Some(safe_err),
                }),
            ));
        }
        Err(_) => {
            let _ = child.start_kill();
            let _ = child.wait().await;
            tracing::error!(
                "❌ Deployment script execution timed out after {} seconds",
                DEPLOY_TIMEOUT_SECS
            );
            return Ok((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(DeployResponse {
                    status: "error".to_string(),
                    output: None,
                    error: Some(format!(
                        "Deployment script execution timed out after {} seconds.",
                        DEPLOY_TIMEOUT_SECS
                    )),
                }),
            ));
        }
    };

    let mut stdout = String::from_utf8_lossy(&stdout_bytes).to_string();
    let mut stderr = String::from_utf8_lossy(&stderr_bytes).to_string();

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

    if status.success() {
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
                status.code()
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

    #[test]
    fn test_pinned_deploy_script_resolution() {
        if std::path::Path::new("scripts").exists() {
            let res = pinned_deploy_script("deploy-bunker-1.ps1");
            assert!(
                res.is_ok(),
                "Expected deploy-bunker-1.ps1 to resolve: {:?}",
                res
            );

            let res_escape = pinned_deploy_script("../Cargo.toml");
            assert!(res_escape.is_err(), "Expected escaping scripts dir to fail");

            let res_nonexistent = pinned_deploy_script("nonexistent_script_xyz.ps1");
            assert!(res_nonexistent.is_err(), "Expected missing script to fail");
        }
    }
}

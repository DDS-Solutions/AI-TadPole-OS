//! @docs ARCHITECTURE:Registry:Mcp
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Agent Runner / MCP Skill Execution Sandbox
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Child process kill_on_drop(true) to prevent orphaned zombie processes.
//! - `[Structural]` Strict output stream limits (8 MiB stdout, 512 KiB stderr) preventing heap exhaustion.
//! - `[Structural]` Sensitive host environment scrubbing (tokens, keys, secrets, TADPOLE_*).
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: `AppError::InfrastructureError`, `AppError::Forbidden`
//! - **Telemetry Targets**: none declared

use std::path::Path;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::process::Command;

use crate::agent::script_skills::SkillDefinition;
use crate::error::{AppError, InfrastructureErrorKind, ProviderId};

pub const DEFAULT_SKILL_TIMEOUT: Duration = Duration::from_secs(60);
pub const MAX_SKILL_STDOUT_BYTES: usize = 8 * 1024 * 1024;
pub const MAX_SKILL_STDERR_BYTES: usize = 512 * 1024;

/// Executes a legacy skill within a constrained and guarded child process sandbox.
pub async fn execute_legacy_skill(
    skill: &SkillDefinition,
    arguments: serde_json::Value,
    workspace_root: &Path,
) -> Result<String, AppError> {
    crate::utils::security::validate_shell_command(&skill.execution_command)
        .map_err(|e| AppError::Forbidden(format!("Skill security violation: {}", e)))?;

    let mut parts = skill.execution_command.split_whitespace();
    let program = parts
        .next()
        .ok_or_else(|| AppError::BadRequest("Empty execution command".to_string()))?;
    let args: Vec<&str> = parts.collect();

    let mut cmd = Command::new(program);
    cmd.args(&args);
    cmd.current_dir(workspace_root);
    cmd.stdin(std::process::Stdio::piped());
    cmd.stdout(std::process::Stdio::piped());
    cmd.stderr(std::process::Stdio::piped());

    // Kill the process immediately if the future/task is dropped
    cmd.kill_on_drop(true);

    // Scrub sensitive host environment variables to prevent accidental credential leakage
    for (k, _) in std::env::vars() {
        let upper = k.to_uppercase();
        if upper.starts_with("TADPOLE_")
            || upper.contains("TOKEN")
            || upper.contains("KEY")
            || upper.contains("SECRET")
            || upper == "ADMIN_TOKEN"
        {
            cmd.env_remove(&k);
        }
    }

    // Set controlled execution environment
    cmd.env("TADPOLE_SKILL_NAME", &skill.name);
    cmd.env(
        "TADPOLE_WORKSPACE",
        workspace_root.to_string_lossy().as_ref(),
    );

    let mut child = cmd.spawn().map_err(AppError::Io)?;

    let input_bytes =
        serde_json::to_vec(&arguments).map_err(|e| AppError::BadRequest(e.to_string()))?;

    if let Some(mut stdin) = child.stdin.take() {
        tokio::spawn(async move {
            let _ = stdin.write_all(&input_bytes).await;
            let _ = stdin.shutdown().await;
        });
    }

    let mut stdout = child.stdout.take().ok_or_else(|| {
        AppError::InternalServerError("Failed to capture skill stdout".to_string())
    })?;
    let mut stderr = child.stderr.take().ok_or_else(|| {
        AppError::InternalServerError("Failed to capture skill stderr".to_string())
    })?;

    let mut raw_stdout = Vec::new();
    let mut raw_stderr = Vec::new();

    let exec_future = async {
        let mut stdout_buf = [0u8; 8192];
        let mut stderr_buf = [0u8; 8192];

        loop {
            tokio::select! {
                n = stdout.read(&mut stdout_buf) => {
                    let n = n.map_err(AppError::Io)?;
                    if n == 0 {
                        let _ = stdout.read_to_end(&mut raw_stdout).await;
                        break;
                    }
                    if raw_stdout.len() + n > MAX_SKILL_STDOUT_BYTES {
                        return Err(AppError::InfrastructureError {
                            provider_id: ProviderId::System,
                            kind: InfrastructureErrorKind::ApiError,
                            detail: format!("Skill stdout exceeded maximum limit of {} bytes", MAX_SKILL_STDOUT_BYTES),
                            help_link: None,
                        });
                    }
                    raw_stdout.extend_from_slice(&stdout_buf[..n]);
                }
                n = stderr.read(&mut stderr_buf) => {
                    let n = n.map_err(AppError::Io)?;
                    if n == 0 {
                        let _ = stderr.read_to_end(&mut raw_stderr).await;
                        break;
                    }
                    if raw_stderr.len() + n > MAX_SKILL_STDERR_BYTES {
                        return Err(AppError::InfrastructureError {
                            provider_id: ProviderId::System,
                            kind: InfrastructureErrorKind::ApiError,
                            detail: format!("Skill stderr exceeded maximum limit of {} bytes", MAX_SKILL_STDERR_BYTES),
                            help_link: None,
                        });
                    }
                    raw_stderr.extend_from_slice(&stderr_buf[..n]);
                }
            }
        }

        let status = child.wait().await.map_err(AppError::Io)?;
        Ok::<_, AppError>(status)
    };

    let status = match tokio::time::timeout(DEFAULT_SKILL_TIMEOUT, exec_future).await {
        Ok(res) => res?,
        Err(_) => {
            return Err(AppError::InfrastructureError {
                provider_id: ProviderId::System,
                kind: InfrastructureErrorKind::Timeout,
                detail: format!(
                    "Skill execution timed out after {:?}",
                    DEFAULT_SKILL_TIMEOUT
                ),
                help_link: None,
            });
        }
    };

    let stdout_str = String::from_utf8_lossy(&raw_stdout).to_string();
    if status.success() {
        Ok(stdout_str)
    } else {
        let stderr_str = String::from_utf8_lossy(&raw_stderr).to_string();
        Err(AppError::InfrastructureError {
            provider_id: ProviderId::System,
            kind: InfrastructureErrorKind::ApiError,
            detail: format!("Skill failed with status {}: {}", status, stderr_str),
            help_link: None,
        })
    }
}

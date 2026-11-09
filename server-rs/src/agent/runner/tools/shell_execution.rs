//! @docs ARCHITECTURE:Runner
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Agent Runner / shell_execution
//!
//! ### AI Assist Note
//! Spawns direct binary executables in the workspace without shell interpreter.
//! Enforces Zero-Trust path confinement, ShellScanner rules, and Sapphire Gate.
//!
//! ### 🔍 Debugging & Observability
//! - **Telemetry Targets**: none declared

use crate::agent::runner::{AgentRunner, RunContext};
use crate::error::AppError;
use std::path::Path;
use tokio::io::AsyncReadExt;

fn validate_command_security(
    command_display: &str,
    scanner: &crate::security::scanner::ShellScanner,
) -> Result<(), String> {
    if let Err(e) = crate::utils::security::validate_shell_command(command_display) {
        return Err(format!("SECURITY BLOCKED: {}", e));
    }

    if let crate::security::scanner::ScannerResult::Risky(reason) = scanner.scan(command_display) {
        return Err(format!("SECURITY BLOCKED: {}", reason));
    }

    Ok(())
}

fn extract_path_candidate(arg: &str) -> Option<&str> {
    if let Some((_, val)) = arg.split_once('=') {
        Some(val)
    } else if !arg.starts_with('-') {
        Some(arg)
    } else {
        None
    }
}

fn is_potential_path(candidate: &str) -> bool {
    let has_seps = candidate.contains('/') || candidate.contains('\\') || candidate.contains("..");
    let has_drive = candidate.len() >= 2
        && candidate
            .chars()
            .next()
            .map(|c| c.is_ascii_alphabetic())
            .unwrap_or(false)
        && candidate.chars().nth(1) == Some(':');
    let is_rooted = candidate.starts_with('/') || candidate.starts_with('\\');
    has_seps || has_drive || is_rooted
}

fn validate_single_candidate(
    candidate: &str,
    arg: &str,
    run_dir: &Path,
    workspace_root: &Path,
) -> Result<(), AppError> {
    let cand_path = Path::new(candidate);
    if cand_path.is_absolute() {
        crate::security::path_guard::validate_path(workspace_root, candidate)
            .map_err(|e| AppError::Forbidden(format!("SECURITY BLOCKED: {}", e)))?;
        return Ok(());
    }
    let full_rel = run_dir.join(cand_path);
    let Ok(rel_str) = full_rel.strip_prefix(workspace_root) else {
        return Err(AppError::Forbidden(format!(
            "Command argument '{}' targets location outside workspace root",
            arg
        )));
    };
    crate::security::path_guard::validate_path(workspace_root, &rel_str.to_string_lossy())
        .map_err(|e| AppError::Forbidden(format!("SECURITY BLOCKED: {}", e)))?;
    Ok(())
}

fn validate_path_sandbox(
    args: &[String],
    run_dir: &Path,
    workspace_root: &Path,
) -> Result<(), AppError> {
    for arg in args {
        let Some(candidate) = extract_path_candidate(arg) else {
            continue;
        };
        if is_potential_path(candidate) {
            validate_single_candidate(candidate, arg, run_dir, workspace_root)?;
        }
    }
    Ok(())
}

struct ProcessStreamResult {
    exit_code: i32,
    stdout_trunc: bool,
    stdout_buf: Vec<u8>,
    stderr_trunc: bool,
    stderr_buf: Vec<u8>,
}

async fn read_bounded_pipe<R: AsyncReadExt + Unpin>(pipe: Option<R>) -> (bool, Vec<u8>) {
    let mut buf = Vec::new();
    let mut is_trunc = false;
    if let Some(stream) = pipe {
        if let Ok(n) = stream.take(32_769).read_to_end(&mut buf).await {
            if n > 32_768 {
                buf.truncate(32_768);
                is_trunc = true;
            }
        }
    }
    (is_trunc, buf)
}

async fn collect_process_output(mut child: tokio::process::Child) -> ProcessStreamResult {
    let stdout_pipe = child.stdout.take();
    let stderr_pipe = child.stderr.take();

    let ((stdout_trunc, stdout_buf), (stderr_trunc, stderr_buf)) = tokio::join!(
        read_bounded_pipe(stdout_pipe),
        read_bounded_pipe(stderr_pipe)
    );

    let exit_code = match child.wait().await {
        Ok(status) => status.code().unwrap_or(-1),
        Err(_) => -1,
    };

    ProcessStreamResult {
        exit_code,
        stdout_trunc,
        stdout_buf,
        stderr_trunc,
        stderr_buf,
    }
}

async fn kill_child_process(mut child: tokio::process::Child) {
    if let Some(pid) = child.id() {
        #[cfg(target_os = "windows")]
        {
            let _ = tokio::process::Command::new("taskkill")
                .args(["/F", "/T", "/PID", &pid.to_string()])
                .output()
                .await;
        }
        #[cfg(not(target_os = "windows"))]
        {
            let _ = tokio::process::Command::new("kill")
                .args(["-9", &format!("-{}", pid)])
                .output()
                .await;
        }
    }
    let _ = child.start_kill();
    let _ = child.wait().await;
}

fn record_execution(ctx: &RunContext, cmd_name: String, exit_code: i32) {
    let timestamp_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0);

    ctx.execution_records
        .lock()
        .push(crate::agent::runner::ProcessExecutionRecord {
            command: cmd_name,
            exit_code,
            timestamp_ms,
        });
}

fn format_io_output(
    runner: &AgentRunner,
    command_display: &str,
    io: ProcessStreamResult,
) -> String {
    let stdout_str = String::from_utf8_lossy(&io.stdout_buf);
    let stderr_str = String::from_utf8_lossy(&io.stderr_buf);
    let combined = format!("{}{}", stdout_str, stderr_str);
    let mut truncated = runner.safe_truncate(&combined, 5000);
    if io.stdout_trunc || io.stderr_trunc {
        truncated.push_str("\n(OUTPUT TRUNCATED AT 32KB SAFETY LIMIT)");
    }
    format!(
        "(PROCESS OUTPUT of '{}' [exit code: {}]):\n\n{}",
        command_display, io.exit_code, truncated
    )
}

impl AgentRunner {
    async fn request_shell_oversight(
        &self,
        ctx: &RunContext,
        fc: &crate::agent::types::ToolCall,
        command_display: &str,
    ) -> Result<bool, AppError> {
        self.broadcast_agent(
            ctx,
            &format!(
                "💎 Oversight: wants to run binary: {}. CRITICAL REVIEW REQUIRED.",
                command_display
            ),
            "error",
        );

        self.submit_oversight(
            crate::agent::types::ToolCallAudit {
                id: uuid::Uuid::new_v4().to_string(),
                agent_id: ctx.identity.agent_id.clone(),
                mission_id: Some(ctx.mission_state.mission_id.clone()),
                skill: "execute_shell".to_string(),
                params: fc.args.clone(),
                department: ctx.identity.department.clone(),
                description: format!("Executing binary process in workspace: {}", command_display),
                timestamp: chrono::Utc::now().to_rfc3339(),
            },
            Some(ctx.mission_state.mission_id.clone()),
        )
        .await
    }

    fn handle_cd_builtin(
        &self,
        ctx: &RunContext,
        args: &[String],
        run_dir: &Path,
        output_text: &mut String,
    ) {
        let target = args.first().map(|s| s.as_str()).unwrap_or(".");
        let cd_args = serde_json::json!({"cwd": run_dir.join(target)});
        match super::validation::shell_working_directory(&cd_args, &ctx.env.workspace_root, None) {
            Ok(canonical) => {
                *ctx.current_dir.lock() = Some(canonical.clone());
                *output_text = format!("(Directory changed to: {})", canonical.display());
            }
            Err(error) => *output_text = format!("(Directory change FAILED: {error})"),
        }
    }

    async fn execute_spawned_child(
        &self,
        ctx: &RunContext,
        child: tokio::process::Child,
        command_display: &str,
        timeout: std::time::Duration,
    ) -> String {
        let Ok(io) = tokio::time::timeout(timeout, collect_process_output(child)).await else {
            record_execution(ctx, format!("{} [TIMED_OUT]", command_display), -1);
            return format!(
                "(TOOL TIMEOUT: Child process '{}' timed out after {:?} and was killed)",
                command_display, timeout
            );
        };
        record_execution(ctx, command_display.to_string(), io.exit_code);
        format_io_output(self, command_display, io)
    }

    /// Handles `execute_shell`: runs a direct binary executable in the workspace.
    pub(crate) async fn handle_execute_shell(
        &self,
        ctx: &RunContext,
        fc: &crate::agent::types::ToolCall,
        output_text: &mut String,
    ) -> Result<(), AppError> {
        let (executable, args) = super::validation::extract_shell_command(&fc.args)?;
        let current_dir = ctx.current_dir.lock().clone();
        let run_dir = super::validation::shell_working_directory(
            &fc.args,
            &ctx.env.workspace_root,
            current_dir.as_deref(),
        )?;
        let full_command_display = super::validation::shell_command_display(&executable, &args);

        if let Err(msg) =
            validate_command_security(&full_command_display, &self.state.security.shell_scanner)
        {
            *output_text = format!("({})", msg);
            return Ok(());
        }

        if let Err(e) = validate_path_sandbox(&args, &run_dir, &ctx.env.workspace_root) {
            *output_text = format!("({})", e);
            return Ok(());
        }

        let approved = self
            .request_shell_oversight(ctx, fc, &full_command_display)
            .await?;
        if !approved {
            *output_text = "(Process execution REJECTED by Oversight)".to_string();
            return Ok(());
        }

        if executable == "cd" {
            self.handle_cd_builtin(ctx, &args, &run_dir, output_text);
            return Ok(());
        }

        let mut cmd = crate::utils::security::create_isolated_command(&executable);
        cmd.args(&args)
            .current_dir(&run_dir)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .env(
                "TADPOLE_WORKSPACE",
                ctx.env.workspace_root.to_string_lossy().as_ref(),
            )
            .env("TADPOLE_AGENT_ID", &ctx.identity.agent_id);

        let timeout_secs = fc
            .args
            .get("timeout")
            .and_then(|v| v.as_u64())
            .unwrap_or(60);
        let command_timeout = std::time::Duration::from_secs(timeout_secs.clamp(1, 300));

        match cmd.spawn() {
            Ok(child) => {
                ctx.commands_run.lock().insert(full_command_display.clone());
                *output_text = self
                    .execute_spawned_child(ctx, child, &full_command_display, command_timeout)
                    .await;
            }
            Err(e) => {
                tracing::error!("❌ Failed to spawn process '{}': {}", executable, e);
                *output_text = format!("(PROCESS SPAWN ERROR for '{}': {})", executable, e);
            }
        }

        Ok(())
    }
}

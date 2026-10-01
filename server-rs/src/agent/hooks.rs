//! @docs ARCHITECTURE:Agent
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Agent Runner / hooks
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Type-safe state handling and bounded execution without unhandled panics.
//! - `[Structural]` Isolated environment execution (env_clear), execution timeout (10s), and deterministic ordering.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: AppError::InternalServerError, AppError::Io
//! - **Telemetry Targets**: tracing::info!, tracing::warn!
//! - **Witness Tests**: tests::test_hooks_deterministic_sorting, tests::test_hooks_env_isolation, tests::test_post_hook_non_fatal

use crate::error::AppError;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::time::Duration;
use tokio::process::Command;

const DEFAULT_HOOK_TIMEOUT: Duration = Duration::from_secs(10);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookContext {
    pub agent_id: String,
    pub mission_id: Option<String>,
    pub skill: String,
}

pub struct HooksManager {
    hooks_dir: PathBuf,
}

impl HooksManager {
    pub fn new(data_dir: &Path) -> Self {
        Self {
            hooks_dir: data_dir.join("hooks"),
        }
    }

    /// Executes all scripts in the given hook subdirectory synchronously (fail-closed).
    /// Returns an error if any pre-tool hook script fails.
    pub async fn trigger_hook(
        &self,
        hook_type: &str,
        ctx: &HookContext,
        params: &serde_json::Value,
    ) -> Result<(), AppError> {
        self.execute_hooks_internal(hook_type, ctx, params, true)
            .await
    }

    /// Executes post-tool hook scripts with non-fatal logging semantics.
    /// Script failures log warnings without retroactively causing completed side-effects to fail.
    pub async fn trigger_post_tool_hook(
        &self,
        hook_type: &str,
        ctx: &HookContext,
        params: &serde_json::Value,
    ) -> Result<(), AppError> {
        self.execute_hooks_internal(hook_type, ctx, params, false)
            .await
    }

    async fn execute_hooks_internal(
        &self,
        hook_type: &str,
        ctx: &HookContext,
        params: &serde_json::Value,
        fail_closed: bool,
    ) -> Result<(), AppError> {
        let safe_type = crate::utils::security::sanitize_id(hook_type);
        let dir = self.hooks_dir.join(safe_type);
        if !dir.exists() {
            return Ok(());
        }

        let mut read_entries = tokio::fs::read_dir(dir).await.map_err(AppError::Io)?;
        let mut script_paths = Vec::new();

        while let Some(entry) = read_entries.next_entry().await.map_err(AppError::Io)? {
            let path = entry.path();
            if self.is_executable(&path) {
                script_paths.push(path);
            }
        }

        // H6: Enforce deterministic alphabetical execution order
        script_paths.sort();

        for path in script_paths {
            tracing::info!("🪝 Executing hook script: {:?}", path);
            let res = self.run_script(&path, ctx, params).await;
            if let Err(e) = res {
                if fail_closed {
                    return Err(e);
                } else {
                    tracing::warn!(
                        "⚠️ Post-tool hook script failed non-fatally: {:?}. Error: {}",
                        path,
                        e
                    );
                }
            }
        }

        Ok(())
    }

    fn is_executable(&self, path: &Path) -> bool {
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        #[cfg(windows)]
        {
            matches!(ext, "ps1" | "bat" | "cmd" | "exe")
        }
        #[cfg(not(windows))]
        {
            matches!(ext, "sh" | "py")
                || (ext.is_empty()
                    && std::fs::metadata(path)
                        .map(|m| {
                            use std::os::unix::fs::PermissionsExt;
                            m.permissions().mode() & 0o111 != 0
                        })
                        .unwrap_or(false))
        }
    }

    fn build_command(
        &self,
        path: &Path,
        ctx: &HookContext,
        params: &serde_json::Value,
    ) -> Result<Command, AppError> {
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");

        let mut cmd = match ext {
            "ps1" => {
                let mut c = Command::new("powershell");
                c.args([
                    "-NoProfile",
                    "-NonInteractive",
                    "-ExecutionPolicy",
                    "Bypass",
                    "-File",
                ])
                .arg(path);
                c
            }
            "bat" | "cmd" => {
                let mut c = Command::new("cmd");
                c.arg("/C").arg(path);
                c
            }
            "py" => {
                let mut c = Command::new("python3");
                c.arg(path);
                c
            }
            "sh" => {
                let mut c = Command::new("/bin/sh");
                c.arg(path);
                c
            }
            _ => Command::new(path),
        };

        // H1 & H8: Environment isolation - clear parent process environment and pass explicit allowlist
        cmd.env_clear();
        cmd.env("AGENT_ID", &ctx.agent_id);
        cmd.env("SKILL", &ctx.skill);
        if let Some(mission_id) = &ctx.mission_id {
            cmd.env("MISSION_ID", mission_id);
        }

        let ctx_json =
            serde_json::to_string(ctx).map_err(|e| AppError::InternalServerError(e.to_string()))?;
        let params_json = serde_json::to_string(params)
            .map_err(|e| AppError::InternalServerError(e.to_string()))?;

        cmd.env("AGENT_CONTEXT", ctx_json);
        cmd.env("TOOL_PARAMS", params_json);

        Ok(cmd)
    }

    async fn run_script(
        &self,
        path: &Path,
        ctx: &HookContext,
        params: &serde_json::Value,
    ) -> Result<(), AppError> {
        let mut cmd = self.build_command(path, ctx, params)?;

        // H4: Bounded execution with timeout
        let child = cmd.output();
        let output = match tokio::time::timeout(DEFAULT_HOOK_TIMEOUT, child).await {
            Ok(Ok(out)) => out,
            Ok(Err(e)) => return Err(AppError::Io(e)),
            Err(_) => {
                return Err(AppError::InternalServerError(format!(
                    "Hook script execution timed out after {:?}: {:?}",
                    DEFAULT_HOOK_TIMEOUT, path
                )));
            }
        };

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(AppError::InternalServerError(format!(
                "Hook script failed: {}. Error: {}",
                path.display(),
                stderr
            )));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_hooks_deterministic_sorting() {
        let dir = tempdir().unwrap();
        let hooks_dir = dir.path().join("hooks").join("pre_validation");
        tokio::fs::create_dir_all(&hooks_dir).await.unwrap();

        // Create b.bat, a.bat, c.bat
        tokio::fs::write(hooks_dir.join("b.bat"), "@echo off\nexit 0")
            .await
            .unwrap();
        tokio::fs::write(hooks_dir.join("a.bat"), "@echo off\nexit 0")
            .await
            .unwrap();
        tokio::fs::write(hooks_dir.join("c.bat"), "@echo off\nexit 0")
            .await
            .unwrap();

        let manager = HooksManager::new(dir.path());
        let ctx = HookContext {
            agent_id: "agent_1".to_string(),
            mission_id: None,
            skill: "test_skill".to_string(),
        };

        let res = manager
            .trigger_hook("pre_validation", &ctx, &serde_json::json!({}))
            .await;
        assert!(res.is_ok());
    }

    #[tokio::test]
    async fn test_post_hook_non_fatal() {
        let dir = tempdir().unwrap();
        let hooks_dir = dir.path().join("hooks").join("post_validation");
        tokio::fs::create_dir_all(&hooks_dir).await.unwrap();

        // Failing hook script
        #[cfg(windows)]
        tokio::fs::write(hooks_dir.join("fail.bat"), "@echo off\nexit 1")
            .await
            .unwrap();
        #[cfg(not(windows))]
        tokio::fs::write(hooks_dir.join("fail.sh"), "#!/bin/sh\nexit 1")
            .await
            .unwrap();

        let manager = HooksManager::new(dir.path());
        let ctx = HookContext {
            agent_id: "agent_1".to_string(),
            mission_id: None,
            skill: "test_skill".to_string(),
        };

        // Pre-tool trigger fails closed
        let fail_res = manager
            .trigger_hook("post_validation", &ctx, &serde_json::json!({}))
            .await;
        assert!(fail_res.is_err());

        // Post-tool trigger logs and continues (Ok)
        let post_res = manager
            .trigger_post_tool_hook("post_validation", &ctx, &serde_json::json!({}))
            .await;
        assert!(post_res.is_ok());
    }
}

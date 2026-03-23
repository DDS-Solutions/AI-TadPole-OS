use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tokio::process::Command;
// use crate::agent::types::ToolCall;

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

    /// Executes all scripts in the given hook subdirectory.
    /// Returns an error if any script fails.
    pub async fn trigger_hook(
        &self,
        hook_type: &str,
        ctx: &HookContext,
        params: &serde_json::Value,
    ) -> anyhow::Result<()> {
        let dir = self.hooks_dir.join(hook_type);
        if !dir.exists() {
            return Ok(());
        }

        let mut entries = tokio::fs::read_dir(dir).await?;
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if self.is_executable(&path) {
                self.run_script(&path, ctx, params).await?;
            }
        }

        Ok(())
    }

    fn is_executable(&self, path: &Path) -> bool {
        let ext = path.extension().and_then(|e| e.to_str());
        #[cfg(windows)]
        {
            matches!(ext, Some("ps1") | Some("bat") | Some("exe"))
        }
        #[cfg(not(windows))]
        {
            matches!(ext, Some("sh") | None)
        }
    }

    async fn run_script(
        &self,
        path: &Path,
        ctx: &HookContext,
        params: &serde_json::Value,
    ) -> anyhow::Result<()> {
        let ctx_json = serde_json::to_string(ctx)?;
        let params_json = serde_json::to_string(params)?;

        let mut cmd = if cfg!(windows) {
            let mut c = Command::new("powershell");
            c.arg("-File").arg(path);
            c
        } else {
            Command::new(path)
        };

        let output = cmd
            .env("AGENT_CONTEXT", ctx_json)
            .env("TOOL_PARAMS", params_json)
            .output()
            .await?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!(
                "Hook script failed: {}. Error: {}",
                path.display(),
                stderr
            ));
        }

        Ok(())
    }
}

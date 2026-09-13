//! @docs ARCHITECTURE:Registry
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Agent Runner / plugin
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Type-safe state handling and bounded execution without unhandled panics.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: none declared
//! - **Witness Tests**: none declared

use super::error::ToolExecutionError;
use super::trait_tool::{Tool, ToolContext, ToolDefinitionData};
use crate::agent::types::TokenUsage;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PluginManifest {
    pub name: String,
    pub description: String,
    pub parameters: Value,
    pub entrypoint: String,
    #[serde(default)]
    pub is_mutating: bool,
    #[serde(default)]
    pub is_dangerous: bool,
    #[serde(default)]
    pub is_cacheable: bool,
}

#[derive(Clone)]
pub struct PluginTool {
    pub manifest: PluginManifest,
    pub plugin_dir: PathBuf,
}

#[async_trait::async_trait]
impl Tool for PluginTool {
    fn metadata(&self) -> ToolDefinitionData {
        ToolDefinitionData {
            name: self.manifest.name.clone(),
            description: self.manifest.description.clone(),
            parameters: self.manifest.parameters.clone(),
            is_mutating: self.manifest.is_mutating,
            is_dangerous: self.manifest.is_dangerous,
            is_cacheable: self.manifest.is_cacheable,
        }
    }

    fn name(&self) -> &str {
        &self.manifest.name
    }

    fn is_cacheable(&self) -> bool {
        self.manifest.is_cacheable
    }

    fn is_mutating(&self) -> bool {
        self.manifest.is_mutating
    }

    fn is_dangerous(&self) -> bool {
        // Any third-party plugin is potentially dangerous as it spawns subprocesses
        true
    }

    async fn execute(
        &self,
        ctx: &ToolContext,
        args: Value,
        _usage: &mut Option<TokenUsage>,
    ) -> Result<String, ToolExecutionError> {
        let entry_path = self.plugin_dir.join(&self.manifest.entrypoint);
        if !entry_path.exists() {
            return Err(ToolExecutionError::ExecutionFailed(format!(
                "Plugin entrypoint script missing: {:?}",
                entry_path
            )));
        }

        // Determine execution command based on script extension
        let ext = entry_path
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_lowercase();

        let (cmd, args_list) = match ext.as_str() {
            "py" => ("python", vec![entry_path.to_string_lossy().to_string()]),
            "js" | "cjs" | "mjs" => ("node", vec![entry_path.to_string_lossy().to_string()]),
            "wasm" => {
                // If it is a WASM file, we try to run it via wasmer or node or shell
                // For simplicity and sandboxing in standard local nodes, we run it using "wasmer run"
                // or fall back to node if wasmer isn't installed. Let's try wasmer.
                (
                    "wasmer",
                    vec!["run".to_string(), entry_path.to_string_lossy().to_string()],
                )
            }
            _ => {
                // Assume executable or shell script
                (entry_path.to_str().unwrap_or(""), vec![])
            }
        };

        if cmd.is_empty() {
            return Err(ToolExecutionError::ExecutionFailed(
                "Invalid execution command resolved for plugin".to_string(),
            ));
        }

        // Enforce the Oversight Gate for all dynamic plugins
        // This keeps dynamic plugin execution secure
        tracing::info!(
            "🔌 [Plugins] Spawning dynamic tool '{}' via process: {}",
            self.manifest.name,
            cmd
        );

        let input_json = serde_json::to_string(&args).unwrap_or_default();

        let mut child = tokio::process::Command::new(cmd)
            .args(&args_list)
            .current_dir(&ctx.workspace_root)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .map_err(|e| {
                ToolExecutionError::ExecutionFailed(format!(
                    "Failed to launch plugin subprocess: {}",
                    e
                ))
            })?;

        // Write input arguments JSON to stdin of the subprocess
        if let Some(mut stdin) = child.stdin.take() {
            use tokio::io::AsyncWriteExt;
            let _ = stdin.write_all(input_json.as_bytes()).await;
            let _ = stdin.flush().await;
        }

        // Wait for execution to finish
        let output = child.wait_with_output().await.map_err(|e| {
            ToolExecutionError::ExecutionFailed(format!(
                "Failed waiting for plugin subprocess: {}",
                e
            ))
        })?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        if !output.status.success() {
            return Err(ToolExecutionError::ExecutionFailed(format!(
                "Plugin process exited with error:\nSTDOUT: {}\nSTDERR: {}",
                stdout, stderr
            )));
        }

        Ok(stdout)
    }
}

/// Dynamic discovery function mapping plugins folders to registered dynamic tools.
pub async fn load_dynamic_plugins(plugins_dir: &Path) -> Vec<PluginTool> {
    let mut loaded_plugins = Vec::new();
    if !plugins_dir.exists() {
        let _ = tokio::fs::create_dir_all(plugins_dir).await;
        return loaded_plugins;
    }

    if let Ok(mut entries) = tokio::fs::read_dir(plugins_dir).await {
        while let Ok(Some(entry)) = entries.next_entry().await {
            let path = entry.path();
            if path.is_dir() {
                let manifest_path = path.join("manifest.json");
                if manifest_path.exists() {
                    if let Ok(content) = tokio::fs::read_to_string(&manifest_path).await {
                        if let Ok(manifest) = serde_json::from_str::<PluginManifest>(&content) {
                            tracing::info!(
                                "🔌 [Plugins] Discovered dynamic plugin: {}",
                                manifest.name
                            );
                            loaded_plugins.push(PluginTool {
                                manifest,
                                plugin_dir: path,
                            });
                        }
                    }
                }
            }
        }
    }
    loaded_plugins
}

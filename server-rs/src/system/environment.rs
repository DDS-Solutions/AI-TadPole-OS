//! @docs ARCHITECTURE:Infrastructure
//!
//! ### AI Assist Note
//! **Core technical resource for the Tadpole OS Sovereign infrastructure.**
//! This module implements high-fidelity logic for the Sovereign Reality layer.
//!
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: Runtime logic error, state desynchronization, or resource exhaustion.
//! - **Telemetry Link**: Search `[environment]` in tracing logs.

use std::env;
use std::path::Path;
use std::process::Command;

/// Probe the host system to identify which runtime environments are active/available.
pub fn detect_environments(_workspace_path: &str) -> Vec<String> {
    let mut envs = Vec::new();

    // 1. VS Code check
    if is_vscode_available() {
        envs.push("vs_code".to_string());
    }

    // 2. K8s check
    if is_kubernetes_node() {
        envs.push("k8s_node".to_string());
    }

    // 3. Headless check
    if is_headless() {
        envs.push("headless".to_string());
    }

    // 4. Docker check
    if is_docker_available() {
        envs.push("docker".to_string());
    }

    // 5. WASM Sandbox check (detects if wasm-codec exists)
    if is_wasm_sandbox_available() {
        envs.push("wasm_sandbox".to_string());
    }

    // 6. Tauri check
    if is_tauri_shell() {
        envs.push("tauri_shell".to_string());
    }

    // 7. Jupyter Lab check
    if is_jupyter_available() {
        envs.push("jupyter_lab".to_string());
    }

    envs
}

fn is_vscode_available() -> bool {
    if env::var("TERM_PROGRAM")
        .map(|v| v == "vscode")
        .unwrap_or(false)
    {
        return true;
    }
    which("code") || which("code.cmd")
}

fn is_kubernetes_node() -> bool {
    if env::var("KUBERNETES_SERVICE_HOST").is_ok() {
        return true;
    }
    Path::new("/var/run/secrets/kubernetes.io").exists()
}

fn is_headless() -> bool {
    #[cfg(unix)]
    {
        env::var("DISPLAY").is_err()
    }
    #[cfg(not(unix))]
    {
        env::var("CI").is_ok()
    }
}

fn is_docker_available() -> bool {
    if Path::new("/var/run/docker.sock").exists() {
        return true;
    }
    which("docker")
}

fn is_wasm_sandbox_available() -> bool {
    Path::new("wasm-codec").exists() || Path::new("../wasm-codec").exists()
}

fn is_tauri_shell() -> bool {
    env::var("TAURI_ENV").is_ok() || env::var("TAURI_PLATFORM").is_ok()
}

fn is_jupyter_available() -> bool {
    which("jupyter")
}

fn which(cmd: &str) -> bool {
    #[cfg(windows)]
    {
        let cmd_to_run = if cmd.contains('.') {
            cmd.to_string()
        } else {
            format!("{}.exe", cmd)
        };
        Command::new("where")
            .arg(&cmd_to_run)
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }
    #[cfg(not(windows))]
    {
        Command::new("which")
            .arg(cmd)
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_environments_via_env_vars() {
        std::env::set_var("TAURI_ENV", "true");
        std::env::set_var("KUBERNETES_SERVICE_HOST", "127.0.0.1");

        let envs = detect_environments("dummy_path");
        assert!(envs.contains(&"tauri_shell".to_string()));
        assert!(envs.contains(&"k8s_node".to_string()));

        std::env::remove_var("TAURI_ENV");
        std::env::remove_var("KUBERNETES_SERVICE_HOST");
    }
}

// Metadata: [environment]

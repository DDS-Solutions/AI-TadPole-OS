//! @docs ARCHITECTURE:Infrastructure
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / environment
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Type-safe state handling and bounded execution without unhandled panics.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: none declared
//! - **Witness Tests**: none declared

use std::env;
use std::path::Path;
use std::process::Command;

/// Probe the host system to identify which runtime environments are active/available.
pub fn detect_environments(workspace_path: &str) -> Vec<String> {
    detect_environments_with_lookup(workspace_path, |k| env::var(k).ok())
}

/// Probe host environments with a customizable environment variable lookup (supports pure unit testing).
pub fn detect_environments_with_lookup<F>(workspace_path: &str, get_env: F) -> Vec<String>
where
    F: Fn(&str) -> Option<String>,
{
    let mut envs = Vec::new();

    // 1. VS Code check
    if is_vscode_available(&get_env) {
        envs.push("vs_code".to_string());
    }

    // 2. K8s check
    if is_kubernetes_node(&get_env) {
        envs.push("k8s_node".to_string());
    }

    // 3. Headless check
    if is_headless(&get_env) {
        envs.push("headless".to_string());
    }

    // 4. Docker check
    if is_docker_available() {
        envs.push("docker".to_string());
    }

    // 5. WASM Sandbox check (detects if wasm-codec exists in workspace or CWD)
    if is_wasm_sandbox_available(workspace_path) {
        envs.push("wasm_sandbox".to_string());
    }

    // 6. Tauri check
    if is_tauri_shell(&get_env) {
        envs.push("tauri_shell".to_string());
    }

    // 7. Jupyter Lab check
    if is_jupyter_available() {
        envs.push("jupyter_lab".to_string());
    }

    envs
}

fn is_vscode_available<F: Fn(&str) -> Option<String>>(get_env: &F) -> bool {
    if get_env("TERM_PROGRAM").as_deref() == Some("vscode") {
        return true;
    }
    which("code") || which("code.cmd")
}

fn is_kubernetes_node<F: Fn(&str) -> Option<String>>(get_env: &F) -> bool {
    if get_env("KUBERNETES_SERVICE_HOST").is_some() {
        return true;
    }
    Path::new("/var/run/secrets/kubernetes.io").exists()
}

fn is_headless<F: Fn(&str) -> Option<String>>(get_env: &F) -> bool {
    #[cfg(unix)]
    {
        get_env("DISPLAY").is_none()
    }
    #[cfg(not(unix))]
    {
        get_env("CI").is_some()
    }
}

fn is_docker_available() -> bool {
    if Path::new("/var/run/docker.sock").exists() {
        return true;
    }
    which("docker")
}

fn is_wasm_sandbox_available(workspace_path: &str) -> bool {
    let ws = Path::new(workspace_path);
    ws.join("wasm-codec").exists()
        || ws.join("../wasm-codec").exists()
        || Path::new("wasm-codec").exists()
        || Path::new("../wasm-codec").exists()
}

fn is_tauri_shell<F: Fn(&str) -> Option<String>>(get_env: &F) -> bool {
    get_env("TAURI_ENV").is_some() || get_env("TAURI_PLATFORM").is_some()
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
        let envs = detect_environments_with_lookup("dummy_path", |k| match k {
            "TAURI_ENV" => Some("true".to_string()),
            "KUBERNETES_SERVICE_HOST" => Some("127.0.0.1".to_string()),
            _ => None,
        });

        assert!(envs.contains(&"tauri_shell".to_string()));
        assert!(envs.contains(&"k8s_node".to_string()));
    }
}

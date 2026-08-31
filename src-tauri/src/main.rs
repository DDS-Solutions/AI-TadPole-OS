//! @docs ARCHITECTURE:Core
//!
//! ### AI Context Alignment
//! - **Subsystem**: System Core / DesktopShell
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Deterministic internal state integrity and strict interface contract compliance.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: none declared
//! - **Witness Tests**: none declared

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

/// Tauri managed state holding the neural token for IPC delivery to the webview.
struct NeuralTokenState(String);

/// IPC command: delivers the neural token to the bundled UI.
/// Security: Only callable from the local webview (same-origin, loopback only).
#[tauri::command]
fn get_neural_token(state: tauri::State<'_, NeuralTokenState>) -> String {
    state.0.clone()
}

//!   Desktop Shell — Tauri entry point and sidecar orchestration
//!
//! @docs ARCHITECTURE:DesktopShell
//!

use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;
use tauri::Manager;
use tauri_plugin_shell::ShellExt;

/// Appends a timestamped entry to the sidecar runtime log.
///
/// ### 🔍 Trace Scope
/// Logs are written to `sidecar_runtime.log` in the installation directory.
/// Critical for debugging "Binary Not Found" or permission errors on Windows.
fn log_to_file(log_path: &PathBuf, message: &str) {
    if let Some(parent) = log_path.parent() {
        if !parent.exists() {
            let _ = std::fs::create_dir_all(parent);
        }
    }
    if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(log_path) {
        let _ = writeln!(
            file,
            "[{}] {}",
            chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
            message
        );
    }
}

/// Resolves NEURAL_TOKEN from environment, or generates/persists a cryptographically
/// random hex token in `<install_dir>/.neural_token` per desktop installation with restricted permissions.
fn get_or_create_neural_token(install_dir: &PathBuf, log_path: &PathBuf) -> String {
    if let Ok(token) = std::env::var("NEURAL_TOKEN") {
        if !token.trim().is_empty() {
            log_to_file(log_path, "[Auth] Using NEURAL_TOKEN from environment");
            return token;
        }
    }

    let token_file = install_dir.join(".neural_token");
    if token_file.exists() {
        if let Ok(token) = std::fs::read_to_string(&token_file) {
            let trimmed = token.trim().to_string();
            if !trimmed.is_empty() {
                log_to_file(log_path, "[Auth] Loaded persisted token from .neural_token");
                return trimmed;
            }
        }
    }

    // Generate random 256-bit hex token
    use rand::RngCore;
    let mut key = [0u8; 32];
    rand::rng().fill_bytes(&mut key);
    let new_token: String = key.iter().map(|b| format!("{:02x}", b)).collect();

    if let Err(e) = std::fs::write(&token_file, &new_token) {
        log_to_file(
            log_path,
            &format!("[Auth] WARN: Failed to write .neural_token: {:?}", e),
        );
    } else {
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = std::fs::set_permissions(&token_file, std::fs::Permissions::from_mode(0o600));
        }
        log_to_file(
            log_path,
            "[Auth] Generated and persisted new random .neural_token with restricted permissions",
        );
    }

    new_token
}

/// Terminates previous session's sidecar process using PID-scoped tracking.
///
/// ### 🛡️ Ghost Process Mitigation
/// Prevents port conflicts without indiscriminately killing other unrelated
/// instances on multi-tenant or development machines.
fn cleanup_existing_sidecars(install_dir: &PathBuf, log_path: &PathBuf) {
    let pid_file = install_dir.join(".sidecar.pid");
    if pid_file.exists() {
        if let Ok(pid_str) = std::fs::read_to_string(&pid_file) {
            if let Ok(pid) = pid_str.trim().parse::<u32>() {
                log_to_file(
                    log_path,
                    &format!("CLEANUP: Terminating previous sidecar session PID: {}", pid),
                );
                #[cfg(target_os = "windows")]
                {
                    let _ = std::process::Command::new("taskkill")
                        .args(["/F", "/PID", &pid.to_string(), "/T"])
                        .output();
                }
                #[cfg(not(target_os = "windows"))]
                {
                    let _ = std::process::Command::new("kill")
                        .args(["-9", &pid.to_string()])
                        .output();
                }
            }
        }
        let _ = std::fs::remove_file(&pid_file);
    }

    std::thread::sleep(std::time::Duration::from_millis(300));
}

fn main() {
    dotenvy::dotenv().ok();
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![get_neural_token])
        .setup(|app| {
            // 1. Resolve paths relative to the exe — works on any drive letter.
            // This ensures "Portable Mode" compatibility (OS-02).
            let exe_path = std::env::current_exe().unwrap_or_default();
            let install_dir = exe_path
                .parent()
                .unwrap_or(std::path::Path::new("."))
                .to_path_buf();
            let log_path = install_dir.join("sidecar_runtime.log");
            let db_path = install_dir.join("tadpole.db");

            log_to_file(
                &log_path,
                &format!("--- Session Started. Install Dir: {:?} ---", install_dir),
            );
            log_to_file(&log_path, &format!("DB Path: {:?}", db_path));

            let neural_token = get_or_create_neural_token(&install_dir, &log_path);

            // Register token as managed state for IPC delivery to the bundled webview (C-6)
            app.manage(NeuralTokenState(neural_token.clone()));

            // 2. Kill previous session ghost process by PID
            cleanup_existing_sidecars(&install_dir, &log_path);

            // 3. Sidecar configuration
            // NOTE: tauri.conf.json uses "bin/server-rs" for BUILD-TIME discovery.
            // At runtime Tauri strips the path prefix — the binary lands at <install_dir>/server-rs.exe
            // so the runtime ID must be just "server-rs" (no bin/ prefix).
            let shell = app.shell();
            let sidecar_id = "server-rs";
            let db_url = format!("sqlite://{}", db_path.to_string_lossy());

            log_to_file(
                &log_path,
                &format!("Attempting to spawn sidecar: '{}'", sidecar_id),
            );

            let resource_path = app
                .path()
                .resource_dir()
                .unwrap_or_else(|_| install_dir.clone());
            log_to_file(&log_path, &format!("Resource Root: {:?}", resource_path));

            // 4. Spawn loop with retries
            let mut attempts = 0;
            while attempts < 3 {
                let sidecar_cmd = match shell.sidecar(sidecar_id) {
                    Ok(cmd) => {
                        log_to_file(&log_path, "[Sidecar] Binary found. Configuring env vars...");
                        cmd.env("DATABASE_URL", &db_url)
                            .env("RESOURCE_ROOT", &resource_path)
                            .env("NEURAL_TOKEN", &neural_token)
                            .env(
                                "BIND_ADDRESS",
                                std::env::var("BIND_ADDRESS")
                                    .unwrap_or_else(|_| "127.0.0.1".to_string()),
                            )
                            .env(
                                "DISABLE_TELEMETRY",
                                std::env::var("DISABLE_TELEMETRY")
                                    .unwrap_or_else(|_| "true".to_string()),
                            )
                            .env("PORT", "8000")
                            .env("RUST_LOG", "info")
                            .current_dir(install_dir.clone())
                    }
                    Err(e) => {
                        log_to_file(
                            &log_path,
                            &format!(
                                "[Sidecar] FATAL: Binary '{}' not found: {:?}",
                                sidecar_id, e
                            ),
                        );
                        break;
                    }
                };

                match sidecar_cmd.spawn() {
                    Ok((mut rx, child)) => {
                        let pid = child.pid();
                        log_to_file(
                            &log_path,
                            &format!("[Sidecar] ✅ Spawned successfully (PID: {})!", pid),
                        );

                        let pid_file = install_dir.join(".sidecar.pid");
                        let _ = std::fs::write(&pid_file, pid.to_string());

                        let log_path_async = log_path.clone();
                        let pid_file_async = pid_file.clone();
                        tauri::async_runtime::spawn(async move {
                            let _keep_alive = child;
                            while let Some(event) = rx.recv().await {
                                match event {
                                    tauri_plugin_shell::process::CommandEvent::Stdout(ref line) => {
                                        log_to_file(
                                            &log_path_async,
                                            &format!(
                                                "[Sidecar-OUT] {}",
                                                String::from_utf8_lossy(line).trim()
                                            ),
                                        );
                                    }
                                    tauri_plugin_shell::process::CommandEvent::Stderr(ref line) => {
                                        log_to_file(
                                            &log_path_async,
                                            &format!(
                                                "[Sidecar-ERR] {}",
                                                String::from_utf8_lossy(line).trim()
                                            ),
                                        );
                                    }
                                    tauri_plugin_shell::process::CommandEvent::Error(msg) => {
                                        log_to_file(
                                            &log_path_async,
                                            &format!("[Sidecar-ERR] Process error: {}", msg),
                                        );
                                    }
                                    tauri_plugin_shell::process::CommandEvent::Terminated(
                                        payload,
                                    ) => {
                                        log_to_file(
                                            &log_path_async,
                                            &format!(
                                                "[Sidecar] Process TERMINATED. Code: {:?}",
                                                payload.code
                                            ),
                                        );
                                        let _ = std::fs::remove_file(&pid_file_async);
                                    }
                                    _ => {}
                                }
                            }
                        });
                        break;
                    }
                    Err(e) => {
                        attempts += 1;
                        log_to_file(
                            &log_path,
                            &format!("[Sidecar] Spawn failed (attempt {}): {:?}", attempts, e),
                        );
                        if attempts < 3 {
                            std::thread::sleep(std::time::Duration::from_secs(1));
                        } else {
                            log_to_file(
                                &log_path,
                                "FATAL: Sidecar failed after 3 attempts. Running in OFFLINE mode.",
                            );
                        }
                    }
                }
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

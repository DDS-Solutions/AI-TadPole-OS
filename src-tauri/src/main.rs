//! @docs ARCHITECTURE:Core
//! @docs ARCHITECTURE:DesktopShell
//!
//! ### AI Context Alignment
//! - **Subsystem**: System Core / DesktopShell
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: none declared

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::fs::OpenOptions;
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use tauri::Manager;
use tauri_plugin_shell::ShellExt;

mod data_dir;

/// Tauri managed state holding the neural token for IPC delivery to the webview.
struct NeuralTokenState(String);

/// IPC command: delivers the neural token to the bundled UI.
/// Security: Only callable from the local webview (same-origin, loopback only).
#[tauri::command]
fn get_neural_token(state: tauri::State<'_, NeuralTokenState>) -> String {
    state.0.clone()
}

/// Appends a timestamped entry to the sidecar runtime log with bounded rotation (5 MB cap).
///
/// ### 🔍 Trace Scope
/// Logs are written to `sidecar_runtime.log` in the resolved data directory.
/// Critical for debugging "Binary Not Found" or permission errors on Windows.
fn log_to_file(log_path: &PathBuf, message: &str) {
    if let Some(parent) = log_path.parent() {
        if !parent.exists() {
            let _ = std::fs::create_dir_all(parent);
        }
    }

    // Bounded log rotation: if file exceeds 5 MB, rotate to .old
    if let Ok(metadata) = std::fs::metadata(log_path) {
        if metadata.len() > 5 * 1024 * 1024 {
            let old_log = log_path.with_extension("log.old");
            let _ = std::fs::remove_file(&old_log);
            let _ = std::fs::rename(log_path, &old_log);
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

/// Single-instance guard using `.app.pid` lockfile.
/// Returns `true` if this is the primary instance, or `false` if another instance is active.
fn check_single_instance(install_dir: &PathBuf, log_path: &PathBuf) -> bool {
    let pid_file = install_dir.join(".app.pid");
    if pid_file.exists() {
        if let Ok(pid_str) = std::fs::read_to_string(&pid_file) {
            if let Ok(pid) = pid_str.trim().parse::<u32>() {
                if pid != std::process::id() {
                    #[cfg(target_os = "windows")]
                    let is_running = {
                        std::process::Command::new("tasklist")
                            .args(["/FI", &format!("PID eq {}", pid), "/FO", "CSV", "/NH"])
                            .output()
                            .map(|out| {
                                let stdout = String::from_utf8_lossy(&out.stdout).to_lowercase();
                                stdout.contains("tadpole") || stdout.contains("ai-tadpole")
                            })
                            .unwrap_or(false)
                    };
                    #[cfg(not(target_os = "windows"))]
                    let is_running = {
                        std::process::Command::new("kill")
                            .args(["-0", &pid.to_string()])
                            .output()
                            .map(|out| out.status.success())
                            .unwrap_or(false)
                    };

                    if is_running {
                        log_to_file(
                            log_path,
                            &format!(
                                "[App] Another instance of AI-Tadpole-OS is already running (PID: {}). Exiting duplicate process.",
                                pid
                            ),
                        );
                        return false;
                    }
                }
            }
        }
    }

    // Persist current process PID
    let _ = std::fs::write(&pid_file, std::process::id().to_string());
    true
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
/// Attempts graceful termination first to allow clean SQLite WAL checkpoints,
/// falling back to forceful termination after a timeout.
fn cleanup_existing_sidecars(install_dir: &PathBuf, log_path: &PathBuf) {
    let pid_file = install_dir.join(".sidecar.pid");
    if pid_file.exists() {
        if let Ok(pid_str) = std::fs::read_to_string(&pid_file) {
            if let Ok(pid) = pid_str.trim().parse::<u32>() {
                #[cfg(target_os = "windows")]
                {
                    let is_sidecar = std::process::Command::new("tasklist")
                        .args(["/FI", &format!("PID eq {}", pid), "/FO", "CSV", "/NH"])
                        .output()
                        .map(|out| {
                            let stdout = String::from_utf8_lossy(&out.stdout).to_lowercase();
                            stdout.contains("server-rs") || stdout.contains("tadpole")
                        })
                        .unwrap_or(false);

                    if is_sidecar {
                        log_to_file(
                            log_path,
                            &format!(
                                "CLEANUP: Requesting graceful termination of sidecar session PID: {}",
                                pid
                            ),
                        );
                        // 1. Graceful termination request
                        let _ = std::process::Command::new("taskkill")
                            .args(["/PID", &pid.to_string(), "/T"])
                            .output();

                        // 2. Poll up to 1.5s for process to exit cleanly
                        let mut exited = false;
                        for _ in 0..15 {
                            std::thread::sleep(Duration::from_millis(100));
                            let still_running = std::process::Command::new("tasklist")
                                .args(["/FI", &format!("PID eq {}", pid), "/FO", "CSV", "/NH"])
                                .output()
                                .map(|out| {
                                    let stdout =
                                        String::from_utf8_lossy(&out.stdout).to_lowercase();
                                    stdout.contains("server-rs") || stdout.contains("tadpole")
                                })
                                .unwrap_or(false);
                            if !still_running {
                                exited = true;
                                break;
                            }
                        }

                        // 3. Fallback to forceful termination if process hung
                        if !exited {
                            log_to_file(
                                log_path,
                                &format!(
                                    "CLEANUP: Sidecar PID: {} did not exit within timeout, force killing...",
                                    pid
                                ),
                            );
                            let _ = std::process::Command::new("taskkill")
                                .args(["/F", "/PID", &pid.to_string(), "/T"])
                                .output();
                        } else {
                            log_to_file(
                                log_path,
                                &format!("CLEANUP: Sidecar PID: {} exited cleanly.", pid),
                            );
                        }
                    } else {
                        log_to_file(
                            log_path,
                            &format!(
                                "CLEANUP: Stale PID {} does not match sidecar image. Skipping termination.",
                                pid
                            ),
                        );
                    }
                }
                #[cfg(not(target_os = "windows"))]
                {
                    let is_sidecar = std::process::Command::new("ps")
                        .args(["-p", &pid.to_string(), "-o", "comm="])
                        .output()
                        .map(|out| {
                            let stdout = String::from_utf8_lossy(&out.stdout).to_lowercase();
                            stdout.contains("server-rs") || stdout.contains("tadpole")
                        })
                        .unwrap_or(false);

                    if is_sidecar {
                        log_to_file(
                            log_path,
                            &format!(
                                "CLEANUP: Requesting graceful SIGTERM for sidecar session PID: {}",
                                pid
                            ),
                        );
                        let _ = std::process::Command::new("kill")
                            .args(["-15", &pid.to_string()])
                            .output();

                        let mut exited = false;
                        for _ in 0..15 {
                            std::thread::sleep(Duration::from_millis(100));
                            let still_running = std::process::Command::new("kill")
                                .args(["-0", &pid.to_string()])
                                .output()
                                .map(|out| out.status.success())
                                .unwrap_or(false);
                            if !still_running {
                                exited = true;
                                break;
                            }
                        }

                        if !exited {
                            log_to_file(
                                log_path,
                                &format!(
                                    "CLEANUP: Sidecar PID: {} did not exit within timeout, sending SIGKILL...",
                                    pid
                                ),
                            );
                            let _ = std::process::Command::new("kill")
                                .args(["-9", &pid.to_string()])
                                .output();
                        } else {
                            log_to_file(
                                log_path,
                                &format!("CLEANUP: Sidecar PID: {} exited cleanly.", pid),
                            );
                        }
                    } else {
                        log_to_file(
                            log_path,
                            &format!(
                                "CLEANUP: Stale PID {} does not match sidecar image. Skipping termination.",
                                pid
                            ),
                        );
                    }
                }
            }
        }
        let _ = std::fs::remove_file(&pid_file);
    }

    std::thread::sleep(Duration::from_millis(200));
}

/// Probes the local sidecar engine health endpoint via standard TCP loopback.
fn probe_engine_health(port: u16) -> bool {
    let addr: SocketAddr = match format!("127.0.0.1:{}", port).parse() {
        Ok(a) => a,
        Err(_) => return false,
    };
    if let Ok(mut stream) = TcpStream::connect_timeout(&addr, Duration::from_millis(400)) {
        let _ = stream.set_read_timeout(Some(Duration::from_millis(400)));
        let _ = stream.set_write_timeout(Some(Duration::from_millis(400)));
        let req = format!(
            "GET /v1/engine/health HTTP/1.1\r\nHost: 127.0.0.1:{}\r\nConnection: close\r\n\r\n",
            port
        );
        if stream.write_all(req.as_bytes()).is_ok() {
            let mut buf = [0u8; 512];
            if let Ok(n) = stream.read(&mut buf) {
                let resp = String::from_utf8_lossy(&buf[..n]);
                return resp.contains("200 OK") && resp.contains("tadpole_");
            }
        }
    }
    false
}

/// Resolves the base data directory for writable assets (database, logs, locks, tokens).
/// On macOS, defaults to ~/Library/Application Support/com.tadpole.os to prevent permission failures
/// within read-only .app bundles. Directory failures stop startup before sidecar launch.
fn resolve_data_dir(install_dir: &std::path::Path) -> std::io::Result<PathBuf> {
    data_dir::prepare_data_dir(
        install_dir,
        std::env::var_os("TADPOLE_DATA_DIR"),
        cfg!(target_os = "macos"),
        std::env::var_os("HOME"),
    )
}

fn main() {
    // 1. Resolve paths relative to the executable — works on any drive letter.
    // This ensures "Portable Mode" compatibility (OS-02).
    let exe_path = std::env::current_exe().unwrap_or_default();
    let install_dir = exe_path
        .parent()
        .unwrap_or(std::path::Path::new("."))
        .to_path_buf();

    let data_dir = match resolve_data_dir(&install_dir) {
        Ok(path) => path,
        Err(error) => {
            eprintln!("[Desktop] Cannot initialize data directory: {error}");
            std::process::exit(1);
        }
    };

    // Prefer loading .env from data_dir (custom configuration), then installation directory
    if dotenvy::from_path(data_dir.join(".env")).is_err() {
        if dotenvy::from_path(install_dir.join(".env")).is_err() {
            let _ = dotenvy::dotenv();
        }
    }

    let log_path = data_dir.join("sidecar_runtime.log");
    let state_db_path = data_dir.join("state").join("tadpole.db");
    let db_path = if state_db_path.exists() {
        state_db_path
    } else {
        data_dir.join("tadpole.db")
    };

    // Single-instance protection guard
    if !check_single_instance(&data_dir, &log_path) {
        std::process::exit(0);
    }

    let install_dir_setup = install_dir.clone();
    let data_dir_setup = data_dir.clone();
    let log_path_setup = log_path.clone();

    let app = tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![get_neural_token])
        .setup(move |app| {
            let install_dir = install_dir_setup;
            let data_dir = data_dir_setup;
            let log_path = log_path_setup;

            log_to_file(
                &log_path,
                &format!(
                    "--- Session Started. Install Dir: {:?}, Data Dir: {:?} ---",
                    install_dir, data_dir
                ),
            );
            log_to_file(&log_path, &format!("DB Path: {:?}", db_path));

            let neural_token = get_or_create_neural_token(&data_dir, &log_path);

            // Register token as managed state for IPC delivery to the bundled webview (C-6)
            app.manage(NeuralTokenState(neural_token.clone()));

            // 2. Kill previous session ghost process by PID
            cleanup_existing_sidecars(&data_dir, &log_path);

            // 3. Sidecar configuration
            // NOTE: tauri.conf.json uses "bin/server-rs" for BUILD-TIME discovery.
            // At runtime Tauri strips the path prefix — the binary lands at <install_dir>/server-rs.exe
            // so the runtime ID must be just "server-rs" (no bin/ prefix).
            let shell = app.shell();
            let sidecar_id = "server-rs";

            // Windows path normalization: SQLite connection URLs require forward slashes
            let db_url = format!("sqlite://{}", db_path.to_string_lossy().replace('\\', "/"));

            log_to_file(
                &log_path,
                &format!("Attempting to spawn sidecar: '{}'", sidecar_id),
            );

            let resource_path = app
                .path()
                .resource_dir()
                .unwrap_or_else(|_| install_dir.clone());
            log_to_file(&log_path, &format!("Resource Root: {:?}", resource_path));

            let port_str = std::env::var("PORT").unwrap_or_else(|_| "8000".to_string());
            let port_val: u16 = port_str.parse().unwrap_or(8000);
            let rust_log_val = std::env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string());

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
                            .env("PORT", &port_str)
                            .env("RUST_LOG", &rust_log_val)
                            .env("TADPOLE_DATA_DIR", &data_dir)
                            .current_dir(data_dir.clone())
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

                        let pid_file = data_dir.join(".sidecar.pid");
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

                        // 5. Post-spawn engine health verification probe
                        let log_path_probe = log_path.clone();
                        std::thread::spawn(move || {
                            std::thread::sleep(Duration::from_millis(500));
                            let mut verified = false;
                            for attempt in 1..=10 {
                                std::thread::sleep(Duration::from_millis(500));
                                if probe_engine_health(port_val) {
                                    log_to_file(
                                        &log_path_probe,
                                        &format!(
                                            "[Sidecar] ✅ Engine health check OK on port {} (attempt {})",
                                            port_val, attempt
                                        ),
                                    );
                                    verified = true;
                                    break;
                                }
                            }
                            if !verified {
                                log_to_file(
                                    &log_path_probe,
                                    &format!(
                                        "[Sidecar] ⚠️ Engine health check did not receive tadpole_ready on port {} within 5.5s",
                                        port_val
                                    ),
                                );
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
                            std::thread::sleep(Duration::from_secs(1));
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
        .build(tauri::generate_context!())
        .expect("error while building tauri application");

    let data_dir_exit = data_dir.clone();
    let log_path_exit = log_path.clone();
    let cleaned_up = Arc::new(AtomicBool::new(false));

    app.run(move |_app_handle, event| match event {
        tauri::RunEvent::ExitRequested { .. } | tauri::RunEvent::Exit => {
            if !cleaned_up.swap(true, Ordering::SeqCst) {
                log_to_file(
                    &log_path_exit,
                    "[App] Exit requested. Cleaning up sidecar processes and locks...",
                );
                cleanup_existing_sidecars(&data_dir_exit, &log_path_exit);
                let _ = std::fs::remove_file(data_dir_exit.join(".app.pid"));
            }
        }
        _ => {}
    });
}

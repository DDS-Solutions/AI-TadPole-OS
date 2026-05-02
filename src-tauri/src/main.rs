//! @docs ARCHITECTURE:Core
//! 
//! ### AI Assist Note
//! **Core technical module for the Tadpole OS hardened engine.**
//! This module implements high-fidelity logic for the Sovereign Reality layer.
//! 
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: Runtime logic error, state desynchronization, or resource exhaustion.
//! - **Telemetry Link**: Search `[main.rs]` in tracing logs.

// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

//!   Desktop Shell — Tauri entry point and sidecar orchestration
//!
//! @docs ARCHITECTURE:DesktopShell
//!
//! ### AI Assist Note
//! Uses exe-relative path discovery for maximum portability across drive letters.
//! Logs write to the installation directory to avoid AppData path resolution issues.

use tauri::Manager;
use tauri_plugin_shell::ShellExt;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;

/// Appends a timestamped entry to the sidecar runtime log.
/// 
/// ### 🔍 Trace Scope
/// Logs are written to `sidecar_runtime.log` in the installation directory. 
/// Critical for debugging "Binary Not Found" or permission errors on Windows.
fn log_to_file(log_path: &PathBuf, message: &str) {
    if let Some(parent) = log_path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(log_path) {
        let _ = writeln!(file, "[{}] {}", chrono::Local::now().format("%Y-%m-%d %H:%M:%S"), message);
    }
}

/// Terminates orphaned engine processes before spawning new ones.
/// 
/// ### 🛡️ Ghost Process Mitigation
/// Prevents port conflicts and "Zombie Agent" loops by force-killing 
/// existing `server-rs` instances across the host process tree.
fn cleanup_existing_sidecars(log_path: &PathBuf) {
    log_to_file(log_path, "CLEANUP: Attempting to terminate any orphaned 'server-rs' processes...");
    
    #[cfg(target_os = "windows")]
    {
        let _ = std::process::Command::new("taskkill")
            .args(["/F", "/IM", "server-rs.exe", "/T"])
            .output();
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = std::process::Command::new("pkill")
            .args(["-f", "server-rs"])
            .output();
    }
    
    std::thread::sleep(std::time::Duration::from_millis(500));
}

fn main() {
    dotenvy::dotenv().ok();
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            // 1. Resolve paths relative to the exe — works on any drive letter.
            // This ensures "Portable Mode" compatibility (OS-02).
            let exe_path = std::env::current_exe().unwrap_or_default();
            let install_dir = exe_path.parent().unwrap_or(std::path::Path::new(".")).to_path_buf();
            let log_path = install_dir.join("sidecar_runtime.log");
            let db_path = install_dir.join("tadpole.db");

            log_to_file(&log_path, &format!("--- Session Started. Install Dir: {:?} ---", install_dir));
            log_to_file(&log_path, &format!("DB Path: {:?}", db_path));

            // 2. Kill ghost processes
            cleanup_existing_sidecars(&log_path);

            // 3. Sidecar configuration
            // NOTE: tauri.conf.json uses "bin/server-rs" for BUILD-TIME discovery.
            // At runtime Tauri strips the path prefix — the binary lands at <install_dir>/server-rs.exe
            // so the runtime ID must be just "server-rs" (no bin/ prefix).
            let shell = app.shell();
            let sidecar_id = "server-rs";
            let db_url = format!("sqlite://{}", db_path.to_string_lossy());

            log_to_file(&log_path, &format!("Attempting to spawn sidecar: '{}'", sidecar_id));

            let resource_path = app.path().resource_dir().unwrap_or_else(|_| install_dir.clone());
            log_to_file(&log_path, &format!("Resource Root: {:?}", resource_path));

            // 4. Spawn loop with retries
            let mut attempts = 0;
            while attempts < 3 {
                let sidecar_cmd = match shell.sidecar(sidecar_id) {
                    Ok(cmd) => {
                        log_to_file(&log_path, "[Sidecar] Binary found. Configuring env vars...");
                        cmd.env("DATABASE_URL", &db_url)
                           .env("RESOURCE_ROOT", &resource_path)
                           .env("NEURAL_TOKEN", std::env::var("NEURAL_TOKEN").unwrap_or_else(|_| "tadpole-os-sidecar-default-2026".to_string()))
                           .env("BIND_ADDRESS", std::env::var("BIND_ADDRESS").unwrap_or_else(|_| "127.0.0.1".to_string()))
                           .env("DISABLE_TELEMETRY", std::env::var("DISABLE_TELEMETRY").unwrap_or_else(|_| "true".to_string()))
                           .env("PORT", "8000")
                           .env("RUST_LOG", "info")
                           .current_dir(install_dir.clone())
                    },
                    Err(e) => {
                        log_to_file(&log_path, &format!("[Sidecar] FATAL: Binary '{}' not found: {:?}", sidecar_id, e));
                        break;
                    }
                };

                match sidecar_cmd.spawn() {
                    Ok((mut rx, child)) => {
                        log_to_file(&log_path, "[Sidecar] ✅ Spawned successfully!");
                        let log_path_async = log_path.clone();
                        tauri::async_runtime::spawn(async move {
                            let _keep_alive = child;
                            while let Some(event) = rx.recv().await {
                                match event {
                                    tauri_plugin_shell::process::CommandEvent::Stdout(ref line) => {
                                        log_to_file(&log_path_async, &format!("[Sidecar-OUT] {}", String::from_utf8_lossy(line).trim()));
                                    }
                                    tauri_plugin_shell::process::CommandEvent::Stderr(ref line) => {
                                        log_to_file(&log_path_async, &format!("[Sidecar-ERR] {}", String::from_utf8_lossy(line).trim()));
                                    }
                                    tauri_plugin_shell::process::CommandEvent::Error(msg) => {
                                        log_to_file(&log_path_async, &format!("[Sidecar-ERR] Process error: {}", msg));
                                    }
                                    tauri_plugin_shell::process::CommandEvent::Terminated(payload) => {
                                        log_to_file(&log_path_async, &format!("[Sidecar] Process TERMINATED. Code: {:?}", payload.code));
                                    }
                                    _ => {}
                                }
                            }
                        });
                        break;
                    },
                    Err(e) => {
                        attempts += 1;
                        log_to_file(&log_path, &format!("[Sidecar] Spawn failed (attempt {}): {:?}", attempts, e));
                        if attempts < 3 {
                            std::thread::sleep(std::time::Duration::from_secs(1));
                        } else {
                            log_to_file(&log_path, "FATAL: Sidecar failed after 3 attempts. Running in OFFLINE mode.");
                        }
                    }
                }
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

// Metadata: [main]

// Metadata: [main]

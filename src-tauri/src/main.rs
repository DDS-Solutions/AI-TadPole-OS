// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::Manager;
use tauri_plugin_shell::ShellExt;
use std::env;
use std::fs::OpenOptions;
use std::io::Write;

fn log_to_file(project_root: &std::path::Path, message: &str) {
    let log_path = project_root.join("sidecar_runtime.log");
    if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(log_path) {
        let _ = writeln!(file, "[{}] {}", chrono::Local::now().format("%Y-%m-%d %H:%M:%S"), message);
    }
}

fn cleanup_existing_sidecars(project_root: &std::path::Path) {
    log_to_file(project_root, "CLEANUP: Attempting to terminate any orphaned 'server-rs' processes...");
    // On Windows, taskkill /F /IM server-rs.exe /T
    let _ = std::process::Command::new("taskkill")
        .args(["/F", "/IM", "server-rs.exe", "/T"])
        .output();
    
    // Also try without .exe just in case
    let _ = std::process::Command::new("taskkill")
        .args(["/F", "/IM", "server-rs", "/T"])
        .output();
    
    // Give OS a moment to release the port
    std::thread::sleep(std::time::Duration::from_millis(500));
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            // 1. Initial Path Discovery (Conservative)
            let exe_path = std::env::current_exe().unwrap_or_default();
            let mut project_root = exe_path.parent().unwrap_or(&exe_path).to_path_buf();
            
            // 2. Walk up to find the root containing the database or .env
            while project_root.parent().is_some() {
                if project_root.join("tadpole.db").exists() || project_root.join(".env").exists() {
                    break;
                }
                project_root = project_root.parent().unwrap().to_path_buf();
            }

            log_to_file(&project_root, &format!("--- Session Started. Project Root: {:?} ---", project_root));

            // NEW: Cleanup existing sidecars to prevent port collisions (os error 10048)
            cleanup_existing_sidecars(&project_root);

            // 3. sidecar configuration
            let shell = app.shell();
            let sidecar_id = "server-rs";
            
            // Log exactly what we're looking for
            let sidecar_cmd = match shell.sidecar(sidecar_id) {
                Ok(cmd) => {
                    log_to_file(&project_root, &format!("FOUND: Sidecar ID '{}' initialized.", sidecar_id));
                    cmd.env("WORKSPACE_ROOT", project_root.to_string_lossy().to_string())
                       .env("PORT", "8000")
                       .env("RUST_LOG", "info")
                },
                Err(e) => {
                    log_to_file(&project_root, &format!("FATAL: Sidecar ID '{}' not found: {:?}", sidecar_id, e));
                    return Ok(()); // Avoid immediate panic crash, keep log writable
                }
            };

            log_to_file(&project_root, &format!("Attempting to spawn from CWD: {:?}", project_root));

            // RETRY LOGIC: Sometimes the port isn't released immediately by the OS.
            let mut spawn_result = sidecar_cmd.spawn();
            let mut attempts = 0;
            while spawn_result.is_err() && attempts < 3 {
                log_to_file(&project_root, &format!("RETRY: Spawn failed (attempt {}). Socket might still be in TIME_WAIT. Retrying in 1s...", attempts + 1));
                std::thread::sleep(std::time::Duration::from_secs(1));
                spawn_result = sidecar_cmd.spawn();
                attempts += 1;
            }

            match spawn_result {
                Ok((mut rx, _child)) => {
                    let root_for_async = project_root.clone();
                    tauri::async_runtime::spawn(async move {
                        log_to_file(&root_for_async, "SPAWNED: Sidecar process running, piping output...");
                        while let Some(event) = rx.recv().await {
                            match event {
                                tauri_plugin_shell::process::CommandEvent::Stdout(line) => {
                                    let line_str = String::from_utf8_lossy(&line);
                                    log_to_file(&root_for_async, &format!("[STDOUT] {}", line_str.trim()));
                                }
                                tauri_plugin_shell::process::CommandEvent::Stderr(line) => {
                                    let line_str = String::from_utf8_lossy(&line);
                                    log_to_file(&root_for_async, &format!("[STDERR] {}", line_str.trim()));
                                }
                                tauri_plugin_shell::process::CommandEvent::Terminated(chunk) => {
                                    log_to_file(&root_for_async, &format!("[TERMINATED] Code: {:?}, Signal: {:?}", chunk.code, chunk.signal));
                                }
                                _ => {}
                            }
                        }
                    });
                }
                Err(e) => {
                    log_to_file(&project_root, &format!("FATAL: Spawn failure: {:?}. Is the binary in src-tauri/bin/ and does it match the target triple?", e));
                }
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

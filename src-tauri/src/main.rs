// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

//! Desktop Shell — Tauri entry point and sidecar orchestration
//!
//! Manages the lifecycle of the Rust backend sidecar, system tray,
//! and native window management for the Tadpole OS desktop experience.
//!
//! @docs ARCHITECTURE:DesktopShell

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
    
    #[cfg(target_os = "windows")]
    {
        // On Windows, taskkill /F /IM server-rs.exe /T
        let _ = std::process::Command::new("taskkill")
            .args(["/F", "/IM", "server-rs.exe", "/T"])
            .output();
        
        // Also try without .exe just in case
        let _ = std::process::Command::new("taskkill")
            .args(["/F", "/IM", "server-rs", "/T"])
            .output();
    }

    #[cfg(not(target_os = "windows"))]
    {
        // On Linux/macOS, use pkill
        let _ = std::process::Command::new("pkill")
            .args(["-f", "server-rs"])
            .output();
    }
    
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
            
            // 1b. AppImage Awareness: If we are in an AppImage, find the original directory
            if let Ok(appimage_path) = std::env::var("APPIMAGE") {
                let appimage_path = std::path::PathBuf::from(appimage_path);
                if let Some(parent) = appimage_path.parent() {
                    project_root = parent.to_path_buf();
                    log_to_file(&project_root, &format!("--- AppImage detected! Overriding root to: {:?} ---", project_root));
                }
            } else {
                // 2. Walk up to find the root containing the database or .env (Traditional install)
                while project_root.parent().is_some() {
                    if project_root.join("tadpole.db").exists() || project_root.join(".env").exists() {
                        break;
                    }
                    project_root = project_root.parent().unwrap().to_path_buf();
                }
            }

            log_to_file(&project_root, &format!("--- Session Started. Project Root: {:?} ---", project_root));

            // Ensure our persistence directories exist in the project root
            let data_dir = project_root.join("data");
            if !data_dir.exists() {
                let _ = std::fs::create_dir_all(&data_dir);
            }

            // NEW: Cleanup existing sidecars to prevent port collisions (os error 10048)
            cleanup_existing_sidecars(&project_root);

            // 3. sidecar configuration
            let shell = app.shell();
            let sidecar_id = "binaries/server-rs";
            
            log_to_file(&project_root, &format!("Attempting to spawn from CWD: {:?}", project_root));

            // RETRY LOGIC: Sometimes the port isn't released immediately by the OS.
            // We recreate the command in each attempt to avoid E0382 (use of moved value).
            let mut final_spawn_result = None;
            let mut attempts = 0;
            
            while attempts < 3 {
                let sidecar_cmd = match shell.sidecar(sidecar_id) {
                    Ok(cmd) => {
                        log_to_file(&project_root, &format!("FOUND: Sidecar ID '{}' initialized (attempt {}).", sidecar_id, attempts + 1));
                        let data_dir = project_root.join("data");
                        if let Err(e) = std::fs::create_dir_all(&data_dir) {
                            log_to_file(&project_root, &format!("[Sidecar] Warning: Failed to create data dir: {:?}", e));
                        }
                        
                        let db_path = data_dir.join("tadpole.db");
                        let db_url = format!("sqlite:{}", db_path.to_string_lossy());
                        
                        log_to_file(&project_root, &format!("[Sidecar] Injecting DATABASE_URL: {}", db_url));

                        cmd.env("WORKSPACE_ROOT", project_root.to_string_lossy().to_string())
                           .env("DATABASE_URL", db_url)
                           .env("NEURAL_TOKEN", "tadpole-os-sidecar-default-2026")
                           .env("PORT", "8000")
                           .env("RUST_LOG", "info")
                    },
                    Err(e) => {
                        log_to_file(&project_root, &format!("FATAL: Sidecar ID '{}' not found in bundle: {:?}", sidecar_id, e));
                        final_spawn_result = Some(Err(e));
                        break; 
                    }
                };

                match sidecar_cmd.spawn() {
                    Ok(res) => {
                        log_to_file(&project_root, "[Sidecar] Spawned successfully!");
                        final_spawn_result = Some(Ok(res));
                        break;
                    },
                    Err(e) => {
                        attempts += 1;
                        log_to_file(&project_root, &format!("ERROR: Sidecar spawn failed (attempt {}): {:?}", attempts, e));
                        if attempts < 3 {
                            log_to_file(&project_root, "RETRY: Waiting 1s before retry...");
                            std::thread::sleep(std::time::Duration::from_secs(1));
                        } else {
                            final_spawn_result = Some(Err(e));
                        }
                    }
                }
            }

            if let Some(res) = final_spawn_result {
                match res {
                    Ok((mut rx, child)) => {
                        let root_for_async = project_root.clone();
                        tauri::async_runtime::spawn(async move {
                            // Keep the child alive by moving it here. 
                            // We can use it later if we need to kill it explicitly.
                            let _keep_alive = child; 
                            log_to_file(&root_for_async, "[Sidecar] Pipeline opened, monitoring output...");
                            while let Some(event) = rx.recv().await {
                                match event {
                                    tauri_plugin_shell::process::CommandEvent::Stdout(line) => {
                                        let line_str = String::from_utf8_lossy(&line);
                                        log_to_file(&root_for_async, &format!("[Sidecar-STDOUT] {}", line_str.trim()));
                                    }
                                    tauri_plugin_shell::process::CommandEvent::Stderr(line) => {
                                        let line_str = String::from_utf8_lossy(&line);
                                        log_to_file(&root_for_async, &format!("[Sidecar-STDERR] {}", line_str.trim()));
                                    }
                                    tauri_plugin_shell::process::CommandEvent::Terminated(chunk) => {
                                        log_to_file(&root_for_async, &format!("[Sidecar-TERMINATED] Exit Code: {:?}, Signal: {:?}", chunk.code, chunk.signal));
                                    }
                                    _ => {}
                                }
                            }
                        });
                    }
                    Err(e) => {
                        log_to_file(&project_root, &format!("FATAL: Final spawn failure: {:?}. Please check if the binary 'src-tauri/bin/server-rs-x86_64-unknown-linux-gnu' exists in the AppImage bundle.", e));
                    }
                }
            } else {
                log_to_file(&project_root, "FATAL: Sidecar spawn result was None.");
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

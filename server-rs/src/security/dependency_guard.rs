//! @docs ARCHITECTURE:Core
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Security & Governance / dependency_guard
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Type-safe state handling and bounded execution without unhandled panics.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: none declared

pub fn get_skill_requirements(skill: &str) -> (Vec<&'static str>, Vec<&'static str>) {
    match skill.to_lowercase().as_str() {
        "git_push" | "git_pull" | "git_commit" | "git" => (vec!["git"], vec![]),
        "docker_run" | "docker_build" | "docker" => (vec!["docker"], vec![]),
        "synthesize_micro_script" | "script_builder" => (vec!["python", "node"], vec![]),
        "notify_discord" => (vec![], vec!["DISCORD_WEBHOOK_URL"]),
        _ => (vec![], vec![]),
    }
}

/// Returns true if `path` is a regular executable file (verifying execute bits on Unix).
fn is_executable_file(path: &std::path::Path) -> bool {
    let Ok(meta) = std::fs::metadata(path) else {
        return false;
    };
    if !meta.is_file() {
        return false;
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        meta.permissions().mode() & 0o111 != 0
    }
    #[cfg(not(unix))]
    {
        true
    }
}

/// Checks if a system binary is available on `PATH` using pure-Rust path resolution
/// (avoids spawning `where`/`which` subprocesses that can stall in parallel Windows test runs).
pub fn is_binary_available(name: &str) -> bool {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return false;
    }
    let Some(path_var) = std::env::var_os("PATH") else {
        return false;
    };
    #[cfg(windows)]
    const WINDOWS_EXTS: &[&str] = &[".exe", ".cmd", ".bat", ".com"];

    for dir in std::env::split_paths(&path_var) {
        if dir.as_os_str().is_empty() {
            continue;
        }
        let direct = dir.join(trimmed);
        if is_executable_file(&direct) {
            return true;
        }
        #[cfg(windows)]
        {
            let lower = trimmed.to_ascii_lowercase();
            let has_exec_ext = WINDOWS_EXTS.iter().any(|ext| lower.ends_with(ext));
            if !has_exec_ext {
                for ext in WINDOWS_EXTS {
                    if is_executable_file(&dir.join(format!("{trimmed}{ext}"))) {
                        return true;
                    }
                }
            }
        }
    }
    false
}

/// Verifies that all required binaries and env variables are present for the given list of skills.
/// Returns a list of error messages for missing dependencies, or Ok if all are present.
pub fn check_skill_dependencies(skills: &[String]) -> Result<(), Vec<String>> {
    let mut missing = Vec::new();

    for skill in skills {
        let lower_skill = skill.to_ascii_lowercase();
        let (binaries, envs) = get_skill_requirements(skill);

        // Verify binaries
        if !binaries.is_empty() {
            let any_available =
                if lower_skill == "synthesize_micro_script" || lower_skill == "script_builder" {
                    binaries.iter().any(|&bin| is_binary_available(bin))
                } else {
                    binaries.iter().all(|&bin| is_binary_available(bin))
                };

            if !any_available {
                missing.push(format!(
                    "Skill '{}' requires system binary: {:?}",
                    skill, binaries
                ));
            }
        }

        // Verify environment variables
        if !envs.is_empty() {
            let all_envs_present = envs.iter().all(|&env| std::env::var(env).is_ok());

            if !all_envs_present {
                missing.push(format!(
                    "Skill '{}' requires environment variables: {:?}",
                    skill, envs
                ));
            }
        }
    }

    if missing.is_empty() {
        Ok(())
    } else {
        Err(missing)
    }
}

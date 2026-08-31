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
//! - **Witness Tests**: none declared

pub fn get_skill_requirements(skill: &str) -> (Vec<&'static str>, Vec<&'static str>) {
    match skill.to_lowercase().as_str() {
        "git_push" | "git_pull" | "git_commit" | "git" => {
            (vec!["git"], vec!["GITHUB_TOKEN", "GIT_SSH_KEY"])
        }
        "docker_run" | "docker_build" | "docker" => (vec!["docker"], vec![]),
        "synthesize_micro_script" | "script_builder" => (vec!["python", "node"], vec![]),
        "notify_discord" => (vec![], vec!["DISCORD_WEBHOOK_URL"]),
        _ => (vec![], vec![]),
    }
}

/// Checks if a system binary is available on the path.
pub fn is_binary_available(name: &str) -> bool {
    let check_cmd = if cfg!(target_os = "windows") {
        "where"
    } else {
        "which"
    };
    std::process::Command::new(check_cmd)
        .arg(name)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}

/// Verifies that all required binaries and env variables are present for the given list of skills.
/// Returns a list of error messages for missing dependencies, or Ok if all are present.
pub fn check_skill_dependencies(skills: &[String]) -> Result<(), Vec<String>> {
    let mut missing = Vec::new();

    for skill in skills {
        let (binaries, envs) = get_skill_requirements(skill);

        // Verify binaries
        if !binaries.is_empty() {
            let any_available = if skill == "synthesize_micro_script" || skill == "script_builder" {
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
            let any_env = if skill.contains("git") {
                envs.iter().any(|&env| std::env::var(env).is_ok())
            } else {
                envs.iter().all(|&env| std::env::var(env).is_ok())
            };

            if !any_env {
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

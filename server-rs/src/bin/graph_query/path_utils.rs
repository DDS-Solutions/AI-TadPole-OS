//! @docs ARCHITECTURE:CodeBaseIntelligence
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / path_utils
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Type-safe state handling and bounded execution without unhandled panics.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: none declared
//! - **Witness Tests**: none declared

use crate::GraphQueryError;
use std::path::{Path, PathBuf};

pub fn lexical_normalize(path: &Path) -> PathBuf {
    let mut resolved = PathBuf::new();
    for component in path.components() {
        match component {
            std::path::Component::Prefix(p) => {
                resolved.push(p.as_os_str());
            }
            std::path::Component::RootDir => {
                resolved.push(std::path::Component::RootDir.as_os_str());
            }
            std::path::Component::CurDir => {}
            std::path::Component::ParentDir => {
                resolved.pop();
            }
            std::path::Component::Normal(c) => {
                resolved.push(c);
            }
        }
    }
    resolved
}

pub fn normalize_query_path(root: &Path, raw: &str) -> Result<String, GraphQueryError> {
    let normalized_str = raw.replace('\\', "/");
    let input_path = Path::new(&normalized_str);
    let absolute_target = if input_path.is_absolute() {
        input_path.to_path_buf()
    } else {
        root.join(input_path)
    };

    let canonical_root = root.canonicalize().map_err(GraphQueryError::Io)?;

    let resolved_target = lexical_normalize(&absolute_target);

    let canonical_target = match resolved_target.canonicalize() {
        Ok(path) => path,
        Err(_) => resolved_target,
    };

    if !canonical_target.starts_with(&canonical_root) {
        return Err(GraphQueryError::Security(format!(
            "Path traversal detected! Target path '{}' is outside root directory '{}'",
            canonical_target.display(),
            canonical_root.display()
        )));
    }

    let relative = canonical_target
        .strip_prefix(&canonical_root)
        .map_err(|e| GraphQueryError::Security(format!("Failed to strip root prefix: {e}")))?;

    Ok(relative.to_string_lossy().replace('\\', "/"))
}

fn run_git_diff_z(root: &Path, args: &[&str]) -> Result<Vec<String>, GraphQueryError> {
    let out = std::process::Command::new("git")
        .args(args)
        .current_dir(root)
        .output()
        .map_err(GraphQueryError::Io)?;

    if !out.status.success() {
        let stderr = String::from_utf8_lossy(&out.stderr);
        return Err(GraphQueryError::Validation(format!(
            "git command {:?} failed with status {}: {}",
            args,
            out.status,
            stderr.trim()
        )));
    }

    let mut results = Vec::new();
    for part in out.stdout.split(|&b| b == 0) {
        if !part.is_empty() {
            let s = String::from_utf8_lossy(part).trim().replace('\\', "/");
            if !s.is_empty() {
                results.push(s);
            }
        }
    }
    Ok(results)
}

fn run_git_status_z(root: &Path) -> Result<Vec<String>, GraphQueryError> {
    let out = std::process::Command::new("git")
        .args(["status", "--porcelain", "-z"])
        .current_dir(root)
        .output()
        .map_err(GraphQueryError::Io)?;

    if !out.status.success() {
        let stderr = String::from_utf8_lossy(&out.stderr);
        return Err(GraphQueryError::Validation(format!(
            "git status failed with status {}: {}",
            out.status,
            stderr.trim()
        )));
    }

    let mut results = Vec::new();
    for part in out.stdout.split(|&b| b == 0) {
        if part.len() >= 3 {
            let path_bytes = &part[3..];
            let s = String::from_utf8_lossy(path_bytes)
                .trim()
                .replace('\\', "/");
            if !s.is_empty() {
                results.push(s);
            }
        }
    }
    Ok(results)
}

pub fn get_git_modified_files(
    root: &Path,
) -> Result<std::collections::HashSet<String>, GraphQueryError> {
    let mut modified = std::collections::HashSet::new();

    // 1. Unstaged modifications (diff --name-only -z)
    for path in run_git_diff_z(root, &["diff", "--name-only", "-z"])? {
        modified.insert(path);
    }

    // 2. Staged modifications (diff --cached --name-only -z)
    for path in run_git_diff_z(root, &["diff", "--cached", "--name-only", "-z"])? {
        modified.insert(path);
    }

    // 3. Untracked and status-modified files (status --porcelain -z)
    for path in run_git_status_z(root)? {
        modified.insert(path);
    }

    Ok(modified)
}

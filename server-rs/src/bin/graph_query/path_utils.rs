//! @docs ARCHITECTURE:CodeBaseIntelligence
//!
//! ### AI Assist Note
//! **Path Utilities**: Resolves query paths securely and normalizes paths.
//! Prevents traversal attacks.
//!
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: Path traversal attack detected, resolution failures.
//! - **Trace Scope**: `server-rs::bin::graph_query::path_utils`

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

    let canonical_root = root
        .canonicalize()
        .map_err(|e| GraphQueryError::Security(format!("Failed to canonicalize root: {e}")))?;

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

pub fn get_git_modified_files(
    root: &Path,
) -> Result<std::collections::HashSet<String>, GraphQueryError> {
    let mut modified = std::collections::HashSet::new();

    // 1. Run git diff --name-only
    let output1 = std::process::Command::new("git")
        .args(["diff", "--name-only"])
        .current_dir(root)
        .output();

    if let Ok(out) = output1 {
        if out.status.success() {
            let stdout = String::from_utf8_lossy(&out.stdout);
            for line in stdout.lines() {
                let trimmed = line.trim();
                if !trimmed.is_empty() {
                    modified.insert(trimmed.replace('\\', "/"));
                }
            }
        }
    }

    // 2. Run git diff --cached --name-only (staged files)
    let output2 = std::process::Command::new("git")
        .args(["diff", "--cached", "--name-only"])
        .current_dir(root)
        .output();

    if let Ok(out) = output2 {
        if out.status.success() {
            let stdout = String::from_utf8_lossy(&out.stdout);
            for line in stdout.lines() {
                let trimmed = line.trim();
                if !trimmed.is_empty() {
                    modified.insert(trimmed.replace('\\', "/"));
                }
            }
        }
    }

    // 3. Run git status --porcelain (untracked files)
    let output3 = std::process::Command::new("git")
        .args(["status", "--porcelain"])
        .current_dir(root)
        .output();

    if let Ok(out) = output3 {
        if out.status.success() {
            let stdout = String::from_utf8_lossy(&out.stdout);
            for line in stdout.lines() {
                let trimmed = line.trim();
                if let Some(file_part) = trimmed.strip_prefix("??") {
                    let file_part = file_part.trim();
                    if !file_part.is_empty() {
                        modified.insert(file_part.replace('\\', "/"));
                    }
                }
            }
        }
    }

    Ok(modified)
}

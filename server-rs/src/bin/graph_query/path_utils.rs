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
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

const GIT_TIMEOUT: Duration = Duration::from_secs(30);

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
        Err(_) => {
            // Check deepest existing ancestor to guard against symlink traversal attacks (D-17)
            let mut ancestor = resolved_target.as_path();
            while !ancestor.exists() && ancestor.parent().is_some() {
                ancestor = ancestor.parent().unwrap();
            }
            if ancestor.exists() {
                let canonical_ancestor = ancestor.canonicalize().map_err(GraphQueryError::Io)?;
                if !canonical_ancestor.starts_with(&canonical_root) {
                    return Err(GraphQueryError::Security(format!(
                        "Path traversal detected! Target ancestor '{}' escapes root directory '{}'",
                        canonical_ancestor.display(),
                        canonical_root.display()
                    )));
                }
            }
            resolved_target
        }
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

pub fn run_git_cmd(root: &Path, args: &[&str]) -> Result<Vec<u8>, GraphQueryError> {
    let mut child = Command::new("git")
        .args([
            "--no-optional-locks",
            "-c",
            "core.fsmonitor=false",
            "-c",
            "core.hooksPath=",
        ])
        .args(args)
        .current_dir(root)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                GraphQueryError::Validation(format!("git binary not found on PATH: {e}"))
            } else {
                GraphQueryError::Io(e)
            }
        })?;

    let start = Instant::now();
    loop {
        match child.try_wait()? {
            Some(status) => {
                let mut stdout = Vec::new();
                let mut stderr = Vec::new();
                if let Some(mut out) = child.stdout.take() {
                    let _ = out.read_to_end(&mut stdout);
                }
                if let Some(mut err) = child.stderr.take() {
                    let _ = err.read_to_end(&mut stderr);
                }
                if !status.success() {
                    let stderr_msg = String::from_utf8_lossy(&stderr);
                    return Err(GraphQueryError::Validation(format!(
                        "git command {:?} failed with status {}: {}",
                        args,
                        status,
                        stderr_msg.trim()
                    )));
                }
                return Ok(stdout);
            }
            None if start.elapsed() > GIT_TIMEOUT => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(GraphQueryError::Validation(format!(
                    "git command {:?} exceeded timeout of {:?}",
                    args, GIT_TIMEOUT
                )));
            }
            None => {
                std::thread::sleep(Duration::from_millis(25));
            }
        }
    }
}

pub fn get_git_repo_prefix(root: &Path) -> Result<String, GraphQueryError> {
    let bytes = run_git_cmd(root, &["rev-parse", "--show-prefix"])?;
    let prefix = String::from_utf8_lossy(&bytes).trim().replace('\\', "/");
    Ok(prefix)
}

fn run_git_diff_z(root: &Path, args: &[&str]) -> Result<Vec<String>, GraphQueryError> {
    let bytes = run_git_cmd(root, args)?;
    let mut results = Vec::new();
    for part in bytes.split(|&b| b == 0) {
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
    let bytes = run_git_cmd(root, &["status", "--porcelain", "-z"])?;
    let mut results = Vec::new();
    let parts: Vec<&[u8]> = bytes.split(|&b| b == 0).collect();
    let mut i = 0;
    while i < parts.len() {
        let part = parts[i];
        if part.len() >= 3 {
            let status_code = &part[..2];
            let path_bytes = &part[3..];
            let s = String::from_utf8_lossy(path_bytes)
                .trim()
                .replace('\\', "/");
            if !s.is_empty() {
                results.push(s);
            }
            // If rename or copy (R or C), the next entry is the original path
            if status_code.starts_with(b"R") || status_code.starts_with(b"C") {
                i += 1;
            }
        }
        i += 1;
    }
    Ok(results)
}

pub fn get_git_modified_files(
    root: &Path,
) -> Result<std::collections::HashSet<String>, GraphQueryError> {
    let prefix = get_git_repo_prefix(root).unwrap_or_default();
    let mut raw_paths = Vec::new();

    // 1. Unstaged modifications (diff --name-only -z)
    raw_paths.extend(run_git_diff_z(root, &["diff", "--name-only", "-z"])?);

    // 2. Staged modifications (diff --cached --name-only -z)
    raw_paths.extend(run_git_diff_z(
        root,
        &["diff", "--cached", "--name-only", "-z"],
    )?);

    // 3. Untracked and status-modified files (status --porcelain -z)
    raw_paths.extend(run_git_status_z(root)?);

    let mut modified = std::collections::HashSet::new();
    for path in raw_paths {
        if prefix.is_empty() {
            modified.insert(path);
        } else if let Some(stripped) = path.strip_prefix(&prefix) {
            modified.insert(stripped.to_string());
        }
    }

    Ok(modified)
}

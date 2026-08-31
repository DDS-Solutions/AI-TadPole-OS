//! @docs ARCHITECTURE:Security
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Agent Runner / paths
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Type-safe state handling and bounded execution without unhandled panics.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: `[paths]`, `[capability]`
//! - **Witness Tests**: none declared

use lru::LruCache;
use once_cell::sync::Lazy;
use parking_lot::Mutex;
use std::num::NonZeroUsize;
use std::path::Path;
use std::sync::Arc;

pub(crate) static REGEX_CACHE: Lazy<Mutex<LruCache<String, Arc<regex::Regex>>>> =
    Lazy::new(|| Mutex::new(LruCache::new(NonZeroUsize::new(128).unwrap())));

#[cfg(test)]
pub fn clear_regex_cache() {
    REGEX_CACHE.lock().clear();
}

/// Checks if the target string matches the glob pattern.
pub(crate) fn matches_glob(pattern: &str, target: &str) -> bool {
    let mut pattern_norm = pattern.replace('\\', "/");
    let mut target_norm = target.replace('\\', "/");

    if pattern_norm.starts_with("//?/") {
        pattern_norm = pattern_norm[4..].to_string();
    }
    if target_norm.starts_with("//?/") {
        target_norm = target_norm[4..].to_string();
    }

    // Defense-in-depth: reject path traversal components in target string
    if target_norm.contains("../") || target_norm.contains("/..") || target_norm == ".." {
        return false;
    }

    // Build a regex pattern from the wildcard/glob string
    let mut regex_str = String::new();
    let chars: Vec<char> = pattern_norm.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        match chars[i] {
            '*' => {
                if i + 1 < chars.len() && chars[i + 1] == '*' {
                    let has_leading_slash = i > 0 && chars[i - 1] == '/';
                    let has_trailing_slash = i + 2 < chars.len() && chars[i + 2] == '/';

                    if has_leading_slash && has_trailing_slash {
                        regex_str.push_str("(?:[^/]+/)*");
                        i += 3;
                    } else {
                        regex_str.push_str(".*");
                        i += 2;
                    }
                } else {
                    regex_str.push_str("[^/]*");
                    i += 1;
                }
            }
            '?' => {
                regex_str.push_str("[^/]");
                i += 1;
            }
            '.' | '+' | '^' | '$' | '(' | ')' | '[' | ']' | '{' | '}' | '|' => {
                regex_str.push('\\');
                regex_str.push(chars[i]);
                i += 1;
            }
            c => {
                regex_str.push(c);
                i += 1;
            }
        }
    }

    let regex_pattern = format!("^{}(/.*)?$", regex_str);

    // Fast path: Check compiled regex cache under short-lived lock
    {
        let mut cache = REGEX_CACHE.lock();
        if let Some(re) = cache.get(&regex_pattern) {
            return re.is_match(&target_norm);
        }
    }

    // Compile regex outside the global cache lock to prevent stall
    match regex::Regex::new(&regex_pattern) {
        Ok(re) => {
            let arc_re = Arc::new(re);
            REGEX_CACHE.lock().put(regex_pattern, arc_re.clone());
            arc_re.is_match(&target_norm)
        }
        Err(_) => target_norm.starts_with(&pattern_norm),
    }
}

/// Resolves an executable name to its absolute canonical path by searching `$PATH`.
pub fn resolve_executable_path(exe_name: &str) -> Option<String> {
    tracing::trace!("[paths] Resolving executable path for '{}'", exe_name);
    let exe_path = Path::new(exe_name);
    if exe_path.is_absolute() {
        match exe_path.canonicalize() {
            Ok(canonical) => {
                let mut path_str = canonical.to_string_lossy().to_string().replace('\\', "/");
                if path_str.starts_with("//?/") {
                    path_str = path_str[4..].to_string();
                }
                return Some(path_str);
            }
            Err(e) => {
                tracing::warn!(
                    "[capability] Failed to canonicalize absolute executable path '{}': {:?}",
                    exe_name,
                    e
                );
                return None;
            }
        }
    }

    let path_var = match std::env::var_os("PATH") {
        Some(p) => p,
        None => {
            tracing::warn!("[capability] PATH environment variable is not set");
            return None;
        }
    };
    let paths = std::env::split_paths(&path_var);

    #[cfg(target_os = "windows")]
    let extensions = vec!["", ".exe", ".cmd", ".bat"];
    #[cfg(not(target_os = "windows"))]
    let extensions = vec![""];

    for path in paths {
        for ext in &extensions {
            let full_name = format!("{}{}", exe_name, ext);
            let check_path = path.join(&full_name);
            if check_path.is_file() {
                if let Ok(canonical) = check_path.canonicalize() {
                    let mut path_str = canonical.to_string_lossy().to_string().replace('\\', "/");
                    if path_str.starts_with("//?/") {
                        path_str = path_str[4..].to_string();
                    }
                    return Some(path_str);
                }
            }
        }
    }
    None
}

/// Sanitizes a file write pattern, blocking path traversal and restricting it to the workspace root.
pub(crate) fn sanitize_allowed_pattern(pattern: &str, workspace_root: &Path) -> Option<String> {
    let workspace_canonical = workspace_root
        .canonicalize()
        .unwrap_or_else(|_| workspace_root.to_path_buf());
    let mut workspace_str = workspace_canonical
        .to_string_lossy()
        .to_string()
        .replace('\\', "/");
    if workspace_str.starts_with("//?/") {
        workspace_str = workspace_str[4..].to_string();
    }
    let norm_pattern = pattern.replace('\\', "/");

    // Block path traversal attempts
    if norm_pattern.contains("../") || norm_pattern.contains("/..") || norm_pattern == ".." {
        return None;
    }

    if norm_pattern.starts_with('/') || norm_pattern.contains(':') {
        // Absolute pattern. Must reside strictly inside the workspace root (exact match or subpath)
        let ws_clean = workspace_str.trim_end_matches('/');
        let is_contained =
            norm_pattern == ws_clean || norm_pattern.starts_with(&format!("{}/", ws_clean));

        if is_contained {
            Some(norm_pattern)
        } else {
            None
        }
    } else {
        // Relative pattern. Clean prefix slashes and prepend the workspace root path
        let workspace_clean = workspace_str.trim_end_matches('/');
        let pattern_clean = norm_pattern.trim_start_matches('/');
        Some(format!("{}/{}", workspace_clean, pattern_clean))
    }
}

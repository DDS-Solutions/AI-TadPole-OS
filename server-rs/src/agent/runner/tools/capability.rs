//! @docs ARCHITECTURE:Security
//!
//! ### AI Assist Note
//! **Capability-Based Security (CBS)**: Implements the **SEC-04** zero-trust
//! model. Tools no longer have ambient authority. They must be invoked with
//! a non-forgeable `CapabilityToken` that defines a set of explicit
//! `Permission` grants.
//!
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: Runtime logic error, state desynchronization, or resource exhaustion.
//! - **Telemetry Link**: Search `[capability]` in tracing logs.

use lru::LruCache;
use once_cell::sync::Lazy;
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use std::num::NonZeroUsize;

static REGEX_CACHE: Lazy<Mutex<LruCache<String, regex::Regex>>> =
    Lazy::new(|| Mutex::new(LruCache::new(NonZeroUsize::new(128).unwrap())));

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Permission {
    /// Grants read access to a specific path or pattern.
    FileRead(String),
    /// Grants write access to a specific path or pattern.
    FileWrite(String),
    /// Grants permission to execute a specific command.
    ShellExecute(String),
    /// Grants permission to spawn sub-agents.
    SpawnAgent,
    /// Grants permission to fetch external URLs.
    NetworkFetch(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityToken {
    pub id: String,
    pub agent_id: String,
    pub mission_id: String,
    pub permissions: Vec<Permission>,
    pub expires_at: chrono::DateTime<chrono::Utc>,
}

fn matches_glob(pattern: &str, target: &str) -> bool {
    let pattern_norm = pattern.replace('\\', "/");
    let target_norm = target.replace('\\', "/");

    if pattern_norm == "." {
        return true;
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
                        regex_str.pop();
                        regex_str.push_str("/(?:.*/)?");
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
                regex_str.push('.');
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

    // Look up in LRU cache to avoid compiling Regex on every check
    let re_opt = {
        let mut cache = REGEX_CACHE.lock();
        cache.get(&regex_pattern).cloned()
    };

    let re = match re_opt {
        Some(re) => re,
        None => {
            if let Ok(re) = regex::Regex::new(&regex_pattern) {
                let mut cache = REGEX_CACHE.lock();
                cache.put(regex_pattern, re.clone());
                re
            } else {
                return target_norm.starts_with(&pattern_norm);
            }
        }
    };

    re.is_match(&target_norm)
}

/// Sanitizes a file write pattern, blocking path traversal and restricting it to the workspace root.
fn sanitize_allowed_pattern(pattern: &str, workspace_root: &std::path::Path) -> Option<String> {
    let workspace_str = workspace_root
        .to_string_lossy()
        .to_string()
        .replace('\\', "/");
    let norm_pattern = pattern.replace('\\', "/");

    // Block path traversal attempts
    if norm_pattern.contains("../") || norm_pattern.contains("/..") || norm_pattern == ".." {
        return None;
    }

    if norm_pattern.starts_with('/') || norm_pattern.contains(':') {
        // Absolute pattern. Must reside inside the workspace root
        if norm_pattern.starts_with(&workspace_str) {
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

impl CapabilityToken {
    /// Verifies if the token contains the required permission.
    #[allow(dead_code)]
    pub fn verify(&self, required: &Permission) -> bool {
        if chrono::Utc::now() > self.expires_at {
            return false;
        }

        self.permissions.iter().any(|p| match (p, required) {
            (Permission::FileRead(p1), Permission::FileRead(p2)) => matches_glob(p1, p2),
            (Permission::FileWrite(p1), Permission::FileWrite(p2)) => matches_glob(p1, p2),
            (p1, p2) => p1 == p2,
        })
    }
}

pub struct ZeroTrustGuard;

impl ZeroTrustGuard {
    /// Generates a capability token based on agent authority, mission scope, and allowed files.
    pub fn mint_token(
        agent_id: &str,
        mission_id: &str,
        authority: crate::agent::types::RoleAuthorityLevel,
        allowed_files: Option<&[String]>,
        workspace_root: &std::path::Path,
    ) -> CapabilityToken {
        let mut permissions = vec![];

        // Default read permission: read-only access to the entire workspace
        let workspace_str = workspace_root
            .to_string_lossy()
            .to_string()
            .replace('\\', "/");
        permissions.push(Permission::FileRead(workspace_str.clone()));

        // Add role-specific execution and spawn permissions
        match authority {
            crate::agent::types::RoleAuthorityLevel::Executive
            | crate::agent::types::RoleAuthorityLevel::Management => {
                permissions.push(Permission::SpawnAgent);
                permissions.push(Permission::ShellExecute("cargo".to_string()));
                permissions.push(Permission::ShellExecute("npm".to_string()));
            }
            _ => {}
        }

        // Add write permissions for Executive, Management, and Specialist
        if matches!(
            authority,
            crate::agent::types::RoleAuthorityLevel::Executive
                | crate::agent::types::RoleAuthorityLevel::Management
                | crate::agent::types::RoleAuthorityLevel::Specialist
        ) {
            if let Some(files) = allowed_files {
                if !files.is_empty() {
                    for pattern in files {
                        if let Some(sanitized) = sanitize_allowed_pattern(pattern, workspace_root) {
                            permissions.push(Permission::FileWrite(sanitized));
                        }
                    }
                } else {
                    permissions.push(Permission::FileWrite(workspace_str.clone()));
                }
            } else {
                permissions.push(Permission::FileWrite(workspace_str.clone()));
            }
        }

        CapabilityToken {
            id: uuid::Uuid::new_v4().to_string(),
            agent_id: agent_id.to_string(),
            mission_id: mission_id.to_string(),
            permissions,
            expires_at: chrono::Utc::now() + chrono::Duration::hours(1),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_matches_glob() {
        assert!(matches_glob(".", "/path/to/file.txt"));
        assert!(matches_glob("/base", "/base/src/main.rs"));

        // Glob wildcard pattern matching
        assert!(matches_glob("/base/src/**/*.rs", "/base/src/main.rs"));
        assert!(matches_glob(
            "/base/src/**/*.rs",
            "/base/src/agent/types.rs"
        ));
        assert!(!matches_glob("/base/src/**/*.rs", "/base/tests/main.rs"));

        // Single star matching
        assert!(matches_glob("/base/src/*.rs", "/base/src/main.rs"));
        assert!(!matches_glob("/base/src/*.rs", "/base/src/agent/types.rs"));
    }

    #[test]
    fn test_sanitize_allowed_pattern() {
        let root = Path::new("/base");

        // Traversal attempts must be rejected
        assert_eq!(sanitize_allowed_pattern("../../etc/passwd", root), None);
        assert_eq!(sanitize_allowed_pattern("src/../../etc/passwd", root), None);
        assert_eq!(sanitize_allowed_pattern("/etc/passwd", root), None);

        // Valid relative patterns are resolved absolute
        assert_eq!(
            sanitize_allowed_pattern("src/**/*.rs", root),
            Some("/base/src/**/*.rs".to_string())
        );

        // Valid absolute patterns are preserved
        assert_eq!(
            sanitize_allowed_pattern("/base/src/**/*.rs", root),
            Some("/base/src/**/*.rs".to_string())
        );
    }

    #[test]
    fn test_token_verify_allowed_files() {
        let root = Path::new("/base");
        let allowed = vec!["src/**/*.rs".to_string(), "Cargo.toml".to_string()];
        let token = ZeroTrustGuard::mint_token(
            "worker",
            "mission",
            crate::agent::types::RoleAuthorityLevel::Specialist,
            Some(&allowed),
            root,
        );

        // Reads are allowed globally in the workspace
        assert!(token.verify(&Permission::FileRead("/base/tests/main.rs".to_string())));

        // Writes inside allowed patterns are permitted
        assert!(token.verify(&Permission::FileWrite("/base/src/main.rs".to_string())));
        assert!(token.verify(&Permission::FileWrite("/base/Cargo.toml".to_string())));

        // Writes outside allowed patterns are blocked
        assert!(!token.verify(&Permission::FileWrite("/base/tests/main.rs".to_string())));
        assert!(!token.verify(&Permission::FileWrite("/base/src/main.js".to_string())));

        // Test traversal pattern ingestion blocks writes
        let allowed_traversal = vec!["../../etc/passwd".to_string(), "/etc/passwd".to_string()];
        let token_traversal = ZeroTrustGuard::mint_token(
            "worker",
            "mission",
            crate::agent::types::RoleAuthorityLevel::Specialist,
            Some(&allowed_traversal),
            root,
        );
        assert!(!token_traversal.verify(&Permission::FileWrite("/etc/passwd".to_string())));
        assert!(
            !token_traversal.verify(&Permission::FileWrite("/base/../../etc/passwd".to_string()))
        );
    }
}

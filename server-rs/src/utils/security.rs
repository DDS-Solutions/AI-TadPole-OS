//! Security Foundation & Hardening Utilities
//!
//! Provides the core primitives for path validation, sensitive-ID
//! sanitization, and regex-based secret redaction (PII/Key protection).
//!
//! @docs ARCHITECTURE:SecurityHardening
//!
//! ### AI Assist Note
//! **Security Utilities**: Orchestrates the core primitives for **Path
//! Validation** (traversal protection), **Sensitive-ID Sanitization**,
//! and **Regex-Based Secret Redaction** (PII/Key protection). Features
//! hierarchical redaction: strings like "Authorization: Bearer <key>"
//! are processed by multiple overlapping patterns (header, token, and
//! prefix-specific regex) to ensure maximum safety. AI agents should
//! use `validate_path` for all filesystem operations to maintain
//! conformance with the **SEC-03** security model (SECUTIL-01).
//!
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: `Path traversal detected` errors during
//!   cross-module file access, excessive redaction of non-sensitive
//!   metadata, or regex performance overhead on large tool outputs.
//! - **Trace Scope**: `server-rs::utils::security`

use anyhow::{anyhow, Result};
use std::path::{Path, PathBuf};

/// Sanitizes and validates a path to prevent directory traversal.
/// Ensures the path is within the provided base directory.
pub fn validate_path(base: &Path, user_path: &str) -> Result<PathBuf> {
    // 1. Normalize the base path to absolute and resolve ".."
    let base_raw = if base.is_absolute() {
        base.to_path_buf()
    } else {
        std::env::current_dir()?.join(base)
    };

    // Helper to normalize a path by resolving components
    fn normalize(p: &Path) -> PathBuf {
        let mut components = Vec::new();
        for component in p.components() {
            match component {
                std::path::Component::Prefix(prefix) => {
                    components.push(std::path::Component::Prefix(prefix))
                }
                std::path::Component::RootDir => components.push(std::path::Component::RootDir),
                std::path::Component::CurDir => {}
                std::path::Component::ParentDir => {
                    if let Some(std::path::Component::Normal(_)) = components.last() {
                        components.pop();
                    }
                }
                std::path::Component::Normal(c) => components.push(std::path::Component::Normal(c)),
            }
        }
        components.iter().collect()
    }

    let base_abs = normalize(&base_raw);
    let joined = base_abs.join(user_path);
    let result = normalize(&joined);

    // 2. Final safety check: result must still be prefixed by the base_abs
    if !result.starts_with(&base_abs) {
        return Err(anyhow!("Path traversal detected: outside authorized base"));
    }

    Ok(result)
}

/// Strictly sanitizes a string to be used as a filename or ID.
pub fn sanitize_id(id: &str) -> String {
    id.chars()
        .filter(|c| c.is_alphanumeric() || *c == '_' || *c == '-')
        .collect()
}

/// Redacts sensitive information from strings (e.g., API keys, secrets).
/// Uses regex to catch JSON keys (apiKey, token, etc.) and common prefixes.
pub fn redact_secrets(input: &str) -> String {
    use once_cell::sync::Lazy;
    use regex::Regex;

    static PATTERNS: Lazy<Vec<(Regex, &'static str)>> = Lazy::new(|| {
        vec![
            // 1. Bearer tokens in headers or strings
            (
                Regex::new(r"(?i)bearer\s+[a-zA-Z0-9\-\._~+/]+=*").unwrap(),
                "bearer [REDACTED]",
            ),
            // 2. Authorization headers
            (
                Regex::new(r"(?i)authorization:\s*[^\s,]+").unwrap(),
                "authorization: [REDACTED]",
            ),
            // 3. JSON keys: "apiKey": "...", "token": "...", etc.
            (
                Regex::new(r#"(?i)("?(?:api_key|secret|password|token|key|credential)"?\s*[:=]\s*)(["'])(?:\\.|[^"'])*(["'])"#).unwrap(),
                r#"$1$2[REDACTED]$3"#,
            ),
            // 4. API Key prefixes
            (
                Regex::new(r"(?i)sk-[a-zA-Z0-9]{20,}").unwrap(),
                "sk-[REDACTED]",
            ), // OpenAI
            (
                Regex::new(r"(?i)AIza[0-9A-Za-z-_]{30,}").unwrap(),
                "AIza[REDACTED]",
            ), // Google
            (
                Regex::new(r"(?i)ghp_[a-zA-Z0-9]{30,}").unwrap(),
                "ghp_[REDACTED]",
            ), // GitHub
        ]
    });

    let mut output = input.to_string();
    for (re, replacement) in PATTERNS.iter() {
        output = re.replace_all(&output, *replacement).to_string();
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_sanitize_id() {
        assert_eq!(sanitize_id("agent123"), "agent123");
        assert_eq!(sanitize_id("agent-123_abc"), "agent-123_abc");
        assert_eq!(sanitize_id("../etc/passwd"), "etcpasswd");
        assert_eq!(sanitize_id("id with spaces"), "idwithspaces");
        assert_eq!(sanitize_id("id!@#$%^&*()"), "id");
    }

    #[test]
    fn test_validate_path_safe() {
        let base = Path::new("/tmp/base");
        // We use a relative path for the user_path
        let result = validate_path(base, "user1/data").unwrap();
        assert!(result.to_string_lossy().contains("user1"));
        assert!(result.to_string_lossy().contains("data"));
    }

    #[test]
    fn test_validate_path_traversal() {
        let base = Path::new("/tmp/base");

        // Attempt to go up
        let result = validate_path(base, "../outside");
        assert!(result.is_err());

        // Attempt to go up and back in
        let result = validate_path(base, "user1/../../outside");
        assert!(result.is_err());

        // Absolute path attempt
        #[cfg(not(windows))]
        {
            let result = validate_path(base, "/etc/passwd");
            assert!(result.is_err());
        }
        #[cfg(windows)]
        {
            let result = validate_path(base, "C:\\Windows\\System32");
            assert!(result.is_err());
        }
    }

    #[test]
    fn test_redact_secrets() {
        let json = r#"{"api_key": "sk-1234567890abcdef1234567890abcdef", "other": "value"}"#;
        let redacted = redact_secrets(json);
        assert!(redacted.contains(r#""api_key": "[REDACTED]""#));
        assert!(redacted.contains(r#""other": "value""#));

        let bearer = "Authorization: Bearer abc123xyz";
        let redacted_bearer = redact_secrets(bearer);
        // It's redacted twice because of overlapping patterns ("Authorization" and "Bearer")
        // This is safe even if a bit greedy.
        assert!(redacted_bearer.contains("[REDACTED]"));
    }
}

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
                Regex::new(
                    r#"(?i)"(apiKey|api_key|token|secret|password|private_key)"\s*:\s*"[^"]*""#,
                )
                .unwrap(),
                r#""$1": "[REDACTED]""#,
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

//! @docs ARCHITECTURE:Security
//!
//! ### AI Assist Note
//! **Security Foundation & Hardening**: Provides the core primitives for path
//! validation, sensitive-ID sanitization, and regex-based secret redaction.
//! Implements **SEC-03** and **SEC-04** zero-trust models.
//!
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: Runtime logic error, state desynchronization, or resource exhaustion.
//! - **Telemetry Link**: Search `[security]` in tracing logs.
//!

// [security]
use crate::error::AppError;
use std::net::IpAddr;
use std::path::{Path, PathBuf};
use unicode_normalization::UnicodeNormalization;

/// A non-forgeable wrapper around a validated path.
/// Can only be created through successful validation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SafePath(PathBuf);

impl SafePath {
    pub fn from_trusted(p: PathBuf) -> Self {
        Self(p)
    }
    pub fn as_path(&self) -> &Path {
        &self.0
    }
    #[allow(dead_code)]
    pub fn to_path_buf(&self) -> PathBuf {
        self.0.clone()
    }
}

pub fn is_production_env() -> bool {
    std::env::var("TADPOLE_ENV")
        .map(|v| v.eq_ignore_ascii_case("production"))
        .unwrap_or(false)
}

fn is_blocked_public_fetch_host(host: &str) -> bool {
    let host = host.trim_end_matches('.').to_ascii_lowercase();
    host == "localhost"
        || host.ends_with(".localhost")
        || host == "host.docker.internal"
        || host.ends_with(".local")
}

fn is_public_routable_ip(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(ip) => {
            !(ip.is_private()
                || ip.is_loopback()
                || ip.is_link_local()
                || ip.is_unspecified()
                || ip.is_broadcast()
                || ip.octets()[0] == 0
                || ip.octets()[0] >= 224
                || is_ipv4_cgnat(ip))
        }
        IpAddr::V6(ip) => {
            !(ip.is_loopback()
                || ip.is_unspecified()
                || is_ipv6_unique_local(ip)
                || is_ipv6_unicast_link_local(ip))
        }
    }
}

fn is_ipv4_cgnat(ip: std::net::Ipv4Addr) -> bool {
    let octets = ip.octets();
    octets[0] == 100 && (64..=127).contains(&octets[1])
}

fn is_ipv6_unique_local(ip: std::net::Ipv6Addr) -> bool {
    (ip.segments()[0] & 0xfe00) == 0xfc00
}

fn is_ipv6_unicast_link_local(ip: std::net::Ipv6Addr) -> bool {
    (ip.segments()[0] & 0xffc0) == 0xfe80
}

/// A validated public outbound HTTP/HTTPS target.
/// Used to prevent DNS rebinding by forcing caller connections to the resolved IP.
#[derive(Debug, Clone)]
pub struct ValidatedUrl {
    pub ip: IpAddr,
    pub host: String,
    pub port: u16,
    #[allow(dead_code)]
    pub url: reqwest::Url,
}

/// Validates an agent/user-controlled outbound URL before the engine fetches it.
/// Blocks local, private, link-local, multicast, and metadata-style targets after DNS resolution.
/// Returns a `ValidatedUrl` struct, which contains the resolved IP, hostname, and port.
/// Callers must use the resolved `ValidatedUrl` elements to perform the network request, rather than the original string, to prevent DNS rebinding.
pub async fn validate_public_http_url(url: &str) -> Result<ValidatedUrl, AppError> {
    let parsed = reqwest::Url::parse(url)
        .map_err(|_| AppError::BadRequest("URL must be absolute and valid".to_string()))?;

    match parsed.scheme() {
        "http" | "https" => {}
        _ => {
            return Err(AppError::Forbidden(
                "Only http and https URLs are allowed for outbound fetches".to_string(),
            ));
        }
    }

    if !parsed.username().is_empty() || parsed.password().is_some() {
        return Err(AppError::Forbidden(
            "URL credentials are not allowed for outbound fetches".to_string(),
        ));
    }

    let host = parsed
        .host_str()
        .ok_or_else(|| AppError::BadRequest("URL host is required".to_string()))?;

    if is_blocked_public_fetch_host(host) {
        return Err(AppError::Forbidden(
            "Local or internal hostnames cannot be fetched by agents".to_string(),
        ));
    }

    let host_str = host.to_string();

    let port = parsed.port_or_known_default().ok_or_else(|| {
        AppError::BadRequest("URL must use a scheme with a known port".to_string())
    })?;

    if let Ok(ip) = host_str.parse::<IpAddr>() {
        if is_public_routable_ip(ip) {
            return Ok(ValidatedUrl {
                ip,
                host: host_str,
                port,
                url: parsed,
            });
        }
        return Err(AppError::Forbidden(
            "Local, private, or reserved IP addresses cannot be fetched by agents".to_string(),
        ));
    }

    let lookup_future = tokio::net::lookup_host((host_str.clone(), port));
    let mut addrs =
        match tokio::time::timeout(std::time::Duration::from_secs(3), lookup_future).await {
            Ok(Ok(a)) => a,
            Ok(Err(_)) => {
                return Err(AppError::BadRequest(
                    "URL host could not be resolved".to_string(),
                ))
            }
            Err(_) => {
                return Err(AppError::BadRequest(
                    "URL host DNS resolution timed out".to_string(),
                ))
            }
        };

    let mut saw_addr = false;
    let mut resolved_ip = None;
    for addr in addrs.by_ref() {
        if resolved_ip.is_none() {
            resolved_ip = Some(addr.ip());
        }
        saw_addr = true;
        if !is_public_routable_ip(addr.ip()) {
            return Err(AppError::Forbidden(
                "Resolved URL target is local, private, or reserved".to_string(),
            ));
        }
    }

    if !saw_addr {
        return Err(AppError::BadRequest(
            "URL host did not resolve to any addresses".to_string(),
        ));
    }

    let ip = resolved_ip.ok_or_else(|| {
        AppError::BadRequest("URL host did not resolve to any addresses".to_string())
    })?;

    Ok(ValidatedUrl {
        ip,
        host: host_str,
        port,
        url: parsed,
    })
}

impl AsRef<Path> for SafePath {
    fn as_ref(&self) -> &Path {
        &self.0
    }
}

impl std::ops::Deref for SafePath {
    type Target = Path;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

fn is_device_file(path: &Path) -> bool {
    let reserved = [
        "CON", "PRN", "AUX", "NUL", "COM0", "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7",
        "COM8", "COM9", "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9",
    ];
    for component in path.components() {
        if let std::path::Component::Normal(os_str) = component {
            let name_str = os_str.to_string_lossy().to_uppercase();
            let base_name = match name_str.split_once('.') {
                Some((base, _ext)) => base,
                None => &name_str,
            };
            if reserved.contains(&base_name) {
                return true;
            }
        }
    }
    let path_str = path.to_string_lossy();
    if path_str.starts_with("/dev/")
        || path_str.starts_with("/proc/")
        || path_str.starts_with("/sys/")
        || path_str.starts_with("/run/")
    {
        return true;
    }
    false
}

fn normalize_lexical(p: &Path) -> PathBuf {
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

/// Sanitizes and validates a path to prevent directory traversal and symlink follow bypasses (S-008).
///
/// ### ⚠️ Security Warning (TOCTOU)
/// This function is subject to a Time-of-Check Time-of-Use (TOCTOU) race condition.
/// Between the validation check and the actual file access, a malicious process can replace
/// directory structures with symlinks. Callers should open file descriptors immediately or
/// restrict system access controls where possible.
fn strip_unc_prefix(p: &Path) -> PathBuf {
    let p_str = p.to_string_lossy();
    if let Some(stripped) = p_str.strip_prefix(r"\\?\") {
        PathBuf::from(stripped)
    } else {
        p.to_path_buf()
    }
}

pub fn validate_path(base: &Path, user_path: &str) -> Result<SafePath, AppError> {
    // 1. Canonicalize the base directory (it must exist)
    let base_canon = std::fs::canonicalize(base)
        .map_err(|e| AppError::BadRequest(format!("Invalid base path: {}", e)))?;
    let base_canon_stripped = strip_unc_prefix(&base_canon);

    // 2. Normalize and join the user path
    let joined = base_canon.join(user_path);

    // 3. Resolve symlinks using std::fs::canonicalize if the path or parent exists
    let resolved = if joined.exists() {
        std::fs::canonicalize(&joined)
            .map_err(|e| AppError::BadRequest(format!("Invalid path resolution: {}", e)))?
    } else {
        let mut parent = joined.as_path();
        let mut depth = 0;
        const MAX_PARENT_DEPTH: usize = 32;
        while let Some(p) = parent.parent() {
            depth += 1;
            if depth > MAX_PARENT_DEPTH {
                return Err(AppError::Forbidden(
                    "Path structure is too deep".to_string(),
                ));
            }
            if p.exists() {
                parent = p;
                break;
            }
            parent = p;
        }
        if parent.exists() {
            let parent_canon = std::fs::canonicalize(parent)
                .map_err(|e| AppError::BadRequest(format!("Invalid parent resolution: {}", e)))?;
            let parent_canon_stripped = strip_unc_prefix(&parent_canon);
            if !parent_canon_stripped.starts_with(&base_canon_stripped) {
                return Err(AppError::Forbidden(
                    "Path traversal detected: parent outside authorized base".to_string(),
                ));
            }
            let relative = joined.strip_prefix(parent).unwrap_or(Path::new(""));
            parent_canon.join(relative)
        } else {
            normalize_lexical(&joined)
        }
    };

    let resolved_norm = normalize_lexical(&resolved);
    let resolved_norm_stripped = strip_unc_prefix(&resolved_norm);
    if !resolved_norm_stripped.starts_with(&base_canon_stripped) {
        return Err(AppError::Forbidden(
            "Path traversal detected: outside authorized base".to_string(),
        ));
    }

    if is_device_file(&resolved_norm) {
        return Err(AppError::Forbidden(
            "Access denied: device files or system directories cannot be accessed".to_string(),
        ));
    }

    Ok(SafePath(resolved_norm))
}

/// Sanitizes a string to be used as a filename or ID, with length cap and Unicode NFKC normalization.
pub fn sanitize_id(id: &str) -> String {
    id.nfkc()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '_' || *c == '-')
        .take(256)
        .collect()
}

/// Redacts sensitive credentials and keys from strings (S-010).
pub fn redact_secrets(input: &str) -> String {
    use once_cell::sync::Lazy;
    use regex::{Regex, RegexSet};

    struct RedactionPattern {
        pattern: &'static str,
        replacement: &'static str,
    }

    static PATTERNS: Lazy<(RegexSet, Vec<(Regex, &'static str)>)> = Lazy::new(|| {
        let rules = vec![
            // 0. Bearer tokens
            RedactionPattern {
                pattern: r"(?i)bearer\s+[a-zA-Z0-9\-\._~+/]+=*",
                replacement: "[REDACTED]",
            },
            // 1. Authorization header value
            RedactionPattern {
                pattern: r"(?i)authorization:\s*[^\s,]+",
                replacement: "[REDACTED]",
            },
            // 2. Generic key-value pairs for credentials
            RedactionPattern {
                pattern: r#"(?i)("?(?:api_key|secret|password|pwd|pass|token|key|credential)"?\s*[:=]\s*)(["']?)(?:\\.|[^"'\s\n])*(["']?)"#,
                replacement: r#"$1$2[REDACTED]$3"#,
            },
            // 3. OpenAI/Anthropic style keys
            RedactionPattern {
                pattern: r"(?i)sk-(?:ant-)?[a-zA-Z0-9]{20,}",
                replacement: "[REDACTED]",
            },
            // 4. Google API keys
            RedactionPattern {
                pattern: r"(?i)AIza[0-9A-Za-z-_]{30,}",
                replacement: "[REDACTED]",
            },
            // 5. GitHub PATs (classic and fine-grained)
            RedactionPattern {
                pattern: r"(?i)(?:ghp|github_pat)_[a-zA-Z0-9_]{30,120}",
                replacement: "[REDACTED]",
            },
            // 6. AWS Access Key IDs
            RedactionPattern {
                pattern: r"(?i)AKIA[0-9A-Z]{16}",
                replacement: "[REDACTED]",
            },
            // 7. Slack tokens
            RedactionPattern {
                pattern: r"(?i)xox[bp]-[a-zA-Z0-9\-]+",
                replacement: "[REDACTED]",
            },
            // 8. JWT tokens
            RedactionPattern {
                pattern: r"(?i)ey[a-zA-Z0-9_-]{10,}\.ey[a-zA-Z0-9_-]{10,}\.[a-zA-Z0-9_-]{10,}",
                replacement: "[REDACTED]",
            },
            // 9. PEM private keys
            RedactionPattern {
                pattern: r"(?s)-----BEGIN [A-Z ]+ PRIVATE KEY-----.+?-----END [A-Z ]+ PRIVATE KEY-----",
                replacement: "[REDACTED]",
            },
            // 10. Database connection string passwords
            RedactionPattern {
                pattern: r"(?i)([a-zA-Z0-9+.-]+://[a-zA-Z0-9_.-]+:)([^@\s]+)(@[a-zA-Z0-9_.-]+)",
                replacement: r#"$1[REDACTED]$3"#,
            },
            // 11. AWS secret access keys
            RedactionPattern {
                pattern: r"(?i)(aws_secret_access_key\s*[:=]\s*)([a-zA-Z0-9/+=]{40})",
                replacement: r#"$1[REDACTED]"#,
            },
        ];

        let set = RegexSet::new(rules.iter().map(|r| r.pattern))
            .expect("Security patterns must be valid regex.");
        let regexes = rules
            .iter()
            .map(|r| (Regex::new(r.pattern).unwrap(), r.replacement))
            .collect();
        (set, regexes)
    });

    let mut output = std::borrow::Cow::Borrowed(input);
    let (set, regexes) = &*PATTERNS;

    if set.is_match(&output) {
        for (idx, (re, replacement)) in regexes.iter().enumerate() {
            if set.matches(&output).matched(idx) {
                if let std::borrow::Cow::Owned(s) = re.replace_all(&output, *replacement) {
                    output = std::borrow::Cow::Owned(s);
                }
            }
        }
    }
    output.into_owned()
}

/// Helper tokenizer to safely extract command tokens while respecting single and double quotes.
fn parse_command_tokens(command: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut in_double_quote = false;
    let mut in_single_quote = false;
    let mut chars = command.chars().peekable();

    while let Some(c) = chars.next() {
        match c {
            '"' if !in_single_quote => {
                in_double_quote = !in_double_quote;
            }
            '\'' if !in_double_quote => {
                in_single_quote = !in_single_quote;
            }
            '\\' => {
                if let Some(&next_c) = chars.peek() {
                    if next_c == '"' || next_c == '\'' || next_c == '\\' || next_c == ' ' {
                        current.push(next_c);
                        chars.next();
                    } else {
                        current.push('\\');
                    }
                } else {
                    current.push('\\');
                }
            }
            c if c.is_whitespace() && !in_double_quote && !in_single_quote => {
                if !current.is_empty() {
                    tokens.push(current.clone());
                    current.clear();
                }
            }
            c => {
                current.push(c);
            }
        }
    }
    if !current.is_empty() {
        tokens.push(current);
    }
    tokens
}

/// Validates a shell command against a ZERO-TRUST whitelist, preventing separators (S-001) and subprocess RCEs.
pub fn validate_shell_command(command: &str) -> Result<(), AppError> {
    // 1. Strip shell comments (starting with #) prior to validation
    let command_no_comment = match command.split_once('#') {
        Some((code, _comment)) => code,
        None => command,
    };
    let lower = command_no_comment.to_lowercase();

    // 2. Block Newline, Carriage Return, Null Byte (RCE separators - S-001)
    if lower.contains('\n') || lower.contains('\r') || lower.contains('\0') {
        return Err(AppError::Forbidden(
            "Newline, carriage return, or null bytes are prohibited in commands".to_string(),
        ));
    }

    // 3. Block Command Substitution & Expansion (Critical Vulnerability)
    if lower.contains("$(") || lower.contains("`") || lower.contains("${") {
        return Err(AppError::Forbidden(
            "Command substitution or variable expansion detected".to_string(),
        ));
    }

    // 4. Block Piping, Chaining, and Input Redirection (including Unicode variants)
    if lower.contains('|')
        || lower.contains('<')
        || lower.contains(';')
        || lower.contains('&')
        || lower.contains('\u{FF1B}')
        || lower.contains('\u{FF5C}')
    {
        return Err(AppError::Forbidden(
            "Piping, chaining, or input redirection prohibited".to_string(),
        ));
    }

    // 5. Harden Output Redirection: Only allow redirection to /dev/null, $null, or standard error/output descriptors
    if lower.contains('>') {
        let mut rest = lower.as_str();
        while let Some(idx) = rest.find('>') {
            let redirect_target = rest[idx + 1..].trim();
            let target_token = redirect_target.split_whitespace().next().unwrap_or("");
            if target_token != "/dev/null"
                && target_token != "$null"
                && !target_token.starts_with("&1")
                && !target_token.starts_with("&2")
            {
                return Err(AppError::Forbidden(
                    "Output redirection is only permitted to /dev/null or $null".to_string(),
                ));
            }
            rest = &rest[idx + 1..];
        }
    }

    // 6. Block wildcard character glob expansion to prevent sandbox blacklist bypasses
    if lower.contains('*') || lower.contains('?') || lower.contains('[') || lower.contains(']') {
        return Err(AppError::Forbidden(
            "Wildcard characters (*, ?, []) are prohibited in commands to prevent sandbox bypass"
                .to_string(),
        ));
    }

    // 7. Whitelist of Allowed Base Commands
    let allowed_commands = [
        "ls", "cd", "pwd", "cat", "echo", "grep", "find", "cargo", "npm", "git", "python", "node",
        "rustc", "mkdir", "cp", "mv", "touch", "test",
    ];

    let tokens = parse_command_tokens(command_no_comment);
    let first_word = match tokens.first() {
        Some(w) => w.to_lowercase(),
        None => {
            return Err(AppError::Forbidden(
                "Empty command is prohibited".to_string(),
            ))
        }
    };

    if !allowed_commands.contains(&first_word.as_str()) {
        return Err(AppError::Forbidden(format!(
            "Command '{}' is not in the authorized whitelist",
            first_word
        )));
    }

    // 8. Command-specific option restrictions to prevent sub-shell escapes (S-002, S-003, S-004)
    if first_word == "find" {
        let dangerous_find = ["-exec", "-execdir", "-ok", "-okdir"];
        for flag in dangerous_find {
            if tokens.iter().any(|t| {
                let tl = t.to_lowercase();
                tl == flag || tl.starts_with(&format!("{}=", flag))
            }) {
                return Err(AppError::Forbidden(format!(
                    "Unauthorized find parameter: '{}'",
                    flag
                )));
            }
        }
    } else if first_word == "node" {
        // Grouped options check (e.g. -pe)
        for token in &tokens {
            if token.starts_with('-') && !token.starts_with("--") {
                for c in token[1..].chars() {
                    if ['e', 'p', 'i', 'r'].contains(&c) {
                        return Err(AppError::Forbidden(format!(
                            "Unauthorized node option: '-{}'",
                            c
                        )));
                    }
                }
            } else {
                let tl = token.to_lowercase();
                if tl == "--eval"
                    || tl.starts_with("--eval=")
                    || tl == "--print"
                    || tl.starts_with("--print=")
                    || tl == "--interactive"
                    || tl.starts_with("--interactive=")
                    || tl == "--require"
                    || tl.starts_with("--require=")
                {
                    return Err(AppError::Forbidden(format!(
                        "Unauthorized node option: '{}'",
                        token
                    )));
                }
            }
        }
    } else if first_word == "python" {
        for token in &tokens {
            if token.starts_with('-') && !token.starts_with("--") {
                for c in token[1..].chars() {
                    if ['c', 'm', 'i'].contains(&c) {
                        return Err(AppError::Forbidden(format!(
                            "Unauthorized python option: '-{}'",
                            c
                        )));
                    }
                }
            }
        }
    } else if first_word == "git" {
        // Parse argv token boundaries (V2-005) to block config/exec/etc. and transport protocols.
        let dangerous_git_subcommands = ["config", "exec", "upload-pack", "receive-pack"];
        let dangerous_git_flags = ["-c", "--git-dir", "--work-tree", "core.pager"];

        for token in &tokens {
            let tl = token.to_lowercase();
            if dangerous_git_subcommands.contains(&tl.as_str()) {
                return Err(AppError::Forbidden(format!(
                    "Unauthorized git parameter or option detected: '{}'",
                    token
                )));
            }
            if dangerous_git_flags
                .iter()
                .any(|&flag| tl == flag || tl.starts_with(&format!("{}=", flag)))
            {
                return Err(AppError::Forbidden(format!(
                    "Unauthorized git parameter or option detected: '{}'",
                    token
                )));
            }
            // Check for transport protocols (e.g. ext::, ssh::, git::, etc. or url variants)
            if tl.contains("ext::")
                || tl.contains("ssh::")
                || tl.contains("git::")
                || tl.contains("file://")
                || tl.contains("ssh://")
                || tl.contains("git://")
            {
                return Err(AppError::Forbidden(
                    "Unauthorized git transport protocol prefix detected".to_string(),
                ));
            }
        }
    } else if first_word == "npm" {
        let dangerous_npm = ["run", "exec", "install", "i", "config"];
        for token in &tokens {
            let tl = token.to_lowercase();
            if dangerous_npm.contains(&tl.as_str()) {
                return Err(AppError::Forbidden(format!(
                    "Unauthorized npm parameter or option detected: '{}'",
                    token
                )));
            }
        }
    } else if first_word == "cargo" {
        let dangerous_cargo = ["run", "test", "bench"];
        for token in &tokens {
            let tl = token.to_lowercase();
            if dangerous_cargo.contains(&tl.as_str()) {
                return Err(AppError::Forbidden(format!(
                    "Unauthorized cargo command detected: '{}'",
                    token
                )));
            }
        }
    }

    // 9. Blacklist specific dangerous flags/paths for allowed commands
    let dangerous_flags = [
        "--erase",
        "--delete",
        "-rf",
        "/etc",
        "/root",
        "/var",
        "/bin",
        "/usr",
        "tadpole.db",
        ".env",
        ".gemini",
        "knowledge",
    ];
    for flag in dangerous_flags {
        if tokens.iter().any(|t| t.to_lowercase().contains(flag)) {
            return Err(AppError::Forbidden(format!(
                "Dangerous flag or path detected: '{}'",
                flag
            )));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_validate_public_http_url_blocks_internal_targets() {
        assert!(validate_public_http_url("http://127.0.0.1:8000")
            .await
            .is_err());
        assert!(validate_public_http_url("http://localhost:8000")
            .await
            .is_err());
        assert!(validate_public_http_url("http://10.0.0.1/status")
            .await
            .is_err());
        assert!(
            validate_public_http_url("http://10.0.0.1/latest/meta-data")
                .await
                .is_err()
        );
        assert!(validate_public_http_url("file:///etc/passwd")
            .await
            .is_err());
        assert!(validate_public_http_url("https://10.0.0.1/")
            .await
            .is_ok());
    }

    #[test]
    fn test_validate_path_traversal() {
        let base = std::env::temp_dir().join(format!("tadpole-test-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&base).ok();

        let base_canon = std::fs::canonicalize(&base).unwrap();

        assert!(validate_path(&base_canon, "../outside").is_err());
        assert!(validate_path(&base_canon, "user1/../../outside").is_err());

        // Windows device names & Unix device folders
        assert!(validate_path(&base_canon, "con").is_err());
        assert!(validate_path(&base_canon, "PRN.txt").is_err());
        assert!(validate_path(&base_canon, "subdir/aux").is_err());
        assert!(validate_path(&base_canon, "/dev/stdout").is_err());
    }

    #[test]
    fn test_validate_shell_zero_trust() {
        // Authorized
        assert!(validate_shell_command("ls -la").is_ok());
        assert!(validate_shell_command("cargo build --release").is_ok());
        assert!(validate_shell_command("npm test").is_ok());

        // Unauthorized Command
        assert!(validate_shell_command("rm -rf .").is_err());
        assert!(validate_shell_command("curl http://evil.com").is_err());

        // Injection Attempts
        assert!(validate_shell_command("ls; rm -rf /").is_err());
        assert!(validate_shell_command("echo $(cat /etc/passwd)").is_err());
        assert!(validate_shell_command("ls `rm -rf /`").is_err());
        assert!(validate_shell_command("cat /etc/passwd > out.txt").is_err());

        // Redirection Comment Bypass Attempt
        assert!(validate_shell_command("cat data/tadpole.db > output.txt # /dev/null").is_err());

        // Wildcard / Globbing Bypass Attempt
        assert!(validate_shell_command("cat data/tad*.db").is_err());

        // git/npm/cargo Dangerous Options/Subcommands
        assert!(validate_shell_command("git -c core.pager=evil diff").is_err());
        assert!(validate_shell_command("npm install package").is_err());
        assert!(validate_shell_command("cargo run").is_err());

        // Dangerous Flags/Paths
        assert!(validate_shell_command("ls /etc/shadow").is_err());
    }

    #[test]
    fn test_validate_shell_injection_bypass_prevention() {
        // Newline command separation (S-001)
        assert!(validate_shell_command("ls\nrm -rf /").is_err());
        assert!(validate_shell_command("ls\rrm -rf /").is_err());
        assert!(validate_shell_command("ls\0rm").is_err());

        // find -exec (S-002)
        assert!(validate_shell_command("find . -exec rm {} +").is_err());
        assert!(validate_shell_command("find . -ok rm {} ;").is_err());

        // node -e / --eval (S-003)
        assert!(validate_shell_command("node -e \"console.log(1)\"").is_err());
        assert!(validate_shell_command("node --eval \"console.log(1)\"").is_err());

        // python -c / -m (S-003)
        assert!(validate_shell_command("python -c \"import os; os.system('ls')\"").is_err());
        assert!(validate_shell_command("python -m http.server").is_err());

        // git transport RCE (S-004)
        assert!(validate_shell_command("git clone ext::sh -c evil").is_err());
    }

    #[test]
    fn test_redact_secrets_extended_patterns() {
        // Anthropic keys
        assert!(redact_secrets("sk-ant-abc12345678901234567890").contains("[REDACTED]"));

        // Slack tokens
        assert!(redact_secrets("xoxb-abc-123").contains("[REDACTED]"));
        assert!(redact_secrets("xoxp-abc-123").contains("[REDACTED]"));

        // JWT
        assert!(redact_secrets("eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIiwiaWF0IjoxNTE2MjM5MDIyfQ.SflKxwRJSMeKKF2QT4fwpMeJf36POk6yJV_adQssw5c").contains("[REDACTED]"));

        // PEM private keys
        let pem =
            "-----BEGIN RSA PRIVATE KEY-----\nMIIEowIBAAKCAQEA...\n-----END RSA PRIVATE KEY-----";
        assert!(redact_secrets(pem).contains("[REDACTED]"));

        // DB connection strings
        assert!(
            redact_secrets("postgres://user:super_secret_password@localhost:5432/db")
                .contains("[REDACTED]")
        );
        assert!(
            !redact_secrets("postgres://user:super_secret_password@localhost:5432/db")
                .contains("super_secret_password")
        );

        // AWS secret keys
        assert!(
            redact_secrets("aws_secret_access_key=1234567890123456789012345678901234567890")
                .contains("[REDACTED]")
        );
    }

    #[test]
    fn test_redact_secrets_idempotence() {
        let inputs = vec![
            "sk-ant-abc12345678901234567890",
            "xoxb-abc-123",
            "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIiwiaWF0IjoxNTE2MjM5MDIyfQ.SflKxwRJSMeKKF2QT4fwpMeJf36POk6yJV_adQssw5c",
            "-----BEGIN RSA PRIVATE KEY-----\nMIIEowIBAAKCAQEA...\n-----END RSA PRIVATE KEY-----",
            "postgres://user:super_secret_password@localhost:5432/db",
            "aws_secret_access_key=1234567890123456789012345678901234567890",
            "some ordinary string with no keys",
        ];
        for input in inputs {
            let once = redact_secrets(input);
            let twice = redact_secrets(&once);
            assert_eq!(once, twice, "Idempotence failed for input: {}", input);
        }
    }

    #[test]
    fn test_redact_secrets_cross_thread_determinism() {
        use std::thread;
        let input = "postgres://user:super_secret_password@localhost:5432/db and sk-ant-abc12345678901234567890";
        let expected = redact_secrets(input);

        let mut handles = Vec::new();
        for _ in 0..10 {
            let inp = input.to_string();
            let exp = expected.clone();
            handles.push(thread::spawn(move || {
                for _ in 0..1000 {
                    assert_eq!(redact_secrets(&inp), exp);
                }
            }));
        }
        for h in handles {
            h.join().unwrap();
        }
    }

    #[test]
    fn test_validate_shell_command_git_transport_bypass() {
        // ssh:// URL transport bypass (SV3-001)
        assert!(validate_shell_command("git clone ssh://user@evil.com/repo.git").is_err());
        // file:// URL bypass (SV3-002)
        assert!(validate_shell_command("git clone file:///etc/passwd /tmp/").is_err());
    }
}

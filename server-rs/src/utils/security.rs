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

use crate::error::AppError;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use std::path::{Path, PathBuf};

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
        .or_else(|_| std::env::var("ENV"))
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

fn is_ipv4_cgnat(ip: Ipv4Addr) -> bool {
    let octets = ip.octets();
    octets[0] == 100 && (64..=127).contains(&octets[1])
}

fn is_ipv6_unique_local(ip: Ipv6Addr) -> bool {
    (ip.segments()[0] & 0xfe00) == 0xfc00
}

fn is_ipv6_unicast_link_local(ip: Ipv6Addr) -> bool {
    (ip.segments()[0] & 0xffc0) == 0xfe80
}

/// Validates an agent/user-controlled outbound URL before the engine fetches it.
/// Blocks local, private, link-local, multicast, and metadata-style targets after DNS resolution.
pub async fn validate_public_http_url(url: &str) -> Result<(), AppError> {
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

    if let Ok(ip) = host.parse::<IpAddr>() {
        if is_public_routable_ip(ip) {
            return Ok(());
        }
        return Err(AppError::Forbidden(
            "Local, private, or reserved IP addresses cannot be fetched by agents".to_string(),
        ));
    }

    let port = parsed.port_or_known_default().ok_or_else(|| {
        AppError::BadRequest("URL must use a scheme with a known port".to_string())
    })?;

    let mut addrs = tokio::net::lookup_host((host, port))
        .await
        .map_err(|_| AppError::BadRequest("URL host could not be resolved".to_string()))?;

    let mut saw_addr = false;
    for addr in addrs.by_ref() {
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

    Ok(())
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
    if path_str.starts_with("/dev/") || path_str.starts_with("/proc/") {
        return true;
    }
    false
}

/// Sanitizes and validates a path to prevent directory traversal.
pub fn validate_path(base: &Path, user_path: &str) -> Result<SafePath, AppError> {
    let base_raw = if base.is_absolute() {
        base.to_path_buf()
    } else {
        std::env::current_dir()?.join(base)
    };

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

    if !result.starts_with(&base_abs) {
        return Err(AppError::Forbidden(
            "Path traversal detected: outside authorized base".to_string(),
        ));
    }

    if is_device_file(&result) {
        return Err(AppError::Forbidden(
            "Access denied: device files or system directories cannot be accessed".to_string(),
        ));
    }

    Ok(SafePath(result))
}

/// Strictly sanitizes a string to be used as a filename or ID.
pub fn sanitize_id(id: &str) -> String {
    id.chars()
        .filter(|c| c.is_alphanumeric() || *c == '_' || *c == '-')
        .collect()
}

/// Redacts sensitive information from strings.
pub fn redact_secrets(input: &str) -> String {
    use once_cell::sync::Lazy;
    use regex::{Regex, RegexSet};

    static PATTERNS: Lazy<(RegexSet, Vec<Regex>)> = Lazy::new(|| {
        let patterns = vec![
            r"(?i)bearer\s+[a-zA-Z0-9\-\._~+/]+=*",
            r"(?i)authorization:\s*[^\s,]+",
            r#"(?i)("?(?:api_key|secret|password|token|key|credential)"?\s*[:=]\s*)(["']?)(?:\\.|[^"'\s\n])*(["']?)"#,
            r"(?i)sk-[a-zA-Z0-9]{20,}",
            r"(?i)AIza[0-9A-Za-z-_]{30,}",
            r"(?i)ghp_[a-zA-Z0-9]{30,}",
            r"(?i)AKIA[0-9A-Z]{16}",
        ];
        let set = RegexSet::new(&patterns).expect("Security patterns must be valid regex.");
        let regexes = patterns.iter().map(|p| Regex::new(p).unwrap()).collect();
        (set, regexes)
    });

    let mut output = input.to_string();
    let (set, regexes) = &*PATTERNS;

    if set.is_match(&output) {
        for (idx, re) in regexes.iter().enumerate() {
            if set.matches(&output).matched(idx) {
                if idx == 2 {
                    output = re.replace_all(&output, r#"$1$2[REDACTED]$3"#).to_string();
                } else {
                    output = re.replace_all(&output, "[REDACTED]").to_string();
                }
            }
        }
    }
    output
}

/// Validates a shell command against a ZERO-TRUST whitelist.
pub fn validate_shell_command(command: &str) -> Result<(), AppError> {
    // 1. Strip shell comments (starting with #) prior to validation
    let command_no_comment = match command.split_once('#') {
        Some((code, _comment)) => code,
        None => command,
    };
    let lower = command_no_comment.to_lowercase();

    // 2. Block Command Substitution & Expansion (Critical Vulnerability)
    if lower.contains("$(") || lower.contains("`") || lower.contains("${") {
        return Err(AppError::Forbidden(
            "Command substitution or variable expansion detected".to_string(),
        ));
    }

    // 3. Block Piping, Chaining, and Input Redirection
    if lower.contains('|') || lower.contains('<') || lower.contains(';') || lower.contains('&') {
        return Err(AppError::Forbidden(
            "Piping, chaining, or input redirection prohibited".to_string(),
        ));
    }

    // 4. Harden Output Redirection: Only allow redirection to /dev/null, $null, or standard error/output descriptors
    if lower.contains('>') {
        let mut rest = lower.as_str();
        while let Some(idx) = rest.find('>') {
            let redirect_target = rest[idx + 1..].trim();
            if !redirect_target.starts_with("/dev/null")
                && !redirect_target.starts_with("$null")
                && !redirect_target.starts_with("&1")
                && !redirect_target.starts_with("&2")
            {
                return Err(AppError::Forbidden(
                    "Output redirection is only permitted to /dev/null or $null".to_string(),
                ));
            }
            rest = &rest[idx + 1..];
        }
    }

    // 5. Block wildcard character glob expansion to prevent sandbox blacklist bypasses
    if lower.contains('*') || lower.contains('?') || lower.contains('[') || lower.contains(']') {
        return Err(AppError::Forbidden(
            "Wildcard characters (*, ?, []) are prohibited in commands to prevent sandbox bypass"
                .to_string(),
        ));
    }

    // 6. Whitelist of Allowed Base Commands
    let allowed_commands = [
        "ls", "cd", "pwd", "cat", "echo", "grep", "find", "cargo", "npm", "git", "python", "node",
        "rustc", "mkdir", "cp", "mv", "touch", "test",
    ];

    let first_word = lower.split_whitespace().next().unwrap_or("");
    if !allowed_commands.contains(&first_word) {
        return Err(AppError::Forbidden(format!(
            "Command '{}' is not in the authorized whitelist",
            first_word
        )));
    }

    // 7. Command-specific option restrictions to prevent sub-shell escapes
    if first_word == "git" {
        let dangerous_git = [
            "-c",
            "--git-dir",
            "--work-tree",
            "core.pager",
            "config",
            "exec",
            "upload-pack",
            "receive-pack",
        ];
        for flag in dangerous_git {
            if lower.contains(flag) {
                return Err(AppError::Forbidden(format!(
                    "Unauthorized git parameter or option detected: '{}'",
                    flag
                )));
            }
        }
    } else if first_word == "npm" {
        let dangerous_npm = ["run", "exec", "install", "i", "config"];
        for flag in dangerous_npm {
            if lower.contains(flag) {
                return Err(AppError::Forbidden(format!(
                    "Unauthorized npm parameter or option detected: '{}'",
                    flag
                )));
            }
        }
    } else if first_word == "cargo" {
        let dangerous_cargo = ["run", "test", "bench"];
        for flag in dangerous_cargo {
            if lower.contains(flag) {
                return Err(AppError::Forbidden(format!(
                    "Unauthorized cargo command detected: '{}'",
                    flag
                )));
            }
        }
    }

    // 8. Blacklist specific dangerous flags/paths for allowed commands
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
        if lower.contains(flag) {
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
        let base = Path::new("/tmp/base");
        assert!(validate_path(base, "../outside").is_err());
        assert!(validate_path(base, "user1/../../outside").is_err());

        // Windows device names & Unix device folders
        assert!(validate_path(base, "con").is_err());
        assert!(validate_path(base, "PRN.txt").is_err());
        assert!(validate_path(base, "subdir/aux").is_err());
        assert!(validate_path(base, "/dev/stdout").is_err());
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
}

// Metadata: [security]

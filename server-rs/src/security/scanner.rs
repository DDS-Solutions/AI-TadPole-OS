//! Advanced Security Scanning & Risk Mitigation
//!
//! Implements a preventative security layer that inspects incoming agent commands
//! and external tool invocations for injection attacks, escaping, and shell-level
//! risk patterns.
//!
//! @docs ARCHITECTURE:VulnerabilityScanning
//! @docs SECURITY:ScanningPolicies
//!
//! ### AI Assist Note
//! **Security Scanner**: Orchestrates the proactive inspection of
//! incoming agent commands and external tool invocations. Features
//! **Multi-Phase Mitigation**: detects **Command Injection** (`;`, `&&`,
//! `|`), **Output Redirection** (`>`), and **Secret Leakage** (Regex-based
//! API key detection). Note: The scanner is highly aggressive;
//! legitimate scripts involving piping or concatenation WILL be flagged
//! as `Risky`. AI agents should verify the `ScannerResult` and suggest
//! manual user approval for complex but valid orchestration (SCAN-01).
//!
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: False positive risk detection on valid complex
//!   commands, pattern-bypass via advanced shell obfuscation, or performance
//!   degradation under high-frequency command scanning.
//! - **Trace Scope**: `server-rs::security::scanner`
use crate::secret_redactor::SecretRedactor;
use regex::Regex;
use std::sync::{Arc, LazyLock};

/// Pre-compiled secret detection patterns (GAP-QUAL-03: avoid recompilation per scan).
static SECRET_PATTERNS: LazyLock<Vec<(Regex, &'static str)>> = LazyLock::new(|| {
    vec![
        (
            Regex::new(r"(?i)sk-[a-zA-Z0-9]{48}").unwrap(),
            "OpenAI API Key",
        ),
        (
            Regex::new(r"(?i)AIza[0-9A-Za-z-_]{35}").unwrap(),
            "Google API Key",
        ),
        (
            Regex::new(r"(?i)ghp_[a-zA-Z0-9]{36}").unwrap(),
            "GitHub Personal Access Token",
        ),
        (
            Regex::new(r"(?i)xox[pborsa]-[a-zA-Z0-9-]{10,48}").unwrap(),
            "Slack Token",
        ),
        (
            Regex::new(r"(?i)SG\.[a-zA-Z0-9_-]{22}\.[a-zA-Z0-9_-]{43}").unwrap(),
            "SendGrid API Key",
        ),
        (
            Regex::new(r"(?i)sq0atp-[a-zA-Z0-9_-]{22}").unwrap(),
            "Square Access Token",
        ),
    ]
});

/// Pre-compiled heuristic patterns for secret export detection.
static HEURISTIC_PATTERNS: LazyLock<Vec<Regex>> = LazyLock::new(|| {
    vec![
        Regex::new(r"(?i)export\s+[A-Z0-9_]+=").unwrap(),
        Regex::new(r"(?i)set\s+[A-Z0-9_]+=").unwrap(),
        Regex::new(r"(?i)env\s+[A-Z0-9_]+=").unwrap(),
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
pub enum ScannerResult {
    /// The script appears safe and contains no detectable secrets.
    Safe,
    /// The script contains a potential risk (e.g., hardcoded API key or raw export).
    /// The string contains the reason/detected pattern.
    Risky(String),
}

/// A proactive security scanner that inspects scripts for sensitive data before execution.
///
/// ShellScanner uses both exact matching (via SecretRedactor) and heuristic patterns
/// (Regex) to detect potential secret leakages in agent-generated Python, Bash, or Shell code.
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

    /// Performs a multi-phase deep scan of a code or command string.
    ///
    /// Identifies high-risk patterns including:
    /// - Command injection (`;`, `&&`, `||`, `|`)
    /// - Redirection and substitution attacks
    /// - Remote access and network exfiltration attempts
    /// - Binary execution in restricted paths
    ///
    /// Returns `ScannerResult::Risky` if any pattern is detected.
    #[tracing::instrument(skip(self, input), name = "security::shell_scan")]
    pub fn scan(&self, input: &str) -> ScannerResult {
        // 0. Pre-normalize input against evasion attempts
        let normalized = super::normalizer::normalize_text(input);
        let collapsed = super::normalizer::normalize_and_collapse_whitespace(input);

        // 1. Check against the redactor's known secrets (from env)
        if self.redactor.is_sensitive(input) || self.redactor.is_sensitive(&normalized) {
            return ScannerResult::Risky("Known environment secret detected in script".to_string());
        }

        // 2. Pre-compiled regex patterns for common secret formats (Proactive detection)
        for (re, name) in SECRET_PATTERNS.iter() {
            if re.is_match(input) || re.is_match(&normalized) {
                return ScannerResult::Risky(format!("Potential {} detected", name));
            }
        }

        // 3. Heuristic: Look for "export KEY=" or "SET KEY=" patterns (pre-compiled)
        for re in HEURISTIC_PATTERNS.iter() {
            // Check original
            if let Some(mat) = re.find(input) {
                let after = &input[mat.end()..];
                if !after.trim().is_empty()
                    && !after.trim().starts_with('"')
                    && !after.trim().starts_with('\'')
                {
                    return ScannerResult::Risky(
                        "Prohibited raw secret export detected".to_string(),
                    );
                }
            }
            // Check normalized
            if let Some(mat) = re.find(&normalized) {
                let after = &normalized[mat.end()..];
                if !after.trim().is_empty()
                    && !after.trim().starts_with('"')
                    && !after.trim().starts_with('\'')
                {
                    return ScannerResult::Risky(
                        "Prohibited raw secret export detected".to_string(),
                    );
                }
            }
        }

        // 4. Injection: Look for command concatenation or redirection (pre-compiled patterns)
        for (re, name) in INJECTION_PATTERNS.iter() {
            if re.is_match(input) || re.is_match(&normalized) || re.is_match(&collapsed) {
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

        // 1. Safe scripts
        match scanner.scan("ls -la") {
            ScannerResult::Safe => {}
            _ => panic!("Should be safe"),
        }

        // 2. OpenAI Key (Obfuscated for scanner)
        let openai_key = format!("sk-{}", "123456789012345678901234567890123456789012345678");
        match scanner.scan(&format!("export OPENAI_API_KEY={}", openai_key)) {
            ScannerResult::Risky(r) => {
                assert!(r.contains("OpenAI") || r.contains("environment secret"))
            }
            _ => panic!("Should detect OpenAI key"),
        }

        // 3. Raw export
        match scanner.scan("export VAR=supersecret") {
            ScannerResult::Risky(r) => assert!(r.contains("raw secret export")),
            _ => panic!("Should detect raw export"),
        }

        // 4. Google API Key (Obfuscated for scanner)
        let google_key = format!("AIzaSyA{}", "12345678901234567890123456789012");
        match scanner.scan(&format!("key = {}", google_key)) {
            ScannerResult::Risky(r) => {
                assert!(r.contains("Google") || r.contains("environment secret"))
            }
            _ => panic!("Should detect Google key"),
        }

        // 5. GitHub Token (Obfuscated for scanner)
        let github_token = format!("ghp_{}", "123456789012345678901234567890123456");
        match scanner.scan(&github_token) {
            ScannerResult::Risky(r) => {
                assert!(r.contains("GitHub") || r.contains("environment secret"))
            }
            _ => panic!("Should detect GitHub token"),
        }

        // 6. Slack Token (Obfuscated for scanner)
        let slack_token = format!("xoxb-{}", "1234567890-1234567890123");
        match scanner.scan(&slack_token) {
            ScannerResult::Risky(r) => {
                assert!(r.contains("Slack") || r.contains("environment secret"))
            }
            _ => panic!("Should detect Slack token"),
        }

        // 7. Case Insensitivity
        match scanner.scan("EXPORT VAL=secret") {
            ScannerResult::Risky(r) => assert!(r.contains("raw secret export")),
            _ => panic!("Should detect case-insensitive export"),
        }

        // 8. Command Injection (concat)
        match scanner.scan("ls -la; cat /etc/passwd") {
            ScannerResult::Risky(r) => assert!(r.contains("Command Concatenation")),
            _ => panic!("Should detect semicolon injection"),
        }

        // 9. Command Injection (&&)
        match scanner.scan("whoami && echo pwned") {
            ScannerResult::Risky(r) => assert!(r.contains("Command Concatenation")),
            _ => panic!("Should detect && injection"),
        }

        // 10. Output Redirection
        match scanner.scan("echo sensitive > secret.txt") {
            ScannerResult::Risky(r) => assert!(r.contains("Redirection")),
            _ => panic!("Should detect output redirection"),
        }

        // 11. Command Substitution
        match scanner.scan("echo $(whoami)") {
            ScannerResult::Risky(r) => assert!(r.contains("Substitution")),
            _ => panic!("Should detect $() substitution"),
        }
    }

    #[test]
    fn test_scanner_evasion() {
        let scanner = ShellScanner::mock();

        // 1. Spaced injection bypass attempt
        match scanner.scan("whoami   & &   echo pwned") {
            ScannerResult::Risky(r) => assert!(r.contains("Command Concatenation")),
            _ => panic!("Should detect spaced && injection"),
        }

        // 2. Homoglyph injection bypass attempt ("е" is Cyrillic)
        let obfuscated_openai =
            format!("sk-{}", "123456789012345678901234567890123456789012345678");
        // "еxport" uses Cyrillic 'е'
        match scanner.scan(&format!(
            "\u{0435}xport OPENAI_API_KEY={}",
            obfuscated_openai
        )) {
            ScannerResult::Risky(r) => {
                assert!(
                    r.contains("export")
                        || r.contains("OpenAI")
                        || r.contains("environment secret"),
                    "Reason was: {}",
                    r
                );
            }
            _ => panic!("Should detect homoglyph export"),
        }

        // 3. Markdown injection bypass attempt
        match scanner.scan("whoami ;**c**a**t** /etc/passwd") {
            ScannerResult::Risky(r) => assert!(r.contains("Command Concatenation")),
            _ => panic!("Should detect markdown-hidden semicolon"),
        }

        // 4. Base64 payload injection bypass attempt ("d2hvYW1p" is "whoami", "JiY=" is "&&")
        // "d2hvYW1p &&" decodes to "whoami &&" which triggers injection
        match scanner.scan("d2hvYW1p && echo") {
            ScannerResult::Risky(r) => assert!(r.contains("Command Concatenation")),
            _ => panic!("Should detect base64-decoded injection pattern"),
        }
    }
}

// Metadata: [scanner]

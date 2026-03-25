//! Runtime log redaction for sensitive values.
//!
//! Holds a set of known secret strings and replaces any occurrence in text
//! with `[REDACTED]`. Applied to error messages, system broadcasts, and
//! mission logs to prevent accidental key exposure in the UI or trace stream.

use std::sync::Arc;

/// Minimum length for a secret to be registered. Very short strings
/// (like "sk") would cause excessive false-positive redactions.
const MIN_SECRET_LEN: usize = 8;

/// Thread-safe secret redactor that holds known sensitive values.
#[derive(Debug, Clone)]
pub struct SecretRedactor {
    /// The actual secret values to scan for (never logged).
    secrets: Arc<Vec<String>>,
}

impl SecretRedactor {
    /// Build a redactor from the current environment.
    /// Only registers non-empty values from known sensitive env vars.
    pub fn from_env() -> Self {
        let sensitive_vars = [
            "NEURAL_TOKEN",
            "GOOGLE_API_KEY",
            "GROQ_API_KEY",
            "OPENAI_API_KEY",
            "ANTHROPIC_API_KEY",
            "INCEPTION_API_KEY",
            "DEEPSEEK_API_KEY",
            "DISCORD_WEBHOOK",
        ];

        let secrets: Vec<String> = sensitive_vars
            .iter()
            .filter_map(|var| std::env::var(var).ok())
            .filter(|val| val.len() >= MIN_SECRET_LEN)
            .collect();

        tracing::info!(
            "🔐 [SecretRedactor] Initialized with {} registered secret(s).",
            secrets.len()
        );

        Self {
            secrets: Arc::new(secrets),
        }
    }

    /// Returns a new copy of the input where any known secret is replaced
    /// with `[REDACTED]`. Performs a simple substring scan.
    pub fn redact(&self, text: &str) -> String {
        let mut result = text.to_string();
        for secret in self.secrets.iter() {
            if result.contains(secret.as_str()) {
                result = result.replace(secret.as_str(), "[REDACTED]");
            }
        }
        result
    }

    /// Returns true if the redactor has any secrets registered.
    #[allow(dead_code)]
    pub fn is_active(&self) -> bool {
        !self.secrets.is_empty()
    }

    /// Checks if a string contains any of the registered secrets.
    /// Used for proactive safety scanning before execution or logging.
    pub fn is_sensitive(&self, text: &str) -> bool {
        for secret in self.secrets.iter() {
            if text.contains(secret.as_str()) {
                return true;
            }
        }
        false
    }

    /// Creates a no-op redactor for testing.
    pub fn noop() -> Self {
        Self {
            secrets: Arc::new(Vec::new()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_redact_known_secret() {
        let redactor = SecretRedactor {
            secrets: Arc::new(vec!["sk-abcdef123456789".to_string()]),
        };
        let input = "Error: Invalid API Key: sk-abcdef123456789";
        let output = redactor.redact(input);
        assert_eq!(output, "Error: Invalid API Key: [REDACTED]");
        assert!(!output.contains("sk-abcdef"));
    }

    #[test]
    fn test_no_false_positive() {
        let redactor = SecretRedactor {
            secrets: Arc::new(vec!["my-secret-key-12345".to_string()]),
        };
        let input = "Normal log message with no secrets";
        assert_eq!(redactor.redact(input), input);
    }

    #[test]
    fn test_multiple_secrets() {
        let redactor = SecretRedactor {
            secrets: Arc::new(vec![
                "secret-one-12345678".to_string(),
                "secret-two-87654321".to_string(),
            ]),
        };
        let input = "Key1: secret-one-12345678, Key2: secret-two-87654321";
        let output = redactor.redact(input);
        assert_eq!(output, "Key1: [REDACTED], Key2: [REDACTED]");
    }

    #[test]
    fn test_noop_redactor() {
        let redactor = SecretRedactor::noop();
        assert!(!redactor.is_active());
        let input = "anything goes";
        assert_eq!(redactor.redact(input), input);
    }
}

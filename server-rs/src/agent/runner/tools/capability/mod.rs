//! @docs ARCHITECTURE:Security
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Agent Runner / mod
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Type-safe state handling and bounded execution without unhandled panics.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: `[capability]`
//! - **Witness Tests**: none declared

pub mod crypto;
pub mod guard;
pub mod paths;
pub mod types;

#[cfg(test)]
#[allow(unused_imports)]
pub use crypto::reset_keyring_for_test;
#[allow(unused_imports)]
pub use crypto::{
    canonical_a2a_message, compute_hmac, set_active_key_id, set_key, sign_a2a_canonical,
    sign_a2a_envelope, verify_a2a_canonical, verify_a2a_envelope,
};
#[allow(unused_imports)]
pub use guard::{is_revoked, revoke_token, revoke_token_with_expiry, ZeroTrustGuard};
#[allow(unused_imports)]
pub use paths::resolve_executable_path;
pub use types::{CapabilityToken, Permission};

/// Logs capability subsystem status.
#[allow(dead_code)]
pub fn log_capability_status() {
    tracing::debug!("[capability] Capability subsystem ready with ZeroTrustGuard");
}

#[cfg(test)]
mod tests {
    use super::crypto::{compute_signature, set_key, KEYRING};
    use super::guard::{revoke_token, ZeroTrustGuard};
    use super::paths::{clear_regex_cache, matches_glob, sanitize_allowed_pattern};
    use super::types::{CapabilityToken, Permission};
    use std::path::Path;

    struct TestCleanupGuard {
        keys_to_remove: Vec<String>,
        tokens_to_unrevoke: Vec<String>,
    }

    impl TestCleanupGuard {
        fn new() -> Self {
            Self {
                keys_to_remove: Vec::new(),
                tokens_to_unrevoke: Vec::new(),
            }
        }

        fn register_key(&mut self, key_id: &str) {
            self.keys_to_remove.push(key_id.to_string());
        }

        fn register_token(&mut self, token_id: &str) {
            self.tokens_to_unrevoke.push(token_id.to_string());
        }
    }

    impl Drop for TestCleanupGuard {
        fn drop(&mut self) {
            if !self.keys_to_remove.is_empty() {
                let mut keyring = KEYRING.lock();
                for key_id in &self.keys_to_remove {
                    keyring.keys.remove(key_id);
                }
            }
            for token_id in &self.tokens_to_unrevoke {
                super::guard::unrevoke_token(token_id);
            }
            clear_regex_cache();
        }
    }

    #[test]
    fn test_canonical_path_matching() {
        clear_regex_cache();

        assert!(matches_glob("/base/src/main.rs", "/base/src/main.rs"));

        // Path prefix matching: /base matches children under /base
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

        // Question mark matching
        assert!(matches_glob("/base/src/main?.rs", "/base/src/main1.rs"));
        assert!(!matches_glob("/base/src/main?.rs", "/base/src/main/.rs"));
    }

    #[test]
    fn test_path_containment_bypass_rejected() {
        let root = Path::new("/base");

        // Containment check must strictly prevent prefix overlap without slash boundary
        assert_eq!(sanitize_allowed_pattern("/basevil/secret.txt", root), None);
        assert_eq!(sanitize_allowed_pattern("/base-backup/data", root), None);
        assert_eq!(
            sanitize_allowed_pattern("/base/src/main.rs", root),
            Some("/base/src/main.rs".to_string())
        );
    }

    #[test]
    fn test_dot_universal_grant_removed() {
        // Pattern "." must not act as universal wildcard
        assert!(!matches_glob(".", "/etc/passwd"));
        assert!(!matches_glob(".", "/base/src/main.rs"));
    }

    #[test]
    fn test_target_traversal_rejected() {
        assert!(!matches_glob("/base", "/base/../etc/passwd"));
        assert!(!matches_glob("/base/src", "/base/src/../../secret"));
    }

    #[test]
    fn test_sanitize_allowed_pattern() {
        let root = Path::new("/base");

        // Traversal attempts must be rejected
        assert_eq!(sanitize_allowed_pattern("../../etc/passwd", root), None);
        assert_eq!(sanitize_allowed_pattern("src/../../etc/passwd", root), None);

        // Valid relative patterns are resolved absolute
        assert_eq!(
            sanitize_allowed_pattern("src/**/*.rs", root),
            Some("/base/src/**/*.rs".to_string())
        );
    }

    #[test]
    fn test_token_verify_allowed_files() {
        clear_regex_cache();

        let root = Path::new("/base");
        let allowed = vec!["src/**/*.rs".to_string(), "Cargo.toml".to_string()];
        let token = ZeroTrustGuard::mint_token(
            "worker",
            "mission",
            crate::agent::types::RoleAuthorityLevel::Specialist,
            Some(&allowed),
            None,
            root,
        );

        // Verify safe tool execution
        assert!(token.verify(&Permission::ToolExec("update_working_memory".to_string())));

        // Reads are allowed globally in the workspace
        assert!(token.verify(&Permission::FileRead("/base/tests/main.rs".to_string())));

        // Writes inside allowed patterns are permitted
        assert!(token.verify(&Permission::FileWrite("/base/src/main.rs".to_string())));
        assert!(token.verify(&Permission::FileWrite("/base/Cargo.toml".to_string())));

        // Writes outside allowed patterns are blocked
        assert!(!token.verify(&Permission::FileWrite("/base/tests/main.rs".to_string())));
    }

    #[test]
    fn test_empty_allowed_files_least_privilege() {
        let root = Path::new("/base");
        let empty_allowed: Vec<String> = vec![];
        let token = ZeroTrustGuard::mint_token(
            "worker",
            "mission",
            crate::agent::types::RoleAuthorityLevel::Specialist,
            Some(&empty_allowed),
            None,
            root,
        );

        // With empty allowed_files slice, no write permissions should be granted
        assert!(!token.verify(&Permission::FileWrite("/base/src/main.rs".to_string())));
        assert!(!token.verify(&Permission::FileWrite("/base/Cargo.toml".to_string())));
    }

    #[test]
    fn test_token_signature_tampering() {
        let root = Path::new("/base");
        let original_token = ZeroTrustGuard::mint_token(
            "worker",
            "mission",
            crate::agent::types::RoleAuthorityLevel::Specialist,
            None,
            None,
            root,
        );

        // Verification initially passes
        assert!(original_token.verify(&Permission::ToolExec("update_working_memory".to_string())));

        // 1. Tamper with permissions
        let mut t1 = original_token.clone();
        t1.permissions.push(Permission::SpawnAgent);
        assert!(!t1.verify(&Permission::ToolExec("update_working_memory".to_string())));

        // 2. Tamper with expiration
        let mut t2 = original_token.clone();
        t2.expires_at = chrono::Utc::now() - chrono::Duration::hours(1);
        assert!(!t2.verify(&Permission::ToolExec("update_working_memory".to_string())));

        // 3. Tamper with agent ID
        let mut t3 = original_token.clone();
        t3.agent_id = "hacker".to_string();
        assert!(!t3.verify(&Permission::ToolExec("update_working_memory".to_string())));
    }

    #[test]
    fn test_token_serialization() {
        let root = Path::new("/base");
        let token = ZeroTrustGuard::mint_token(
            "worker",
            "mission",
            crate::agent::types::RoleAuthorityLevel::Specialist,
            None,
            None,
            root,
        );

        let json = serde_json::to_string(&token).unwrap();
        let deserialized: CapabilityToken = serde_json::from_str(&json).unwrap();

        assert_eq!(token, deserialized);
        assert!(deserialized.verify(&Permission::ToolExec("update_working_memory".to_string())));
    }

    #[test]
    fn test_token_revocation() {
        let mut guard = TestCleanupGuard::new();
        let root = Path::new("/base");
        let token = ZeroTrustGuard::mint_token(
            "worker",
            "mission",
            crate::agent::types::RoleAuthorityLevel::Specialist,
            None,
            None,
            root,
        );

        guard.register_token(&token.id);

        assert!(token.verify(&Permission::ToolExec("update_working_memory".to_string())));

        // Revoke token
        revoke_token(&token.id);

        // Verification must now fail
        assert!(!token.verify(&Permission::ToolExec("update_working_memory".to_string())));
    }

    #[test]
    fn test_wildcard_toolexec_rejected() {
        let root = Path::new("/base");
        let token = ZeroTrustGuard::mint_token(
            "worker",
            "mission",
            crate::agent::types::RoleAuthorityLevel::Specialist,
            None,
            None,
            root,
        );

        // Explicit safe tools are permitted
        assert!(token.verify(&Permission::ToolExec("update_working_memory".to_string())));
        assert!(token.verify(&Permission::ToolExec("store_knowledge".to_string())));

        // Wildcard ToolExec("*") is strictly rejected by ZeroTrust verify
        assert!(!token.verify(&Permission::ToolExec("*".to_string())));
    }

    #[test]
    fn test_shell_execute_matching() {
        let root = Path::new("/base");
        let token = ZeroTrustGuard::mint_token(
            "worker",
            "mission",
            crate::agent::types::RoleAuthorityLevel::Executive,
            None,
            None,
            root,
        );

        // Executing cargo build should be allowed since "cargo" was granted
        assert!(token.verify(&Permission::ShellExecute(
            "cargo build --release".to_string()
        )));
        // Executing npm test should be allowed since "npm" was granted
        assert!(token.verify(&Permission::ShellExecute("npm test".to_string())));
        // Executing other commands should be blocked
        assert!(!token.verify(&Permission::ShellExecute("rm -rf /".to_string())));
    }

    #[test]
    fn test_shell_execute_quoted_path_matching() {
        let root = Path::new("/base");
        let token = ZeroTrustGuard::mint_token(
            "worker",
            "mission",
            crate::agent::types::RoleAuthorityLevel::Executive,
            None,
            None,
            root,
        );

        // Quoted command lines must parse executable cleanly
        assert!(token.verify(&Permission::ShellExecute(
            "\"cargo\" build --release".to_string()
        )));
        assert!(token.verify(&Permission::ShellExecute("npm \"test\"".to_string())));
    }

    #[test]
    fn test_network_fetch_matching() {
        let mut guard = TestCleanupGuard::new();
        let permissions = vec![Permission::NetworkFetch(
            "https://api.github.com/*".to_string(),
        )];
        let id = "test".to_string();
        let agent_id = "test".to_string();
        let mission_id = "test".to_string();
        let expires_at = chrono::Utc::now() + chrono::Duration::hours(1);
        let key = [0u8; 32];
        let signature =
            compute_signature(&id, &agent_id, &mission_id, &permissions, &expires_at, &key);

        let token = CapabilityToken {
            id,
            agent_id,
            mission_id,
            permissions,
            expires_at,
            key_id: "test_key".to_string(),
            signature,
        };

        set_key("test_key", key);
        guard.register_key("test_key");

        assert!(token.verify(&Permission::NetworkFetch(
            "https://api.github.com/repos".to_string()
        )));
        assert!(!token.verify(&Permission::NetworkFetch("https://google.com".to_string())));
    }

    #[test]
    fn test_a2a_envelope_sign_verify_and_tamper() {
        let envelope = r#"{"from":"agent_a","to":"agent_b","payload":"secure_msg"}"#;
        let header = super::crypto::sign_a2a_envelope(envelope);
        assert!(super::crypto::verify_a2a_envelope(envelope, &header));

        // Tamper payload
        let tampered = r#"{"from":"agent_a","to":"agent_b","payload":"injected_msg"}"#;
        assert!(!super::crypto::verify_a2a_envelope(tampered, &header));

        // Tamper signature header
        assert!(!super::crypto::verify_a2a_envelope(
            envelope,
            "dev:0000000000000000"
        ));
        assert!(!super::crypto::verify_a2a_envelope(
            envelope,
            "invalid_header"
        ));
    }

    #[test]
    fn test_working_memory_permission_granted() {
        let root = Path::new("/base");
        let token = ZeroTrustGuard::mint_token(
            "agent_1",
            "mission_1",
            crate::agent::types::RoleAuthorityLevel::Specialist,
            None,
            None,
            root,
        );

        assert!(token.verify(&Permission::ToolExec("update_working_memory".to_string())));
        assert!(token.verify(&Permission::ToolExec("share_finding".to_string())));
        assert!(token.verify(&Permission::ToolExec("complete_mission".to_string())));
    }
}

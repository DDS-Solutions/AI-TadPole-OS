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
use rand::Rng;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet};
use std::num::NonZeroUsize;
use std::sync::Arc;
use subtle::ConstantTimeEq;

static REGEX_CACHE: Lazy<Mutex<LruCache<String, Arc<regex::Regex>>>> =
    Lazy::new(|| Mutex::new(LruCache::new(NonZeroUsize::new(128).unwrap())));

struct Keyring {
    keys: HashMap<String, [u8; 32]>,
    active_key_id: String,
}

static KEYRING: Lazy<Mutex<Keyring>> = Lazy::new(|| {
    let mut keys = HashMap::new();

    let curr = std::env::var("CAPABILITY_KEY_CURR").ok();
    let prev = std::env::var("CAPABILITY_KEY_PREV").ok();

    let active_key_id = if let Some(curr_val) = curr {
        if let Ok(key_bytes) = hex::decode(&curr_val) {
            if key_bytes.len() == 32 {
                let mut key = [0u8; 32];
                key.copy_from_slice(&key_bytes);
                keys.insert("curr".to_string(), key);
                "curr".to_string()
            } else {
                panic!("CAPABILITY_KEY_CURR must be a 32-byte hex string (64 characters)");
            }
        } else {
            panic!("CAPABILITY_KEY_CURR is not a valid hex string");
        }
    } else {
        // Fallback to random key for development/testing
        let mut key = [0u8; 32];
        rand::rng().fill_bytes(&mut key);
        keys.insert("dev".to_string(), key);
        "dev".to_string()
    };

    if let Some(prev_val) = prev {
        if let Ok(key_bytes) = hex::decode(&prev_val) {
            if key_bytes.len() == 32 {
                let mut key = [0u8; 32];
                key.copy_from_slice(&key_bytes);
                keys.insert("prev".to_string(), key);
            }
        }
    }

    Mutex::new(Keyring {
        keys,
        active_key_id,
    })
});

static REVOCATION_LIST: Lazy<Mutex<HashSet<String>>> = Lazy::new(|| Mutex::new(HashSet::new()));

/// Revokes a capability token by ID.
#[allow(dead_code)]
pub fn revoke_token(id: &str) {
    tracing::warn!("[capability] REVOKED: token_id={}", id);
    REVOCATION_LIST.lock().insert(id.to_string());
}

/// Checks if a capability token has been revoked.
pub fn is_revoked(id: &str) -> bool {
    REVOCATION_LIST.lock().contains(id)
}

/// Adds or updates a signing key in the keyring.
#[allow(dead_code)]
pub fn set_key(id: &str, key: [u8; 32]) {
    let mut keyring = KEYRING.lock();
    keyring.keys.insert(id.to_string(), key);
}

/// Sets the active key ID for future mints.
#[allow(dead_code)]
pub fn set_active_key_id(id: &str) -> Result<(), String> {
    let mut keyring = KEYRING.lock();
    if keyring.keys.contains_key(id) {
        keyring.active_key_id = id.to_string();
        Ok(())
    } else {
        Err(format!("Key ID '{}' not found in keyring", id))
    }
}

/// Resets the keyring back to its environment variable configuration.
#[cfg(test)]
#[allow(dead_code)]
pub fn reset_keyring_for_test() {
    let mut keyring = KEYRING.lock();
    keyring.keys.clear();

    let curr = std::env::var("CAPABILITY_KEY_CURR").ok();
    let prev = std::env::var("CAPABILITY_KEY_PREV").ok();

    keyring.active_key_id = if let Some(curr_val) = curr {
        if let Ok(key_bytes) = hex::decode(&curr_val) {
            if key_bytes.len() == 32 {
                let mut key = [0u8; 32];
                key.copy_from_slice(&key_bytes);
                keyring.keys.insert("curr".to_string(), key);
                "curr".to_string()
            } else {
                "dev".to_string()
            }
        } else {
            "dev".to_string()
        }
    } else {
        let mut key = [0u8; 32];
        let mut rng = rand::rng();
        rng.fill_bytes(&mut key);
        keyring.keys.insert("dev".to_string(), key);
        "dev".to_string()
    };

    if let Some(prev_val) = prev {
        if let Ok(key_bytes) = hex::decode(&prev_val) {
            if key_bytes.len() == 32 {
                let mut key = [0u8; 32];
                key.copy_from_slice(&key_bytes);
                keyring.keys.insert("prev".to_string(), key);
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
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
    /// Grants permission to execute a tool by name without filesystem/network access.
    /// While the capability token explicitly checks and grants this permission (supporting wildcards like `*`),
    /// security is also enforced by the CBS skill-gate and oversight.
    ToolExec(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CapabilityToken {
    pub id: String,
    pub agent_id: String,
    pub mission_id: String,
    pub permissions: Vec<Permission>,
    pub expires_at: chrono::DateTime<chrono::Utc>,
    pub key_id: String,
    pub signature: Vec<u8>,
}

fn encode_permission_canonical(permission: &Permission, buf: &mut Vec<u8>) {
    match permission {
        Permission::FileRead(s) => {
            buf.push(0);
            buf.extend_from_slice(&(s.len() as u32).to_le_bytes());
            buf.extend_from_slice(s.as_bytes());
        }
        Permission::FileWrite(s) => {
            buf.push(1);
            buf.extend_from_slice(&(s.len() as u32).to_le_bytes());
            buf.extend_from_slice(s.as_bytes());
        }
        Permission::ShellExecute(s) => {
            buf.push(2);
            buf.extend_from_slice(&(s.len() as u32).to_le_bytes());
            buf.extend_from_slice(s.as_bytes());
        }
        Permission::SpawnAgent => {
            buf.push(3);
        }
        Permission::NetworkFetch(s) => {
            buf.push(4);
            buf.extend_from_slice(&(s.len() as u32).to_le_bytes());
            buf.extend_from_slice(s.as_bytes());
        }
        Permission::ToolExec(s) => {
            buf.push(5);
            buf.extend_from_slice(&(s.len() as u32).to_le_bytes());
            buf.extend_from_slice(s.as_bytes());
        }
    }
}

fn canonical_message(
    id: &str,
    agent_id: &str,
    mission_id: &str,
    permissions: &[Permission],
    expires_at: i64,
) -> Vec<u8> {
    let mut buf = Vec::new();
    buf.extend_from_slice(b"V1\0"); // version tag
    buf.extend_from_slice(&(id.len() as u32).to_le_bytes());
    buf.extend_from_slice(id.as_bytes());
    buf.extend_from_slice(&(agent_id.len() as u32).to_le_bytes());
    buf.extend_from_slice(agent_id.as_bytes());
    buf.extend_from_slice(&(mission_id.len() as u32).to_le_bytes());
    buf.extend_from_slice(mission_id.as_bytes());
    buf.extend_from_slice(&expires_at.to_le_bytes());
    buf.extend_from_slice(&(permissions.len() as u32).to_le_bytes());
    for p in permissions {
        encode_permission_canonical(p, &mut buf);
    }
    buf
}

fn compute_signature(
    id: &str,
    agent_id: &str,
    mission_id: &str,
    permissions: &[Permission],
    expires_at: &chrono::DateTime<chrono::Utc>,
    key: &[u8; 32],
) -> Vec<u8> {
    let mut ipad = [0x36u8; 64];
    let mut opad = [0x5cu8; 64];
    for i in 0..32 {
        ipad[i] ^= key[i];
        opad[i] ^= key[i];
    }

    let msg = canonical_message(
        id,
        agent_id,
        mission_id,
        permissions,
        expires_at.timestamp(),
    );

    let mut inner_hasher = Sha256::new();
    inner_hasher.update(ipad);
    inner_hasher.update(&msg);
    let inner_hash = inner_hasher.finalize();

    let mut outer_hasher = Sha256::new();
    outer_hasher.update(opad);
    outer_hasher.update(inner_hash);
    outer_hasher.finalize().to_vec()
}

/// Checks if the target string matches the glob pattern.
///
/// ### Prefix Matching Behavior
/// For filesystem path matching (e.g. `FileRead` and `FileWrite`), this matcher appends
/// a suffix pattern `(/.*)?$` to the compiled regex. This ensures that directory-level
/// permissions recursively cover all children and subdirectories within that path.
///
/// For example, a granted permission for `/base` will match `/base/src/main.rs` and
/// `/base/Cargo.toml`.
///
/// While a specific file permission like `/base/Cargo.toml` could technically match
/// a hypothetical path `/base/Cargo.toml/subfile` due to the suffix, this does not
/// compromise security because:
/// 1. Any actual access targets are canonicalized absolute paths that correspond to
///    existing entities or paths inside the allowed workspace directory.
/// 2. OS-level filesystem verification will block attempting to treat a regular file
///    like `Cargo.toml` as a directory path segment.
fn matches_glob(pattern: &str, target: &str) -> bool {
    let mut pattern_norm = pattern.replace('\\', "/");
    let mut target_norm = target.replace('\\', "/");

    if pattern_norm.starts_with("//?/") {
        pattern_norm = pattern_norm[4..].to_string();
    }
    if target_norm.starts_with("//?/") {
        target_norm = target_norm[4..].to_string();
    }

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
                        // Do NOT pop leading slash, just replace **/ with (?:[^/]+/)*
                        regex_str.push_str("(?:[^/]+/)*");
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
                regex_str.push_str("[^/]");
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

    // Bounded global lock across lookup/compile
    let mut cache = REGEX_CACHE.lock();
    if let Some(re) = cache.get(&regex_pattern) {
        return re.is_match(&target_norm);
    }

    match regex::Regex::new(&regex_pattern) {
        Ok(re) => {
            let arc_re = Arc::new(re);
            cache.put(regex_pattern, arc_re.clone());
            arc_re.is_match(&target_norm)
        }
        Err(_) => target_norm.starts_with(&pattern_norm),
    }
}

/// Resolves an executable name to its absolute canonical path by searching `$PATH`.
pub fn resolve_executable_path(exe_name: &str) -> Option<String> {
    let exe_path = std::path::Path::new(exe_name);
    if exe_path.is_absolute() {
        match exe_path.canonicalize() {
            Ok(canonical) => {
                let mut path_str = canonical.to_string_lossy().to_string().replace('\\', "/");
                if path_str.starts_with("//?/") {
                    path_str = path_str[4..].to_string();
                }
                return Some(path_str);
            }
            Err(e) => {
                tracing::warn!(
                    "[capability] Failed to canonicalize absolute executable path '{}': {:?}",
                    exe_name,
                    e
                );
                return None; // Fail-safe: do not return an uncanonicalized/unresolved path.
            }
        }
    }

    let path_var = match std::env::var_os("PATH") {
        Some(p) => p,
        None => {
            tracing::warn!("[capability] PATH environment variable is not set");
            return None;
        }
    };
    let paths = std::env::split_paths(&path_var);

    #[cfg(target_os = "windows")]
    let extensions = vec!["", ".exe", ".cmd", ".bat"];
    #[cfg(not(target_os = "windows"))]
    let extensions = vec![""];

    for path in paths {
        for ext in &extensions {
            let full_name = format!("{}{}", exe_name, ext);
            let check_path = path.join(&full_name);
            if check_path.is_file() {
                if let Ok(canonical) = check_path.canonicalize() {
                    let mut path_str = canonical.to_string_lossy().to_string().replace('\\', "/");
                    if path_str.starts_with("//?/") {
                        path_str = path_str[4..].to_string();
                    }
                    return Some(path_str);
                }
            }
        }
    }
    None
}

/// Sanitizes a file write pattern, blocking path traversal and restricting it to the workspace root.
fn sanitize_allowed_pattern(pattern: &str, workspace_root: &std::path::Path) -> Option<String> {
    let workspace_canonical = workspace_root
        .canonicalize()
        .unwrap_or_else(|_| workspace_root.to_path_buf());
    let mut workspace_str = workspace_canonical
        .to_string_lossy()
        .to_string()
        .replace('\\', "/");
    if workspace_str.starts_with("//?/") {
        workspace_str = workspace_str[4..].to_string();
    }
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
    #[must_use]
    pub fn verify(&self, required: &Permission) -> bool {
        // 1. Retrieve the matching verification key from keyring
        let keyring = KEYRING.lock();
        let key = match keyring.keys.get(&self.key_id) {
            Some(k) => k,
            None => {
                tracing::warn!(
                    "[capability] SIGNATURE FAILURE: unknown key_id={} token_id={} agent_id={} mission_id={}",
                    self.key_id, self.id, self.agent_id, self.mission_id
                );
                return false;
            }
        };

        // 2. Verify token signature integrity (constant time)
        let computed = compute_signature(
            &self.id,
            &self.agent_id,
            &self.mission_id,
            &self.permissions,
            &self.expires_at,
            key,
        );
        if computed.len() != self.signature.len() || !bool::from(computed.ct_eq(&self.signature)) {
            tracing::warn!(
                "[capability] SIGNATURE FAILURE: signature mismatch token_id={} agent_id={} mission_id={}",
                self.id, self.agent_id, self.mission_id
            );
            return false;
        }

        // 3. Check token expiration (allow 5-second grace window for clock skew)
        let now = chrono::Utc::now();
        if now > self.expires_at + chrono::Duration::seconds(5) {
            tracing::warn!(
                "[capability] EXPIRED: token_id={} agent_id={} mission_id={} expired_at={} now={}",
                self.id,
                self.agent_id,
                self.mission_id,
                self.expires_at,
                now
            );
            return false;
        }

        // 4. Check revocation status
        if is_revoked(&self.id) {
            tracing::warn!(
                "[capability] REVOKED: token_id={} agent_id={} mission_id={}",
                self.id,
                self.agent_id,
                self.mission_id
            );
            return false;
        }

        // 5. Match permissions (strict enforcement on ToolExec, no early returns)
        let verified = self.permissions.iter().any(|p| match (p, required) {
            (Permission::FileRead(p1), Permission::FileRead(p2)) => matches_glob(p1, p2),
            (Permission::FileWrite(p1), Permission::FileWrite(p2)) => matches_glob(p1, p2),
            (Permission::ShellExecute(p1), Permission::ShellExecute(p2)) => {
                let exe_name = p2.split_whitespace().next().unwrap_or("");
                let resolved_exe =
                    resolve_executable_path(exe_name).unwrap_or_else(|| exe_name.to_string());

                p1 == &resolved_exe
                    || p1 == exe_name
                    || matches_glob(p1, &resolved_exe)
                    || p1 == p2
                    || matches_glob(p1, p2)
            }
            (Permission::NetworkFetch(p1), Permission::NetworkFetch(p2)) => matches_glob(p1, p2),
            (Permission::ToolExec(p1), Permission::ToolExec(p2)) => matches_glob(p1, p2),
            (p1, p2) => p1 == p2,
        });

        if !verified {
            tracing::warn!(
                "[capability] DENY: token_id={} agent_id={} mission_id={} required={:?}",
                self.id,
                self.agent_id,
                self.mission_id,
                required
            );
        }
        verified
    }
}

pub struct ZeroTrustGuard;

impl ZeroTrustGuard {
    /// Generates a capability token based on agent authority, mission scope, and allowed files.
    #[must_use]
    pub fn mint_token(
        agent_id: &str,
        mission_id: &str,
        authority: crate::agent::types::RoleAuthorityLevel,
        allowed_files: Option<&[String]>,
        workspace_root: &std::path::Path,
    ) -> CapabilityToken {
        tracing::debug!(
            "[capability] Minting token for agent={} mission={} authority={:?}",
            agent_id,
            mission_id,
            authority
        );
        let mut permissions = vec![];
        let mut seen = HashSet::new();

        // Default tool execution permission: allow all non-mutating/safe tools by default
        let tool_perm = Permission::ToolExec("*".to_string());
        if seen.insert(tool_perm.clone()) {
            permissions.push(tool_perm);
        }

        // Canonicalize workspace root
        let workspace_canonical = workspace_root
            .canonicalize()
            .unwrap_or_else(|_| workspace_root.to_path_buf());
        let mut workspace_str = workspace_canonical
            .to_string_lossy()
            .to_string()
            .replace('\\', "/");
        if workspace_str.starts_with("//?/") {
            workspace_str = workspace_str[4..].to_string();
        }

        // Default read permission: read-only access to the entire workspace
        let read_perm = Permission::FileRead(workspace_str.clone());
        if seen.insert(read_perm.clone()) {
            permissions.push(read_perm);
        }

        // Add role-specific execution and spawn permissions
        match authority {
            crate::agent::types::RoleAuthorityLevel::Executive
            | crate::agent::types::RoleAuthorityLevel::Management => {
                let spawn_perm = Permission::SpawnAgent;
                if seen.insert(spawn_perm.clone()) {
                    permissions.push(spawn_perm);
                }

                // Resolve absolute paths for cargo and npm to prevent PATH hijacking
                let cargo_exe =
                    resolve_executable_path("cargo").unwrap_or_else(|| "cargo".to_string());
                let cargo_perm = Permission::ShellExecute(cargo_exe);
                if seen.insert(cargo_perm.clone()) {
                    permissions.push(cargo_perm);
                }

                let npm_exe = resolve_executable_path("npm").unwrap_or_else(|| "npm".to_string());
                let npm_perm = Permission::ShellExecute(npm_exe);
                if seen.insert(npm_perm.clone()) {
                    permissions.push(npm_perm);
                }
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
                            let write_perm = Permission::FileWrite(sanitized);
                            if seen.insert(write_perm.clone()) {
                                permissions.push(write_perm);
                            }
                        }
                    }
                } else {
                    let write_perm = Permission::FileWrite(workspace_str.clone());
                    if seen.insert(write_perm.clone()) {
                        permissions.push(write_perm);
                    }
                }
            } else {
                let write_perm = Permission::FileWrite(workspace_str.clone());
                if seen.insert(write_perm.clone()) {
                    permissions.push(write_perm);
                }
            }
        }

        // Authority-based token duration
        let duration = match authority {
            crate::agent::types::RoleAuthorityLevel::Executive => chrono::Duration::hours(2),
            crate::agent::types::RoleAuthorityLevel::Management => chrono::Duration::hours(1),
            _ => chrono::Duration::minutes(30),
        };

        let id = uuid::Uuid::new_v4().to_string();
        let expires_at = chrono::Utc::now() + duration;

        // Retrieve active key and compute signature
        let keyring = KEYRING.lock();
        let key_id = keyring.active_key_id.clone();
        let key = keyring
            .keys
            .get(&key_id)
            .expect("Active key must exist in keyring");
        let signature =
            compute_signature(&id, agent_id, mission_id, &permissions, &expires_at, key);

        CapabilityToken {
            id,
            agent_id: agent_id.to_string(),
            mission_id: mission_id.to_string(),
            permissions,
            expires_at,
            key_id,
            signature,
        }
    }
}

#[cfg(test)]
#[allow(dead_code)]
pub fn clear_revocations() {
    REVOCATION_LIST.lock().clear();
}

#[cfg(test)]
pub fn clear_regex_cache() {
    REGEX_CACHE.lock().clear();
}

#[cfg(test)]
mod tests {
    use super::*;
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
            if !self.tokens_to_unrevoke.is_empty() {
                let mut revocations = REVOCATION_LIST.lock();
                for token_id in &self.tokens_to_unrevoke {
                    revocations.remove(token_id);
                }
            }
            clear_regex_cache();
        }
    }

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

        // Question mark matching
        assert!(matches_glob("/base/src/main?.rs", "/base/src/main1.rs"));
        assert!(!matches_glob("/base/src/main?.rs", "/base/src/main/.rs"));
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
            root,
        );

        // Verify the token signature passes
        assert!(token.verify(&Permission::ToolExec("some_tool".to_string())));

        // Reads are allowed globally in the workspace
        assert!(token.verify(&Permission::FileRead("/base/tests/main.rs".to_string())));

        // Writes inside allowed patterns are permitted
        assert!(token.verify(&Permission::FileWrite("/base/src/main.rs".to_string())));
        assert!(token.verify(&Permission::FileWrite("/base/Cargo.toml".to_string())));

        // Writes outside allowed patterns are blocked
        assert!(!token.verify(&Permission::FileWrite("/base/tests/main.rs".to_string())));
    }

    #[test]
    fn test_token_signature_tampering() {
        let root = Path::new("/base");
        let original_token = ZeroTrustGuard::mint_token(
            "worker",
            "mission",
            crate::agent::types::RoleAuthorityLevel::Specialist,
            None,
            root,
        );

        // Verification initially passes
        assert!(original_token.verify(&Permission::ToolExec("any".to_string())));

        // 1. Tamper with permissions
        let mut t1 = original_token.clone();
        t1.permissions.push(Permission::SpawnAgent);
        assert!(!t1.verify(&Permission::ToolExec("any".to_string())));

        // 2. Tamper with agent_id
        let mut t2 = original_token.clone();
        t2.agent_id = "malicious_agent".to_string();
        assert!(!t2.verify(&Permission::ToolExec("any".to_string())));

        // 3. Tamper with mission_id
        let mut t3 = original_token.clone();
        t3.mission_id = "malicious_mission".to_string();
        assert!(!t3.verify(&Permission::ToolExec("any".to_string())));

        // 4. Tamper with expires_at
        let mut t4 = original_token.clone();
        t4.expires_at = chrono::Utc::now() + chrono::Duration::hours(100);
        assert!(!t4.verify(&Permission::ToolExec("any".to_string())));
    }

    #[test]
    fn test_token_json_roundtrip() {
        let root = Path::new("/base");
        let token = ZeroTrustGuard::mint_token(
            "worker",
            "mission",
            crate::agent::types::RoleAuthorityLevel::Specialist,
            None,
            root,
        );

        let serialized = serde_json::to_string(&token).unwrap();
        let deserialized: CapabilityToken = serde_json::from_str(&serialized).unwrap();

        // Verification still passes after round-trip
        assert!(deserialized.verify(&Permission::ToolExec("any".to_string())));
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
            root,
        );

        guard.register_token(&token.id);

        assert!(token.verify(&Permission::ToolExec("any".to_string())));

        // Revoke token
        revoke_token(&token.id);

        // Verification must now fail
        assert!(!token.verify(&Permission::ToolExec("any".to_string())));
    }

    #[test]
    fn test_shell_execute_matching() {
        let root = Path::new("/base");
        let token = ZeroTrustGuard::mint_token(
            "worker",
            "mission",
            crate::agent::types::RoleAuthorityLevel::Executive,
            None,
            root,
        );

        // Executing cargo build should be allowed since "cargo" was granted (and resolved in path)
        assert!(token.verify(&Permission::ShellExecute(
            "cargo build --release".to_string()
        )));
        // Executing npm test should be allowed since "npm" was granted
        assert!(token.verify(&Permission::ShellExecute("npm test".to_string())));
        // Executing other commands should be blocked
        assert!(!token.verify(&Permission::ShellExecute("rm -rf /".to_string())));
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
            key_id: "test_key".to_string(), // use an isolated test key
            signature,
        };

        // Inject the signature key for test_key to keyring for verification
        set_key("test_key", key);
        guard.register_key("test_key");

        assert!(token.verify(&Permission::NetworkFetch(
            "https://api.github.com/repos".to_string()
        )));
        assert!(!token.verify(&Permission::NetworkFetch("https://google.com".to_string())));
    }
}

// Metadata: [capability]

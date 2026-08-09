//! @docs ARCHITECTURE:Security
//!
//! ### AI Assist Note
//! **CBS Guard & Lifecycle**: Implements `ZeroTrustGuard` token minting, verification, and revocation tracking.

use once_cell::sync::Lazy;
use parking_lot::Mutex;
use std::collections::{HashMap, HashSet};
use std::path::Path;
use subtle::ConstantTimeEq;

use super::crypto::{compute_signature, KEYRING};
use super::paths::{matches_glob, resolve_executable_path, sanitize_allowed_pattern};
use super::types::{CapabilityToken, Permission};

static REVOCATION_LIST: Lazy<Mutex<HashMap<String, chrono::DateTime<chrono::Utc>>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

/// Revokes a capability token by ID with an explicit expiration timestamp.
#[allow(dead_code)]
pub fn revoke_token_with_expiry(id: &str, expires_at: chrono::DateTime<chrono::Utc>) {
    tracing::warn!("[capability] REVOKED: token_id={} expires_at={}", id, expires_at);
    REVOCATION_LIST.lock().insert(id.to_string(), expires_at);
}

/// Revokes a capability token by ID.
#[allow(dead_code)]
pub fn revoke_token(id: &str) {
    let default_exp = chrono::Utc::now() + chrono::Duration::hours(24);
    revoke_token_with_expiry(id, default_exp);
}

/// Checks if a capability token has been revoked and prunes expired tokens.
pub fn is_revoked(id: &str) -> bool {
    let mut list = REVOCATION_LIST.lock();
    let now = chrono::Utc::now();
    list.retain(|_, exp| *exp > now);
    list.contains_key(id)
}

#[cfg(test)]
#[allow(dead_code)]
pub fn clear_revocations() {
    REVOCATION_LIST.lock().clear();
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
                if let Some(resolved_exe) = resolve_executable_path(exe_name) {
                    p1 == &resolved_exe
                        || p1 == exe_name
                        || matches_glob(p1, &resolved_exe)
                        || p1 == p2
                        || matches_glob(p1, p2)
                } else {
                    p1 == exe_name || p1 == p2
                }
            }
            (Permission::NetworkFetch(p1), Permission::NetworkFetch(p2)) => matches_glob(p1, p2),
            (Permission::ToolExec(p1), Permission::ToolExec(p2)) => {
                if p1 == "*" || p2 == "*" {
                    false
                } else {
                    p1 == p2 || matches_glob(p1, p2)
                }
            }
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
    /// Generates a capability token based on agent authority, mission scope, allowed files, and assigned skills.
    #[must_use]
    pub fn mint_token(
        agent_id: &str,
        mission_id: &str,
        authority: crate::agent::types::RoleAuthorityLevel,
        allowed_files: Option<&[String]>,
        allowed_skills: Option<&[String]>,
        workspace_root: &Path,
    ) -> CapabilityToken {
        tracing::debug!(
            "[capability] Minting token for agent={} mission={} authority={:?}",
            agent_id,
            mission_id,
            authority
        );
        let mut permissions = vec![];
        let mut seen = HashSet::new();

        // Explicit tool execution grants for non-mutating safe tools & core memory tools (no ambient '*' wildcards)
        let safe_tools = [
            "read_file",
            "get_file_contents",
            "read_codebase_file",
            "list_files",
            "grep_search",
            "list_file_symbols",
            "get_symbol_body",
            "search_web",
            "fetch_url",
            "search_global_vault",
            "update_working_memory",
            "share_finding",
            "store_knowledge",
            "search_knowledge",
            "complete_mission",
        ];
        for tool in safe_tools {
            let tool_perm = Permission::ToolExec(tool.to_string());
            if seen.insert(tool_perm.clone()) {
                permissions.push(tool_perm);
            }
        }

        if let Some(skills) = allowed_skills {
            for skill in skills {
                let tool_perm = Permission::ToolExec(skill.to_string());
                if seen.insert(tool_perm.clone()) {
                    permissions.push(tool_perm);
                }
            }
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
        if agent_id == crate::agent::constants::AGENT_CEO
            || matches!(
                authority,
                crate::agent::types::RoleAuthorityLevel::Executive
                    | crate::agent::types::RoleAuthorityLevel::Management
            )
        {
            let alpha_perm = Permission::ToolExec("issue_alpha_directive".to_string());
            if seen.insert(alpha_perm.clone()) {
                permissions.push(alpha_perm);
            }
        }

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

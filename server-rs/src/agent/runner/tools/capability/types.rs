//! @docs ARCHITECTURE:Security
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Agent Runner / types
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Type-safe state handling and bounded execution without unhandled panics.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: none declared
//! - **Witness Tests**: none declared

use serde::{Deserialize, Serialize};

/// System Capability Permissions.
///
/// NOTE: Discriminants and binary tags in `encode_permission_canonical` are wire-format-stable
/// and must never be renumbered or changed without a protocol version bump.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Permission {
    /// Grants read access to a specific path or pattern (Tag: 0).
    FileRead(String),
    /// Grants write access to a specific path or pattern (Tag: 1).
    FileWrite(String),
    /// Grants permission to execute a specific command or command pattern (Tag: 2).
    ShellExecute(String),
    /// Grants permission to spawn sub-agents (Tag: 3).
    SpawnAgent,
    /// Grants permission to switch active model slot (Tag: 6).
    ModelSwitch,
    /// Grants permission to fetch external URLs (Tag: 4).
    NetworkFetch(String),
    /// Grants permission to execute a safe named tool that does not require dedicated filesystem/network grants (Tag: 5).
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

/// [types] Encodes permissions into canonical binary format for HMAC signing.
pub fn encode_permission_canonical(permission: &Permission, buf: &mut Vec<u8>) {
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
        Permission::ModelSwitch => {
            buf.push(6);
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

pub fn canonical_message(
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

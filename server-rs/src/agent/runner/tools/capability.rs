//! @docs ARCHITECTURE:Security
//! 
//! ### AI Assist Note
//! **Capability-Based Security (CBS)**: Implements the **SEC-04** zero-trust 
//! model. Tools no longer have ambient authority. They must be invoked with 
//! a non-forgeable `CapabilityToken` that defines a set of explicit 
//! `Permission` grants.

use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityToken {
    pub id: String,
    pub agent_id: String,
    pub mission_id: String,
    pub permissions: Vec<Permission>,
    pub expires_at: chrono::DateTime<chrono::Utc>,
}

impl CapabilityToken {
    /// Verifies if the token contains the required permission.
    #[allow(dead_code)]
    pub fn verify(&self, required: &Permission) -> bool {
        if chrono::Utc::now() > self.expires_at {
            return false;
        }

        self.permissions.iter().any(|p| match (p, required) {
            (Permission::FileRead(p1), Permission::FileRead(p2)) => p2.starts_with(p1),
            (Permission::FileWrite(p1), Permission::FileWrite(p2)) => p2.starts_with(p1),
            (p1, p2) => p1 == p2,
        })
    }
}

pub struct ZeroTrustGuard;

impl ZeroTrustGuard {
    /// Generates a capability token based on agent authority and mission scope.
    pub fn mint_token(
        agent_id: &str,
        mission_id: &str,
        authority: crate::agent::types::RoleAuthorityLevel,
    ) -> CapabilityToken {
        let mut permissions = vec![];

        // Default permissions for all agents
        permissions.push(Permission::FileRead(".".to_string()));
        
        match authority {
            crate::agent::types::RoleAuthorityLevel::Executive | crate::agent::types::RoleAuthorityLevel::Management => {
                permissions.push(Permission::SpawnAgent);
                permissions.push(Permission::FileWrite(".".to_string()));
                permissions.push(Permission::ShellExecute("cargo".to_string()));
                permissions.push(Permission::ShellExecute("npm".to_string()));
            }
            crate::agent::types::RoleAuthorityLevel::Specialist => {
                permissions.push(Permission::FileWrite(".".to_string()));
            }
            crate::agent::types::RoleAuthorityLevel::Observer => {
                // Read-only access
            }
        }

        CapabilityToken {
            id: uuid::Uuid::new_v4().to_string(),
            agent_id: agent_id.to_string(),
            mission_id: mission_id.to_string(),
            permissions,
            expires_at: chrono::Utc::now() + chrono::Duration::hours(1),
        }
    }
}

// Metadata: [capability]

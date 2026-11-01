//! @docs ARCHITECTURE:SecurityModel
//!
//! ### AI Assist Note
//! - **Subsystem**: Security / Permission facts
//! - Typed decisions and floor semantics are independent of storage and prompting.
//!
//! ### 🔍 Debugging & Observability
//! - **Telemetry Targets**: none declared

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Copy)]
pub enum PermissionMode {
    /// Tool is always allowed (e.g., read-only safe commands).
    Allow,
    /// Tool is always denied (e.g., restricted system access).
    Deny,
    /// Tool execution is paused until the user provides explicit approval.
    Prompt,
}

impl PermissionMode {
    pub fn as_canonical_str(&self) -> &'static str {
        match self {
            PermissionMode::Allow => "allow",
            PermissionMode::Deny => "deny",
            PermissionMode::Prompt => "prompt",
        }
    }

    /// Clamps permission mode against a mandatory security floor (SEC-06).
    /// Controls can be tightened (e.g. Prompt -> Deny) but never weakened below the floor.
    pub fn clamp_to_floor(self, floor: PermissionMode) -> PermissionMode {
        match (self, floor) {
            (PermissionMode::Deny, _) => PermissionMode::Deny,
            (PermissionMode::Allow, PermissionMode::Prompt) => PermissionMode::Prompt,
            (PermissionMode::Allow, PermissionMode::Deny) => PermissionMode::Deny,
            (PermissionMode::Prompt, PermissionMode::Deny) => PermissionMode::Deny,
            (mode, _) => mode,
        }
    }
}

impl std::str::FromStr for PermissionMode {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_lowercase().as_str() {
            "allow" => Ok(PermissionMode::Allow),
            "deny" => Ok(PermissionMode::Deny),
            "prompt" => Ok(PermissionMode::Prompt),
            _ => Err(anyhow!("Invalid permission mode: '{}'", s)),
        }
    }
}

impl std::fmt::Display for PermissionMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PermissionMode::Allow => write!(f, "Allow"),
            PermissionMode::Deny => write!(f, "Deny"),
            PermissionMode::Prompt => write!(f, "Prompt"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, Copy)]
pub enum CapabilityClass {
    Execute,
    Install,
    Modify,
    Delete,
    Approve,
}

impl CapabilityClass {
    pub fn as_canonical_str(&self) -> &'static str {
        match self {
            CapabilityClass::Execute => "execute",
            CapabilityClass::Install => "install",
            CapabilityClass::Modify => "modify",
            CapabilityClass::Delete => "delete",
            CapabilityClass::Approve => "approve",
        }
    }

    /// Returns the non-overridable security floor (SEC-06).
    pub fn mandatory_floor(&self) -> PermissionMode {
        match self {
            CapabilityClass::Execute => PermissionMode::Prompt,
            CapabilityClass::Install => PermissionMode::Prompt,
            CapabilityClass::Modify => PermissionMode::Prompt,
            CapabilityClass::Delete => PermissionMode::Prompt,
            CapabilityClass::Approve => PermissionMode::Deny,
        }
    }
}

impl std::str::FromStr for CapabilityClass {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_lowercase().as_str() {
            "execute" => Ok(CapabilityClass::Execute),
            "install" => Ok(CapabilityClass::Install),
            "modify" => Ok(CapabilityClass::Modify),
            "delete" => Ok(CapabilityClass::Delete),
            "approve" => Ok(CapabilityClass::Approve),
            _ => Err(anyhow!("Invalid capability class: '{}'", s)),
        }
    }
}

impl std::fmt::Display for CapabilityClass {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CapabilityClass::Execute => write!(f, "Execute"),
            CapabilityClass::Install => write!(f, "Install"),
            CapabilityClass::Modify => write!(f, "Modify"),
            CapabilityClass::Delete => write!(f, "Delete"),
            CapabilityClass::Approve => write!(f, "Approve"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PermissionDecisionSource {
    AgentPolicy,
    RolePolicy,
    GlobalCapabilityPolicy,
    GlobalLegacyPolicy,
    DomainPolicy,
    DefaultPrompt,
    DatabaseFailure,
    PolicyDataError,
    SecurityFloor,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PermissionDecision {
    pub mode: PermissionMode,
    pub source: PermissionDecisionSource,
    pub reason: Option<String>,
}

impl PermissionDecision {
    pub fn new(
        mode: PermissionMode,
        source: PermissionDecisionSource,
        reason: Option<String>,
    ) -> Self {
        Self {
            mode,
            source,
            reason,
        }
    }

    pub fn with_reason(
        mode: PermissionMode,
        source: PermissionDecisionSource,
        reason: impl Into<String>,
    ) -> Self {
        Self {
            mode,
            source,
            reason: Some(reason.into()),
        }
    }
}

/// Strongly typed cache key for structural matching & cache invalidation.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PolicyCacheKey {
    Legacy {
        agent_id: Option<String>,
        role: Option<String>,
        tool_name: String,
    },
    Capability {
        agent_id: Option<String>,
        role: Option<String>,
        capability: CapabilityClass,
        resource: String,
    },
}

//! @docs ARCHITECTURE:Registry
//! 
//! ### AI Assist Note
//! **Core technical resource for the Tadpole OS Sovereign infrastructure.**
//! This module implements high-fidelity logic for the Sovereign Reality layer.
//! 
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: Runtime logic error, state desynchronization, or resource exhaustion.
//! - **Telemetry Link**: Search `[service_traits]` in tracing logs.

use std::collections::HashMap;
use crate::agent::types::RoleAuthorityLevel;

/// Interface for rendering system prompts with variable interpolation.
pub trait PromptRendererTrait: Send + Sync {
    /// Renders a template string using the provided variable map.
    fn render(&self, template: &str, variables: &HashMap<&str, String>) -> String;
    
    /// Returns the default system prompt template.
    fn default_system_template(&self) -> &'static str;
}

/// Interface for Access Control Lists (ACL) governance.
pub trait AclServiceTrait: Send + Sync {
    /// Checks if a tool is allowed for a specific agent/role/authority level.
    fn is_tool_allowed(&self, agent_id: &str, role: &str, authority: RoleAuthorityLevel, tool_name: &str) -> bool;
    
    /// Returns mandatory protocols for a given agent and role.
    fn get_role_protocols(&self, agent_id: &str, role: &str, authority: RoleAuthorityLevel) -> Vec<String>;
}

// Metadata: [service_traits]

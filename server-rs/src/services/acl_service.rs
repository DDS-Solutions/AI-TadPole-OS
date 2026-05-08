//! @docs ARCHITECTURE:UI-Services
//! 
//! ### AI Assist Note
//! **Core technical resource for the Tadpole OS Sovereign infrastructure.**
//! This module implements high-fidelity logic for the Sovereign Reality layer.
//! 
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: Runtime logic error, state desynchronization, or resource exhaustion.
//! - **Telemetry Link**: Search `[acl_service]` in tracing logs.

use crate::agent::types::RoleAuthorityLevel;
use crate::agent::constants::*;
use crate::agent::runner::service_traits::AclServiceTrait;

pub struct AclService;

impl AclServiceTrait for AclService {
    /// Checks if a tool is allowed for a specific agent/role.
    fn is_tool_allowed(&self, agent_id: &str, _role: &str, authority: RoleAuthorityLevel, tool_name: &str) -> bool {
        // CEO/Executive logic
        if agent_id == AGENT_CEO || authority == RoleAuthorityLevel::Executive {
            match tool_name {
                "issue_alpha_directive" | "share_finding" | "search_global_vault" | "update_working_memory" | "complete_mission" => true,
                "spawn_subagent" => false, // CEO must use alpha_directive
                "read_file" | "write_file" | "delete_file" => false, // CEO doesn't do I/O
                _ => true,
            }
        } else if agent_id == AGENT_COO {
            match tool_name {
                "spawn_subagent" => true, // COO spawns Alpha
                "issue_alpha_directive" => false,
                _ => true,
            }
        } else if authority == RoleAuthorityLevel::Observer {
            match tool_name {
                "read_file" | "list_files" | "search_global_vault" => true,
                _ => false, // No mutations for observers
            }
        } else {
            // Specialists
            !matches!(tool_name, "issue_alpha_directive")
        }
    }

    /// Returns the mandatory protocols for a role.
    fn get_role_protocols(&self, agent_id: &str, role: &str, _authority: RoleAuthorityLevel) -> Vec<String> {
        let mut protocols = Vec::new();

        match agent_id {
            AGENT_CEO => {
                protocols.push("CEO PROTOCOL: You are a STRATEGIC ROUTER. You MUST delegate via 'issue_alpha_directive' for all complex missions. You are FORBIDDEN from using 'spawn_subagent' directly.".to_string());
            }
            AGENT_COO => {
                protocols.push("COO PROTOCOL: You MUST delegate the mission to the Alpha Node. Use 'spawn_subagent' with agent_id 'alpha'. Direct specialist recruitment is SYSTEM-BLOCKED.".to_string());
            }
            AGENT_ALPHA => {
                protocols.push("ALPHA COMMAND: You are the Swarm Mission Commander. You are responsible for recruiting and synthesizing specialists (Researcher, Coder, etc.).".to_string());
            }
            _ => {
                protocols.push(format!("SPECIALIST AUTONOMY: You are tactical specialist {}. You MUST resolve your mission independently using your assigned tools.", role));
                protocols.push(format!("COMMANDER IS BUSY: You are under the supervision of the Alpha Node. Do NOT attempt to recruit '{}', '{}', or '{}' for assistance. Resolve the task yourself.", AGENT_ALPHA, AGENT_COO, AGENT_CEO));
            }
        }

        protocols
    }
}

// Metadata: [acl_service]

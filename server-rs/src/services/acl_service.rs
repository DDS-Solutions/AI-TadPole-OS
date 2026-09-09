//! @docs ARCHITECTURE:Security:Acl
//!
//! ### AI Context Alignment
//! - **Subsystem**: Frontend Service Layer / acl_service
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Asynchronous service calls normalize response envelopes and propagate typed errors.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: none declared
//! - **Witness Tests**: none declared

use crate::agent::constants::*;
use crate::agent::runner::service_traits::AclServiceTrait;
use crate::agent::types::RoleAuthorityLevel;

pub struct AclService;

impl AclServiceTrait for AclService {
    /// Checks if a tool is allowed for a specific agent/role under a Zero-Trust Default-Deny matrix.
    fn is_tool_allowed(
        &self,
        agent_id: &str,
        role: &str,
        authority: RoleAuthorityLevel,
        tool_name: &str,
    ) -> bool {
        let allowed = match authority {
            // Tier 1: Observer — Strictly read-only inspection tools
            RoleAuthorityLevel::Observer => matches!(
                tool_name,
                "read_file"
                    | "list_files"
                    | "read_codebase_file"
                    | "list_file_symbols"
                    | "get_symbol_body"
                    | "search_global_vault"
                    | "search_mission_knowledge"
                    | "get_agent_metrics"
            ),

            // Tier 2: Executive — Strategic oversight & directive delegation
            RoleAuthorityLevel::Executive => {
                if agent_id == AGENT_CEO {
                    // CEO Persona: Strategic orchestrator; uses alpha_directive, no direct subagents, no file mutation
                    matches!(
                        tool_name,
                        "issue_alpha_directive"
                            | "share_finding"
                            | "search_global_vault"
                            | "update_working_memory"
                            | "complete_mission"
                            | "pin_mission"
                            | "get_agent_metrics"
                    )
                } else if agent_id == AGENT_COO {
                    // COO Persona: Swarm dispatcher; recruits Alpha node and coordinates missions
                    matches!(
                        tool_name,
                        "spawn_subagent"
                            | "send_mission_directive"
                            | "share_finding"
                            | "search_global_vault"
                            | "update_working_memory"
                            | "complete_mission"
                            | "get_agent_metrics"
                    )
                } else {
                    // Generic Executive: Strategic management tools
                    matches!(
                        tool_name,
                        "spawn_subagent"
                            | "send_mission_directive"
                            | "request_peer_audit"
                            | "submit_peer_review"
                            | "share_finding"
                            | "search_global_vault"
                            | "update_working_memory"
                            | "complete_mission"
                            | "get_agent_metrics"
                    )
                }
            }

            // Tier 3: Management (Alpha Node, Leads, Commanders) — Tactical coordination and subagent recruitment
            RoleAuthorityLevel::Management => {
                if agent_id == AGENT_COO {
                    // COO Persona: Dispatcher
                    matches!(
                        tool_name,
                        "spawn_subagent"
                            | "send_mission_directive"
                            | "share_finding"
                            | "search_global_vault"
                            | "update_working_memory"
                            | "complete_mission"
                            | "get_agent_metrics"
                    )
                } else {
                    // Alpha Commander / Team Leads: Tactical coordination, specialist recruitment, and synthesis
                    matches!(
                        tool_name,
                        "spawn_subagent"
                            | "recruit_specialist"
                            | "send_mission_directive"
                            | "request_peer_audit"
                            | "submit_peer_review"
                            | "archive_to_global_vault"
                            | "search_global_vault"
                            | "synthesize_micro_script"
                            | "refactor_synthesized_skill"
                            | "share_finding"
                            | "complete_mission"
                            | "read_file"
                            | "write_file"
                            | "list_files"
                            | "delete_file"
                            | "read_codebase_file"
                            | "list_file_symbols"
                            | "get_symbol_body"
                            | "update_working_memory"
                            | "script_builder"
                            | "fetch_url"
                            | "notify_discord"
                            | "get_agent_metrics"
                    )
                }
            }

            // Tier 4: Tactical Specialists (Leaf Workers) — Execution tools only, NO subagent spawning
            RoleAuthorityLevel::Specialist => {
                if agent_id == AGENT_ALPHA {
                    // Legacy Alpha mapping as Specialist fallback: allowed to manage swarm
                    matches!(
                        tool_name,
                        "spawn_subagent"
                            | "recruit_specialist"
                            | "send_mission_directive"
                            | "request_peer_audit"
                            | "submit_peer_review"
                            | "archive_to_global_vault"
                            | "search_global_vault"
                            | "synthesize_micro_script"
                            | "refactor_synthesized_skill"
                            | "share_finding"
                            | "complete_mission"
                            | "read_file"
                            | "write_file"
                            | "list_files"
                            | "delete_file"
                            | "read_codebase_file"
                            | "list_file_symbols"
                            | "get_symbol_body"
                            | "update_working_memory"
                            | "script_builder"
                            | "fetch_url"
                            | "notify_discord"
                            | "get_agent_metrics"
                    )
                } else {
                    // Leaf Specialists (Coder, Researcher, Tester): Default-deny on spawn_subagent & issue_alpha_directive
                    matches!(
                        tool_name,
                        "read_file"
                            | "write_file"
                            | "list_files"
                            | "delete_file"
                            | "read_codebase_file"
                            | "list_file_symbols"
                            | "get_symbol_body"
                            | "search_global_vault"
                            | "search_mission_knowledge"
                            | "share_finding"
                            | "complete_mission"
                            | "update_working_memory"
                            | "synthesize_micro_script"
                            | "refactor_synthesized_skill"
                            | "script_builder"
                            | "fetch_url"
                            | "request_peer_audit"
                            | "submit_peer_review"
                            | "get_agent_metrics"
                    )
                }
            }
        };

        if !allowed {
            tracing::warn!(
                target: "acl_service",
                agent_id = %agent_id,
                role = %role,
                authority = ?authority,
                tool = %tool_name,
                "[acl_service] Access DENIED for tool invocation"
            );
        } else {
            tracing::debug!(
                target: "acl_service",
                agent_id = %agent_id,
                tool = %tool_name,
                "[acl_service] Access GRANTED"
            );
        }

        allowed
    }

    /// Returns the mandatory protocols for a role.
    fn get_role_protocols(
        &self,
        agent_id: &str,
        role: &str,
        _authority: RoleAuthorityLevel,
    ) -> Vec<String> {
        let mut protocols = Vec::new();

        match agent_id {
            AGENT_CEO => {
                protocols.push("CEO PROTOCOL: You are a STRATEGIC ROUTER. You MUST delegate via 'issue_alpha_directive' for all complex missions. Direct worker recruitment or file I/O is blocked by ACL.".to_string());
            }
            AGENT_COO => {
                protocols.push("COO PROTOCOL: You MUST delegate the mission to the Alpha Node. Use 'spawn_subagent' with agent_id 'alpha'. Direct specialist recruitment is SYSTEM-BLOCKED.".to_string());
            }
            AGENT_ALPHA => {
                protocols.push("ALPHA COMMAND: You are the Swarm Mission Commander. You are responsible for recruiting and synthesizing specialists (Researcher, Coder, etc.).".to_string());
            }
            _ => {
                protocols.push(format!("SPECIALIST AUTONOMY: You are tactical specialist {}. You MUST resolve your mission independently using your assigned tools.", role));
                protocols.push(format!("COMMANDER IS BUSY: You are under the supervision of the Alpha Node. Do NOT attempt to recruit '{}', '{}', or '{}' for assistance. Subagent recruitment is blocked by ACL.", AGENT_ALPHA, AGENT_COO, AGENT_CEO));
            }
        }

        protocols
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ceo_acl_permissions() {
        let acl = AclService;
        assert!(acl.is_tool_allowed(
            AGENT_CEO,
            "CEO",
            RoleAuthorityLevel::Executive,
            "issue_alpha_directive"
        ));
        assert!(acl.is_tool_allowed(
            AGENT_CEO,
            "CEO",
            RoleAuthorityLevel::Executive,
            "complete_mission"
        ));
        assert!(!acl.is_tool_allowed(
            AGENT_CEO,
            "CEO",
            RoleAuthorityLevel::Executive,
            "spawn_subagent"
        ));
        assert!(!acl.is_tool_allowed(AGENT_CEO, "CEO", RoleAuthorityLevel::Executive, "read_file"));
        assert!(!acl.is_tool_allowed(
            AGENT_CEO,
            "CEO",
            RoleAuthorityLevel::Executive,
            "write_file"
        ));
        assert!(!acl.is_tool_allowed(
            AGENT_CEO,
            "CEO",
            RoleAuthorityLevel::Executive,
            "unregistered_tool"
        ));
    }

    #[test]
    fn test_coo_acl_permissions() {
        let acl = AclService;
        assert!(acl.is_tool_allowed(
            AGENT_COO,
            "COO",
            RoleAuthorityLevel::Executive,
            "spawn_subagent"
        ));
        assert!(acl.is_tool_allowed(
            AGENT_COO,
            "COO",
            RoleAuthorityLevel::Executive,
            "complete_mission"
        ));
        assert!(!acl.is_tool_allowed(
            AGENT_COO,
            "COO",
            RoleAuthorityLevel::Executive,
            "issue_alpha_directive"
        ));
        assert!(!acl.is_tool_allowed(
            AGENT_COO,
            "COO",
            RoleAuthorityLevel::Executive,
            "write_file"
        ));
        assert!(!acl.is_tool_allowed(
            AGENT_COO,
            "COO",
            RoleAuthorityLevel::Executive,
            "unknown_tool"
        ));
    }

    #[test]
    fn test_management_tier_permissions() {
        let acl = AclService;
        assert!(acl.is_tool_allowed(
            "lead-1",
            "Lead",
            RoleAuthorityLevel::Management,
            "spawn_subagent"
        ));
        assert!(acl.is_tool_allowed(
            "lead-1",
            "Lead",
            RoleAuthorityLevel::Management,
            "recruit_specialist"
        ));
        assert!(acl.is_tool_allowed(
            "lead-1",
            "Lead",
            RoleAuthorityLevel::Management,
            "read_file"
        ));
        assert!(acl.is_tool_allowed(
            "lead-1",
            "Lead",
            RoleAuthorityLevel::Management,
            "write_file"
        ));
        assert!(!acl.is_tool_allowed(
            "lead-1",
            "Lead",
            RoleAuthorityLevel::Management,
            "issue_alpha_directive"
        ));
        assert!(!acl.is_tool_allowed(
            "lead-1",
            "Lead",
            RoleAuthorityLevel::Management,
            "unregistered_tool"
        ));
    }

    #[test]
    fn test_alpha_commander_acl_permissions() {
        let acl = AclService;
        assert!(acl.is_tool_allowed(
            AGENT_ALPHA,
            "Commander",
            RoleAuthorityLevel::Management,
            "spawn_subagent"
        ));
        assert!(acl.is_tool_allowed(
            AGENT_ALPHA,
            "Commander",
            RoleAuthorityLevel::Management,
            "recruit_specialist"
        ));
        assert!(acl.is_tool_allowed(
            AGENT_ALPHA,
            "Commander",
            RoleAuthorityLevel::Management,
            "read_file"
        ));
        assert!(acl.is_tool_allowed(
            AGENT_ALPHA,
            "Commander",
            RoleAuthorityLevel::Management,
            "write_file"
        ));
        assert!(!acl.is_tool_allowed(
            AGENT_ALPHA,
            "Commander",
            RoleAuthorityLevel::Management,
            "issue_alpha_directive"
        ));
        assert!(!acl.is_tool_allowed(
            AGENT_ALPHA,
            "Commander",
            RoleAuthorityLevel::Management,
            "unknown_tool"
        ));
    }

    #[test]
    fn test_specialist_acl_blocks_spawn_subagent() {
        let acl = AclService;
        assert!(acl.is_tool_allowed(
            "specialist-coder",
            "Coder",
            RoleAuthorityLevel::Specialist,
            "read_file"
        ));
        assert!(acl.is_tool_allowed(
            "specialist-coder",
            "Coder",
            RoleAuthorityLevel::Specialist,
            "write_file"
        ));
        assert!(acl.is_tool_allowed(
            "specialist-coder",
            "Coder",
            RoleAuthorityLevel::Specialist,
            "complete_mission"
        ));
        // Specialists must NOT be allowed to spawn subagents or issue alpha directives
        assert!(!acl.is_tool_allowed(
            "specialist-coder",
            "Coder",
            RoleAuthorityLevel::Specialist,
            "spawn_subagent"
        ));
        assert!(!acl.is_tool_allowed(
            "specialist-coder",
            "Coder",
            RoleAuthorityLevel::Specialist,
            "recruit_specialist"
        ));
        assert!(!acl.is_tool_allowed(
            "specialist-coder",
            "Coder",
            RoleAuthorityLevel::Specialist,
            "issue_alpha_directive"
        ));
        assert!(!acl.is_tool_allowed(
            "specialist-coder",
            "Coder",
            RoleAuthorityLevel::Specialist,
            "unknown_random_tool"
        ));
    }

    #[test]
    fn test_observer_acl_strictly_read_only() {
        let acl = AclService;
        assert!(acl.is_tool_allowed(
            "obs-1",
            "Auditor",
            RoleAuthorityLevel::Observer,
            "read_file"
        ));
        assert!(acl.is_tool_allowed(
            "obs-1",
            "Auditor",
            RoleAuthorityLevel::Observer,
            "list_files"
        ));
        assert!(acl.is_tool_allowed(
            "obs-1",
            "Auditor",
            RoleAuthorityLevel::Observer,
            "search_global_vault"
        ));
        // Mutation tools must be denied
        assert!(!acl.is_tool_allowed(
            "obs-1",
            "Auditor",
            RoleAuthorityLevel::Observer,
            "write_file"
        ));
        assert!(!acl.is_tool_allowed(
            "obs-1",
            "Auditor",
            RoleAuthorityLevel::Observer,
            "delete_file"
        ));
        assert!(!acl.is_tool_allowed(
            "obs-1",
            "Auditor",
            RoleAuthorityLevel::Observer,
            "spawn_subagent"
        ));
    }
}

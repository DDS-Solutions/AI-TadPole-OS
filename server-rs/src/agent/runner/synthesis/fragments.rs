//! @docs ARCHITECTURE:Registry
//! 
//! ### AI Assist Note
//! **Core technical resource for the Tadpole OS Sovereign infrastructure.**
//! This module implements high-fidelity logic for the Sovereign Reality layer.
//! 
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: Runtime logic error, state desynchronization, or resource exhaustion.
//! - **Telemetry Link**: Search `[fragments]` in tracing logs.

use crate::agent::runner::RunContext;
use crate::state::AppState;
use crate::agent::constants::*;
use crate::agent::types::RoleAuthorityLevel;

pub fn get_hierarchy_label(level: RoleAuthorityLevel) -> &'static str {
    match level {
        RoleAuthorityLevel::Executive => "OVERLORD (Tier 0)",
        RoleAuthorityLevel::Management => "ORCHESTRATOR (Tier 1)",
        RoleAuthorityLevel::Specialist => "SPECIALIST (Tier 2)",
        RoleAuthorityLevel::Observer => "AUDITOR (Tier 3)",
    }
}

pub fn generate_swarm_protocols(ctx: &RunContext, state: &AppState) -> String {
    let mut swarm_protocols = state.resources.acl.get_role_protocols(
        &ctx.agent_id,
        &ctx.role,
        ctx.authority_level,
    );

    // Add shared/invariant swarm rules
    swarm_protocols.push("NO NARRATION: Do not explain strategy or tool options. CALL THE TOOLS. Text-only responses are MISSION FAILURE.".to_string());
    swarm_protocols.push("COMPLETION: Complete ALL steps autonomously before reporting back.".to_string());
    swarm_protocols.push("OVERSIGHT TRUST: Never ask for permission. The system handles approval flows automatically.".to_string());
    swarm_protocols.push("SOURCE OF TRUTH: Prioritize codebase tools (read_codebase_file) over general RAG for code queries.".to_string());
    swarm_protocols.push(format!("RECURSION PROTECTION: You are FORBIDDEN from recruiting agents in your LINEAGE ({:?}) or YOURSELF ({}).", &ctx.lineage, ctx.agent_id));

    swarm_protocols
        .iter()
        .enumerate()
        .map(|(i, p)| format!("{}. {}", i + 1, p))
        .collect::<Vec<_>>()
        .join("\n")
}

pub fn generate_structural_components(ctx: &RunContext, state: &AppState) -> (String, String, String, bool, String, String) {
    let cluster_specialists = state.registry.list_active_specialists();

    // --- Cluster Directory ---
    let cluster_directory = if cluster_specialists.is_empty() {
        "No specialized agents currently registered in the cluster pool. You are encouraged to recruit a NEW specialist ID (e.g. 'researcher', 'coder') if needed.".to_string()
    } else {
        let is_orchestrator = ctx.agent_id == AGENT_CEO
            || ctx.agent_id == AGENT_COO
            || ctx.agent_id == AGENT_ALPHA;

        cluster_specialists
            .into_iter()
            .map(|id| {
                if is_orchestrator {
                    id
                } else {
                    match id.as_str() {
                        AGENT_CEO => "[SYSTEM_OVERLORD]".to_string(),
                        AGENT_COO => "[SYSTEM_ORCHESTRATOR]".to_string(),
                        AGENT_ALPHA => "[SYSTEM_COMMANDER]".to_string(),
                        other => other.to_string(),
                    }
                }
            })
            .collect::<Vec<_>>()
            .join("\n")
    };

    // --- Mission Bias ---
    let filesystem_bias_mandate = if has_file_system_capability(ctx) {
        "DATA_ACCESS: PERMITTED. You represent a specialist with localized I/O authority.".to_string()
    } else {
        "DATA_ACCESS: RESTRICTED. Direct filesystem mutation is disabled. Use swarm recruitment for I/O.".to_string()
    };

    // --- Lineage Display ---
    let is_orchestrator = ctx.agent_id == AGENT_CEO || ctx.agent_id == AGENT_COO || ctx.agent_id == AGENT_ALPHA;
    let lineage_display = if ctx.lineage.is_empty() {
        "None (You are the root node)".to_string()
    } else {
        let path_parts: Vec<String> = ctx
            .lineage
            .iter()
            .map(|id| {
                if is_orchestrator {
                    id.clone()
                } else {
                    match id.as_str() {
                        AGENT_CEO => "OVERLORD".to_string(),
                        AGENT_COO => "ORCHESTRATOR".to_string(),
                        AGENT_ALPHA => "COMMANDER".to_string(),
                        _ => id.clone(),
                    }
                }
            })
            .collect();
        path_parts.join(" -> ")
    };

    // --- Prefixes ---
    let safe_mode_prefix = if ctx.safe_mode {
        "🔒 TOOLS FORBIDDEN: You are in CONVERSATIONAL MODE. Do NOT attempt to call any tools. \
         Respond directly with natural language. You are chatting with the Overlord (user). \
         Be helpful, insightful, and conversational. No tool calls, no mission completion — just talk.\n".to_string()
    } else {
        "🔓 SECURITY: ACTIVE. You have authority to propose and execute mutations within your workspace scope.\n".to_string()
    };

    let tool_mode_prefix = if ctx.analysis {
        "🛠️ TOOL EVOLUTION: You are in ANALYSIS mode. Propose architectural changes.".to_string()
    } else {
        "⚙️ TOOL REFINEMENT: Standard operation.".to_string()
    };

    (cluster_directory, filesystem_bias_mandate, lineage_display, is_orchestrator, safe_mode_prefix, tool_mode_prefix)
}

pub fn has_file_system_capability(ctx: &RunContext) -> bool {
    ctx.skills.iter().any(|s| {
        s == "filesystem"
            || s == "read_file"
            || s == "write_file"
            || s == "list_files"
            || s == "delete_file"
    })
}

pub fn generate_skill_fragments(ctx: &RunContext, state: &AppState) -> String {
    let mut skill_fragments = Vec::new();
    for skill_name in &ctx.skills {
        if let Some(skill_entry) = state.registry.skills.skills.get(skill_name) {
            let skill = skill_entry.value();
            if let Some(instructions) = &skill.full_instructions {
                skill_fragments.push(format!("### [{}] Full Instructions:\n{}", skill.name, instructions));
            }
            if let Some(constraints) = &skill.negative_constraints {
                if !constraints.is_empty() {
                    skill_fragments.push(format!("### [{}] Negative Constraints (PROHIBITED USES):\n- {}", skill.name, constraints.join("\n- ")));
                }
            }
        }
    }
    if skill_fragments.is_empty() { String::new() } else { format!("\nSKILL-SPECIFIC DEEP INSTRUCTIONS:\n{}\n", skill_fragments.join("\n\n")) }
}

pub fn generate_workflow_fragments(ctx: &RunContext, state: &AppState) -> String {
    let mut workflow_fragments = Vec::new();
    for workflow_name in &ctx.workflows {
        if let Some(wf_entry) = state.registry.skills.workflows.get(workflow_name) {
            let wf = wf_entry.value();
            workflow_fragments.push(format!("### [{}] Workflow Procedure:\n{}", wf.name, wf.content));
        }
    }
    if workflow_fragments.is_empty() { String::new() } else { format!("\nWORKFLOW-SPECIFIC PROCEDURES:\n{}\n", workflow_fragments.join("\n\n")) }
}

// Metadata: [fragments]

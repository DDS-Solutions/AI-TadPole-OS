//! @docs ARCHITECTURE:Agent
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Agent Runner / socratic
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Type-safe state handling and bounded execution without unhandled panics.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: none declared
//! - **Witness Tests**: none declared

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct ScopeContract {
    pub target_paths: Vec<String>,
    pub blast_radius_level: u8,
    pub mutation_allowed: bool,
    pub database_mutation_allowed: bool,
    pub mission_vector: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct PerformanceThreshold {
    pub budget_usd_cap: f64,
    pub max_swarm_depth: u32,
    pub target_turn_latency_ms: u32,
    pub active_model_slot: u8,
    pub timeout_seconds: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct ArchitectureMode {
    pub mode_name: String,
    pub privacy_shield_enforced: bool,
    pub standards_compliance: Vec<String>,
    pub assigned_persona: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct SocraticContextEnvelope {
    pub envelope_version: String,
    pub status: String,
    pub target_agent_id: String,
    pub target_agent_name: String,
    pub target_agent_role: String,
    pub scope_contract: ScopeContract,
    pub performance_threshold: PerformanceThreshold,
    pub architecture_mode: ArchitectureMode,
    pub pre_cleared_failure_modes: Vec<String>,
}

impl SocraticContextEnvelope {
    /// Compiles a standard deterministic Socratic Context Envelope for an agent execution context.
    pub fn compile(
        agent_id: &str,
        agent_name: &str,
        agent_role: &str,
        mission_vector: &str,
        target_paths: Option<Vec<String>>,
        budget_cap: Option<f64>,
        active_slot: Option<u8>,
        is_privacy_mode: bool,
    ) -> Self {
        let slot = active_slot.unwrap_or(2);
        let latency_target = if slot == 2 { 1500 } else { 10000 };
        let paths =
            target_paths.unwrap_or_else(|| vec!["src/".to_string(), "server-rs/src/".to_string()]);

        let role_lower = agent_role.to_lowercase();
        let vector_lower = mission_vector.to_lowercase();
        let is_security_or_audit = vector_lower.contains("audit")
            || vector_lower.contains("security")
            || role_lower.contains("security")
            || role_lower.contains("auditor")
            || role_lower.contains("qa");

        let is_read_only = vector_lower.contains("read")
            || vector_lower.contains("inspect")
            || vector_lower.contains("query")
            || (is_security_or_audit
                && !vector_lower.contains("fix")
                && !vector_lower.contains("remediate")
                && !vector_lower.contains("refactor")
                && !vector_lower.contains("build"));

        let mutation_allowed = !is_read_only;
        let blast_radius_level = if is_read_only {
            0
        } else if is_security_or_audit {
            3
        } else {
            2
        };
        let database_mutation_allowed =
            vector_lower.contains("migrate") || vector_lower.contains("schema");

        let mode_name = if is_security_or_audit {
            "Nexus Engineer Mode (Zero Trust Auditor + Principal QA)".to_string()
        } else {
            "Sovereign 3-Layer Architecture (Directives -> Orchestration -> Execution)".to_string()
        };

        Self {
            envelope_version: "1.0".to_string(),
            status: "PRE_CLEARED_GATE_PASS".to_string(),
            target_agent_id: agent_id.to_string(),
            target_agent_name: agent_name.to_string(),
            target_agent_role: agent_role.to_string(),
            scope_contract: ScopeContract {
                target_paths: paths,
                blast_radius_level,
                mutation_allowed,
                database_mutation_allowed,
                mission_vector: mission_vector.to_string(),
            },
            performance_threshold: PerformanceThreshold {
                budget_usd_cap: budget_cap.unwrap_or(1.0),
                max_swarm_depth: 5,
                target_turn_latency_ms: latency_target,
                active_model_slot: slot,
                timeout_seconds: 300,
            },
            architecture_mode: ArchitectureMode {
                mode_name,
                privacy_shield_enforced: is_privacy_mode,
                standards_compliance: vec![
                    "Zero Trust".to_string(),
                    "L1/L2/L3 agentskills.io".to_string(),
                    "DESIGN.md".to_string(),
                ],
                assigned_persona: agent_role.to_string(),
            },
            pre_cleared_failure_modes: vec![
                "Circuit Breaker: Halt after 3 non-convergent iterations as Logic-Blocker (Directive #3).".to_string(),
                "Air-Gap Shield: Local Ollama fallback strictly enforced if cloud providers blocked (PRIVACY_MODE).".to_string(),
                "Tool Execution Rule: Check execution/ for existing tools before writing ad-hoc scripts (Layer 1/2/3).".to_string(),
                "Verification Gate: Require automated validation pass (parity_guard / vitest) before state commit.".to_string(),
            ],
        }
    }

    /// Formats the Socratic Context Envelope as markdown for prompt pre-injection.
    pub fn to_markdown(&self) -> String {
        let paths = self.scope_contract.target_paths.join(", ");
        let mut md = format!(
            "<!-- SOCRATIC_GATE_ENVELOPE: PRE-CLEARED -->\n\
            ### 🛡️ Pre-Injected Socratic Context Contract (Zero-Stall Gate Pass)\n\
            *Target Node: `{}` (`{}`) | Mission: {}*\n\n\
            1. 🎯 **[SCOPE_CONTRACT]**\n\
               - **Target Paths**: `{}`\n\
               - **Blast Radius Level**: Level {} (Mutations: {})\n\
               - **Database Mutation**: {}\n\n\
            2. ⚡ **[PERFORMANCE_THRESHOLD]**\n\
               - **Fiscal Budget Cap**: `${:.2} USD` (Local Mode: Zero Egress)\n\
               - **Max Swarm Depth**: `Depth <= {}` | **Turn Latency Target**: `< {}ms` (Slot {})\n\
               - **Execution Timeout**: `{}s`\n\n\
            3. 🏛️ **[ARCHITECTURE_MODE]**\n\
               - **Active Mode**: `{}`\n\
               - **Privacy Shield**: `{}`\n\
               - **Governance Compliance**: `Zero Trust`, `L1/L2/L3 agentskills.io`, `DESIGN.md`\n\n\
            4. ⚖️ **[PRE-CLEARED FAILURE POLICIES & TRADE-OFFS]**\n",
            self.target_agent_name,
            self.target_agent_role,
            self.scope_contract.mission_vector,
            paths,
            self.scope_contract.blast_radius_level,
            if self.scope_contract.mutation_allowed { "Allowed" } else { "Read-Only" },
            if self.scope_contract.database_mutation_allowed { "Allowed" } else { "Blocked (Read-Only Registry)" },
            self.performance_threshold.budget_usd_cap,
            self.performance_threshold.max_swarm_depth,
            self.performance_threshold.target_turn_latency_ms,
            self.performance_threshold.active_model_slot,
            self.performance_threshold.timeout_seconds,
            self.architecture_mode.mode_name,
            if self.architecture_mode.privacy_shield_enforced { "ENFORCED (100% Local Air-Gap)" } else { "Standard" },
        );

        for policy in &self.pre_cleared_failure_modes {
            md.push_str(&format!("   - {}\n", policy));
        }
        md.push_str("<!-- /SOCRATIC_GATE_ENVELOPE -->\n\n");
        md
    }

    /// Auto-injects the Socratic markdown header into the initial prompt text if not already present.
    pub fn inject_into_prompt(&self, prompt: &str) -> String {
        if prompt.contains("SOCRATIC_GATE_ENVELOPE") {
            prompt.to_string()
        } else {
            format!("{}{}", self.to_markdown(), prompt)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_socratic_envelope_compilation() {
        let envelope = SocraticContextEnvelope::compile(
            "99",
            "Agent 99 (QA-99)",
            "quality-auditor",
            "System Audit",
            Some(vec!["src/".to_string()]),
            Some(1.50),
            Some(2),
            true,
        );

        assert_eq!(envelope.target_agent_id, "99");
        assert_eq!(envelope.performance_threshold.active_model_slot, 2);
        assert_eq!(envelope.performance_threshold.target_turn_latency_ms, 1500);
        assert!(envelope.architecture_mode.privacy_shield_enforced);
        assert!(envelope
            .architecture_mode
            .mode_name
            .contains("Nexus Engineer Mode"));

        let md = envelope.to_markdown();
        assert!(md.contains("SOCRATIC_GATE_ENVELOPE: PRE-CLEARED"));
        assert!(md.contains("Target Node: `Agent 99 (QA-99)`"));
        assert!(md.contains("Turn Latency Target"));
        assert!(md.contains("1500ms"));
    }

    #[test]
    fn test_socratic_envelope_injection() {
        let envelope = SocraticContextEnvelope::compile(
            "system_architect",
            "System Architect",
            "architect",
            "Feature Blueprint",
            None,
            None,
            None,
            false,
        );

        let base_prompt = "Build the new telemetry widget.";
        let injected = envelope.inject_into_prompt(base_prompt);

        assert!(injected.starts_with("<!-- SOCRATIC_GATE_ENVELOPE: PRE-CLEARED -->"));
        assert!(injected.ends_with("Build the new telemetry widget."));

        // Idempotence: do not inject twice
        let injected_twice = envelope.inject_into_prompt(&injected);
        assert_eq!(injected, injected_twice);
    }

    #[test]
    fn test_socratic_dynamic_scoping() {
        // Read-only inspection vector
        let read_envelope = SocraticContextEnvelope::compile(
            "auditor_1",
            "Auditor",
            "security-auditor",
            "Security Audit and Inspection",
            None,
            None,
            None,
            false,
        );
        assert!(!read_envelope.scope_contract.mutation_allowed);
        assert_eq!(read_envelope.scope_contract.blast_radius_level, 0);

        // Active refactoring vector
        let refactor_envelope = SocraticContextEnvelope::compile(
            "dev_1",
            "Developer",
            "fullstack-developer",
            "Feature Refactor and Fix",
            None,
            None,
            None,
            false,
        );
        assert!(refactor_envelope.scope_contract.mutation_allowed);
        assert_eq!(refactor_envelope.scope_contract.blast_radius_level, 2);
    }
}

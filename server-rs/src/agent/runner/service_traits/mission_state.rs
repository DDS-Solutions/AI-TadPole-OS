//! @docs ARCHITECTURE:State
//!
//! ### AI Assist Note
//! - **Subsystem**: Sovereign Engine / Agent Runner / service_traits / mission_state
//! - **Architecture**: `@docs ARCHITECTURE:State`
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural] [Version Sync]` After every successful `save_agent_db`, the in-memory `registry.agents` version
//!   MUST be updated to prevent optimistic locking conflicts in concurrent runs.
//! - `[Structural] [Sentinel Gate]` `requires_verification` is the policy predicate for the complete_mission gate.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: `Failed to sync agent status update to database`
//! - **Telemetry Targets**: none declared
//! - **Witness Tests**: `test_requires_verification`

/// Determines whether a mission requires passing deterministic verification before completion.
pub fn requires_verification(
    modified_files: &[String],
    commands_run: &std::collections::HashSet<String>,
    safe_mode: bool,
) -> bool {
    let has_mutations = !modified_files.is_empty() || !commands_run.is_empty();
    has_mutations || !safe_mode
}

#[derive(Clone, Copy, Default, Debug)]
pub struct DefaultMissionStateManager;

#[async_trait::async_trait]
impl super::ports::MissionStateManager for DefaultMissionStateManager {
    async fn yield_phase_transition(
        &self,
        state: &crate::state::AppState,
        agent_id: &str,
        phase: &str,
    ) {
        state.yield_phase_transition(agent_id, phase).await;
    }

    fn update_status(
        &self,
        state: &crate::state::AppState,
        agent_id: &str,
        mission_id: &str,
        status: &str,
        task: Option<&str>,
    ) {
        if let Some(mut entry) = state.registry.agents.get_mut(agent_id) {
            let agent = entry.value_mut();
            agent.health.status = status.to_string();
            agent.state.current_task = task.map(|t| t.to_string());

            // Sync active mission for high-speed pulse telemetry
            if status == "idle" {
                agent.state.active_mission = None;
            } else if agent.state.active_mission.is_none() {
                agent.state.active_mission = Some(serde_json::json!({ "id": mission_id }));
            }

            // Sync status to the database (F-10)
            let pool = state.resources.pool.clone();
            let registry = state.registry.clone();
            let mut agent_clone = agent.clone();
            let agent_id_owned = agent_id.to_string();
            tokio::spawn(async move {
                if let Err(e) =
                    crate::agent::persistence::save_agent_db(&pool, &mut agent_clone).await
                {
                    tracing::error!("❌ Failed to sync agent status update to database: {}", e);
                } else if let Some(mut entry) = registry.agents.get_mut(&agent_id_owned) {
                    if entry.version < agent_clone.version {
                        entry.version = agent_clone.version;
                    }
                }
            });
        }

        let task_data = state
            .registry
            .agents
            .get(agent_id)
            .and_then(|a| a.state.current_task.clone());

        let _ = state.comms.telemetry_tx.send(serde_json::json!({
            "type": "agent:status",
            "agent_id": agent_id,
            "mission_id": mission_id,
            "status": status,
            "current_task": task_data
        }));
    }

    async fn set_mission_spec(
        &self,
        state: &crate::state::AppState,
        mission_id: &str,
        agent_id: &str,
        spec_content: &str,
    ) -> Result<(), crate::error::AppError> {
        crate::agent::mission::set_mission_spec(
            &state.resources.pool,
            mission_id,
            agent_id,
            spec_content,
        )
        .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_requires_verification() {
        let empty_mods = vec![];
        let empty_cmds = std::collections::HashSet::new();

        let with_mods = vec!["src/main.rs".to_string()];
        let mut with_cmds = std::collections::HashSet::new();
        with_cmds.insert("cargo build".to_string());

        // Safe mode with NO mutations: verification NOT required
        assert!(!requires_verification(&empty_mods, &empty_cmds, true));

        // Mutating missions ALWAYS require verification even if safe_mode is true
        assert!(requires_verification(&with_mods, &empty_cmds, true));
        assert!(requires_verification(&empty_mods, &with_cmds, true));

        // Normal mode (safe_mode = false) always requires verification
        assert!(requires_verification(&empty_mods, &empty_cmds, false));
    }
}

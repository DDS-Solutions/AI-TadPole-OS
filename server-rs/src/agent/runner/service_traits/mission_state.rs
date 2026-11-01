//! @docs ARCHITECTURE:State
//!
//! ### AI Assist Note
//! - **Subsystem**: Sovereign Engine / Agent Runner / service_traits / mission_state
//! - **Architecture**: `@docs ARCHITECTURE:State`
//!
//! - Background status saves share write ordering and cancellation ownership with rollback and stop.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: `Failed to sync agent status update to database`
//! - **Telemetry Targets**: none declared

mod outcome;
pub(crate) use outcome::{persist_agent_outcome, AgentOutcome, OutcomeHealth};

/// Apply only lifecycle-owned fields; configuration and economics have other owners.
fn apply_status(
    agent: &mut crate::agent::types::EngineAgent,
    mission_id: &str,
    status: &str,
    task: Option<&str>,
) {
    agent.health.status = status.to_string();
    agent.state.current_task = task.map(str::to_string);
    if status == "idle" {
        agent.state.active_mission = None;
    } else if agent
        .state
        .active_mission
        .as_ref()
        .and_then(|mission| mission.get("id"))
        .and_then(serde_json::Value::as_str)
        != Some(mission_id)
    {
        agent.state.active_mission = Some(serde_json::json!({
            "id": mission_id,
            "started_at": std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_millis() as u64,
        }));
    }
}

/// Synchronous lifecycle transition for runner/control paths that own their persistence.
pub(crate) fn transition_agent_status(
    state: &crate::state::AppState,
    agent_id: &str,
    mission_id: &str,
    status: &str,
    task: Option<&str>,
) {
    if let Some(mut agent) = state.registry.agents.get_mut(agent_id) {
        apply_status(&mut agent, mission_id, status, task);
    }
}

pub(crate) fn publish_agent_status(
    state: &crate::state::AppState,
    agent_id: &str,
    mission_id: &str,
    status: &str,
) {
    let agent_info = state.registry.agents.get(agent_id).map(|a| {
        let task = a.state.current_task.clone();
        let tokens = a.economics.tokens_used;
        let elapsed_ms = a.state.active_mission.as_ref().and_then(|m| {
            m.get("started_at")
                .and_then(|v| v.as_u64())
                .map(|started_at| {
                    let now_ms = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_millis() as u64;
                    now_ms.saturating_sub(started_at)
                })
        });
        (task, tokens, elapsed_ms)
    });

    let (task, tokens_used, elapsed_ms) = match agent_info {
        Some((t, tok, el)) => (t, tok, el),
        None => {
            tracing::warn!(
                "⚠️ [Telemetry] Attempted to broadcast status for unregistered agent '{}'",
                agent_id
            );
            (None, 0, None)
        }
    };

    let _ = state.comms.telemetry_tx.send(serde_json::json!({
        "type": "agent:status",
        "agent_id": agent_id,
        "mission_id": mission_id,
        "status": status,
        "current_task": task,
        "tokens_used_so_far": tokens_used,
        "elapsed_ms": elapsed_ms,
    }));
}

/// Determines whether a mission requires passing deterministic verification before completion.
pub fn requires_verification(
    modified_files: &[String],
    commands_run: &std::collections::HashSet<String>,
    safe_mode: bool,
) -> bool {
    let has_mutations = !modified_files.is_empty() || !commands_run.is_empty();
    has_mutations || !safe_mode
}

/// Execution evidence is independent of model prose and operator sign-off.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum VerificationStatus {
    Verified,
    NotRequired,
    ReviewRequired,
}

impl VerificationStatus {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Verified => "verified",
            Self::NotRequired => "not_required",
            Self::ReviewRequired => "unverified",
        }
    }
}

pub(crate) fn verification_status(ctx: &super::super::RunContext) -> VerificationStatus {
    let last_mutation = ctx
        .last_mutation_timestamp_ms
        .load(std::sync::atomic::Ordering::Acquire);
    if super::super::oversight::verify_execution_witnesses(
        &ctx.execution_records.lock(),
        last_mutation,
    ) {
        VerificationStatus::Verified
    } else if requires_verification(
        &ctx.modified_files.lock(),
        &ctx.commands_run.lock(),
        ctx.safe_mode,
    ) {
        VerificationStatus::ReviewRequired
    } else {
        VerificationStatus::NotRequired
    }
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
        let Ok((persistence_guard, cancellation)) = state.comms.begin_execution(agent_id) else {
            return;
        };
        let generation = state.comms.agent_generation(agent_id);
        if cancellation.is_cancelled() {
            return;
        }
        if let Some(mut entry) = state.registry.agents.get_mut(agent_id) {
            let agent = entry.value_mut();
            apply_status(agent, mission_id, status, task);

            // Sync status to the database (F-10)
            let pool = state.resources.pool.clone();
            let registry = state.registry.clone();
            let comms = state.comms.clone();
            let agent_id_owned = agent_id.to_string();
            tokio::spawn(async move {
                let _persistence = persistence_guard;
                let _write = comms.agent_write_gate.lock().await;
                if cancellation.is_cancelled()
                    || comms.agent_generation(&agent_id_owned) != generation
                {
                    return;
                }
                // Read after prior writes publish their versions; coalesce to the current state.
                let Some(mut agent_clone) = registry
                    .agents
                    .get(&agent_id_owned)
                    .map(|entry| entry.value().clone())
                else {
                    return;
                };
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

        publish_agent_status(state, agent_id, mission_id, status);
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
    use super::super::ports::MissionStateManager;
    use super::*;

    #[tokio::test]
    async fn status_save_is_owned_by_stop_and_preserves_paused_state() {
        let state = std::sync::Arc::new(crate::state::AppState::new_minimal_mock().await);
        let mut agent = crate::agent::types::EngineAgent::default();
        agent.identity.id = "status-owner".to_string();
        crate::agent::persistence::save_agent_db(&state.resources.pool, &mut agent)
            .await
            .unwrap();
        state
            .registry
            .agents
            .insert(agent.identity.id.clone(), agent);

        let write = state.comms.agent_write_gate.lock().await;
        DefaultMissionStateManager.update_status(
            &state,
            "status-owner",
            "mission",
            "busy",
            Some("old task"),
        );
        let stopped_state = state.clone();
        let stopped = tokio::spawn(async move {
            stopped_state
                .comms
                .stop_executions(Some("status-owner"))
                .await
        });
        tokio::time::timeout(std::time::Duration::from_secs(5), async {
            while !state.comms.stopped_agents.contains("status-owner") {
                tokio::task::yield_now().await;
            }
        })
        .await
        .unwrap();
        assert!(
            !stopped.is_finished(),
            "Stop must wait for its pending status save"
        );
        drop(write);
        stopped.await.unwrap().unwrap();
        assert!(state.comms.active_executions.is_empty());
        state
            .registry
            .agents
            .get_mut("status-owner")
            .unwrap()
            .health
            .status = "suspended".to_string();
        state.save_agents().await.unwrap();
        let saved =
            crate::agent::persistence::load_agent_by_id_db(&state.resources.pool, "status-owner")
                .await
                .unwrap()
                .unwrap();
        assert_eq!(saved.health.status, "suspended");
        assert_eq!(
            saved.version,
            state.registry.agents.get("status-owner").unwrap().version
        );
    }

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

    #[test]
    fn verification_requires_fresh_success_and_retains_read_only_status() {
        let mut ctx = crate::agent::runner::RunContext::default();
        assert_eq!(
            verification_status(&ctx),
            VerificationStatus::ReviewRequired
        );
        ctx.safe_mode = true;
        assert_eq!(verification_status(&ctx), VerificationStatus::NotRequired);
        ctx.modified_files.lock().push("source.rs".into());
        ctx.last_mutation_timestamp_ms
            .store(100, std::sync::atomic::Ordering::Release);
        ctx.execution_records
            .lock()
            .push(crate::agent::runner::ProcessExecutionRecord {
                command: "cargo test".into(),
                exit_code: 0,
                timestamp_ms: 99,
            });
        assert_eq!(
            verification_status(&ctx),
            VerificationStatus::ReviewRequired
        );
        ctx.execution_records.lock()[0].timestamp_ms = 101;
        assert_eq!(verification_status(&ctx), VerificationStatus::Verified);
        ctx.execution_records.lock()[0].exit_code = 1;
        assert_eq!(
            verification_status(&ctx),
            VerificationStatus::ReviewRequired
        );
    }
}

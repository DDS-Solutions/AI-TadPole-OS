//! @docs ARCHITECTURE:ModelRouting
//!
//! ### AI Assist Note
//! - **Subsystem**: Sovereign Engine / Agent Runner / service_traits / routing
//! - **Architecture**: `@docs ARCHITECTURE:ModelRouting`
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural] [Privacy]` In privacy mode, `DefaultModelRouter` MUST resolve a local model — never a cloud model.
//! - `[Structural] [Fallback]` If no local slot is configured, synthesize an Ollama fallback config.
//!
//! ### 🔍 Debugging & Observability
//! - **Witness Tests**: `test_agent_mission_state_resolve`

#[derive(Clone, Copy, Default, Debug)]
pub struct DefaultModelRouter;

impl DefaultModelRouter {
    pub fn select_model_slot(
        &self,
        state: &crate::state::AppState,
        models: &crate::agent::types::AgentModels,
        preferred: super::ports::SlotKind,
        cluster_id: Option<&str>,
    ) -> super::ports::SlotSelection {
        <Self as super::ports::ModelRouter>::select_model_slot(
            self, state, models, preferred, cluster_id,
        )
    }
}

impl super::ports::ModelRouter for DefaultModelRouter {
    fn select_model_slot(
        &self,
        state: &crate::state::AppState,
        models: &crate::agent::types::AgentModels,
        preferred: super::ports::SlotKind,
        cluster_id: Option<&str>,
    ) -> super::ports::SlotSelection {
        let privacy_mode_active = state.governance.is_privacy_mode_enabled(cluster_id);

        let preferred_config = match preferred {
            super::ports::SlotKind::Planning => models.planning_slot.as_ref(),
            super::ports::SlotKind::Execution => models.execution_slot.as_ref(),
            super::ports::SlotKind::Default => Some(&models.model),
        };

        if privacy_mode_active {
            // 1. Preferred local slot
            if let Some(config) = preferred_config {
                if crate::agent::model_routing::is_local_model_config(config) {
                    return super::ports::SlotSelection {
                        config: config.clone(),
                        kind: preferred,
                        privacy_local_override: true,
                    };
                }
            }

            // 2. Default local model
            if crate::agent::model_routing::is_local_model_config(&models.model) {
                return super::ports::SlotSelection {
                    config: models.model.clone(),
                    kind: super::ports::SlotKind::Default,
                    privacy_local_override: true,
                };
            }

            // 3. Alternate local slot
            let alternate = match preferred {
                super::ports::SlotKind::Planning => {
                    Some((super::ports::SlotKind::Execution, &models.execution_slot))
                }
                super::ports::SlotKind::Execution => {
                    Some((super::ports::SlotKind::Planning, &models.planning_slot))
                }
                super::ports::SlotKind::Default => None,
            };
            if let Some((kind, Some(config))) = alternate {
                if crate::agent::model_routing::is_local_model_config(config) {
                    return super::ports::SlotSelection {
                        config: config.clone(),
                        kind,
                        privacy_local_override: true,
                    };
                }
            }

            // 4. Synthesized Ollama fallback
            return super::ports::SlotSelection {
                config: crate::agent::model_routing::privacy_fallback_config(),
                kind: super::ports::SlotKind::Default,
                privacy_local_override: true,
            };
        }

        if let Some(config) = preferred_config {
            return super::ports::SlotSelection {
                config: config.clone(),
                kind: preferred,
                privacy_local_override: false,
            };
        }

        super::ports::SlotSelection {
            config: models.model.clone(),
            kind: super::ports::SlotKind::Default,
            privacy_local_override: false,
        }
    }
}

#[derive(Clone, Copy, Default, Debug)]
pub struct DefaultPromptService;

impl DefaultPromptService {
    pub async fn build_system_prompt(
        &self,
        runner: &super::super::AgentRunner,
        ctx: &super::super::RunContext,
        payload_message: &str,
    ) -> String {
        <Self as super::ports::PromptService>::build_system_prompt(
            self,
            runner,
            ctx,
            payload_message,
        )
        .await
    }
}

#[async_trait::async_trait]
impl super::ports::PromptService for DefaultPromptService {
    async fn build_system_prompt(
        &self,
        runner: &super::super::AgentRunner,
        ctx: &super::super::RunContext,
        payload_message: &str,
    ) -> String {
        runner.build_system_prompt(ctx, payload_message).await
    }
}

#[cfg(test)]
mod tests {
    use super::super::ports::AgentMissionState;

    #[test]
    fn test_agent_mission_state_resolve() {
        assert_eq!(
            AgentMissionState::resolve("some random notes", false, false),
            AgentMissionState::SpecificationGeneration
        );
        assert_eq!(
            AgentMissionState::resolve("--- [ROOM: system::spec] ---\nContent", false, false),
            AgentMissionState::Reasoning
        );
        assert_eq!(
            AgentMissionState::resolve("## Unified Technical Specification\nContent", false, false),
            AgentMissionState::Reasoning
        );
        assert_eq!(
            AgentMissionState::resolve("some notes", true, false),
            AgentMissionState::Reasoning
        );
        assert_eq!(
            AgentMissionState::resolve("some notes", false, true),
            AgentMissionState::Reasoning
        );
    }
}

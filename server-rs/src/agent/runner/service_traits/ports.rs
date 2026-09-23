//! @docs ARCHITECTURE:Core
//!
//! ### AI Assist Note
//! - **Subsystem**: Sovereign Engine / Agent Runner / service_traits / ports
//! - **Architecture**: `@docs ARCHITECTURE:Core` — Interface Contract Layer
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural] [Stability]` Trait signatures in this file are the public API contract — changes require a
//!   full blast-radius audit via `npm run graph:blast:guard`.
//! - `[Structural] [Async Safety]` All async traits MUST use `#[async_trait::async_trait]`.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: none declared

use crate::agent::types::RoleAuthorityLevel;
use std::collections::HashMap;

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
    fn is_tool_allowed(
        &self,
        agent_id: &str,
        role: &str,
        authority: RoleAuthorityLevel,
        tool_name: &str,
    ) -> bool;

    /// Returns mandatory protocols for a given agent and role.
    fn get_role_protocols(
        &self,
        agent_id: &str,
        role: &str,
        authority: RoleAuthorityLevel,
    ) -> Vec<String>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SlotKind {
    Planning,
    Execution,
    Default,
}

#[derive(Debug, Clone)]
pub struct SlotSelection {
    pub config: crate::agent::types::ModelConfig,
    pub kind: SlotKind,
    pub privacy_local_override: bool,
}

#[derive(Debug, Clone)]
pub struct ToolOrchestrationResult {
    pub observation_buffer: String,
    pub mission_completed: bool,
    pub final_report: Option<String>,
    pub active_slot_override: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum AgentMissionState {
    Initial,
    SpecificationGeneration,
    AwaitingToolCalls,
    Reasoning,
    Execution,
    Finalizing,
    Halted,
}

impl AgentMissionState {
    /// Resolves the current mission phase from the spec content and mode flags.
    pub fn resolve(spec: &str, safe_mode: bool, is_fast_path: bool) -> Self {
        if safe_mode || is_fast_path {
            return AgentMissionState::Reasoning;
        }
        if !spec.contains("--- [ROOM: system::spec] ---")
            && !spec.contains("## Unified Technical Specification")
        {
            AgentMissionState::SpecificationGeneration
        } else {
            AgentMissionState::Reasoning
        }
    }
}

pub trait ModelRouter: Send + Sync {
    fn select_model_slot(
        &self,
        state: &crate::state::AppState,
        models: &crate::agent::types::AgentModels,
        preferred: SlotKind,
        cluster_id: Option<&str>,
    ) -> SlotSelection;
}

#[async_trait::async_trait]
pub trait PromptService: Send + Sync {
    async fn build_system_prompt(
        &self,
        runner: &super::super::AgentRunner,
        ctx: &super::super::RunContext,
        payload_message: &str,
    ) -> String;
}

#[async_trait::async_trait]
pub trait ToolExecutor: Send + Sync {
    async fn execute_tool(
        &self,
        ctx: &super::super::RunContext,
        fc: &crate::agent::types::ToolCall,
        user_message: &str,
    ) -> Result<(String, Option<crate::agent::types::TokenUsage>), crate::error::AppError>;

    fn update_status(&self, agent_id: &str, mission_id: &str, status: &str, task: Option<&str>);

    fn get_provider_timeout_secs(&self) -> u64;

    fn get_tool_timeout_secs(&self) -> u64 {
        let p = self.get_provider_timeout_secs();
        if p > 0 {
            p.clamp(5, 600)
        } else {
            60
        }
    }

    fn handle_tool_failure_refinement(
        &self,
        ctx: &super::super::RunContext,
        fc: &crate::agent::types::ToolCall,
        local_text: &mut String,
    );

    fn accumulate_usage(
        &self,
        accumulated: &mut Option<crate::agent::types::TokenUsage>,
        new_usage: Option<crate::agent::types::TokenUsage>,
    );

    fn verify_mission_success(&self, observation_buffer: &str) -> bool;

    fn verify_execution_witnesses(
        &self,
        records: &[super::super::ProcessExecutionRecord],
        last_mutation_ms: u64,
    ) -> bool;

    fn broadcast_agent(&self, ctx: &super::super::RunContext, msg: &str, level: &str);

    fn handle_tool_failure_slot_swap(&self, agent_id: &str) -> Option<String>;
}

#[async_trait::async_trait]
pub trait ToolOrchestrator: Send + Sync {
    async fn execute_tools(
        &self,
        executor: std::sync::Arc<dyn ToolExecutor>,
        active_ctx: &super::super::RunContext,
        function_calls: Vec<crate::agent::types::ToolCall>,
        user_message: &str,
        usage: &mut Option<crate::agent::types::TokenUsage>,
    ) -> Result<ToolOrchestrationResult, crate::error::AppError>;
}

#[async_trait::async_trait]
pub trait WorkflowCoordinator: Send + Sync {
    async fn execute_workflow(
        &self,
        runner: &super::super::AgentRunner,
        ctx: &super::super::RunContext,
        payload: &crate::agent::types::TaskPayload,
    ) -> Result<
        super::super::IntelligenceOutput,
        (
            crate::error::AppError,
            Option<crate::agent::types::TokenUsage>,
        ),
    >;
}

#[async_trait::async_trait]
#[allow(dead_code)]
pub trait MissionStateManager: Send + Sync {
    async fn yield_phase_transition(
        &self,
        state: &crate::state::AppState,
        agent_id: &str,
        phase: &str,
    );
    fn update_status(
        &self,
        state: &crate::state::AppState,
        agent_id: &str,
        mission_id: &str,
        status: &str,
        task: Option<&str>,
    );
    async fn set_mission_spec(
        &self,
        state: &crate::state::AppState,
        mission_id: &str,
        agent_id: &str,
        spec_content: &str,
    ) -> Result<(), crate::error::AppError>;
}

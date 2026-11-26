//! @docs ARCHITECTURE:Runner
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Agent Runner / step_telemetry
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: none declared

use crate::agent::runner::{AgentRunner, RunContext};

pub(crate) struct TurnInferenceResult<'a> {
    pub turn_text: &'a str,
    pub function_calls: &'a [crate::agent::types::ToolCall],
    pub turn_usage: &'a Option<crate::agent::types::TokenUsage>,
    pub latency_ms: u64,
    pub reasoning_turn: usize,
}

pub(crate) struct ReasoningStepTelemetry<'a> {
    pub step_id: &'a str,
    pub parent_id: Option<String>,
    pub content: &'a str,
    pub model: &'a str,
    pub slot: &'a str,
    pub turn_index: usize,
    pub latency_ms: u64,
    pub usage: &'a Option<crate::agent::types::TokenUsage>,
    pub finish_reason: &'a str,
}

impl AgentRunner {
    pub(crate) fn broadcast_reasoning_step(
        &self,
        ctx: &RunContext,
        data: ReasoningStepTelemetry<'_>,
    ) {
        let redacted_content =
            crate::agent::sanitizer::Sanitizer::redact_sensitive_data(data.content);
        let access_list = ctx.conductor_plan.as_ref().map(|plan| {
            plan.steps
                .iter()
                .filter(|s| s.target_agent == ctx.identity.agent_id)
                .flat_map(|s| s.access_list.clone())
                .collect::<Vec<u32>>()
        });

        let input_tokens = data.usage.as_ref().map(|u| u.input_tokens).unwrap_or(0);
        let output_tokens = data.usage.as_ref().map(|u| u.output_tokens).unwrap_or(0);
        let cost_usd = crate::agent::rates::calculate_cost_with_custom_rates(
            ctx.model_config.input_cost_per_m,
            ctx.model_config.output_cost_per_m,
            Some(&ctx.model_config.provider.to_string()),
            &ctx.model_config.model_id,
            input_tokens,
            output_tokens,
        );

        let _ = self.state.comms.telemetry_tx.send(serde_json::json!({
            "type": "agent:reasoning_step",
            "agent_id": ctx.identity.agent_id,
            "mission_id": ctx.mission_state.mission_id,
            "step": {
                "id": data.step_id,
                "parent_id": data.parent_id,
                "content": redacted_content,
                "model": data.model,
                "provider": ctx.provider_name,
                "slot": data.slot,
                "lineage": ctx.mission_state.lineage,
                "access_list": access_list,
                "turn_index": data.turn_index,
                "latency_ms": data.latency_ms,
                "input_tokens": input_tokens,
                "output_tokens": output_tokens,
                "cost_usd": cost_usd,
                "finish_reason": data.finish_reason,
            }
        }));
    }

    pub(crate) fn broadcast_tool_step(
        &self,
        ctx: &RunContext,
        parent_id: Option<String>,
        tool_names: Vec<String>,
    ) {
        let step_id = uuid::Uuid::new_v4().to_string();
        let tool_count = tool_names.len();
        let _ = self.state.comms.telemetry_tx.send(serde_json::json!({
            "type": "agent:reasoning_step",
            "agent_id": ctx.identity.agent_id,
            "mission_id": ctx.mission_state.mission_id,
            "step": {
                "id": step_id,
                "parent_id": parent_id,
                "content": "",
                "model": "tool_orchestrator",
                "slot": "tool",
                "tool_names": tool_names,
                "tool_count": tool_count,
                "lineage": ctx.mission_state.lineage,
            }
        }));
    }

    pub(crate) fn emit_turn_telemetry(
        &self,
        ctx: &RunContext,
        active_ctx: &RunContext,
        active_slot_kind: super::super::service_traits::SlotKind,
        parent_id: &mut Option<String>,
        result: TurnInferenceResult<'_>,
    ) {
        ctx.execution_metrics.record_turn();
        if let Some(ref u) = result.turn_usage {
            ctx.execution_metrics.record_tokens(
                u.input_tokens,
                u.output_tokens,
                u.input_tokens + u.output_tokens,
            );
        }
        let metrics_snap = ctx
            .execution_metrics
            .snapshot(&ctx.mission_state.mission_id, &ctx.identity.agent_id);
        let _ = self.state.comms.telemetry_tx.send(serde_json::json!({
            "type": "agent:execution_metrics",
            "metrics": metrics_snap,
        }));

        let current_step_id = uuid::Uuid::new_v4().to_string();
        let slot = match active_slot_kind {
            super::super::service_traits::SlotKind::Planning => "planning",
            super::super::service_traits::SlotKind::Execution => "execution",
            super::super::service_traits::SlotKind::Default => "default",
        };

        let finish_reason = if result.turn_text.contains("<halting_signal/>")
            || result.turn_text.contains("<halt/>")
        {
            "halted"
        } else if result.function_calls.is_empty() {
            "stop"
        } else {
            "tool_use"
        };

        self.broadcast_reasoning_step(
            ctx,
            ReasoningStepTelemetry {
                step_id: &current_step_id,
                parent_id: parent_id.clone(),
                content: result.turn_text,
                model: &active_ctx.model_config.model_id,
                slot,
                turn_index: result.reasoning_turn,
                latency_ms: result.latency_ms,
                usage: result.turn_usage,
                finish_reason,
            },
        );

        if !result.function_calls.is_empty() {
            let tool_names = result
                .function_calls
                .iter()
                .map(|fc| fc.name.clone())
                .collect();
            self.broadcast_tool_step(ctx, Some(current_step_id.clone()), tool_names);
        }

        *parent_id = Some(current_step_id);
    }
}

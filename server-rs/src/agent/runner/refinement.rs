//! @docs ARCHITECTURE:Evolution
//!
//! ### AI Assist Note
//! **Autonomous Refinement**: Analyzes tool execution failures and provides
//! proactive "Refinement Hints" to agents. Enables the self-healing swarm
//! by suggesting 'refactor_synthesized_skill' when dynamic tools break.
//!
//! ### 🔍 Debugging & Observability
//! - **Trace Scope**: `server-rs::agent::runner::refinement`

use super::{AgentRunner, RunContext};
use crate::agent::types::ToolCall;

impl AgentRunner {
    /// Analyzes a tool execution result and appends evolution hints if failures are detected.
    pub(crate) fn handle_tool_failure_refinement(
        &self,
        _ctx: &RunContext,
        fc: &ToolCall,
        output_text: &mut String,
    ) {
        let is_error = output_text.to_lowercase().contains("error") 
            || output_text.contains("FAILED") 
            || output_text.contains("Exception")
            || output_text.contains("SyntaxError");

        if !is_error {
            return;
        }

        // Check if it's a synthesized skill
        // Standard path: execution/agent_generated/skills/
        let is_synthesized = self.state.registry.skills.skills.get(&fc.name)
            .map(|s| s.category == "agent_generated")
            .unwrap_or(false);

        if is_synthesized {
            let hint = format!(
                "\n\n🧬 [Evolution Hint]: The autonomously synthesized skill '{}' failed. \
                You can use 'refactor_synthesized_skill' to fix the Python logic or update the schema. \
                Use 'get_file_contents' on 'execution/agent_generated/skills/{}.py' to debug the source.",
                fc.name, fc.name.to_lowercase().replace(' ', "_")
            );
            output_text.push_str(&hint);
            tracing::info!("🧬 [Evolution] Injected refinement hint for synthesized skill: {}", fc.name);
        } else if fc.name == "spawn_subagent" {
             let hint = "\n\n💡 [Swarm Hint]: If recruitment failed, ensure the agent ID exists or try a more general role (e.g., 'researcher', 'coder').";
             output_text.push_str(hint);
        }
    }
}

// Metadata: [refinement]

// Metadata: [refinement]

#[cfg(test)]
mod tests {
    use crate::state::AppState;
    use crate::agent::runner::{AgentRunner, RunContext};
    use crate::agent::types::ToolCall;
    use crate::agent::script_skills::SkillDefinition;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_refinement_no_error() {
        let state = Arc::new(AppState::new_minimal_mock().await);
        let runner = AgentRunner::new(state.clone());
        let ctx = RunContext::default();
        let fc = ToolCall {
            name: "test_tool".to_string(),
            args: serde_json::json!({}),
        };
        let mut output = "Everything is fine".to_string();
        
        runner.handle_tool_failure_refinement(&ctx, &fc, &mut output);
        assert_eq!(output, "Everything is fine");
    }

    #[tokio::test]
    async fn test_refinement_spawn_subagent_hint() {
        let state = Arc::new(AppState::new_minimal_mock().await);
        let runner = AgentRunner::new(state.clone());
        let ctx = RunContext::default();
        let fc = ToolCall {
            name: "spawn_subagent".to_string(),
            args: serde_json::json!({}),
        };
        let mut output = "Error: recruitment failed".to_string();
        
        runner.handle_tool_failure_refinement(&ctx, &fc, &mut output);
        assert!(output.contains("💡 [Swarm Hint]"));
    }

    #[tokio::test]
    async fn test_refinement_synthesized_skill_hint() {
        let state = Arc::new(AppState::new_minimal_mock().await);
        let runner = AgentRunner::new(state.clone());
        let ctx = RunContext::default();
        
        // Register a synthesized skill
        let skill = SkillDefinition {
            id: None,
            name: "my_dynamic_skill".to_string(),
            description: "A synthesized skill".to_string(),
            execution_command: "python skill.py".to_string(),
            schema: serde_json::json!({}),
            oversight_required: false,
            doc_url: None,
            tags: None,
            full_instructions: None,
            negative_constraints: None,
            verification_script: None,
            category: "agent_generated".to_string(),
        };
        state.registry.skills.skills.insert(skill.name.clone(), skill.clone());

        let fc = ToolCall {
            name: "my_dynamic_skill".to_string(),
            args: serde_json::json!({}),
        };
        let mut output = "SyntaxError: invalid syntax".to_string();
        
        runner.handle_tool_failure_refinement(&ctx, &fc, &mut output);
        assert!(output.contains("🧬 [Evolution Hint]"));
        assert!(output.contains("refactor_synthesized_skill"));
    }
}

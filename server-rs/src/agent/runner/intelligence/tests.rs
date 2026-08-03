//! @docs ARCHITECTURE:Runner
//!
//! ### AI Assist Note
//! **Intelligence Tests**: Unit testing suite verifying the DoomLoopDetector, sentinel gates, and model routing.
//!
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: Failures here represent logic regressions in the intelligence loop.
//!

#[cfg(test)]
#[allow(clippy::module_inception)]
mod tests {
    use super::super::loop_detector::{DoomLoopDetector, normalize_json};
    use super::super::sentinel::scrub_mythos_tags;
    use super::super::hierarchy::resolve_hierarchy_label;
    use crate::agent::constants::*;
    use crate::agent::runner::service_traits::SlotKind;
    use crate::agent::runner::{AgentRunner, RunContext};
    use crate::agent::types::{ModelConfig, ModelProvider};
    use crate::state::AppState;
    use std::sync::Arc;

    #[test]
    fn test_doom_loop_detector() {
        let mut detector = DoomLoopDetector::new();
        // Period 1 loop (A -> A -> A)
        assert!(!detector.check("1", "ls", "{}", "file1.txt"));
        assert!(!detector.check("1", "ls", "{}", "file1.txt"));
        assert!(detector.check("1", "ls", "{}", "file1.txt"));

        // Reset and test Period 2 loop (A -> B -> A -> B)
        let mut detector = DoomLoopDetector::new();
        assert!(!detector.check("1", "ls", "{}", "file1.txt"));
        assert!(!detector.check("1", "cat", "{}", "content"));
        assert!(!detector.check("1", "ls", "{}", "file1.txt"));
        assert!(detector.check("1", "cat", "{}", "content"));
    }

    #[test]
    fn test_scrub_mythos_tags() {
        let input =
            "Thinking... <thinking>I should search</thinking> <halting_signal/> Done. <halt/>";
        let expected = "Thinking... I should search  Done.";
        assert_eq!(scrub_mythos_tags(input), expected);

        let clean = "No tags here";
        assert_eq!(scrub_mythos_tags(clean), "No tags here");
    }

    #[test]
    fn test_hierarchy_labeling() {
        assert_eq!(
            resolve_hierarchy_label(AGENT_CEO, "Chief Executive Officer"),
            "CEO (Strategic Intelligence Lead)"
        );
        assert_eq!(
            resolve_hierarchy_label(AGENT_COO, "Chief Operations"),
            "COO (Operations Director)"
        );
        assert_eq!(
            resolve_hierarchy_label(AGENT_ALPHA, "Commander"),
            "ALPHA NODE (Swarm Mission Commander)"
        );
        assert_eq!(
            resolve_hierarchy_label("specialist-42", "Engineer"),
            "AGENT (Task Specialist)"
        );
        assert_eq!(
            resolve_hierarchy_label("custom-agent", "Chief COO Officer"),
            "AGENT (Task Specialist)"
        );
    }

    #[test]
    fn test_normalize_json_depth_limit() {
        let mut val = serde_json::json!("leaf");
        for _ in 0..40 {
            val = serde_json::json!({"nested": val});
        }
        let result = normalize_json(&val);
        assert!(result.contains("[DEPTH_EXCEEDED]"));
    }

    #[test]
    fn test_resolve_mission_state() {
        use crate::agent::runner::service_traits::AgentMissionState;

        let state = AgentMissionState::resolve("empty context", false, false);
        assert_eq!(state, AgentMissionState::SpecificationGeneration);

        let state = AgentMissionState::resolve("--- [ROOM: system::spec] --- some spec", false, false);
        assert_eq!(state, AgentMissionState::Reasoning);

        let state = AgentMissionState::resolve("empty context", true, false);
        assert_eq!(state, AgentMissionState::Reasoning);

        let state = AgentMissionState::resolve("empty context", false, true);
        assert_eq!(state, AgentMissionState::Reasoning);
    }

    #[tokio::test]
    async fn test_privacy_mode_prefers_local_default_for_planning_slot() {
        let state = Arc::new(AppState::new_minimal_mock().await);
        state
            .governance
            .privacy_mode
            .store(true, std::sync::atomic::Ordering::Relaxed);
        let runner = AgentRunner::new(state);

        let mut agent = crate::agent::types::EngineAgent::default();
        agent.models.model = ModelConfig {
            provider: ModelProvider::Ollama,
            model_id: "llama3-local".to_string(),
            ..Default::default()
        };
        agent.models.planning_slot = Some(ModelConfig {
            provider: ModelProvider::Anthropic,
            model_id: "Claude Opus 4.5".to_string(),
            ..Default::default()
        });

        let selection =
            runner
                .model_router
                .select_model_slot(&runner.state, &agent.models, SlotKind::Planning, None);

        assert_eq!(selection.kind, SlotKind::Default);
        assert!(selection.privacy_local_override);
        assert_eq!(selection.config.provider, ModelProvider::Ollama);
        assert_eq!(selection.config.model_id, "llama3-local");
    }

    #[tokio::test]
    async fn test_privacy_mode_keeps_local_preferred_slot() {
        let state = Arc::new(AppState::new_minimal_mock().await);
        state
            .governance
            .privacy_mode
            .store(true, std::sync::atomic::Ordering::Relaxed);
        let runner = AgentRunner::new(state);

        let mut agent = crate::agent::types::EngineAgent::default();
        agent.models.model = ModelConfig {
            provider: ModelProvider::Ollama,
            model_id: "llama3-default".to_string(),
            ..Default::default()
        };
        agent.models.execution_slot = Some(ModelConfig {
            provider: ModelProvider::Openai,
            model_id: "qwen2.5-coder-local".to_string(),
            base_url: Some("http://127.0.0.1:1234/v1".to_string()),
            ..Default::default()
        });

        let selection = runner.model_router.select_model_slot(
            &runner.state,
            &agent.models,
            SlotKind::Execution,
            None,
        );

        assert_eq!(selection.kind, SlotKind::Execution);
        assert!(selection.privacy_local_override);
        assert_eq!(selection.config.provider, ModelProvider::Openai);
        assert_eq!(selection.config.model_id, "qwen2.5-coder-local");
    }

    #[tokio::test]
    async fn test_hybrid_mode_uses_preferred_cloud_slot() {
        let state = Arc::new(AppState::new_minimal_mock().await);
        let runner = AgentRunner::new(state);

        let mut agent = crate::agent::types::EngineAgent::default();
        agent.models.model = ModelConfig {
            provider: ModelProvider::Ollama,
            model_id: "llama3-default".to_string(),
            ..Default::default()
        };
        agent.models.planning_slot = Some(ModelConfig {
            provider: ModelProvider::Anthropic,
            model_id: "Claude Opus 4.5".to_string(),
            ..Default::default()
        });

        let selection =
            runner
                .model_router
                .select_model_slot(&runner.state, &agent.models, SlotKind::Planning, None);

        assert_eq!(selection.kind, SlotKind::Planning);
        assert!(!selection.privacy_local_override);
        assert_eq!(selection.config.provider, ModelProvider::Anthropic);
        assert_eq!(selection.config.model_id, "Claude Opus 4.5");
    }

    #[tokio::test]
    async fn test_enforce_sentinel_gate_orchestrator() {
        let state = Arc::new(AppState::new_minimal_mock().await);
        let runner = AgentRunner::new(state.clone());

        let mut ctx = RunContext::default();
        ctx.agent_id = AGENT_CEO.to_string();

        let mut output_text = "I am the CEO".to_string();
        let mut function_calls = vec![];
        let mut usage = None;

        let result = runner
            .enforce_sentinel_gate(
                &ctx,
                "system",
                "user",
                &mut output_text,
                &mut function_calls,
                &mut usage,
            )
            .await;

        assert!(result.is_ok());
        assert_eq!(output_text, "I am the CEO");
        assert!(function_calls.is_empty());
    }

    #[tokio::test]
    async fn test_enforce_sentinel_gate_specialist_narrative_leak() {
        let state = Arc::new(AppState::new_minimal_mock().await);
        let runner = AgentRunner::new(state.clone());

        let mut ctx = RunContext::default();
        ctx.agent_id = "specialist-1".to_string();
        ctx.role = "Specialist".to_string();
        ctx.safe_mode = false;

        let mut output_text = "I am just talking".to_string();
        let mut function_calls = vec![];
        let mut usage = None;

        let result = runner
            .enforce_sentinel_gate(
                &ctx,
                "system",
                "user",
                &mut output_text,
                &mut function_calls,
                &mut usage,
            )
            .await;

        assert!(result.is_ok());
        assert!(output_text.contains("DEGRADED"));
    }

    #[tokio::test]
    async fn test_compress_monologue() {
        let state = Arc::new(AppState::new_minimal_mock().await);
        let runner = AgentRunner::new(state.clone());
        let ctx = RunContext::default();

        let mut monologue = vec!["Turn 1".to_string(), "Turn 2".to_string()];
        let res = runner.compress_monologue(&ctx, &mut monologue).await;
        assert!(res.is_ok());
        assert_eq!(monologue.len(), 2);
        assert_eq!(monologue[0], "Turn 1");

        let input_with_large_log =
            "Some reasoning\n```json\n".to_string() + &"a".repeat(2500) + "\n```";
        let truncated = AgentRunner::truncate_embedded_tool_logs(&input_with_large_log);
        assert!(truncated.contains("[Raw tool result evicted to save context"));
        assert!(!truncated.contains(&"a".repeat(2500)));

        let history = "Some long paragraph that will be kept.\n```\nbody of code block\n```\nAnother short line.";
        let summary = AgentRunner::deterministic_fallback_summarize(history);
        assert!(summary.contains("DETERMINISTIC FALLBACK SUMMARY"));
        assert!(summary.contains("[Code block header omitted]"));
        assert!(!summary.contains("body of code block"));

        let mut monologue = vec![
            "Huge turn 1: ".to_string() + &"a".repeat(5000) + "\n```\ncode block\n```",
            "Huge turn 2: ".to_string() + &"b".repeat(4000),
            "Tail 1".to_string(),
            "Tail 2".to_string(),
            "Tail 3".to_string(),
            "Tail 4".to_string(),
        ];
        let res = runner.compress_monologue(&ctx, &mut monologue).await;
        assert!(res.is_ok());
        assert_eq!(monologue.len(), 5);
        assert_eq!(monologue[1], "Tail 1");
        assert_eq!(monologue[2], "Tail 2");
        assert_eq!(monologue[3], "Tail 3");
        assert_eq!(monologue[4], "Tail 4");
    }

    #[tokio::test]
    async fn test_privacy_mode_all_cloud_returns_synthesized_ollama_fallback() {
        let state = Arc::new(AppState::new_minimal_mock().await);
        state
            .governance
            .privacy_mode
            .store(true, std::sync::atomic::Ordering::Relaxed);
        let runner = AgentRunner::new(state);

        let mut agent = crate::agent::types::EngineAgent::default();
        agent.models.model = ModelConfig {
            provider: ModelProvider::Openai,
            model_id: "gpt-4-cloud-default".to_string(),
            ..Default::default()
        };
        agent.models.planning_slot = Some(ModelConfig {
            provider: ModelProvider::Anthropic,
            model_id: "Claude-cloud-planning".to_string(),
            ..Default::default()
        });

        let selection =
            runner
                .model_router
                .select_model_slot(&runner.state, &agent.models, SlotKind::Planning, None);

        assert_eq!(selection.kind, SlotKind::Default);
        assert!(selection.privacy_local_override);
        assert_eq!(selection.config.provider, ModelProvider::Ollama);
        assert_eq!(selection.config.model_id, "phi3.5-safe:latest");
    }
}

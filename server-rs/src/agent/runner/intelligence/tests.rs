//! @docs ARCHITECTURE:Runner
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Agent Runner / tests
//!
//! ### AI Assist Note
//! - Verify effective execution settings through a local provider; do not call live cloud models in regressions.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: none declared

#[cfg(test)]
#[allow(clippy::module_inception)]
mod tests {
    use super::super::hierarchy::resolve_hierarchy_label;
    use super::super::loop_detector::{normalize_json, DoomLoopDetector, LoopCheckResult};
    use super::super::sentinel::{find_ungrounded_citations, scrub_mythos_tags};
    use crate::agent::constants::*;
    use crate::agent::runner::service_traits::SlotKind;
    use crate::agent::runner::{AgentRunner, RunContext};
    use crate::agent::types::{ModelConfig, ModelProvider};
    use crate::state::AppState;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_fast_path_uses_one_effective_reasoning_pass() {
        use crate::agent::types::{EngineAgent, TaskPayload};
        use axum::{routing::post, Json, Router};
        use std::sync::atomic::{AtomicUsize, Ordering};

        let calls = Arc::new(AtomicUsize::new(0));
        let counter = calls.clone();
        let app = Router::new().route("/v1/chat/completions", post(move || {
            let counter = counter.clone();
            async move {
                let pass = counter.fetch_add(1, Ordering::SeqCst) + 1;
                Json(serde_json::json!({
                    "choices": [{"message": {"role": "assistant", "content": format!("Answer from pass {pass}.")}, "finish_reason": "stop"}],
                    "usage": {"prompt_tokens": 1, "completion_tokens": 1, "total_tokens": 2}
                }))
            }
        }));
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let server = tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });
        let state = Arc::new(AppState::new_minimal_mock().await);
        let runner = AgentRunner::new(state.clone());
        let mut agent = EngineAgent::default();
        agent.identity.id = "fast-path-regression".into();
        agent.economics.budget_usd = 1.0;
        crate::agent::persistence::save_agent_db(&state.resources.pool, &mut agent)
            .await
            .unwrap();
        state
            .registry
            .agents
            .insert(agent.identity.id.clone(), agent);
        let mission = crate::agent::mission::create_mission(
            &state.resources.pool,
            "fast-path-regression",
            "Read status",
            1.0,
        )
        .await
        .unwrap();
        let mut ctx = RunContext::default();
        ctx.identity.agent_id = "fast-path-regression".into();
        ctx.mission_state.mission_id = mission.id;
        ctx.mission_state.budget_usd = 1.0;
        ctx.safe_mode = true; // Isolate reasoning passes from specialist enforcement calls.
        ctx.reasoning_depth = 2;
        ctx.provider_name = "openai".into();
        ctx.model_config = ModelConfig {
            provider: ModelProvider::Openai,
            model_id: "gpt-4o-mini".into(),
            api_key: Some("local-test-only".into()),
            base_url: Some(format!("http://{address}/v1")),
            input_cost_per_m: Some(0.0),
            output_cost_per_m: Some(0.0),
            ..Default::default()
        };
        let payload = TaskPayload {
            message: "read file status".into(),
            ..Default::default()
        };
        let output = runner.handle_reasoning_phase(&ctx, &payload).await.unwrap();
        assert_eq!(calls.load(Ordering::SeqCst), 1, "A fast-path request must use its effective one-pass depth, not the saved multi-pass depth");
        assert_eq!(output.text, "Answer from pass 1.");

        calls.store(0, Ordering::SeqCst);
        let payload = TaskPayload {
            message: "Review the execution flow and report your assessment.".into(),
            ..Default::default()
        };
        assert!(!AgentRunner::is_fast_path_query(&payload.message));
        let output = runner.handle_reasoning_phase(&ctx, &payload).await.unwrap();
        server.abort();
        assert_eq!(
            calls.load(Ordering::SeqCst),
            2,
            "An ordinary request must retain its configured multi-pass depth"
        );
        assert_eq!(output.text, "Answer from pass 2.");
    }

    #[test]
    fn test_doom_loop_detector() {
        let mut detector = DoomLoopDetector::new();
        // Period 1 loop (A -> A -> A):
        // 1st call -> Ok, 2nd call -> Warning, 3rd call -> Terminal
        assert_eq!(
            detector.check("1", "ls", "{}", "file1.txt"),
            LoopCheckResult::Ok
        );
        assert_eq!(
            detector.check("1", "ls", "{}", "file1.txt"),
            LoopCheckResult::Warning
        );
        assert_eq!(
            detector.check("1", "ls", "{}", "file1.txt"),
            LoopCheckResult::Terminal
        );

        // Reset and test Period 2 loop (A -> B -> A -> B -> A -> B)
        // 1st cycle (A, B) -> Ok, Ok
        // 2nd cycle (A, B) -> Ok, Warning
        // 3rd cycle (A, B) -> Ok, Terminal
        let mut detector = DoomLoopDetector::new();
        assert_eq!(
            detector.check("1", "ls", "{}", "file1.txt"),
            LoopCheckResult::Ok
        );
        assert_eq!(
            detector.check("1", "cat", "{}", "content"),
            LoopCheckResult::Ok
        );
        assert_eq!(
            detector.check("1", "ls", "{}", "file1.txt"),
            LoopCheckResult::Ok
        );
        assert_eq!(
            detector.check("1", "cat", "{}", "content"),
            LoopCheckResult::Warning
        );
        assert_eq!(
            detector.check("1", "ls", "{}", "file1.txt"),
            LoopCheckResult::Warning
        );
        assert_eq!(
            detector.check("1", "cat", "{}", "content"),
            LoopCheckResult::Terminal
        );
    }

    #[test]
    fn test_scrub_mythos_tags() {
        let input =
            "Thinking... <thinking>I should search</thinking> <halting_signal/> Done. <halt/>";
        let expected = "Thinking... I should search  Done.";
        assert_eq!(scrub_mythos_tags(input), expected);

        let variations = "Start <thinking >nested</thinking > Middle <THINKING>caps</THINKING> End <HALTING_SIGNAL /> <halt />";
        let expected_var = "Start nested Middle caps End";
        assert_eq!(scrub_mythos_tags(variations), expected_var);

        let clean = "No tags here";
        assert_eq!(scrub_mythos_tags(clean), "No tags here");
    }

    #[test]
    fn test_is_fast_path_query_heuristics() {
        // Complex actions should never be fast path even if short
        assert!(!AgentRunner::is_fast_path_query("fix the login bug"));
        assert!(!AgentRunner::is_fast_path_query("add a retry mechanism"));
        assert!(!AgentRunner::is_fast_path_query(
            "create new database table"
        ));
        assert!(!AgentRunner::is_fast_path_query("patch user routes"));
        assert!(!AgentRunner::is_fast_path_query(
            "write tests for persistence"
        ));
        assert!(!AgentRunner::is_fast_path_query(
            "implement websocket client"
        ));

        // Action requests even with polite framing should never be fast path
        assert!(!AgentRunner::is_fast_path_query(
            "can you fix the login bug?"
        ));
        assert!(!AgentRunner::is_fast_path_query(
            "please add a retry mechanism"
        ));

        // Interrogative / read questions should be fast path
        assert!(AgentRunner::is_fast_path_query(
            "what is the current status?"
        ));
        assert!(AgentRunner::is_fast_path_query("who are you"));
        assert!(AgentRunner::is_fast_path_query(
            "how does token counting work?"
        ));
        assert!(AgentRunner::is_fast_path_query("why did the step fail?"));
        assert!(AgentRunner::is_fast_path_query("explain the architecture"));
        assert!(AgentRunner::is_fast_path_query("get metrics"));

        // Substring collision prevention (e.g. 'address' containing 'add')
        assert!(AgentRunner::is_fast_path_query("what is the user address?"));

        // Inquiries referencing complex actions for explanation / learning
        assert!(AgentRunner::is_fast_path_query("how do I add a test?"));
        assert!(AgentRunner::is_fast_path_query(
            "what is the fix for bug 42?"
        ));
        assert!(AgentRunner::is_fast_path_query(
            "explain how to fix this error"
        ));
    }

    #[test]
    fn test_normalize_json_special_chars() {
        // 1. Structural disambiguation: pre-fix these normalized identically; escaping must keep them distinct
        let a = serde_json::json!({ "a": { "b": 2 } });
        let b = serde_json::json!({ r#"a":{"b"#.to_string(): 2 });
        assert_ne!(normalize_json(&a), normalize_json(&b));

        // 2. Character escaping: quotes and backslashes in keys are safely escaped
        let val = serde_json::json!({
            "key\"with\"quotes": "value",
            "key\\with\\backslashes": 123,
            "normal_key": true
        });
        let normalized = normalize_json(&val);
        assert!(normalized.contains(r#""key\"with\"quotes""#));
        assert!(normalized.contains(r#""key\\with\\backslashes""#));
    }

    #[tokio::test]
    async fn test_reasoning_turn_guard_resets_on_drop() {
        use super::super::hierarchy::ReasoningTurnGuard;

        let state = Arc::new(AppState::new_minimal_mock().await);
        let agent_id = "test-agent-guard";

        // Register dummy agent in registry
        let mut agent = crate::agent::types::EngineAgent::default();
        agent.identity.id = agent_id.to_string();
        state.registry.agents.insert(agent_id.to_string(), agent);

        {
            let _guard = ReasoningTurnGuard::new(agent_id.to_string(), state.clone());
            if let Some(mut entry) = state.registry.agents.get_mut(agent_id) {
                entry.value_mut().state.current_reasoning_turn = 5;
            }
            assert_eq!(
                state
                    .registry
                    .agents
                    .get(agent_id)
                    .unwrap()
                    .state
                    .current_reasoning_turn,
                5
            );
        }

        // Guard dropped
        assert_eq!(
            state
                .registry
                .agents
                .get(agent_id)
                .unwrap()
                .state
                .current_reasoning_turn,
            0
        );
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

        let state =
            AgentMissionState::resolve("--- [ROOM: system::spec] --- some spec", false, false);
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

        let selection = runner.model_router.select_model_slot(
            &runner.state,
            &agent.models,
            SlotKind::Planning,
            None,
        );

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

        let selection = runner.model_router.select_model_slot(
            &runner.state,
            &agent.models,
            SlotKind::Planning,
            None,
        );

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
        ctx.identity.agent_id = AGENT_CEO.to_string();

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
        state
            .governance
            .null_providers_test_mode
            .store(true, std::sync::atomic::Ordering::Relaxed);
        let runner = AgentRunner::new(state.clone());

        let mut ctx = RunContext::default();
        ctx.identity.agent_id = "specialist-1".to_string();
        ctx.identity.role = "Specialist".to_string();
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

        let selection = runner.model_router.select_model_slot(
            &runner.state,
            &agent.models,
            SlotKind::Planning,
            None,
        );

        assert_eq!(selection.kind, SlotKind::Default);
        assert!(selection.privacy_local_override);
        assert_eq!(selection.config.provider, ModelProvider::Ollama);
        assert_eq!(selection.config.model_id, "phi3.5-safe:latest");
    }

    #[test]
    fn test_find_ungrounded_citations() {
        let temp_dir = std::env::temp_dir();
        let hallucinated_text = "As the COO, we concluded the Discovery Phase. Findings are documented in `Glossary_Drift_Report.md`. We will update to /api/v3/endpoint.";
        let violations = find_ungrounded_citations(hallucinated_text, &temp_dir);
        assert_eq!(violations.len(), 2);
        assert!(violations.iter().any(|v| v.contains("/api/v3")));
        assert!(violations
            .iter()
            .any(|v| v.contains("Glossary_Drift_Report.md")));

        // Valid grounded text
        let grounded_text = "The mission is running normally with standard Axum routes.";
        let clean = find_ungrounded_citations(grounded_text, &temp_dir);
        assert!(clean.is_empty());
    }
}

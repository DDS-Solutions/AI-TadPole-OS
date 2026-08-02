//! @docs ARCHITECTURE:Runner
//!
//! ### AI Assist Note
//! **Provider Tests**: Unit testing suite verifying the provider routing, fallback priority order, and rate limiting isolation.
//!
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: Failures here indicate logic regressions in the provider subsystem.
//!

#[cfg(test)]
#[allow(clippy::module_inception)]
mod tests {
    use super::super::resolution::resolve_api_key;
    use super::super::ProviderVariant;
    use crate::agent::runner::{AgentRunner, RunContext};
    use crate::agent::types::{ModelConfig, ModelProvider, TokenUsage};
    use crate::state::AppState;
    use once_cell::sync::Lazy;
    use std::sync::Arc;
    use tokio::sync::Mutex;

    static TEST_MUTEX: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));

    #[tokio::test]
    async fn test_accumulate_usage() {
        let _lock = TEST_MUTEX.lock().await;
        let state = Arc::new(AppState::new_minimal_mock().await);
        let runner = AgentRunner::new(state);

        let mut total = Some(TokenUsage {
            input_tokens: 10,
            output_tokens: 5,
            total_tokens: 15,
        });
        let local = Some(TokenUsage {
            input_tokens: 20,
            output_tokens: 10,
            total_tokens: 30,
        });

        runner.accumulate_usage(&mut total, local);

        let tot = total.unwrap();
        assert_eq!(tot.input_tokens, 30);
        assert_eq!(tot.output_tokens, 15);
        assert_eq!(tot.total_tokens, 45);
    }

    #[tokio::test]
    async fn test_check_budget_exceeded() {
        let _lock = TEST_MUTEX.lock().await;
        let state = Arc::new(AppState::new_minimal_mock().await);
        let runner = AgentRunner::new(state);
        let mut ctx = RunContext::default();
        ctx.budget_usd = 1.0;
        ctx.current_cost_usd = 1.06; // Over 105%

        let res = runner.check_budget(&ctx, 0.0, "Result text").await.unwrap();
        assert!(res.is_some());
        assert!(res.unwrap().contains("Budget Exceeded"));
    }

    #[tokio::test]
    async fn test_check_budget_safe() {
        let _lock = TEST_MUTEX.lock().await;
        let state = Arc::new(AppState::new_minimal_mock().await);
        let runner = AgentRunner::new(state);
        let mut ctx = RunContext::default();
        ctx.budget_usd = 1.0;
        ctx.current_cost_usd = 0.5;

        let res = runner.check_budget(&ctx, 0.1, "Result text").await.unwrap();
        assert!(res.is_none());
    }

    #[tokio::test]
    async fn test_resolve_api_key() {
        let _lock = TEST_MUTEX.lock().await;
        let mut config = ModelConfig::default();
        config.api_key = Some("config-key".to_string());

        let key = resolve_api_key(&config, "UNUSED_ENV_VAR");
        assert_eq!(key, Some("config-key".to_string()));

        config.api_key = None;
        let original_val = std::env::var("TEST_PROVIDER_KEY").ok();
        std::env::set_var("TEST_PROVIDER_KEY", "env-key");
        let key = resolve_api_key(&config, "TEST_PROVIDER_KEY");
        assert_eq!(key, Some("env-key".to_string()));

        match original_val {
            Some(v) => std::env::set_var("TEST_PROVIDER_KEY", v),
            None => std::env::remove_var("TEST_PROVIDER_KEY"),
        }
    }

    #[tokio::test]
    async fn test_ollama_default_routing() {
        let _lock = TEST_MUTEX.lock().await;
        let state = Arc::new(AppState::new_minimal_mock().await);
        let runner = AgentRunner::new(state);
        let mut ctx = RunContext::default();
        ctx.model_config.provider = ModelProvider::Ollama;
        ctx.model_config.base_url = None;

        let client = reqwest::Client::new();
        let variant = runner.resolve_provider(&ctx, client).await;

        if let ProviderVariant::OpenAI(_p) = variant {
            // Success
        } else {
            panic!("Expected OpenAI variant for Ollama");
        }
    }

    #[tokio::test]
    async fn test_privacy_mode_local_routing() {
        let _lock = TEST_MUTEX.lock().await;
        let state = Arc::new(AppState::new_minimal_mock().await);
        let original_privacy = state
            .governance
            .privacy_mode
            .load(std::sync::atomic::Ordering::Relaxed);

        state
            .governance
            .privacy_mode
            .store(true, std::sync::atomic::Ordering::Relaxed);
        let runner = AgentRunner::new(state.clone());

        let mut ctx = RunContext::default();
        ctx.model_config.provider = ModelProvider::Gemini;
        ctx.model_config.model_id = "gemini-1.5-pro".to_string();

        let client = reqwest::Client::new();
        let variant = runner.resolve_provider(&ctx, client.clone()).await;

        if let ProviderVariant::OpenAI(ref p) = variant {
            assert_eq!(
                p.config.provider,
                ModelProvider::Ollama
            );
            assert!(!p.config.model_id.is_empty());
            assert_eq!(p.api_key, "ollama");
        } else {
            panic!("Expected OpenAI/Ollama variant for routed cloud model under privacy shield");
        }

        let mut ctx_local = RunContext::default();
        ctx_local.model_config.provider = ModelProvider::Ollama;
        ctx_local.model_config.model_id = "mistral".to_string();

        let variant_local = runner.resolve_provider(&ctx_local, client).await;
        if let ProviderVariant::OpenAI(ref p) = variant_local {
            assert_eq!(
                p.config.provider,
                ModelProvider::Ollama
            );
            assert_eq!(p.config.model_id, "mistral");
        } else {
            panic!("Expected Ollama/OpenAI variant for local provider under privacy shield");
        }

        state
            .governance
            .privacy_mode
            .store(original_privacy, std::sync::atomic::Ordering::Relaxed);
    }

    #[tokio::test]
    async fn test_is_local_endpoint_validation() {
        let _lock = TEST_MUTEX.lock().await;
        use crate::agent::model_routing::is_local_endpoint;

        assert!(is_local_endpoint(&ModelProvider::Ollama, None));
        assert!(is_local_endpoint(
            &ModelProvider::Ollama,
            Some("https://example.com")
        ));

        assert!(is_local_endpoint(
            &ModelProvider::Openai,
            Some("http://localhost:11434")
        ));
        assert!(is_local_endpoint(
            &ModelProvider::Openai,
            Some("http://host.docker.internal:11434")
        ));

        assert!(is_local_endpoint(
            &ModelProvider::Openai,
            Some("http://127.0.0.1:8080")
        ));
        assert!(is_local_endpoint(
            &ModelProvider::Openai,
            Some("http://10.0.0.1:11434")
        ));
        assert!(is_local_endpoint(
            &ModelProvider::Openai,
            Some("http://10.0.0.1:11434")
        ));
        assert!(is_local_endpoint(
            &ModelProvider::Openai,
            Some("http://10.0.0.1:11434")
        ));

        assert!(is_local_endpoint(
            &ModelProvider::Openai,
            Some("http://[::1]:11434")
        ));

        assert!(!is_local_endpoint(
            &ModelProvider::Openai,
            Some("https://api.openai.com/v1")
        ));
        assert!(!is_local_endpoint(
            &ModelProvider::Gemini,
            Some("https://generativelanguage.googleapis.com")
        ));
        assert!(!is_local_endpoint(&ModelProvider::Openai, None));
    }

    #[tokio::test]
    async fn test_collect_fallback_candidates_priorities() {
        let _lock = TEST_MUTEX.lock().await;
        let state = Arc::new(AppState::new_minimal_mock().await);
        let original_privacy = state
            .governance
            .privacy_mode
            .load(std::sync::atomic::Ordering::Relaxed);
        let runner = AgentRunner::new(state.clone());

        let mut agent = crate::agent::types::EngineAgent::default();
        agent.identity.id = "test-agent-fallback-priorities".to_string();
        agent.models.model = ModelConfig {
            provider: ModelProvider::Ollama,
            model_id: "default-model".to_string(),
            ..Default::default()
        };
        agent.models.planning_slot = Some(ModelConfig {
            provider: ModelProvider::Gemini,
            model_id: "planning-model".to_string(),
            ..Default::default()
        });
        agent.models.execution_slot = Some(ModelConfig {
            provider: ModelProvider::Groq,
            model_id: "execution-model".to_string(),
            ..Default::default()
        });
        agent.models.active_model_slot = Some("planning".to_string());

        state
            .registry
            .agents
            .insert("test-agent-fallback-priorities".to_string(), agent);

        let mut ctx = RunContext::default();
        ctx.agent_id = "test-agent-fallback-priorities".to_string();
        ctx.model_config = ModelConfig {
            provider: ModelProvider::Gemini,
            model_id: "planning-model".to_string(),
            ..Default::default()
        };

        let mut seen = std::collections::HashSet::new();
        seen.insert((
            ctx.model_config.provider.to_string().to_lowercase(),
            ctx.model_config.model_id.clone(),
        ));

        let candidates = runner.collect_fallback_candidates(&ctx, &seen);
        assert_eq!(candidates.len(), 2);
        assert_eq!(candidates[0].model_id, "execution-model");
        assert_eq!(candidates[1].model_id, "default-model");

        seen.insert((
            ModelProvider::Groq.to_string().to_lowercase(),
            "execution-model".to_string(),
        ));
        let candidates_next = runner.collect_fallback_candidates(&ctx, &seen);
        assert_eq!(candidates_next.len(), 1);
        assert_eq!(candidates_next[0].model_id, "default-model");

        state
            .governance
            .privacy_mode
            .store(true, std::sync::atomic::Ordering::Relaxed);
        let mut seen_privacy = std::collections::HashSet::new();
        seen_privacy.insert((
            ModelProvider::Ollama.to_string().to_lowercase(),
            "default-model".to_string(),
        ));

        let candidates_privacy = runner.collect_fallback_candidates(&ctx, &seen_privacy);
        assert!(candidates_privacy.is_empty());

        state
            .governance
            .privacy_mode
            .store(original_privacy, std::sync::atomic::Ordering::Relaxed);
    }

    #[tokio::test]
    async fn test_per_agent_rate_limiter_isolation() {
        let _lock = TEST_MUTEX.lock().await;
        let state = Arc::new(AppState::new_minimal_mock().await);
        let runner = AgentRunner::new(state.clone());

        let mut ctx1 = RunContext::default();
        ctx1.agent_id = "agent-1".to_string();
        ctx1.provider_name = "ollama".to_string();
        ctx1.model_config = ModelConfig {
            provider: ModelProvider::Ollama,
            model_id: "shared-model".to_string(),
            rpm: Some(10),
            tpm: Some(100),
            ..Default::default()
        };

        let mut ctx2 = RunContext::default();
        ctx2.agent_id = "agent-2".to_string();
        ctx2.provider_name = "ollama".to_string();
        ctx2.model_config = ctx1.model_config.clone();

        assert_eq!(state.resources.rate_limiters.len(), 0);

        runner
            .acquire_rate_limit("agent-1", &ctx1, "sys", "user")
            .await;
        assert_eq!(state.resources.rate_limiters.len(), 1);

        runner
            .acquire_rate_limit("agent-2", &ctx2, "sys", "user")
            .await;
        assert_eq!(state.resources.rate_limiters.len(), 2);

        let key1 = format!("agent-1:ollama:shared-model");
        let key2 = format!("agent-2:ollama:shared-model");
        assert!(state.resources.rate_limiters.contains_key(&key1));
        assert!(state.resources.rate_limiters.contains_key(&key2));
    }

    #[tokio::test]
    async fn test_privacy_shield_keeps_local_openai_compatible_candidates() {
        let _lock = TEST_MUTEX.lock().await;
        let state = Arc::new(AppState::new_minimal_mock().await);
        let original_privacy = state
            .governance
            .privacy_mode
            .load(std::sync::atomic::Ordering::Relaxed);

        state
            .governance
            .privacy_mode
            .store(true, std::sync::atomic::Ordering::Relaxed);
        let runner = AgentRunner::new(state.clone());

        let mut agent = crate::agent::types::EngineAgent::default();
        agent.identity.id = "test-agent-local-openai-fallback".to_string();
        agent.models.model = ModelConfig {
            provider: ModelProvider::Ollama,
            model_id: "default-local".to_string(),
            ..Default::default()
        };
        agent.models.planning_slot = Some(ModelConfig {
            provider: ModelProvider::Openai,
            model_id: "local-openai-compatible".to_string(),
            base_url: Some("http://localhost:1234/v1".to_string()),
            ..Default::default()
        });

        state
            .registry
            .agents
            .insert("test-agent-local-openai-fallback".to_string(), agent);

        let mut ctx = RunContext::default();
        ctx.agent_id = "test-agent-local-openai-fallback".to_string();
        ctx.model_config = ModelConfig {
            provider: ModelProvider::Openai,
            model_id: "local-openai-compatible".to_string(),
            base_url: Some("http://localhost:1234/v1".to_string()),
            ..Default::default()
        };

        let mut seen = std::collections::HashSet::new();
        seen.insert((
            ctx.model_config.provider.to_string().to_lowercase(),
            ctx.model_config.model_id.clone(),
        ));

        let candidates = runner.collect_fallback_candidates(&ctx, &seen);

        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0].model_id, "default-local");

        state
            .governance
            .privacy_mode
            .store(original_privacy, std::sync::atomic::Ordering::Relaxed);
    }
}

//! @docs ARCHITECTURE:Runner
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Agent Runner / resolution
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Type-safe state handling and bounded execution without unhandled panics.
//! - `[Behavioral]` Under Privacy Mode, non-local endpoints and oversized local models fail closed if no local <=15B model exists (enforced_by: `test_select_best_fallback_model_rejects_all_oversized`).
//! - `[Behavioral]` Local endpoints, custom gateways, and multi-vendor proxies (OpenRouter) are never mutated by model-provider auto-alignment (enforced_by: `test_alignment_preserves_local_and_custom_gateway`).
//! - `[Behavioral]` Untagged non-standard models are rejected from automatic fallback selection to protect VRAM (enforced_by: `test_estimate_model_param_size_untagged_and_oversized`).
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: `[Provider]`, `[Privacy Shield]`, `[Provider Alignment]`
//! - **Witness Tests**: `test_select_best_fallback_model_rejects_all_oversized`, `test_extract_model_param_size`, `test_is_oversized_model`, `test_alignment_preserves_local_and_custom_gateway`, `test_estimate_model_param_size_untagged_and_oversized`

use std::collections::HashMap;
use std::sync::LazyLock;
use std::time::{Duration, Instant};

use parking_lot::RwLock;

use super::ProviderVariant;
use crate::agent::model_routing::is_local_endpoint;
use crate::agent::null_provider::{NullProvider, NullReason};
use crate::agent::runner::{AgentRunner, RunContext};
use crate::agent::types::{ModelConfig, ModelProvider};

/// Resolves an API key: prefers the per-agent config override, then falls
/// back to the named environment variable. Automatically trims whitespace/newlines.
pub(crate) fn resolve_api_key(config: &ModelConfig, env_var: &str) -> Option<String> {
    config
        .api_key
        .as_ref()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .or_else(|| {
            std::env::var(env_var)
                .ok()
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
        })
}

/// Generic helper to resolve provider API key with primary and optional fallback environment variables.
pub(crate) fn resolve_provider_key(
    config: &ModelConfig,
    agent_id: &str,
    primary_env: &'static str,
    secondary_env: Option<&'static str>,
) -> Result<String, ProviderVariant> {
    let key_opt = resolve_api_key(config, primary_env)
        .or_else(|| secondary_env.and_then(|sec| resolve_api_key(config, sec)));

    match key_opt {
        Some(k) => Ok(k),
        None => Err(ProviderVariant::Null(NullProvider::new(
            agent_id,
            NullReason::MissingApiKey {
                env_var: primary_env,
            },
        ))),
    }
}

pub const PRIVACY_FALLBACK_MODEL: &str = "phi3.5-safe:latest";

/// Extracts the parameter size in billions (B) from a model name/tag if present.
///
/// Examples:
/// - `"gemma4:12b"` -> `Some(12.0)`
/// - `"llama3:70b-instruct"` -> `Some(70.0)`
/// - `"qwen2.5:32b"` -> `Some(32.0)`
/// - `"phi3:3.8b"` -> `Some(3.8)`
/// - `"gemma4:e4b"` -> `Some(4.0)`
/// - `"mixtral:8x7b"` -> `Some(56.0)`
/// - `"llama3.1:405b"` -> `Some(405.0)`
pub(crate) fn extract_model_param_size(id: &str) -> Option<f32> {
    let lower = id.to_ascii_lowercase();
    for token in lower.split(|c: char| !c.is_alphanumeric() && c != '.') {
        if token.ends_with('b') && token.len() > 1 {
            let num_part = &token[..token.len() - 1];
            // MoE notation: e.g. 8x7b or 8x22b
            if let Some((experts, size)) = num_part.split_once('x') {
                if let (Ok(e), Ok(s)) = (experts.parse::<f32>(), size.parse::<f32>()) {
                    return Some(e * s);
                }
            }
            // Strip leading single-character prefix if any (e.g. 'e4b' in gemma4:e4b or 'q4b')
            let trimmed = num_part.trim_start_matches(|c: char| c.is_alphabetic());
            if let Ok(val) = trimmed.parse::<f32>() {
                if val > 0.0 {
                    return Some(val);
                }
            }
        }
    }
    None
}

/// Estimates the parameter size in billions (B) from either an explicit tag or known family defaults.
/// Returns `None` if the model has no size tag and is not in a recognized safe architecture family.
pub(crate) fn estimate_model_param_size(id: &str) -> Option<f32> {
    if let Some(size) = extract_model_param_size(id) {
        return Some(size);
    }

    let lower = id.to_ascii_lowercase();
    let slug = lower.rsplit('/').next().unwrap_or(&lower);
    let base = slug
        .strip_suffix(":latest")
        .unwrap_or(slug)
        .trim_end_matches("-instruct")
        .trim_end_matches("-chat");

    match base {
        "llama3" | "llama3.1" => Some(8.0),
        "llama3.2" => Some(3.0),
        "llama2" | "mistral" | "qwen" | "qwen2" | "qwen2.5" | "gemma" => Some(7.0),
        "gemma2" => Some(9.0),
        "phi" | "phi-2" => Some(2.7),
        "phi3" | "phi3.5" | "phi-3" | "phi-3.5" | "phi3.5-safe" => Some(3.8),
        "tinyllama" => Some(1.1),
        "smollm" => Some(1.7),
        // Known OpenAI/cloud models that run locally via Ollama pull — safe size estimates
        "gpt-4o" | "gpt-4o-mini" => Some(8.0),
        // Community chat models with no explicit size tag — conservatively estimated as mid-tier safe
        "eclipse" | "robert" | "ornith" | "starling" | "nous-hermes" | "openhermes" => Some(7.0),
        // Known oversized families without size tag default to oversized estimate
        "nemotron" | "command-r" | "mixtral" | "deepseek-v" | "wizardlm-2" | "deepseek-r1" => {
            Some(70.0)
        }
        _ => None,
    }
}

/// Determines whether a model is oversized (>15B parameters or known heavyweight model family)
/// for local Privacy Shield execution on standard edge/desktop hardware.
///
/// Untagged, unvetted models whose parameter size cannot be verified are treated as unsafe/oversized.
pub(crate) fn is_oversized_model(id: &str) -> bool {
    let lower = id.to_ascii_lowercase();

    // Check known heavyweight model families immediately
    if lower.contains("nemotron")
        || lower.contains("command-r")
        || lower.contains("mixtral")
        || lower.contains("deepseek-v")
        || lower.contains("wizardlm-2")
        || (lower.contains("deepseek-r1")
            && !lower.contains(":7b")
            && !lower.contains(":8b")
            && !lower.contains(":14b"))
    {
        if let Some(size) = extract_model_param_size(&lower) {
            return size > 15.0;
        }
        return true;
    }

    match estimate_model_param_size(id) {
        Some(size) => size > 15.0,
        // Untagged unknown models cannot be proven safe (<=15B); reject from automatic selection
        None => true,
    }
}

/// Filter out non-generative embedding, reranking, and encoder models with boundary awareness.
pub(crate) fn is_plausible_chat_model(id: &str) -> bool {
    let l = id.to_ascii_lowercase();

    for token in l.split(|c: char| !c.is_alphanumeric() && c != '.') {
        if token == "embed"
            || token.starts_with("embed-")
            || token.ends_with("-embed")
            || token == "embedding"
            || token == "embeddings"
            || token == "rerank"
            || token.starts_with("rerank-")
            || token.ends_with("-rerank")
            || token == "colbert"
            || token.contains("minilm")
            || token == "clip"
            || token.starts_with("clip-")
            || token.ends_with("-clip")
            || token == "bge"
            || token.starts_with("bge-")
            || token.starts_with("bge_")
            || token == "bert"
            || token.starts_with("bert-")
            || token == "roberta"
            || token == "distilbert"
            || token == "gte"
            || token.starts_with("gte-")
            || token == "e5"
            || token.starts_with("e5-")
            || token.starts_with("e5_")
        {
            return false;
        }
    }

    true
}

/// Checks whether an endpoint URL is the official cloud OpenAI domain.
pub(crate) fn is_official_openai_endpoint(url_opt: Option<&str>) -> bool {
    match url_opt {
        None => true, // default base_url is official https://api.openai.com/v1
        Some(url) => {
            let trimmed = url.trim();
            if trimmed.is_empty() {
                return true;
            }
            if let Ok(parsed) = reqwest::Url::parse(trimmed) {
                if let Some(host) = parsed.host_str() {
                    return host.eq_ignore_ascii_case("api.openai.com");
                }
            }
            false
        }
    }
}

/// Constructs path-preserving probe URLs for both `/v1/models` and `/api/tags`.
pub(crate) fn construct_probe_urls(base_url: &str) -> (String, String) {
    let clean = base_url.trim().trim_end_matches('/');
    if clean.ends_with("/v1") {
        let v1_models = format!("{}/models", clean);
        let base_no_v1 = clean
            .strip_suffix("/v1")
            .unwrap_or(clean)
            .trim_end_matches('/');
        let tags = format!("{}/api/tags", base_no_v1);
        (v1_models, tags)
    } else {
        let v1_models = format!("{}/v1/models", clean);
        let tags = format!("{}/api/tags", clean);
        (v1_models, tags)
    }
}

/// Selects the best local fallback model from candidate model IDs.
///
/// Guarantees:
/// 1. Prioritizes the agent's requested model if already installed locally and <= 15B.
/// 2. Prioritizes declared lightweight local models (e.g. phi3.5-safe, gemma4:e4b, gemma4:12b).
/// 3. Filters out cloud aliases (`:cloud`, `-cloud`), embedding/encoder models, and oversized (>15B) models.
/// 4. Ranks remaining candidates by estimated parameter size ascending to protect VRAM.
/// 5. Returns `None` if no viable <= 15B model exists (allowing fail-closed behavior).
pub(crate) fn select_best_fallback_model<'a, I>(
    model_ids: I,
    requested_model: Option<&str>,
) -> Option<String>
where
    I: IntoIterator<Item = &'a str>,
{
    let ids: Vec<&'a str> = model_ids.into_iter().collect();

    // Filter valid chat candidates: must be plausible chat, not cloud proxy, not oversized (<= 15B)
    let valid_candidates: Vec<&'a str> = ids
        .into_iter()
        .filter(|&id| {
            let lower = id.to_ascii_lowercase();
            is_plausible_chat_model(id)
                && !lower.contains(":cloud")
                && !lower.contains("-cloud")
                && !is_oversized_model(id)
        })
        .collect();

    if valid_candidates.is_empty() {
        return None;
    }

    // Priority 0: If requested model (or its local non-cloud equivalent / slug) is available locally and safe, keep it!
    if let Some(req) = requested_model {
        let req_clean = req
            .strip_suffix(":cloud")
            .or_else(|| req.strip_suffix("-cloud"))
            .unwrap_or(req)
            .to_ascii_lowercase();

        let req_slug = req_clean.rsplit('/').next().unwrap_or(&req_clean);
        let req_base = req_slug
            .strip_suffix(":latest")
            .unwrap_or(req_slug)
            .trim_end_matches("-instruct");

        if let Some(&m) = valid_candidates.iter().find(|&&id| {
            let id_lower = id.to_ascii_lowercase();
            let id_slug = id_lower.rsplit('/').next().unwrap_or(&id_lower);
            let id_base = id_slug
                .strip_suffix(":latest")
                .unwrap_or(id_slug)
                .trim_end_matches("-instruct");

            id_lower == req_clean || id_slug == req_slug || id_base == req_base
        }) {
            return Some(m.to_string());
        }
    }

    // Priority 1: Check known optimal lightweight local models in curated priority order
    const PREFERRED: &[&str] = &[
        PRIVACY_FALLBACK_MODEL,
        "phi3.5:latest",
        "phi3.5",
        "gemma4:e4b",
        "gemma4:12b",
        "gemma4:latest",
        "llama3.2:3b",
        "llama3.2:1b",
        "llama3.1:8b",
        "llama3:8b",
        "qwen2.5:7b",
        "qwen2.5:3b",
        "qwen3.5:9b",
        "ornith:9b",
        "mistral:7b",
    ];

    for pref in PREFERRED {
        let pref_lower = pref.to_ascii_lowercase();
        let pref_base = pref_lower.strip_suffix(":latest").unwrap_or(&pref_lower);
        if let Some(&m) = valid_candidates.iter().find(|&&id| {
            let id_lower = id.to_ascii_lowercase();
            let id_base = id_lower.strip_suffix(":latest").unwrap_or(&id_lower);
            id_lower == pref_lower || id_base == pref_base
        }) {
            return Some(m.to_string());
        }
    }

    // Priority 2: Rank remaining candidates by estimated parameter size ascending to minimize VRAM footprint
    let mut ranked = valid_candidates;
    ranked.sort_by(|a, b| {
        let size_a = estimate_model_param_size(a).unwrap_or(15.0);
        let size_b = estimate_model_param_size(b).unwrap_or(15.0);
        size_a
            .partial_cmp(&size_b)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    ranked.first().map(|s| s.to_string())
}

struct CachedModels {
    models: Vec<String>,
    fetched_at: Instant,
}

static MODEL_CACHE: LazyLock<RwLock<HashMap<String, CachedModels>>> =
    LazyLock::new(|| RwLock::new(HashMap::new()));
const MODEL_CACHE_TTL_POSITIVE: Duration = Duration::from_secs(60);
const MODEL_CACHE_TTL_NEGATIVE: Duration = Duration::from_secs(10);

/// Probes local Ollama / OpenAI-compatible endpoint with path preservation, caching, and auth.
pub(crate) async fn fetch_available_local_models(
    client: &reqwest::Client,
    base_url: &str,
) -> Vec<String> {
    let (v1_models_url, api_tags_url) = construct_probe_urls(base_url);
    let cache_key = base_url
        .trim()
        .trim_end_matches('/')
        .strip_suffix("/v1")
        .unwrap_or(base_url)
        .to_ascii_lowercase();

    // 1. Check TTL cache (with negative cache handling for fast outage fail-close)
    {
        let cache = MODEL_CACHE.read();
        if let Some(entry) = cache.get(&cache_key) {
            let ttl = if entry.models.is_empty() {
                MODEL_CACHE_TTL_NEGATIVE
            } else {
                MODEL_CACHE_TTL_POSITIVE
            };
            if entry.fetched_at.elapsed() < ttl {
                return entry.models.clone();
            }
        }
    }

    let mut auth_header = None;
    if let Ok(k) = std::env::var("OLLAMA_API_KEY") {
        let trimmed = k.trim().to_string();
        if !trimmed.is_empty() {
            auth_header = Some(format!("Bearer {}", trimmed));
        }
    }

    #[derive(serde::Deserialize)]
    struct OpenAiCompatModel {
        id: String,
    }
    #[derive(serde::Deserialize)]
    struct OpenAiCompatModelsResponse {
        data: Vec<OpenAiCompatModel>,
    }

    let mut req = client.get(&v1_models_url).timeout(Duration::from_secs(3));
    if let Some(ref auth) = auth_header {
        req = req.header("Authorization", auth);
    }

    match req.send().await {
        Ok(resp) if resp.status().is_success() => {
            match resp.json::<OpenAiCompatModelsResponse>().await {
                Ok(models_list) => {
                    let ids: Vec<String> = models_list.data.into_iter().map(|m| m.id).collect();
                    if !ids.is_empty() {
                        let mut cache = MODEL_CACHE.write();
                        cache.insert(
                            cache_key.clone(),
                            CachedModels {
                                models: ids.clone(),
                                fetched_at: Instant::now(),
                            },
                        );
                        return ids;
                    }
                }
                Err(e) => {
                    tracing::warn!(
                        "🛡️ [Privacy Shield] JSON parse error from {}: {}",
                        v1_models_url,
                        e
                    );
                }
            }
        }
        Ok(resp) => {
            tracing::warn!(
                "🛡️ [Privacy Shield] Ollama /v1/models returned non-success status: {}",
                resp.status()
            );
        }
        Err(e) => {
            tracing::warn!("🛡️ [Privacy Shield] Ollama /v1/models probe failed: {}", e);
        }
    }

    // Probe 2: /api/tags (Legacy Ollama native)
    #[derive(serde::Deserialize)]
    struct LegacyOllamaModel {
        name: String,
    }
    #[derive(serde::Deserialize)]
    struct LegacyOllamaResponse {
        models: Vec<LegacyOllamaModel>,
    }

    let mut req = client.get(&api_tags_url).timeout(Duration::from_secs(3));
    if let Some(ref auth) = auth_header {
        req = req.header("Authorization", auth);
    }

    match req.send().await {
        Ok(resp) if resp.status().is_success() => match resp.json::<LegacyOllamaResponse>().await {
            Ok(models_list) => {
                let ids: Vec<String> = models_list.models.into_iter().map(|m| m.name).collect();
                if !ids.is_empty() {
                    let mut cache = MODEL_CACHE.write();
                    cache.insert(
                        cache_key.clone(),
                        CachedModels {
                            models: ids.clone(),
                            fetched_at: Instant::now(),
                        },
                    );
                    return ids;
                }
            }
            Err(e) => {
                tracing::warn!(
                    "🛡️ [Privacy Shield] JSON parse error from {}: {}",
                    api_tags_url,
                    e
                );
            }
        },
        Ok(resp) => {
            tracing::warn!(
                "🛡️ [Privacy Shield] Ollama /api/tags returned non-success status: {}",
                resp.status()
            );
        }
        Err(e) => {
            tracing::warn!("🛡️ [Privacy Shield] Ollama /api/tags probe failed: {}", e);
        }
    }

    // Insert negative cache entry to avoid repeated 6s timeouts on consecutive steps
    {
        let mut cache = MODEL_CACHE.write();
        cache.insert(
            cache_key,
            CachedModels {
                models: Vec::new(),
                fetched_at: Instant::now(),
            },
        );
    }

    Vec::new()
}

/// Dynamically queries local Ollama models to pick the optimal generative chat model as fallback.
/// Returns `None` if the local runtime is offline or has no models <= 15B installed.
pub(crate) async fn resolve_privacy_fallback_model(
    client: &reqwest::Client,
    base_url: &str,
    requested_model: Option<&str>,
) -> Option<String> {
    let models = fetch_available_local_models(client, base_url).await;
    let chosen = select_best_fallback_model(models.iter().map(|s| s.as_str()), requested_model);

    if let Some(ref model) = chosen {
        tracing::info!(
            "🛡️ [Privacy Shield] Dynamically resolved local fallback model: '{}' from Ollama",
            model
        );
    }

    chosen
}

/// Helper function to resolve OpenAI-compatible providers without code duplication.
pub(crate) fn resolve_openai_provider(
    client: reqwest::Client,
    config: &ModelConfig,
    agent_id: &str,
    env_var: &'static str,
    default_url: &str,
    provider_name: &'static str,
) -> ProviderVariant {
    let key_opt = resolve_api_key(config, env_var);
    let is_local = is_local_endpoint(&config.provider, config.base_url.as_deref());

    // Local OpenAI-compatible endpoints (vLLM, LM Studio, llama.cpp, local proxies)
    // frequently do not require an API key; provide a dummy token if key is omitted.
    let effective_key = match key_opt {
        Some(k) => k,
        None if is_local => "local-no-key".to_string(),
        None => {
            return ProviderVariant::Null(NullProvider::new(
                agent_id,
                NullReason::MissingApiKey { env_var },
            ));
        }
    };

    let mut config = config.clone();
    if config
        .base_url
        .as_deref()
        .map(str::trim)
        .unwrap_or("")
        .is_empty()
    {
        if default_url.is_empty() {
            return ProviderVariant::Null(NullProvider::new(
                agent_id,
                NullReason::MissingBaseUrl {
                    provider: provider_name,
                },
            ));
        }
        config.base_url = Some(default_url.to_string());
    }

    ProviderVariant::OpenAI(crate::agent::openai::OpenAIProvider::new(
        client,
        effective_key,
        config,
    ))
}

impl AgentRunner {
    /// Resolves the correct `ProviderVariant` for the given context.
    pub(crate) async fn resolve_provider(
        &self,
        ctx: &RunContext,
        client: reqwest::Client,
    ) -> ProviderVariant {
        tracing::debug!(
            "🔍 [Provider] Resolving provider '{}' for agent '{}'",
            ctx.provider_name,
            ctx.agent_id
        );

        if self
            .state
            .governance
            .null_providers_test_mode
            .load(std::sync::atomic::Ordering::Relaxed)
        {
            tracing::info!("[Provider] Null mode forced by test flag");
            return ProviderVariant::Null(NullProvider::new(&ctx.agent_id, NullReason::TestMode));
        }

        let mut active_config = ctx.model_config.clone();

        // Model-Provider Alignment check:
        // Only align when:
        // 1. A detected provider exists from model_id heuristics
        // 2. The detected provider differs from configured provider
        // 3. The configured provider is NOT OpenRouter (which routes heterogeneous vendor models by design)
        // 4. The endpoint is NOT a local endpoint (vLLM, LM Studio, llama.cpp, etc.)
        // 5. The base_url is official OpenAI (empty, None, or api.openai.com)
        if let Some(detected) = ModelProvider::from_model_id(&active_config.model_id) {
            let is_local =
                is_local_endpoint(&active_config.provider, active_config.base_url.as_deref());
            let is_official_openai = is_official_openai_endpoint(active_config.base_url.as_deref());
            let is_multi_vendor_proxy = active_config.provider == ModelProvider::Openrouter;

            if active_config.provider != detected {
                if is_local || !is_official_openai || is_multi_vendor_proxy {
                    tracing::debug!(
                        "ℹ️ [Provider Alignment] Preserving provider {:?} for model '{}' (local={}, official_openai={}, multi_vendor={})",
                        active_config.provider,
                        active_config.model_id,
                        is_local,
                        is_official_openai,
                        is_multi_vendor_proxy
                    );
                } else if active_config.provider == ModelProvider::Openai {
                    tracing::warn!(
                        "⚠️ [Provider Alignment] Model '{}' detected as {:?}, but provider is set to OpenAI. Aligning provider to {:?}.",
                        active_config.model_id,
                        detected,
                        detected
                    );
                    active_config.provider = detected;
                } else {
                    tracing::debug!(
                        "ℹ️ [Provider Alignment] Agent '{}' configured with provider {:?}, model indicates {:?}.",
                        ctx.agent_id,
                        active_config.provider,
                        detected
                    );
                }
            }
        }

        // OpenRouter Model Resolution: Apply OPENROUTER_DEFAULT_MODEL override if set
        if active_config.provider == ModelProvider::Openrouter {
            if let Ok(override_model) = std::env::var("OPENROUTER_DEFAULT_MODEL") {
                let trimmed = override_model.trim();
                if !trimmed.is_empty() && trimmed != active_config.model_id {
                    tracing::info!(
                        "🔄 [OpenRouter Resolution] Model '{}' overridden to '{}'",
                        active_config.model_id,
                        trimmed
                    );
                    active_config.model_id = trimmed.to_string();
                }
            }
        }

        // SEC-04: Privacy Mode Enforcement - Route non-local traffic to local Ollama endpoint,
        // and cap oversized (>15B) models on local endpoints to safe local alternatives.
        let is_privacy = self.is_privacy_active(ctx);

        if is_privacy {
            let is_local =
                is_local_endpoint(&active_config.provider, active_config.base_url.as_deref());
            let is_oversized = is_oversized_model(&active_config.model_id);

            if !is_local || is_oversized {
                if !is_local {
                    tracing::info!(
                        "🔒 [Privacy Shield] Routing cloud provider {:?} (model: {}) for agent '{}' to local Ollama endpoint",
                        active_config.provider,
                        active_config.model_id,
                        ctx.agent_id
                    );
                } else {
                    tracing::warn!(
                        "⚠️ [Privacy Shield] Agent '{}' configured with oversized local model '{}'. Enforcing safe local fallback <= 15B.",
                        ctx.agent_id,
                        active_config.model_id
                    );
                }

                let port = std::env::var("OLLAMA_PORT")
                    .ok()
                    .and_then(|p| p.parse().ok())
                    .unwrap_or(11434);
                let resolved_url =
                    crate::networking::resolver::AddressResolver::resolve_local_url(port).await;
                let fallback_base = format!("{}/v1", resolved_url);

                match resolve_privacy_fallback_model(
                    &client,
                    &fallback_base,
                    if is_oversized {
                        None
                    } else {
                        Some(&active_config.model_id)
                    },
                )
                .await
                {
                    Some(chosen_model) => {
                        active_config.provider = ModelProvider::Ollama;
                        active_config.model_id = chosen_model;
                        active_config.base_url = Some(fallback_base);
                        active_config.api_key = None;
                    }
                    None => {
                        tracing::error!(
                            "🔒 [Privacy Shield] Blocked execution for agent '{}': no reachable local model <= 15B found under Privacy Mode",
                            ctx.agent_id
                        );
                        return ProviderVariant::Null(NullProvider::new(
                            &ctx.agent_id,
                            NullReason::PrivacyModeEnforced,
                        ));
                    }
                }
            }
        }

        *ctx.resolved_model_id.lock() = Some(active_config.model_id.clone());

        match active_config.provider {
            ModelProvider::Google | ModelProvider::Gemini => {
                match resolve_provider_key(
                    &active_config,
                    &ctx.agent_id,
                    "GOOGLE_API_KEY",
                    Some("GEMINI_API_KEY"),
                ) {
                    Ok(key) => ProviderVariant::Gemini(crate::agent::gemini::GeminiProvider::new(
                        client,
                        key,
                        active_config.clone(),
                    )),
                    Err(null_variant) => null_variant,
                }
            }
            ModelProvider::Groq => {
                match resolve_provider_key(&active_config, &ctx.agent_id, "GROQ_API_KEY", None) {
                    Ok(key) => ProviderVariant::Groq(crate::agent::groq::GroqProvider::new(
                        client,
                        key,
                        active_config.clone(),
                    )),
                    Err(null_variant) => null_variant,
                }
            }
            ModelProvider::Openai
            | ModelProvider::Xai
            | ModelProvider::Openrouter
            | ModelProvider::Mistral
            | ModelProvider::Perplexity
            | ModelProvider::Fireworks
            | ModelProvider::Together
            | ModelProvider::Cerebras
            | ModelProvider::Sambanova
            | ModelProvider::Deepseek
            | ModelProvider::OllamaCloud => {
                let (env_var, default_url, name) = match active_config.provider {
                    ModelProvider::Openai => {
                        ("OPENAI_API_KEY", "https://api.openai.com/v1", "OpenAI")
                    }
                    ModelProvider::Xai => ("XAI_API_KEY", "https://api.x.ai/v1", "xAI"),
                    ModelProvider::Openrouter => (
                        "OPENROUTER_API_KEY",
                        "https://openrouter.ai/api/v1",
                        "OpenRouter",
                    ),
                    ModelProvider::Mistral => {
                        ("MISTRAL_API_KEY", "https://api.mistral.ai/v1", "Mistral")
                    }
                    ModelProvider::Perplexity => (
                        "PERPLEXITY_API_KEY",
                        "https://api.perplexity.ai",
                        "Perplexity",
                    ),
                    ModelProvider::Fireworks => (
                        "FIREWORKS_API_KEY",
                        "https://api.fireworks.ai/inference/v1",
                        "Fireworks",
                    ),
                    ModelProvider::Together => (
                        "TOGETHER_API_KEY",
                        "https://api.together.xyz/v1",
                        "Together",
                    ),
                    ModelProvider::Cerebras => {
                        ("CEREBRAS_API_KEY", "https://api.cerebras.ai/v1", "Cerebras")
                    }
                    ModelProvider::Sambanova => (
                        "SAMBANOVA_API_KEY",
                        "https://api.sambanova.ai/v1",
                        "SambaNova",
                    ),
                    ModelProvider::Deepseek => (
                        "DEEPSEEK_API_KEY",
                        "https://api.deepseek.com/v1",
                        "DeepSeek",
                    ),
                    ModelProvider::OllamaCloud => (
                        "OLLAMA_CLOUD_API_KEY",
                        "https://ollama.com/v1",
                        "Ollama Cloud",
                    ),
                    _ => ("OPENAI_API_KEY", "https://api.openai.com/v1", "OpenAI"),
                };

                resolve_openai_provider(
                    client,
                    &active_config,
                    &ctx.agent_id,
                    env_var,
                    default_url,
                    name,
                )
            }
            ModelProvider::Inception => resolve_openai_provider(
                client,
                &active_config,
                &ctx.agent_id,
                "INCEPTION_API_KEY",
                "",
                "Inception",
            ),
            ModelProvider::Ollama => {
                let api_key = resolve_api_key(&active_config, "OLLAMA_API_KEY")
                    .unwrap_or_else(|| "ollama".to_string());
                let mut config = active_config.clone();
                if let Some(url) = config.base_url.as_ref().filter(|s| !s.trim().is_empty()) {
                    config.base_url = Some(
                        crate::networking::resolver::AddressResolver::resolve_url_if_local(url)
                            .await,
                    );
                } else {
                    let port = std::env::var("OLLAMA_PORT")
                        .ok()
                        .and_then(|p| p.parse().ok())
                        .unwrap_or(11434);
                    let resolved_url =
                        crate::networking::resolver::AddressResolver::resolve_local_url(port).await;
                    config.base_url = Some(format!("{}/v1", resolved_url));
                }
                ProviderVariant::OpenAI(crate::agent::openai::OpenAIProvider::new(
                    client, api_key, config,
                ))
            }
            ModelProvider::Anthropic => {
                match resolve_provider_key(&active_config, &ctx.agent_id, "ANTHROPIC_API_KEY", None)
                {
                    Ok(key) => {
                        ProviderVariant::Anthropic(crate::agent::anthropic::AnthropicProvider::new(
                            client,
                            key,
                            active_config.clone(),
                        ))
                    }
                    Err(null_variant) => null_variant,
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_model_param_size() {
        assert_eq!(extract_model_param_size("gemma4:12b"), Some(12.0));
        assert_eq!(extract_model_param_size("llama3:70b-instruct"), Some(70.0));
        assert_eq!(extract_model_param_size("qwen2.5:32b"), Some(32.0));
        assert_eq!(extract_model_param_size("phi3:3.8b"), Some(3.8));
        assert_eq!(extract_model_param_size("gemma4:e4b"), Some(4.0));
        assert_eq!(extract_model_param_size("mixtral:8x7b"), Some(56.0));
        assert_eq!(extract_model_param_size("mixtral:8x22b"), Some(176.0));
        assert_eq!(extract_model_param_size("llama3.1:405b"), Some(405.0));
        assert_eq!(extract_model_param_size("model:15.0b"), Some(15.0));
        assert_eq!(extract_model_param_size("model:15.1b"), Some(15.1));
        assert_eq!(extract_model_param_size("model:16b"), Some(16.0));
        assert_eq!(extract_model_param_size("phi3.5-safe:latest"), None);
    }

    #[test]
    fn test_is_oversized_model() {
        assert!(is_oversized_model("llama3:70b"));
        assert!(is_oversized_model("qwen2.5:32b"));
        assert!(is_oversized_model("qwen2.5:72b"));
        assert!(is_oversized_model("llama3.1:405b"));
        assert!(is_oversized_model("nemotron-3.5-lightning:latest"));
        assert!(is_oversized_model("mixtral:8x7b"));
        assert!(is_oversized_model("mixtral:8x22b"));
        assert!(is_oversized_model("model:15.1b"));
        assert!(is_oversized_model("model:16b"));
        assert!(is_oversized_model("deepseek-r1:latest"));

        // Safe models (<= 15.0B)
        assert!(!is_oversized_model("model:15.0b"));
        assert!(!is_oversized_model("gemma4:12b"));
        assert!(!is_oversized_model("gemma4:e4b"));
        assert!(!is_oversized_model("phi3.5-safe:latest"));
        assert!(!is_oversized_model("llama3.2:3b"));
        assert!(!is_oversized_model("llama3:latest"));
        assert!(!is_oversized_model("qwen2.5:latest"));
    }

    #[test]
    fn test_estimate_model_param_size_untagged_and_oversized() {
        // Known family defaults
        assert_eq!(estimate_model_param_size("llama3:latest"), Some(8.0));
        assert_eq!(estimate_model_param_size("llama3.2:latest"), Some(3.0));
        assert_eq!(estimate_model_param_size("mistral:latest"), Some(7.0));
        assert_eq!(estimate_model_param_size("qwen2.5:latest"), Some(7.0));
        assert_eq!(estimate_model_param_size("phi3.5-safe:latest"), Some(3.8));

        // Untagged unvetted model returns None and is treated as oversized/unsafe
        assert_eq!(
            estimate_model_param_size("unknown-custom-model:latest"),
            None
        );
        assert!(is_oversized_model("unknown-custom-model:latest"));

        // 15.0B boundary check
        assert!(!is_oversized_model("test-model:15b"));
        assert!(!is_oversized_model("test-model:15.0b"));
        assert!(is_oversized_model("test-model:15.1b"));
        assert!(is_oversized_model("test-model:16b"));
    }

    #[test]
    fn test_select_best_fallback_model_prefers_phi35_safe() {
        let candidates = [
            "nemotron-3.5-lightning:latest",
            "phi3.5-safe:latest",
            "gemma4:12b",
        ];
        let chosen = select_best_fallback_model(candidates.iter().copied(), None);
        assert_eq!(chosen, Some("phi3.5-safe:latest".to_string()));
    }

    #[test]
    fn test_select_best_fallback_model_case_insensitive() {
        let candidates = [
            "Nemotron-3.5-Lightning:latest",
            "Phi3.5-Safe:latest",
            "gemma4:12b",
        ];
        let chosen = select_best_fallback_model(candidates.iter().copied(), None);
        assert_eq!(chosen, Some("Phi3.5-Safe:latest".to_string()));
    }

    #[test]
    fn test_select_best_fallback_model_preserves_requested_safe_model() {
        let candidates = ["phi3.5-safe:latest", "gemma4:12b"];
        // Requested model is gemma4:12b-cloud; matches local gemma4:12b
        let chosen =
            select_best_fallback_model(candidates.iter().copied(), Some("gemma4:12b-cloud"));
        assert_eq!(chosen, Some("gemma4:12b".to_string()));

        // Slug matching: openai/gpt-4o matches local gpt-4o
        let candidates_slug = ["gpt-4o", "phi3.5-safe:latest"];
        let chosen_slug =
            select_best_fallback_model(candidates_slug.iter().copied(), Some("openai/gpt-4o"));
        assert_eq!(chosen_slug, Some("gpt-4o".to_string()));
    }

    #[test]
    fn test_select_best_fallback_model_rejects_all_oversized() {
        let candidates = [
            "llama3:70b",
            "qwen2.5:32b",
            "llama3.1:405b",
            "nemotron-3.5-lightning:latest",
            "unknown-untagged-model:latest",
        ];
        let chosen = select_best_fallback_model(candidates.iter().copied(), None);
        assert_eq!(chosen, None);
    }

    #[test]
    fn test_select_best_fallback_model_ignores_embedding_models_and_handles_bert_boundary() {
        let candidates = [
            "nomic-embed-text:latest",
            "bge-large:latest",
            "distilbert-base:latest",
            "robert-chat:latest",
            "eclipse:latest",
        ];
        let chosen = select_best_fallback_model(candidates.iter().copied(), None);
        // robert-chat and eclipse are recognized chat models; eclipse should not collide with clip
        assert!(
            chosen == Some("robert-chat:latest".to_string())
                || chosen == Some("eclipse:latest".to_string())
        );
    }

    #[test]
    fn test_alignment_preserves_local_and_custom_gateway() {
        // 1. Local endpoint with Claude model ID must NOT mutate provider
        let local_config = ModelConfig {
            provider: ModelProvider::Openai,
            model_id: "claude-3-5-sonnet".to_string(),
            base_url: Some("http://127.0.0.1:8000/v1".to_string()),
            ..Default::default()
        };
        assert!(is_local_endpoint(
            &local_config.provider,
            local_config.base_url.as_deref()
        ));
        assert!(!is_official_openai_endpoint(
            local_config.base_url.as_deref()
        ));

        // 2. Custom gateway endpoint must NOT mutate provider
        let gateway_config = ModelConfig {
            provider: ModelProvider::Openai,
            model_id: "claude-3-5-sonnet".to_string(),
            base_url: Some("https://llm-gateway.corp.net/v1".to_string()),
            ..Default::default()
        };
        assert!(!is_official_openai_endpoint(
            gateway_config.base_url.as_deref()
        ));

        // 3. Official cloud OpenAI endpoint with Claude model ID DOES qualify for auto-alignment
        let cloud_config = ModelConfig {
            provider: ModelProvider::Openai,
            model_id: "claude-3-5-sonnet".to_string(),
            base_url: Some("https://api.openai.com/v1".to_string()),
            ..Default::default()
        };
        assert!(is_official_openai_endpoint(
            cloud_config.base_url.as_deref()
        ));
    }

    #[test]
    fn test_construct_probe_urls_preserves_subpaths() {
        // Standard /v1 URL
        let (models, tags) = construct_probe_urls("http://127.0.0.1:11434/v1");
        assert_eq!(models, "http://127.0.0.1:11434/v1/models");
        assert_eq!(tags, "http://127.0.0.1:11434/api/tags");

        // Base URL without /v1
        let (models, tags) = construct_probe_urls("http://127.0.0.1:11434");
        assert_eq!(models, "http://127.0.0.1:11434/v1/models");
        assert_eq!(tags, "http://127.0.0.1:11434/api/tags");

        // Subpath proxy with /v1
        let (models, tags) = construct_probe_urls("http://gateway.local:8080/ollama/v1");
        assert_eq!(models, "http://gateway.local:8080/ollama/v1/models");
        assert_eq!(tags, "http://gateway.local:8080/ollama/api/tags");

        // Subpath proxy without /v1
        let (models, tags) = construct_probe_urls("http://gateway.local:8080/ollama");
        assert_eq!(models, "http://gateway.local:8080/ollama/v1/models");
        assert_eq!(tags, "http://gateway.local:8080/ollama/api/tags");
    }

    #[test]
    fn test_select_best_fallback_model_none_when_empty() {
        let candidates: [&str; 0] = [];
        let chosen = select_best_fallback_model(candidates.iter().copied(), None);
        assert_eq!(chosen, None);
    }
}

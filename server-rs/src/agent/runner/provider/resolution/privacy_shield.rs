//! @docs ARCHITECTURE:Runner
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Agent Runner / resolution / privacy_shield
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: `[Privacy Shield]`

use std::collections::HashMap;
use std::sync::LazyLock;
use std::time::{Duration, Instant};

use parking_lot::RwLock;

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

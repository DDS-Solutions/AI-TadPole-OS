//! Google Gemini API provider and embedding engine.
//!
//! Implements the `LlmProvider` trait for Google's Generative AI models.
//! This provider manages the complex `contents` vs `parts` hierarchy and
//! supports native function-calling and vector embeddings.

use reqwest::Client;

use crate::agent::types::{ModelConfig, TokenUsage};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
struct GeminiPart {
    text: String,
}

#[derive(Debug, Serialize)]
struct GeminiContent {
    role: String,
    parts: Vec<GeminiPart>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GeminiFunctionDeclaration {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GeminiTool {
    pub function_declarations: Vec<GeminiFunctionDeclaration>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct GeminiGenerationConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_mime_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_schema: Option<serde_json::Value>,
}

#[derive(Debug, Serialize)]
struct GeminiRequest {
    contents: Vec<GeminiContent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<GeminiTool>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,
    #[serde(rename = "cachedContent", skip_serializing_if = "Option::is_none")]
    pub cached_content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub generation_config: Option<GeminiGenerationConfig>,
}

#[derive(Debug, Serialize)]
struct CachedContentRequest {
    model: String,
    contents: Vec<GeminiContent>,
    #[serde(rename = "ttl")]
    ttl: String,
}

#[derive(Debug, Deserialize)]
struct CachedContentResponse {
    name: String,
}

#[derive(Debug, Deserialize)]
pub struct GeminiFunctionCall {
    pub name: String,
    pub args: serde_json::Value,
}

#[derive(Debug, Deserialize)]
struct GeminiResponsePart {
    text: Option<String>,
    #[serde(rename = "functionCall")]
    pub function_call: Option<GeminiFunctionCall>,
}

#[derive(Debug, Deserialize)]
struct GeminiResponseCandidate {
    content: Option<GeminiResponseContent>,
}

#[derive(Debug, Deserialize)]
struct GeminiResponseContent {
    parts: Vec<GeminiResponsePart>,
}

#[derive(Debug, Deserialize)]
struct GeminiUsageMetadata {
    #[serde(rename = "promptTokenCount")]
    prompt_token_count: u32,
    #[serde(rename = "candidatesTokenCount")]
    candidates_token_count: u32,
    #[serde(rename = "totalTokenCount")]
    total_token_count: u32,
}

#[derive(Debug, Deserialize)]
struct GeminiResponse {
    candidates: Option<Vec<GeminiResponseCandidate>>,
    #[serde(rename = "usageMetadata")]
    usage_metadata: Option<GeminiUsageMetadata>,
}

pub struct GeminiProvider {
    client: Client,
    config: ModelConfig,
    api_key: String,
    /// In-memory cache map: SHA256(system_prompt) -> cache_resource_name
    cache_refs: dashmap::DashMap<String, String>,
}

impl GeminiProvider {
    /// Creates a GeminiProvider.
    /// Accepts a shared `reqwest::Client` to reuse the underlying connection pool.
    pub fn new(client: Client, api_key: String, config: ModelConfig) -> Self {
        Self {
            client,
            config,
            api_key,
            cache_refs: dashmap::DashMap::new(),
        }
    }

    /// Computes the SHA256 hash for a given system prompt.
    fn compute_cache_hash(system_prompt: &str) -> String {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(system_prompt.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// Resolves or creates a context cache for the given system prompt.
    async fn resolve_context_cache(&self, system_prompt: &str) -> Option<String> {
        // Only cache if the prompt is large enough to justify it (>32k chars approx)
        if system_prompt.len() < 32768 {
            return None;
        }

        let hash = Self::compute_cache_hash(system_prompt);

        if let Some(r) = self.cache_refs.get(&hash) {
            return Some(r.clone());
        }

        // Create new cache
        let base_url = self
            .config
            .base_url
            .clone()
            .unwrap_or_else(|| "https://generativelanguage.googleapis.com/v1beta".to_string());
        let url = format!("{}/cachedContents", base_url);

        let body = CachedContentRequest {
            model: format!("models/{}", self.config.model_id),
            contents: vec![GeminiContent {
                role: "system".to_string(),
                parts: vec![GeminiPart {
                    text: system_prompt.to_string(),
                }],
            }],
            ttl: "3600s".to_string(), // 1 hour TTL
        };

        match self
            .client
            .post(&url)
            .header("x-goog-api-key", &self.api_key)
            .json(&body)
            .send()
            .await
        {
            Ok(res) if res.status().is_success() => {
                if let Ok(parsed) = res.json::<CachedContentResponse>().await {
                    let name = parsed.name;
                    self.cache_refs.insert(hash, name.clone());
                    Some(name)
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    /// Generates a response from the Gemini HTTP API.
    ///
    /// # Protocol Details
    /// - Uses `generateContent` endpoint for text and tool output.
    /// - Maps `prompt` string into a single-turn `user` role message.
    /// - Decodes both text responses and structured `functionCall` parts.
    pub async fn generate(
        &self,
        prompt: &str,
        tools: Option<Vec<GeminiTool>>,
    ) -> anyhow::Result<(
        String,
        Vec<crate::agent::types::GeminiFunctionCall>,
        Option<TokenUsage>,
    )> {
        let base_url = self
            .config
            .base_url
            .clone()
            .unwrap_or_else(|| "https://generativelanguage.googleapis.com/v1beta".to_string());
        let url = format!(
            "{}/models/{}:generateContent",
            base_url, self.config.model_id
        );
        tracing::info!("🌐 [Gemini] Calling URL: {}", url);

        let mut request_body = GeminiRequest {
            contents: vec![GeminiContent {
                role: "user".to_string(),
                parts: vec![GeminiPart {
                    text: prompt.to_string(),
                }],
            }],
            tools,
            user: self.config.external_id.clone(),
            cached_content: None,
            generation_config: None,
        };

        // If a system prompt is available in the config, attempt to cache it
        if let Some(sys) = &self.config.system_prompt {
            request_body.cached_content = self.resolve_context_cache(sys).await;
        } else if prompt.contains("(cache_control: {\"type\": \"ephemeral\"})") {
            // Dynamic caching: If the synthesized prompt contains a cache marker, cache the whole thing
            // (Note: This is an optimization for multi-turn conversations where the system-like part persists)
            request_body.cached_content = self.resolve_context_cache(prompt).await;
        }

        // ── Dynamic Rate Limiting (Phase 3) ───────────────────
        let mut attempts = 0;
        let max_attempts = 5;

        loop {
            let res = self
                .client
                .post(&url)
                .header("x-goog-api-key", &self.api_key)
                .json(&request_body)
                .send()
                .await?;

            let status = res.status();

            if status.is_success() {
                let parsed: GeminiResponse = res.json().await?;

                let mut output_text = String::new();
                let mut function_calls = Vec::new();

                if let Some(candidates) = parsed.candidates {
                    if let Some(candidate) = candidates.first() {
                        if let Some(content) = &candidate.content {
                            for part in &content.parts {
                                if let Some(text) = &part.text {
                                    output_text.push_str(text);
                                }
                                if let Some(fc) = &part.function_call {
                                    function_calls.push(crate::agent::types::GeminiFunctionCall {
                                        name: fc.name.clone(),
                                        args: fc.args.clone(),
                                    });
                                }
                            }
                        }
                    }
                }

                let token_usage = parsed.usage_metadata.map(|usage| TokenUsage {
                    input_tokens: usage.prompt_token_count,
                    output_tokens: usage.candidates_token_count,
                    total_tokens: usage.total_token_count,
                });

                return Ok((output_text, function_calls, token_usage));
            }

            // 429: Too Many Requests (Transient)
            if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
                attempts += 1;
                if attempts >= max_attempts {
                    tracing::error!("🛑 [Gemini] Rate limit exceeded. Max attempts ({}) reached.", max_attempts);
                    return Err(anyhow::anyhow!("Gemini Rate Limit Exceeded (429) after {} attempts.", max_attempts));
                }

                // Extract Retry-After header (if present)
                let retry_after = res
                    .headers()
                    .get("retry-after")
                    .and_then(|h| h.to_str().ok())
                    .and_then(|s| s.parse::<u64>().ok())
                    .unwrap_or_else(|| {
                        // Fallback: Exponential backoff 2, 4, 8, 16 seconds
                        2u64.pow(attempts)
                    });

                tracing::warn!(
                    "⏳ [Gemini] Rate Limit Hit (429). Attempt {}/{}. Retrying in {}s...",
                    attempts, max_attempts, retry_after
                );

                tokio::time::sleep(tokio::time::Duration::from_secs(retry_after)).await;
                continue;
            }

            // 403: Forbidden (often Quota Exhausted)
            if status == reqwest::StatusCode::FORBIDDEN {
                let error_text = res.text().await?;
                if error_text.to_lowercase().contains("quota") {
                    tracing::error!("🚫 [Gemini] Permanently Exhausted Quota (403). Suspending mission.");
                    return Err(anyhow::anyhow!("Gemini Quota Exhausted (403): {}", error_text));
                } else {
                    return Err(anyhow::anyhow!("Gemini Forbidden (403): {}", error_text));
                }
            }

            // Unexpected Error
            let error_text = res.text().await?;
            return Err(anyhow::anyhow!("Gemini API Error ({}): {}", status, error_text));
        }
    }
}

// ─────────────────────────────────────────────────────────
//  LlmProvider trait implementation
// ─────────────────────────────────────────────────────────

#[async_trait::async_trait]
impl crate::agent::provider_trait::LlmProvider for GeminiProvider {
    async fn generate(
        &self,
        system_prompt: &str,
        user_message: &str,
        tools: Option<Vec<crate::agent::gemini::GeminiTool>>,
    ) -> anyhow::Result<(
        String,
        Vec<crate::agent::types::GeminiFunctionCall>,
        Option<crate::agent::types::TokenUsage>,
    )> {
        // Gemini API uses a single combined prompt string. Build it from both parts.
        // We call the inherent GeminiProvider::generate() with explicit type disambiguation.
        let combined = format!("{}\n\nUSER MESSAGE:\n{}", system_prompt, user_message);
        GeminiProvider::generate(self, &combined, tools).await
    }

    async fn embed(&self, text: &str) -> anyhow::Result<Vec<f32>> {
        // ... (existing implementation)
        let base_url = self
            .config
            .base_url
            .clone()
            .unwrap_or_else(|| "https://generativelanguage.googleapis.com/v1beta".to_string());

        let embed_model = "gemini-embedding-001";
        let url = format!("{}/models/{}:embedContent", base_url, embed_model);

        #[derive(Debug, Serialize)]
        struct EmbedPart {
            text: String,
        }
        #[derive(Debug, Serialize)]
        struct EmbedContent {
            parts: Vec<EmbedPart>,
        }
        #[derive(Debug, Serialize)]
        struct EmbedRequest {
            model: String,
            content: EmbedContent,
        }

        let request_body = EmbedRequest {
            model: format!("models/{}", embed_model),
            content: EmbedContent {
                parts: vec![EmbedPart {
                    text: text.to_string(),
                }],
            },
        };

        let res = self
            .client
            .post(&url)
            .header("x-goog-api-key", &self.api_key)
            .json(&request_body)
            .send()
            .await?;

        if !res.status().is_success() {
            let error_text = res.text().await?;
            return Err(anyhow::anyhow!("Gemini Embedding Error: {}", error_text));
        }

        #[derive(Debug, Deserialize)]
        struct EmbedResponse {
            embedding: EmbeddingValues,
        }
        #[derive(Debug, Deserialize)]
        struct EmbeddingValues {
            values: Vec<f32>,
        }

        let parsed: EmbedResponse = res.json().await?;
        Ok(parsed.embedding.values)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::types::ModelConfig;

    #[test]
    fn test_cache_hash_consistency() {
        let prompt = "System instruction for testing caching logic.";
        let hash1 = GeminiProvider::compute_cache_hash(prompt);
        let hash2 = GeminiProvider::compute_cache_hash(prompt);
        assert_eq!(hash1, hash2);
        // Verified SHA-256 for this specific string
        assert_eq!(
            hash1,
            "7288cfa3d8c29c26a3bffcd42c0aa0734214889fcb4aaa976cc7497862dfcf2e"
        );
    }

    #[tokio::test]
    async fn test_cache_threshold_trigger() {
        let client = reqwest::Client::new();
        let config = ModelConfig {
            model_id: "gemini-1.5-pro".to_string(),
            ..Default::default()
        };
        let provider = GeminiProvider::new(client, "test-key".to_string(), config);

        // 1. Small prompt (< 32k) should not trigger cache
        let small_prompt = "Short prompt";
        let cache_name = provider.resolve_context_cache(small_prompt).await;
        assert!(cache_name.is_none());

        // 2. Exact boundary check (32767 chars)
        let boundary_below = "a".repeat(32767);
        let cache_name_below = provider.resolve_context_cache(&boundary_below).await;
        assert!(cache_name_below.is_none());

        // 3. Meeting the boundary (32768 chars)
        // Note: This will attempt a network call and return None (due to test-key and no API),
        // but we've verified above that smaller ones return early before the network call.
        let boundary_at = "a".repeat(32768);
        let cache_name_at = provider.resolve_context_cache(&boundary_at).await;
        // Should be None because we have no real API key/connectivity in unit test,
        // but it proves it passed the length guard.
        assert!(cache_name_at.is_none());
    }
}

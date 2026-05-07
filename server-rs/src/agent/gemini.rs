//! @docs ARCHITECTURE:Agent
//!
//! ### AI Assist Note
//! **Gemini Native Bridge**: Implements the `LlmProvider` trait for Google's
//! **Gemini 1.5 Pro/Flash** models. Orchestrates **Context Caching**
//! (32k+ char threshold) to reduce latency and cost in multi-turn
//! missions. Features **Safe Multimodal Handling** and automated mapping
//! from the engine's flat message structure to the `contents[] -> parts[]`
//! hierarchy.
//!
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: 429 Rate Limit (transient), 403 Quota Exhausted
//!   (fatal), or cache miss due to TTL expiration.
//! - **Trace Scope**: `server-rs::agent::gemini`

use reqwest::Client;
use crate::agent::types::{ModelConfig, TokenUsage, ToolCall, ToolDefinition};
use crate::error::AppError;
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
    pub tools: Option<Vec<ToolDefinition>>,
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
struct GeminiResponsePart {
    text: Option<String>,
    #[serde(rename = "functionCall")]
    pub function_call: Option<ToolCall>,
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
        hex::encode(hasher.finalize())
    }

    async fn resolve_context_cache(&self, system_prompt: &str) -> Option<String> {
        if system_prompt.len() < 32768 {
            return None;
        }

        let hash = Self::compute_cache_hash(system_prompt);

        if let Some(r) = self.cache_refs.get(&hash) {
            return Some(r.clone());
        }

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
            ttl: "3600s".to_string(),
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

    pub async fn generate(
        &self,
        system_prompt: &str,
        user_message: &str,
        tools: Option<Vec<ToolDefinition>>,
    ) -> Result<(String, Vec<ToolCall>, Option<TokenUsage>), AppError> {
        let base_url = self
            .config
            .base_url
            .clone()
            .unwrap_or_else(|| "https://generativelanguage.googleapis.com/v1beta".to_string());
        let url = format!(
            "{}/models/{}:generateContent",
            base_url, self.config.model_id
        );

        let combined_prompt = format!("{}\n\nUSER MESSAGE:\n{}", system_prompt, user_message);

        let mut request_body = GeminiRequest {
            contents: vec![GeminiContent {
                role: "user".to_string(),
                parts: vec![GeminiPart {
                    text: combined_prompt.clone(),
                }],
            }],
            tools,
            user: self.config.external_id.clone(),
            cached_content: None,
            generation_config: None,
        };

        if let Some(sys) = &self.config.system_prompt {
            request_body.cached_content = self.resolve_context_cache(sys).await;
        } else if combined_prompt.contains("(cache_control: {\"type\": \"ephemeral\"})") {
            request_body.cached_content = self.resolve_context_cache(&combined_prompt).await;
        }

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
                let parsed: GeminiResponse = res.json().await.map_err(|e| AppError::InternalServerError(e.to_string()))?;

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
                                    function_calls.push(ToolCall {
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

            if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
                attempts += 1;
                if attempts >= max_attempts {
                    return Err(AppError::RateLimit(format!(
                        "Gemini Rate Limit Exceeded after {} attempts.",
                        max_attempts
                    )));
                }

                let retry_after = res
                    .headers()
                    .get("retry-after")
                    .and_then(|h| h.to_str().ok())
                    .and_then(|s| s.parse::<u64>().ok())
                    .unwrap_or_else(|| {
                        2u64.pow(attempts)
                    });

                tracing::warn!(
                    "⏳ [Gemini] Rate Limit Hit (429). Attempt {}/{}. Retrying in {}s...",
                    attempts,
                    max_attempts,
                    retry_after
                );

                tokio::time::sleep(tokio::time::Duration::from_secs(retry_after)).await;
                continue;
            }

            if status == reqwest::StatusCode::FORBIDDEN {
                let error_text = res.text().await?;
                if error_text.to_lowercase().contains("quota") {
                    return Err(AppError::InfrastructureError {
                        provider_id: "gemini".to_string(),
                        detail: format!("Quota Exhausted: {}", error_text),
                        help_link: Some("https://console.cloud.google.com/billing".to_string()),
                    });
                } else {
                    return Err(AppError::Forbidden(error_text));
                }
            }

            let error_text = res.text().await?;
            return Err(AppError::InfrastructureError {
                provider_id: "gemini".to_string(),
                detail: error_text,
                help_link: None,
            });
        }
    }
}

#[async_trait::async_trait]
impl crate::agent::provider_trait::LlmProvider for GeminiProvider {
    async fn generate(
        &self,
        system_prompt: &str,
        user_message: &str,
        tools: Option<Vec<ToolDefinition>>,
    ) -> Result<(
        String,
        Vec<ToolCall>,
        Option<TokenUsage>,
    ), AppError> {
        GeminiProvider::generate(self, system_prompt, user_message, tools).await
    }

    async fn embed(&self, text: &str) -> Result<Vec<f32>, AppError> {
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
            return Err(AppError::InfrastructureError {
                provider_id: "gemini_embedding".to_string(),
                detail: error_text,
                help_link: None,
            });
        }

        #[derive(Debug, Deserialize)]
        struct EmbedResponse {
            embedding: EmbeddingValues,
        }
        #[derive(Debug, Deserialize)]
        struct EmbeddingValues {
            values: Vec<f32>,
        }

        let parsed: EmbedResponse = res.json().await.map_err(|e| AppError::InternalServerError(e.to_string()))?;
        Ok(parsed.embedding.values)
    }
}

// Metadata: [gemini]

// Metadata: [gemini]

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
struct GeminiRequest {
    contents: Vec<GeminiContent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<GeminiTool>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,
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
}

impl GeminiProvider {
    /// Creates a GeminiProvider.
    /// Accepts a shared `reqwest::Client` to reuse the underlying connection pool.
    pub fn new(client: Client, api_key: String, config: ModelConfig) -> Self {
        Self {
            client,
            config,
            api_key,
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

        let request_body = GeminiRequest {
            contents: vec![GeminiContent {
                role: "user".to_string(),
                parts: vec![GeminiPart {
                    text: prompt.to_string(),
                }],
            }],
            tools,
            user: self.config.external_id.clone(),
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
            return Err(anyhow::anyhow!("Gemini API Error: {}", error_text));
        }

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

        Ok((output_text, function_calls, token_usage))
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
        let base_url = self
            .config
            .base_url
            .clone()
            .unwrap_or_else(|| "https://generativelanguage.googleapis.com/v1beta".to_string());
        let url = format!(
            "{}/models/{}:embedContent",
            base_url, self.config.model_id
        );

        #[derive(Debug, Serialize)]
        struct EmbedPart { text: String }
        #[derive(Debug, Serialize)]
        struct EmbedContent { parts: Vec<EmbedPart> }
        #[derive(Debug, Serialize)]
        struct EmbedRequest { model: String, content: EmbedContent }

        let request_body = EmbedRequest {
            model: format!("models/{}", self.config.model_id),
            content: EmbedContent {
                parts: vec![EmbedPart { text: text.to_string() }]
            }
        };

        let res = self.client.post(&url)
            .header("x-goog-api-key", &self.api_key)
            .json(&request_body)
            .send()
            .await?;

        if !res.status().is_success() {
            let error_text = res.text().await?;
            return Err(anyhow::anyhow!("Gemini Embedding Error: {}", error_text));
        }

        #[derive(Debug, Deserialize)]
        struct EmbedResponse { embedding: EmbeddingValues }
        #[derive(Debug, Deserialize)]
        struct EmbeddingValues { values: Vec<f32> }

        let parsed: EmbedResponse = res.json().await?;
        Ok(parsed.embedding.values)
    }
}

//! @docs ARCHITECTURE:Agent
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Agent Runner / openai
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Type-safe state handling and bounded execution without unhandled panics.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: `[OML-01]`, `[Recovery]`, `[OpenAI]`
//! - **Witness Tests**: none declared

use crate::agent::recovery::extract_tool_call;
use crate::agent::types::{ModelConfig, ModelProvider, TokenUsage, ToolCall, ToolDefinition};
use crate::error::{AppError, InfrastructureErrorKind, ProviderId};
use reqwest::{header, Client};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
struct OpenAIMessage {
    role: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    content: Option<String>,
}

#[derive(Debug, Serialize)]
struct OpenAITool {
    #[serde(rename = "type")]
    tool_type: String,
    function: OpenAIFunctionDefinition,
}

#[derive(Debug, Serialize)]
struct OpenAIFunctionDefinition {
    name: String,
    description: String,
    parameters: serde_json::Value,
}

#[derive(Debug, Serialize)]
struct OpenAIRequest {
    model: String,
    messages: Vec<OpenAIMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    user: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tools: Option<Vec<OpenAITool>>,
    #[serde(flatten)]
    #[serde(skip_serializing_if = "Option::is_none")]
    extra_parameters: Option<std::collections::HashMap<String, serde_json::Value>>,
}

/// Deserializer that accepts either a JSON-encoded string or a structured JSON map/value.
fn deserialize_arguments<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let val = serde_json::Value::deserialize(deserializer)?;
    match val {
        serde_json::Value::String(s) => Ok(s),
        serde_json::Value::Null => Ok(String::new()),
        other => Ok(other.to_string()),
    }
}

#[derive(Debug, Deserialize, Default)]
struct OpenAIChoice {
    #[serde(default)]
    message: Option<OpenAIResponseMessage>,
    #[serde(default)]
    delta: Option<OpenAIResponseMessage>,
    #[serde(default)]
    text: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
struct OpenAIResponseMessage {
    #[serde(default)]
    content: Option<String>,
    #[serde(default)]
    reasoning: Option<String>,
    #[serde(default, rename = "tool_calls")]
    tool_calls: Option<Vec<OpenAIToolCall>>,
}

#[derive(Debug, Deserialize, Default)]
struct OpenAIToolCall {
    #[serde(default)]
    function: Option<OpenAIFunctionCall>,
}

#[derive(Debug, Deserialize, Default)]
struct OpenAIFunctionCall {
    #[serde(default)]
    name: String,
    #[serde(default, deserialize_with = "deserialize_arguments")]
    arguments: String,
}

#[derive(Debug, Deserialize, Default)]
struct OpenAIUsage {
    #[serde(default)]
    prompt_tokens: Option<u64>,
    #[serde(default)]
    completion_tokens: Option<u64>,
    #[serde(default)]
    total_tokens: Option<u64>,
}

#[derive(Debug, Deserialize, Default)]
struct OpenAIResponse {
    #[serde(default)]
    choices: Vec<OpenAIChoice>,
    #[serde(default)]
    usage: Option<OpenAIUsage>,
}

pub struct OpenAIProvider {
    pub(crate) client: Client,
    pub(crate) config: ModelConfig,
    pub(crate) api_key: String,
}

impl OpenAIProvider {
    pub fn new(client: Client, api_key: String, config: ModelConfig) -> Self {
        Self {
            client,
            config,
            api_key,
        }
    }

    pub async fn generate(
        &self,
        system_prompt: &str,
        user_message: &str,
        tools: Option<Vec<ToolDefinition>>,
    ) -> Result<(String, Vec<ToolCall>, Option<TokenUsage>), AppError> {
        self.generate_internal(system_prompt, user_message, tools, None)
            .await
    }

    /// Main generation loop for OpenAI-compatible models.
    ///
    /// ### 🛠️ Resiliency Layers
    /// 1. **Tool Mapping**: Translating Gemini-style tools to OpenAI's tool-call schema.
    /// 2. **Retry Logic**: Injecting error-correction instructions if previous attempts failed.
    /// 3. **Native vs Bridge**: Detecting if the model supports native tools or needs tag parsing.
    async fn generate_internal(
        &self,
        system_prompt: &str,
        user_message: &str,
        tools: Option<Vec<ToolDefinition>>,
        retry_msg: Option<String>,
    ) -> Result<(String, Vec<ToolCall>, Option<TokenUsage>), AppError> {
        self.generate_with_fallback(system_prompt, user_message, tools, retry_msg, false)
            .await
    }

    /// ### 📡 Intelligence Orchestration: OpenAI Native/Bridge (generate_with_fallback)
    /// Orchestrates the primary generative heartbeat for OpenAI-compatible providers.
    /// Handles native tool calls, tag extraction, and recursive failure recovery.
    ///
    /// ### 🧬 Logic: Resilient Execution (OML-01)
    /// 1. **IPv4 Loopback Enforcement**: Resolves IPv4/IPv6 localhost conflicts
    ///    by forcing `127.0.0.1` for local sidecars.
    /// 2. **Tool Schema Mapping**: Translates engine-standard tools into
    ///    OpenAI's `tool_type: function` format.
    /// 3. **Dynamic Quantization Fallback**: If a local provider (Ollama)
    ///    returns an OOM (Out Of Memory) error, the engine automatically
    ///    downgrades the model to a `Q4_K_M` quantization to prevent mission failure.
    /// 4. **In-Flight JSON Correction**: Intercepts `400` errors related to
    ///    malformed tool use and performs a manual regex extraction pass.
    /// 5. **Hallucination Cleanup**: Manually extracts and strips raw function
    ///    tags leaked into the content by models like Llama-3 or Phi-3.
    async fn generate_with_fallback(
        &self,
        system_prompt: &str,
        user_message: &str,
        tools: Option<Vec<ToolDefinition>>,
        retry_msg: Option<String>,
        is_fallback: bool,
    ) -> Result<(String, Vec<ToolCall>, Option<TokenUsage>), AppError> {
        let mut url = if let Some(base) = self
            .config
            .base_url
            .as_ref()
            .filter(|s| !s.trim().is_empty())
        {
            base.clone()
        } else if self.config.provider == crate::agent::types::ModelProvider::Ollama {
            "http://127.0.0.1:11434/v1".to_string()
        } else {
            "https://api.openai.com/v1".to_string()
        };

        // ### 🛡️ Sector Defense: Localhost Conflict Resolution
        // Resolve IPv4/IPv6 localhost conflict on Windows and Docker boundaries dynamically.
        if url.contains("localhost") || url.contains("127.0.0.1") {
            url = crate::networking::resolver::AddressResolver::resolve_url_if_local(&url).await;
        }

        // Canonicalize the completions endpoint
        if !url.ends_with("/chat/completions") && !url.contains("/v1/audio") {
            if !url.ends_with('/') {
                url.push('/');
            }
            url.push_str("chat/completions");
        }

        // Map Gemini tools to OpenAI tools
        let openai_tools = tools.as_ref().map(|ts| {
            ts.iter()
                .flat_map(|t| {
                    t.function_declarations.iter().map(|f| OpenAITool {
                        tool_type: "function".to_string(),
                        function: OpenAIFunctionDefinition {
                            name: f.name.clone(),
                            description: f.description.clone(),
                            parameters: f.parameters.clone(),
                        },
                    })
                })
                .collect::<Vec<OpenAITool>>()
        });

        let mut messages = Vec::new();

        let has_vectors = self
            .config
            .steering_vectors
            .as_ref()
            .is_some_and(|v| !v.is_empty());

        messages.push(OpenAIMessage {
            role: "system".to_string(),
            content: Some(system_prompt.to_string()),
        });

        if has_vectors {
            tracing::debug!("🧠 Steering vectors active in addition to system prompt.");
        }

        messages.push(OpenAIMessage {
            role: "user".to_string(),
            content: Some(user_message.to_string()),
        });

        // If this is a retry, append the failed generation and correction instruction
        if let Some(ref r) = retry_msg {
            messages.push(OpenAIMessage {
                role: "assistant".to_string(),
                content: Some(r.clone()),
            });
            messages.push(OpenAIMessage {
                role: "user".to_string(),
                content: Some("CRITICAL ERROR: Your previous tool call was malformed. Please fix the JSON syntax and try again. Ensure all arguments are inside the brackets and there are no stray characters.".to_string()),
            });
        }

        let request_body = OpenAIRequest {
            model: self.config.model_id.clone(),
            messages,
            temperature: self.config.temperature,
            max_tokens: {
                let mut limit = self.config.max_tokens;
                if limit.is_none()
                    && (self.config.provider == ModelProvider::Ollama
                        || self.config.provider == ModelProvider::OllamaCloud)
                {
                    limit = Some(2048);
                }
                limit
            },
            user: self.config.external_id.clone(),
            tools: if openai_tools.as_ref().is_none_or(|t| t.is_empty()) {
                None
            } else if !self.config.supports_native_tools() {
                tracing::warn!(
                    "🛡️ [Provider] Suppressing tools for '{}' (Incompatible with bridge)",
                    self.config.model_id
                );
                None
            } else {
                openai_tools
            },
            extra_parameters: {
                let mut p = self.config.extra_parameters.clone().unwrap_or_default();
                if let Some(ref vectors) = self.config.steering_vectors {
                    if !vectors.is_empty() {
                        p.insert("steering_vectors".to_string(), serde_json::json!(vectors));
                    }
                }
                if p.is_empty() {
                    None
                } else {
                    Some(p)
                }
            },
        };

        // ### 📡 Resiliency: Exponential Backoff Retry Loop
        let mut attempts = 0;
        let max_attempts = 5; // 🛡️ [Hardening] Increase from 3 for stress tests

        let res = loop {
            let res_result = self
                .client
                .post(&url)
                .header(header::AUTHORIZATION, format!("Bearer {}", self.api_key))
                .json(&request_body)
                .send()
                .await;

            match res_result {
                Ok(r) => {
                    if r.status() == reqwest::StatusCode::TOO_MANY_REQUESTS
                        && attempts < max_attempts - 1
                    {
                        attempts += 1;
                        let delay = (1500u64 * (attempts as u64)) + (rand::random::<u64>() % 300);
                        tracing::warn!(
                            "⏳ [OpenAI/OpenRouter] Upstream rate limit (HTTP 429). Backing off for {}ms... (Attempt {}/{})",
                            delay,
                            attempts + 1,
                            max_attempts
                        );
                        tokio::time::sleep(std::time::Duration::from_millis(delay)).await;
                        continue;
                    }
                    break r;
                }
                Err(e) => {
                    let is_ollama_offline = self.config.provider == ModelProvider::Ollama
                        && (e.is_connect() || e.is_timeout());

                    let app_err = if is_ollama_offline {
                        AppError::OllamaOffline(e.to_string())
                    } else {
                        AppError::from(e)
                    };

                    if app_err.is_retryable() && attempts < max_attempts - 1 {
                        attempts += 1;
                        // Add jitter (±100ms) to prevent "thundering herd" issues during recovery
                        let jitter = (rand::random::<u64>() % 200) as i64 - 100;
                        let delay = ((500u64 << attempts) as i64 + jitter).max(100) as u64;

                        tracing::warn!(
                            "⚠️ [OpenAI] Request failed ({}). Retrying in {}ms... (Attempt {}/{})",
                            app_err,
                            delay,
                            attempts + 1,
                            max_attempts
                        );
                        tokio::time::sleep(std::time::Duration::from_millis(delay)).await;
                        continue;
                    }

                    tracing::error!(
                        "🚨 [OpenAI] FATAL: Provider connectivity failure after {} attempts. URL: {} | Error: {}",
                        max_attempts, url, app_err
                    );
                    return Err(app_err);
                }
            }
        };

        if !res.status().is_success() {
            let status = res.status();
            let error_text = res
                .text()
                .await
                .unwrap_or_else(|_| "Failed to read error response body".to_string());

            // ### 🧠 Resilience: Dynamic Quantization Fallback (OML-01)
            // Intercept OOM (Out Of Memory) from local providers (Ollama, vLLM, Mercury).
            // Logic: If the primary high-precision model (e.g. Llama-3-70b) fails
            // due to VRAM exhaustion, we down-rank to a Q4_K_M quantization
            // to ensure mission continuity without user intervention.
            let is_local = self.config.provider == crate::agent::types::ModelProvider::Ollama
                || self
                    .config
                    .base_url
                    .as_ref()
                    .is_some_and(|u| u.contains("127.0.0.1") || u.contains("localhost"));

            if is_local && is_oom_error(&error_text) && !is_fallback {
                let current_model = &self.config.model_id;
                // Avoid infinite fallback loops
                if !current_model.contains(":q") && !current_model.contains("-q") {
                    let fallback_model = format!("{}:q4_K_M", current_model);
                    tracing::warn!("🚨 [OML-01] OOM detected for '{}'. Attempting Dynamic Quantization Fallback to '{}'...", current_model, fallback_model);

                    let mut fallback_config = self.config.clone();
                    fallback_config.model_id = fallback_model;

                    let fallback_provider = OpenAIProvider {
                        client: self.client.clone(),
                        config: fallback_config,
                        api_key: self.api_key.clone(),
                    };

                    return Box::pin(fallback_provider.generate_with_fallback(
                        system_prompt,
                        user_message,
                        tools,
                        retry_msg,
                        true,
                    ))
                    .await;
                }
            }

            // In-Flight JSON Correction
            if status == 400 && error_text.contains("tool_use_failed") {
                if let Ok(err_json) = serde_json::from_str::<serde_json::Value>(&error_text) {
                    if let Some(failed_gen) = err_json["error"]["failed_generation"].as_str() {
                        tracing::info!(
                            "🛠️ [OpenAI] Native tool failure detected. Generation: {}",
                            failed_gen
                        );
                        // 1. Attempt manual balanced-brace parsing of the failed generation
                        if let Some((name, args_str, raw_match)) = extract_tool_call(failed_gen) {
                            let mut json_str = args_str.trim().to_string();
                            if !json_str.starts_with('{') {
                                json_str.insert(0, '{');
                            }
                            if !json_str.ends_with('}') {
                                json_str.push('}');
                            }

                            let args: serde_json::Value = serde_json::from_str(&json_str)
                                .unwrap_or_else(|e| {
                                    tracing::warn!("🛠️ [Recovery] Failed to parse natively intercepted JSON ({}): {}", e, json_str);
                                    serde_json::json!({})
                                });

                            tracing::info!("🛠️ [OpenAI] Successfully intercepted and recovered tool call '{}' natively.", name);

                            let mut recovered_text = failed_gen.to_string();
                            tracing::debug!(
                                "🛠️ [OpenAI] Stripping recovered tool tag: {}",
                                raw_match
                            );
                            recovered_text =
                                recovered_text.replace(&raw_match, "").trim().to_string();

                            return Ok((recovered_text, vec![ToolCall { name, args }], None));
                        }

                        tracing::warn!(
                            "🛠️ [OpenAI] Recovery failed. Regex did not match failed generation: {}",
                            failed_gen
                        );
                    }
                }
            }

            // Map status codes to AppError
            match status.as_u16() {
                429 => return Err(AppError::RateLimit(error_text)),
                400 => return Err(AppError::BadRequest(error_text)),
                401 | 403 => return Err(AppError::Unauthorized(error_text)),
                _ => {
                    return Err(AppError::InfrastructureError {
                        provider_id: ProviderId::OpenAi,
                        kind: InfrastructureErrorKind::ApiError,
                        detail: error_text,
                        help_link: None,
                    })
                }
            }
        }

        let raw_text = res
            .text()
            .await
            .map_err(|e| AppError::InfrastructureError {
                provider_id: ProviderId::OpenAi,
                kind: InfrastructureErrorKind::NetworkError,
                detail: format!("Failed to read response body stream: {}", e),
                help_link: None,
            })?;

        let parsed: OpenAIResponse = match serde_json::from_str(&raw_text) {
            Ok(p) => p,
            Err(err) => {
                tracing::error!(
                    "🚨 [OpenAI] Failed to decode response JSON ({}). Raw response: {}",
                    err,
                    crate::error::safe_truncate_bytes(&raw_text, 1024)
                );

                // Check if provider returned an API error payload with a 200 OK status
                if let Ok(err_val) = serde_json::from_str::<serde_json::Value>(&raw_text) {
                    if let Some(err_obj) = err_val.get("error") {
                        let msg = err_obj
                            .get("message")
                            .and_then(|m| m.as_str())
                            .or_else(|| err_obj.as_str())
                            .unwrap_or("Unknown upstream API error in 200 OK body");
                        return Err(AppError::InfrastructureError {
                            provider_id: ProviderId::OpenAi,
                            kind: InfrastructureErrorKind::ApiError,
                            detail: format!("Upstream API error: {}", msg),
                            help_link: None,
                        });
                    }
                }

                return Err(AppError::InfrastructureError {
                    provider_id: ProviderId::OpenAi,
                    kind: InfrastructureErrorKind::ApiError,
                    detail: format!(
                        "Error decoding response body ({}): {}",
                        err,
                        crate::error::safe_truncate_bytes(&raw_text, 500)
                    ),
                    help_link: None,
                });
            }
        };

        let choice = parsed
            .choices
            .first()
            .ok_or_else(|| AppError::InfrastructureError {
                provider_id: ProviderId::OpenAi,
                kind: InfrastructureErrorKind::ApiError,
                detail: "No completion choices returned from OpenAI provider".to_string(),
                help_link: None,
            })?;

        let message = choice.message.as_ref().or(choice.delta.as_ref());

        let mut output_text = message
            .and_then(|m| m.content.clone().or_else(|| m.reasoning.clone()))
            .or_else(|| choice.text.clone())
            .unwrap_or_default();

        let mut function_calls = Vec::new();
        if let Some(tool_calls) = message.and_then(|m| m.tool_calls.as_ref()) {
            for tc in tool_calls {
                if let Some(ref func) = tc.function {
                    let args: serde_json::Value = if func.arguments.trim().is_empty() {
                        serde_json::json!({})
                    } else {
                        serde_json::from_str(&func.arguments)
                            .unwrap_or_else(|_| serde_json::json!({}))
                    };
                    function_calls.push(ToolCall {
                        name: func.name.clone(),
                        args,
                    });
                }
            }
        }

        // ### 🛠️ Recovery & Cleanup: Tag-based Tool Extraction
        while let Some((name, args_str, raw_match)) = extract_tool_call(&output_text) {
            let mut json_str = args_str.trim().to_string();
            // JSON bracket healing
            if !json_str.starts_with('{') {
                json_str.insert(0, '{');
            }
            if !json_str.ends_with('}') {
                json_str.push('}');
            }

            let args: serde_json::Value = serde_json::from_str(&json_str).unwrap_or_else(|_| {
                tracing::warn!(
                    "🛠️ [Recovery] Failed to parse recovered JSON from raw format: {}",
                    json_str
                );
                serde_json::json!({})
            });

            tracing::info!("🛠️ [Recovery] Extracted function call from tags: {}", name);
            function_calls.push(ToolCall { name, args });

            // Remove the raw tool call from the output text
            output_text = output_text.replace(&raw_match, "").trim().to_string();
        }

        let token_usage = parsed.usage.map(|u| {
            let input = u.prompt_tokens.unwrap_or(0);
            let output = u.completion_tokens.unwrap_or(0);
            let total = u.total_tokens.unwrap_or(input + output);
            TokenUsage {
                input_tokens: input,
                output_tokens: output,
                total_tokens: total,
            }
        });

        Ok((output_text, function_calls, token_usage))
    }
}

// ─────────────────────────────────────────────────────────
//  LlmProvider trait implementation
// ─────────────────────────────────────────────────────────

#[async_trait::async_trait]
impl crate::agent::provider_trait::LlmProvider for OpenAIProvider {
    async fn generate(
        &self,
        system_prompt: &str,
        user_message: &str,
        tools: Option<Vec<ToolDefinition>>,
    ) -> Result<(String, Vec<ToolCall>, Option<TokenUsage>), AppError> {
        OpenAIProvider::generate(self, system_prompt, user_message, tools).await
    }

    async fn embed(&self, text: &str) -> Result<Vec<f32>, AppError> {
        let mut url = self
            .config
            .base_url
            .clone()
            .unwrap_or_else(|| "https://api.openai.com/v1".to_string());

        // Resolve IPv4/IPv6 localhost conflict on Windows and Docker boundaries dynamically.
        if url.contains("localhost") || url.contains("127.0.0.1") {
            url = crate::networking::resolver::AddressResolver::resolve_url_if_local(&url).await;
        }

        if !url.ends_with("/embeddings") {
            if !url.ends_with('/') {
                url.push('/');
            }
            url.push_str("embeddings");
        }

        #[derive(Debug, Serialize)]
        struct EmbedRequest {
            model: String,
            input: String,
        }

        let request_body = EmbedRequest {
            model: self.config.model_id.clone(),
            input: text.to_string(),
        };

        let res = self
            .client
            .post(&url)
            .header(header::AUTHORIZATION, format!("Bearer {}", self.api_key))
            .json(&request_body)
            .send()
            .await?;

        if !res.status().is_success() {
            let error_text = res.text().await?;
            return Err(AppError::InfrastructureError {
                provider_id: ProviderId::OpenAi,
                kind: InfrastructureErrorKind::ApiError,
                detail: error_text,
                help_link: None,
            });
        }

        #[derive(Debug, Deserialize)]
        struct EmbedData {
            embedding: Vec<f32>,
        }
        #[derive(Debug, Deserialize)]
        struct EmbedResponse {
            data: Vec<EmbedData>,
        }

        let raw_text = res
            .text()
            .await
            .map_err(|e| AppError::InfrastructureError {
                provider_id: ProviderId::OpenAi,
                kind: InfrastructureErrorKind::NetworkError,
                detail: format!("Failed to read embedding response body: {}", e),
                help_link: None,
            })?;

        let parsed: EmbedResponse = serde_json::from_str(&raw_text).map_err(|e| {
            tracing::error!(
                "🚨 [OpenAI] Failed to decode embedding response ({}). Raw: {}",
                e,
                crate::error::safe_truncate_bytes(&raw_text, 1024)
            );
            AppError::InfrastructureError {
                provider_id: ProviderId::OpenAi,
                kind: InfrastructureErrorKind::ApiError,
                detail: format!(
                    "Error decoding embedding response ({}): {}",
                    e,
                    crate::error::safe_truncate_bytes(&raw_text, 500)
                ),
                help_link: None,
            }
        })?;
        Ok(parsed
            .data
            .first()
            .map(|d| d.embedding.clone())
            .unwrap_or_default())
    }
}

/// Detects if an error message indicates an Out-Of-Memory condition.
fn is_oom_error(text: &str) -> bool {
    let lower = text.to_lowercase();

    // Check specific multi-word patterns
    let oom_phrases = [
        "out of memory",
        "insufficient vram",
        "not enough space",
        "more vram than available",
    ];
    if oom_phrases.iter().any(|&phrase| lower.contains(phrase)) {
        return true;
    }

    // Check exact word "oom" to avoid false positives with words like "room"
    lower
        .split(|c: char| !c.is_alphanumeric())
        .any(|word| word == "oom")
}

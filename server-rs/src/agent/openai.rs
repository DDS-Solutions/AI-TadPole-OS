//! @docs ARCHITECTURE:Agent
//!
//! ### AI Assist Note
//! **Universal OpenAI Bridge**: Implements the `LlmProvider` trait for
//! OpenAI-compatible endpoints, including **GPT-4o**, **O1**, and **Groq**.
//! Optimized for **Hallucination Recovery**: features a regex-based
//! extraction engine (`FUNCTION_REGEX`) to repair malformed tool calls
//! (common in Llama 3 models). Intercepts 400 errors to perform
//! **In-Flight JSON Correction** before failing.
//!
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: 429 Rate Limit on bursts (RPM/TPM), 400 Bad Request
//!   on tool schema mismatch, or regex bypass on multi-line JSON blocks
//!   exceeding the capture window.
//! - **Trace Scope**: `server-rs::agent::openai`

use crate::agent::types::{ModelConfig, TokenUsage, ToolCall, ToolDefinition};
use once_cell::sync::Lazy;
use regex::Regex;
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
    user: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tools: Option<Vec<OpenAITool>>,
    #[serde(flatten)]
    #[serde(skip_serializing_if = "Option::is_none")]
    extra_parameters: Option<std::collections::HashMap<String, serde_json::Value>>,
}

#[derive(Debug, Deserialize)]
struct OpenAIChoice {
    message: OpenAIResponseMessage,
}

#[derive(Debug, Deserialize)]
struct OpenAIResponseMessage {
    content: Option<String>,
    #[serde(rename = "tool_calls")]
    tool_calls: Option<Vec<OpenAIToolCall>>,
}

#[derive(Debug, Deserialize)]
struct OpenAIToolCall {
    function: OpenAIFunctionCall,
}

#[derive(Debug, Deserialize)]
struct OpenAIFunctionCall {
    name: String,
    arguments: String,
}

#[derive(Debug, Deserialize)]
struct OpenAIUsage {
    prompt_tokens: u32,
    completion_tokens: u32,
    total_tokens: u32,
}

#[derive(Debug, Deserialize)]
struct OpenAIResponse {
    choices: Vec<OpenAIChoice>,
    usage: Option<OpenAIUsage>,
}

pub struct OpenAIProvider {
    client: Client,
    config: ModelConfig,
    api_key: String,
}

/// Regex for extracting tool calls from raw text (Groq/Llama 3 style)
/// Enhanced to handle hallucinated '=' or '(' after the function name, and extra closing tags.
static FUNCTION_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?s)<function=([a-zA-Z0-9_-]+)[^\{]*(\{.*?\})[^>]*>?").expect("Static tool-call parser regex MUST be valid."));

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
    ) -> anyhow::Result<(String, Vec<ToolCall>, Option<TokenUsage>)> {
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
    ) -> anyhow::Result<(String, Vec<ToolCall>, Option<TokenUsage>)> {
        self.generate_with_fallback(system_prompt, user_message, tools, retry_msg, false).await
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
    ) -> anyhow::Result<(String, Vec<ToolCall>, Option<TokenUsage>)> {
        let mut url = self
            .config
            .base_url
            .clone()
            .unwrap_or_else(|| "https://api.openai.com/v1".to_string());

        // ### 🛡️ Sector Defense: Localhost Conflict Resolution
        // Resolve IPv4/IPv6 localhost conflict on Windows by forcing 127.0.0.1.
        // This prevents 500 errors when local sidecars (Ollama/Mercury) only 
        // bind to the IPv4 loopback.
        if url.contains("localhost") {
            url = url.replace("localhost", "127.0.0.1");
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
        
        let has_vectors = self.config.steering_vectors.as_ref().is_some_and(|v| !v.is_empty());
        
        if !has_vectors {
            messages.push(OpenAIMessage {
                role: "system".to_string(),
                content: Some(system_prompt.to_string()),
            });
        } else {
            tracing::info!("🧠 [OBLITERATUS] Steering vectors active. Stripping bulky system prompt.");
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
            model: self.config.model_id.to_lowercase(),
            messages,
            temperature: self.config.temperature,
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
                if p.is_empty() { None } else { Some(p) }
            },
        };

        let res = self
            .client
            .post(url)
            .header(header::AUTHORIZATION, format!("Bearer {}", self.api_key))
            .json(&request_body)
            .send()
            .await?;

        if !res.status().is_success() {
            let status = res.status();
            let error_text = res.text().await?;

            // ### 🧠 Resilience: Dynamic Quantization Fallback (OML-01)
            // Intercept OOM (Out Of Memory) from local providers (Ollama, vLLM, Mercury).
            // Logic: If the primary high-precision model (e.g. Llama-3-70b) fails 
            // due to VRAM exhaustion, we down-rank to a Q4_K_M quantization 
            // to ensure mission continuity without user intervention.
            let is_local = self.config.provider == crate::agent::types::ModelProvider::Ollama 
                || self.config.base_url.as_ref().is_some_and(|u| u.contains("127.0.0.1") || u.contains("localhost"));
            
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
                        true
                    )).await;
                }
            }

            // In-Flight JSON Correction
            // Common with Llama-based models on specific API bridges (Groq, Together, etc).
            // If the provider rejects the tool use, we manually extract the "failed_generation" 
            // field and attempt a regex-based recovery pass before bubbling up the error.
            if status == 400 && error_text.contains("tool_use_failed") {
                if let Ok(err_json) = serde_json::from_str::<serde_json::Value>(&error_text) {
                    if let Some(failed_gen) = err_json["error"]["failed_generation"].as_str() {
                        tracing::info!(
                            "🛠️ [OpenAI] Native tool failure detected. Generation: {}",
                            failed_gen
                        );
                        // 1. Attempt manual regex parsing of the failed generation
                        if let Some(caps) = FUNCTION_REGEX.captures(failed_gen) {
                            let name = caps
                                .get(1)
                                .map(|m| m.as_str().to_string())
                                .unwrap_or_default();
                            let args_str = caps.get(2).map(|m| m.as_str()).unwrap_or("");

                            let mut json_str = args_str.trim().to_string();
                            // Cleanup hallucinated chars commonly added by Llama models
                            if json_str.ends_with(')') {
                                json_str.pop();
                            }
                            if json_str.starts_with('(') {
                                json_str.remove(0);
                            }
                            let json_str = json_str.trim();

                            let mut final_json = json_str.to_string();
                            if !final_json.starts_with('{') {
                                final_json.insert(0, '{');
                            }
                            if !final_json.ends_with('}') {
                                final_json.push('}');
                            }

                            let args: serde_json::Value = serde_json::from_str(&final_json)
                                .unwrap_or_else(|e| {
                                    tracing::warn!("🛠️ [Recovery] Failed to parse natively intercepted JSON ({}): {}", e, final_json);
                                    serde_json::json!({})
                                });

                            tracing::info!("🛠️ [OpenAI] Successfully intercepted and recovered tool call '{}' natively.", name);

                            let mut recovered_text = failed_gen.to_string();
                            if let Some(mat) = caps.get(0) {
                                let raw_match = mat.as_str();
                                tracing::debug!(
                                    "🛠️ [OpenAI] Stripping recovered tool tag: {}",
                                    raw_match
                                );
                                recovered_text =
                                    recovered_text.replace(raw_match, "").trim().to_string();
                            }

                            return Ok((
                                recovered_text,
                                vec![ToolCall { name, args }],
                                None,
                            ));
                        }

                        tracing::warn!(
                            "🛠️ [OpenAI] Recovery failed. Regex did not match failed generation."
                        );
                    } else {
                        tracing::warn!("🛠️ [OpenAI] Failed to extract 'failed_generation' field from 400 response.");
                    }
                }
            }

            return Err(anyhow::anyhow!(
                "OpenAI API Error (RecoveredBody): {}",
                error_text
            ));
        }

        let parsed: OpenAIResponse = res.json().await?;

        let choice = parsed
            .choices
            .first()
            .ok_or_else(|| anyhow::anyhow!("No completion return from OpenAI"))?;

        let mut output_text = choice.message.content.clone().unwrap_or_default();

        let mut function_calls = Vec::new();
        if let Some(tool_calls) = &choice.message.tool_calls {
            for tc in tool_calls {
                let args: serde_json::Value =
                    serde_json::from_str(&tc.function.arguments).unwrap_or(serde_json::json!({}));
                function_calls.push(ToolCall {
                    name: tc.function.name.clone(),
                    args,
                });
            }
        }

        // ### 🛠️ Recovery & Cleanup: Tag-based Tool Extraction
        // Even if native tool calls were found, some models (Llama 3, Phi) might 
        // still leak raw function tags in the message content. We manually 
        // extract these and strip them to keep the UI clean.
        while let Some(caps) = FUNCTION_REGEX.captures(&output_text) {
            let name = caps
                .get(1)
                .map(|m| m.as_str().to_string())
                .unwrap_or_default();
            let args_str = caps.get(2).map(|m| m.as_str()).unwrap_or("");

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

            // Remove the raw tool call from the output text so it doesn't leak to the frontend
            let mat = caps.get(0).expect("OpenAI regex match missing internal capture group 0.");
            let raw_match = mat.as_str();
            tracing::debug!("🛠️ [OpenAI] Stripping raw tool tag: {}", raw_match);
            output_text = output_text.replace(raw_match, "").trim().to_string();
        }

        let token_usage = parsed.usage.map(|u| TokenUsage {
            input_tokens: u.prompt_tokens,
            output_tokens: u.completion_tokens,
            total_tokens: u.total_tokens,
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
    ) -> anyhow::Result<(
        String,
        Vec<ToolCall>,
        Option<TokenUsage>,
    )> {
        OpenAIProvider::generate(self, system_prompt, user_message, tools).await
    }

    async fn embed(&self, text: &str) -> anyhow::Result<Vec<f32>> {
        let mut url = self
            .config
            .base_url
            .clone()
            .unwrap_or_else(|| "https://api.openai.com/v1".to_string());
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
            return Err(anyhow::anyhow!("OpenAI Embedding Error: {}", error_text));
        }

        #[derive(Debug, Deserialize)]
        struct EmbedData {
            embedding: Vec<f32>,
        }
        #[derive(Debug, Deserialize)]
        struct EmbedResponse {
            data: Vec<EmbedData>,
        }

        let parsed: EmbedResponse = res.json().await?;
        Ok(parsed
            .data
            .first()
            .map(|d| d.embedding.clone())
            .unwrap_or_default())
    }
}

/// Detects if an error message indicates an Out-Of-Memory condition.
fn is_oom_error(text: &str) -> bool {
    let oom_keywords = ["out of memory", "insufficient vram", "not enough space", "oom"];
    let lower = text.to_lowercase();
    oom_keywords.iter().any(|k| lower.contains(k))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_openai_regex_hallucinated_equals() {
        let text = r#"<function=spawn_subagent={"agent_id": "graphic_designer", "message": "Design a simple logo and badge for Tadpole OS"}</function>"#;
        let caps = FUNCTION_REGEX.captures(text).unwrap();
        assert_eq!(caps.get(1).unwrap().as_str(), "spawn_subagent");
        assert!(caps.get(2).unwrap().as_str().contains("graphic_designer"));
    }

    #[test]
    fn test_openai_regex_double_equals() {
        let text = r#"<function=spawn_subagent=={"agent_id": "graphic_designer"}</function>"#;
        let caps = FUNCTION_REGEX.captures(text).unwrap();
        assert_eq!(caps.get(1).unwrap().as_str(), "spawn_subagent");
    }

    #[test]
    fn test_openai_regex_parentheses() {
        let text = r#"<function=share_finding({"a": 1})></function>"#;
        let caps = FUNCTION_REGEX.captures(text).unwrap();
        assert_eq!(caps.get(1).unwrap().as_str(), "share_finding");
        assert_eq!(caps.get(2).unwrap().as_str(), "{\"a\": 1}");
    }

    #[test]
    fn test_openai_regex_tadpole_alpha() {
        let text =
            r#"<function=share_finding({"topic": "Mission Plan", "finding": "..."})></function>"#;
        let caps = FUNCTION_REGEX
            .captures(text)
            .expect("Tadpole Alpha regex failed!");
        assert_eq!(caps.get(1).unwrap().as_str(), "share_finding");
    }

    #[test]
    fn test_is_oom_error_logic() {
        assert!(is_oom_error("error: insufficient vram for this model"));
        assert!(is_oom_error("{\"error\": \"out of memory\"}"));
        assert!(is_oom_error("CUDA OOM detected"));
        assert!(!is_oom_error("rate limit exceeded"));
    }

    #[test]
    fn test_steering_vector_request_serialization() {
        let mut extra = std::collections::HashMap::new();
        extra.insert("test_key".to_string(), serde_json::json!("test_val"));
        
        let request = OpenAIRequest {
            model: "llama3".to_string(),
            messages: vec![],
            temperature: None,
            user: None,
            tools: None,
            extra_parameters: Some({
                let mut map = extra;
                map.insert("steering_vectors".to_string(), serde_json::json!(vec!["vector_1"]));
                map
            }),
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("\"test_key\":\"test_val\""));
        assert!(json.contains("\"steering_vectors\":[\"vector_1\"]"));
        // Ensure flattening works (no nested 'extra_parameters' key)
        assert!(!json.contains("\"extra_parameters\""));
    }
}

// Metadata: [openai]

// Metadata: [openai]

//! OpenAI-compatible API bridge and recovery engine.
//!
//! This provider implements the `LlmProvider` trait for any OpenAI-spec endpoint.
//! It features a robust regex-based "hallucination recovery" system that can 
//! extract and repair malformed tool calls from Llama-family models.

use crate::agent::types::{GeminiFunctionCall, ModelConfig, TokenUsage};
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
static FUNCTION_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?s)<function=([a-zA-Z0-9_-]+)[^\{]*(\{.*?\})[^>]*>?")
        .unwrap()
});

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
        tools: Option<Vec<crate::agent::gemini::GeminiTool>>,
    ) -> anyhow::Result<(String, Vec<GeminiFunctionCall>, Option<TokenUsage>)> {
        self.generate_internal(system_prompt, user_message, tools, None)
            .await
    }

    /// Main generation loop for OpenAI-compatible models.
    /// 
    /// This function handles:
    /// 1. **Tool Mapping**: Translating Gemini-style tools to OpenAI's tool-call schema.
    /// 2. **Retry Logic**: Injecting error-correction instructions if previous attempts failed.
    /// 3. **Native vs Bridge**: Detecting if the model supports native tools or needs tag parsing.
    async fn generate_internal(
        &self,
        system_prompt: &str,
        user_message: &str,
        tools: Option<Vec<crate::agent::gemini::GeminiTool>>,
        retry_msg: Option<String>,
    ) -> anyhow::Result<(String, Vec<GeminiFunctionCall>, Option<TokenUsage>)> {
        let mut url = self
            .config
            .base_url
            .clone()
            .unwrap_or_else(|| "https://api.openai.com/v1".to_string());
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

        let mut messages = vec![
            OpenAIMessage {
                role: "system".to_string(),
                content: Some(system_prompt.to_string()),
            },
            OpenAIMessage {
                role: "user".to_string(),
                content: Some(user_message.to_string()),
            },
        ];

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
            } else {
                if !self.config.supports_native_tools() {
                    tracing::warn!(
                        "🛡️ [Provider] Suppressing tools for '{}' (Incompatible with bridge)",
                        self.config.model_id
                    );
                    None
                } else {
                    openai_tools
                }
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

            // Handle Tool Call hallucination errors (400 Bad Request)
            // Common with Llama-based models on specific API bridges (Groq, Together, etc)
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
                            if json_str.ends_with(')') { json_str.pop(); }
                            if json_str.starts_with('(') { json_str.remove(0); }
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
                                vec![GeminiFunctionCall { name, args }],
                                None,
                            ));
                        }

                        tracing::warn!("🛠️ [OpenAI] Recovery failed. Regex did not match failed generation.");
                    } else {
                        tracing::warn!("🛠️ [OpenAI] Failed to extract 'failed_generation' field from 400 response.");
                    }
                }
            }

            return Err(anyhow::anyhow!("OpenAI API Error (RecoveredBody): {}", error_text));
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
                function_calls.push(GeminiFunctionCall {
                    name: tc.function.name.clone(),
                    args,
                });
            }
        }

        // RECOVERY & CLEANUP: Check for manual function tags
        // Even if native tool calls were found, some models might still include the raw tags in the content.
        while let Some(caps) = FUNCTION_REGEX.captures(&output_text) {
            let name = caps
                .get(1)
                .map(|m| m.as_str().to_string())
                .unwrap_or_default();
            let args_str = caps.get(2).map(|m| m.as_str()).unwrap_or("");

            let mut json_str = args_str.trim().to_string();
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
            function_calls.push(GeminiFunctionCall { name, args });

            // Remove the raw tool call from the output text so it doesn't leak into the chat
            let mat = caps.get(0).unwrap();
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
        tools: Option<Vec<crate::agent::gemini::GeminiTool>>,
    ) -> anyhow::Result<(
        String,
        Vec<crate::agent::types::GeminiFunctionCall>,
        Option<crate::agent::types::TokenUsage>,
    )> {
        OpenAIProvider::generate(self, system_prompt, user_message, tools).await
    }

    async fn embed(&self, text: &str) -> anyhow::Result<Vec<f32>> {
        let mut url = self.config.base_url.clone()
            .unwrap_or_else(|| "https://api.openai.com/v1".to_string());
        if !url.ends_with("/embeddings") {
            if !url.ends_with('/') { url.push('/'); }
            url.push_str("embeddings");
        }

        #[derive(Debug, Serialize)]
        struct EmbedRequest { model: String, input: String }

        let request_body = EmbedRequest {
            model: self.config.model_id.clone(),
            input: text.to_string(),
        };

        let res = self.client.post(&url)
            .header(header::AUTHORIZATION, format!("Bearer {}", self.api_key))
            .json(&request_body)
            .send()
            .await?;

        if !res.status().is_success() {
            let error_text = res.text().await?;
            return Err(anyhow::anyhow!("OpenAI Embedding Error: {}", error_text));
        }

        #[derive(Debug, Deserialize)]
        struct EmbedData { embedding: Vec<f32> }
        #[derive(Debug, Deserialize)]
        struct EmbedResponse { data: Vec<EmbedData> }

        let parsed: EmbedResponse = res.json().await?;
        Ok(parsed.data.first().map(|d| d.embedding.clone()).unwrap_or_default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_openai_regex_hallucinated_equals() {
        let text = r#"<function=spawn_subagent={"agentId": "graphic_designer", "message": "Design a simple logo and badge for Tadpole OS"}</function>"#;
        let caps = FUNCTION_REGEX.captures(text).unwrap();
        assert_eq!(caps.get(1).unwrap().as_str(), "spawn_subagent");
        assert!(caps.get(2).unwrap().as_str().contains("graphic_designer"));
    }

    #[test]
    fn test_openai_regex_double_equals() {
        let text = r#"<function=spawn_subagent=={"agentId": "graphic_designer"}</function>"#;
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
        let text = r#"<function=share_finding({"topic": "Mission Plan", "finding": "..."})></function>"#;
        let caps = FUNCTION_REGEX.captures(text).expect("Tadpole Alpha regex failed!");
        assert_eq!(caps.get(1).unwrap().as_str(), "share_finding");
    }
}

//! @docs ARCHITECTURE:Agent
//!
//! ### AI Assist Note
//! **High-Speed Groq Bridge**: Implements the `LlmProvider` trait for
//! **Groq's LPUs**, specialized for **Llama-3** and **Mixtral**
//! inference. Features native **Whisper-large-v3** support for high-fidelity
//! audio transcription. Inherits the **Hallucination Recovery** engine
//! (`FUNCTION_REGEX`) to repair fragmented tool tags in ultra-fast
//! generations.
//!
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: 429 Rate Limit (RPM/TPM) due to aggressive swarm
//!   bursting, 400 Bad Request on "tool_use_failed" hallucinations, or
//!   multipart boundary errors during Whisper transcription.
//! - **Trace Scope**: `server-rs::agent::groq`

use crate::agent::types::{ModelConfig, TokenUsage, ToolCall, ToolDefinition};
use crate::error::AppError;
use once_cell::sync::Lazy;
use regex::Regex;
use reqwest::{header, Client};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
struct GroqMessage {
    role: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    content: Option<String>,
}

#[derive(Debug, Serialize)]
struct GroqTool {
    #[serde(rename = "type")]
    tool_type: String,
    function: GroqFunctionDefinition,
}

#[derive(Debug, Serialize)]
struct GroqFunctionDefinition {
    name: String,
    description: String,
    parameters: serde_json::Value,
}

#[derive(Debug, Serialize)]
struct GroqRequest {
    model: String,
    messages: Vec<GroqMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    user: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tools: Option<Vec<GroqTool>>,
}

#[derive(Debug, Deserialize)]
struct GroqChoice {
    message: GroqResponseMessage,
}

#[derive(Debug, Deserialize)]
struct GroqResponseMessage {
    content: Option<String>,
    #[serde(rename = "tool_calls")]
    tool_calls: Option<Vec<GroqToolCall>>,
}

#[derive(Debug, Deserialize)]
struct GroqToolCall {
    function: GroqFunctionCall,
}

#[derive(Debug, Deserialize)]
struct GroqFunctionCall {
    name: String,
    arguments: String,
}

#[derive(Debug, Deserialize)]
struct GroqUsage {
    prompt_tokens: u32,
    completion_tokens: u32,
    total_tokens: u32,
}

#[derive(Debug, Deserialize)]
struct GroqResponse {
    choices: Vec<GroqChoice>,
    usage: Option<GroqUsage>,
}

pub struct GroqProvider {
    client: Client,
    config: ModelConfig,
    api_key: String,
}

/// Regex for extracting tool calls from raw text (Groq/Llama 3 style)
/// Enhanced to handle hallucinated '=' or '(' after the function name, and extra closing tags.
static FUNCTION_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?s)<function=([a-zA-Z0-9_-]+)[^\{]*(\{.*?\})[^>]*>?").expect("Static Groq tool-call parser regex MUST be valid."));

impl GroqProvider {
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

    async fn generate_internal(
        &self,
        system_prompt: &str,
        user_message: &str,
        tools: Option<Vec<ToolDefinition>>,
        retry_msg: Option<String>,
    ) -> Result<(String, Vec<ToolCall>, Option<TokenUsage>), AppError> {
        let mut url = self
            .config
            .base_url
            .as_deref()
            .unwrap_or("https://api.groq.com/openai/v1/chat/completions")
            .to_string();
        if !url.ends_with("/chat/completions") {
            if !url.ends_with('/') {
                url.push('/');
            }
            url.push_str("chat/completions");
        }

        // Map Gemini tools to Groq/OpenAI tools
        let groq_tools = tools.as_ref().map(|ts| {
            ts.iter()
                .flat_map(|t| {
                    t.function_declarations.iter().map(|f| GroqTool {
                        tool_type: "function".to_string(),
                        function: GroqFunctionDefinition {
                            name: f.name.clone(),
                            description: f.description.clone(),
                            parameters: f.parameters.clone(),
                        },
                    })
                })
                .collect::<Vec<GroqTool>>()
        });

        let mut messages = vec![
            GroqMessage {
                role: "system".to_string(),
                content: Some(system_prompt.to_string()),
            },
            GroqMessage {
                role: "user".to_string(),
                content: Some(user_message.to_string()),
            },
        ];

        // If this is a retry, append the failed generation and correction instruction
        if let Some(ref r) = retry_msg {
            messages.push(GroqMessage {
                role: "assistant".to_string(),
                content: Some(r.clone()),
            });
            messages.push(GroqMessage {
                role: "user".to_string(),
                content: Some("CRITICAL ERROR: Your previous tool call was malformed. Please fix the JSON syntax and try again. Ensure all arguments are inside the brackets and there are no stray characters.".to_string()),
            });
        }

        let request_body = GroqRequest {
            model: self.config.model_id.clone(),
            messages,
            temperature: self.config.temperature,
            user: self.config.external_id.clone(),
            tools: if groq_tools.as_ref().is_none_or(|t| t.is_empty()) {
                None
            } else {
                groq_tools
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

            if status == 400 && error_text.contains("tool_use_failed") {
                if let Ok(err_json) = serde_json::from_str::<serde_json::Value>(&error_text) {
                    if let Some(failed_gen) = err_json["error"]["failed_generation"].as_str() {
                        tracing::info!(
                            "🛠️ [Groq] Native tool failure detected. Generation: {}",
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

                            tracing::info!("🛠️ [Groq] Successfully intercepted and recovered tool call '{}' natively.", name);

                            let mut recovered_text = failed_gen.to_string();
                            if let Some(mat) = caps.get(0) {
                                let raw_match = mat.as_str();
                                tracing::debug!(
                                    "🛠️ [Groq] Stripping recovered tool tag: {}",
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

                        // 2. If recovery fails, fallback to LLM self-correction
                        if retry_msg.is_none() {
                            tracing::warn!("🛠️ [Groq] Tool call failed natively. Attempting self-correction retry...");
                            let result = Box::pin(self.generate_internal(
                                system_prompt,
                                user_message,
                                tools,
                                Some(failed_gen.to_string()),
                            ))
                            .await;
                            return result;
                        }
                    }
                }
            }

            match status.as_u16() {
                429 => return Err(AppError::RateLimit(error_text)),
                400 => return Err(AppError::BadRequest(error_text)),
                401 | 403 => return Err(AppError::Unauthorized(error_text)),
                _ => return Err(AppError::InfrastructureError {
                    provider_id: "groq".to_string(),
                    detail: error_text,
                    help_link: None,
                }),
            }
        }

        let parsed: GroqResponse = res.json().await.map_err(|e| AppError::InternalServerError(e.to_string()))?;

        let choice = parsed
            .choices
            .first()
            .ok_or_else(|| AppError::InternalServerError("No completion return from Groq".to_string()))?;

        let mut output_text = choice.message.content.clone().unwrap_or_default();
        tracing::debug!("🛠️ [Groq] RAW output from LLM: {:?}", output_text);

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

        // RECOVERY & CLEANUP: Check for manual function tags (Llama 3 style)
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
                    "🛠️ [Recovery] Failed to parse recovered JSON from Groq format: {}",
                    json_str
                );
                serde_json::json!({})
            });

            tracing::info!("🛠️ [Recovery] Extracted function call from tags: {}", name);
            function_calls.push(ToolCall { name, args });

            // Remove the raw tool call from the output text
            if let Some(mat) = caps.get(0) {
                let raw_match = mat.as_str();
                output_text = output_text.replace(raw_match, "").trim().to_string();
            }
        }

        let token_usage = parsed.usage.map(|u| TokenUsage {
            input_tokens: u.prompt_tokens,
            output_tokens: u.completion_tokens,
            total_tokens: u.total_tokens,
        });

        Ok((output_text, function_calls, token_usage))
    }

    pub async fn transcribe(&self, audio_data: Vec<u8>, filename: &str) -> Result<String, AppError> {
        use reqwest::multipart;
        let url = "https://api.groq.com/openai/v1/audio/transcriptions";

        let part = multipart::Part::bytes(audio_data)
            .file_name(filename.to_string())
            .mime_str("audio/wav")
            .map_err(|e| AppError::BadRequest(e.to_string()))?;

        let form = multipart::Form::new()
            .part("file", part)
            .text("model", self.config.model_id.clone());

        let res = self
            .client
            .post(url)
            .header(header::AUTHORIZATION, format!("Bearer {}", self.api_key))
            .multipart(form)
            .send()
            .await?;

        if !res.status().is_success() {
            let error_text = res.text().await?;
            return Err(AppError::InfrastructureError {
                provider_id: "groq_whisper".to_string(),
                detail: error_text,
                help_link: None,
            });
        }

        #[derive(Deserialize)]
        struct TranscriptionResponse {
            text: String,
        }

        let parsed: TranscriptionResponse = res.json().await.map_err(|e| AppError::InternalServerError(e.to_string()))?;
        Ok(parsed.text)
    }
}

#[async_trait::async_trait]
impl crate::agent::provider_trait::LlmProvider for GroqProvider {
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
        GroqProvider::generate(self, system_prompt, user_message, tools).await
    }

    async fn embed(&self, _text: &str) -> Result<Vec<f32>, AppError> {
        Err(AppError::NotImplemented("Groq does not natively support embeddings".to_string()))
    }
}

// Metadata: [groq]

// Metadata: [groq]

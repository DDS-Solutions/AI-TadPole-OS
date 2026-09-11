//! @docs ARCHITECTURE:Agent
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Agent Runner / anthropic
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Type-safe state handling and bounded execution without unhandled panics.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: none declared
//! - **Witness Tests**: none declared

use crate::agent::provider_trait::LlmProvider;
use crate::agent::types::{ModelConfig, TokenUsage, ToolCall, ToolDefinition};
use crate::error::{AppError, InfrastructureErrorKind, ProviderId};
use async_trait::async_trait;
use reqwest::{header, Client};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
struct AnthropicMessage {
    role: String,
    content: Vec<AnthropicContentBlock>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum AnthropicContentBlock {
    Text {
        text: String,
    },
    ToolUse {
        id: String,
        name: String,
        input: serde_json::Value,
    },
}

#[derive(Debug, Serialize)]
struct AnthropicTool {
    name: String,
    description: String,
    input_schema: serde_json::Value,
}

#[derive(Debug, Serialize)]
struct AnthropicRequest {
    model: String,
    max_tokens: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    system: Option<String>,
    messages: Vec<AnthropicMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tools: Option<Vec<AnthropicTool>>,
}

#[derive(Debug, Deserialize)]
struct AnthropicResponseContentBlock {
    #[serde(rename = "type")]
    block_type: String,
    #[serde(default)]
    text: Option<String>,
    #[serde(default)]
    _id: Option<String>,
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    input: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize, Default)]
struct AnthropicUsage {
    #[serde(default)]
    input_tokens: Option<u64>,
    #[serde(default)]
    output_tokens: Option<u64>,
}

#[derive(Debug, Deserialize, Default)]
struct AnthropicResponse {
    #[serde(default)]
    content: Vec<AnthropicResponseContentBlock>,
    #[serde(default)]
    usage: Option<AnthropicUsage>,
}

pub struct AnthropicProvider {
    client: Client,
    config: ModelConfig,
    api_key: String,
}

impl AnthropicProvider {
    pub fn new(client: Client, api_key: String, config: ModelConfig) -> Self {
        Self {
            client,
            config,
            api_key,
        }
    }
}

#[async_trait]
impl LlmProvider for AnthropicProvider {
    async fn generate(
        &self,
        system_prompt: &str,
        user_message: &str,
        tools: Option<Vec<ToolDefinition>>,
    ) -> Result<(String, Vec<ToolCall>, Option<TokenUsage>), AppError> {
        let url = self
            .config
            .base_url
            .as_deref()
            .unwrap_or("https://api.anthropic.com/v1/messages");

        // Map generic tools to Anthropic input_schema
        let anthropic_tools = tools.as_ref().map(|ts| {
            ts.iter()
                .flat_map(|t| {
                    t.function_declarations.iter().map(|f| AnthropicTool {
                        name: f.name.clone(),
                        description: f.description.clone(),
                        input_schema: f.parameters.clone(),
                    })
                })
                .collect::<Vec<AnthropicTool>>()
        });

        let messages = vec![AnthropicMessage {
            role: "user".to_string(),
            content: vec![AnthropicContentBlock::Text {
                text: user_message.to_string(),
            }],
        }];

        let request_body = AnthropicRequest {
            model: self.config.model_id.clone(),
            max_tokens: self.config.max_tokens.unwrap_or(4096),
            system: Some(system_prompt.to_string()),
            messages,
            temperature: self.config.temperature,
            tools: if anthropic_tools.as_ref().is_none_or(|t| t.is_empty()) {
                None
            } else {
                anthropic_tools
            },
        };

        let res = self
            .client
            .post(url)
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header(header::CONTENT_TYPE, "application/json")
            .json(&request_body)
            .send()
            .await?;

        if !res.status().is_success() {
            let status = res.status();
            let error_text = res.text().await?;

            match status.as_u16() {
                429 => return Err(AppError::RateLimit(error_text)),
                400 => return Err(AppError::BadRequest(error_text)),
                401 | 403 => return Err(AppError::Unauthorized(error_text)),
                _ => {
                    return Err(AppError::InfrastructureError {
                        provider_id: ProviderId::Anthropic,
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
                provider_id: ProviderId::Anthropic,
                kind: InfrastructureErrorKind::NetworkError,
                detail: format!("Failed to read Anthropic response body stream: {}", e),
                help_link: None,
            })?;

        let parsed: AnthropicResponse = match serde_json::from_str(&raw_text) {
            Ok(p) => p,
            Err(err) => {
                tracing::error!(
                    "🚨 [Anthropic] Failed to decode response JSON ({}). Raw response: {}",
                    err,
                    crate::error::safe_truncate_bytes(&raw_text, 1024)
                );

                if let Ok(err_val) = serde_json::from_str::<serde_json::Value>(&raw_text) {
                    if let Some(err_obj) = err_val.get("error") {
                        let msg = err_obj
                            .get("message")
                            .and_then(|m| m.as_str())
                            .or_else(|| err_obj.as_str())
                            .unwrap_or("Unknown upstream API error in 200 OK body");
                        return Err(AppError::InfrastructureError {
                            provider_id: ProviderId::Anthropic,
                            kind: InfrastructureErrorKind::ApiError,
                            detail: format!("Upstream API error: {}", msg),
                            help_link: None,
                        });
                    }
                }

                return Err(AppError::InfrastructureError {
                    provider_id: ProviderId::Anthropic,
                    kind: InfrastructureErrorKind::ApiError,
                    detail: format!(
                        "Error decoding Anthropic response body ({}): {}",
                        err,
                        crate::error::safe_truncate_bytes(&raw_text, 500)
                    ),
                    help_link: None,
                });
            }
        };

        let mut output_text = String::new();
        let mut function_calls = Vec::new();

        for block in parsed.content {
            match block.block_type.as_str() {
                "text" => {
                    if let Some(t) = block.text {
                        output_text.push_str(&t);
                    }
                }
                "tool_use" => {
                    if let (Some(name), Some(args)) = (block.name, block.input) {
                        function_calls.push(ToolCall { name, args });
                    }
                }
                _ => {}
            }
        }

        let token_usage = parsed.usage.map(|u| {
            let input = u.input_tokens.unwrap_or(0);
            let output = u.output_tokens.unwrap_or(0);
            TokenUsage {
                input_tokens: input,
                output_tokens: output,
                total_tokens: input + output,
            }
        });

        Ok((output_text, function_calls, token_usage))
    }

    async fn embed(&self, _text: &str) -> Result<Vec<f32>, AppError> {
        Err(AppError::NotImplemented(
            "Anthropic does not support native embeddings at this time.".to_string(),
        ))
    }
}

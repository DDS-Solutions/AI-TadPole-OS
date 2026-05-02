//! @docs ARCHITECTURE:Agent
//!
//! ### AI Assist Note
//! **Anthropic Claude 3.5 Bridge**: Implements the `LlmProvider` trait for
//! Anthropic's Messages API. Optimized for **Complex Tool Orchestration**:
//! maps the engine's generic `ToolCall` protocol to Anthropic's `tool_use`
//! blocks. Features **Structured Context Padding**: automatically separates
//! the system prompt into the dedicated top-level `system` field required
//! by Claude 3.5 Sonnet (LLM-03).
//!
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: API version mismatches (requires `anthropic-version` header),
//!   400 Bad Request on `input_schema` validation, or tool call duplication.
//! - **Trace Scope**: `server-rs::agent::anthropic`

use crate::agent::types::{TokenUsage, ToolCall, ToolDefinition, ModelConfig};
use crate::agent::provider_trait::LlmProvider;
use crate::error::AppError;
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
    Text { text: String },
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

#[derive(Debug, Deserialize)]
struct AnthropicUsage {
    input_tokens: u32,
    output_tokens: u32,
}

#[derive(Debug, Deserialize)]
struct AnthropicResponse {
    content: Vec<AnthropicResponseContentBlock>,
    usage: AnthropicUsage,
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
        let url = self.config.base_url.as_deref().unwrap_or("https://api.anthropic.com/v1/messages");

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
            tools: if anthropic_tools.as_ref().is_none_or(|t| t.is_empty()) { None } else { anthropic_tools },
        };

        let res = self.client.post(url)
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
                _ => return Err(AppError::InfrastructureError {
                    provider_id: "anthropic".to_string(),
                    detail: error_text,
                    help_link: None,
                }),
            }
        }

        let parsed: AnthropicResponse = res.json().await.map_err(|e| AppError::InternalServerError(e.to_string()))?;

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

        let token_usage = Some(TokenUsage {
            input_tokens: parsed.usage.input_tokens,
            output_tokens: parsed.usage.output_tokens,
            total_tokens: parsed.usage.input_tokens + parsed.usage.output_tokens,
        });

        Ok((output_text, function_calls, token_usage))
    }

    async fn embed(&self, _text: &str) -> Result<Vec<f32>, AppError> {
        Err(AppError::NotImplemented("Anthropic does not support native embeddings at this time.".to_string()))
    }
}

// Metadata: [anthropic]

// Metadata: [anthropic]

//! @docs ARCHITECTURE:Registry
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Agent Runner / config
//! - **Primary Entrypoints**: `ModelConfig`, `ModelEntry`, `Validatable`
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: none declared

use super::capabilities::{lacks_native_tool_bridge, ModelCapabilities};
use super::provider::{Modality, ModelProvider};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub trait Validatable {
    fn validate(&self) -> Result<(), String>;
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, specta::Type)]
pub struct ConnectorConfig {
    pub r#type: String,
    pub uri: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct ModelConfig {
    pub provider: ModelProvider,
    #[serde(default, alias = "modelId")]
    pub model_id: String,
    #[serde(default, alias = "apiKey")]
    pub api_key: Option<String>,
    #[serde(default, alias = "baseUrl")]
    pub base_url: Option<String>,
    #[serde(default, alias = "systemPrompt")]
    pub system_prompt: Option<String>,
    pub temperature: Option<f32>,
    #[serde(default, alias = "maxTokens")]
    pub max_tokens: Option<u32>,
    #[serde(default, alias = "externalId")]
    pub external_id: Option<String>,
    #[serde(default)]
    pub rpm: Option<u32>,
    #[serde(default)]
    pub rpd: Option<u32>,
    #[serde(default)]
    pub tpm: Option<u32>,
    #[serde(default)]
    pub tpd: Option<u32>,
    #[serde(default, alias = "skills")]
    pub skills: Option<Vec<String>>,
    #[serde(default)]
    pub workflows: Option<Vec<String>>,
    #[serde(default, alias = "mcpTools")]
    pub mcp_tools: Option<Vec<String>>,
    #[serde(default, alias = "steeringVectors")]
    pub steering_vectors: Option<Vec<String>>,
    #[serde(default, alias = "reasoningDepth")]
    pub reasoning_depth: Option<u32>,
    #[serde(default, alias = "actThreshold")]
    pub act_threshold: Option<f32>,
    #[serde(default, alias = "maxTurns")]
    pub max_turns: Option<u32>,
    #[serde(default, alias = "connectorConfigs")]
    pub connector_configs: Option<Vec<ConnectorConfig>>,
    #[serde(default, alias = "extraParameters")]
    pub extra_parameters: Option<HashMap<String, serde_json::Value>>,
    #[serde(default, alias = "customHeaders")]
    pub custom_headers: Option<HashMap<String, String>>,
    #[serde(default, alias = "contextWindow")]
    pub context_window: Option<u32>,
    #[serde(default, alias = "inputCostPerM")]
    pub input_cost_per_m: Option<f64>,
    #[serde(default, alias = "outputCostPerM")]
    pub output_cost_per_m: Option<f64>,
}

impl ModelConfig {
    #[allow(dead_code)]
    pub fn is_openai_compatible(&self) -> bool {
        match self.provider {
            ModelProvider::Openai
            | ModelProvider::Groq
            | ModelProvider::Perplexity
            | ModelProvider::Together
            | ModelProvider::Fireworks
            | ModelProvider::Deepseek
            | ModelProvider::Xai
            | ModelProvider::Mistral
            | ModelProvider::Openrouter
            | ModelProvider::Cerebras
            | ModelProvider::Sambanova
            | ModelProvider::Inception => true,
            _ => {
                let model_lower = self.model_id.to_lowercase();
                let base_url_is_openai = self
                    .base_url
                    .as_ref()
                    .map(|url| url.contains("api.openai.com") || url.contains("/v1"))
                    .unwrap_or(false);

                base_url_is_openai
                    || model_lower.contains("gpt")
                    || model_lower.contains("o1-")
                    || model_lower.contains("o3-")
            }
        }
    }

    /// Returns false for model families that cannot receive a native `tools` array.
    /// Delegates to the shared exclusion list in `capabilities.rs`.
    pub fn supports_native_tools(&self) -> bool {
        !lacks_native_tool_bridge(&self.model_id)
    }

    /// Returns a copy of `self` with every unset `Option` field filled from `other`.
    /// Values already present on `self` always win; `extra_parameters` are merged per key.
    ///
    /// **Credential binding**: `api_key` is inherited only when the merged `base_url`
    /// equals `other.base_url`. A key is never paired with an endpoint it was not
    /// configured for.
    pub fn merge(&self, other: &Self) -> Self {
        let mut merged = self.clone();

        macro_rules! merge_option {
            ($($field:ident),+ $(,)?) => {
                $(
                    if merged.$field.is_none() {
                        merged.$field = other.$field.clone();
                    }
                )+
            };
        }

        merge_option!(
            base_url,
            system_prompt,
            temperature,
            max_tokens,
            external_id,
            rpm,
            rpd,
            tpm,
            tpd,
            skills,
            workflows,
            mcp_tools,
            steering_vectors,
            reasoning_depth,
            act_threshold,
            max_turns,
            connector_configs,
            custom_headers,
            context_window,
            input_cost_per_m,
            output_cost_per_m,
        );

        if merged.api_key.is_none() && merged.base_url == other.base_url {
            merged.api_key = other.api_key.clone();
        }

        if let Some(other_extras) = &other.extra_parameters {
            let mut extras = merged.extra_parameters.unwrap_or_default();
            for (k, v) in other_extras {
                extras.entry(k.clone()).or_insert_with(|| v.clone());
            }
            merged.extra_parameters = Some(extras);
        }

        merged
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, specta::Type)]
pub struct ModelEntry {
    pub id: String,
    pub name: String,
    pub provider_id: String,
    #[serde(default)]
    pub provider: Option<ModelProvider>,
    #[serde(default)]
    pub rpm: Option<u32>,
    #[serde(default)]
    pub tpm: Option<u32>,
    #[serde(default)]
    pub rpd: Option<u32>,
    #[serde(default)]
    pub tpd: Option<u32>,
    #[serde(default)]
    pub modality: Modality,
    #[serde(default)]
    pub capabilities: ModelCapabilities,
    #[serde(default)]
    pub provider_model_id: Option<String>,
    #[serde(default, alias = "contextWindow")]
    pub context_window: Option<u32>,
    #[serde(default, alias = "inputCostPerM")]
    pub input_cost_per_m: Option<f64>,
    #[serde(default, alias = "outputCostPerM")]
    pub output_cost_per_m: Option<f64>,
}

impl Validatable for ModelEntry {
    fn validate(&self) -> Result<(), String> {
        if self.name.trim().is_empty() {
            return Err("Model name cannot be empty".to_string());
        }
        if self.provider_id.trim().is_empty() {
            return Err("Model must be assigned to a Provider ID".to_string());
        }
        Ok(())
    }
}

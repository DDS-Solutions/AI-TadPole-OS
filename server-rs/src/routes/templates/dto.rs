//! @docs ARCHITECTURE:Networking
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / HTTP Routes / templates / dto
//! - **Primary Entrypoints**: `TemplateCatalogEntry`, `InstallTemplateRequest`, `ImportTemplateRequest`, `InstalledSwarmSummary`, `InstallationReceipt`
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Serialized wire representations align strictly with frontend TypeScript contracts.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none declared
//! - **Telemetry Targets**: none declared
//! - **Witness Tests**: `routes::templates::dto::tests::*`

use crate::error::AppError;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct TemplateCatalogEntry {
    pub id: String,
    pub name: String,
    pub description: String,
    pub repository_url: String,
    pub path: String,
    #[serde(default)]
    pub required_models: Vec<String>,
    #[serde(default)]
    pub required_skills: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ModelOverrideConfig {
    pub provider: String,
    #[serde(alias = "model_id", alias = "modelId")]
    pub model_id: String,
    #[serde(default, alias = "base_url", alias = "baseUrl")]
    pub base_url: Option<String>,
}

pub fn parse_model_provider(s: &str) -> Result<crate::agent::types::ModelProvider, AppError> {
    match s.trim().to_lowercase().as_str() {
        "openai" => Ok(crate::agent::types::ModelProvider::Openai),
        "anthropic" => Ok(crate::agent::types::ModelProvider::Anthropic),
        "google" | "gemini" => Ok(crate::agent::types::ModelProvider::Google),
        "ollama" => Ok(crate::agent::types::ModelProvider::Ollama),
        "groq" => Ok(crate::agent::types::ModelProvider::Groq),
        "mistral" => Ok(crate::agent::types::ModelProvider::Mistral),
        "perplexity" => Ok(crate::agent::types::ModelProvider::Perplexity),
        "fireworks" => Ok(crate::agent::types::ModelProvider::Fireworks),
        "together" => Ok(crate::agent::types::ModelProvider::Together),
        "deepseek" => Ok(crate::agent::types::ModelProvider::Deepseek),
        "xai" => Ok(crate::agent::types::ModelProvider::Xai),
        "inception" => Ok(crate::agent::types::ModelProvider::Inception),
        "openrouter" => Ok(crate::agent::types::ModelProvider::Openrouter),
        "cerebras" => Ok(crate::agent::types::ModelProvider::Cerebras),
        other => Err(AppError::BadRequest(format!(
            "Unknown or unsupported model provider '{}'",
            other
        ))),
    }
}

pub fn apply_model_override(
    agent: &mut crate::agent::types::EngineAgent,
    override_cfg: &ModelOverrideConfig,
) -> Result<(), AppError> {
    let provider = parse_model_provider(&override_cfg.provider)?;
    agent.models.model.provider = provider;
    agent.models.model.model_id = override_cfg.model_id.clone();
    agent.models.model_id = Some(override_cfg.model_id.clone());
    if let Some(ref base_url) = override_cfg.base_url {
        agent.models.model.base_url = Some(base_url.clone());
    }
    Ok(())
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InstallTemplateRequest {
    pub repository_url: String,
    pub path: String,
    #[serde(default)]
    pub git_ref: Option<String>,
    #[serde(default, alias = "model_override", alias = "modelOverride")]
    pub model_override: Option<ModelOverrideConfig>,
    #[serde(default)]
    pub overwrite: bool,
    #[serde(default)]
    pub namespace: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ImportTemplateRequest {
    pub swarm: serde_json::Value,
    #[serde(default)]
    pub agents: Vec<RawAgentAsset>,
    #[serde(default)]
    pub workflows: Vec<RawWorkflowAsset>,
    #[serde(default)]
    pub mcps: Option<serde_json::Value>,
    #[serde(default)]
    pub overwrite: bool,
    #[serde(default, alias = "model_override", alias = "modelOverride")]
    pub model_override: Option<ModelOverrideConfig>,
    #[serde(default)]
    pub namespace: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RawAgentAsset {
    pub filename: String,
    pub content: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RawWorkflowAsset {
    pub filename: String,
    pub content: String,
}

#[derive(Debug, Serialize)]
pub struct InstallTemplateResponse {
    pub status: String,
    pub message: String,
    pub receipt: InstallationReceipt,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct InstallationReceipt {
    pub template_path: String,
    pub source_revision: Option<String>,
    pub agents: InstallAssetReceipt,
    pub workflows: InstallAssetReceipt,
    pub skills: InstallAssetReceipt,
    pub swarm_manifest: InstallAssetReceipt,
    pub mcp_servers: InstallAssetReceipt,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct InstallAssetReceipt {
    pub status: String,
    pub available: usize,
    pub installed: usize,
    pub replaced: usize,
}

impl InstallAssetReceipt {
    pub fn complete(count: usize) -> Self {
        Self::new(count, count, 0)
    }

    pub fn new(available: usize, installed: usize, replaced: usize) -> Self {
        let status = if installed == 0 {
            "not_present".to_string()
        } else if replaced > 0 {
            "replaced".to_string()
        } else {
            "installed".to_string()
        };

        Self {
            status,
            available,
            installed,
            replaced,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct InstalledSwarmSummary {
    pub id: String,
    pub name: String,
    pub description: String,
    #[serde(default)]
    pub industry: Option<String>,
    #[serde(default)]
    pub installed_at: Option<String>,
    pub template_path: String,
    pub agents: Vec<String>,
    pub workflows: Vec<String>,
    #[serde(default)]
    pub skills: Vec<String>,
    pub mcp_servers: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct InstalledTemplatesResponse {
    pub swarms: Vec<InstalledSwarmSummary>,
}

#[derive(Debug, Deserialize)]
pub struct UninstallTemplateRequest {
    pub swarm_id: String,
    #[serde(default = "default_archive_true")]
    pub archive: bool,
}

fn default_archive_true() -> bool {
    true
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct UninstallTemplateResponse {
    pub status: String,
    pub message: String,
    pub uninstalled_agents: Vec<String>,
    pub uninstalled_workflows: Vec<String>,
    pub uninstalled_skills: Vec<String>,
    pub uninstalled_mcp_servers: Vec<String>,
    #[serde(default)]
    pub archived_path: Option<String>,
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    #[test]
    fn test_parse_model_provider() {
        assert_eq!(parse_model_provider("openai").unwrap(), crate::agent::types::ModelProvider::Openai);
        assert_eq!(parse_model_provider("openrouter").unwrap(), crate::agent::types::ModelProvider::Openrouter);
        assert_eq!(parse_model_provider("ollama").unwrap(), crate::agent::types::ModelProvider::Ollama);
        assert_eq!(parse_model_provider("google").unwrap(), crate::agent::types::ModelProvider::Google);
        assert_eq!(parse_model_provider("gemini").unwrap(), crate::agent::types::ModelProvider::Google);
        assert!(parse_model_provider("unsupported_backend").is_err());
    }

    #[test]
    fn test_install_template_request_deserialization() {
        let json_data = r#"{"repository_url": "https://github.com/example/repo.git", "path": "templates/marketing", "git_ref": "v1.0.0", "model_override": {"provider": "openrouter", "model_id": "stealth/ox-alpha"}, "overwrite": true, "namespace": "mkt"}"#;
        let req: InstallTemplateRequest = serde_json::from_str(json_data).unwrap();
        assert_eq!(req.repository_url, "https://github.com/example/repo.git");
        assert_eq!(req.path, "templates/marketing");
        assert_eq!(req.git_ref, Some("v1.0.0".to_string()));
        assert!(req.overwrite);
        assert_eq!(req.namespace, Some("mkt".to_string()));
        assert_eq!(
            req.model_override,
            Some(ModelOverrideConfig {
                provider: "openrouter".to_string(),
                model_id: "stealth/ox-alpha".to_string(),
                base_url: None,
            })
        );
    }

    #[test]
    fn test_import_template_request_deserialization() {
        let json_data = r#"{"swarm": {"name": "Test Swarm"}, "overwrite": true, "namespace": "test_ns", "modelOverride": {"provider": "ollama", "modelId": "gemma4:e4b", "baseUrl": "http://127.0.0.1:11434"}}"#;
        let req: ImportTemplateRequest = serde_json::from_str(json_data).unwrap();
        assert!(req.overwrite);
        assert_eq!(req.namespace, Some("test_ns".to_string()));
        assert_eq!(
            req.model_override,
            Some(ModelOverrideConfig {
                provider: "ollama".to_string(),
                model_id: "gemma4:e4b".to_string(),
                base_url: Some("http://127.0.0.1:11434".to_string()),
            })
        );
    }

    #[test]
    fn test_uninstall_template_request_defaults_archive_to_true() {
        let json_data = r#"{"swarm_id": "marketing"}"#;
        let req: UninstallTemplateRequest = serde_json::from_str(json_data).unwrap();
        assert_eq!(req.swarm_id, "marketing");
        assert!(req.archive);
    }

    #[test]
    fn receipt_distinguishes_installed_and_absent_assets() {
        assert_eq!(InstallAssetReceipt::complete(2).status, "installed");
        assert_eq!(InstallAssetReceipt::complete(2).installed, 2);
        assert_eq!(InstallAssetReceipt::complete(0).status, "not_present");
        assert_eq!(InstallAssetReceipt::complete(0).installed, 0);

        let replaced_receipt = InstallAssetReceipt::new(3, 3, 2);
        assert_eq!(replaced_receipt.status, "replaced");
        assert_eq!(replaced_receipt.replaced, 2);
    }

    #[test]
    fn test_installed_swarm_summary_wire_roundtrip() {
        let summary = InstalledSwarmSummary {
            id: "mkt_test".to_string(),
            name: "Marketing Swarm".to_string(),
            description: "Lead generation cluster".to_string(),
            industry: Some("Marketing".to_string()),
            installed_at: Some("2026-08-25T12:00:00Z".to_string()),
            template_path: "templates/marketing".to_string(),
            agents: vec!["lead_gen".to_string()],
            workflows: vec!["daily_sync.md".to_string()],
            skills: vec!["search.py".to_string()],
            mcp_servers: vec!["hubspot".to_string()],
        };

        let json_str = serde_json::to_string(&summary).unwrap();
        assert!(json_str.contains("\"installed_at\":\"2026-08-25T12:00:00Z\""));
        assert!(json_str.contains("\"skills\":[\"search.py\"]"));

        let deserialized: InstalledSwarmSummary = serde_json::from_str(&json_str).unwrap();
        assert_eq!(summary, deserialized);
    }
}

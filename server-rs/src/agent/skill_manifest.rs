//! @docs ARCHITECTURE:Agent
//!
//! ### AI Assist Note
//! **Security Gate**: Registry of available tool schemas and documentation.
//! Enforces **Hard-Coded Safety Boundaries** by automatically forcing
//! `requires_oversight = true` if `Permission::ShellExecute` or
//! `Permission::BudgetSpend` are requested. Validates **Schema Parity**
//! across disparate skill implementations.
//!
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: Unsupported `schema_version`, missing `skill.json` in
//!   a discovered directory, or validation failure for critical permissions.
//! - **Trace Scope**: `server-rs::agent::skill_manifest`
//!

use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use crate::error::AppError;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, specta::Type)]
#[serde(rename_all = "snake_case")]
pub enum DangerLevel {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, specta::Type)]
#[serde(rename_all = "kebab-case")]
pub enum Permission {
    #[serde(rename = "network:outbound")]
    NetworkOutbound,
    #[serde(rename = "filesystem:read")]
    FilesystemRead,
    #[serde(rename = "filesystem:write")]
    FilesystemWrite,
    #[serde(rename = "shell:execute")]
    ShellExecute,
    #[serde(rename = "budget:spend")]
    BudgetSpend,
    #[serde(untagged)]
    Unknown(String), // Fallback for forward compat
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct SkillParameter {
    pub r#type: String,
    pub required: Option<bool>,
    pub default: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct SkillHooks {
    pub before_execute: Option<String>,
    pub after_execute: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct SkillManifest {
    pub schema_version: String,
    pub name: String,
    pub display_name: Option<String>,
    pub description: String,
    pub version: String,
    pub author: Option<String>,

    #[serde(default)]
    pub permissions: Vec<Permission>,

    pub toolset_group: Option<String>,
    pub danger_level: DangerLevel,

    #[serde(default)]
    pub requires_oversight: bool,

    #[serde(default)]
    pub parameters: HashMap<String, SkillParameter>,

    pub hooks: Option<SkillHooks>,
    #[serde(default = "default_category")]
    pub category: String,
}

fn default_category() -> String {
    "user".to_string()
}

impl Default for SkillManifest {
    fn default() -> Self {
        Self {
            schema_version: "1".to_string(),
            name: "unknown".to_string(),
            display_name: None,
            description: "".to_string(),
            version: "1.1.353".to_string(),
            author: None,
            permissions: vec![],
            toolset_group: None,
            danger_level: DangerLevel::Low,
            requires_oversight: false,
            parameters: HashMap::new(),
            hooks: None,
            category: default_category(),
        }
    }
}

impl SkillManifest {
    /// Validates the manifest and enforces hard-coded security gates.
    ///
    /// ### 🛡️ Security Mapping
    /// If a skill requests `ShellExecute` or `BudgetSpend` permissions,
    /// this function automatically sets `requires_oversight = true` regardless
    /// of what the manifest JSON specified. This is a non-bypassable guard.
    pub fn validate(&mut self) -> Result<(), AppError> {
        if self.schema_version != "1" {
            return Err(AppError::BadRequest(format!(
                "Unsupported schema_version: {}",
                self.schema_version
            )));
        }

        // Security Gate: auto-set requires_oversight if demanding dangerous permissions
        for perm in &self.permissions {
            match perm {
                Permission::ShellExecute | Permission::BudgetSpend => {
                    self.requires_oversight = true;
                }
                Permission::Unknown(p) => {
                    tracing::warn!("Skill {} requested unknown permission: {}", self.name, p);
                }
                _ => {}
            }
        }

        Ok(())
    }

    /// Validates runtime arguments against the skill's defined parameters.
    pub fn validate_arguments(&self, args: &serde_json::Value) -> Result<(), AppError> {
        let empty_map = serde_json::Map::new();
        let args_obj = args.as_object().unwrap_or(&empty_map);

        for (param_name, param_def) in &self.parameters {
            let is_required = param_def.required.unwrap_or(false);
            let value_opt = args_obj.get(param_name);

            match value_opt {
                None | Some(serde_json::Value::Null) => {
                    if is_required {
                        return Err(AppError::BadRequest(format!(
                            "Missing required parameter: '{}' for skill '{}'",
                            param_name, self.name
                        )));
                    }
                }
                Some(val) => {
                    // Validate type
                    let expected_type = param_def.r#type.to_lowercase();
                    match expected_type.as_str() {
                        "string" => {
                            if !val.is_string() {
                                return Err(AppError::BadRequest(format!(
                                    "Parameter '{}' for skill '{}' expects type 'string', got: {}",
                                    param_name, self.name, val
                                )));
                            }
                        }
                        "integer" | "number" => {
                            if !val.is_number() {
                                return Err(AppError::BadRequest(format!(
                                    "Parameter '{}' for skill '{}' expects type '{}', got: {}",
                                    param_name, self.name, expected_type, val
                                )));
                            }
                        }
                        "boolean" => {
                            if !val.is_boolean() {
                                return Err(AppError::BadRequest(format!(
                                    "Parameter '{}' for skill '{}' expects type 'boolean', got: {}",
                                    param_name, self.name, val
                                )));
                            }
                        }
                        "array" => {
                            if !val.is_array() {
                                return Err(AppError::BadRequest(format!(
                                    "Parameter '{}' for skill '{}' expects type 'array', got: {}",
                                    param_name, self.name, val
                                )));
                            }
                        }
                        "object" => {
                            if !val.is_object() {
                                return Err(AppError::BadRequest(format!(
                                    "Parameter '{}' for skill '{}' expects type 'object', got: {}",
                                    param_name, self.name, val
                                )));
                            }
                        }
                        _ => {
                            // If type is unknown, log a warning but allow for extensibility
                            tracing::warn!(
                                "Skill '{}' parameter '{}' has unknown schema type: '{}'",
                                self.name,
                                param_name,
                                expected_type
                            );
                        }
                    }
                }
            }
        }
        Ok(())
    }
}

pub struct SkillRegistry {
    pub manifests: DashMap<String, SkillManifest>,
}

impl SkillRegistry {
    pub fn new() -> Self {
        Self {
            manifests: DashMap::new(),
        }
    }

    pub fn get(&self, name: &str) -> Option<SkillManifest> {
        self.manifests.get(name).map(|m| m.value().clone())
    }

    pub fn insert(&self, manifest: SkillManifest) {
        self.manifests.insert(manifest.name.clone(), manifest);
    }

    /// Discovery Engine: Traverses the `data/skills` directory and
    /// hydro-loads all valid `skill.json` manifests into the registry.
    pub fn load_all() -> Self {
        let registry = Self::new();

        let mut data_dir = PathBuf::from("data");
        data_dir.push("skills");

        if !data_dir.exists() {
            if let Err(err) = fs::create_dir_all(&data_dir) {
                tracing::error!("Failed to create skills directory: {}", err);
                return registry;
            }
            tracing::info!("Created missing skills directory at {:?}", data_dir);
        }

        let entries = match fs::read_dir(&data_dir) {
            Ok(e) => e,
            Err(err) => {
                tracing::error!("Failed to read skills directory: {}", err);
                return registry;
            }
        };

        for entry in entries.filter_map(Result::ok) {
            let path = entry.path();
            if path.is_dir() {
                let manifest_path = path.join("skill.json");
                if manifest_path.exists() {
                    match Self::load_manifest(&manifest_path) {
                        Ok(mut manifest) => {
                            if let Err(e) = manifest.validate() {
                                tracing::error!(
                                    "Failed to validate manifest {:?}: {}",
                                    manifest_path,
                                    e
                                );
                                continue;
                            }
                            registry.insert(manifest);
                        }
                        Err(e) => {
                            tracing::error!("Failed to load manifest {:?}: {}", manifest_path, e)
                        }
                    }
                }
            }
        }

        registry
    }

    fn load_manifest(path: &PathBuf) -> Result<SkillManifest, AppError> {
        let file_contents = fs::read_to_string(path).map_err(AppError::Io)?;
        let manifest: SkillManifest = serde_json::from_str(&file_contents)
            .map_err(|e| AppError::BadRequest(e.to_string()))?;
        Ok(manifest)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_manifest_security_gate() {
        let mut manifest = SkillManifest {
            permissions: vec![Permission::ShellExecute],
            requires_oversight: false,
            ..Default::default()
        };

        manifest.validate().unwrap();

        assert!(manifest.requires_oversight);
    }

    #[test]
    fn test_manifest_budget_gate() {
        let mut manifest = SkillManifest {
            permissions: vec![Permission::BudgetSpend],
            requires_oversight: false,
            ..Default::default()
        };

        manifest.validate().unwrap();

        assert!(manifest.requires_oversight);
    }

    #[test]
    fn test_manifest_schema_validation() {
        let mut manifest = SkillManifest {
            schema_version: "2".to_string(), // Invalid schema
            ..Default::default()
        };

        let result = manifest.validate();
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Bad Request: Unsupported schema_version: 2"
        );
    }

    #[test]
    fn test_validate_arguments() {
        let mut parameters = HashMap::new();
        parameters.insert(
            "url".to_string(),
            SkillParameter {
                r#type: "string".to_string(),
                required: Some(true),
                default: None,
            },
        );
        parameters.insert(
            "timeout".to_string(),
            SkillParameter {
                r#type: "integer".to_string(),
                required: Some(false),
                default: Some(serde_json::json!(30)),
            },
        );
        parameters.insert(
            "enabled".to_string(),
            SkillParameter {
                r#type: "boolean".to_string(),
                required: Some(false),
                default: None,
            },
        );

        let manifest = SkillManifest {
            name: "test_skill".to_string(),
            parameters,
            ..Default::default()
        };

        // 1. Success case: URL string is provided, timeout integer is provided
        let valid_args = serde_json::json!({
            "url": "https://google.com",
            "timeout": 15
        });
        assert!(manifest.validate_arguments(&valid_args).is_ok());

        // 2. Missing required URL parameter
        let missing_required = serde_json::json!({
            "timeout": 15
        });
        let res_missing = manifest.validate_arguments(&missing_required);
        assert!(res_missing.is_err());
        assert!(res_missing
            .unwrap_err()
            .to_string()
            .contains("Missing required parameter"));

        // 3. Incorrect type for url parameter (expects string, gets number)
        let invalid_type = serde_json::json!({
            "url": 123
        });
        let res_type = manifest.validate_arguments(&invalid_type);
        assert!(res_type.is_err());
        assert!(res_type
            .unwrap_err()
            .to_string()
            .contains("expects type 'string'"));
    }
}

// Metadata: [skill_manifest]

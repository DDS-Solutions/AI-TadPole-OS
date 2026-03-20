use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DangerLevel {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillParameter {
    pub r#type: String,
    pub required: Option<bool>,
    pub default: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillHooks {
    pub before_execute: Option<String>,
    pub after_execute: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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
}

impl Default for SkillManifest {
    fn default() -> Self {
        Self {
            schema_version: "1".to_string(),
            name: "unknown".to_string(),
            display_name: None,
            description: "".to_string(),
            version: "1.0.0".to_string(),
            author: None,
            permissions: vec![],
            toolset_group: None,
            danger_level: DangerLevel::Low,
            requires_oversight: false,
            parameters: HashMap::new(),
            hooks: None,
        }
    }
}

impl SkillManifest {
    pub fn validate(&mut self) -> anyhow::Result<()> {
        if self.schema_version != "1" {
            anyhow::bail!("Unsupported schema_version: {}", self.schema_version);
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
}

pub struct SkillRegistry {
    pub manifests: HashMap<String, SkillManifest>,
}

impl SkillRegistry {
    pub fn new() -> Self {
        Self {
            manifests: HashMap::new(),
        }
    }

    pub fn get(&self, name: &str) -> Option<&SkillManifest> {
        self.manifests.get(name)
    }

    pub fn load_all() -> Self {
        let mut registry = Self::new();

        let mut data_dir = PathBuf::from("data");
        data_dir.push("skills");

        if !data_dir.exists() {
            tracing::warn!("Skills directory not found at {:?}", data_dir);
            return registry;
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
                            registry.manifests.insert(manifest.name.clone(), manifest);
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

    fn load_manifest(path: &PathBuf) -> anyhow::Result<SkillManifest> {
        let file_contents = fs::read_to_string(path)?;
        let manifest: SkillManifest = serde_json::from_str(&file_contents)?;
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
            "Unsupported schema_version: 2"
        );
    }
}

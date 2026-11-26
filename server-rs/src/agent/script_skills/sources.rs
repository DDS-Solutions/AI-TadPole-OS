//! @docs ARCHITECTURE:Agent
//!
//! ### AI Assist Note
//! - **Subsystem**: Script skills / Ordered artifact sources
//! - Sources preserve existing last-source-wins precedence and generated-skill oversight on reload.
//!
//! ### 🔍 Debugging & Observability
//! - **Telemetry Targets**: none declared

use super::{
    codec::parse_skill_md,
    repository::{read_file_safe, ArtifactKind, ArtifactOrigin, ArtifactRepository},
    HookDefinition, ScriptSkillsRegistry, SkillDefinition, WorkflowDefinition,
};
use std::{collections::HashMap, path::PathBuf};
use tokio::fs;

#[derive(Clone, Copy)]
enum Format {
    SkillJson,
    SkillMarkdown,
    Workflow,
    Hook,
}

struct Source {
    directory: PathBuf,
    format: Format,
    origin: ArtifactOrigin,
}

fn ordered_sources(repository: &ArtifactRepository) -> Vec<Source> {
    use ArtifactOrigin::{Generated, User};
    vec![
        Source {
            directory: repository.directory(ArtifactKind::Skill, User),
            format: Format::SkillJson,
            origin: User,
        },
        Source {
            directory: repository.directory(ArtifactKind::Skill, Generated),
            format: Format::SkillJson,
            origin: Generated,
        },
        Source {
            directory: repository.workspace_root.join(".agent/skills"),
            format: Format::SkillMarkdown,
            origin: User,
        },
        Source {
            directory: repository.directory(ArtifactKind::Workflow, User),
            format: Format::Workflow,
            origin: User,
        },
        Source {
            directory: repository.workspace_root.join(".agent/workflows"),
            format: Format::Workflow,
            origin: User,
        },
        Source {
            directory: repository.directory(ArtifactKind::Workflow, Generated),
            format: Format::Workflow,
            origin: Generated,
        },
        Source {
            directory: repository.directory(ArtifactKind::Hook, User),
            format: Format::Hook,
            origin: User,
        },
        Source {
            directory: repository.directory(ArtifactKind::Hook, Generated),
            format: Format::Hook,
            origin: Generated,
        },
    ]
}

#[derive(Default)]
pub(super) struct RegistrySnapshot {
    skills: HashMap<String, SkillDefinition>,
    workflows: HashMap<String, WorkflowDefinition>,
    hooks: HashMap<String, HookDefinition>,
}

impl RegistrySnapshot {
    pub(super) async fn load(repository: &ArtifactRepository) -> Self {
        let mut snapshot = Self::default();
        for source in ordered_sources(repository) {
            let _ = fs::create_dir_all(&source.directory).await;
            let mut entries = match fs::read_dir(&source.directory).await {
                Ok(entries) => entries,
                Err(error) => {
                    tracing::warn!(
                        "Failed to read artifact source {:?}: {}",
                        source.directory,
                        error
                    );
                    continue;
                }
            };
            while let Ok(Some(entry)) = entries.next_entry().await {
                let path = entry.path();
                let path = match source.format {
                    Format::SkillMarkdown if path.is_dir() => path.join("SKILL.md"),
                    Format::SkillJson | Format::Hook
                        if path.extension().and_then(|extension| extension.to_str())
                            == Some("json") =>
                    {
                        path
                    }
                    Format::Workflow
                        if path.extension().and_then(|extension| extension.to_str())
                            == Some("md") =>
                    {
                        path
                    }
                    _ => continue,
                };
                let content = match read_file_safe(&path).await {
                    Ok(content) => content,
                    Err(error) => {
                        tracing::warn!("Failed to read artifact {:?}: {}", path, error);
                        continue;
                    }
                };
                match source.format {
                    Format::SkillJson | Format::SkillMarkdown => {
                        let skill = match source.format {
                            Format::SkillJson => serde_json::from_str::<SkillDefinition>(&content)
                                .map_err(|error| error.to_string()),
                            _ => parse_skill_md(&content)
                                .ok_or_else(|| "Invalid skill frontmatter".to_string()),
                        };
                        match skill {
                            Ok(mut skill) => {
                                if source.origin == ArtifactOrigin::Generated {
                                    skill.category = source.origin.category().to_string();
                                    skill.oversight_required = true;
                                }
                                snapshot.skills.insert(skill.name.clone(), skill);
                            }
                            Err(error) => {
                                tracing::warn!("Failed to parse skill {:?}: {}", path, error)
                            }
                        }
                    }
                    Format::Workflow => {
                        let Some(name) = path
                            .file_stem()
                            .map(|name| name.to_string_lossy().to_string())
                        else {
                            continue;
                        };
                        snapshot.workflows.insert(
                            name.clone(),
                            WorkflowDefinition {
                                id: None,
                                name,
                                content,
                                doc_url: None,
                                tags: None,
                                category: source.origin.category().to_string(),
                            },
                        );
                    }
                    Format::Hook => match serde_json::from_str::<HookDefinition>(&content) {
                        Ok(mut hook) => {
                            if source.origin == ArtifactOrigin::Generated {
                                hook.category = source.origin.category().to_string();
                            }
                            snapshot.hooks.insert(hook.name.clone(), hook);
                        }
                        Err(error) => tracing::warn!("Failed to parse hook {:?}: {}", path, error),
                    },
                }
            }
        }
        snapshot
    }

    pub(super) fn publish(self, registry: &ScriptSkillsRegistry) {
        // Keep insert-then-retain and the existing per-map publication order.
        publish_map(self.skills, &registry.skills);
        publish_map(self.workflows, &registry.workflows);
        publish_map(self.hooks, &registry.hooks);
    }
}

fn publish_map<T>(entries: HashMap<String, T>, target: &dashmap::DashMap<String, T>) {
    let keys: std::collections::HashSet<String> = entries.keys().cloned().collect();
    for (key, value) in entries {
        target.insert(key, value);
    }
    target.retain(|key, _| keys.contains(key));
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn generated_skill_requires_oversight_but_builtin_markdown_retains_precedence() {
        let temp = tempfile::tempdir().unwrap();
        let registry = ScriptSkillsRegistry::mock(temp.path().to_path_buf());
        registry.repository.ensure_directories().await.unwrap();
        let raw = r#"{"name":"same","description":"generated","execution_command":"","schema":{},"oversight_required":false,"category":"user"}"#;
        fs::write(
            temp.path()
                .join("execution/agent_generated/skills/same.json"),
            raw,
        )
        .await
        .unwrap();
        registry.reload_all().await.unwrap();
        assert!(registry.skills.get("same").unwrap().oversight_required);
        assert_eq!(
            registry.skills.get("same").unwrap().category,
            "agent_generated"
        );
        fs::create_dir_all(temp.path().join(".agent/skills/builtin"))
            .await
            .unwrap();
        fs::write(
            temp.path().join(".agent/skills/builtin/SKILL.md"),
            "---\nname: same\ndescription: builtin\noversight: false\n---\nBody"
                .replace('\n', "\r\n"),
        )
        .await
        .unwrap();
        registry.reload_all().await.unwrap();
        assert_eq!(registry.skills.get("same").unwrap().description, "builtin");
        assert!(!registry.skills.get("same").unwrap().oversight_required);
    }
    #[tokio::test]
    async fn generated_workflow_wins_and_removed_source_is_pruned() {
        let temp = tempfile::tempdir().unwrap();
        let registry = ScriptSkillsRegistry::mock(temp.path().to_path_buf());
        registry.repository.ensure_directories().await.unwrap();
        fs::write(temp.path().join("directives/demo.md"), "user")
            .await
            .unwrap();
        fs::create_dir_all(temp.path().join(".agent/workflows"))
            .await
            .unwrap();
        fs::write(temp.path().join(".agent/workflows/demo.md"), "builtin")
            .await
            .unwrap();
        fs::write(
            temp.path()
                .join("execution/agent_generated/workflows/demo.md"),
            "generated",
        )
        .await
        .unwrap();
        registry.reload_all().await.unwrap();
        assert_eq!(registry.workflows.get("demo").unwrap().content, "generated");
        fs::remove_file(
            temp.path()
                .join("execution/agent_generated/workflows/demo.md"),
        )
        .await
        .unwrap();
        registry.reload_all().await.unwrap();
        assert_eq!(registry.workflows.get("demo").unwrap().content, "builtin");
        fs::write(temp.path().join("execution/broken.json"), "{")
            .await
            .unwrap();
        registry.reload_all().await.unwrap();
        assert!(!registry.skills.contains_key("broken"));
    }
}

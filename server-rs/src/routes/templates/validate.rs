//! @docs ARCHITECTURE:Networking
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / HTTP Routes / templates / validate
//! - **Primary Entrypoints**: `validate_agent_payload`, `validate_workflow_content`, `validate_template_assets`
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Validators act as authoritative boundaries: return safe destination names, never raw input.
//! - `[Structural]` Pre-execution risk scanning via skillspector on all workflows and skills.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: `AppError::BadRequest`, `AppError::Forbidden`, `AppError::InternalServerError`
//! - **Telemetry Targets**: none declared
//! - **Witness Tests**: `routes::templates::validate::tests::*`

use super::dto::{apply_model_override, ModelOverrideConfig};
use super::naming::{apply_swarm_namespace, apply_workflow_namespace, sanitize_agent_filename, sanitize_error_str, sanitize_workflow_filename};
use crate::error::AppError;
use std::path::{Path, PathBuf};

pub const ALLOWED_SKILL_EXTENSIONS: &[&str] = &["json", "py", "js", "ts", "sh", "ps1", "bat", "md"];

pub fn is_allowed_skill_extension(ext: &str) -> bool {
    ALLOWED_SKILL_EXTENSIONS.contains(&ext.to_lowercase().as_str())
}

#[derive(Debug, Clone)]
pub struct ValidatedAgent {
    pub safe_filename: String,
    pub agent: crate::agent::types::EngineAgent,
    pub serialized: Vec<u8>,
}

#[derive(Debug)]
pub struct ValidatedTemplateAssets {
    pub safe_name: String,
    pub agents: Vec<ValidatedAgent>,
    pub workflows: Vec<(String, PathBuf)>,
    pub skills: Vec<PathBuf>,
    pub swarm_manifest: Option<PathBuf>,
    pub merged_mcp_config: Option<Vec<u8>>,
    pub mcp_server_count: usize,
    pub mcp_server_names: Vec<String>,
    pub mcp_replaced_count: usize,
}

pub fn validate_agent_payload(
    raw_filename: &str,
    content: &serde_json::Value,
    model_override: Option<&ModelOverrideConfig>,
    namespace: Option<&str>,
) -> Result<ValidatedAgent, AppError> {
    let clean_base = sanitize_agent_filename(raw_filename)?;

    let mut agent: crate::agent::types::EngineAgent = serde_json::from_value(content.clone())
        .map_err(|error| {
            AppError::BadRequest(format!(
                "Invalid agent profile '{}': {}",
                sanitize_error_str(raw_filename),
                error
            ))
        })?;

    let clean_agent_id = crate::utils::security::sanitize_id(&agent.identity.id);
    if clean_agent_id.is_empty() {
        return Err(AppError::BadRequest(format!(
            "Invalid agent identity id in profile '{}'",
            sanitize_error_str(raw_filename)
        )));
    }
    agent.identity.id = clean_agent_id;

    if let Some(override_cfg) = model_override {
        apply_model_override(&mut agent, override_cfg)?;
    }

    let (_, safe_filename) = apply_swarm_namespace(&mut agent, &clean_base, namespace);

    let serialized = serde_json::to_vec_pretty(&agent).map_err(|error| {
        AppError::InternalServerError(format!(
            "Failed to serialize agent profile '{}': {}",
            sanitize_error_str(raw_filename),
            error
        ))
    })?;

    Ok(ValidatedAgent {
        safe_filename,
        agent,
        serialized,
    })
}

pub async fn validate_workflow_content(
    raw_filename: &str,
    content: &str,
) -> Result<String, AppError> {
    let safe_name = sanitize_workflow_filename(raw_filename)?;

    crate::agent::workflows::WorkflowExecutionState::new(safe_name.clone(), content)?;

    // Scan using isolated temp directory to prevent file locking issues on Windows
    let temp_dir = tempfile::tempdir().map_err(AppError::Io)?;
    let temp_file_path = temp_dir.path().join(format!("{}.md", safe_name));
    tokio::fs::write(&temp_file_path, content.as_bytes())
        .await
        .map_err(AppError::Io)?;

    let scan = crate::security::skillspector::scan_path(&temp_file_path)
        .await
        .map_err(|error| {
            AppError::Forbidden(format!(
                "Security scan failed for workflow '{}': {}",
                sanitize_error_str(raw_filename),
                error
            ))
        })?;

    if scan.risk_score >= crate::security::skillspector::RISK_REJECT_THRESHOLD {
        return Err(AppError::Forbidden(format!(
            "Security audit failed: workflow '{}' has high risk score {}",
            sanitize_error_str(raw_filename),
            scan.risk_score
        )));
    }

    Ok(safe_name)
}

pub(crate) async fn collect_regular_files(
    directory: &Path,
    asset_kind: &str,
) -> Result<Vec<PathBuf>, AppError> {
    if !tokio::fs::try_exists(directory)
        .await
        .map_err(AppError::Io)?
    {
        return Ok(Vec::new());
    }
    let metadata = tokio::fs::symlink_metadata(directory)
        .await
        .map_err(AppError::Io)?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err(AppError::Forbidden(format!(
            "Template {} source '{}' must be a real directory",
            asset_kind,
            directory.display()
        )));
    }

    let mut files = Vec::new();
    let mut entries = tokio::fs::read_dir(directory).await.map_err(AppError::Io)?;
    while let Some(entry) = entries.next_entry().await.map_err(AppError::Io)? {
        let metadata = tokio::fs::symlink_metadata(entry.path())
            .await
            .map_err(AppError::Io)?;
        if metadata.file_type().is_symlink() {
            return Err(AppError::Forbidden(format!(
                "Template {} '{}' may not be a symbolic link",
                asset_kind,
                entry.path().display()
            )));
        }
        if metadata.is_dir() {
            return Err(AppError::BadRequest(format!(
                "Nested {} directory '{}' is not supported",
                asset_kind,
                entry.path().display()
            )));
        }
        if metadata.is_file() {
            files.push(entry.path());
        }
    }
    files.sort();
    Ok(files)
}

pub(crate) fn require_extension(path: &Path, expected: &str, asset_kind: &str) -> Result<(), AppError> {
    let actual_ext = path
        .extension()
        .and_then(|extension| extension.to_str())
        .unwrap_or("")
        .to_lowercase();
    if actual_ext != expected.to_lowercase() {
        return Err(AppError::Forbidden(format!(
            "Template {} '{}' must use the .{} extension",
            asset_kind,
            path.display(),
            expected
        )));
    }
    Ok(())
}

/// Note on skill destinations: Basenames obtained from `collect_regular_files`
/// contain no path separators and are safe by construction when joined to `execution/`.
pub(crate) fn validated_file_name<'a>(
    path: &'a Path,
    asset_kind: &str,
) -> Result<&'a std::ffi::OsStr, AppError> {
    path.file_name().ok_or_else(|| {
        AppError::BadRequest(format!(
            "Validated {} path '{}' has no filename",
            asset_kind,
            path.display()
        ))
    })
}

pub async fn validate_template_assets(
    source_path: &Path,
    workspace_root: &Path,
    template_path: &str,
    model_override: Option<&ModelOverrideConfig>,
    namespace: Option<&str>,
    allow_overwrite: bool,
) -> Result<ValidatedTemplateAssets, AppError> {
    let safe_name = crate::utils::security::sanitize_id(&template_path.replace(['/', '\\'], "_"));

    let agent_paths = collect_regular_files(&source_path.join("agents"), "agent").await?;
    let mut parsed_agents = Vec::new();
    for agent_path in agent_paths {
        require_extension(&agent_path, "json", "agent")?;
        let content_str = tokio::fs::read_to_string(&agent_path)
            .await
            .map_err(AppError::Io)?;
        let val: serde_json::Value = serde_json::from_str(&content_str).map_err(|error| {
            AppError::BadRequest(format!(
                "Invalid agent profile JSON '{}': {}",
                agent_path.display(),
                error
            ))
        })?;
        let file_name = agent_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("agent.json");
        let validated_agent = validate_agent_payload(file_name, &val, model_override, namespace)?;
        parsed_agents.push(validated_agent);
    }

    let workflows_raw = collect_regular_files(&source_path.join("workflows"), "workflow").await?;
    let mut workflows = Vec::new();
    for workflow in workflows_raw {
        require_extension(&workflow, "md", "workflow")?;
        let content = tokio::fs::read_to_string(&workflow)
            .await
            .map_err(AppError::Io)?;
        let file_name = workflow
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("workflow.md");
        let safe_base = validate_workflow_content(file_name, &content).await?;
        let namespaced_name = apply_workflow_namespace(&safe_base, namespace);
        workflows.push((namespaced_name, workflow));
    }

    let skills = collect_regular_files(&source_path.join("skills"), "skill").await?;
    for skill in &skills {
        let extension = skill
            .extension()
            .and_then(|extension| extension.to_str())
            .unwrap_or("")
            .to_lowercase();
        if !is_allowed_skill_extension(&extension) {
            return Err(AppError::Forbidden(format!(
                "Security boundary: Refusing skill '{}' with unsupported extension",
                skill.display()
            )));
        }
        let scan = crate::security::skillspector::scan_path(skill)
            .await
            .map_err(|error| {
                AppError::Forbidden(format!(
                    "Security scan failed for skill '{}': {}",
                    skill.display(),
                    error
                ))
            })?;
        if scan.risk_score >= crate::security::skillspector::RISK_REJECT_THRESHOLD {
            return Err(AppError::Forbidden(format!(
                "Security audit failed: template skill '{}' has high risk score {}",
                skill.display(),
                scan.risk_score
            )));
        }
    }

    let swarm_manifest_path = source_path.join("swarm.json");
    let swarm_manifest = if tokio::fs::try_exists(&swarm_manifest_path)
        .await
        .map_err(AppError::Io)?
    {
        let metadata = tokio::fs::symlink_metadata(&swarm_manifest_path)
            .await
            .map_err(AppError::Io)?;
        if metadata.file_type().is_symlink() || !metadata.is_file() {
            return Err(AppError::Forbidden(
                "Template swarm.json must be a regular file".to_string(),
            ));
        }
        let content = tokio::fs::read_to_string(&swarm_manifest_path)
            .await
            .map_err(AppError::Io)?;
        let value: serde_json::Value = serde_json::from_str(&content)
            .map_err(|error| AppError::BadRequest(format!("Invalid swarm.json: {}", error)))?;
        if !value.is_object() {
            return Err(AppError::BadRequest(
                "swarm.json must contain a JSON object".to_string(),
            ));
        }
        Some(swarm_manifest_path)
    } else {
        None
    };

    let (merged_mcp_config, mcp_server_count, mcp_server_names, mcp_replaced_count) =
        super::mcp_store::prepare_incoming_mcp_config(source_path, workspace_root, allow_overwrite).await?;

    // Early validation preflight for collisions
    if !allow_overwrite {
        let mut destinations = Vec::new();
        for agent in &parsed_agents {
            destinations.push(
                workspace_root
                    .join("data/swarm_config/agents")
                    .join(&agent.safe_filename),
            );
        }
        for (filename, _) in &workflows {
            destinations.push(
                workspace_root
                    .join("directives")
                    .join(filename),
            );
        }
        for skill in &skills {
            destinations.push(
                workspace_root
                    .join("execution")
                    .join(validated_file_name(skill, "skill")?),
            );
        }
        for destination in destinations {
            if tokio::fs::try_exists(&destination)
                .await
                .map_err(AppError::Io)?
            {
                return Err(AppError::Forbidden(format!(
                    "Security boundary: Refusing to overwrite existing file '{}'",
                    destination.display()
                )));
            }
        }
    }

    Ok(ValidatedTemplateAssets {
        safe_name,
        agents: parsed_agents,
        workflows,
        skills,
        swarm_manifest,
        merged_mcp_config,
        mcp_server_count,
        mcp_server_names,
        mcp_replaced_count,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_allowed_skill_extensions() {
        assert!(is_allowed_skill_extension("py"));
        assert!(is_allowed_skill_extension("json"));
        assert!(is_allowed_skill_extension("sh"));
        assert!(is_allowed_skill_extension("ps1"));
        assert!(is_allowed_skill_extension("bat"));
        assert!(is_allowed_skill_extension("md"));
        assert!(!is_allowed_skill_extension("exe"));
        assert!(!is_allowed_skill_extension("dll"));
        assert!(!is_allowed_skill_extension(""));
    }

    #[test]
    fn test_validate_agent_payload_valid_and_invalid() {
        let valid_json = serde_json::json!({
            "id": "valid-agent",
            "name": "Valid Agent",
            "role": "Specialist",
            "department": "Support",
            "description": "Handles tickets",
            "status": "active"
        });
        let res = validate_agent_payload("agent.json", &valid_json, None, None);
        assert!(res.is_ok());

        let invalid_ext = validate_agent_payload("agent.txt", &valid_json, None, None);
        assert!(matches!(invalid_ext, Err(AppError::Forbidden(_))));

        let invalid_schema = serde_json::json!({ "unknown_root": 123 });
        let invalid_res = validate_agent_payload("agent.json", &invalid_schema, None, None);
        assert!(matches!(invalid_res, Err(AppError::BadRequest(_))));
    }

    #[test]
    fn test_validate_agent_payload_normalizes_traversal_filenames() {
        let valid_json = serde_json::json!({
            "id": "traversal-agent",
            "name": "Traversal Agent",
            "role": "Analyst",
            "department": "Security",
            "description": "Tests traversal",
            "status": "active"
        });

        let res1 = validate_agent_payload("../../../../tmp/pwned.json", &valid_json, None, None);
        assert!(res1.is_ok());
        let validated1 = res1.unwrap();
        assert_eq!(validated1.safe_filename, "pwned.json");

        let res2 = validate_agent_payload("../../evil.json", &valid_json, None, Some("safe_ns"));
        assert!(res2.is_ok());
        let validated2 = res2.unwrap();
        assert_eq!(validated2.safe_filename, "safe_ns__evil.json");
    }

    #[tokio::test]
    async fn test_validate_workflow_content() {
        let valid_md = "---\nname: daily_sync\n---\n## Step 1: Initial Sync\nRun daily sync\n## Step 2: Verification\nVerify status";
        let res = validate_workflow_content("daily_sync.md", valid_md).await;
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), "daily_sync");

        let bad_ext = validate_workflow_content("daily_sync.txt", valid_md).await;
        assert!(matches!(bad_ext, Err(AppError::Forbidden(_))));
    }

    #[tokio::test]
    async fn test_validate_workflow_content_normalizes_traversal_filenames() {
        let valid_md = "---\nname: evil\n---\n## Step 1: Attack\nTest traversal";
        let res1 = validate_workflow_content("../../../../tmp/evil.md", valid_md).await;
        assert!(res1.is_ok());
        let safe = res1.unwrap();
        assert_eq!(safe, "evil");

        let namespaced = apply_workflow_namespace(&safe, Some("mkt"));
        assert_eq!(namespaced, "mkt__evil.md");
    }

    #[tokio::test]
    async fn test_collect_regular_files_rejects_nested_directories() {
        let temp = tempfile::tempdir().unwrap();
        let agents_dir = temp.path().join("agents");
        tokio::fs::create_dir_all(&agents_dir).await.unwrap();

        let nested_dir = agents_dir.join("sub_team");
        tokio::fs::create_dir_all(&nested_dir).await.unwrap();

        let res = collect_regular_files(&agents_dir, "agent").await;
        assert!(matches!(res, Err(AppError::BadRequest(_))));
    }

    #[tokio::test]
    async fn test_validate_template_assets_collision_refusal_when_overwrite_false() {
        let temp_repo = tempfile::tempdir().unwrap();
        let temp_workspace = tempfile::tempdir().unwrap();

        let agents_dir = temp_repo.path().join("agents");
        tokio::fs::create_dir_all(&agents_dir).await.unwrap();
        tokio::fs::write(
            agents_dir.join("worker.json"),
            serde_json::json!({
                "id": "worker",
                "name": "Worker",
                "role": "Execution",
                "department": "Ops",
                "description": "Performs tasks",
                "status": "active"
            }).to_string().as_bytes()
        ).await.unwrap();

        // Pre-create conflicting agent in workspace
        let ws_agent_dest = temp_workspace.path().join("data/swarm_config/agents").join("worker.json");
        tokio::fs::create_dir_all(ws_agent_dest.parent().unwrap()).await.unwrap();
        tokio::fs::write(&ws_agent_dest, b"existing").await.unwrap();

        let res = validate_template_assets(
            temp_repo.path(),
            temp_workspace.path(),
            "my_template",
            None,
            None,
            false,
        ).await;

        assert!(matches!(res, Err(AppError::Forbidden(_))));
    }
}

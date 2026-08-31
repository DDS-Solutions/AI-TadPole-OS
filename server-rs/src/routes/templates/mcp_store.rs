//! @docs ARCHITECTURE:Networking
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / HTTP Routes / templates / mcp_store
//! - **Primary Entrypoints**: `merge_mcp_config_data`, `merge_incoming_mcp_config`, `merge_and_save_mcp_config`, `prepare_incoming_mcp_config`, `prune_mcp_servers`
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Sole owner of .agent/mcp_config.json mutations and merges.
//! - `[Structural]` All read-modify-write spans are strictly serialized via internal Mutex.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: `AppError::BadRequest`, `AppError::Forbidden`, `AppError::InternalServerError`
//! - **Telemetry Targets**: none declared
//! - **Witness Tests**: `routes::templates::mcp_store::tests::*`

use crate::error::AppError;
use crate::utils::fs_transaction::InstallTransaction;
use std::path::Path;

/// Internal process-wide serialization lock for `.agent/mcp_config.json` modifications.
static MCP_LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

/// Pure function to merge incoming MCP servers into existing bytes with collision validation and accurate replacement counting.
pub fn merge_mcp_config_data(
    incoming: &crate::agent::mcp::McpConfig,
    existing_bytes: Option<&[u8]>,
    allow_overwrite: bool,
) -> Result<(Vec<u8>, usize, Vec<String>, usize), AppError> {
    for (server_name, server_config) in &incoming.mcp_servers {
        crate::agent::mcp::validate_mcp_server_config(server_name, server_config)?;
    }
    let incoming_count = incoming.mcp_servers.len();
    let mut server_names: Vec<String> = incoming.mcp_servers.keys().cloned().collect();
    server_names.sort();

    let mut merged: crate::agent::mcp::McpConfig = match existing_bytes {
        Some(bytes) if !bytes.is_empty() => serde_json::from_slice(bytes).map_err(|error| {
            AppError::BadRequest(format!(
                "Existing MCP configuration is corrupted or invalid: {}",
                error
            ))
        })?,
        _ => crate::agent::mcp::McpConfig {
            mcp_servers: std::collections::HashMap::new(),
        },
    };

    let mut replaced_count = 0;
    for (server_name, server_config) in &incoming.mcp_servers {
        if merged.mcp_servers.contains_key(server_name) {
            if !allow_overwrite {
                return Err(AppError::Forbidden(format!(
                    "Security boundary: Refusing to overwrite existing MCP server '{}'",
                    server_name
                )));
            }
            replaced_count += 1;
        }
        merged
            .mcp_servers
            .insert(server_name.clone(), server_config.clone());
    }

    let serialized = serde_json::to_vec_pretty(&merged).map_err(|error| {
        AppError::InternalServerError(format!("Failed to serialize MCP configuration: {}", error))
    })?;
    Ok((serialized, incoming_count, server_names, replaced_count))
}

/// Merges an incoming McpConfig structure with workspace .agent/mcp_config.json under exclusive lock.
pub async fn merge_incoming_mcp_config(
    incoming: &crate::agent::mcp::McpConfig,
    workspace_root: &Path,
    allow_overwrite: bool,
) -> Result<(Vec<u8>, usize, Vec<String>, usize), AppError> {
    let _guard = MCP_LOCK.lock().await;
    let destination = workspace_root.join(".agent/mcp_config.json");
    let existing_bytes = if tokio::fs::try_exists(&destination)
        .await
        .map_err(AppError::Io)?
    {
        Some(tokio::fs::read(&destination).await.map_err(AppError::Io)?)
    } else {
        None
    };

    merge_mcp_config_data(incoming, existing_bytes.as_deref(), allow_overwrite)
}

/// Merges incoming MCP configuration and commits changes atomically to .agent/mcp_config.json under exclusive lock.
pub async fn merge_and_save_mcp_config(
    workspace_root: &Path,
    incoming: &crate::agent::mcp::McpConfig,
    allow_overwrite: bool,
) -> Result<(usize, usize, Vec<String>), AppError> {
    let _guard = MCP_LOCK.lock().await;
    let destination = workspace_root.join(".agent/mcp_config.json");
    let existing_bytes = if tokio::fs::try_exists(&destination)
        .await
        .map_err(AppError::Io)?
    {
        Some(tokio::fs::read(&destination).await.map_err(AppError::Io)?)
    } else {
        None
    };

    let (serialized, incoming_count, server_names, replaced_count) =
        merge_mcp_config_data(incoming, existing_bytes.as_deref(), allow_overwrite)?;

    let mut tx = InstallTransaction::default();
    tx.replace_atomically(&destination, &serialized).await?;
    tx.commit().await;

    Ok((incoming_count, replaced_count, server_names))
}

/// Prepares and validates incoming MCP configuration from template source path under exclusive lock.
pub async fn prepare_incoming_mcp_config(
    source_path: &Path,
    workspace_root: &Path,
    allow_overwrite: bool,
) -> Result<(Option<Vec<u8>>, usize, Vec<String>, usize), AppError> {
    let incoming_path = source_path.join("mcps.json");
    if !tokio::fs::try_exists(&incoming_path)
        .await
        .map_err(AppError::Io)?
    {
        return Ok((None, 0, Vec::new(), 0));
    }

    let metadata = tokio::fs::symlink_metadata(&incoming_path)
        .await
        .map_err(AppError::Io)?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err(AppError::Forbidden(
            "Template mcps.json must be a regular file".to_string(),
        ));
    }

    let incoming_content = tokio::fs::read_to_string(&incoming_path)
        .await
        .map_err(AppError::Io)?;
    let incoming: crate::agent::mcp::McpConfig = serde_json::from_str(&incoming_content)
        .map_err(|error| AppError::BadRequest(format!("Invalid mcps.json: {}", error)))?;

    let (serialized, count, names, replaced) =
        merge_incoming_mcp_config(&incoming, workspace_root, allow_overwrite).await?;
    Ok((Some(serialized), count, names, replaced))
}

/// Prunes uninstalled MCP servers from .agent/mcp_config.json under exclusive lock.
pub async fn prune_mcp_servers(
    workspace_root: &Path,
    servers_to_prune: &[String],
) -> Result<Vec<String>, AppError> {
    if servers_to_prune.is_empty() {
        return Ok(Vec::new());
    }

    let _guard = MCP_LOCK.lock().await;
    let mcp_path = workspace_root.join(".agent/mcp_config.json");

    if !tokio::fs::try_exists(&mcp_path).await.unwrap_or(false) {
        return Ok(Vec::new());
    }

    let content = tokio::fs::read_to_string(&mcp_path)
        .await
        .map_err(AppError::Io)?;
    let mut config: crate::agent::mcp::McpConfig = serde_json::from_str(&content).map_err(|e| {
        AppError::BadRequest(format!(
            "Cannot prune MCP servers: existing MCP configuration is corrupted: {}",
            e
        ))
    })?;

    let mut uninstalled = Vec::new();
    for server_name in servers_to_prune {
        if config.mcp_servers.remove(server_name).is_some() {
            uninstalled.push(server_name.clone());
        }
    }

    let serialized = serde_json::to_vec_pretty(&config).map_err(|e| {
        AppError::InternalServerError(format!(
            "Failed to serialize pruned MCP configuration: {}",
            e
        ))
    })?;

    let mut tx = InstallTransaction::default();
    tx.replace_atomically(&mcp_path, &serialized).await?;
    tx.commit().await;

    Ok(uninstalled)
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    #[test]
    fn test_merge_mcp_config_data_conflict_without_overwrite() {
        let mut incoming_servers = std::collections::HashMap::new();
        incoming_servers.insert(
            "stripe".to_string(),
            crate::agent::mcp::McpServerConfig {
                command: "node".to_string(),
                args: vec!["stripe.js".to_string()],
                env: None,
            },
        );
        let incoming = crate::agent::mcp::McpConfig {
            mcp_servers: incoming_servers,
        };

        let mut existing_servers = std::collections::HashMap::new();
        existing_servers.insert(
            "stripe".to_string(),
            crate::agent::mcp::McpServerConfig {
                command: "python".to_string(),
                args: vec!["stripe.py".to_string()],
                env: None,
            },
        );
        let existing = crate::agent::mcp::McpConfig {
            mcp_servers: existing_servers,
        };
        let existing_bytes = serde_json::to_vec(&existing).unwrap();

        let res = merge_mcp_config_data(&incoming, Some(&existing_bytes), false);
        assert!(matches!(res, Err(AppError::Forbidden(_))));

        let overwrite_res = merge_mcp_config_data(&incoming, Some(&existing_bytes), true);
        assert!(overwrite_res.is_ok());
        let (bytes, count, names, replaced) = overwrite_res.unwrap();
        assert_eq!(count, 1);
        assert_eq!(names, vec!["stripe"]);
        assert_eq!(replaced, 1);
        assert!(!bytes.is_empty());
    }

    #[test]
    fn test_merge_mcp_config_data_rejects_corrupted_existing() {
        let incoming = crate::agent::mcp::McpConfig {
            mcp_servers: std::collections::HashMap::new(),
        };
        let bad_bytes = b"not json at all";
        let res = merge_mcp_config_data(&incoming, Some(bad_bytes), true);
        assert!(matches!(res, Err(AppError::BadRequest(_))));
    }

    #[tokio::test]
    async fn test_merge_and_save_mcp_config() {
        let temp = tempfile::tempdir().unwrap();
        let mut incoming_servers = std::collections::HashMap::new();
        incoming_servers.insert(
            "github".to_string(),
            crate::agent::mcp::McpServerConfig {
                command: "node".to_string(),
                args: vec!["github.js".to_string()],
                env: None,
            },
        );
        let incoming = crate::agent::mcp::McpConfig {
            mcp_servers: incoming_servers,
        };

        let res = merge_and_save_mcp_config(temp.path(), &incoming, true).await;
        assert!(res.is_ok());
        let (count, replaced, names) = res.unwrap();
        assert_eq!(count, 1);
        assert_eq!(replaced, 0);
        assert_eq!(names, vec!["github"]);

        let mcp_file = temp.path().join(".agent/mcp_config.json");
        assert!(mcp_file.exists());
    }
}

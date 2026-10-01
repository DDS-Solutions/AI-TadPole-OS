//! @docs ARCHITECTURE:Registry:Mcp
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Agent Runner / MCP Configuration Store
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Configuration path traversal containment and schema validation.
//! - `[Structural]` Thread-safe cached configuration snapshotting to eliminate per-call disk I/O.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: `AppError::Forbidden`, `AppError::NotFound`, `AppError::BadRequest`
//! - **Telemetry Targets**: none declared

use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;

use super::config::{validate_mcp_server_config, McpConfig, McpServerConfig};
use crate::error::AppError;

/// Thread-safe configuration store with cached snapshotting.
#[derive(Clone)]
pub struct McpConfigStore {
    config_path: PathBuf,
    cache: Arc<RwLock<Option<Arc<McpConfig>>>>,
}

impl McpConfigStore {
    pub fn new(config_path: PathBuf) -> Self {
        Self {
            config_path,
            cache: Arc::new(RwLock::new(None)),
        }
    }

    pub fn config_path(&self) -> &Path {
        &self.config_path
    }

    /// Invalidates the cached configuration snapshot.
    pub async fn invalidate(&self) {
        let mut write_lock = self.cache.write().await;
        *write_lock = None;
    }

    /// Returns a cached configuration snapshot, loading and validating from disk on cache miss.
    pub async fn get_config(&self) -> Result<Arc<McpConfig>, AppError> {
        // Fast path: cached snapshot
        {
            let read_lock = self.cache.read().await;
            if let Some(ref snapshot) = *read_lock {
                return Ok(snapshot.clone());
            }
        }

        // Slow path: load, validate, and cache
        let mut write_lock = self.cache.write().await;
        if let Some(ref snapshot) = *write_lock {
            return Ok(snapshot.clone());
        }

        let authorized_base = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        let safe_path = crate::utils::security::validate_path(
            &authorized_base,
            &self.config_path.to_string_lossy(),
        )
        .map_err(|e| AppError::Forbidden(e.to_string()))?;

        let content = tokio::fs::read_to_string(safe_path)
            .await
            .map_err(AppError::Io)?;
        let config: McpConfig =
            serde_json::from_str(&content).map_err(|e| AppError::BadRequest(e.to_string()))?;

        for (name, server_cfg) in &config.mcp_servers {
            validate_mcp_server_config(name, server_cfg)?;
        }

        let snapshot = Arc::new(config);
        *write_lock = Some(snapshot.clone());
        Ok(snapshot)
    }

    /// Retrieves a specific server configuration from the cached snapshot.
    pub async fn get_server_config(&self, server_name: &str) -> Result<McpServerConfig, AppError> {
        let config = self.get_config().await?;
        config.mcp_servers.get(server_name).cloned().ok_or_else(|| {
            AppError::NotFound(format!("MCP server '{}' not found in config", server_name))
        })
    }
}

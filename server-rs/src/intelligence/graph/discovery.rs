//! @docs ARCHITECTURE:Intelligence
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Intelligence / File Discovery
//! - **Primary Entrypoints**: `FileDiscoverer`, `FileDiscoveryService`
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none declared
//! - **Telemetry Targets**: none declared
//! - **Witness Tests**: none declared

use super::models::{GraphError, EXCLUDED_DIRS, MAX_FILE_SIZE_BYTES};
use super::path_utils::sanitize_log_path;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Service trait to discover files within the workspace root
pub trait FileDiscoverer: Send + Sync {
    fn discover(&self, root: &Path) -> Result<Vec<PathBuf>, GraphError>;
}

/// Default implementation of the discovery service
pub struct FileDiscoveryService {
    pub exclusions: Vec<String>,
}

impl Default for FileDiscoveryService {
    fn default() -> Self {
        Self {
            exclusions: EXCLUDED_DIRS.iter().map(|s| s.to_string()).collect(),
        }
    }
}

impl FileDiscoverer for FileDiscoveryService {
    fn discover(&self, root: &Path) -> Result<Vec<PathBuf>, GraphError> {
        let canonical_root = root.canonicalize().map_err(|e| {
            GraphError::WorkspaceRootNotFound(format!(
                "Failed to canonicalize root {}: {}",
                root.display(),
                e
            ))
        })?;

        let exclusions = &self.exclusions;
        let mut files = Vec::new();

        for entry in WalkDir::new(root)
            .follow_links(false)
            .into_iter()
            .filter_entry(|e| {
                let name = e.file_name().to_string_lossy();
                !exclusions.iter().any(|ex| ex.eq_ignore_ascii_case(&name))
            })
        {
            let entry = match entry {
                Ok(e) => e,
                Err(e) => {
                    tracing::warn!("⚠️ [Graph] Directory traversal warning: {}", e);
                    continue;
                }
            };

            if !entry.path().is_file() {
                continue;
            }

            let path = entry.path();
            let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
            let is_code = ext == "rs" || ext == "ts" || ext == "tsx";
            let is_wiki = ext == "md"
                && path
                    .to_string_lossy()
                    .replace('\\', "/")
                    .contains("docs/wiki");
            if !is_code && !is_wiki {
                continue;
            }

            // 🛡️ [DoS Protection] Enforce unified size limit
            let metadata = match std::fs::metadata(path) {
                Ok(m) => m,
                Err(e) => {
                    tracing::warn!(
                        "⚠️ [Graph] Failed to read metadata for {}: {}",
                        sanitize_log_path(path, root),
                        e
                    );
                    continue;
                }
            };

            if metadata.len() > MAX_FILE_SIZE_BYTES {
                tracing::warn!(
                    "⚠️ [Graph] Skipping oversized file ({} bytes): {}",
                    metadata.len(),
                    sanitize_log_path(path, root)
                );
                continue;
            }

            // 🛡️ Path Boundary Verification (Symlink Protection)
            let canonical_path = match path.canonicalize() {
                Ok(p) => p,
                Err(e) => {
                    tracing::warn!(
                        "⚠️ [Graph] Failed to canonicalize path {}: {}",
                        sanitize_log_path(path, root),
                        e
                    );
                    continue;
                }
            };

            if !canonical_path.starts_with(&canonical_root) {
                tracing::warn!(
                    "⚠️ [Graph] Security Violation: Path {} points outside workspace root {}",
                    sanitize_log_path(&canonical_path, root),
                    sanitize_log_path(&canonical_root, root)
                );
                continue;
            }

            files.push(path.to_path_buf());
        }

        files.sort();
        Ok(files)
    }
}

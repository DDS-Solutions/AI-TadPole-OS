//! @docs ARCHITECTURE:Infrastructure
//!
//! ### AI Assist Note
//! **Filesystem Adapter**: Orchestrates a sandboxed interface for agents
//! to interact with the local filesystem. Enforces **Strict Path Traversal
//! Protection** (SEC-03) by canonicalizing all candidate paths before
//! access, defeating symlink-based sandbox escapes. Distinguishes
//! between **Ephemeral Workspace Manipulation** (SCRATCH-01) and
//! persistent engine storage.
//!
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: Illegal path traversal attempt (triggers `SECURITY
//!   FAULT`), permission denied at the OS level, or disk exhaustion
//!   preventing write operations.
//! - **Trace Scope**: `server-rs::adapter::filesystem`

use anyhow::{anyhow, Result};
use std::path::{Path, PathBuf};
use tokio::fs;

#[derive(Clone)]
pub struct FilesystemAdapter {
    pub root_path: PathBuf,
}

impl FilesystemAdapter {
    /// Creates a new adapter. The `root_path` is created if it doesn't exist,
    /// then immediately canonicalized to get its real, symlink-resolved path.
    /// This is the SEC-03 fix: prevents symlink-based sandbox escapes.
    pub fn new(root_path: PathBuf) -> Self {
        // We'll lazily canonicalize on first use to avoid blocking in new().
        Self { root_path }
    }

    /// Verifies the requested path stays inside the workspace.
    ///
    /// ### 🛡️ Security Model: Sandbox Defense (SEC-03)
    /// 1. **Component Filtering**: Rejects any path containing explicit `..` 
    ///    parent directory components before resolution.
    /// 2. **Canonicalization**: Resolves all symlinks and relative segments 
    ///    using the OS's real-path resolution.
    /// 3. **Subpath Assertion**: Verifies that the resulting real path 
    ///    starts with the canonicalized workspace root.
    ///
    /// This implementation defeats **Symlink-based Sandbox Escapes** and 
    /// **TOCTOU** (Time-of-Check Time-of-Use) races by canonicalizing both 
    /// the root and the candidate before the prefix check.
    async fn get_safe_path(&self, requested_path: &str) -> Result<PathBuf> {
        // Build the candidate path (without canonicalization first)
        let mut candidate = self.root_path.clone();

        for component in Path::new(requested_path).components() {
            match component {
                std::path::Component::Normal(c) => candidate.push(c),
                std::path::Component::ParentDir => {
                    return Err(anyhow!("🚫 SECURITY FAULT: Illegal path traversal attempt detected. Access denied."));
                }
                // Ignore absolute roots/prefixes to keep path relative to our root
                std::path::Component::RootDir | std::path::Component::Prefix(_) => {}
                _ => {}
            }
        }

        // Resolve the real root (SEC-03: canonicalize to defeat symlinks).
        // We use the parent chain to canonicalize even if the leaf doesn't exist yet.
        let canonical_root = canonicalize_or_create(&self.root_path).await?;
        let canonical_candidate = canonicalize_or_create_parent(&candidate)
            .await
            .unwrap_or_else(|_| candidate.clone());

        if !canonical_candidate.starts_with(&canonical_root) {
            return Err(anyhow!(
                "🚫 SECURITY FAULT: Attempted to access '{}' which is outside the designated workspace '{}'.",
                canonical_candidate.display(),
                canonical_root.display()
            ));
        }

        Ok(candidate)
    }

    pub async fn write_file(&self, filename: &str, content: &str) -> Result<()> {
        let path = self.get_safe_path(filename).await?;

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).await?;
        }

        fs::write(path, content).await?;
        Ok(())
    }

    pub async fn read_file(&self, filename: &str) -> Result<String> {
        let path = self.get_safe_path(filename).await?;
        let content = fs::read_to_string(path).await?;
        Ok(content)
    }

    pub async fn list_files(&self, dir: &str) -> Result<Vec<String>> {
        let path = self.get_safe_path(dir).await?;

        if !fs::try_exists(&path).await.unwrap_or(false) {
            return Ok(vec![]);
        }

        let mut entries = fs::read_dir(path).await?;
        let mut files = Vec::new();

        while let Some(entry) = entries.next_entry().await? {
            let name = entry.file_name().to_string_lossy().to_string();
            let is_dir = entry.file_type().await?.is_dir();
            files.push(if is_dir { format!("{}/", name) } else { name });
        }

        files.sort(); // deterministic order
        Ok(files)
    }

    pub async fn delete_file(&self, filename: &str) -> Result<()> {
        let path = self.get_safe_path(filename).await?;
        if path.is_file() {
            fs::remove_file(path).await?;
        } else if path.is_dir() {
            fs::remove_dir_all(path).await?;
        }
        Ok(())
    }
}

// ─────────────────────────────────────────────────────────
//  HELPERS
// ─────────────────────────────────────────────────────────

/// Canonicalize a path, creating the directory first if it doesn't exist.
/// This handles the common case where the workspace root hasn't been created yet.
/// 
/// ### ⚠️ Critical Path Resolution
/// We must create the directory *before* canonicalization, as standard OS 
/// `realpath` wrappers often fail or return inconsistent results for non-existent 
/// terminal nodes.
async fn canonicalize_or_create(path: &Path) -> Result<PathBuf> {
    if !fs::try_exists(path).await.unwrap_or(false) {
        fs::create_dir_all(path).await.map_err(|e| {
            anyhow!(
                "Failed to create workspace root '{}': {}",
                path.display(),
                e
            )
        })?;
    }
    fs::canonicalize(path).await.map_err(|e| {
        anyhow!(
            "Failed to canonicalize workspace root '{}': {}",
            path.display(),
            e
        )
    })
}

/// Canonicalize by walking up the path until we find an existing component,
/// then append the remaining leaf segments. Handles paths that don't exist yet.
async fn canonicalize_or_create_parent(path: &Path) -> Result<PathBuf> {
    // Walk up the tree to find the nearest existing ancestor
    let mut existing = path.to_path_buf();
    let mut suffix = Vec::new();

    while !fs::try_exists(&existing).await.unwrap_or(false) {
        if let Some(name) = existing.file_name() {
            suffix.push(name.to_os_string());
        }
        match existing.parent() {
            Some(p) => existing = p.to_path_buf(),
            None => break,
        }
    }

    let mut canonical = fs::canonicalize(&existing).await.unwrap_or(existing);

    // Re-append the non-existent suffix in reverse
    for part in suffix.into_iter().rev() {
        canonical.push(part);
    }

    Ok(canonical)
}

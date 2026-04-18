//! @docs ARCHITECTURE:Infrastructure
//!
//! ### AI Assist Note
//! **Vault Adapter**: Orchestrates the persistence of mission findings
//! and high-priority system logs to the local vault. Automatically
//! injects **Horizontal Rules** (`---`) and **UTC Timestamps** before
//! content blocks to maintain a human-readable discovery ledger.
//! Enforces **Path Traversal Protection** to ensure all logs remain
//! within the designated vault boundaries (VLT-01).
//!
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: Illegal path traversal attempt (`Illegal path
//!   traversal detected`), permission denied on vault root, or disk
//!   exhaustion preventing log append operations.
//! - **Trace Scope**: `server-rs::adapter::vault`

use anyhow::Result;
use std::path::PathBuf;
use tokio::fs;

pub struct VaultAdapter {
    pub root_path: PathBuf,
}

impl VaultAdapter {
    pub fn new(root_path: PathBuf) -> Self {
        Self { root_path }
    }

    /// Verifies that the path is within the vault and contains no traversal attempts.
    fn get_safe_path(&self, filename: &str) -> Result<PathBuf> {
        let mut path = self.root_path.clone();
        for component in std::path::Path::new(filename).components() {
            if let std::path::Component::Normal(c) = component {
                path.push(c);
            } else if let std::path::Component::ParentDir = component {
                return Err(anyhow::anyhow!(
                    "Illegal path traversal detected in vault adapter"
                ));
            }
        }

        if !path.starts_with(&self.root_path) {
            return Err(anyhow::anyhow!("Attempted to access file outside of vault"));
        }

        Ok(path)
    }

    /// Appends findings to a markdown file in the vault.
    pub async fn append_to_file(&self, filename: &str, content: &str) -> Result<()> {
        let path = self.get_safe_path(filename)?;

        // Ensure directory exists
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).await?;
        }

        let mut file_content = if fs::try_exists(&path).await.unwrap_or(false) {
            fs::read_to_string(&path).await?
        } else {
            String::new()
        };

        file_content.push_str("\n\n---\n");
        file_content.push_str(&format!("### Logged at: {}\n", chrono::Utc::now()));
        file_content.push_str(content);

        fs::write(&path, file_content).await?;
        Ok(())
    }

    #[allow(dead_code)]
    pub async fn read_file(&self, filename: &str) -> Result<String> {
        let path = self.get_safe_path(filename)?;
        Ok(fs::read_to_string(path).await?)
    }
}

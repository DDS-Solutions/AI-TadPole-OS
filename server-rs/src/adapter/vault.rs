//! @docs ARCHITECTURE:Infrastructure
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / vault
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Type-safe state handling and bounded execution without unhandled panics.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: none declared
//! - **Witness Tests**: none declared

use anyhow::{anyhow, Result};
use std::path::{Path, PathBuf};
use tokio::fs;
use tokio::io::AsyncWriteExt;

pub struct VaultAdapter {
    pub root_path: PathBuf,
}

impl VaultAdapter {
    pub fn new(root_path: PathBuf) -> Self {
        Self { root_path }
    }

    /// Verifies that the path is within the vault and contains no traversal attempts or escaping symlinks.
    async fn get_safe_path(&self, filename: &str) -> Result<PathBuf> {
        let mut candidate = self.root_path.clone();
        for component in std::path::Path::new(filename).components() {
            match component {
                std::path::Component::Normal(c) => candidate.push(c),
                std::path::Component::ParentDir => {
                    return Err(anyhow!("Illegal path traversal detected in vault adapter"));
                }
                std::path::Component::RootDir | std::path::Component::Prefix(_) => {}
                _ => {}
            }
        }

        let canonical_root = canonicalize_or_create(&self.root_path).await?;
        let canonical_candidate =
            canonicalize_or_create_parent(&candidate, &canonical_root).await?;

        // Inspect leaf file for symlink escape if it exists
        if let Ok(meta) = fs::symlink_metadata(&canonical_candidate).await {
            if meta.file_type().is_symlink() {
                let resolved = fs::canonicalize(&canonical_candidate).await.map_err(|e| {
                    anyhow!(
                        "Unresolvable symlink at '{}': {}",
                        canonical_candidate.display(),
                        e
                    )
                })?;
                let resolved_str = resolved.to_string_lossy().replace(r"\\?\", "");
                let root_str = canonical_root.to_string_lossy().replace(r"\\?\", "");
                if !resolved.starts_with(&canonical_root) && !resolved_str.starts_with(&root_str) {
                    return Err(anyhow!(
                        "Attempted to access file outside of vault via symlink '{}' -> '{}'",
                        canonical_candidate.display(),
                        resolved.display()
                    ));
                }
            }
        }

        let canon_candidate_norm = normalize_path_str(&canonical_candidate);
        let canon_root_norm = normalize_path_str(&canonical_root);
        let root_prefix = if canon_root_norm.ends_with('/') {
            canon_root_norm.clone()
        } else {
            format!("{}/", canon_root_norm)
        };

        if !canonical_candidate.starts_with(&canonical_root)
            && !canon_candidate_norm.starts_with(&root_prefix)
            && canon_candidate_norm != canon_root_norm
        {
            return Err(anyhow!("Attempted to access file outside of vault"));
        }

        Ok(canonical_candidate)
    }

    /// Appends findings to a markdown file in the vault.
    pub async fn append_to_file(&self, filename: &str, content: &str) -> Result<()> {
        let path = self.get_safe_path(filename).await?;

        // Ensure directory exists
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).await?;
        }

        let entry = format!(
            "\n\n---\n### Logged at: {}\n{}",
            chrono::Utc::now(),
            content
        );

        let mut file = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
            .await?;

        file.write_all(entry.as_bytes()).await?;
        Ok(())
    }

    #[allow(dead_code)]
    pub async fn read_file(&self, filename: &str) -> Result<String> {
        let path = self.get_safe_path(filename).await?;
        Ok(fs::read_to_string(path).await?)
    }
}

fn normalize_path_str(p: &Path) -> String {
    let s = p.to_string_lossy().replace('\\', "/");
    if let Some(stripped) = s.strip_prefix("//?/") {
        stripped.to_string()
    } else {
        s
    }
}

async fn canonicalize_or_create(path: &Path) -> Result<PathBuf> {
    if !fs::try_exists(path).await.unwrap_or(false) {
        fs::create_dir_all(path)
            .await
            .map_err(|e| anyhow!("Failed to create vault root '{}': {}", path.display(), e))?;
    }
    fs::canonicalize(path).await.map_err(|e| {
        anyhow!(
            "Failed to canonicalize vault root '{}': {}",
            path.display(),
            e
        )
    })
}

async fn canonicalize_or_create_parent(path: &Path, canonical_root: &Path) -> Result<PathBuf> {
    let mut existing = path.to_path_buf();
    let mut suffix = Vec::new();

    loop {
        match fs::symlink_metadata(&existing).await {
            Ok(meta) => {
                if meta.file_type().is_symlink() {
                    match fs::canonicalize(&existing).await {
                        Ok(target_canon) => {
                            let canon_target_norm = normalize_path_str(&target_canon);
                            let canon_root_norm = normalize_path_str(canonical_root);
                            let root_prefix = if canon_root_norm.ends_with('/') {
                                canon_root_norm.clone()
                            } else {
                                format!("{}/", canon_root_norm)
                            };

                            if !target_canon.starts_with(canonical_root)
                                && !canon_target_norm.starts_with(&root_prefix)
                                && canon_target_norm != canon_root_norm
                            {
                                return Err(anyhow!(
                                    "Attempted to access file outside of vault via symlink parent '{}' -> '{}'",
                                    existing.display(),
                                    target_canon.display()
                                ));
                            }
                            existing = target_canon;
                        }
                        Err(e) => {
                            return Err(anyhow!(
                                "Unresolvable symlink detected at '{}': {}",
                                existing.display(),
                                e
                            ));
                        }
                    }
                }
                break;
            }
            Err(_) => {
                if let Some(name) = existing.file_name() {
                    suffix.push(name.to_os_string());
                }
                match existing.parent() {
                    Some(p) => existing = p.to_path_buf(),
                    None => break,
                }
            }
        }
    }

    let mut canonical = fs::canonicalize(&existing).await.unwrap_or(existing);

    for part in suffix.into_iter().rev() {
        canonical.push(part);
    }

    Ok(canonical)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_vault_append_and_read() {
        let temp_dir = tempfile::tempdir().unwrap();
        let vault_root = temp_dir.path().join("vault");
        let adapter = VaultAdapter::new(vault_root);

        let result = adapter
            .append_to_file("notes.md", "Research finding 1")
            .await;
        assert!(result.is_ok());

        let content = adapter.read_file("notes.md").await.unwrap();
        assert!(content.contains("Research finding 1"));
    }

    #[tokio::test]
    async fn test_vault_traversal_rejected() {
        let temp_dir = tempfile::tempdir().unwrap();
        let vault_root = temp_dir.path().join("vault");
        let adapter = VaultAdapter::new(vault_root);

        let result = adapter
            .append_to_file("../outside.md", "malicious payload")
            .await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Illegal path traversal"));
    }

    #[tokio::test]
    async fn test_vault_symlink_escape_rejected() {
        let temp_dir = tempfile::tempdir().unwrap();
        let vault_root = temp_dir.path().join("vault");
        tokio::fs::create_dir_all(&vault_root).await.unwrap();

        let outside_dir = temp_dir.path().join("outside");
        tokio::fs::create_dir_all(&outside_dir).await.unwrap();
        let outside_file = outside_dir.join("secret.txt");
        tokio::fs::write(&outside_file, "initial").await.unwrap();

        let symlink_path = vault_root.join("leak_link.md");
        #[cfg(windows)]
        let created = std::os::windows::fs::symlink_file(&outside_file, &symlink_path).is_ok();
        #[cfg(unix)]
        let created = std::os::unix::fs::symlink(&outside_file, &symlink_path).is_ok();

        if created {
            let adapter = VaultAdapter::new(vault_root);
            let res = adapter
                .append_to_file("leak_link.md", "malicious write")
                .await;
            assert!(
                res.is_err(),
                "Symlink redirecting outside vault must be rejected"
            );
            let after = tokio::fs::read_to_string(&outside_file).await.unwrap();
            assert_eq!(after, "initial", "Outside file must not have been modified");
        }
    }
}

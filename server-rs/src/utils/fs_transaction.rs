//! @docs ARCHITECTURE:UtilityFoundation
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / UtilityFoundation / fs_transaction
//! - **Primary Entrypoints**: `InstallTransaction`, `InstallUndo`, `TemporaryClone`
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Transactional file staging with deterministic automatic rollback on uncommitted drop.
//! - `[Structural]` Atomic replacement with backup retention and kernel-level `create_new` refusal on collision.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: `AppError::Io`, `AppError::Forbidden`
//! - **Telemetry Targets**: none declared
//! - **Witness Tests**: `utils::fs_transaction::tests::*`

use crate::error::AppError;
use std::path::{Path, PathBuf};
use tokio::io::AsyncWriteExt;

#[derive(Debug)]
pub enum InstallUndo {
    Remove(PathBuf),
    Restore {
        destination: PathBuf,
        backup: PathBuf,
    },
}

#[derive(Debug, Default)]
pub struct InstallTransaction {
    pub undo: Vec<InstallUndo>,
    pub committed: bool,
}

pub struct TemporaryClone(pub PathBuf);

impl Drop for TemporaryClone {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

impl InstallTransaction {
    pub async fn copy_new(&mut self, source: &Path, destination: &Path, allow_overwrite: bool) -> Result<bool, AppError> {
        if let Some(parent) = destination.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .map_err(AppError::Io)?;
        }

        let contents = tokio::fs::read(source).await.map_err(AppError::Io)?;
        self.write_new(destination, &contents, allow_overwrite).await
    }

    pub async fn write_new(&mut self, destination: &Path, contents: &[u8], allow_overwrite: bool) -> Result<bool, AppError> {
        if let Some(parent) = destination.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .map_err(AppError::Io)?;
        }

        if !allow_overwrite {
            let mut file = match tokio::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(destination)
                .await
            {
                Ok(f) => f,
                Err(err) if err.kind() == std::io::ErrorKind::AlreadyExists => {
                    return Err(AppError::Forbidden(format!(
                        "Security boundary: Refusing to overwrite existing file '{}'",
                        destination.display()
                    )));
                }
                Err(err) => return Err(AppError::Io(err)),
            };

            if let Err(err) = file.write_all(contents).await {
                drop(file);
                let _ = tokio::fs::remove_file(destination).await;
                return Err(AppError::Io(err));
            }
            if let Err(err) = file.flush().await {
                drop(file);
                let _ = tokio::fs::remove_file(destination).await;
                return Err(AppError::Io(err));
            }

            self.undo.push(InstallUndo::Remove(destination.to_path_buf()));
            Ok(false)
        } else {
            let existed = tokio::fs::try_exists(destination).await.map_err(AppError::Io)?;
            self.replace_atomically(destination, contents).await?;
            Ok(existed)
        }
    }

    pub async fn replace_atomically(
        &mut self,
        destination: &Path,
        contents: &[u8],
    ) -> Result<(), AppError> {
        if let Some(parent) = destination.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .map_err(AppError::Io)?;
        }

        let transaction_id = uuid::Uuid::new_v4();
        let file_name = destination
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("file.tmp");
        let parent = destination.parent().unwrap_or_else(|| Path::new("."));
        let staged = parent.join(format!(".{}.{}.tmp", file_name, transaction_id));
        let backup = parent.join(format!(".{}.{}.bak", file_name, transaction_id));

        let mut staged_file = tokio::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&staged)
            .await
            .map_err(AppError::Io)?;
        if let Err(error) = staged_file.write_all(contents).await {
            drop(staged_file);
            let _ = tokio::fs::remove_file(&staged).await;
            return Err(AppError::Io(error));
        }
        if let Err(error) = staged_file.flush().await {
            drop(staged_file);
            let _ = tokio::fs::remove_file(&staged).await;
            return Err(AppError::Io(error));
        }
        drop(staged_file);

        if tokio::fs::try_exists(destination)
            .await
            .map_err(AppError::Io)?
        {
            if let Err(error) = tokio::fs::rename(destination, &backup).await {
                let _ = tokio::fs::remove_file(&staged).await;
                return Err(AppError::Io(error));
            }
            if let Err(error) = tokio::fs::rename(&staged, destination).await {
                let _ = tokio::fs::rename(&backup, destination).await;
                let _ = tokio::fs::remove_file(&staged).await;
                return Err(AppError::Io(error));
            }
            self.undo.push(InstallUndo::Restore {
                destination: destination.to_path_buf(),
                backup,
            });
        } else {
            if let Err(error) = tokio::fs::rename(&staged, destination).await {
                let _ = tokio::fs::remove_file(&staged).await;
                return Err(AppError::Io(error));
            }
            self.undo
                .push(InstallUndo::Remove(destination.to_path_buf()));
        }

        Ok(())
    }

    pub async fn commit(mut self) {
        for undo in &self.undo {
            if let InstallUndo::Restore { backup, .. } = undo {
                let _ = tokio::fs::remove_file(backup).await;
            }
        }
        self.committed = true;
    }
}

impl Drop for InstallTransaction {
    fn drop(&mut self) {
        if self.committed {
            return;
        }

        for undo in self.undo.iter().rev() {
            match undo {
                InstallUndo::Remove(path) => {
                    let _ = std::fs::remove_file(path);
                }
                InstallUndo::Restore {
                    destination,
                    backup,
                } => {
                    let _ = std::fs::remove_file(destination);
                    let _ = std::fs::rename(backup, destination);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn install_transaction_rolls_back_created_files_after_collision() {
        let temp = tempfile::tempdir().unwrap();
        let source_one = temp.path().join("source-one.json");
        let source_two = temp.path().join("source-two.json");
        let destination_one = temp.path().join("installed").join("one.json");
        let destination_two = temp.path().join("installed").join("two.json");
        tokio::fs::write(&source_one, b"one").await.unwrap();
        tokio::fs::write(&source_two, b"two").await.unwrap();
        tokio::fs::create_dir_all(destination_two.parent().unwrap())
            .await
            .unwrap();
        tokio::fs::write(&destination_two, b"existing")
            .await
            .unwrap();

        {
            let mut transaction = InstallTransaction::default();
            transaction
                .copy_new(&source_one, &destination_one, false)
                .await
                .unwrap();
            let error = transaction
                .copy_new(&source_two, &destination_two, false)
                .await
                .unwrap_err();
            assert!(matches!(error, AppError::Forbidden(_)));
        }

        assert!(!destination_one.exists());
        assert_eq!(
            tokio::fs::read_to_string(&destination_two).await.unwrap(),
            "existing"
        );
    }

    #[tokio::test]
    async fn test_write_new_atomic_create_new_refusal() {
        let temp = tempfile::tempdir().unwrap();
        let destination = temp.path().join("existing_file.txt");
        tokio::fs::write(&destination, b"already exists").await.unwrap();

        let mut tx = InstallTransaction::default();
        let res = tx.write_new(&destination, b"new content", false).await;
        assert!(matches!(res, Err(AppError::Forbidden(_))));
        assert_eq!(tokio::fs::read_to_string(&destination).await.unwrap(), "already exists");
    }

    #[tokio::test]
    async fn install_transaction_restores_replaced_configuration_on_rollback() {
        let temp = tempfile::tempdir().unwrap();
        let destination = temp.path().join(".agent").join("mcp_config.json");
        tokio::fs::create_dir_all(destination.parent().unwrap())
            .await
            .unwrap();
        tokio::fs::write(&destination, b"original").await.unwrap();

        {
            let mut transaction = InstallTransaction::default();
            transaction
                .replace_atomically(&destination, b"replacement")
                .await
                .unwrap();
            assert_eq!(
                tokio::fs::read_to_string(&destination).await.unwrap(),
                "replacement"
            );
        }

        assert_eq!(
            tokio::fs::read_to_string(&destination).await.unwrap(),
            "original"
        );
    }

    #[tokio::test]
    async fn install_transaction_keeps_files_after_commit() {
        let temp = tempfile::tempdir().unwrap();
        let source = temp.path().join("source.json");
        let destination = temp.path().join("installed").join("source.json");
        tokio::fs::write(&source, b"installed").await.unwrap();

        let mut transaction = InstallTransaction::default();
        transaction.copy_new(&source, &destination, false).await.unwrap();
        transaction.commit().await;

        assert_eq!(
            tokio::fs::read_to_string(&destination).await.unwrap(),
            "installed"
        );
    }
}

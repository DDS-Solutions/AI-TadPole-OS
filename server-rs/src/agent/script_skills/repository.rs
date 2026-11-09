//! @docs ARCHITECTURE:Agent
//!
//! ### AI Assist Note
//! - **Subsystem**: Script skills / Contained artifact repository
//! - Path validation, safe reads, atomic replacement and deletion live at the filesystem boundary.
//!
//! ### 🔍 Debugging & Observability
//! - **Telemetry Targets**: none declared

use crate::error::AppError;
use std::path::{Path, PathBuf};
use tokio::fs;

#[derive(Debug, Clone, Copy)]
pub(super) enum ArtifactKind {
    Skill,
    Workflow,
    Hook,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ArtifactOrigin {
    User,
    Generated,
}

impl ArtifactOrigin {
    pub(super) fn category(self) -> &'static str {
        match self {
            Self::User => "user",
            Self::Generated => "agent_generated",
        }
    }
}

pub(super) struct ArtifactRepository {
    pub(super) workspace_root: PathBuf,
}

impl ArtifactRepository {
    pub(super) fn new(workspace_root: PathBuf) -> Self {
        Self { workspace_root }
    }

    pub(super) fn directory(&self, kind: ArtifactKind, origin: ArtifactOrigin) -> PathBuf {
        if origin == ArtifactOrigin::Generated {
            self.workspace_root
                .join("execution/agent_generated")
                .join(match kind {
                    ArtifactKind::Skill => "skills",
                    ArtifactKind::Workflow => "workflows",
                    ArtifactKind::Hook => "hooks",
                })
        } else {
            self.workspace_root.join(match kind {
                ArtifactKind::Skill => "execution",
                ArtifactKind::Workflow => "directives",
                ArtifactKind::Hook => "hooks",
            })
        }
    }

    pub(super) async fn ensure_directories(&self) -> Result<(), AppError> {
        for kind in [
            ArtifactKind::Skill,
            ArtifactKind::Workflow,
            ArtifactKind::Hook,
        ] {
            for origin in [ArtifactOrigin::User, ArtifactOrigin::Generated] {
                fs::create_dir_all(self.directory(kind, origin))
                    .await
                    .map_err(AppError::Io)?;
            }
        }
        Ok(())
    }

    pub(super) async fn save(
        &self,
        kind: ArtifactKind,
        origin: ArtifactOrigin,
        name: &str,
        content: &[u8],
    ) -> Result<(), AppError> {
        let directory = self.directory(kind, origin);
        let suffix = match kind {
            ArtifactKind::Workflow => "md",
            _ => "json",
        };
        let filename = format!("{}.{}", crate::utils::security::sanitize_id(name), suffix);
        let path = crate::utils::security::validate_path(&directory, &filename)
            .map_err(|error| AppError::InternalServerError(error.to_string()))?;
        write_atomic(&path, content).await
    }

    // Existing deletion is best effort for both writable origins. Keep this
    // contract while moving ownership; changing failure propagation is separate.
    pub(super) async fn delete(&self, kind: ArtifactKind, name: &str) {
        let suffix = match kind {
            ArtifactKind::Workflow => "md",
            _ => "json",
        };
        let filename = format!("{}.{}", crate::utils::security::sanitize_id(name), suffix);
        for origin in [ArtifactOrigin::User, ArtifactOrigin::Generated] {
            if let Ok(path) =
                crate::utils::security::validate_path(&self.directory(kind, origin), &filename)
            {
                let _ = delete_safe(&path).await;
            }
        }
    }
}

pub(super) async fn read_file_safe(path: &Path) -> Result<String, AppError> {
    let meta = fs::symlink_metadata(path)
        .await
        .map_err(|e| AppError::Io(e))?;

    if meta.file_type().is_symlink() {
        return Err(AppError::Forbidden(
            "Security boundary: Symlinks are not allowed".into(),
        ));
    }
    if meta.len() > 1_048_576 {
        return Err(AppError::Forbidden(format!(
            "Security boundary: File size exceeds limit of 1 MiB ({} bytes)",
            meta.len()
        )));
    }

    let content = fs::read_to_string(path)
        .await
        .map_err(|e| AppError::Io(e))?;
    Ok(content)
}

async fn write_atomic(path: &Path, content: &[u8]) -> Result<(), AppError> {
    let parent = path.parent().ok_or_else(|| {
        AppError::InternalServerError("Target path has no parent directory".to_string())
    })?;
    let temp_name = format!("tmp_{}.tmp", uuid::Uuid::new_v4());
    let temp_path = parent.join(temp_name);

    if let Err(e) = fs::write(&temp_path, content).await {
        let _ = fs::remove_file(&temp_path).await;
        return Err(AppError::Io(e));
    }

    if let Err(e) = fs::rename(&temp_path, path).await {
        let _ = fs::remove_file(&temp_path).await;
        return Err(AppError::Io(e));
    }
    Ok(())
}

async fn delete_safe(path: &Path) -> Result<(), AppError> {
    match fs::symlink_metadata(path).await {
        Ok(meta) => {
            if meta.file_type().is_symlink() {
                return Err(AppError::Forbidden(
                    "Security boundary: Symlinks are not allowed".into(),
                ));
            }
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            return Ok(());
        }
        Err(e) => return Err(AppError::Io(e)),
    }

    match fs::remove_file(path).await {
        Ok(_) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(AppError::Io(e)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn save_and_delete_are_scoped_to_artifact_origins() {
        let temp = tempfile::tempdir().unwrap();
        let repository = ArtifactRepository::new(temp.path().to_path_buf());
        repository.ensure_directories().await.unwrap();
        repository
            .save(
                ArtifactKind::Skill,
                ArtifactOrigin::Generated,
                "demo",
                b"{}",
            )
            .await
            .unwrap();
        let path = repository
            .directory(ArtifactKind::Skill, ArtifactOrigin::Generated)
            .join("demo.json");
        assert_eq!(read_file_safe(&path).await.unwrap(), "{}");
        repository.delete(ArtifactKind::Skill, "demo").await;
        assert!(!path.exists());
        assert!(temp.path().join("execution").is_dir());
    }
    #[tokio::test]
    async fn oversized_artifacts_are_rejected_before_reading() {
        let temp = tempfile::tempdir().unwrap();
        let file = temp.path().join("large.json");
        fs::write(&file, vec![b'x'; 1_048_577]).await.unwrap();
        assert!(matches!(
            read_file_safe(&file).await,
            Err(AppError::Forbidden(_))
        ));
    }
}

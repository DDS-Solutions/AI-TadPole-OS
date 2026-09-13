//! @docs ARCHITECTURE:Networking
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / HTTP Routes / templates / source
//! - **Primary Entrypoints**: `clone_template_repository`, `cloned_revision`, `validate_git_ref`
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Public URL validation, DNS pinning, `kill_on_drop`, and `git_ref` character validation.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: `AppError::BadRequest`, `AppError::InternalServerError`
//! - **Telemetry Targets**: none declared
//! - **Witness Tests**: `routes::templates::source::tests::*`

use crate::error::AppError;
use std::path::Path;
use std::time::Duration;

pub const GIT_CLONE_TIMEOUT: Duration = Duration::from_secs(60);
pub const GIT_REV_PARSE_TIMEOUT: Duration = Duration::from_secs(10);

/// Validates that a git_ref is a safe branch/tag/commit identifier and does not start with leading '-' flags.
pub fn validate_git_ref(git_ref: &str) -> Result<&str, AppError> {
    let trimmed = git_ref.trim();
    if trimmed.starts_with('-')
        || trimmed.is_empty()
        || !trimmed
            .chars()
            .all(|c| c.is_alphanumeric() || c == '.' || c == '-' || c == '_' || c == '/')
    {
        return Err(AppError::BadRequest(format!(
            "Invalid git_ref '{}'",
            super::naming::sanitize_error_str(git_ref)
        )));
    }
    Ok(trimmed)
}

pub async fn clone_template_repository(
    repository_url: &str,
    git_ref: Option<&str>,
    target_dir: &Path,
) -> Result<(), AppError> {
    let validated_target = crate::utils::security::validate_public_http_url(repository_url).await?;

    let resolve_arg = format!(
        "http.curloptResolve={}:{}:{}",
        validated_target.host, validated_target.port, validated_target.ip
    );

    let mut git_cmd = tokio::process::Command::new("git");
    git_cmd.kill_on_drop(true);
    if cfg!(target_os = "windows") {
        git_cmd.arg("-c").arg("http.sslBackend=schannel");
    }
    git_cmd
        .arg("-c")
        .arg(&resolve_arg)
        .arg("clone")
        .arg("--depth")
        .arg("1");

    if let Some(git_ref_val) = git_ref {
        let safe_ref = validate_git_ref(git_ref_val)?;
        git_cmd.arg("--branch").arg(safe_ref);
    }

    let output_fut = git_cmd.arg(repository_url).arg(target_dir).output();

    let output = match tokio::time::timeout(GIT_CLONE_TIMEOUT, output_fut).await {
        Ok(Ok(o)) => o,
        Ok(Err(e)) => {
            return Err(AppError::InternalServerError(format!(
                "Failed to execute git: {}",
                e
            )));
        }
        Err(_) => {
            return Err(AppError::InternalServerError(
                "Git clone timed out after 60 seconds".to_string(),
            ));
        }
    };

    if !output.status.success() {
        let stderr_str = String::from_utf8_lossy(&output.stderr);
        tracing::warn!(
            "⚠️ [Templates] Git clone failed: {}",
            super::naming::sanitize_error_str(&stderr_str)
        );
        return Err(AppError::InternalServerError(
            "Failed to clone template repository".to_string(),
        ));
    }

    Ok(())
}

pub async fn cloned_revision(repository_root: &Path) -> Option<String> {
    let mut command = tokio::process::Command::new("git");
    command
        .kill_on_drop(true)
        .arg("-C")
        .arg(repository_root)
        .arg("rev-parse")
        .arg("HEAD");
    let output = tokio::time::timeout(GIT_REV_PARSE_TIMEOUT, command.output())
        .await
        .ok()?
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let revision = String::from_utf8(output.stdout).ok()?;
    let revision = revision.trim();
    (!revision.is_empty()).then(|| revision.to_string())
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    #[test]
    fn test_validate_git_ref_safe_and_unsafe() {
        assert_eq!(validate_git_ref("main").unwrap(), "main");
        assert_eq!(validate_git_ref("v1.0.0").unwrap(), "v1.0.0");
        assert_eq!(
            validate_git_ref("feature/templates_v2").unwrap(),
            "feature/templates_v2"
        );
        assert_eq!(
            validate_git_ref("release-2026.08").unwrap(),
            "release-2026.08"
        );

        assert!(validate_git_ref("--upload-pack=evil").is_err());
        assert!(validate_git_ref("-b").is_err());
        assert!(validate_git_ref("").is_err());
        assert!(validate_git_ref("   ").is_err());
        assert!(validate_git_ref("ref;rm -rf /").is_err());
    }
}

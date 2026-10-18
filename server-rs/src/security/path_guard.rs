//! @docs ARCHITECTURE:Security:PathGuard
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Security / PathGuard
//! - **Primary Entrypoints**: `validate_path`, `SafePath`
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` SafePath is a non-forgeable wrapper around a validated path.
//! - `[Structural]` Prevents directory traversal, symlink escapes, UNC bypasses, and device file access.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: `AppError`
//! - **Telemetry Targets**: none declared
//! - **Witness Tests**: none declared

use crate::error::AppError;
use std::path::{Path, PathBuf};

/// A non-forgeable wrapper around a validated path.
/// Can only be created through successful validation or explicit trusted sources.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SafePath(pub(crate) PathBuf);

impl SafePath {
    pub fn from_trusted(p: PathBuf) -> Self {
        Self(p)
    }
    pub fn as_path(&self) -> &Path {
        &self.0
    }
    #[allow(dead_code)]
    pub fn to_path_buf(&self) -> PathBuf {
        self.0.clone()
    }
}

impl AsRef<Path> for SafePath {
    fn as_ref(&self) -> &Path {
        &self.0
    }
}

impl std::ops::Deref for SafePath {
    type Target = Path;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub(crate) fn is_device_file(path: &Path) -> bool {
    let reserved = [
        "CON", "PRN", "AUX", "NUL", "COM0", "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7",
        "COM8", "COM9", "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9",
    ];
    for component in path.components() {
        if let std::path::Component::Normal(os_str) = component {
            let name_str = os_str.to_string_lossy().to_uppercase();
            let base_name = match name_str.split_once('.') {
                Some((base, _ext)) => base,
                None => &name_str,
            };
            if reserved.contains(&base_name) {
                return true;
            }
        }
    }
    let path_str = path.to_string_lossy();
    if path_str.starts_with("/dev/")
        || path_str.starts_with("/proc/")
        || path_str.starts_with("/sys/")
        || path_str.starts_with("/run/")
    {
        return true;
    }
    false
}

pub(crate) fn normalize_lexical(p: &Path) -> PathBuf {
    let mut components = Vec::new();
    for component in p.components() {
        match component {
            std::path::Component::Prefix(prefix) => {
                components.push(std::path::Component::Prefix(prefix))
            }
            std::path::Component::RootDir => components.push(std::path::Component::RootDir),
            std::path::Component::CurDir => {}
            std::path::Component::ParentDir => {
                if let Some(std::path::Component::Normal(_)) = components.last() {
                    components.pop();
                }
            }
            std::path::Component::Normal(c) => components.push(std::path::Component::Normal(c)),
        }
    }
    components.iter().collect()
}

pub(crate) fn strip_unc_prefix(p: &Path) -> PathBuf {
    let p_str = p.to_string_lossy();
    if let Some(stripped) = p_str.strip_prefix(r"\\?\") {
        PathBuf::from(stripped)
    } else {
        p.to_path_buf()
    }
}

/// Sanitizes and validates a path to prevent directory traversal and symlink follow bypasses (S-008).
pub fn validate_path(base: &Path, user_path: &str) -> Result<SafePath, AppError> {
    if user_path.contains('\0') {
        return Err(AppError::Forbidden(
            "Null byte detected in path".to_string(),
        ));
    }

    // 1. Canonicalize the base directory (it must exist)
    let base_canon = std::fs::canonicalize(base)
        .map_err(|e| AppError::BadRequest(format!("Invalid base path: {}", e)))?;
    let base_canon_stripped = strip_unc_prefix(&base_canon);

    // 2. Normalize and join the user path
    let joined = base_canon.join(user_path);

    // 3. Resolve symlinks using std::fs::canonicalize if the path or parent exists
    let resolved = if joined.exists() {
        std::fs::canonicalize(&joined)
            .map_err(|e| AppError::BadRequest(format!("Invalid path resolution: {}", e)))?
    } else {
        let mut parent = joined.as_path();
        let mut depth = 0;
        const MAX_PARENT_DEPTH: usize = 32;
        while let Some(p) = parent.parent() {
            depth += 1;
            if depth > MAX_PARENT_DEPTH {
                return Err(AppError::Forbidden(
                    "Path structure is too deep".to_string(),
                ));
            }
            if p.exists() {
                parent = p;
                break;
            }
            parent = p;
        }
        if parent.exists() {
            let parent_canon = std::fs::canonicalize(parent)
                .map_err(|e| AppError::BadRequest(format!("Invalid parent resolution: {}", e)))?;
            let parent_canon_stripped = strip_unc_prefix(&parent_canon);
            if !parent_canon_stripped.starts_with(&base_canon_stripped) {
                return Err(AppError::Forbidden(
                    "Path traversal detected: parent outside authorized base".to_string(),
                ));
            }
            let relative = joined.strip_prefix(parent).unwrap_or(Path::new(""));
            parent_canon.join(relative)
        } else {
            normalize_lexical(&joined)
        }
    };

    let resolved_norm = normalize_lexical(&resolved);
    let resolved_norm_stripped = strip_unc_prefix(&resolved_norm);
    if !resolved_norm_stripped.starts_with(&base_canon_stripped) {
        return Err(AppError::Forbidden(
            "Path traversal detected: outside authorized base".to_string(),
        ));
    }

    if is_device_file(&resolved_norm) {
        return Err(AppError::Forbidden(
            "Access denied: device files or system directories cannot be accessed".to_string(),
        ));
    }

    Ok(SafePath(resolved_norm))
}

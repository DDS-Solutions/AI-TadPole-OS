//! @docs ARCHITECTURE:Intelligence
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Intelligence / Path Utilities
//! - **Primary Entrypoints**: `to_unix_path`, `sanitize_log_path`, `derive_stable_salt`, `obfuscate_path`
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none declared
//! - **Telemetry Targets**: none declared
//! - **Witness Tests**: none declared

use super::models::GraphError;
use sha2::{Digest, Sha256};
use std::path::Path;

pub fn to_unix_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

pub fn sanitize_log_path(path: &Path, root: &Path) -> String {
    if let Ok(rel) = path.strip_prefix(root) {
        rel.to_string_lossy().replace('\\', "/")
    } else {
        let filename = path
            .file_name()
            .and_then(|f| f.to_str())
            .unwrap_or("unknown_file");
        format!("<redacted>/{}", filename)
    }
}

/// Helper to derive a stable obfuscation salt based on the workspace root directory.
/// Fallback to environment variable TADPOLE_GRAPH_SALT if defined.
pub fn derive_stable_salt(base_dir: &Path) -> String {
    if let Ok(salt) = std::env::var("TADPOLE_GRAPH_SALT") {
        if salt.len() >= 4 {
            return salt;
        }
    }
    let mut hasher = Sha256::new();
    hasher.update(base_dir.to_string_lossy().as_bytes());
    let hex_hash = hex::encode(hasher.finalize());
    hex_hash[..32].to_string()
}

/// Helper to obfuscate physical file path structures deterministically
/// while preserving UX force-graph clustering and file basenames.
pub fn obfuscate_path(path_str: &str, salt: &str) -> Result<String, GraphError> {
    if salt.len() < 4 {
        return Err(GraphError::KeyNormalization(format!(
            "Salt is too short: got {} bytes, minimum is 4 bytes",
            salt.len()
        )));
    }
    let path = Path::new(path_str);
    let file_name = path
        .file_name()
        .and_then(|f| f.to_str())
        .ok_or_else(|| GraphError::PathOutOfBounds(format!("Invalid file path: {}", path_str)))?;
    let parent = path.parent().ok_or_else(|| {
        GraphError::PathOutOfBounds(format!("Path has no parent structure: {}", path_str))
    })?;
    let parent_str = parent.to_string_lossy();

    if parent_str.is_empty() {
        Ok(file_name.to_string())
    } else {
        let mut hasher = Sha256::new();
        hasher.update(salt.as_bytes());
        hasher.update(b":");
        hasher.update(parent_str.as_bytes());
        let result = hasher.finalize();
        let hash_val = hex::encode(result);
        let obf_prefix = hash_val.get(..16).unwrap_or(&hash_val);
        Ok(format!("{}/{}", obf_prefix, file_name))
    }
}

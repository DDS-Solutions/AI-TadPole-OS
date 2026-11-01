//! @docs ARCHITECTURE:Intelligence
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Intelligence / Code Parsing
//! - **Primary Entrypoints**: `CodeParser`, `CodeParsingService`
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none declared
//! - **Telemetry Targets**: none declared
//! - **Witness Tests**: none declared

use super::models::{GraphError, MAX_FILE_SIZE_BYTES};
use super::path_utils::{sanitize_log_path, to_unix_path};
use crate::utils::parser::SymbolExtractor;
use rayon::prelude::*;
use std::path::{Path, PathBuf};

/// Service trait to perform file parsing
pub trait CodeParser: Send + Sync {
    fn parse_files(
        &self,
        files: &[PathBuf],
        root: &Path,
    ) -> Result<
        Vec<(
            PathBuf,
            String,
            Option<(
                Vec<crate::utils::parser::Symbol>,
                Vec<crate::utils::parser::Reference>,
                std::time::SystemTime,
                u64,
            )>,
        )>,
        GraphError,
    >;
}

/// Default implementation of the parsing service
pub struct CodeParsingService;

impl CodeParser for CodeParsingService {
    fn parse_files(
        &self,
        files: &[PathBuf],
        root: &Path,
    ) -> Result<
        Vec<(
            PathBuf,
            String,
            Option<(
                Vec<crate::utils::parser::Symbol>,
                Vec<crate::utils::parser::Reference>,
                std::time::SystemTime,
                u64,
            )>,
        )>,
        GraphError,
    > {
        let canonical_root = root.canonicalize().map_err(|e| {
            GraphError::WorkspaceRootNotFound(format!(
                "Failed to canonicalize root {}: {}",
                root.display(),
                e
            ))
        })?;

        let updates: Vec<(
            PathBuf,
            String,
            Option<(
                Vec<crate::utils::parser::Symbol>,
                Vec<crate::utils::parser::Reference>,
                std::time::SystemTime,
                u64,
            )>,
        )> = files
            .par_iter()
            .map_init(SymbolExtractor::new, |extractor, path| {
                // 🛡️ Time-check boundary verification before reading (Mitigates symlink TOCTOU races)
                let canonical_path = match path.canonicalize() {
                    Ok(p) => p,
                    Err(e) => {
                        tracing::warn!(
                            "⚠️ [Graph] Skipping unreadable or missing file {:?}: {}",
                            path,
                            e
                        );
                        return (path.clone(), to_unix_path(path), None);
                    }
                };

                if !canonical_path.starts_with(&canonical_root) {
                    tracing::warn!(
                        "🛡️ [Graph] Security Violation: Path {} points outside workspace root {}",
                        sanitize_log_path(&canonical_path, root),
                        sanitize_log_path(&canonical_root, root)
                    );
                    return (path.clone(), to_unix_path(path), None);
                }

                let rel_path = match path.strip_prefix(root) {
                    Ok(rel) => to_unix_path(rel),
                    Err(_) => {
                        tracing::warn!(
                            "⚠️ [Graph] Path integrity lost: {} is not inside root {}",
                            path.display(),
                            root.display()
                        );
                        return (path.clone(), to_unix_path(path), None);
                    }
                };

                let metadata = match std::fs::metadata(path) {
                    Ok(m) => m,
                    Err(e) => {
                        tracing::warn!(
                            "⚠️ [Graph] Failed to read metadata for {}: {}",
                            path.display(),
                            e
                        );
                        return (path.clone(), rel_path, None);
                    }
                };

                let file_size = metadata.len();
                if file_size > MAX_FILE_SIZE_BYTES {
                    tracing::warn!(
                        "⚠️ [Graph] Skipping file {} because its size ({} bytes) exceeds the {}MB limit.",
                        path.display(),
                        file_size,
                        MAX_FILE_SIZE_BYTES / (1024 * 1024)
                    );
                    let mtime = metadata.modified().unwrap_or(std::time::SystemTime::now());
                    return (
                        path.clone(),
                        rel_path,
                        Some((Vec::new(), Vec::new(), mtime, file_size)),
                    );
                }

                match std::fs::read_to_string(path) {
                    Ok(content) => {
                        let symbols = extractor.extract_symbols(path, &content);
                        let refs = extractor.extract_references(path, &content);
                        let mtime = metadata.modified().unwrap_or(std::time::SystemTime::now());
                        (
                            path.clone(),
                            rel_path,
                            Some((symbols, refs, mtime, file_size)),
                        )
                    }
                    Err(e) => {
                        tracing::warn!(
                            "⚠️ [Graph] Failed to read file content {}: {}",
                            path.display(),
                            e
                        );
                        (path.clone(), rel_path, None)
                    }
                }
            })
            .collect();

        Ok(updates)
    }
}

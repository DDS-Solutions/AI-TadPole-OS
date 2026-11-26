//! @docs ARCHITECTURE:Intelligence
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Intelligence / Cache Management
//! - **Primary Entrypoints**: `CacheManager`, `CacheManagementService`
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none declared
//! - **Telemetry Targets**: none declared
//! - **Witness Tests**: none declared

use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Service trait to manage changed/deleted state checking
pub trait CacheManager: Send + Sync {
    fn check_changes(
        &self,
        files: &[PathBuf],
        metadata: &HashMap<PathBuf, (std::time::SystemTime, u64)>,
        root: &Path,
    ) -> (Vec<PathBuf>, Vec<PathBuf>); // (files_to_parse, deleted_paths)
}

/// Default implementation of the cache management service
pub struct CacheManagementService;

impl CacheManager for CacheManagementService {
    fn check_changes(
        &self,
        files: &[PathBuf],
        metadata: &HashMap<PathBuf, (std::time::SystemTime, u64)>,
        _root: &Path,
    ) -> (Vec<PathBuf>, Vec<PathBuf>) {
        let active_paths: std::collections::HashSet<&PathBuf> = files.iter().collect();
        let deleted_paths: Vec<PathBuf> = metadata
            .keys()
            .filter(|p| !active_paths.contains(p))
            .cloned()
            .collect();

        let mut files_to_parse = Vec::new();
        for path in files {
            let mut needs_parse = true;
            if let Ok(m) = std::fs::metadata(path) {
                if let (Ok(mtime), size) = (m.modified(), m.len()) {
                    if let Some(&(cached_mtime, cached_size)) = metadata.get(path) {
                        if cached_mtime == mtime && cached_size == size {
                            needs_parse = false;
                        }
                    }
                }
            }
            if needs_parse {
                files_to_parse.push(path.clone());
            }
        }

        (files_to_parse, deleted_paths)
    }
}

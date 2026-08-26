//! @docs ARCHITECTURE:Registry
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Agent Runner / cache
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Type-safe state handling and bounded execution without unhandled panics.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: none declared
//! - **Witness Tests**: none declared

use crate::agent::types::TokenUsage;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

#[derive(Clone, Debug)]
pub struct CacheValue {
    pub output: String,
    pub usage: Option<TokenUsage>,
    pub file_path: Option<PathBuf>,
    pub mtime: Option<SystemTime>,
    pub size: u64,
}

pub struct SharedToolCache {
    cache: HashMap<(String, String, String), CacheValue>,
}

impl SharedToolCache {
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
        }
    }

    pub fn get(
        &mut self,
        tool_name: &str,
        args_str: &str,
        workspace_root: &str,
    ) -> Option<(String, Option<TokenUsage>)> {
        let key = (
            tool_name.to_string(),
            args_str.to_string(),
            workspace_root.to_string(),
        );

        if let Some(val) = self.cache.get(&key) {
            if let Some(ref path) = val.file_path {
                if let Ok(metadata) = std::fs::metadata(path) {
                    let current_mtime = metadata.modified().ok();
                    let current_size = metadata.len();

                    if val.mtime != current_mtime || val.size != current_size {
                        self.cache.remove(&key);
                        return None;
                    }
                } else {
                    self.cache.remove(&key);
                    return None;
                }
            }
            return Some((val.output.clone(), val.usage.clone()));
        }
        None
    }

    pub fn insert(
        &mut self,
        tool_name: &str,
        args_str: &str,
        workspace_root: &str,
        output: String,
        usage: Option<TokenUsage>,
        file_path: Option<PathBuf>,
    ) {
        let key = (
            tool_name.to_string(),
            args_str.to_string(),
            workspace_root.to_string(),
        );
        let mut mtime = None;
        let mut size = 0;

        if let Some(ref path) = file_path {
            if let Ok(metadata) = std::fs::metadata(path) {
                mtime = metadata.modified().ok();
                size = metadata.len();
            }
        }

        self.cache.insert(
            key,
            CacheValue {
                output,
                usage,
                file_path,
                mtime,
                size,
            },
        );
    }

    pub fn invalidate_path(&mut self, path: &Path) {
        let keys_to_remove: Vec<_> = self
            .cache
            .iter()
            .filter(|(_, val)| val.file_path.as_ref().map(|p| p == path).unwrap_or(false))
            .map(|(k, _)| k.clone())
            .collect();

        for k in keys_to_remove {
            self.cache.remove(&k);
        }
    }

    pub fn clear(&mut self) {
        self.cache.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn test_shared_tool_cache_basic() {
        let mut cache = SharedToolCache::new();
        let tool = "read_file";
        let args = r#"{"path": "test.txt"}"#;
        let root = "ws";

        assert!(cache.get(tool, args, root).is_none());

        cache.insert(tool, args, root, "file content".to_string(), None, None);

        let hit = cache.get(tool, args, root);
        assert!(hit.is_some());
        assert_eq!(hit.unwrap().0, "file content");
    }

    #[test]
    fn test_shared_tool_cache_mtime_invalidation() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.txt");

        let mut file = File::create(&file_path).unwrap();
        write!(file, "hello").unwrap();

        let mut cache = SharedToolCache::new();
        let tool = "read_file";
        let args = r#"{"path": "test.txt"}"#;
        let root = dir.path().to_string_lossy().to_string();

        cache.insert(
            tool,
            args,
            &root,
            "hello".to_string(),
            None,
            Some(file_path.clone()),
        );

        let hit = cache.get(tool, args, &root);
        assert!(hit.is_some());

        // Modify file
        std::thread::sleep(std::time::Duration::from_millis(15));
        let mut file = File::create(&file_path).unwrap();
        write!(file, "hello modified").unwrap();

        let hit_after_mod = cache.get(tool, args, &root);
        assert!(hit_after_mod.is_none());
    }

    #[test]
    fn test_shared_tool_cache_invalidate_path() {
        let mut cache = SharedToolCache::new();
        let tool = "read_file";
        let args = r#"{"path": "test.txt"}"#;
        let root = "ws";
        let path = PathBuf::from("ws/test.txt");

        cache.insert(
            tool,
            args,
            root,
            "hello".to_string(),
            None,
            Some(path.clone()),
        );

        cache.invalidate_path(&path);

        assert!(cache.get(tool, args, root).is_none());
    }
}

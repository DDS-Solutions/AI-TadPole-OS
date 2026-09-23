//! @docs ARCHITECTURE:Agent:Blackboard
//!
//! ### AI Assist Note
//! **Swarm Shared Blackboard Engine**: Provides high-performance, thread-safe,
//! in-memory key-value scratchpad storage for multi-agent missions. Decouples
//! large data exchanges from prompt context histories, slashing input token consumption.
//! Features O(1) Arc pointer sharing, generic tag collection, and UTF-8 safe truncation.
//!
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: Key collision, mission isolation breach, or stale read/write races.
//! - **Telemetry Link**: Search `[blackboard]` in tracing logs.

use chrono::{DateTime, Utc};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::debug;

pub const SUMMARY_TRUNCATE_TO: usize = 57;
pub const SUMMARY_TRUNCATE_AT: usize = 60;
pub const MAX_SUMMARY_ENTRIES: usize = 50;

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum BlackboardError {
    #[error("Mission '{0}' not found on blackboard")]
    MissionNotFound(String),
    #[error("Key '{key}' version conflict: expected v{expected}, but found v{actual}")]
    VersionConflict {
        key: String,
        expected: u64,
        actual: u64,
    },
}

/// An individual entry on the shared blackboard.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlackboardEntry {
    pub key: String,
    pub value: serde_json::Value,
    pub author_agent_id: String,
    pub tags: Vec<String>,
    pub version: u64,
    pub updated_at: DateTime<Utc>,
}

pub type MissionBoard = Arc<DashMap<String, Arc<BlackboardEntry>>>;

/// Swarm-wide shared blackboard partitioned by `mission_id`.
#[derive(Debug, Clone, Default)]
pub struct SharedBlackboard {
    // mission_id -> (key -> Arc<BlackboardEntry>)
    missions: Arc<DashMap<String, MissionBoard>>,
}

impl SharedBlackboard {
    pub fn new() -> Self {
        Self {
            missions: Arc::new(DashMap::new()),
        }
    }

    /// Fast-paths mission board retrieval or initializes a new board on first write.
    fn get_or_create_board(&self, mission_id: &str) -> MissionBoard {
        if let Some(board) = self.missions.get(mission_id) {
            board.clone()
        } else {
            self.missions
                .entry(mission_id.to_string())
                .or_insert_with(|| Arc::new(DashMap::new()))
                .clone()
        }
    }

    /// Sets or updates a key-value entry on the mission blackboard with flexible tag collection.
    pub fn set<T, S>(
        &self,
        mission_id: &str,
        key: &str,
        value: serde_json::Value,
        author_agent_id: &str,
        tags: T,
    ) -> Arc<BlackboardEntry>
    where
        T: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let mission_map = self.get_or_create_board(mission_id);
        let tags_vec: Vec<String> = tags.into_iter().map(Into::into).collect();
        let now = Utc::now();

        let mut map_entry = mission_map.entry(key.to_string());
        let entry = match map_entry {
            dashmap::mapref::entry::Entry::Occupied(ref mut occ) => {
                let next_version = occ.get().version + 1;
                let new_entry = Arc::new(BlackboardEntry {
                    key: key.to_string(),
                    value,
                    author_agent_id: author_agent_id.to_string(),
                    tags: tags_vec,
                    version: next_version,
                    updated_at: now,
                });
                occ.insert(new_entry.clone());
                new_entry
            }
            dashmap::mapref::entry::Entry::Vacant(vac) => {
                let new_entry = Arc::new(BlackboardEntry {
                    key: key.to_string(),
                    value,
                    author_agent_id: author_agent_id.to_string(),
                    tags: tags_vec,
                    version: 1,
                    updated_at: now,
                });
                vac.insert(new_entry.clone());
                new_entry
            }
        };

        debug!(
            target: "blackboard",
            mission = %mission_id,
            key = %key,
            version = entry.version,
            agent = %author_agent_id,
            "entry updated"
        );
        entry
    }

    /// Optimistic Concurrency Control (OCC): Compare-and-Swap an entry on the blackboard.
    ///
    /// - If `expected_version == 0`, key MUST NOT exist yet (insert v1).
    /// - If `expected_version > 0`, key MUST exist at that exact version, bumped to `expected_version + 1`.
    /// - If version mismatches, returns `BlackboardError::VersionConflict`.
    pub fn compare_and_swap<T, S>(
        &self,
        mission_id: &str,
        key: &str,
        expected_version: u64,
        value: serde_json::Value,
        author_agent_id: &str,
        tags: T,
    ) -> Result<Arc<BlackboardEntry>, BlackboardError>
    where
        T: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let mission_map = self.get_or_create_board(mission_id);
        let tags_vec: Vec<String> = tags.into_iter().map(Into::into).collect();
        let now = Utc::now();

        let mut map_entry = mission_map.entry(key.to_string());
        match map_entry {
            dashmap::mapref::entry::Entry::Occupied(ref mut occ) => {
                let current_version = occ.get().version;
                if current_version != expected_version {
                    return Err(BlackboardError::VersionConflict {
                        key: key.to_string(),
                        expected: expected_version,
                        actual: current_version,
                    });
                }
                let next_version = current_version + 1;
                let new_entry = Arc::new(BlackboardEntry {
                    key: key.to_string(),
                    value,
                    author_agent_id: author_agent_id.to_string(),
                    tags: tags_vec,
                    version: next_version,
                    updated_at: now,
                });
                occ.insert(new_entry.clone());
                debug!(
                    target: "blackboard",
                    mission = %mission_id,
                    key = %key,
                    version = next_version,
                    agent = %author_agent_id,
                    "cas entry updated"
                );
                Ok(new_entry)
            }
            dashmap::mapref::entry::Entry::Vacant(vac) => {
                if expected_version != 0 {
                    return Err(BlackboardError::VersionConflict {
                        key: key.to_string(),
                        expected: expected_version,
                        actual: 0,
                    });
                }
                let new_entry = Arc::new(BlackboardEntry {
                    key: key.to_string(),
                    value,
                    author_agent_id: author_agent_id.to_string(),
                    tags: tags_vec,
                    version: 1,
                    updated_at: now,
                });
                vac.insert(new_entry.clone());
                debug!(
                    target: "blackboard",
                    mission = %mission_id,
                    key = %key,
                    version = 1,
                    agent = %author_agent_id,
                    "cas initial entry created"
                );
                Ok(new_entry)
            }
        }
    }

    /// Retrieves an entry by key from a mission blackboard (O(1) Arc pointer clone).
    pub fn get(&self, mission_id: &str, key: &str) -> Option<Arc<BlackboardEntry>> {
        let mission_map = self.missions.get(mission_id)?;
        mission_map.get(key).map(|v| Arc::clone(v.value()))
    }

    /// Checks if a key exists on the mission blackboard.
    pub fn contains_key(&self, mission_id: &str, key: &str) -> bool {
        let Some(mission_map) = self.missions.get(mission_id) else {
            return false;
        };
        mission_map.contains_key(key)
    }

    /// Removes a key from the mission blackboard. Returns true if key was present.
    pub fn remove_key(&self, mission_id: &str, key: &str) -> bool {
        let Some(mission_map) = self.missions.get(mission_id) else {
            return false;
        };
        mission_map.remove(key).is_some()
    }

    /// Returns the number of entries on a mission blackboard.
    pub fn len(&self, mission_id: &str) -> usize {
        let Some(mission_map) = self.missions.get(mission_id) else {
            return 0;
        };
        mission_map.len()
    }

    /// Checks if a mission blackboard is empty.
    pub fn is_empty(&self, mission_id: &str) -> bool {
        self.len(mission_id) == 0
    }

    /// Lists all entries matching an optional tag filter.
    pub fn list(&self, mission_id: &str, tag_filter: Option<&str>) -> Vec<Arc<BlackboardEntry>> {
        let Some(mission_map) = self.missions.get(mission_id) else {
            return Vec::new();
        };

        let mut results: Vec<Arc<BlackboardEntry>> = mission_map
            .iter()
            .filter_map(|entry| {
                if let Some(tag) = tag_filter {
                    if entry.tags.iter().any(|t| t.eq_ignore_ascii_case(tag)) {
                        Some(Arc::clone(entry.value()))
                    } else {
                        None
                    }
                } else {
                    Some(Arc::clone(entry.value()))
                }
            })
            .collect();

        // Deterministic ordering by key
        results.sort_unstable_by(|a, b| a.key.cmp(&b.key));
        results
    }

    /// Generates a compact, deterministic Markdown summary of keys on the blackboard for prompt context injection.
    /// Defuses backticks to prevent markdown prompt-injection, bounds keys and total entries to prevent context blowup,
    /// and sorts deterministically to maximize provider-side prompt cache hits.
    pub fn export_summary(&self, mission_id: &str) -> String {
        let Some(mission_map) = self.missions.get(mission_id) else {
            return String::new();
        };

        if mission_map.is_empty() {
            return String::new();
        }

        // Snapshot and sort deterministically by key
        let mut entries: Vec<Arc<BlackboardEntry>> =
            mission_map.iter().map(|e| Arc::clone(e.value())).collect();
        entries.sort_unstable_by(|a, b| a.key.cmp(&b.key));
        let total_count = entries.len();
        entries.truncate(MAX_SUMMARY_ENTRIES);

        let mut lines = Vec::with_capacity(entries.len() + 2);
        lines.push("### 📋 Shared Mission Blackboard:".to_string());

        for entry in entries {
            let defused_key = entry.key.replace('`', "'");
            let defused_author = entry.author_agent_id.replace('`', "'");

            let val_summary = match &entry.value {
                serde_json::Value::String(s) => {
                    let char_count = s.chars().count();
                    let defused = s.replace('`', "'");
                    if char_count > SUMMARY_TRUNCATE_AT {
                        let truncated: String = defused.chars().take(SUMMARY_TRUNCATE_TO).collect();
                        format!("\"{}...\"", truncated)
                    } else {
                        format!("\"{}\"", defused)
                    }
                }
                serde_json::Value::Array(arr) => format!("[{} items]", arr.len()),
                serde_json::Value::Object(obj) => {
                    let mut keys: Vec<&str> = obj.keys().map(String::as_str).collect();
                    keys.sort_unstable();
                    if keys.len() > 10 {
                        let preview = keys.iter().take(8).copied().collect::<Vec<_>>().join(", ");
                        format!("{{{}, ... +{} keys}}", preview, keys.len() - 8)
                    } else {
                        format!("{{{}}}", keys.join(", "))
                    }
                }
                other => other.to_string(),
            };

            lines.push(format!(
                "- **`{}`** (v{}, by `{}`): {}",
                defused_key, entry.version, defused_author, val_summary
            ));
        }

        if total_count > MAX_SUMMARY_ENTRIES {
            lines.push(format!(
                "... (and {} more keys omitted for prompt bounds)",
                total_count - MAX_SUMMARY_ENTRIES
            ));
        }

        lines.join("\n")
    }

    /// Clears blackboard entries for a terminated mission to prevent memory accumulation.
    /// Note: Callers should ensure the mission is terminated before clearing.
    pub fn clear_mission(&self, mission_id: &str) {
        self.missions.remove(mission_id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_blackboard_concurrency_and_versioning() {
        let bb = SharedBlackboard::new();
        let m_id = "mission_swarm_001";

        let e1 = bb.set(
            m_id,
            "target_files",
            serde_json::json!(["src/main.rs", "src/lib.rs"]),
            "agent_alpha",
            vec!["ast"],
        );
        assert_eq!(e1.version, 1);

        let e2 = bb.set(
            m_id,
            "target_files",
            serde_json::json!(["src/main.rs", "src/lib.rs", "src/db.rs"]),
            "agent_beta",
            vec!["ast", "expanded"],
        );
        assert_eq!(e2.version, 2);

        let fetched = bb.get(m_id, "target_files").unwrap();
        assert_eq!(fetched.version, 2);
        assert_eq!(fetched.author_agent_id, "agent_beta");

        let ast_items = bb.list(m_id, Some("ast"));
        assert_eq!(ast_items.len(), 1);

        let summary = bb.export_summary(m_id);
        assert!(summary.contains("target_files"));
        assert!(summary.contains("agent_beta"));
    }

    #[test]
    fn test_blackboard_utf8_multibyte_truncation_safety() {
        let bb = SharedBlackboard::new();
        let m_id = "mission_utf8_test";

        // Multi-byte characters: emojis (4 bytes each) and CJK characters (3 bytes each)
        let complex_unicode = "🦀⚡🚀🎯🔥 Sovereign Reality Tadpole OS: 日本語と絵文字のテスト文字列が正しく処理されることを確認します。";
        bb.set(
            m_id,
            "unicode_key",
            serde_json::json!(complex_unicode),
            "agent_alpha",
            ["unicode"],
        );

        // export_summary must not panic on multi-byte boundaries
        let summary = bb.export_summary(m_id);
        assert!(summary.contains("unicode_key"));
        assert!(summary.contains("..."));
    }

    #[test]
    fn test_blackboard_clear_and_isolation() {
        let bb = SharedBlackboard::new();
        bb.set(
            "m_1",
            "k1",
            serde_json::json!("v1"),
            "agent_1",
            Vec::<String>::new(),
        );
        bb.set(
            "m_2",
            "k2",
            serde_json::json!("v2"),
            "agent_2",
            Vec::<String>::new(),
        );

        assert!(bb.get("m_1", "k1").is_some());
        assert!(bb.get("m_2", "k2").is_some());

        // Clear mission 1 only
        bb.clear_mission("m_1");

        assert!(bb.get("m_1", "k1").is_none());
        assert!(
            bb.get("m_2", "k2").is_some(),
            "Mission 2 entries must remain unaffected"
        );
    }

    #[tokio::test]
    async fn test_blackboard_concurrent_versioning() {
        let bb = Arc::new(SharedBlackboard::new());
        let m_id = "concurrent_mission";
        let key = "shared_counter";

        let mut handles = Vec::new();
        for i in 0..20 {
            let bb_clone = bb.clone();
            let handle = tokio::spawn(async move {
                bb_clone.set(
                    m_id,
                    key,
                    serde_json::json!({ "worker": i }),
                    &format!("agent_{}", i),
                    Vec::<String>::new(),
                );
            });
            handles.push(handle);
        }

        for h in handles {
            h.await.unwrap();
        }

        let entry = bb.get(m_id, key).expect("Entry must exist");
        assert_eq!(
            entry.version, 20,
            "Version must be exactly 20 after 20 atomic writes"
        );
    }

    #[test]
    fn test_blackboard_compare_and_swap_occ() {
        let bb = SharedBlackboard::new();
        let m_id = "cas_mission";
        let key = "shared_doc";

        // 1. Initial insert with expected_version = 0 must succeed
        let v1 = bb
            .compare_and_swap(
                m_id,
                key,
                0,
                serde_json::json!({"content": "draft v1"}),
                "agent_a",
                vec!["doc"],
            )
            .expect("Initial CAS insert should succeed");
        assert_eq!(v1.version, 1);

        // 2. Inserting with expected_version = 0 again must fail with VersionConflict
        let conflict = bb.compare_and_swap(
            m_id,
            key,
            0,
            serde_json::json!({"content": "clobber"}),
            "agent_b",
            Vec::<String>::new(),
        );
        assert!(matches!(
            conflict,
            Err(BlackboardError::VersionConflict {
                expected: 0,
                actual: 1,
                ..
            })
        ));

        // 3. Updating with valid expected_version = 1 must bump to v2
        let v2 = bb
            .compare_and_swap(
                m_id,
                key,
                1,
                serde_json::json!({"content": "draft v2"}),
                "agent_b",
                Vec::<String>::new(),
            )
            .expect("Valid CAS update should succeed");
        assert_eq!(v2.version, 2);

        // 4. Stale update with expected_version = 1 must be rejected
        let stale = bb.compare_and_swap(
            m_id,
            key,
            1,
            serde_json::json!({"content": "stale draft"}),
            "agent_a",
            Vec::<String>::new(),
        );
        assert!(matches!(
            stale,
            Err(BlackboardError::VersionConflict {
                expected: 1,
                actual: 2,
                ..
            })
        ));
    }

    #[test]
    fn test_blackboard_lifecycle_primitives() {
        let bb = SharedBlackboard::new();
        let m_id = "lifecycle_test";

        assert!(bb.is_empty(m_id));
        assert_eq!(bb.len(m_id), 0);

        bb.set(
            m_id,
            "k1",
            serde_json::json!("val1"),
            "author",
            Vec::<String>::new(),
        );
        bb.set(
            m_id,
            "k2",
            serde_json::json!("val2"),
            "author",
            Vec::<String>::new(),
        );

        assert!(!bb.is_empty(m_id));
        assert_eq!(bb.len(m_id), 2);
        assert!(bb.contains_key(m_id, "k1"));
        assert!(bb.contains_key(m_id, "k2"));
        assert!(!bb.contains_key(m_id, "k3"));

        assert!(bb.remove_key(m_id, "k1"));
        assert!(!bb.remove_key(m_id, "k1")); // Already removed
        assert_eq!(bb.len(m_id), 1);
        assert!(!bb.contains_key(m_id, "k1"));
    }

    #[test]
    fn test_blackboard_summary_deterministic_sort_and_bounds() {
        let bb = SharedBlackboard::new();
        let m_id = "summary_test";

        // Insert keys in non-alphabetical order with backticks and large objects
        bb.set(
            m_id,
            "z_key`injected",
            serde_json::json!("normal string"),
            "agent_`z",
            Vec::<String>::new(),
        );
        bb.set(
            m_id,
            "a_key",
            serde_json::json!({
                "k1": 1, "k2": 2, "k3": 3, "k4": 4, "k5": 5,
                "k6": 6, "k7": 7, "k8": 8, "k9": 9, "k10": 10, "k11": 11
            }),
            "agent_a",
            Vec::<String>::new(),
        );

        let summary = bb.export_summary(m_id);
        // Must defuse backticks to prevent markdown prompt injection
        assert!(!summary.contains("`injected`"));
        assert!(summary.contains("z_key'injected"));
        // Deterministic sorting: a_key must appear before z_key
        let a_idx = summary.find("a_key").unwrap();
        let z_idx = summary.find("z_key").unwrap();
        assert!(
            a_idx < z_idx,
            "Entries must be sorted alphabetically by key"
        );
        // Object keys bounded
        assert!(summary.contains("+3 keys"));
    }

    #[test]
    fn test_blackboard_utf8_truncation_exact_boundary() {
        let bb = SharedBlackboard::new();
        let m_id = "truncation_exact_test";

        // Exactly 60 characters (no truncation)
        let exact_60 = "a".repeat(60);
        bb.set(
            m_id,
            "k_60",
            serde_json::json!(exact_60),
            "agent",
            Vec::<String>::new(),
        );
        let summary_60 = bb.export_summary(m_id);
        assert!(!summary_60.contains("..."));

        // 61 characters (truncated to 57 + ...)
        let exact_61 = "b".repeat(61);
        bb.set(
            m_id,
            "k_61",
            serde_json::json!(exact_61),
            "agent",
            Vec::<String>::new(),
        );
        let summary_61 = bb.export_summary(m_id);
        assert!(summary_61.contains(&format!("\"{}...\"", "b".repeat(57))));
    }
}

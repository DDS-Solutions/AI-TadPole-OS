//! @docs ARCHITECTURE:Core:Intelligence
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / graph_store / model
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Strongly-typed graph rows, serialization envelopes, and atomic identifiers.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: none declared
//! - **Witness Tests**: `intelligence::graph_store::model::tests`

use crate::utils::parser::Reference;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::atomic::{AtomicI64, Ordering};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum EdgeKind {
    Calls,
    References,
    ImportsFrom,
    Tests,
    Contains,
}

impl EdgeKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            EdgeKind::Calls => "CALLS",
            EdgeKind::References => "REFERENCES",
            EdgeKind::ImportsFrom => "IMPORTS_FROM",
            EdgeKind::Tests => "TESTS",
            EdgeKind::Contains => "CONTAINS",
        }
    }

    pub fn is_reference_or_call(&self) -> bool {
        matches!(
            self,
            EdgeKind::Calls | EdgeKind::References | EdgeKind::ImportsFrom
        )
    }
}

impl fmt::Display for EdgeKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for EdgeKind {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "CALLS" => Ok(EdgeKind::Calls),
            "REFERENCES" => Ok(EdgeKind::References),
            "IMPORTS_FROM" => Ok(EdgeKind::ImportsFrom),
            "TESTS" | "TESTED_BY" => Ok(EdgeKind::Tests),
            "CONTAINS" => Ok(EdgeKind::Contains),
            other => Err(format!("Unknown EdgeKind: {other}")),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SymbolKind {
    File,
    Class,
    Function,
    Test,
}

impl SymbolKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            SymbolKind::File => "File",
            SymbolKind::Class => "Class",
            SymbolKind::Function => "Function",
            SymbolKind::Test => "Test",
        }
    }
}

pub fn default_max_flows() -> usize {
    250
}

pub fn default_community_rules() -> Vec<CommunityRule> {
    vec![
        CommunityRule {
            pattern: "/server-rs/".to_string(),
            id: 1,
            name: "server-rs-core".to_string(),
        },
        CommunityRule {
            pattern: "/src-tauri/".to_string(),
            id: 2,
            name: "tauri-shell".to_string(),
        },
        CommunityRule {
            pattern: "/src/".to_string(),
            id: 3,
            name: "frontend-app".to_string(),
        },
        CommunityRule {
            pattern: "/execution/".to_string(),
            id: 4,
            name: "execution-tools".to_string(),
        },
        CommunityRule {
            pattern: "/.agent/".to_string(),
            id: 5,
            name: "agent-assets".to_string(),
        },
        // Both /data/ and /migrations/ represent data storage and migration assets;
        // they are intentionally grouped under community id 6 ("data-migrations").
        CommunityRule {
            pattern: "/data/".to_string(),
            id: 6,
            name: "data-migrations".to_string(),
        },
        CommunityRule {
            pattern: "/migrations/".to_string(),
            id: 6,
            name: "data-migrations".to_string(),
        },
    ]
}

#[derive(Debug, Clone, Serialize)]
pub struct GraphDbRefreshSummary {
    pub db_path: PathBuf,
    pub node_count: usize,
    pub edge_count: usize,
    pub risk_count: usize,
    pub community_count: usize,
    pub flow_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommunityRule {
    pub pattern: String,
    pub id: i64,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphConfig {
    #[serde(default = "default_community_rules")]
    pub community_rules: Vec<CommunityRule>,
    #[serde(default = "default_max_flows")]
    pub max_flows: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ConfigPayload {
    LegacyRules(Vec<CommunityRule>),
    FullConfig(GraphConfig),
}

#[derive(Debug, Clone)]
pub struct FileRecord {
    pub absolute_path: String,
    pub relative_path: String,
    pub name: String,
    pub language: String,
    pub is_test: bool,
    pub symbols: Vec<SymbolRecord>,
    pub refs: Vec<Reference>,
    pub imports: Vec<String>,
    pub file_hash: String,
}

#[derive(Debug, Clone)]
pub struct RawFileRecord {
    pub absolute_path: String,
    pub relative_path: String,
    pub name: String,
    pub language: String,
    pub is_test: bool,
    pub content: String,
    pub file_hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolRecord {
    pub name: String,
    pub kind: String,
    pub line_start: i64,
    pub line_end: i64,
    pub signature: String,
    pub parent_name: Option<String>,
    pub params: Option<String>,
    pub return_type: Option<String>,
    pub modifiers: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedFileData {
    pub symbols: Vec<SymbolRecord>,
    pub refs: Vec<Reference>,
    pub imports: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct NodeRow {
    pub id: i64,
    pub kind: String,
    pub name: String,
    pub qualified_name: String,
    pub file_path: String,
    pub line_start: Option<i64>,
    pub line_end: Option<i64>,
    pub language: String,
    pub parent_name: Option<String>,
    pub params: Option<String>,
    pub return_type: Option<String>,
    pub modifiers: Option<String>,
    pub is_test: bool,
    pub file_hash: String,
    pub extra: String,
    pub signature: String,
    pub community_id: Option<i64>,
}

#[derive(Debug, Clone, Eq, Hash, PartialEq, Ord, PartialOrd)]
pub struct EdgeRow {
    pub kind: EdgeKind,
    pub source_qualified: String,
    pub target_qualified: String,
    pub file_path: String,
    pub line: i64,
    pub extra: String,
}

#[derive(Debug, Clone)]
pub struct RiskRow {
    pub node_id: i64,
    pub qualified_name: String,
    pub risk_score: f64,
    pub caller_count: i64,
    pub test_coverage: String,
    pub security_relevant: bool,
}

#[derive(Debug, Clone)]
pub struct CommunityRow {
    pub id: i64,
    pub name: String,
    pub cohesion: f64,
    pub size: i64,
    pub dominant_language: String,
    pub description: String,
    pub risk: String,
}

#[derive(Debug, Clone)]
pub struct FlowRow {
    pub id: i64,
    pub name: String,
    pub entry_point_id: i64,
    pub entry_point: String,
    pub depth: i64,
    pub node_count: i64,
    pub node_ids: Vec<i64>,
    pub critical_path: Vec<String>,
    pub criticality: f64,
    pub file_count: i64,
}

#[derive(Debug, Clone)]
pub struct GraphSnapshot {
    pub root: PathBuf,
    pub git_branch: Option<String>,
    pub git_head_sha: Option<String>,
    pub nodes: Vec<NodeRow>,
    pub edges: Vec<EdgeRow>,
    pub risks: Vec<RiskRow>,
    pub communities: Vec<CommunityRow>,
    pub flows: Vec<FlowRow>,
    pub cache_updates: Vec<(String, String, String)>, // (file_path, file_hash, cache_json)
    pub files_present: Vec<String>,
}

pub struct IdGenerator {
    counter: AtomicI64,
}

impl IdGenerator {
    pub fn new(start: i64) -> Self {
        Self {
            counter: AtomicI64::new(start),
        }
    }

    pub fn next_id(&self) -> i64 {
        self.counter.fetch_add(1, Ordering::SeqCst)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_edge_kind_parsing_and_legacy_compatibility() {
        assert_eq!("CALLS".parse::<EdgeKind>().unwrap(), EdgeKind::Calls);
        assert_eq!("TESTS".parse::<EdgeKind>().unwrap(), EdgeKind::Tests);
        assert_eq!("TESTED_BY".parse::<EdgeKind>().unwrap(), EdgeKind::Tests);
        assert_eq!(
            "IMPORTS_FROM".parse::<EdgeKind>().unwrap(),
            EdgeKind::ImportsFrom
        );
        assert_eq!("CONTAINS".parse::<EdgeKind>().unwrap(), EdgeKind::Contains);
        assert!("INVALID".parse::<EdgeKind>().is_err());
    }

    #[test]
    fn test_id_generator_sequence() {
        let gen = IdGenerator::new(10);
        assert_eq!(gen.next_id(), 10);
        assert_eq!(gen.next_id(), 11);
        assert_eq!(gen.next_id(), 12);
    }
}

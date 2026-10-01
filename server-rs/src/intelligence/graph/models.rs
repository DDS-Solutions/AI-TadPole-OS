//! @docs ARCHITECTURE:Intelligence
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Intelligence / Graph Models
//! - **Primary Entrypoints**: `SymbolNode`, `SymbolEdge`, `GraphStateRepository`, `GraphError`
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none declared
//! - **Telemetry Targets**: none declared
//! - **Witness Tests**: none declared

use petgraph::graph::NodeIndex;
use serde::{Deserialize, Serialize};
use specta::Type;
use std::collections::HashMap;
use std::path::PathBuf;

pub const MAX_DISCOVERED_FILES: usize = 10_000;
/// Maximum number of nodes allowed in the CodeSymbolGraph.
pub const MAX_NODES: usize = 20_000;
/// Maximum number of edges allowed in the CodeSymbolGraph.
pub const MAX_EDGES: usize = 100_000;
/// Maximum file size for scanned code files (2MB).
pub const MAX_FILE_SIZE_BYTES: u64 = 2 * 1024 * 1024;

#[derive(Debug, thiserror::Error)]
pub enum GraphError {
    #[error("Workspace root not found: {0}")]
    WorkspaceRootNotFound(String),
    #[error("Path lies outside workspace boundary: {0}")]
    PathOutOfBounds(String),
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Invalid key normalizer state: {0}")]
    KeyNormalization(String),
    #[error("Internal error: {0}")]
    Internal(String),
}

/// Directories excluded from all codebase knowledge graph discovery operations.
pub const EXCLUDED_DIRS: &[&str] = &[
    "target",
    "node_modules",
    ".git",
    "dist",
    "scratch",
    "3rdparty",
    ".tmp",
    "tmp",
    "workspaces",
    ".agent",
    "coverage",
    ".fallow",
    ".gemini",
    "logs",
    ".vscode",
    "reports",
    ".code-review-graph",
];

/// A node in the knowledge graph representing a code symbol.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct SymbolNode {
    pub name: String,
    pub path: String,
    pub kind: String,
    pub signature: String,
    pub start_line: u32,
    pub end_line: u32,
    pub docstring: Option<String>,
    pub docstring_range: Option<crate::utils::parser::SymbolRange>,
}

/// An edge in the knowledge graph representing a dependency.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct SymbolEdge {
    pub kind: String,
}

/// Repository containing the cached AST parse structures and file metadata
pub struct GraphStateRepository {
    pub file_metadata: HashMap<PathBuf, (std::time::SystemTime, u64)>,
    pub parse_cache: HashMap<
        String,
        (
            Vec<crate::utils::parser::Symbol>,
            Vec<crate::utils::parser::Reference>,
        ),
    >,
}

impl Default for GraphStateRepository {
    fn default() -> Self {
        Self {
            file_metadata: HashMap::new(),
            parse_cache: HashMap::new(),
        }
    }
}

/// Trait defining normalisation behavior for index keys
pub trait KeyNormalizer: Send + Sync {
    fn normalize_key(&self, path: &str, name: &str) -> String;
}

/// Default normalization replacing null bytes to prevent DoS collisions
pub struct DefaultKeyNormalizer;

impl KeyNormalizer for DefaultKeyNormalizer {
    fn normalize_key(&self, path: &str, name: &str) -> String {
        // Sanitize both null bytes and the separator character to prevent spoofing/collision
        let clean_path = path.replace(['\0', '\x01'], "_");
        let clean_name = name.replace(['\0', '\x01'], "_");
        format!("{clean_path}\x01{clean_name}")
    }
}

pub fn index_key(path: &str, name: &str) -> String {
    DefaultKeyNormalizer.normalize_key(path, name)
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct GraphConfig {
    pub ignored_symbol_names: Vec<String>,
}

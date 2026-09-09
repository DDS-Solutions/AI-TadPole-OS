//! @docs ARCHITECTURE:Intelligence
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / graph
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Type-safe state handling and bounded execution without unhandled panics.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: `[Graph]`
//! - **Witness Tests**: `intelligence::graph::tests`

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

use crate::utils::parser::SymbolExtractor;
use petgraph::graph::{DiGraph, NodeIndex};
use petgraph::visit::EdgeRef;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use specta::Type;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

use sha2::{Digest, Sha256};

fn to_unix_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

fn sanitize_log_path(path: &Path, root: &Path) -> String {
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

fn index_key(path: &str, name: &str) -> String {
    DefaultKeyNormalizer.normalize_key(path, name)
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

/// Service trait to discover files within the workspace root
pub trait FileDiscoverer: Send + Sync {
    fn discover(&self, root: &Path) -> Result<Vec<PathBuf>, GraphError>;
}

/// Default implementation of the discovery service
pub struct FileDiscoveryService {
    pub exclusions: Vec<String>,
}

impl Default for FileDiscoveryService {
    fn default() -> Self {
        Self {
            exclusions: EXCLUDED_DIRS.iter().map(|s| s.to_string()).collect(),
        }
    }
}

impl FileDiscoverer for FileDiscoveryService {
    fn discover(&self, root: &Path) -> Result<Vec<PathBuf>, GraphError> {
        let canonical_root = root.canonicalize().map_err(|e| {
            GraphError::WorkspaceRootNotFound(format!(
                "Failed to canonicalize root {}: {}",
                root.display(),
                e
            ))
        })?;

        let exclusions = &self.exclusions;
        let mut files = Vec::new();

        for entry in WalkDir::new(root)
            .follow_links(false)
            .into_iter()
            .filter_entry(|e| {
                let name = e.file_name().to_string_lossy();
                !exclusions.iter().any(|ex| ex.eq_ignore_ascii_case(&name))
            })
        {
            let entry = match entry {
                Ok(e) => e,
                Err(e) => {
                    tracing::warn!("⚠️ [Graph] Directory traversal warning: {}", e);
                    continue;
                }
            };

            if !entry.path().is_file() {
                continue;
            }

            let path = entry.path();
            let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
            let is_code = ext == "rs" || ext == "ts" || ext == "tsx";
            let is_wiki = ext == "md"
                && path
                    .to_string_lossy()
                    .replace('\\', "/")
                    .contains("docs/wiki");
            if !is_code && !is_wiki {
                continue;
            }

            // 🛡️ [DoS Protection] Enforce unified size limit
            let metadata = match std::fs::metadata(path) {
                Ok(m) => m,
                Err(e) => {
                    tracing::warn!(
                        "⚠️ [Graph] Failed to read metadata for {}: {}",
                        sanitize_log_path(path, root),
                        e
                    );
                    continue;
                }
            };

            if metadata.len() > MAX_FILE_SIZE_BYTES {
                tracing::warn!(
                    "⚠️ [Graph] Skipping oversized file ({} bytes): {}",
                    metadata.len(),
                    sanitize_log_path(path, root)
                );
                continue;
            }

            // 🛡️ Path Boundary Verification (Symlink Protection)
            let canonical_path = match path.canonicalize() {
                Ok(p) => p,
                Err(e) => {
                    tracing::warn!(
                        "⚠️ [Graph] Failed to canonicalize path {}: {}",
                        sanitize_log_path(path, root),
                        e
                    );
                    continue;
                }
            };

            if !canonical_path.starts_with(&canonical_root) {
                tracing::warn!(
                    "⚠️ [Graph] Security Violation: Path {} points outside workspace root {}",
                    sanitize_log_path(&canonical_path, root),
                    sanitize_log_path(&canonical_root, root)
                );
                continue;
            }

            files.push(path.to_path_buf());
        }

        files.sort();
        Ok(files)
    }
}

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

/// Service trait to synthesize the graph from cached/parsed inputs
pub trait GraphSynthesizer: Send + Sync {
    fn synthesize(
        &self,
        graph: &mut CodeSymbolGraph,
        salt: &str,
        to_delete: &[PathBuf],
        updates: Vec<(
            PathBuf,
            String,
            Option<(
                Vec<crate::utils::parser::Symbol>,
                Vec<crate::utils::parser::Reference>,
                std::time::SystemTime,
                u64,
            )>,
        )>,
    ) -> Result<(), GraphError>;
}

/// Default implementation of the graph synthesis service
pub struct GraphSynthesisEngine;

impl GraphSynthesisEngine {
    fn update_caches(
        &self,
        graph: &mut CodeSymbolGraph,
        to_delete: &[PathBuf],
        updates: Vec<(
            PathBuf,
            String,
            Option<(
                Vec<crate::utils::parser::Symbol>,
                Vec<crate::utils::parser::Reference>,
                std::time::SystemTime,
                u64,
            )>,
        )>,
    ) {
        // 1. Remove deleted files from caches
        for path in to_delete {
            let rel_path = to_unix_path(path.strip_prefix(&graph.root).unwrap_or(path));
            graph.repository.file_metadata.remove(path);
            graph.repository.parse_cache.remove(&rel_path);
        }

        // 2. Apply parsed updates sequentially to avoid concurrent mutation issues
        for (path, rel_path, opt_data) in updates {
            if let Some((symbols, refs, mtime, size)) = opt_data {
                graph
                    .repository
                    .parse_cache
                    .insert(rel_path, (symbols, refs));
                graph.repository.file_metadata.insert(path, (mtime, size));
            } else {
                graph.repository.parse_cache.remove(&rel_path);
                graph.repository.file_metadata.remove(&path);
            }
        }
    }

    fn rebuild_scratch_nodes(
        &self,
        repository: &GraphStateRepository,
        salt: &str,
    ) -> Result<
        (
            DiGraph<SymbolNode, SymbolEdge>,
            HashMap<String, NodeIndex>,
            HashMap<String, String>,
            HashMap<String, Vec<NodeIndex>>,
        ),
        GraphError,
    > {
        let mut scratch_graph = DiGraph::new();
        let mut scratch_index = HashMap::new();
        let mut scratch_obf = HashMap::new();
        let mut name_to_indices: HashMap<String, Vec<NodeIndex>> = HashMap::new();

        // Deterministic sorting of parse_cache keys
        let mut sorted_keys: Vec<&String> = repository.parse_cache.keys().collect();
        sorted_keys.sort();

        for rel_path in sorted_keys {
            let (symbols, _) = &repository.parse_cache[rel_path];
            let obf_path = obfuscate_path(rel_path, salt)?;
            scratch_obf.insert(obf_path.clone(), rel_path.to_string());

            for sym in symbols {
                if scratch_graph.node_count() >= MAX_NODES {
                    tracing::warn!(
                        "⚠️ [Graph] Hit MAX_NODES ({MAX_NODES}), skipping additional symbols to prevent memory exhaustion."
                    );
                    break;
                }

                let key = index_key(rel_path, &sym.name);
                let node = SymbolNode {
                    name: sym.name.clone(),
                    path: obf_path.clone(),
                    kind: sym.kind.clone(),
                    signature: sym.signature.clone(),
                    start_line: (sym.range.start_line + 1) as u32,
                    end_line: (sym.range.end_line + 1) as u32,
                    docstring: sym.docstring.clone(),
                    docstring_range: sym.docstring_range.clone(),
                };

                let idx = scratch_graph.add_node(node);
                scratch_index.insert(key, idx);

                let entry = name_to_indices.entry(sym.name.clone()).or_default();
                if entry.len() < 1000 {
                    entry.push(idx);
                } else {
                    tracing::warn!(
                        "⚠️ [Graph] Soft limit (1,000) exceeded for symbol '{}' (path: {}). Disabling indexing for this duplicate to prevent memory exhaustion.",
                        sym.name,
                        rel_path
                    );
                }
            }
        }
        tracing::info!("🔍 [Graph] Indexed {} symbols.", scratch_index.len());
        Ok((scratch_graph, scratch_index, scratch_obf, name_to_indices))
    }

    fn rebuild_scratch_edges(
        &self,
        scratch_graph: &mut DiGraph<SymbolNode, SymbolEdge>,
        scratch_index: &HashMap<String, NodeIndex>,
        repository: &GraphStateRepository,
        name_to_indices: &HashMap<String, Vec<NodeIndex>>,
    ) -> Result<(), GraphError> {
        let mut added_edges = std::collections::HashSet::new();

        let mut sorted_keys: Vec<&String> = repository.parse_cache.keys().collect();
        sorted_keys.sort();

        for rel_path in sorted_keys {
            let (symbols, refs) = &repository.parse_cache[rel_path];
            if symbols.is_empty() || refs.is_empty() {
                continue;
            }

            // Sort symbols stably by start_byte, placing outer spans first if starts are equal
            let mut sorted_syms: Vec<&crate::utils::parser::Symbol> = symbols.iter().collect();
            sorted_syms.sort_by(|a, b| {
                a.range
                    .start_byte
                    .cmp(&b.range.start_byte)
                    .then_with(|| b.range.end_byte.cmp(&a.range.end_byte))
            });

            // Sort references by start_byte
            let mut sorted_refs: Vec<&crate::utils::parser::Reference> = refs.iter().collect();
            sorted_refs.sort_by_key(|r| r.range.start_byte);

            let mut active_stack: Vec<&crate::utils::parser::Symbol> = Vec::new();
            let mut sym_iter = sorted_syms.into_iter().peekable();

            for r in sorted_refs {
                while let Some(&sym) = sym_iter.peek() {
                    if sym.range.start_byte <= r.range.start_byte {
                        active_stack.push(sym);
                        sym_iter.next();
                    } else {
                        break;
                    }
                }

                while let Some(top) = active_stack.last() {
                    if top.range.end_byte <= r.range.start_byte {
                        active_stack.pop();
                    } else {
                        break;
                    }
                }

                if let Some(src_sym) = active_stack.last() {
                    if r.range.end_byte <= src_sym.range.end_byte {
                        let src_key = index_key(rel_path, &src_sym.name);
                        if let Some(&src_idx) = scratch_index.get(&src_key) {
                            if let Some(target_indices) = name_to_indices.get(&r.name) {
                                // Bound fan-out to prevent combinatorial edge explosion
                                for &target_idx in target_indices.iter().take(20) {
                                    if src_idx != target_idx
                                        && added_edges.insert((src_idx, target_idx))
                                    {
                                        if scratch_graph.edge_count() >= MAX_EDGES {
                                            tracing::warn!(
                                                "⚠️ [Graph] Hit MAX_EDGES ({MAX_EDGES}), skipping remaining edges."
                                            );
                                            return Ok(());
                                        }
                                        scratch_graph.add_edge(
                                            src_idx,
                                            target_idx,
                                            SymbolEdge {
                                                kind: "ref".to_string(),
                                            },
                                        );
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }
}

impl GraphSynthesizer for GraphSynthesisEngine {
    fn synthesize(
        &self,
        graph: &mut CodeSymbolGraph,
        salt: &str,
        to_delete: &[PathBuf],
        updates: Vec<(
            PathBuf,
            String,
            Option<(
                Vec<crate::utils::parser::Symbol>,
                Vec<crate::utils::parser::Reference>,
                std::time::SystemTime,
                u64,
            )>,
        )>,
    ) -> Result<(), GraphError> {
        self.update_caches(graph, to_delete, updates);

        // Build into scratch structures
        let (mut scratch_graph, scratch_index, scratch_obf, name_to_indices) =
            self.rebuild_scratch_nodes(&graph.repository, salt)?;
        self.rebuild_scratch_edges(
            &mut scratch_graph,
            &scratch_index,
            &graph.repository,
            &name_to_indices,
        )?;

        // Atomic swap to prevent torn state on any failure
        graph.graph = scratch_graph;
        graph.index = scratch_index;
        graph.obfuscated_to_real_path = scratch_obf;

        tracing::info!(
            "✅ [Graph] Knowledge graph build complete (Nodes: {}, Edges: {}).",
            graph.graph.node_count(),
            graph.graph.edge_count()
        );

        Ok(())
    }
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
struct GraphConfig {
    pub ignored_symbol_names: Vec<String>,
}

fn load_ignored_symbols(root: &Path) -> std::collections::HashSet<String> {
    let default_ignored = &[
        // Proxy traps
        "get",
        "set",
        "has",
        "deleteProperty",
        "ownKeys",
        "getOwnPropertyDescriptor",
        "defineProperty",
        "preventExtensions",
        "isExtensible",
        "getPrototypeOf",
        "setPrototypeOf",
        "apply",
        "construct",
        // Standard built-ins / overrides
        "constructor",
        "toString",
        "valueOf",
        "toJSON",
        // React Component Lifecycle / standard methods
        "render",
        "componentDidMount",
        "componentDidUpdate",
        "componentWillUnmount",
        "shouldComponentUpdate",
        "getDerivedStateFromProps",
        "getDerivedStateFromError",
        "componentDidCatch",
        // Workspace / Oversight
        "Workspace_Status",
    ];

    let mut ignored_set: std::collections::HashSet<String> =
        default_ignored.iter().map(|&s| s.to_string()).collect();

    let config_path = root.join(".agent/graph_config.json");
    if config_path.exists() {
        match std::fs::read_to_string(&config_path) {
            Ok(content) => match serde_json::from_str::<GraphConfig>(&content) {
                Ok(config) => {
                    ignored_set.extend(config.ignored_symbol_names);
                }
                Err(e) => {
                    tracing::warn!(
                        "⚠️ [Graph] Failed to parse .agent/graph_config.json: {}. Merging defaults.",
                        e
                    );
                }
            },
            Err(e) => {
                tracing::warn!(
                    "⚠️ [Graph] Failed to read .agent/graph_config.json: {}. Using default ignored symbols.",
                    e
                );
            }
        }
    }

    ignored_set
}

/// The core Knowledge Graph engine.
pub struct CodeSymbolGraph {
    pub graph: DiGraph<SymbolNode, SymbolEdge>,
    pub index: HashMap<String, NodeIndex>, // key: path + "\x01" + name
    pub obfuscated_to_real_path: HashMap<String, String>,
    pub repository: GraphStateRepository,
    pub ignored_symbols: std::collections::HashSet<String>,
    root: PathBuf,
}

impl CodeSymbolGraph {
    /// Creates a new, empty knowledge graph.
    pub fn new(root: PathBuf) -> Self {
        let ignored_symbols = load_ignored_symbols(&root);
        Self {
            graph: DiGraph::new(),
            index: HashMap::new(),
            obfuscated_to_real_path: HashMap::new(),
            repository: GraphStateRepository::default(),
            ignored_symbols,
            root,
        }
    }

    /// Scans the workspace and populates the graph with symbols and references.
    #[allow(clippy::type_complexity)]
    pub fn build(&mut self, salt: &str) -> Result<(), GraphError> {
        tracing::info!(
            "🔍 [Graph] Building symbol-level knowledge graph for {}...",
            self.root.display()
        );

        let discovery = FileDiscoveryService::default();
        let cache_mgr = CacheManagementService;
        let parser = CodeParsingService;
        let synthesizer = GraphSynthesisEngine;

        // 1. Discovery
        let discovered_files = discovery.discover(&self.root)?;

        if discovered_files.len() > MAX_DISCOVERED_FILES {
            return Err(GraphError::Internal(format!(
                "Workspace size limit exceeded: found {} files, max allowed is {}",
                discovered_files.len(),
                MAX_DISCOVERED_FILES
            )));
        }

        // 2. Cache check
        let (to_parse, to_delete) = cache_mgr.check_changes(
            &discovered_files,
            &self.repository.file_metadata,
            &self.root,
        );

        // Optimization: return early if no updates and graph is populated
        if to_parse.is_empty() && to_delete.is_empty() && !self.index.is_empty() {
            tracing::info!(
                "✅ [Graph] Knowledge graph is already up-to-date. (Nodes: {}, Edges: {})",
                self.graph.node_count(),
                self.graph.edge_count()
            );
            return Ok(());
        }

        // 3. Parsing
        let updates = parser.parse_files(&to_parse, &self.root)?;

        // 4. Synthesis
        synthesizer.synthesize(self, salt, &to_delete, updates)?;

        Ok(())
    }

    /// Produces structural review candidates from parsed static references.
    /// A zero-reference result is not proof of dead code because JSX, reflection,
    /// event registration, and other dynamic usage may be outside parser coverage.
    pub fn find_anomalies(&self) -> Vec<String> {
        let mut anomalies = Vec::new();

        for idx in self.graph.node_indices() {
            if let Some(node) = self.graph.node_weight(idx) {
                let real_path = self
                    .obfuscated_to_real_path
                    .get(&node.path)
                    .map(|p| p.as_str())
                    .unwrap_or(&node.path);

                // Skip declaration files, config files, and generated code
                if real_path.ends_with(".d.ts")
                    || real_path.ends_with("vite.config.ts")
                    || real_path.ends_with("playwright.config.ts")
                    || real_path.ends_with(".test.ts")
                    || real_path.ends_with(".test.tsx")
                    || real_path.ends_with(".spec.ts")
                    || real_path.ends_with(".spec.tsx")
                {
                    continue;
                }

                // Skip backend, shell, and WASM codec crates, plus scratch files, checking exact path components
                let path_obj = Path::new(real_path);
                let has_excluded_component = path_obj.components().any(|c| {
                    let name = c.as_os_str().to_string_lossy();
                    name == "server-rs"
                        || name == "src-tauri"
                        || name == "wasm-codec"
                        || name == "scratch"
                        || name == "generated"
                        || name == "contracts"
                        || name == "test"
                        || name == "tests"
                        || name == "__tests__"
                });
                if has_excluded_component {
                    continue;
                }

                if real_path.contains("pages/")
                    || real_path.contains("pages\\")
                    || real_path.contains("components/ui/")
                    || real_path.contains("components\\ui\\")
                    || real_path.ends_with("App.tsx")
                    || real_path.ends_with("main.tsx")
                {
                    continue;
                }

                if self.ignored_symbols.contains(&node.name)
                    || node.kind == "module"
                    || node.name == "__module__"
                {
                    continue;
                }

                // Skip entrypoints, tests, and standard route/event handlers / breaker helpers
                let name_lower = node.name.to_lowercase();
                if name_lower == "main"
                    || name_lower == "app"
                    || name_lower.contains("test")
                    || name_lower.contains("route")
                    || name_lower.contains("handler")
                    || name_lower.contains("register")
                    || name_lower.contains("force_")
                    || name_lower.contains("invalidate_")
                    || name_lower.contains("persist_")
                    || name_lower.contains("breaker")
                {
                    continue;
                }

                let incoming = self
                    .graph
                    .edges_directed(idx, petgraph::Direction::Incoming)
                    .count();
                if incoming == 0 {
                    anomalies.push(format!(
                        "Review candidate (0 parsed incoming references; dynamic usage may be untracked): {} in {}",
                        node.name, node.path
                    ));
                }
            }
        }

        anomalies
    }

    /// Calculates the "Blast Radius" for a given symbol.
    /// Returns a list of symbols that directly or indirectly depend on it.
    pub fn calculate_blast_radius(
        &self,
        symbol_name: &str,
        path: &str,
        depth_limit: usize,
    ) -> Vec<SymbolNode> {
        let real_path = self
            .obfuscated_to_real_path
            .get(path)
            .map(|p| p.as_str())
            .unwrap_or(path);
        let key = index_key(real_path, symbol_name);
        let mut affected = Vec::new();

        if let Some(&start_idx) = self.index.get(&key) {
            // BFS to find all symbols that reference this one up to depth limit
            // Note: edges are (source -> target), so we need to traverse in REVERSE (target -> source)
            let mut visited = std::collections::HashSet::new();
            let mut queue = std::collections::VecDeque::new();
            queue.push_back((start_idx, 0));
            visited.insert(start_idx);

            let mut affected_indices = vec![start_idx];
            while let Some((current_idx, depth)) = queue.pop_front() {
                if depth >= depth_limit {
                    continue; // Shield against malicious/adversarial large depth chains
                }
                // Find all neighbors that point to current_idx
                for edge in self
                    .graph
                    .edges_directed(current_idx, petgraph::Direction::Incoming)
                {
                    let neighbor_idx = edge.source();
                    if visited.insert(neighbor_idx) {
                        affected_indices.push(neighbor_idx);
                        queue.push_back((neighbor_idx, depth + 1));
                    }
                }
            }

            // Perform single contiguous clone of final affected payloads to avoid traversal allocation pressure
            for idx in affected_indices {
                affected.push(self.graph[idx].clone());
            }
        }

        affected
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn test_empty_blast_radius_nonexistent() {
        let dir = tempdir().unwrap();
        let graph = CodeSymbolGraph::new(dir.path().to_path_buf());
        let affected = graph.calculate_blast_radius("nonexistent", "src/lib.rs", 50);
        assert!(
            affected.is_empty(),
            "Blast radius of nonexistent symbol must be empty"
        );
    }

    #[test]
    fn test_happy_path_symbol_dependency() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("main.rs");

        // Write mock code content with two symbols: main and helper
        let mut file = File::create(&file_path).unwrap();
        writeln!(file, "fn helper() {{ }}").unwrap();
        writeln!(file, "fn main() {{ helper(); }}").unwrap();

        let mut graph = CodeSymbolGraph::new(dir.path().to_path_buf());
        let salt = uuid::Uuid::new_v4().to_string();
        graph.build(&salt).unwrap();

        // Check that nodes and edges are populated
        assert!(
            graph.graph.node_count() >= 2,
            "Should index at least 2 symbols"
        );

        // Calculate blast radius for helper() - main() should be affected
        let affected = graph.calculate_blast_radius("helper", "main.rs", 50);
        assert!(
            !affected.is_empty(),
            "helper blast radius should not be empty"
        );
        let has_main = affected.iter().any(|node| node.name == "main");
        assert!(has_main, "main should depend on helper");
    }

    #[test]
    fn test_circular_dependency_handling() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("main.rs");

        // Write circular dependency mock code
        let mut file = File::create(&file_path).unwrap();
        writeln!(file, "fn alpha() {{ beta(); }}").unwrap();
        writeln!(file, "fn beta() {{ alpha(); }}").unwrap();

        let mut graph = CodeSymbolGraph::new(dir.path().to_path_buf());
        let salt = uuid::Uuid::new_v4().to_string();
        graph.build(&salt).unwrap();

        // BFS should handle the cycle gracefully and terminate without infinite loop
        let affected_alpha = graph.calculate_blast_radius("alpha", "main.rs", 50);
        let affected_beta = graph.calculate_blast_radius("beta", "main.rs", 50);

        assert!(!affected_alpha.is_empty());
        assert!(!affected_beta.is_empty());
    }

    #[test]
    fn test_incremental_ast_caching() {
        let dir = tempdir().unwrap();
        let file_a = dir.path().join("a.rs");
        let file_b = dir.path().join("b.rs");

        // Write initial files
        let mut f_a = File::create(&file_a).unwrap();
        writeln!(f_a, "fn helper() {{ }}").unwrap();
        drop(f_a);

        let mut f_b = File::create(&file_b).unwrap();
        writeln!(f_b, "fn main() {{ helper(); }}").unwrap();
        drop(f_b);

        let mut graph = CodeSymbolGraph::new(dir.path().to_path_buf());
        let salt = "test_salt".to_string();

        // 1. Initial build
        graph.build(&salt).unwrap();
        assert_eq!(graph.repository.file_metadata.len(), 2);
        assert_eq!(graph.repository.parse_cache.len(), 2);
        assert!(graph.index.contains_key(&index_key("a.rs", "helper")));
        assert!(graph.index.contains_key(&index_key("b.rs", "main")));

        // Record initial metadata
        let meta_a_before = *graph.repository.file_metadata.get(&file_a).unwrap();
        let meta_b_before = *graph.repository.file_metadata.get(&file_b).unwrap();

        // Sleep/Wait a moment to ensure mtime changes if we write (though size change is enough)
        std::thread::sleep(std::time::Duration::from_millis(10));

        // 2. Modify file_b, keep file_a untouched
        let mut f_b_mod = File::create(&file_b).unwrap();
        writeln!(f_b_mod, "fn main() {{ helper(); // modified comment \n }}").unwrap();
        drop(f_b_mod);

        graph.build(&salt).unwrap();

        // file_a metadata should be completely identical (cached)
        let meta_a_after = *graph.repository.file_metadata.get(&file_a).unwrap();
        assert_eq!(meta_a_before, meta_a_after);

        // file_b metadata should have changed
        let meta_b_after = *graph.repository.file_metadata.get(&file_b).unwrap();
        assert_ne!(meta_b_before, meta_b_after);

        // 3. Delete file_a and verify cleanup
        std::fs::remove_file(&file_a).unwrap();
        graph.build(&salt).unwrap();

        assert_eq!(graph.repository.file_metadata.len(), 1);
        assert_eq!(graph.repository.parse_cache.len(), 1);
        assert!(!graph.repository.file_metadata.contains_key(&file_a));
        assert!(!graph.repository.parse_cache.contains_key("a.rs"));
        assert!(!graph.index.contains_key(&index_key("a.rs", "helper")));
        assert!(graph.index.contains_key(&index_key("b.rs", "main")));
    }

    #[test]
    fn test_blast_radius_deep_cycle_limit() {
        let dir = tempdir().unwrap();
        let mut graph = CodeSymbolGraph::new(dir.path().to_path_buf());

        let obf_path = "obf/path.rs".to_string();
        graph
            .obfuscated_to_real_path
            .insert(obf_path.clone(), "path.rs".to_string());

        let mut indices = Vec::new();
        for i in 1..=55 {
            let name = format!("S_{i}");
            let node = SymbolNode {
                name: name.clone(),
                path: obf_path.clone(),
                kind: "func".to_string(),
                signature: format!("fn S_{i}()"),
                start_line: i,
                end_line: i + 1,
                docstring: None,
                docstring_range: None,
            };
            let idx = graph.graph.add_node(node);
            graph.index.insert(index_key("path.rs", &name), idx);
            indices.push(idx);
        }

        // Add reverse reference edges (S_N references S_N-1, so S_N -> S_N-1, meaning incoming to S_N-1 from S_N)
        for i in 1..55 {
            graph.graph.add_edge(
                indices[i],     // source: S_{i+1}
                indices[i - 1], // target: S_i
                SymbolEdge {
                    kind: "ref".to_string(),
                },
            );
        }
        // S_1 references S_55 (indices[0] -> indices[54])
        graph.graph.add_edge(
            indices[0],  // source: S_1
            indices[54], // target: S_55
            SymbolEdge {
                kind: "ref".to_string(),
            },
        );

        let affected = graph.calculate_blast_radius("S_55", "path.rs", 50);

        // Output must contain start node (S_55) + exactly 50 nodes matching the depth limit
        assert_eq!(
            affected.len(),
            51,
            "Visited count should respect depth limit of 50 steps"
        );
    }

    #[test]
    fn test_blast_radius_isolated_node() {
        let dir = tempdir().unwrap();
        let mut graph = CodeSymbolGraph::new(dir.path().to_path_buf());

        let obf_path = "obf/path.rs".to_string();
        graph
            .obfuscated_to_real_path
            .insert(obf_path.clone(), "path.rs".to_string());

        let node = SymbolNode {
            name: "X".to_string(),
            path: obf_path.clone(),
            kind: "func".to_string(),
            signature: "fn X()".to_string(),
            start_line: 1,
            end_line: 2,
            docstring: None,
            docstring_range: None,
        };
        let idx = graph.graph.add_node(node);
        graph.index.insert(index_key("path.rs", "X"), idx);

        let affected = graph.calculate_blast_radius("X", "path.rs", 50);
        assert_eq!(affected.len(), 1);
        assert_eq!(affected[0].name, "X");
    }

    #[test]
    fn test_full_cycle_with_mixed_changes() {
        let dir = tempdir().unwrap();
        let file_a = dir.path().join("a.rs");
        let file_b = dir.path().join("b.rs");
        let file_c = dir.path().join("c.rs");

        // Baseline files
        std::fs::write(&file_a, "fn a_func() {}").unwrap();
        std::fs::write(&file_b, "fn b_func() { a_func(); }").unwrap();
        std::fs::write(&file_c, "fn c_func() {}").unwrap();

        let mut graph = CodeSymbolGraph::new(dir.path().to_path_buf());
        let salt = "salt".to_string();

        // Baseline build
        graph.build(&salt).unwrap();
        assert_eq!(graph.repository.file_metadata.len(), 3);
        assert_eq!(graph.repository.parse_cache.len(), 3);
        assert!(graph.index.contains_key(&index_key("a.rs", "a_func")));
        assert!(graph.index.contains_key(&index_key("b.rs", "b_func")));
        assert!(graph.index.contains_key(&index_key("c.rs", "c_func")));

        // Record old metadata
        let meta_b_before = *graph.repository.file_metadata.get(&file_b).unwrap();
        let meta_c_before = *graph.repository.file_metadata.get(&file_c).unwrap();

        std::thread::sleep(std::time::Duration::from_millis(10));

        // Mixed Changes:
        // 1. Modify B.rs (change size/content slightly)
        std::fs::write(&file_b, "fn b_func() { c_func(); } // modified").unwrap();
        // 2. Delete A.rs
        std::fs::remove_file(&file_a).unwrap();
        // 3. Add D.rs
        let file_d = dir.path().join("d.rs");
        std::fs::write(&file_d, "fn d_func() {}").unwrap();

        // Re-build
        graph.build(&salt).unwrap();

        // Assert caches are correct
        assert_eq!(graph.repository.file_metadata.len(), 3); // B.rs, C.rs, D.rs
        assert_eq!(graph.repository.parse_cache.len(), 3);
        assert!(!graph.repository.file_metadata.contains_key(&file_a));
        assert!(!graph.repository.parse_cache.contains_key("a.rs"));
        assert!(graph.repository.file_metadata.contains_key(&file_b));
        assert!(graph.repository.file_metadata.contains_key(&file_c));
        assert!(graph.repository.file_metadata.contains_key(&file_d));

        // Assert B has updated, C is untouched (cached), D is added
        let meta_b_after = *graph.repository.file_metadata.get(&file_b).unwrap();
        let meta_c_after = *graph.repository.file_metadata.get(&file_c).unwrap();
        assert_ne!(meta_b_before, meta_b_after);
        assert_eq!(meta_c_before, meta_c_after);

        // Assert graph nodes & edges are correct
        assert!(!graph.index.contains_key(&index_key("a.rs", "a_func")));
        assert!(graph.index.contains_key(&index_key("b.rs", "b_func")));
        assert!(graph.index.contains_key(&index_key("c.rs", "c_func")));
        assert!(graph.index.contains_key(&index_key("d.rs", "d_func")));

        // Edge changes: B should now depend on C, not A
        let affected_c = graph.calculate_blast_radius("c_func", "c.rs", 50);
        assert!(affected_c.iter().any(|node| node.name == "b_func"));

        // If one file is unreadable or missing during parsing phase, assert it gracefully skips and returns None
        std::fs::remove_file(&file_b).unwrap();
        let files_list = vec![file_c, file_d, file_b.clone()]; // B.rs is missing now
        let cache_mgr = CacheManagementService;
        let (to_parse, _to_delete) =
            cache_mgr.check_changes(&files_list, &graph.repository.file_metadata, &graph.root);
        assert!(to_parse.contains(&file_b));

        let parser = CodeParsingService;
        let parse_res = parser.parse_files(&to_parse, &graph.root);
        assert!(
            parse_res.is_ok(),
            "Missing file should not abort batch parsing"
        );
        let updates = parse_res.unwrap();
        let b_update = updates.iter().find(|(p, _, _)| p == &file_b).unwrap();
        assert!(
            b_update.2.is_none(),
            "Unreadable file should yield None update"
        );
    }

    #[test]
    fn test_typescript_import_export_handling() {
        let dir = tempdir().unwrap();
        let file_a = dir.path().join("a.tsx");
        let file_b = dir.path().join("b.tsx");

        // Write non-circular TSX import/export files
        std::fs::write(&file_a, "export function foo() { return 42; }").unwrap();
        std::fs::write(
            &file_b,
            "import { foo } from './a';\nexport function bar() { foo(); }",
        )
        .unwrap();

        let mut graph = CodeSymbolGraph::new(dir.path().to_path_buf());
        let salt = uuid::Uuid::new_v4().to_string();
        graph.build(&salt).unwrap();

        // Verify that nodes are registered in the graph index
        assert!(graph.index.contains_key(&index_key("a.tsx", "foo")));
        assert!(graph.index.contains_key(&index_key("b.tsx", "bar")));

        // Calculate blast radius for foo - bar should be affected since bar references foo
        let affected = graph.calculate_blast_radius("foo", "a.tsx", 50);
        assert!(affected.iter().any(|node| node.name == "bar"));
    }

    #[test]
    fn test_typescript_circular_dependency() {
        let dir = tempdir().unwrap();
        let file_a = dir.path().join("a.tsx");
        let file_b = dir.path().join("b.tsx");

        // Write circular TSX import/export files
        std::fs::write(
            &file_a,
            "import { bar } from './b';\nexport function foo() { bar(); }",
        )
        .unwrap();
        std::fs::write(
            &file_b,
            "import { foo } from './a';\nexport function bar() { foo(); }",
        )
        .unwrap();

        let mut graph = CodeSymbolGraph::new(dir.path().to_path_buf());
        let salt = uuid::Uuid::new_v4().to_string();
        graph.build(&salt).unwrap();

        // Verify that nodes are registered
        assert!(graph.index.contains_key(&index_key("a.tsx", "foo")));
        assert!(graph.index.contains_key(&index_key("b.tsx", "bar")));

        // Verify circular blast radius terminates successfully and contains both
        let affected_foo = graph.calculate_blast_radius("foo", "a.tsx", 50);
        assert!(affected_foo.iter().any(|node| node.name == "bar"));

        let affected_bar = graph.calculate_blast_radius("bar", "b.tsx", 50);
        assert!(affected_bar.iter().any(|node| node.name == "foo"));
    }

    #[test]
    fn test_deterministic_build_ordering() {
        let dir = tempdir().unwrap();
        let file_a = dir.path().join("z_last.rs");
        let file_b = dir.path().join("a_first.rs");

        std::fs::write(&file_a, "fn z_func() {}").unwrap();
        std::fs::write(&file_b, "fn a_func() { z_func(); }").unwrap();

        let mut graph1 = CodeSymbolGraph::new(dir.path().to_path_buf());
        graph1.build("deterministic_salt").unwrap();

        let mut graph2 = CodeSymbolGraph::new(dir.path().to_path_buf());
        graph2.build("deterministic_salt").unwrap();

        // Node counts and edge counts match
        assert_eq!(graph1.graph.node_count(), graph2.graph.node_count());
        assert_eq!(graph1.graph.edge_count(), graph2.graph.edge_count());

        // Node indices and key mappings match run-to-run
        for (key, idx1) in &graph1.index {
            let idx2 = graph2.index.get(key).expect("Key must exist in graph2");
            assert_eq!(idx1, idx2, "NodeIndex must be deterministic across builds");
        }
    }
}

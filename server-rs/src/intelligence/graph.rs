//! @docs ARCHITECTURE:Core
//!
//! ### AI Assist Note
//! **! Symbol-Level Knowledge Graph — Codebase Topology**
//! This module implements high-fidelity logic for the Sovereign Reality layer.
//!
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: Runtime logic error, state desynchronization, or resource exhaustion.
//! - **Telemetry Link**: Search `[graph]` in tracing logs.

//!   Symbol-Level Knowledge Graph — Codebase Topology
//!
//! @docs ARCHITECTURE:Intelligence
//!
//! ### AI Assist Note
//! **Knowledge Graph**: Builds a directed graph of code symbols
//! (functions, structs, traits) and their interdependencies.
//! Enables **Blast Radius Analysis**: helps agents understand the
//! impact of changing a specific symbol by tracing outgoing edges.

use crate::utils::parser::SymbolExtractor;
use petgraph::graph::{DiGraph, NodeIndex};
use petgraph::visit::EdgeRef;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use specta::Type;
use std::collections::HashMap;
use std::path::PathBuf;
use walkdir::WalkDir;

use sha2::{Digest, Sha256};
use std::path::Path;

fn to_unix_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

fn index_key(path: &str, name: &str) -> String {
    format!("{path}\0{name}")
}


/// Helper to obfuscate physical file path structures deterministically
/// while preserving UX force-graph clustering and file basenames.
pub fn obfuscate_path(path_str: &str, salt: &str) -> String {
    let path = Path::new(path_str);
    let file_name = path
        .file_name()
        .and_then(|f| f.to_str())
        .unwrap_or("unknown");
    let parent = path.parent().unwrap_or(Path::new("")).to_string_lossy();

    if parent.is_empty() {
        file_name.to_string()
    } else {
        let mut hasher = Sha256::new();
        hasher.update(salt.as_bytes());
        hasher.update(parent.as_bytes());
        let result = hasher.finalize();
        let hash_val = hex::encode(result);
        let obf_prefix = hash_val.get(..16).unwrap_or(&hash_val);
        format!("{}/{}", obf_prefix, file_name)
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
}

/// An edge in the knowledge graph representing a dependency.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct SymbolEdge {
    pub kind: String,
}

/// The core Knowledge Graph engine.
pub struct CodeSymbolGraph {
    pub graph: DiGraph<SymbolNode, SymbolEdge>,
    pub index: HashMap<String, NodeIndex>, // key: path + "\0" + name
    pub obfuscated_to_real_path: HashMap<String, String>,
    pub file_metadata: HashMap<PathBuf, (std::time::SystemTime, u64)>,
    pub parse_cache: HashMap<
        String,
        (
            Vec<crate::utils::parser::Symbol>,
            Vec<crate::utils::parser::Reference>,
        ),
    >,
    root: PathBuf,
}

impl CodeSymbolGraph {
    /// Creates a new, empty knowledge graph.
    pub fn new(root: PathBuf) -> Self {
        Self {
            graph: DiGraph::new(),
            index: HashMap::new(),
            obfuscated_to_real_path: HashMap::new(),
            file_metadata: HashMap::new(),
            parse_cache: HashMap::new(),
            root,
        }
    }

    /// Scans the workspace and populates the graph with symbols and references.
    #[allow(clippy::type_complexity)]
    pub fn build(&mut self, salt: &str) {
        tracing::info!(
            "🔍 [Graph] Building symbol-level knowledge graph for {}...",
            self.root.display()
        );

        // 1. Gather all target files to scan with eager directory walking pruning
        let files: Vec<PathBuf> = WalkDir::new(&self.root)
            .into_iter()
            .filter_entry(|e| {
                let name = e.file_name().to_string_lossy();
                // Prune these directories immediately to prevent WalkDir from descending into them
                name != "target"
                    && name != "node_modules"
                    && name != ".git"
                    && name != "dist"
                    && name != "scratch"
            })
            .filter_map(|e| e.ok())
            .filter(|e| e.path().is_file())
            .map(|e| e.path().to_path_buf())
            .filter(|path| {
                let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
                if ext != "rs" && ext != "ts" && ext != "tsx" {
                    return false;
                }

                // 🛡️ [DoS Protection] Enforce 2MB size limit to avoid scanning massive database/build/artifact dumps
                if let Ok(metadata) = std::fs::metadata(path) {
                    if metadata.len() > 2 * 1024 * 1024 {
                        tracing::warn!(
                            "⚠️ [Graph] Skipping oversized file ({} bytes): {}",
                            metadata.len(),
                            path.display()
                        );
                        return false;
                    }
                } else {
                    return false;
                }
                true
            })
            .collect();

        // 1.5. Identify deleted files and clean up cache
        let active_paths: std::collections::HashSet<&PathBuf> = files.iter().collect();
        let deleted_paths: Vec<PathBuf> = self
            .file_metadata
            .keys()
            .filter(|p| !active_paths.contains(p))
            .cloned()
            .collect();
        let has_deleted = !deleted_paths.is_empty();

        for path in deleted_paths {
            let rel_path = to_unix_path(
                path.strip_prefix(&self.root).unwrap_or(&path)
            );
            self.file_metadata.remove(&path);
            self.parse_cache.remove(&rel_path);
        }

        // 1.6. Identify modified or new files
        let mut files_to_parse = Vec::new();
        for path in &files {
            let mut needs_parse = true;
            if let Ok(m) = std::fs::metadata(path) {
                if let (Ok(mtime), size) = (m.modified(), m.len()) {
                    if let Some(&(cached_mtime, cached_size)) = self.file_metadata.get(path) {
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

        // Optimization: if no files modified or deleted, and graph is already populated, return early
        if files_to_parse.is_empty() && !has_deleted && !self.index.is_empty() {
            tracing::info!(
                "✅ [Graph] Knowledge graph is already up-to-date. (Nodes: {}, Edges: {})",
                self.graph.node_count(),
                self.graph.edge_count()
            );
            return;
        }

        // 2. Parse only the new or modified files in parallel using Rayon (Single-Pass reading)
        let parsed_updates: Vec<(
            PathBuf,
            String,
            Option<(
                Vec<crate::utils::parser::Symbol>,
                Vec<crate::utils::parser::Reference>,
                std::time::SystemTime,
                u64,
            )>,
        )> = files_to_parse
            .par_iter()
            .map_init(SymbolExtractor::new, |extractor, path| {
                let rel_path = to_unix_path(
                    path.strip_prefix(&self.root).unwrap_or(path)
                );
                match std::fs::read_to_string(path) {
                    Ok(content) => {
                        let symbols = extractor.extract_symbols(path, &content);
                        let refs = extractor.extract_references(path, &content);
                        if let Ok(m) = std::fs::metadata(path) {
                            if let (Ok(mtime), size) = (m.modified(), m.len()) {
                                return (
                                    path.clone(),
                                    rel_path,
                                    Some((symbols, refs, mtime, size)),
                                );
                            }
                        }
                        (path.clone(), rel_path, None)
                    }
                    Err(e) => {
                        tracing::warn!("⚠️ [Graph] Failed to read file {}: {}", path.display(), e);
                        (path.clone(), rel_path, None)
                    }
                }
            })
            .collect();

        // Update caches
        for (path, rel_path, opt_data) in parsed_updates {
            if let Some((symbols, refs, mtime, size)) = opt_data {
                self.parse_cache.insert(rel_path, (symbols, refs));
                self.file_metadata.insert(path, (mtime, size));
            } else {
                self.parse_cache.remove(&rel_path);
                self.file_metadata.remove(&path);
            }
        }

        // 3. Clear existing graph structure to rebuild in-memory
        self.graph.clear();
        self.index.clear();
        self.obfuscated_to_real_path.clear();

        // 4. Add nodes and compile Inverted Name Index
        let mut name_to_indices: HashMap<String, Vec<NodeIndex>> = HashMap::new();
        for (rel_path, (symbols, _)) in &self.parse_cache {
            let obf_path = obfuscate_path(rel_path, salt);
            self.obfuscated_to_real_path
                .insert(obf_path.clone(), rel_path.to_string());
            for sym in symbols {
                let key = index_key(rel_path, &sym.name);
                let node = SymbolNode {
                    name: sym.name.clone(),
                    path: obf_path.clone(), // Store obfuscated path directly in the graph node
                    kind: sym.kind.clone(),
                    signature: sym.signature.clone(),
                    start_line: (sym.range.start_line + 1) as u32,
                    end_line: (sym.range.end_line + 1) as u32,
                };
                let idx = self.graph.add_node(node);
                self.index.insert(key, idx);
                name_to_indices
                    .entry(sym.name.clone())
                    .or_default()
                    .push(idx);
            }
        }

        tracing::info!("🔍 [Graph] Indexed {} symbols.", self.index.len());

        // 5. Extract references and add edges (Dependencies)
        let mut added_edges = std::collections::HashSet::new();
        for (rel_path, (symbols, refs)) in &self.parse_cache {
            if symbols.is_empty() {
                continue;
            }
            for r in refs {
                // Find the tightest (deepest nested) source symbol in THIS file that contains this reference range
                let mut tightest_src: Option<(&crate::utils::parser::Symbol, usize)> = None;

                
                // Binary search for the symbol starting closest to the reference start_byte
                let search_start = match symbols.binary_search_by(|sym| sym.range.start_byte.cmp(&r.range.start_byte)) {
                    Ok(idx) => idx,
                    Err(idx) => idx.saturating_sub(1),
                };
                
                for i in (0..=search_start).rev() {
                    let src_sym = &symbols[i];
                    if src_sym.range.start_byte > r.range.start_byte {
                        continue;
                    }
                    if r.range.start_byte >= src_sym.range.start_byte
                        && r.range.end_byte <= src_sym.range.end_byte
                    {
                        let span_size = src_sym.range.end_byte - src_sym.range.start_byte;
                        match tightest_src {
                            None => {
                                tightest_src = Some((src_sym, span_size));
                            }
                            Some((_, current_min_span)) => {
                                if span_size < current_min_span {
                                    tightest_src = Some((src_sym, span_size));
                                }
                            }
                        }
                    }
                }

                if let Some((src_sym, _)) = tightest_src {
                    let src_key = index_key(rel_path, &src_sym.name);
                    if let Some(&src_idx) = self.index.get(&src_key) {
                        if let Some(target_indices) = name_to_indices.get(&r.name) {
                            for &target_idx in target_indices {
                                if src_idx != target_idx
                                    && added_edges.insert((src_idx, target_idx))
                                {
                                    self.graph.add_edge(
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

        tracing::info!(
            "✅ [Graph] Knowledge graph build complete (Nodes: {}, Edges: {}).",
            self.graph.node_count(),
            self.graph.edge_count()
        );
    }

    /// Audits the graph for structural anomalies (dead code).
    pub fn find_anomalies(&self) -> Vec<String> {
        let mut anomalies = Vec::new();

        for idx in self.graph.node_indices() {
            if let Some(node) = self.graph.node_weight(idx) {
                let real_path = self.obfuscated_to_real_path.get(&node.path).map(|p| p.as_str()).unwrap_or(&node.path);

                // Skip TypeScript/JavaScript files due to AST reference resolution limitations
                if real_path.ends_with(".ts") || real_path.ends_with(".tsx") {
                    continue;
                }

                // Skip backend, shell, and WASM codec crates since Rust compiler dead-code and public-export patterns are handled natively
                if real_path.starts_with("server-rs/")
                    || real_path.starts_with("src-tauri/")
                    || real_path.starts_with("wasm-codec/")
                {
                    continue;
                }

                // Skip scratch/ files since they are temporary development scripts
                if real_path.contains("scratch/") {
                    continue;
                }

                // Skip entrypoints, tests, and standard route/event handlers
                let name_lower = node.name.to_lowercase();
                if name_lower == "main"
                    || name_lower.contains("test")
                    || name_lower.contains("route")
                    || name_lower.contains("handler")
                    || name_lower.contains("register")
                {
                    continue;
                }

                let incoming = self
                    .graph
                    .edges_directed(idx, petgraph::Direction::Incoming)
                    .count();
                if incoming == 0 {
                    anomalies.push(format!(
                        "Unused symbol (0 incoming references): {} in {}",
                        node.name, node.path
                    ));
                }
            }
        }

        anomalies
    }

    /// Calculates the "Blast Radius" for a given symbol.
    /// Returns a list of symbols that directly or indirectly depend on it.
    pub fn calculate_blast_radius(&self, symbol_name: &str, path: &str) -> Vec<SymbolNode> {
        let real_path = self.obfuscated_to_real_path.get(path).map(|p| p.as_str()).unwrap_or(path);
        let key = index_key(real_path, symbol_name);
        let mut affected = Vec::new();

        if let Some(&start_idx) = self.index.get(&key) {
            // BFS to find all symbols that reference this one up to depth 50
            // Note: edges are (source -> target), so we need to traverse in REVERSE (target -> source)
            let mut visited = std::collections::HashSet::new();
            let mut queue = std::collections::VecDeque::new();
            queue.push_back((start_idx, 0));
            visited.insert(start_idx);

            let mut affected_indices = Vec::new();
            while let Some((current_idx, depth)) = queue.pop_front() {
                if depth >= 50 {
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
        let affected = graph.calculate_blast_radius("nonexistent", "src/lib.rs");
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
        graph.build(&salt);

        // Check that nodes and edges are populated
        assert!(
            graph.graph.node_count() >= 2,
            "Should index at least 2 symbols"
        );

        // Calculate blast radius for helper() - main() should be affected
        let affected = graph.calculate_blast_radius("helper", "main.rs");
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
        graph.build(&salt);

        // BFS should handle the cycle gracefully and terminate without infinite loop
        let affected_alpha = graph.calculate_blast_radius("alpha", "main.rs");
        let affected_beta = graph.calculate_blast_radius("beta", "main.rs");

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
        graph.build(&salt);
        assert_eq!(graph.file_metadata.len(), 2);
        assert_eq!(graph.parse_cache.len(), 2);
        assert!(graph
            .index
            .contains_key(&index_key("a.rs", "helper")));
        assert!(graph
            .index
            .contains_key(&index_key("b.rs", "main")));

        // Record initial metadata
        let meta_a_before = *graph.file_metadata.get(&file_a).unwrap();
        let meta_b_before = *graph.file_metadata.get(&file_b).unwrap();

        // Sleep/Wait a moment to ensure mtime changes if we write (though size change is enough)
        std::thread::sleep(std::time::Duration::from_millis(10));

        // 2. Modify file_b, keep file_a untouched
        let mut f_b_mod = File::create(&file_b).unwrap();
        writeln!(f_b_mod, "fn main() {{ helper(); // modified comment \n }}").unwrap();
        drop(f_b_mod);

        graph.build(&salt);

        // file_a metadata should be completely identical (cached)
        let meta_a_after = *graph.file_metadata.get(&file_a).unwrap();
        assert_eq!(meta_a_before, meta_a_after);

        // file_b metadata should have changed
        let meta_b_after = *graph.file_metadata.get(&file_b).unwrap();
        assert_ne!(meta_b_before, meta_b_after);

        // 3. Delete file_a and verify cleanup
        std::fs::remove_file(&file_a).unwrap();
        graph.build(&salt);

        assert_eq!(graph.file_metadata.len(), 1);
        assert_eq!(graph.parse_cache.len(), 1);
        assert!(!graph.file_metadata.contains_key(&file_a));
        assert!(!graph.parse_cache.contains_key("a.rs"));
        assert!(!graph
            .index
            .contains_key(&index_key("a.rs", "helper")));
        assert!(graph
            .index
            .contains_key(&index_key("b.rs", "main")));
    }
}

// Metadata: [graph]

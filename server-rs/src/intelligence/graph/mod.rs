//! @docs ARCHITECTURE:Intelligence
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Intelligence / Graph
//! - **Primary Entrypoints**: `CodeSymbolGraph`
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Type-safe state handling and bounded execution without unhandled panics.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none declared
//! - **Telemetry Targets**: none declared
//! - **Witness Tests**: none declared

pub mod cache;
pub mod discovery;
pub mod models;
pub mod parsing;
pub mod path_utils;
pub mod synthesis;

pub use cache::*;
pub use discovery::*;
pub use models::*;
pub use parsing::*;
pub use path_utils::*;
pub use synthesis::*;

use petgraph::graph::{DiGraph, NodeIndex};
use petgraph::visit::EdgeRef;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// The core Knowledge Graph engine.
pub struct CodeSymbolGraph {
    pub graph: DiGraph<SymbolNode, SymbolEdge>,
    pub index: HashMap<String, NodeIndex>, // key: path + "\x01" + name
    pub obfuscated_to_real_path: HashMap<String, String>,
    pub repository: GraphStateRepository,
    pub ignored_symbols: std::collections::HashSet<String>,
    pub root: PathBuf,
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

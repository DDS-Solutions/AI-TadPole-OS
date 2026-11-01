//! @docs ARCHITECTURE:Intelligence
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Intelligence / Graph Synthesis
//! - **Primary Entrypoints**: `GraphSynthesizer`, `GraphSynthesisEngine`, `load_ignored_symbols`
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none declared
//! - **Telemetry Targets**: none declared
//! - **Witness Tests**: none declared

use super::models::{
    index_key, GraphConfig, GraphError, GraphStateRepository, SymbolEdge, SymbolNode, MAX_EDGES,
    MAX_NODES,
};
use super::path_utils::{obfuscate_path, to_unix_path};
use super::CodeSymbolGraph;
use petgraph::graph::{DiGraph, NodeIndex};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

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
    pub fn update_caches(
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

    pub fn rebuild_scratch_nodes(
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

    pub fn rebuild_scratch_edges(
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

pub fn load_ignored_symbols(root: &Path) -> std::collections::HashSet<String> {
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

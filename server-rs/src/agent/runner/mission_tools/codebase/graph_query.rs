//! @docs ARCHITECTURE:Registry
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Agent Runner / codebase access
//!
//! ### AI Assist Note
//! - Keep repository path authorization separate from symbol parsing and graph queries.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: ToolExecutionError, path validation failures
//! - **Telemetry Targets**: none declared

use super::*;

use petgraph::visit::EdgeRef;

impl AgentRunner {
    /// Handles "query_symbol_graph": queries the in-memory AST symbol graph for symbol lookup or blast radius analysis.
    pub(crate) async fn handle_query_symbol_graph(
        &self,
        _ctx: &RunContext,
        fc: &crate::agent::types::ToolCall,
    ) -> Result<String, ToolExecutionError> {
        let action = fc
            .args
            .get("action")
            .and_then(|v| v.as_str())
            .unwrap_or("lookup");
        let symbol_opt = fc
            .args
            .get("symbol")
            .and_then(|v| v.as_str())
            .map(|s| s.trim())
            .filter(|s| !s.is_empty());
        let path_opt = fc
            .args
            .get("path")
            .and_then(|v| v.as_str())
            .map(|s| s.trim())
            .filter(|s| !s.is_empty());
        let max_depth = fc
            .args
            .get("max_depth")
            .and_then(|v| v.as_u64())
            .unwrap_or(20)
            .clamp(1, 50) as usize;

        if symbol_opt.is_none() && path_opt.is_none() {
            return Ok(
                "(QUERY FAILED: Please provide at least a 'symbol' name or 'path' to inspect the graph.)"
                    .to_string(),
            );
        }

        // 1. Ensure symbol graph is built
        let graph_lock = self.state.resources.get_symbol_graph().await;
        let is_empty = {
            let guard = graph_lock.read();
            guard.index.is_empty()
        };

        if is_empty {
            let lock_clone = std::sync::Arc::clone(&graph_lock);
            let salt = self.state.resources.obfuscation_salt.clone();
            let _ = tokio::task::spawn_blocking(move || {
                let mut graph = lock_clone.write();
                if graph.index.is_empty() {
                    let _ = graph.build(&salt);
                }
            })
            .await;
        }

        let guard = graph_lock.read();

        if guard.graph.node_count() == 0 {
            return Ok(
                "(SYMBOL GRAPH EMPTY: No symbols currently indexed in the codebase graph.)"
                    .to_string(),
            );
        }

        match action {
            "blast_radius" => {
                let symbol_name = match symbol_opt {
                    Some(s) => s,
                    None => {
                        return Ok(
                            "(QUERY FAILED: 'symbol' is required for action 'blast_radius'.)"
                                .to_string(),
                        );
                    }
                };

                let mut target_node: Option<crate::intelligence::graph::SymbolNode> = None;
                for idx in guard.graph.node_indices() {
                    if let Some(node) = guard.graph.node_weight(idx) {
                        if node.name == symbol_name {
                            if let Some(p) = path_opt {
                                if node.path.ends_with(p) || p.ends_with(&node.path) {
                                    target_node = Some(node.clone());
                                    break;
                                }
                            } else {
                                target_node = Some(node.clone());
                                break;
                            }
                        }
                    }
                }

                let target = match target_node {
                    Some(n) => n,
                    None => {
                        return Ok(format!(
                            "(SYMBOL '{}' NOT FOUND in symbol graph for blast radius analysis)",
                            symbol_name
                        ));
                    }
                };

                let affected = guard.calculate_blast_radius(&target.name, &target.path, max_depth);
                let mut out = format!(
                    "(BLAST RADIUS FOR '{}' in {} [Depth Limit: {}]):\n\nAffected downstream symbols ({}):\n",
                    target.name,
                    target.path,
                    max_depth,
                    affected.len()
                );

                if affected.is_empty() {
                    out.push_str("  (No downstream dependents affected)\n");
                } else {
                    for (i, node) in affected.iter().take(30).enumerate() {
                        out.push_str(&format!(
                            "  {}. [{}] {} ({}:{}-{})\n",
                            i + 1,
                            node.kind,
                            node.name,
                            node.path,
                            node.start_line,
                            node.end_line
                        ));
                    }
                    if affected.len() > 30 {
                        out.push_str(&format!(
                            "  ... and {} more affected symbols\n",
                            affected.len() - 30
                        ));
                    }
                }

                Ok(out)
            }
            _ => {
                // "lookup" action (default)
                use petgraph::Direction;

                let mut matches_found = Vec::new();
                for idx in guard.graph.node_indices() {
                    if let Some(node) = guard.graph.node_weight(idx) {
                        let name_matches = match symbol_opt {
                            Some(s) => node.name.eq_ignore_ascii_case(s),
                            None => true,
                        };
                        let path_matches = match path_opt {
                            Some(p) => node.path.ends_with(p) || p.ends_with(&node.path),
                            None => true,
                        };
                        if name_matches && path_matches {
                            let callers: Vec<String> = guard
                                .graph
                                .edges_directed(idx, Direction::Incoming)
                                .filter_map(|e| guard.graph.node_weight(e.source()))
                                .map(|n| format!("{} ({})", n.name, n.path))
                                .take(10)
                                .collect();

                            let callees: Vec<String> = guard
                                .graph
                                .edges_directed(idx, Direction::Outgoing)
                                .filter_map(|e| guard.graph.node_weight(e.target()))
                                .map(|n| format!("{} ({})", n.name, n.path))
                                .take(10)
                                .collect();

                            matches_found.push((node.clone(), callers, callees));
                            if matches_found.len() >= 10 {
                                break;
                            }
                        }
                    }
                }

                if matches_found.is_empty() {
                    let query_desc = match (symbol_opt, path_opt) {
                        (Some(s), Some(p)) => format!("symbol '{}' in path '{}'", s, p),
                        (Some(s), None) => format!("symbol '{}'", s),
                        (None, Some(p)) => format!("path '{}'", p),
                        (None, None) => "query".to_string(),
                    };
                    return Ok(format!(
                        "(NO SYMBOLS MATCHED: {} in the codebase symbol graph)",
                        query_desc
                    ));
                }

                let mut out = format!(
                    "(SYMBOL GRAPH LOOKUP RESULTS - {} match(es)):\n\n",
                    matches_found.len()
                );
                for (node, callers, callees) in matches_found {
                    out.push_str(&format!(
                        "• [{}] {} at {}:{}-{}\n  Signature: {}\n",
                        node.kind,
                        node.name,
                        node.path,
                        node.start_line,
                        node.end_line,
                        if node.signature.is_empty() {
                            "N/A"
                        } else {
                            &node.signature
                        }
                    ));
                    if let Some(ref doc) = node.docstring {
                        if !doc.trim().is_empty() {
                            out.push_str(&format!("  Doc: {}\n", doc.trim()));
                        }
                    }
                    if !callers.is_empty() {
                        out.push_str(&format!(
                            "  Callers ({}): {}\n",
                            callers.len(),
                            callers.join(", ")
                        ));
                    } else {
                        out.push_str("  Callers: None (direct or external entrypoint)\n");
                    }
                    if !callees.is_empty() {
                        out.push_str(&format!(
                            "  Callees ({}): {}\n",
                            callees.len(),
                            callees.join(", ")
                        ));
                    }
                    out.push('\n');
                }

                Ok(out)
            }
        }
    }
}

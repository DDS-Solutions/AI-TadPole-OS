//! @docs ARCHITECTURE:CodeBaseIntelligence
//!
//! ### AI Assist Note
//! **Query Manager**: Coordinates the lookup, blast radius, and export formatting.
//!
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: Missing symbol queries, serialization errors.
//! - **Trace Scope**: `server-rs::bin::graph_query::query_manager`

use crate::graph::{CodeSymbolGraph, SymbolNode};
use petgraph::graph::NodeIndex;
use petgraph::visit::EdgeRef;
use serde::Serialize;
use std::path::Path;

static AUDIT_USAGE_TEMPLATES: &[&str] = &[
    "npm run graph:lookup -- --name SymbolName",
    "npm run graph:file -- --path src/pages/Neural_Map.tsx",
    "npm run graph:blast -- --path server-rs/src/routes/intelligence.rs --name get_blast_radius",
    "npm run graph:export",
];

#[derive(Serialize, Clone)]
pub struct ReportMetadata {
    pub generated_at: String,
    pub root: String,
    pub stats: GraphStats,
}

#[derive(Serialize)]
pub struct GraphExport {
    #[serde(flatten)]
    pub metadata: ReportMetadata,
    pub nodes: Vec<SymbolNode>,
    pub links: Vec<GraphLink>,
    pub anomalies: Vec<String>,
}

#[derive(Serialize)]
pub struct AuditContext {
    #[serde(flatten)]
    pub metadata: ReportMetadata,
    pub top_connected: Vec<SymbolContext>,
    pub anomalies: Vec<String>,
    pub usage: Vec<String>,
}

#[derive(Clone, Serialize)]
pub struct GraphStats {
    pub node_count: usize,
    pub edge_count: usize,
    pub anomaly_count: usize,
}

#[derive(Serialize)]
pub struct GraphLink {
    pub source: String,
    pub target: String,
}

#[derive(Serialize)]
pub struct SymbolContext {
    pub id: String,
    pub node: SymbolNode,
    pub callers: Vec<SymbolNode>,
    pub callees: Vec<SymbolNode>,
    pub blast_radius: Vec<SymbolNode>,
}

#[derive(Serialize)]
pub struct FileContext {
    pub path: String,
    pub symbols: Vec<SymbolContext>,
}

pub fn node_id(node: &SymbolNode) -> String {
    format!("{}:{}", node.path, node.name)
}

pub fn build_export(graph: &CodeSymbolGraph, root: &Path) -> GraphExport {
    let nodes = graph
        .graph
        .node_indices()
        .map(|idx| graph.graph[idx].clone())
        .collect::<Vec<_>>();
    let links = graph
        .graph
        .edge_references()
        .map(|edge| GraphLink {
            source: node_id(&graph.graph[edge.source()]),
            target: node_id(&graph.graph[edge.target()]),
        })
        .collect::<Vec<_>>();
    let anomalies = graph.find_anomalies();
    let stats = GraphStats {
        node_count: nodes.len(),
        edge_count: links.len(),
        anomaly_count: anomalies.len(),
    };

    GraphExport {
        metadata: ReportMetadata {
            generated_at: chrono::Utc::now().to_rfc3339(),
            root: root.display().to_string(),
            stats,
        },
        nodes,
        links,
        anomalies,
    }
}

pub fn build_audit_context(graph: &CodeSymbolGraph, root: &Path, limit: usize) -> AuditContext {
    let export = build_export(graph, root);
    let query_manager = GraphQueryManager::new(graph);

    let mut ranked = graph
        .graph
        .node_indices()
        .map(|idx| {
            let incoming = graph
                .graph
                .edges_directed(idx, petgraph::Direction::Incoming)
                .count();
            let outgoing = graph
                .graph
                .edges_directed(idx, petgraph::Direction::Outgoing)
                .count();
            (idx, incoming + outgoing)
        })
        .collect::<Vec<_>>();
    ranked.sort_by_key(|b| std::cmp::Reverse(b.1));

    let top_connected = ranked
        .into_iter()
        .take(limit)
        .map(|(idx, _)| query_manager.symbol_context(idx, limit))
        .collect::<Vec<_>>();

    AuditContext {
        metadata: export.metadata.clone(),
        top_connected,
        anomalies: export.anomalies.into_iter().take(limit).collect(),
        usage: AUDIT_USAGE_TEMPLATES
            .iter()
            .map(|s| s.to_string())
            .collect(),
    }
}

pub struct GraphQueryManager<'a> {
    pub graph: &'a CodeSymbolGraph,
}

impl<'a> GraphQueryManager<'a> {
    pub fn new(graph: &'a CodeSymbolGraph) -> Self {
        Self { graph }
    }

    pub fn lookup_by_name(&self, name: &str, limit: usize) -> Vec<SymbolContext> {
        self.graph
            .graph
            .node_indices()
            .filter(|idx| self.graph.graph[*idx].name == name)
            .take(limit)
            .map(|idx| self.symbol_context(idx, limit))
            .collect()
    }

    pub fn lookup_by_file(&self, path: &str, limit: usize) -> Vec<SymbolContext> {
        let target_path = self
            .graph
            .obfuscated_to_real_path
            .iter()
            .find(|(_, real)| *real == path)
            .map(|(obf, _)| obf.as_str())
            .unwrap_or(path);

        self.graph
            .graph
            .node_indices()
            .filter(|idx| self.graph.graph[*idx].path == target_path)
            .take(limit)
            .map(|idx| self.symbol_context(idx, limit))
            .collect()
    }

    pub fn symbol_context(&self, idx: NodeIndex, limit: usize) -> SymbolContext {
        let callers = self
            .related_nodes(idx, petgraph::Direction::Incoming, limit)
            .into_iter()
            .cloned()
            .collect();
        let callees = self
            .related_nodes(idx, petgraph::Direction::Outgoing, limit)
            .into_iter()
            .cloned()
            .collect();
        let node = self.graph.graph[idx].clone();
        let blast_radius = self
            .graph
            .calculate_blast_radius(&node.name, &node.path, 50)
            .into_iter()
            .take(limit)
            .collect();

        SymbolContext {
            id: node_id(&node),
            node,
            callers,
            callees,
            blast_radius,
        }
    }

    fn related_nodes(
        &self,
        idx: NodeIndex,
        direction: petgraph::Direction,
        limit: usize,
    ) -> Vec<&'a SymbolNode> {
        self.graph
            .graph
            .edges_directed(idx, direction)
            .map(|edge| match direction {
                petgraph::Direction::Incoming => edge.source(),
                petgraph::Direction::Outgoing => edge.target(),
            })
            .take(limit)
            .map(|neighbor| &self.graph.graph[neighbor])
            .collect()
    }
}

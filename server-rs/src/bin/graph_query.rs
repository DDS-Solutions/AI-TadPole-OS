#![allow(
    clippy::too_many_arguments,
    clippy::type_complexity,
    clippy::enum_variant_names,
    clippy::collapsible_match,
    clippy::unnecessary_map_or,
    clippy::derivable_impls,
    clippy::redundant_closure
)]
//! @docs ARCHITECTURE:CodeBaseIntelligence
//! Scriptable symbol graph queries for coding agents and audits.
//!
//! ### AI Assist Note
//! - **Purpose**: Runs independent scriptable symbol graph queries.
//! - **Usage**: Executed during audits or by agent commands.
//!
//! ### 🔍 Debugging & Observability
//! - **Telemetry Link**: Run graph intelligence commands or inspect stdout.
//!
//! This binary intentionally stays small and independent from the server boot path.
//! It reuses the production parser and graph modules, then emits targeted JSON or
//! readable summaries that shell-based audits can consume.

#[path = "../utils/parser.rs"]
pub mod parser;

pub mod utils {
    pub use crate::parser;
}

#[path = "../intelligence/graph.rs"]
mod graph;

use clap::{Parser, Subcommand};
use graph::{CodeSymbolGraph, SymbolNode};
use petgraph::graph::NodeIndex;
use petgraph::visit::EdgeRef;
use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};

static AUDIT_USAGE_TEMPLATES: &[&str] = &[
    "npm run graph:lookup -- --name SymbolName",
    "npm run graph:file -- --path src/pages/Neural_Map.tsx",
    "npm run graph:blast -- --path server-rs/src/routes/intelligence.rs --name get_blast_radius",
    "npm run graph:export",
];

#[derive(Parser, Debug)]
#[command(
    name = "graph_query",
    about = "Scriptable symbol graph queries for coding agents and audits"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Export the full symbol graph
    Export {
        #[arg(long, default_value = ".")]
        root: PathBuf,
        #[arg(long)]
        out: Option<PathBuf>,
        #[arg(long)]
        pretty: bool,
    },
    /// Generate high-connectivity audit context
    AuditContext {
        #[arg(long, default_value = ".")]
        root: PathBuf,
        #[arg(long)]
        out: Option<PathBuf>,
        #[arg(long, default_value_t = 20, value_parser = clap::value_parser!(u32).range(1..1000))]
        limit: u32,
    },
    /// Lookup symbols by name
    Lookup {
        #[arg(long)]
        name: String,
        #[arg(long, default_value = ".")]
        root: PathBuf,
        #[arg(long, default_value_t = 20, value_parser = clap::value_parser!(u32).range(1..1000))]
        limit: u32,
        #[arg(long)]
        json: bool,
        #[arg(long)]
        out: Option<PathBuf>,
    },
    /// Query symbol context for a specific file path
    File {
        #[arg(long)]
        path: String,
        #[arg(long, default_value = ".")]
        root: PathBuf,
        #[arg(long, default_value_t = 20, value_parser = clap::value_parser!(u32).range(1..1000))]
        limit: u32,
        #[arg(long)]
        json: bool,
        #[arg(long)]
        out: Option<PathBuf>,
    },
    /// Calculate the blast radius of a symbol in a file
    Blast {
        #[arg(long)]
        name: String,
        #[arg(long)]
        path: String,
        #[arg(long, default_value = ".")]
        root: PathBuf,
        #[arg(long)]
        json: bool,
        #[arg(long)]
        out: Option<PathBuf>,
    },
}

#[derive(Debug, thiserror::Error)]
enum GraphQueryError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("Graph build error: {0}")]
    GraphBuild(String),
    #[error("Security violation: {0}")]
    Security(String),
}

#[derive(Serialize, Clone)]
struct ReportMetadata {
    generated_at: String,
    root: String,
    stats: GraphStats,
}

#[derive(Serialize)]
struct GraphExport {
    #[serde(flatten)]
    metadata: ReportMetadata,
    nodes: Vec<SymbolNode>,
    links: Vec<GraphLink>,
    anomalies: Vec<String>,
}

#[derive(Serialize)]
struct AuditContext {
    #[serde(flatten)]
    metadata: ReportMetadata,
    top_connected: Vec<SymbolContext>,
    anomalies: Vec<String>,
    usage: Vec<String>,
}

#[derive(Clone, Serialize)]
struct GraphStats {
    node_count: usize,
    edge_count: usize,
    anomaly_count: usize,
}

#[derive(Serialize)]
struct GraphLink {
    source: String,
    target: String,
}

#[derive(Serialize)]
struct SymbolContext {
    id: String,
    node: SymbolNode,
    callers: Vec<SymbolNode>,
    callees: Vec<SymbolNode>,
    blast_radius: Vec<SymbolNode>,
}

#[derive(Serialize)]
struct FileContext {
    path: String,
    symbols: Vec<SymbolContext>,
}

/// Manager facilitating code symbol graph lookup and context extraction queries.
///
/// ### Lifetime Invariant
/// The lifetime `'a` of the `GraphQueryManager` reference scope is bound to the backing
/// `CodeSymbolGraph` instance. The query manager instance cannot outlive the initialized graph.
struct GraphQueryManager<'a> {
    graph: &'a CodeSymbolGraph,
}

impl<'a> GraphQueryManager<'a> {
    fn new(graph: &'a CodeSymbolGraph) -> Self {
        Self { graph }
    }

    fn lookup_by_name(&self, name: &str, limit: usize) -> Vec<SymbolContext> {
        self.graph
            .graph
            .node_indices()
            .filter(|idx| self.graph.graph[*idx].name == name)
            .take(limit)
            .map(|idx| self.symbol_context(idx, limit))
            .collect()
    }

    fn lookup_by_file(&self, path: &str, limit: usize) -> Vec<SymbolContext> {
        self.graph
            .graph
            .node_indices()
            .filter(|idx| self.graph.graph[*idx].path == path)
            .take(limit)
            .map(|idx| self.symbol_context(idx, limit))
            .collect()
    }

    fn symbol_context(&self, idx: NodeIndex, limit: usize) -> SymbolContext {
        let callers = self.related_nodes(idx, petgraph::Direction::Incoming, limit)
            .into_iter()
            .cloned()
            .collect();
        let callees = self.related_nodes(idx, petgraph::Direction::Outgoing, limit)
            .into_iter()
            .cloned()
            .collect();
        let node = self.graph.graph[idx].clone();
        let blast_radius = self.graph
            .calculate_blast_radius(&node.name, &node.path)
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

fn main() {
    if let Err(err) = run() {
        eprintln!("graph_query: {err}");
        std::process::exit(1);
    }
}

fn setup_graph(root_path: &Path) -> Result<(CodeSymbolGraph, PathBuf), GraphQueryError> {
    let root = root_path
        .canonicalize()
        .map_err(|e| GraphQueryError::Security(format!("failed to resolve root {}: {e}", root_path.display())))?;
    let mut graph = CodeSymbolGraph::new(root.clone());
    let salt = uuid::Uuid::new_v4().to_string();
    graph.build(&salt).map_err(|e| GraphQueryError::GraphBuild(e.to_string()))?;
    Ok((graph, root))
}

fn run() -> Result<(), GraphQueryError> {
    let args = Cli::parse();

    let root_path = match &args.command {
        Commands::Export { root, .. } => root,
        Commands::AuditContext { root, .. } => root,
        Commands::Lookup { root, .. } => root,
        Commands::File { root, .. } => root,
        Commands::Blast { root, .. } => root,
    };

    let (graph, root) = setup_graph(root_path)?;
    let query_manager = GraphQueryManager::new(&graph);

    match args.command {
        Commands::Export { out, pretty, .. } => {
            let payload = build_export(&graph, &root);
            emit_json(&payload, out.as_ref(), pretty)?;
        }
        Commands::AuditContext { out, limit, .. } => {
            let payload = build_audit_context(&graph, &root, limit as usize);
            emit_json(&payload, out.as_ref(), true)?;
        }
        Commands::Lookup { name, limit, json, out, .. } => {
            let contexts = query_manager.lookup_by_name(&name, limit as usize);
            emit_query(
                &contexts,
                json || out.is_some(),
                out.as_ref(),
            )?;
        }
        Commands::File { path, limit, json, out, .. } => {
            let file_path = normalize_query_path(&root, &path)?;
            let payload = FileContext {
                path: file_path.clone(),
                symbols: query_manager.lookup_by_file(&file_path, limit as usize),
            };
            emit_query(
                &payload,
                json || out.is_some(),
                out.as_ref(),
            )?;
        }
        Commands::Blast { name, path, json, out, .. } => {
            let file_path = normalize_query_path(&root, &path)?;
            let blast_radius = graph.calculate_blast_radius(&name, &file_path);
            emit_query(
                &blast_radius,
                json || out.is_some(),
                out.as_ref(),
            )?;
        }
    }

    Ok(())
}

fn build_export(graph: &CodeSymbolGraph, root: &Path) -> GraphExport {
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

fn build_audit_context(graph: &CodeSymbolGraph, root: &Path, limit: usize) -> AuditContext {
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
        usage: AUDIT_USAGE_TEMPLATES.iter().map(|s| s.to_string()).collect(),
    }
}

fn lexical_normalize(path: &Path) -> PathBuf {
    let mut resolved = PathBuf::new();
    for component in path.components() {
        match component {
            std::path::Component::Prefix(p) => {
                resolved.push(p.as_os_str());
            }
            std::path::Component::RootDir => {
                resolved.push(std::path::Component::RootDir.as_os_str());
            }
            std::path::Component::CurDir => {}
            std::path::Component::ParentDir => {
                resolved.pop();
            }
            std::path::Component::Normal(c) => {
                resolved.push(c);
            }
        }
    }
    resolved
}

fn normalize_query_path(root: &Path, raw: &str) -> Result<String, GraphQueryError> {
    let normalized_str = raw.replace('\\', "/");
    let input_path = Path::new(&normalized_str);
    let absolute_target = if input_path.is_absolute() {
        input_path.to_path_buf()
    } else {
        root.join(input_path)
    };

    let canonical_root = root.canonicalize()
        .map_err(|e| GraphQueryError::Security(format!("Failed to canonicalize root: {e}")))?;

    let resolved_target = lexical_normalize(&absolute_target);

    let canonical_target = match resolved_target.canonicalize() {
        Ok(path) => path,
        Err(_) => resolved_target,
    };

    if !canonical_target.starts_with(&canonical_root) {
        return Err(GraphQueryError::Security(format!(
            "Path traversal detected! Target path '{}' is outside root directory '{}'",
            canonical_target.display(),
            canonical_root.display()
        )));
    }

    let relative = canonical_target
        .strip_prefix(&canonical_root)
        .map_err(|e| GraphQueryError::Security(format!("Failed to strip root prefix: {e}")))?;

    Ok(relative.to_string_lossy().replace('\\', "/"))
}

fn node_id(node: &SymbolNode) -> String {
    format!("{}:{}", node.path, node.name)
}

fn emit_json<T: Serialize>(payload: &T, out: Option<&PathBuf>, pretty: bool) -> Result<(), GraphQueryError> {
    let json = if pretty {
        serde_json::to_string_pretty(payload)
    } else {
        serde_json::to_string(payload)
    }?;

    if let Some(out_path) = out {
        if let Some(parent) = out_path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(out_path, json)?;
        println!("Wrote {}", out_path.display());
    } else {
        println!("{json}");
    }
    Ok(())
}

fn emit_query<T: Serialize>(payload: &T, json: bool, out: Option<&PathBuf>) -> Result<(), GraphQueryError> {
    if json || out.is_some() {
        emit_json(payload, out, true)
    } else {
        let value = serde_json::to_value(payload)?;
        print_human(&value);
        Ok(())
    }
}

fn print_human(value: &serde_json::Value) {
    match value {
        serde_json::Value::Array(items) => {
            if items.is_empty() {
                println!("No graph matches.");
                return;
            }
            for item in items {
                print_context(item);
            }
        }
        serde_json::Value::Object(_) => print_context(value),
        _ => println!("{value}"),
    }
}

fn print_context(value: &serde_json::Value) {
    if let Some(symbols) = value.get("symbols").and_then(|v| v.as_array()) {
        println!("File: {}", value["path"].as_str().unwrap_or(""));
        for symbol in symbols {
            print_context(symbol);
        }
        return;
    }

    let Some(node) = value.get("node") else {
        println!("{value}");
        return;
    };

    let name = node["name"].as_str().unwrap_or("");
    let path = node["path"].as_str().unwrap_or("");
    let kind = node["kind"].as_str().unwrap_or("");
    let start = node["start_line"].as_u64().unwrap_or(0);
    let end = node["end_line"].as_u64().unwrap_or(0);
    println!("{kind} {name} @ {path}:{start}-{end}");
    print_count("callers", value);
    print_count("callees", value);
    print_count("blast_radius", value);
    println!();
}

fn print_count(key: &str, value: &serde_json::Value) {
    let count = value
        .get(key)
        .and_then(|v| v.as_array())
        .map(|items| items.len())
        .unwrap_or(0);
    println!("  {key}: {count}");
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_lexical_normalize() {
        assert_eq!(
            lexical_normalize(Path::new("D:/foo/bar/../baz")),
            PathBuf::from("D:/foo/baz")
        );
        assert_eq!(
            lexical_normalize(Path::new("foo/bar/../baz")),
            PathBuf::from("foo/baz")
        );
    }

    #[test]
    fn test_normalize_query_path_happy_path() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().canonicalize().unwrap();

        let sub = root.join("src").join("pages");
        fs::create_dir_all(&sub).unwrap();
        let file = sub.join("Neural_Map.tsx");
        fs::write(&file, "").unwrap();

        let norm1 = normalize_query_path(&root, "./src/pages/Neural_Map.tsx").unwrap();
        assert_eq!(norm1, "src/pages/Neural_Map.tsx");

        let norm2 = normalize_query_path(&root, &file.to_string_lossy()).unwrap();
        assert_eq!(norm2, "src/pages/Neural_Map.tsx");
    }

    #[test]
    fn test_normalize_query_path_traversal_attack() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().canonicalize().unwrap();

        let raw = "../etc/passwd";
        let res = normalize_query_path(&root, raw);
        assert!(res.is_err());
        if let Err(GraphQueryError::Security(msg)) = res {
            assert!(msg.contains("Path traversal detected"));
        } else {
            panic!("Expected Security error, got: {:?}", res);
        }
    }

    #[test]
    fn test_graph_query_manager_circular_dependency() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().to_path_buf();

        let file_path = root.join("main.rs");
        let mut file = fs::File::create(&file_path).unwrap();
        writeln!(file, "fn alpha() {{ beta(); }}").unwrap();
        writeln!(file, "fn beta() {{ alpha(); }}").unwrap();
        drop(file);

        let mut graph = CodeSymbolGraph::new(root.clone());
        let salt = "salt".to_string();
        graph.build(&salt).unwrap();

        let manager = GraphQueryManager::new(&graph);

        let contexts = manager.lookup_by_name("alpha", 10);
        assert_eq!(contexts.len(), 1);
        let ctx = &contexts[0];

        assert!(ctx.callers.iter().any(|node| node.name == "beta"));
        assert!(ctx.callees.iter().any(|node| node.name == "beta"));
        assert!(ctx.blast_radius.iter().any(|node| node.name == "beta"));
    }

    #[test]
    fn test_graph_query_manager_disconnected_subgraphs() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().to_path_buf();

        let file_path = root.join("main.rs");
        let mut file = fs::File::create(&file_path).unwrap();
        writeln!(file, "fn a() {{ b(); }}").unwrap();
        writeln!(file, "fn b() {{ }}").unwrap();
        writeln!(file, "fn x() {{ y(); }}").unwrap();
        writeln!(file, "fn y() {{ }}").unwrap();
        drop(file);

        let mut graph = CodeSymbolGraph::new(root.clone());
        let salt = "salt".to_string();
        graph.build(&salt).unwrap();

        let manager = GraphQueryManager::new(&graph);

        let contexts_a = manager.lookup_by_name("a", 10);
        assert_eq!(contexts_a.len(), 1);
        let ctx_a = &contexts_a[0];

        assert!(ctx_a.callees.iter().any(|n| n.name == "b"));
        assert!(!ctx_a.callees.iter().any(|n| n.name == "y"));
        assert!(!ctx_a.callees.iter().any(|n| n.name == "x"));
    }

    #[test]
    fn test_setup_graph_failure() {
        let nonexistent = Path::new("D:/this/directory/does/not/exist");
        let res = setup_graph(nonexistent);
        assert!(res.is_err());
    }

    #[test]
    fn test_query_limits_clamped() {
        let invalid_args = vec!["graph_query", "audit-context", "--limit", "1500"];
        let res = Cli::try_parse_from(invalid_args);
        assert!(res.is_err(), "Limit greater than 1000 should be rejected by parser");

        let invalid_args_zero = vec!["graph_query", "audit-context", "--limit", "0"];
        let res_zero = Cli::try_parse_from(invalid_args_zero);
        assert!(res_zero.is_err(), "Limit of 0 should be rejected by parser");

        let valid_args = vec!["graph_query", "audit-context", "--limit", "500"];
        let res_valid = Cli::try_parse_from(valid_args);
        assert!(res_valid.is_ok(), "Limit between 1 and 999 should be accepted");
    }
}

// Metadata: [graph_query]

// Metadata: [graph_query]

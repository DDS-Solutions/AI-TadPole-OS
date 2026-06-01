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

use graph::{CodeSymbolGraph, SymbolNode};
use petgraph::graph::NodeIndex;
use petgraph::visit::EdgeRef;
use serde::Serialize;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug)]
struct Options {
    command: String,
    root: PathBuf,
    out: Option<PathBuf>,
    name: Option<String>,
    path: Option<String>,
    limit: usize,
    json: bool,
    pretty: bool,
}

#[derive(Serialize)]
struct GraphExport {
    generated_at: String,
    root: String,
    stats: GraphStats,
    nodes: Vec<SymbolNode>,
    links: Vec<GraphLink>,
    anomalies: Vec<String>,
}

#[derive(Serialize)]
struct AuditContext {
    generated_at: String,
    root: String,
    stats: GraphStats,
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

fn main() {
    if let Err(err) = run() {
        eprintln!("graph_query: {err}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let options = parse_args(env::args().skip(1).collect())?;
    if options.command == "help" {
        print_help();
        return Ok(());
    }

    let root = options
        .root
        .canonicalize()
        .map_err(|e| format!("failed to resolve root {}: {e}", options.root.display()))?;
    let mut graph = CodeSymbolGraph::new(root.clone());
    let salt = uuid::Uuid::new_v4().to_string();
    graph.build(&salt);

    match options.command.as_str() {
        "export" => {
            let payload = build_export(&graph, &root);
            emit_json(&payload, options.out.as_ref(), options.pretty)?;
        }
        "audit-context" => {
            let payload = build_audit_context(&graph, &root, options.limit);
            emit_json(&payload, options.out.as_ref(), true)?;
        }
        "lookup" => {
            let name = required(&options.name, "--name")?;
            let contexts = lookup_by_name(&graph, name, options.limit);
            emit_query(&contexts, options.json || options.out.is_some(), options.out.as_ref())?;
        }
        "file" => {
            let path = required(&options.path, "--path")?;
            let file_path = normalize_query_path(&root, path);
            let payload = FileContext {
                path: file_path.clone(),
                symbols: lookup_by_file(&graph, &file_path, options.limit),
            };
            emit_query(&payload, options.json || options.out.is_some(), options.out.as_ref())?;
        }
        "blast" => {
            let name = required(&options.name, "--name")?;
            let path = required(&options.path, "--path")?;
            let file_path = normalize_query_path(&root, path);
            let blast_radius = graph.calculate_blast_radius(name, &file_path);
            emit_query(&blast_radius, options.json || options.out.is_some(), options.out.as_ref())?;
        }
        unknown => return Err(format!("unknown command '{unknown}'. Run graph_query help.")),
    }

    Ok(())
}

fn parse_args(args: Vec<String>) -> Result<Options, String> {
    let mut command = "help".to_string();
    let mut root = env::current_dir().map_err(|e| format!("failed to read current dir: {e}"))?;
    let mut out = None;
    let mut name = None;
    let mut path = None;
    let mut limit = 20usize;
    let mut json = false;
    let mut pretty = false;

    let mut i = 0;
    if let Some(first) = args.first() {
        if !first.starts_with("--") {
            command = first.clone();
            i = 1;
        }
    }

    while i < args.len() {
        match args[i].as_str() {
            "--root" => {
                i += 1;
                root = PathBuf::from(required_arg(&args, i, "--root")?);
            }
            "--out" => {
                i += 1;
                out = Some(PathBuf::from(required_arg(&args, i, "--out")?));
            }
            "--name" => {
                i += 1;
                name = Some(required_arg(&args, i, "--name")?);
            }
            "--path" => {
                i += 1;
                path = Some(required_arg(&args, i, "--path")?);
            }
            "--limit" => {
                i += 1;
                let raw = required_arg(&args, i, "--limit")?;
                limit = raw
                    .parse::<usize>()
                    .map_err(|_| format!("--limit must be a positive integer, got '{raw}'"))?;
            }
            "--json" => json = true,
            "--pretty" => pretty = true,
            "--help" | "-h" => command = "help".to_string(),
            other => return Err(format!("unknown option '{other}'")),
        }
        i += 1;
    }

    Ok(Options {
        command,
        root,
        out,
        name,
        path,
        limit,
        json,
        pretty,
    })
}

fn required_arg(args: &[String], index: usize, flag: &str) -> Result<String, String> {
    args.get(index)
        .cloned()
        .ok_or_else(|| format!("{flag} requires a value"))
}

fn required<'a>(value: &'a Option<String>, flag: &str) -> Result<&'a str, String> {
    value
        .as_deref()
        .ok_or_else(|| format!("missing required {flag}"))
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
        generated_at: chrono::Utc::now().to_rfc3339(),
        root: root.display().to_string(),
        stats,
        nodes,
        links,
        anomalies,
    }
}

fn build_audit_context(graph: &CodeSymbolGraph, root: &Path, limit: usize) -> AuditContext {
    let export = build_export(graph, root);
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
        .map(|(idx, _)| symbol_context(graph, idx, limit))
        .collect::<Vec<_>>();

    AuditContext {
        generated_at: export.generated_at,
        root: export.root,
        stats: export.stats,
        top_connected,
        anomalies: export.anomalies.into_iter().take(limit).collect(),
        usage: vec![
            "npm run graph:lookup -- --name SymbolName".to_string(),
            "npm run graph:file -- --path src/pages/Neural_Map.tsx".to_string(),
            "npm run graph:blast -- --path server-rs/src/routes/intelligence.rs --name get_blast_radius".to_string(),
            "npm run graph:export".to_string(),
        ],
    }
}

fn lookup_by_name(graph: &CodeSymbolGraph, name: &str, limit: usize) -> Vec<SymbolContext> {
    graph
        .graph
        .node_indices()
        .filter(|idx| graph.graph[*idx].name == name)
        .take(limit)
        .map(|idx| symbol_context(graph, idx, limit))
        .collect()
}

fn lookup_by_file(graph: &CodeSymbolGraph, path: &str, limit: usize) -> Vec<SymbolContext> {
    graph
        .graph
        .node_indices()
        .filter(|idx| graph.graph[*idx].path == path)
        .take(limit)
        .map(|idx| symbol_context(graph, idx, limit))
        .collect()
}

fn symbol_context(graph: &CodeSymbolGraph, idx: NodeIndex, limit: usize) -> SymbolContext {
    let callers = related_nodes(graph, idx, petgraph::Direction::Incoming, limit);
    let callees = related_nodes(graph, idx, petgraph::Direction::Outgoing, limit);
    let node = graph.graph[idx].clone();
    let blast_radius = graph
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
    graph: &CodeSymbolGraph,
    idx: NodeIndex,
    direction: petgraph::Direction,
    limit: usize,
) -> Vec<SymbolNode> {
    graph
        .graph
        .edges_directed(idx, direction)
        .map(|edge| match direction {
            petgraph::Direction::Incoming => edge.source(),
            petgraph::Direction::Outgoing => edge.target(),
        })
        .take(limit)
        .map(|neighbor| graph.graph[neighbor].clone())
        .collect()
}

fn normalize_query_path(root: &Path, raw: &str) -> String {
    let normalized = raw.replace('\\', "/");
    let as_path = PathBuf::from(&normalized);
    if as_path.is_absolute() {
        if let Ok(stripped) = as_path.strip_prefix(root) {
            return stripped.to_string_lossy().replace('\\', "/");
        }
    }
    normalized.trim_start_matches("./").to_string()
}

fn node_id(node: &SymbolNode) -> String {
    format!("{}:{}", node.path, node.name)
}

fn emit_json<T: Serialize>(payload: &T, out: Option<&PathBuf>, pretty: bool) -> Result<(), String> {
    let json = if pretty {
        serde_json::to_string_pretty(payload)
    } else {
        serde_json::to_string(payload)
    }
    .map_err(|e| format!("failed to serialize graph payload: {e}"))?;

    if let Some(out_path) = out {
        if let Some(parent) = out_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("failed to create {}: {e}", parent.display()))?;
        }
        fs::write(out_path, json)
            .map_err(|e| format!("failed to write {}: {e}", out_path.display()))?;
        println!("Wrote {}", out_path.display());
    } else {
        println!("{json}");
    }
    Ok(())
}

fn emit_query<T: Serialize>(payload: &T, json: bool, out: Option<&PathBuf>) -> Result<(), String> {
    if json || out.is_some() {
        emit_json(payload, out, true)
    } else {
        let value = serde_json::to_value(payload)
            .map_err(|e| format!("failed to serialize query payload: {e}"))?;
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

fn print_help() {
    println!(
        "Usage:
  graph_query export [--root PATH] [--out PATH] [--pretty]
  graph_query audit-context [--root PATH] [--out PATH] [--limit N]
  graph_query lookup --name NAME [--root PATH] [--limit N] [--json]
  graph_query file --path PATH [--root PATH] [--limit N] [--json]
  graph_query blast --path PATH --name NAME [--root PATH] [--json]

Examples:
  npm run graph:lookup -- --name get_blast_radius
  npm run graph:file -- --path server-rs/src/routes/intelligence.rs
  npm run graph:blast -- --path server-rs/src/routes/intelligence.rs --name get_blast_radius
  npm run graph:export"
    );
}

// Metadata: [graph_query]

// Metadata: [graph_query]

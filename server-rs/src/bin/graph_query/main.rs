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
//!
//! ### AI Assist Note
//! **Graph Query CLI**: Command line interface for querying the symbol graph, calculating blast radius,
//! and exporting audit contexts.
//!
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: Parameter parsing errors, file path traversal.
//! - **Trace Scope**: `server-rs::bin::graph_query::main`

#[path = "../../utils/parser.rs"]
pub mod parser;

pub mod utils {
    pub use crate::parser;
}

#[path = "../../intelligence/graph.rs"]
pub mod graph;

pub mod doc_guard;
pub mod path_utils;
pub mod query_manager;
pub mod visualizer;

use clap::{Parser, Subcommand};
use graph::CodeSymbolGraph;
use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Parser, Debug)]
#[command(
    name = "graph_query",
    about = "Scriptable symbol graph queries for coding agents and audits"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
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
        #[arg(long, default_value_t = 50)]
        depth: usize,
        #[arg(long, default_value = "text")]
        format: String,
    },
    /// Validate backticked symbols inside docstrings against the graph connections
    Validate {
        #[arg(long, default_value = ".")]
        root: PathBuf,
        #[arg(long)]
        strict: bool,
        #[arg(long)]
        out: Option<PathBuf>,
        #[arg(long)]
        diff: bool,
        #[arg(long)]
        fix: bool,
    },
}

#[derive(Debug, thiserror::Error)]
pub enum GraphQueryError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("Graph build error: {0}")]
    GraphBuild(String),
    #[error("Security violation: {0}")]
    Security(String),
}

fn main() {
    if let Err(err) = run() {
        eprintln!("graph_query: {err}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), GraphQueryError> {
    let args = Cli::parse();

    let root_path = match &args.command {
        Commands::Export { root, .. } => root,
        Commands::AuditContext { root, .. } => root,
        Commands::Lookup { root, .. } => root,
        Commands::File { root, .. } => root,
        Commands::Blast { root, .. } => root,
        Commands::Validate { root, .. } => root,
    };

    let (graph, root) = setup_graph(root_path)?;
    let query_manager = query_manager::GraphQueryManager::new(&graph);

    match args.command {
        Commands::Export { out, pretty, .. } => {
            let payload = query_manager::build_export(&graph, &root);
            emit_json(&payload, out.as_ref(), pretty)?;
        }
        Commands::AuditContext { out, limit, .. } => {
            let payload = query_manager::build_audit_context(&graph, &root, limit as usize);
            emit_json(&payload, out.as_ref(), true)?;
        }
        Commands::Lookup {
            name,
            limit,
            json,
            out,
            ..
        } => {
            let contexts = query_manager.lookup_by_name(&name, limit as usize);
            emit_query(&contexts, json || out.is_some(), out.as_ref())?;
        }
        Commands::File {
            path,
            limit,
            json,
            out,
            ..
        } => {
            let file_path = path_utils::normalize_query_path(&root, &path)?;
            let payload = query_manager::FileContext {
                path: file_path.clone(),
                symbols: query_manager.lookup_by_file(&file_path, limit as usize),
            };
            emit_query(&payload, json || out.is_some(), out.as_ref())?;
        }
        Commands::Blast {
            name,
            path,
            json,
            out,
            depth,
            format,
            ..
        } => {
            let file_path = path_utils::normalize_query_path(&root, &path)?;
            let blast_radius = graph.calculate_blast_radius(&name, &file_path, depth);

            if format == "mermaid" {
                let mermaid_content = visualizer::generate_mermaid_diagram(&blast_radius, &graph);
                if let Some(ref out_path) = out {
                    if let Some(parent) = out_path.parent() {
                        fs::create_dir_all(parent).map_err(GraphQueryError::Io)?;
                    }
                    fs::write(out_path, &mermaid_content).map_err(GraphQueryError::Io)?;
                    println!("Saved Mermaid diagram to: {}", out_path.display());
                } else {
                    println!("{}", mermaid_content);
                }
            } else if format == "html" {
                let html_content = visualizer::generate_html_visualizer(&blast_radius, &graph);
                if let Some(ref out_path) = out {
                    if let Some(parent) = out_path.parent() {
                        fs::create_dir_all(parent).map_err(GraphQueryError::Io)?;
                    }
                    fs::write(out_path, &html_content).map_err(GraphQueryError::Io)?;
                    println!("Saved HTML visualizer to: {}", out_path.display());
                } else {
                    println!("{}", html_content);
                }
            } else {
                emit_query(&blast_radius, json || out.is_some(), out.as_ref())?;
            }
        }
        Commands::Validate {
            strict,
            out,
            diff,
            fix,
            ..
        } => {
            doc_guard::validate_graph_docstrings(&graph, &root, strict, out.as_deref(), diff, fix)?;
        }
    }

    Ok(())
}

fn setup_graph(root_path: &Path) -> Result<(CodeSymbolGraph, PathBuf), GraphQueryError> {
    let root = root_path.canonicalize().map_err(|e| {
        GraphQueryError::Security(format!(
            "failed to resolve root {}: {e}",
            root_path.display()
        ))
    })?;
    let mut graph = CodeSymbolGraph::new(root.clone());
    let salt = graph::derive_stable_salt(&root);
    graph
        .build(&salt)
        .map_err(|e| GraphQueryError::GraphBuild(e.to_string()))?;
    Ok((graph, root))
}

fn emit_json<T: Serialize>(
    payload: &T,
    out: Option<&PathBuf>,
    pretty: bool,
) -> Result<(), GraphQueryError> {
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

fn emit_query<T: Serialize>(
    payload: &T,
    json: bool,
    out: Option<&PathBuf>,
) -> Result<(), GraphQueryError> {
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
    use crate::doc_guard::{extract_backticked_symbols, filter_symbol, validate_graph_docstrings};
    use crate::path_utils::{lexical_normalize, normalize_query_path};
    use crate::query_manager::GraphQueryManager;
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
        assert!(
            res.is_err(),
            "Limit greater than 1000 should be rejected by parser"
        );

        let invalid_args_zero = vec!["graph_query", "audit-context", "--limit", "0"];
        let res_zero = Cli::try_parse_from(invalid_args_zero);
        assert!(res_zero.is_err(), "Limit of 0 should be rejected by parser");

        let valid_args = vec!["graph_query", "audit-context", "--limit", "500"];
        let res_valid = Cli::try_parse_from(valid_args);
        assert!(
            res_valid.is_ok(),
            "Limit between 1 and 999 should be accepted"
        );
    }

    #[test]
    fn test_extract_backticked_symbols() {
        let doc = "This contains `some_symbol` and `another_one()`. Also `multiple\nlines` should not match.";
        let res = extract_backticked_symbols(doc);
        assert_eq!(res, vec!["some_symbol", "another_one"]);
    }

    #[test]
    fn test_filter_symbol() {
        let mut whitelist = std::collections::HashSet::new();
        whitelist.insert("true".to_string());
        whitelist.insert("u8".to_string());

        assert_eq!(
            filter_symbol("some_var", false, &whitelist),
            Some("some_var".to_string())
        );
        assert_eq!(
            filter_symbol("some_var()", false, &whitelist),
            Some("some_var".to_string())
        );
        assert_eq!(filter_symbol("true", false, &whitelist), None);
        assert_eq!(filter_symbol("invalid-name", false, &whitelist), None);
        assert_eq!(filter_symbol("invalid/path", false, &whitelist), None);

        assert_eq!(
            filter_symbol("valid-wiki-link", true, &whitelist),
            Some("valid-wiki-link".to_string())
        );
        assert_eq!(
            filter_symbol("path/to/doc.md", true, &whitelist),
            Some("path/to/doc.md".to_string())
        );
        assert_eq!(filter_symbol("https://tadpole.os", true, &whitelist), None);
    }

    #[test]
    fn test_validate_graph_docstrings_success() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().to_path_buf();

        let file_path = root.join("main.rs");
        let mut file = fs::File::create(&file_path).unwrap();
        writeln!(file, "/// Refers to `alpha`").unwrap();
        writeln!(file, "fn alpha() {{}}").unwrap();
        drop(file);

        let mut graph = CodeSymbolGraph::new(root.clone());
        let salt = "salt".to_string();
        graph.build(&salt).unwrap();

        let res = validate_graph_docstrings(&graph, &root, true, None, false, false);
        assert!(
            res.is_ok(),
            "Validation should pass since 'alpha' exists in the file"
        );
    }

    #[test]
    fn test_validate_graph_docstrings_failure() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().to_path_buf();

        let file_path = root.join("main.rs");
        let mut file = fs::File::create(&file_path).unwrap();
        writeln!(file, "/// Refers to `non_existent`").unwrap();
        writeln!(file, "fn alpha() {{}}").unwrap();
        drop(file);

        let mut graph = CodeSymbolGraph::new(root.clone());
        let salt = "salt".to_string();
        graph.build(&salt).unwrap();

        let res = validate_graph_docstrings(&graph, &root, true, None, false, false);
        assert!(
            res.is_err(),
            "Validation should fail because 'non_existent' does not exist in code"
        );
    }

    #[test]
    fn test_validate_graph_docstrings_auto_fix() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().to_path_buf();

        let file_path = root.join("main.rs");
        let mut file = fs::File::create(&file_path).unwrap();
        writeln!(file, "/// Refers to `alph`").unwrap();
        writeln!(file, "fn alpha() {{}}").unwrap();
        drop(file);

        let mut graph = CodeSymbolGraph::new(root.clone());
        let salt = "salt".to_string();
        graph.build(&salt).unwrap();

        // 1. Run validation with fix disabled. It should fail.
        let res_no_fix = validate_graph_docstrings(&graph, &root, true, None, false, false);
        assert!(res_no_fix.is_err());

        // 2. Run validation with fix enabled. It should succeed and write it to disk.
        let res_fix = validate_graph_docstrings(&graph, &root, true, None, false, true);
        assert!(res_fix.is_ok());

        // 3. Read file back to verify the docstring has been replaced with `alpha`.
        let content = fs::read_to_string(&file_path).unwrap();
        assert!(content.contains("/// Refers to `alpha`"));
    }
}

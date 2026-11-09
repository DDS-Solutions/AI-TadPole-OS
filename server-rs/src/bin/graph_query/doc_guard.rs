//! @docs ARCHITECTURE:CodeBaseIntelligence
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / doc_guard
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Type-safe state handling and bounded execution without unhandled panics.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: `[FAIL]`, `[ADG-SG]`, `[OK]`
//! - **Witness Tests**: none declared

use crate::graph::{CodeSymbolGraph, SymbolNode};
use crate::parser::SymbolRange;
use crate::path_utils::get_git_modified_files;
use crate::GraphQueryError;
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;

pub mod autofix;
pub mod report;
pub mod symbols;

pub use autofix::{apply_autofixes, AutoFixContext};
pub use report::{
    print_audit_summary, save_report, AuditStats, DocGuardOptions, FileValidationFailure,
    ReportPayload, ValidationReport,
};
pub use symbols::{
    build_whitelist, disk_candidate_exists, extract_backticked_symbols, filter_symbol,
    tokenize_code_body, BUILTIN_KEYWORDS,
};

struct SymbolCheckContext<'a> {
    sym: &'a str,
    real_path: &'a str,
    full_path: &'a Path,
    root: &'a Path,
    graph: &'a CodeSymbolGraph,
    global_symbols: &'a HashSet<&'a str>,
    code_tokens: &'a HashSet<String>,
}

fn check_symbol_exists(ctx: &SymbolCheckContext<'_>) -> bool {
    if let Some((symbols, _)) = ctx.graph.repository.parse_cache.get(ctx.real_path) {
        if symbols.iter().any(|s| s.name == ctx.sym) {
            return true;
        }
    }
    if let Some((_, refs)) = ctx.graph.repository.parse_cache.get(ctx.real_path) {
        if refs.iter().any(|r| r.name == ctx.sym) {
            return true;
        }
    }
    if ctx.global_symbols.contains(ctx.sym) {
        return true;
    }
    if ctx.sym.contains('/') || ctx.sym.contains('\\') || ctx.sym.contains('.') {
        let parent = ctx.full_path.parent().unwrap_or(ctx.root);
        if disk_candidate_exists(ctx.root, parent, ctx.sym) {
            return true;
        }
    }
    ctx.code_tokens.contains(ctx.sym)
}

fn find_best_suggestion<'a>(
    real_path: &str,
    sym: &str,
    graph: &'a CodeSymbolGraph,
) -> Option<&'a str> {
    let (symbols, _) = graph.repository.parse_cache.get(real_path)?;
    let mut best: Option<(&str, f64)> = None;
    for candidate in symbols {
        let sim = strsim::jaro_winkler(sym, &candidate.name);
        if sim > 0.8 && best.as_ref().map_or(true, |(_, b_sim)| sim > *b_sim) {
            best = Some((candidate.name.as_str(), sim));
        }
    }
    best.map(|(name, _)| name)
}

fn record_suggestion_edit(
    real_path: &str,
    sym: &str,
    graph: &CodeSymbolGraph,
    range: Option<&SymbolRange>,
    edits: &mut Vec<(SymbolRange, String, String)>,
) {
    let Some(suggested) = find_best_suggestion(real_path, sym, graph) else {
        return;
    };
    println!("   💡 Suggestion: Did you mean `{suggested}`?");
    if let Some(r) = range {
        edits.push((r.clone(), sym.to_string(), suggested.to_string()));
    }
}

struct NodeAuditContext<'a> {
    real_path: &'a str,
    full_path: &'a Path,
    root: &'a Path,
    graph: &'a CodeSymbolGraph,
    global_symbols: &'a HashSet<&'a str>,
    code_tokens: &'a HashSet<String>,
    whitelist: &'a HashSet<String>,
}

struct FileAuditOutcome {
    checked_symbols: usize,
    missing_symbols: Vec<String>,
    file_edits: Vec<(SymbolRange, String, String)>,
}

fn audit_node_docstrings(nodes: &[&SymbolNode], ctx: &NodeAuditContext<'_>) -> FileAuditOutcome {
    let is_md = ctx.real_path.ends_with(".md");
    let mut checked_symbols = 0;
    let mut missing_symbols = Vec::new();
    let mut file_edits = Vec::new();
    let mut seen_ranges = HashSet::new();

    for node in nodes {
        let Some(ref doc) = node.docstring else {
            continue;
        };
        if let Some(ref range) = node.docstring_range {
            if !seen_ranges.insert((range.start_byte, range.end_byte)) {
                continue;
            }
        }
        for raw_sym in extract_backticked_symbols(doc) {
            let Some(sym) = filter_symbol(&raw_sym, is_md, ctx.whitelist) else {
                continue;
            };
            checked_symbols += 1;
            let check_ctx = SymbolCheckContext {
                sym: &sym,
                real_path: ctx.real_path,
                full_path: ctx.full_path,
                root: ctx.root,
                graph: ctx.graph,
                global_symbols: ctx.global_symbols,
                code_tokens: ctx.code_tokens,
            };
            if check_symbol_exists(&check_ctx) {
                continue;
            }
            record_suggestion_edit(
                ctx.real_path,
                &sym,
                ctx.graph,
                node.docstring_range.as_ref(),
                &mut file_edits,
            );
            missing_symbols.push(sym);
        }
    }
    FileAuditOutcome {
        checked_symbols,
        missing_symbols,
        file_edits,
    }
}

fn group_nodes_by_file(graph: &CodeSymbolGraph) -> HashMap<String, Vec<&SymbolNode>> {
    let mut file_to_nodes: HashMap<String, Vec<&SymbolNode>> = HashMap::new();
    for idx in graph.graph.node_indices() {
        let node = &graph.graph[idx];
        if node.docstring.is_some() {
            let real_path = graph
                .obfuscated_to_real_path
                .get(&node.path)
                .cloned()
                .unwrap_or_else(|| node.path.clone());
            file_to_nodes.entry(real_path).or_default().push(node);
        }
    }
    file_to_nodes
}

fn print_file_result(real_path: &str, missing: &[String], checked_count: usize, is_md: bool) {
    const RED: &str = "\x1b[91m";
    const GREEN: &str = "\x1b[92m";
    const RESET: &str = "\x1b[0m";

    if !missing.is_empty() {
        let prefix = if is_md {
            "Mismatched references in header not found in body or disk"
        } else {
            "Mismatched symbols in header not found in code"
        };
        let formatted = missing
            .iter()
            .map(|x| format!("`{x}`"))
            .collect::<Vec<_>>()
            .join(", ");
        println!("{RED}[FAIL]{RESET} [ADG-SG] {real_path}: {prefix}: {formatted}");
    } else if checked_count > 0 {
        let label = if is_md { "references" } else { "symbols" };
        println!("{GREEN}[OK]{RESET} [ADG-SG] {real_path}: Verified {checked_count} {label}");
    }
}

struct SingleFileContext<'a> {
    root: &'a Path,
    graph: &'a CodeSymbolGraph,
    global_symbols: &'a HashSet<&'a str>,
    whitelist: &'a HashSet<String>,
    fix_enabled: bool,
}

fn validate_single_file(
    real_path: &str,
    nodes: &[&SymbolNode],
    ctx: &SingleFileContext<'_>,
) -> (usize, Option<FileValidationFailure>) {
    let is_md = real_path.ends_with(".md");
    let full_path = ctx.root.join(real_path);
    let file_content = fs::read_to_string(&full_path).ok();
    let initial_hash = file_content.as_ref().map(|c| {
        let mut hasher = Sha256::new();
        hasher.update(c.as_bytes());
        hasher.finalize()
    });
    let code_tokens = file_content
        .as_deref()
        .map(tokenize_code_body)
        .unwrap_or_default();

    let node_ctx = NodeAuditContext {
        real_path,
        full_path: &full_path,
        root: ctx.root,
        graph: ctx.graph,
        global_symbols: ctx.global_symbols,
        code_tokens: &code_tokens,
        whitelist: ctx.whitelist,
    };
    let outcome = audit_node_docstrings(nodes, &node_ctx);

    let missing = if ctx.fix_enabled && !outcome.file_edits.is_empty() && file_content.is_some() {
        let autofix_ctx = AutoFixContext {
            full_path: &full_path,
            real_path,
            initial_hash: initial_hash.as_deref(),
            content: file_content.as_deref().unwrap_or(""),
        };
        apply_autofixes(autofix_ctx, outcome.file_edits, outcome.missing_symbols)
    } else {
        outcome.missing_symbols
    };

    let failure = if !missing.is_empty() {
        Some(FileValidationFailure {
            file_path: real_path.to_string(),
            missing_symbols: missing.clone(),
        })
    } else {
        None
    };

    print_file_result(real_path, &missing, outcome.checked_symbols, is_md);
    (outcome.checked_symbols, failure)
}

pub fn validate_graph_docstrings(
    graph: &CodeSymbolGraph,
    root: &Path,
    options: DocGuardOptions,
) -> Result<(), GraphQueryError> {
    println!("🔍 Starting Active Documentation Guard - Symbol Graph Validator...");
    let whitelist = build_whitelist(root);
    let diff_files = if options.diff_only {
        get_git_modified_files(root).ok()
    } else {
        None
    };

    let global_symbols: HashSet<&str> = graph
        .graph
        .node_indices()
        .map(|idx| graph.graph[idx].name.as_str())
        .collect();
    let file_to_nodes = group_nodes_by_file(graph);
    let mut sorted_paths: Vec<_> = file_to_nodes.keys().cloned().collect();
    sorted_paths.sort();

    let file_ctx = SingleFileContext {
        root,
        graph,
        global_symbols: &global_symbols,
        whitelist: &whitelist,
        fix_enabled: options.fix_enabled,
    };

    let mut stats = AuditStats {
        total_files: 0,
        total_symbols: 0,
        failed_files: 0,
    };
    let mut failures = Vec::new();

    for real_path in &sorted_paths {
        if let Some(ref filter) = diff_files {
            if !filter.contains(real_path) {
                continue;
            }
        }
        stats.total_files += 1;
        let (symbols_count, failure) =
            validate_single_file(real_path, &file_to_nodes[real_path], &file_ctx);
        stats.total_symbols += symbols_count;
        if let Some(fail) = failure {
            stats.failed_files += 1;
            failures.push(fail);
        }
    }

    let total_files = stats.total_files;
    if options.diff_only && total_files == 0 {
        return Err(GraphQueryError::Validation(
            "Diff mode matched 0 files in repository changeset — refusing to report false success"
                .into(),
        ));
    }

    let anomalies = graph.find_anomalies();
    let has_errors = print_audit_summary(&stats, &anomalies, options.strict);

    if let Some(out_path) = options.out {
        let payload = ReportPayload {
            strict: options.strict,
            has_errors,
            failures,
            anomalies,
        };
        save_report(out_path, &stats, payload)?;
    }

    if has_errors {
        Err(GraphQueryError::Validation(
            "Symbol Gate validation failed".into(),
        ))
    } else {
        println!("\n\x1b[92m[OK] Symbol Gate validation achieved!\x1b[0m\n");
        Ok(())
    }
}

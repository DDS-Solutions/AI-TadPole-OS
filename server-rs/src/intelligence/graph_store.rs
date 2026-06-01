//! @docs ARCHITECTURE:Core
//! 
//! ### AI Assist Note
//! **! Persistent code-review graph store.**
//! This module implements high-fidelity logic for the Sovereign Reality layer.
//! 
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: Runtime logic error, state desynchronization, or resource exhaustion.
//! - **Telemetry Link**: Search `[graph_store]` in tracing logs.
//!
//! Persistent code-review graph store.
//!
//! Rebuilds `.code-review-graph/graph.db` from the live workspace so external
//! audit tooling and startup telemetry can read a current symbol graph.

use crate::error::AppError;
use crate::utils::parser::{Reference, Symbol, SymbolExtractor, SymbolRange};
use once_cell::sync::Lazy;
use regex::Regex;
use serde::Serialize;
use std::collections::{HashMap, HashSet, VecDeque};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use walkdir::WalkDir;

mod db;

static PY_FUNC_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"^\s*def\s+([A-Za-z_][A-Za-z0-9_]*)").unwrap());
static PY_CLASS_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"^\s*class\s+([A-Za-z_][A-Za-z0-9_]*)").unwrap());
static SQL_CLASS_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?i)^\s*CREATE\s+TABLE\s+(?:IF\s+NOT\s+EXISTS\s+)?([A-Za-z_][A-Za-z0-9_]*)").unwrap());
static JS_FUNC_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"^\s*(?:export\s+)?(?:async\s+)?function\s+([A-Za-z_][A-Za-z0-9_]*)").unwrap());
static JS_CLASS_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"^\s*(?:export\s+)?class\s+([A-Za-z_][A-Za-z0-9_]*)").unwrap());
static JS_VAR_FUNC_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"^\s*(?:export\s+)?(?:const|let|var)\s+([A-Za-z_][A-Za-z0-9_]*)\s*=").unwrap());
static SH_FUNC_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"^\s*(?:function\s+)?([A-Za-z_][A-Za-z0-9_-]*)\s*(?:\(\))?\s*\{").unwrap());
static IMPORT_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r#"(?i)(?:import|from|require|use)\s+["']?([A-Za-z0-9_./:-]+)"#).unwrap());
static CALL_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"\b([A-Za-z_][A-Za-z0-9_]*)\s*\(").unwrap());

static RS_IMPORT_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"^\s*use\s+([A-Za-z0-9_:]+)").unwrap());
static OTHER_IMPORT_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r#"^\s*import\s+(?:.+?\s+from\s+)?["']?([A-Za-z0-9_@./-]+)"#).unwrap());

#[derive(Debug, Clone, Serialize)]
pub struct GraphDbRefreshSummary {
    pub db_path: PathBuf,
    pub node_count: usize,
    pub edge_count: usize,
    pub risk_count: usize,
    pub community_count: usize,
    pub flow_count: usize,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct CommunityRule {
    pub pattern: String,
    pub id: i64,
    pub name: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SymbolKind {
    File,
    Class,
    Function,
    Test,
}

impl SymbolKind {
    fn as_str(&self) -> &'static str {
        match self {
            SymbolKind::File => "File",
            SymbolKind::Class => "Class",
            SymbolKind::Function => "Function",
            SymbolKind::Test => "Test",
        }
    }
}

#[derive(Debug, Clone)]
struct FileRecord {
    absolute_path: String,
    relative_path: String,
    name: String,
    language: String,
    is_test: bool,
    symbols: Vec<SymbolRecord>,
    refs: Vec<Reference>,
    imports: Vec<String>,
    file_hash: String,
}

#[derive(Debug, Clone)]
struct SymbolRecord {
    name: String,
    kind: String,
    line_start: i64,
    line_end: i64,
    signature: String,
    parent_name: Option<String>,
    params: Option<String>,
    return_type: Option<String>,
    modifiers: Option<String>,
}

#[derive(Debug, Clone)]
struct NodeRow {
    id: i64,
    kind: String,
    name: String,
    qualified_name: String,
    file_path: String,
    line_start: Option<i64>,
    line_end: Option<i64>,
    language: String,
    parent_name: Option<String>,
    params: Option<String>,
    return_type: Option<String>,
    modifiers: Option<String>,
    is_test: bool,
    file_hash: String,
    extra: String,
    signature: String,
    community_id: Option<i64>,
}

#[derive(Debug, Clone, Eq, Hash, PartialEq)]
struct EdgeRow {
    kind: String,
    source_qualified: String,
    target_qualified: String,
    file_path: String,
    line: i64,
    extra: String,
}

#[derive(Debug, Clone)]
struct RiskRow {
    node_id: i64,
    qualified_name: String,
    risk_score: f64,
    caller_count: i64,
    test_coverage: String,
    security_relevant: bool,
}

#[derive(Debug, Clone)]
struct CommunityRow {
    id: i64,
    name: String,
    cohesion: f64,
    size: i64,
    dominant_language: String,
    description: String,
    risk: String,
}

#[derive(Debug, Clone)]
struct FlowRow {
    id: i64,
    name: String,
    entry_point_id: i64,
    entry_point: String,
    depth: i64,
    node_count: i64,
    node_ids: Vec<i64>,
    critical_path: Vec<String>,
    criticality: f64,
    file_count: i64,
}

#[derive(Debug, Clone)]
struct GraphSnapshot {
    root: PathBuf,
    nodes: Vec<NodeRow>,
    edges: Vec<EdgeRow>,
    risks: Vec<RiskRow>,
    communities: Vec<CommunityRow>,
    flows: Vec<FlowRow>,
}

pub async fn refresh_code_review_graph_db(
    root: PathBuf,
    db_path: PathBuf,
    salt: String,
) -> Result<GraphDbRefreshSummary, AppError> {
    let root = root
        .canonicalize()
        .map_err(|e| AppError::InternalServerError(format!("failed to resolve graph root: {e}")))?;
    let build_root = root.clone();
    let snapshot = tokio::task::spawn_blocking(move || build_snapshot(build_root, &salt))
        .await
        .map_err(|e| {
            AppError::InternalServerError(format!("graph DB refresh task panicked: {e}"))
        })??;

    if let Some(parent) = db_path.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }

    let pool = db::open_graph_pool(&db_path).await?;
    db::ensure_schema(&pool).await?;
    db::write_snapshot(&pool, &snapshot).await?;
    pool.close().await;

    Ok(GraphDbRefreshSummary {
        db_path,
        node_count: snapshot.nodes.len(),
        edge_count: snapshot.edges.len(),
        risk_count: snapshot.risks.len(),
        community_count: snapshot.communities.len(),
        flow_count: snapshot.flows.len(),
    })
}

fn build_snapshot(root: PathBuf, salt: &str) -> Result<GraphSnapshot, AppError> {
    let files = scan_files(&root, salt)?;

    let mut nodes = Vec::new();
    let mut next_id = 1i64;
    for file in &files {
        nodes.push(NodeRow {
            id: next_id,
            kind: SymbolKind::File.as_str().to_string(),
            name: file.name.clone(),
            qualified_name: file.absolute_path.clone(),
            file_path: file.absolute_path.clone(),
            line_start: Some(1),
            line_end: None,
            language: file.language.clone(),
            parent_name: None,
            params: None,
            return_type: None,
            modifiers: None,
            is_test: file.is_test,
            file_hash: file.file_hash.clone(),
            extra: serde_json::json!({ "relative_path": file.relative_path }).to_string(),
            signature: file.relative_path.clone(),
            community_id: None,
        });
        next_id = next_id.saturating_add(1);
        for sym in &file.symbols {
            let qualified_name = qualified_symbol(&file.absolute_path, sym);
            nodes.push(NodeRow {
                id: next_id,
                kind: normalize_kind(&sym.kind, file.is_test).as_str().to_string(),
                name: sym.name.clone(),
                qualified_name,
                file_path: file.absolute_path.clone(),
                line_start: Some(sym.line_start),
                line_end: Some(sym.line_end),
                language: file.language.clone(),
                parent_name: sym.parent_name.clone(),
                params: sym.params.clone(),
                return_type: sym.return_type.clone(),
                modifiers: sym.modifiers.clone(),
                is_test: file.is_test,
                file_hash: file.file_hash.clone(),
                extra: "{}".to_string(),
                signature: sym.signature.clone(),
                community_id: None,
            });
            next_id = next_id.saturating_add(1);
        }
    }

    let mut by_name: HashMap<Arc<str>, Vec<Arc<str>>> = HashMap::new();
    let mut file_nodes = HashMap::new();
    for node in &nodes {
        let qn_arc: Arc<str> = node.qualified_name.as_str().into();
        let name_arc: Arc<str> = node.name.as_str().into();
        let file_path_arc: Arc<str> = node.file_path.as_str().into();
        if node.kind == "File" {
            file_nodes.insert(file_path_arc, qn_arc);
        } else {
            by_name
                .entry(name_arc)
                .or_default()
                .push(qn_arc);
        }
    }

    let mut edges = HashSet::new();
    for file in &files {
        let file_path_arc: Arc<str> = file.absolute_path.as_str().into();
        let source_file_qn = file_nodes
            .get(&file_path_arc)
            .map(|s| s.to_string())
            .unwrap_or_else(|| file.absolute_path.clone());

        if let Some(file_qn) = file_nodes.get(&file_path_arc) {
            for sym in &file.symbols {
                edges.insert(EdgeRow {
                    kind: "CONTAINS".to_string(),
                    source_qualified: file_qn.to_string(),
                    target_qualified: qualified_symbol(&file.absolute_path, sym),
                    file_path: file.absolute_path.clone(),
                    line: sym.line_start,
                    extra: "{}".to_string(),
                });
            }
        }

        for import in &file.imports {
            for target in match_targets(import, &by_name) {
                edges.insert(EdgeRow {
                    kind: "IMPORTS_FROM".to_string(),
                    source_qualified: source_file_qn.clone(),
                    target_qualified: target.to_string(),
                    file_path: file.absolute_path.clone(),
                    line: 0,
                    extra: serde_json::json!({ "import": import }).to_string(),
                });
            }
        }

        for reference in &file.refs {
            let Some(source_sym) = tightest_symbol(file, reference) else {
                continue;
            };
            let source = qualified_symbol(&file.absolute_path, source_sym);
            for target in match_targets(&reference.name, &by_name) {
                if target.as_ref() == source.as_str() {
                    continue;
                }
                let kind = if file.is_test { "TESTED_BY" } else { "CALLS" };
                edges.insert(EdgeRow {
                    kind: kind.to_string(),
                    source_qualified: source.clone(),
                    target_qualified: target.to_string(),
                    file_path: file.absolute_path.clone(),
                    line: (reference.range.start_line as i64).saturating_add(1),
                    extra: serde_json::json!({ "reference": reference.name }).to_string(),
                });
            }
        }
    }

    let mut edges = edges.into_iter().collect::<Vec<_>>();
    edges.sort_by(|a, b| {
        (
            &a.kind,
            &a.source_qualified,
            &a.target_qualified,
            &a.file_path,
            a.line,
        )
            .cmp(&(
                &b.kind,
                &b.source_qualified,
                &b.target_qualified,
                &b.file_path,
                b.line,
            ))
    });

    let config_path = root.join(".code-review-graph").join("config.json");
    let rules = if config_path.exists() {
        std::fs::read_to_string(&config_path)
            .ok()
            .and_then(|content| serde_json::from_str::<Vec<CommunityRule>>(&content).ok())
            .unwrap_or_else(default_community_rules)
    } else {
        default_community_rules()
    };

    assign_communities(&mut nodes, &rules);
    let communities = build_communities(&nodes, &edges, &rules);
    let risks = build_risks(&nodes, &edges);
    let flows = build_flows(&nodes, &edges, &risks);

    Ok(GraphSnapshot {
        root,
        nodes,
        edges,
        risks,
        communities,
        flows,
    })
}

fn scan_files(root: &Path, salt: &str) -> Result<Vec<FileRecord>, AppError> {
    let mut extractor = SymbolExtractor::new();
    let mut records = Vec::new();
    for entry in WalkDir::new(root)
        .into_iter()
        .filter_entry(|e| {
            let name = e.file_name().to_string_lossy();
            !matches!(
                name.as_ref(),
                "target" | "node_modules" | ".git" | "dist" | "scratch"
            )
        })
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_file())
    {
        let path = entry.path();
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        if !matches!(
            ext,
            "rs" | "ts" | "tsx" | "py" | "sql" | "js" | "cjs" | "mjs" | "ps1" | "sh"
        ) {
            continue;
        }
        if std::fs::metadata(path).map(|m| m.len()).unwrap_or(0) > 2 * 1024 * 1024 {
            continue;
        }
        let content = match std::fs::read_to_string(path) {
            Ok(content) => content,
            Err(_) => continue,
        };
        let absolute_path = path
            .canonicalize()
            .unwrap_or_else(|_| path.to_path_buf())
            .to_string_lossy()
            .to_string();
        let relative_path = path
            .strip_prefix(root)
            .unwrap_or(path)
            .to_string_lossy()
            .to_string()
            .replace('\\', "/");
        let language = language_for_ext(ext).to_string();
        let is_test = is_test_path(&relative_path);
        let (symbols, refs, imports) = if matches!(ext, "rs" | "ts" | "tsx") {
            let symbols = extractor
                .extract_symbols(path, &content)
                .into_iter()
                .map(symbol_to_record)
                .collect();
            (
                symbols,
                extractor.extract_references(path, &content),
                extract_imports(ext, &content),
            )
        } else {
            lightweight_extract(ext, &content)
        };
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();

        let mut hash_input = content.as_bytes().to_vec();
        hash_input.extend_from_slice(salt.as_bytes());
        let hash = md5::compute(&hash_input);
        let file_hash = format!("{:x}", hash);

        records.push(FileRecord {
            absolute_path,
            relative_path,
            name,
            language,
            is_test,
            symbols,
            refs,
            imports,
            file_hash,
        });
    }
    records.sort_by(|a, b| a.absolute_path.cmp(&b.absolute_path));
    Ok(records)
}

fn symbol_to_record(sym: Symbol) -> SymbolRecord {
    SymbolRecord {
        name: sym.name,
        kind: sym.kind,
        line_start: (sym.range.start_line as i64).saturating_add(1),
        line_end: (sym.range.end_line as i64).saturating_add(1),
        signature: sym.signature,
        parent_name: None,
        params: None,
        return_type: None,
        modifiers: None,
    }
}

fn lightweight_extract(
    ext: &str,
    content: &str,
) -> (Vec<SymbolRecord>, Vec<Reference>, Vec<String>) {
    let mut symbols = Vec::new();
    let mut refs = Vec::new();
    let mut imports = Vec::new();
    let patterns = match ext {
        "py" => vec![
            (&*PY_FUNC_RE, "func"),
            (&*PY_CLASS_RE, "class"),
        ],
        "sql" => vec![
            (&*SQL_CLASS_RE, "class"),
        ],
        "js" | "cjs" | "mjs" => vec![
            (&*JS_FUNC_RE, "func"),
            (&*JS_CLASS_RE, "class"),
            (&*JS_VAR_FUNC_RE, "func"),
        ],
        "ps1" | "sh" => vec![
            (&*SH_FUNC_RE, "func"),
        ],
        _ => Vec::new(),
    };
    for (line_idx, line) in content.lines().enumerate() {
        for (re, kind) in &patterns {
            if let Some(cap) = re.captures(line) {
                let name = cap.get(1).map(|m| m.as_str()).unwrap_or("").to_string();
                if !name.is_empty() {
                    symbols.push(SymbolRecord {
                        name,
                        kind: kind.to_string(),
                        line_start: (line_idx as i64).saturating_add(1),
                        line_end: (line_idx as i64).saturating_add(1),
                        signature: line.trim().to_string(),
                        parent_name: None,
                        params: None,
                        return_type: None,
                        modifiers: None,
                    });
                }
            }
        }
        for cap in IMPORT_RE.captures_iter(line) {
            if let Some(name) = cap.get(1) {
                imports.push(name.as_str().to_string());
            }
        }
        for cap in CALL_RE.captures_iter(line) {
            if let Some(name) = cap.get(1) {
                refs.push(Reference {
                    name: name.as_str().to_string(),
                    range: SymbolRange {
                        start_byte: 0,
                        end_byte: 0,
                        start_line: line_idx,
                        end_line: line_idx,
                    },
                });
            }
        }
    }
    (symbols, refs, imports)
}

fn extract_imports(ext: &str, content: &str) -> Vec<String> {
    let re = if ext == "rs" {
        &*RS_IMPORT_RE
    } else {
        &*OTHER_IMPORT_RE
    };
    content
        .lines()
        .filter_map(|line| {
            re.captures(line)
                .and_then(|c| c.get(1))
                .map(|m| m.as_str().to_string())
        })
        .collect()
}

fn tightest_symbol<'a>(file: &'a FileRecord, reference: &Reference) -> Option<&'a SymbolRecord> {
    file.symbols
        .iter()
        .filter(|sym| {
            (reference.range.start_line as i64).saturating_add(1) >= sym.line_start
                && (reference.range.end_line as i64).saturating_add(1) <= sym.line_end
        })
        .min_by_key(|sym| sym.line_end - sym.line_start)
}

fn match_targets(name: &str, by_name: &HashMap<Arc<str>, Vec<Arc<str>>>) -> Vec<Arc<str>> {
    let direct = name.rsplit([':', '/', '.', '\\']).next().unwrap_or(name);
    by_name.get(direct).cloned().unwrap_or_default()
}

fn default_community_rules() -> Vec<CommunityRule> {
    vec![
        CommunityRule {
            pattern: "/server-rs/".to_string(),
            id: 1,
            name: "server-rs-core".to_string(),
        },
        CommunityRule {
            pattern: "/src-tauri/".to_string(),
            id: 2,
            name: "tauri-shell".to_string(),
        },
        CommunityRule {
            pattern: "/src/".to_string(),
            id: 3,
            name: "frontend-app".to_string(),
        },
        CommunityRule {
            pattern: "/execution/".to_string(),
            id: 4,
            name: "execution-tools".to_string(),
        },
        CommunityRule {
            pattern: "/.agent/".to_string(),
            id: 5,
            name: "agent-assets".to_string(),
        },
        CommunityRule {
            pattern: "/data/".to_string(),
            id: 6,
            name: "data-migrations".to_string(),
        },
        CommunityRule {
            pattern: "/migrations/".to_string(),
            id: 6,
            name: "data-migrations".to_string(),
        },
    ]
}

fn assign_communities(nodes: &mut [NodeRow], rules: &[CommunityRule]) {
    for node in nodes {
        let rel = node.file_path.replace('\\', "/");
        let mut assigned = false;
        for rule in rules {
            if rel.contains(&rule.pattern) {
                node.community_id = Some(rule.id);
                assigned = true;
                break;
            }
        }
        if !assigned {
            node.community_id = Some(7);
        }
    }
}

fn build_communities(nodes: &[NodeRow], edges: &[EdgeRow], rules: &[CommunityRule]) -> Vec<CommunityRow> {
    let mut by_id: HashMap<i64, Vec<&NodeRow>> = HashMap::new();
    for node in nodes {
        by_id
            .entry(node.community_id.unwrap_or(7))
            .or_default()
            .push(node);
    }
    let mut rows = Vec::new();
    for (id, group) in by_id {
        let mut langs = HashMap::<String, usize>::new();
        for node in &group {
            *langs.entry(node.language.clone()).or_default() += 1;
        }
        let dominant_language = langs
            .into_iter()
            .max_by_key(|(_, count)| *count)
            .map(|(lang, _)| lang)
            .unwrap_or_default();
        let names = group
            .iter()
            .map(|n| n.qualified_name.as_str())
            .collect::<HashSet<_>>();
        let internal = edges
            .iter()
            .filter(|e| {
                names.contains(e.source_qualified.as_str())
                    && names.contains(e.target_qualified.as_str())
            })
            .count();
        let cohesion = if group.len() <= 1 {
            0.0
        } else {
            (internal as f64 / group.len() as f64).min(1.0)
        };
        let name = rules
            .iter()
            .find(|r| r.id == id)
            .map(|r| r.name.as_str())
            .unwrap_or_else(|| {
                if id == 7 {
                    "workspace-other"
                } else {
                    "unknown-community"
                }
            });
        rows.push(CommunityRow {
            id,
            name: name.to_string(),
            cohesion,
            size: group.len() as i64,
            dominant_language,
            description: format!("{} symbols grouped by workspace area", group.len()),
            risk: "heuristic".to_string(),
        });
    }
    rows.sort_by_key(|row| row.id);
    rows
}

fn build_risks(nodes: &[NodeRow], edges: &[EdgeRow]) -> Vec<RiskRow> {
    let tested = edges
        .iter()
        .filter(|e| e.kind == "TESTED_BY")
        .map(|e| e.target_qualified.clone())
        .collect::<HashSet<_>>();
    let mut caller_counts = HashMap::<String, i64>::new();
    for edge in edges {
        if matches!(edge.kind.as_str(), "CALLS" | "REFERENCES" | "IMPORTS_FROM") {
            *caller_counts
                .entry(edge.target_qualified.clone())
                .or_default() += 1;
        }
    }
    nodes
        .iter()
        .filter(|node| node.kind != "File")
        .map(|node| {
            let caller_count = *caller_counts.get(&node.qualified_name).unwrap_or(&0);
            let is_tested = tested.contains(&node.qualified_name) || node.is_test;
            let security_relevant = is_security_relevant(node);
            let mut score = 0.15;
            if !is_tested {
                score += 0.2;
            }
            if security_relevant {
                score += 0.3;
            }
            if caller_count > 0 {
                score += ((caller_count as f64).log10() * 0.15).min(0.15);
            }
            if node.file_path.contains("\\routes\\") || node.file_path.contains("/routes/") {
                score += 0.05;
            }
            if node.kind == "Class" {
                score += 0.02;
            }
            RiskRow {
                node_id: node.id,
                qualified_name: node.qualified_name.clone(),
                risk_score: score.min(0.85),
                caller_count,
                test_coverage: if is_tested { "tested" } else { "untested" }.to_string(),
                security_relevant,
            }
        })
        .collect()
}

fn build_flows(nodes: &[NodeRow], edges: &[EdgeRow], risks: &[RiskRow]) -> Vec<FlowRow> {
    let risk_by_qn = risks
        .iter()
        .map(|r| (r.qualified_name.as_str(), r))
        .collect::<HashMap<_, _>>();
    let id_by_qn = nodes
        .iter()
        .map(|n| (n.qualified_name.as_str(), n.id))
        .collect::<HashMap<_, _>>();
    let mut adjacency = HashMap::<&str, Vec<&str>>::new();
    for edge in edges {
        if matches!(edge.kind.as_str(), "CALLS" | "REFERENCES" | "IMPORTS_FROM") {
            adjacency
                .entry(edge.source_qualified.as_str())
                .or_default()
                .push(edge.target_qualified.as_str());
        }
    }
    let mut entries = nodes
        .iter()
        .filter(|node| {
            node.kind != "File"
                && (node.name == "main"
                    || node.name.ends_with("_handler")
                    || node.file_path.contains("\\routes\\")
                    || node.file_path.contains("/routes/")
                    || node.file_path.contains("\\pages\\")
                    || node.file_path.contains("/pages/")
                    || node.file_path.contains("\\services\\")
                    || node.file_path.contains("/services/")
                    || node.file_path.contains("\\stores\\")
                    || node.file_path.contains("/stores/"))
        })
        .take(250)
        .collect::<Vec<_>>();
    entries.sort_by(|a, b| a.qualified_name.cmp(&b.qualified_name));

    let mut flows = Vec::new();
    for (idx, entry) in entries.into_iter().enumerate() {
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();
        let mut parents = HashMap::new();

        let entry_qn = entry.qualified_name.as_str();
        visited.insert(entry_qn);
        queue.push_back((entry_qn, 0i64));

        while let Some((current, depth)) = queue.pop_front() {
            if depth >= 6 {
                continue;
            }
            for next in adjacency.get(current).into_iter().flatten().take(20) {
                if visited.insert(next) {
                    parents.insert(*next, current);
                    queue.push_back((next, depth + 1));
                }
            }
        }
        let node_ids = visited
            .iter()
            .filter_map(|qn| id_by_qn.get(qn).copied())
            .collect::<Vec<_>>();
        let files = nodes
            .iter()
            .filter(|node| visited.contains(node.qualified_name.as_str()))
            .map(|node| node.file_path.as_str())
            .collect::<HashSet<_>>();
        let security_hits = visited
            .iter()
            .filter(|qn| {
                risk_by_qn
                    .get(**qn)
                    .map(|r| r.security_relevant)
                    .unwrap_or(false)
            })
            .count();
        let criticality = ((node_ids.len() as f64 * 0.015)
            + (files.len() as f64 * 0.02)
            + (security_hits as f64 * 0.1))
            .min(1.0);

        let target_node = visited
            .iter()
            .max_by(|a, b| {
                let r_a = risk_by_qn.get(*a).map(|r| r.risk_score).unwrap_or(0.0);
                let r_b = risk_by_qn.get(*b).map(|r| r.risk_score).unwrap_or(0.0);
                r_a.partial_cmp(&r_b).unwrap_or(std::cmp::Ordering::Equal)
            });

        let mut path = Vec::new();
        if let Some(mut curr) = target_node.copied() {
            path.push(curr.to_string());
            while let Some(parent) = parents.get(curr) {
                path.push(parent.to_string());
                curr = *parent;
            }
            path.reverse();
        }
        let critical_path = path.into_iter().take(25).collect::<Vec<_>>();

        flows.push(FlowRow {
            id: idx as i64 + 1,
            name: entry.name.clone(),
            entry_point_id: entry.id,
            entry_point: entry.qualified_name.clone(),
            depth: 6,
            node_count: node_ids.len() as i64,
            node_ids,
            critical_path,
            criticality,
            file_count: files.len() as i64,
        });
    }
    flows
}

fn normalize_kind(kind: &str, is_test: bool) -> SymbolKind {
    if is_test {
        return SymbolKind::Test;
    }
    match kind {
        "struct" | "enum" | "trait" | "class" | "interface" | "type" | "impl" => SymbolKind::Class,
        _ => SymbolKind::Function,
    }
}


fn language_for_ext(ext: &str) -> &'static str {
    match ext {
        "rs" => "rust",
        "ts" => "typescript",
        "tsx" => "tsx",
        "py" => "python",
        "sql" => "sql",
        "ps1" => "powershell",
        "sh" => "bash",
        _ => "javascript",
    }
}

fn is_test_path(path: &str) -> bool {
    let lower = path.to_lowercase();
    lower.contains(".test.")
        || lower.contains("_test.")
        || lower.contains("_tests.")
        || lower.contains("/tests/")
        || lower.ends_with("tests.rs")
}

fn is_security_relevant(node: &NodeRow) -> bool {
    const TERMS: &[&str] = &[
        "auth",
        "crypto",
        "token",
        "secret",
        "permission",
        "policy",
        "shell",
        "command",
        "process",
        "route",
        "persist",
        "db",
        "network",
        "provider",
        "execute",
        "tool",
        "quota",
        "acl",
        "key",
    ];
    let haystack = format!(
        "{} {} {}",
        node.name.to_lowercase(),
        node.file_path.to_lowercase(),
        node.signature.to_lowercase()
    );
    TERMS.iter().any(|term| haystack.contains(term))
}

fn qualified(file_path: &str, symbol: &str) -> String {
    format!("{file_path}::{symbol}")
}

fn qualified_symbol(file_path: &str, symbol: &SymbolRecord) -> String {
    format!(
        "{}@{}-{}",
        qualified(file_path, &symbol.name),
        symbol.line_start,
        symbol.line_end
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::Row;
    use tempfile::tempdir;

    #[tokio::test]
    async fn refresh_creates_idempotent_graph_db() {
        let dir = tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join("src")).unwrap();
        std::fs::write(
            dir.path().join("src/lib.rs"),
            "fn helper() {}\nfn main() { helper(); }\n",
        )
        .unwrap();
        let db = dir.path().join(".code-review-graph/graph.db");
        let first = refresh_code_review_graph_db(
            dir.path().to_path_buf(),
            db.clone(),
            "test-salt".to_string(),
        )
        .await
        .unwrap();
        let second = refresh_code_review_graph_db(
            dir.path().to_path_buf(),
            db.clone(),
            "test-salt".to_string(),
        )
        .await
        .unwrap();
        assert_eq!(first.node_count, second.node_count);
        assert_eq!(first.edge_count, second.edge_count);
    }

    #[tokio::test]
    async fn refresh_removes_deleted_symbols() {
        let dir = tempdir().unwrap();
        std::fs::write(dir.path().join("a.rs"), "fn alpha() {}\n").unwrap();
        std::fs::write(dir.path().join("b.rs"), "fn beta() { alpha(); }\n").unwrap();
        let db = dir.path().join(".code-review-graph/graph.db");
        refresh_code_review_graph_db(dir.path().to_path_buf(), db.clone(), "salt".to_string())
            .await
            .unwrap();
        std::fs::remove_file(dir.path().join("b.rs")).unwrap();
        refresh_code_review_graph_db(dir.path().to_path_buf(), db.clone(), "salt".to_string())
            .await
            .unwrap();
        let pool = db::open_graph_pool(&db).await.unwrap();
        let count: i64 = sqlx::query("SELECT COUNT(*) FROM nodes WHERE name = 'beta'")
            .fetch_one(&pool)
            .await
            .unwrap()
            .get(0);
        assert_eq!(count, 0);
    }

    #[tokio::test]
    async fn refresh_handles_duplicate_symbol_names_in_same_file() {
        let dir = tempdir().unwrap();
        std::fs::write(
            dir.path().join("lib.rs"),
            "struct AppError;\nimpl AppError { fn one() {} }\nimpl AppError { fn two() {} }\n",
        )
        .unwrap();
        let db = dir.path().join(".code-review-graph/graph.db");
        refresh_code_review_graph_db(dir.path().to_path_buf(), db.clone(), "salt".to_string())
            .await
            .unwrap();

        let pool = db::open_graph_pool(&db).await.unwrap();
        let count: i64 =
            sqlx::query("SELECT COUNT(DISTINCT qualified_name) FROM nodes WHERE name = 'AppError'")
                .fetch_one(&pool)
                .await
                .unwrap()
                .get(0);
        assert_eq!(count, 3);
    }

    #[tokio::test]
    async fn risk_index_marks_security_and_fts_rebuilds() {
        let dir = tempdir().unwrap();
        std::fs::write(
            dir.path().join("auth.rs"),
            "fn validate_token() {}\nfn caller() { validate_token(); }\n",
        )
        .unwrap();
        let db = dir.path().join(".code-review-graph/graph.db");
        refresh_code_review_graph_db(dir.path().to_path_buf(), db.clone(), "salt".to_string())
            .await
            .unwrap();
        let pool = db::open_graph_pool(&db).await.unwrap();
        let security: i64 = sqlx::query(
            "SELECT security_relevant FROM risk_index WHERE qualified_name LIKE '%validate_token%' LIMIT 1",
        )
        .fetch_one(&pool)
        .await
        .unwrap()
        .get(0);
        let fts_count: i64 =
            sqlx::query("SELECT COUNT(*) FROM nodes_fts WHERE nodes_fts MATCH 'validate_token'")
                .fetch_one(&pool)
                .await
                .unwrap()
                .get(0);
        assert_eq!(security, 1);
        assert!(fts_count > 0);
    }
}

// Metadata: [graph_store]

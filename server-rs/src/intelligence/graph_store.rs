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
use rayon::prelude::*;
use regex::Regex;
use serde::Serialize;
use std::collections::{HashMap, HashSet, VecDeque};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use walkdir::WalkDir;

mod db;

static PY_FUNC_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^\s*def\s+([A-Za-z_][A-Za-z0-9_]*)").unwrap());
static PY_CLASS_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^\s*class\s+([A-Za-z_][A-Za-z0-9_]*)").unwrap());
static SQL_CLASS_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)^\s*CREATE\s+TABLE\s+(?:IF\s+NOT\s+EXISTS\s+)?([A-Za-z_][A-Za-z0-9_]*)")
        .unwrap()
});
static JS_FUNC_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^\s*(?:export\s+)?(?:async\s+)?function\s+([A-Za-z_][A-Za-z0-9_]*)").unwrap()
});
static JS_CLASS_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^\s*(?:export\s+)?class\s+([A-Za-z_][A-Za-z0-9_]*)").unwrap());
static JS_VAR_FUNC_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^\s*(?:export\s+)?(?:const|let|var)\s+([A-Za-z_][A-Za-z0-9_]*)\s*=").unwrap()
});
static SH_FUNC_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^\s*(?:function\s+)?([A-Za-z_][A-Za-z0-9_-]*)\s*(?:\(\))?\s*\{").unwrap()
});
static IMPORT_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"(?i)(?:import|from|require|use)\s+["']?([A-Za-z0-9_./:-]+)"#).unwrap()
});
static CALL_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"\b([A-Za-z_][A-Za-z0-9_]*)\s*\(").unwrap());

static RS_IMPORT_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"^\s*use\s+([A-Za-z0-9_:]+)").unwrap());
static OTHER_IMPORT_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r#"^\s*import\s+(?:.+?\s+from\s+)?["']?([A-Za-z0-9_@./-]+)"#).unwrap());

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
struct RawFileRecord {
    absolute_path: String,
    relative_path: String,
    name: String,
    language: String,
    is_test: bool,
    content: String,
    file_hash: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
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

#[derive(Debug, Clone, Eq, Hash, PartialEq, Ord, PartialOrd)]
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
    cache_updates: Vec<(String, String, String)>, // (file_path, file_hash, cache_json)
    files_present: Vec<String>, // list of all files found in current scan
}

pub async fn refresh_code_review_graph_db(
    root: PathBuf,
    db_path: PathBuf,
    salt: String,
) -> Result<GraphDbRefreshSummary, AppError> {
    let root = root
        .canonicalize()
        .map_err(|e| AppError::InternalServerError(format!("failed to resolve graph root: {e}")))?;

    if let Some(parent) = db_path.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }

    let pool = db::open_graph_pool(&db_path).await?;
    db::ensure_schema(&pool).await?;

    let cache = db::read_file_cache(&pool).await.unwrap_or_default();

    let build_root = root.clone();
    let snapshot = tokio::task::spawn_blocking(move || build_snapshot(build_root, &salt, cache))
        .await
        .map_err(|e| {
            AppError::InternalServerError(format!("graph DB refresh task panicked: {e}"))
        })??;

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

trait LanguageProcessor: Send + Sync {
    fn extract(
        &self,
        extractor: &mut SymbolExtractor,
        path: &Path,
        content: &str,
    ) -> Result<(Vec<SymbolRecord>, Vec<Reference>, Vec<String>), AppError>;
}

struct RustProcessor;
impl LanguageProcessor for RustProcessor {
    fn extract(
        &self,
        extractor: &mut SymbolExtractor,
        path: &Path,
        content: &str,
    ) -> Result<(Vec<SymbolRecord>, Vec<Reference>, Vec<String>), AppError> {
        let symbols = extractor
            .extract_symbols(path, content)
            .into_iter()
            .map(symbol_to_record)
            .collect();
        let refs = extractor.extract_references(path, content);
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        let imports = extract_imports(ext, content);
        Ok((symbols, refs, imports))
    }
}

struct TypeScriptProcessor;
impl LanguageProcessor for TypeScriptProcessor {
    fn extract(
        &self,
        extractor: &mut SymbolExtractor,
        path: &Path,
        content: &str,
    ) -> Result<(Vec<SymbolRecord>, Vec<Reference>, Vec<String>), AppError> {
        let symbols = extractor
            .extract_symbols(path, content)
            .into_iter()
            .map(symbol_to_record)
            .collect();
        let refs = extractor.extract_references(path, content);
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        let imports = extract_imports(ext, content);
        Ok((symbols, refs, imports))
    }
}

struct RegexProcessor;
impl LanguageProcessor for RegexProcessor {
    fn extract(
        &self,
        _extractor: &mut SymbolExtractor,
        path: &Path,
        content: &str,
    ) -> Result<(Vec<SymbolRecord>, Vec<Reference>, Vec<String>), AppError> {
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        Ok(lightweight_extract(ext, content))
    }
}

fn lookup_processor(ext: &str) -> Option<&'static dyn LanguageProcessor> {
    static RUST_PROC: RustProcessor = RustProcessor;
    static TS_PROC: TypeScriptProcessor = TypeScriptProcessor;
    static REGEX_PROC: RegexProcessor = RegexProcessor;

    match ext {
        "rs" => Some(&RUST_PROC),
        "ts" | "tsx" => Some(&TS_PROC),
        "py" | "sql" | "js" | "cjs" | "mjs" | "ps1" | "sh" => Some(&REGEX_PROC),
        _ => None,
    }
}

struct IdGenerator {
    counter: std::sync::atomic::AtomicI64,
}

impl IdGenerator {
    pub fn new(start: i64) -> Self {
        Self {
            counter: std::sync::atomic::AtomicI64::new(start),
        }
    }

    pub fn next_id(&self) -> i64 {
        self.counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst)
    }
}

struct FileScanner {
    root: PathBuf,
}

impl FileScanner {
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }

    pub fn scan(&self) -> Vec<PathBuf> {
        WalkDir::new(&self.root)
            .into_iter()
            .filter_entry(|e| {
                let name = e.file_name().to_string_lossy();
                !matches!(
                    name.as_ref(),
                    "target" | "node_modules" | ".git" | "dist" | "scratch" | ".tmp" | "tmp" | "coverage" | ".fallow" | ".gemini" | "logs" | "workspaces" | ".vscode" | "reports" | ".code-review-graph"
                )
            })
            .filter_map(|e| e.ok())
            .filter(|e| {
                if !e.path().is_file() {
                    return false;
                }
                let path = e.path();
                let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
                if !matches!(
                    ext,
                    "rs" | "ts" | "tsx" | "py" | "sql" | "js" | "cjs" | "mjs" | "ps1" | "sh"
                ) {
                    return false;
                }
                match std::fs::metadata(path) {
                    Ok(m) => m.len() <= 2 * 1024 * 1024,
                    Err(e) => {
                        tracing::warn!("Failed to read metadata for {:?}: {}", path, e);
                        false
                    }
                }
            })
            .map(|e| e.path().to_path_buf())
            .collect()
    }
}

struct SymbolProcessor {
    extractor: SymbolExtractor,
}

impl SymbolProcessor {
    pub fn new() -> Self {
        Self {
            extractor: SymbolExtractor::new(),
        }
    }

    pub fn process(&mut self, raw: RawFileRecord) -> Result<FileRecord, AppError> {
        let path = Path::new(&raw.absolute_path);
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");

        let (symbols, refs, imports) = if let Some(proc) = lookup_processor(ext) {
            proc.extract(&mut self.extractor, path, &raw.content)?
        } else {
            (Vec::new(), Vec::new(), Vec::new())
        };

        let validated_symbols = symbols
            .into_iter()
            .filter(|sym| {
                if sym.line_start <= 0 || sym.line_end < sym.line_start {
                    tracing::warn!(
                        "Skipping invalid symbol '{}' range in {:?}: start={}, end={}",
                        sym.name,
                        raw.absolute_path,
                        sym.line_start,
                        sym.line_end
                    );
                    false
                } else {
                    true
                }
            })
            .collect();

        Ok(FileRecord {
            absolute_path: raw.absolute_path,
            relative_path: raw.relative_path,
            name: raw.name,
            language: raw.language,
            is_test: raw.is_test,
            symbols: validated_symbols,
            refs,
            imports,
            file_hash: raw.file_hash,
        })
    }
}

struct GraphBuilder {
    id_gen: IdGenerator,
}

impl GraphBuilder {
    pub fn new() -> Self {
        Self {
            id_gen: IdGenerator::new(1),
        }
    }

    pub fn build(&self, files: &[FileRecord]) -> (Vec<NodeRow>, Vec<EdgeRow>) {
        let mut nodes = Vec::new();

        for file in files {
            nodes.push(NodeRow {
                id: self.id_gen.next_id(),
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

            for sym in &file.symbols {
                let qualified_name = qualified_symbol(&file.absolute_path, sym);
                nodes.push(NodeRow {
                    id: self.id_gen.next_id(),
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
                by_name.entry(name_arc).or_default().push(qn_arc);
            }
        }

        let mut edges = Vec::new();
        for file in files {
            let file_path_arc: Arc<str> = file.absolute_path.as_str().into();
            let source_file_qn = match file_nodes.get(&file_path_arc) {
                Some(qn) => qn.to_string(),
                None => {
                    tracing::warn!(
                        "File path {:?} not found in file_nodes map; using absolute path directly.",
                        file_path_arc
                    );
                    file.absolute_path.clone()
                }
            };

            if let Some(file_qn) = file_nodes.get(&file_path_arc) {
                for sym in &file.symbols {
                    edges.push(EdgeRow {
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
                    edges.push(EdgeRow {
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
                    edges.push(EdgeRow {
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

        edges.sort();
        edges.dedup();

        (nodes, edges)
    }
}

struct MetricEngine;

impl MetricEngine {
    pub fn new() -> Self {
        Self
    }

    pub fn process(
        &self,
        nodes: &mut [NodeRow],
        edges: &[EdgeRow],
        rules: &[CommunityRule],
    ) -> (Vec<CommunityRow>, Vec<RiskRow>, Vec<FlowRow>) {
        assign_communities(nodes, rules);
        let communities = build_communities(nodes, edges, rules);
        let risks = build_risks(nodes, edges);
        let flows = build_flows(nodes, edges, &risks);
        (communities, risks, flows)
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct CachedFileData {
    symbols: Vec<SymbolRecord>,
    refs: Vec<Reference>,
    imports: Vec<String>,
}

fn build_snapshot(
    root: PathBuf,
    salt: &str,
    cache: std::collections::HashMap<String, (String, String)>,
) -> Result<GraphSnapshot, AppError> {
    let scanner = FileScanner::new(root.clone());
    let paths = scanner.scan();

    let processed_results: Vec<Result<(FileRecord, Option<(String, String, String)>), AppError>> = paths
        .into_par_iter()
        .map_init(
            || SymbolProcessor::new(),
            |processor, path| {
                let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");

                let absolute_path = match path.canonicalize() {
                    Ok(p) => {
                        if !p.starts_with(&root) {
                            return Err(AppError::InternalServerError(format!(
                                "Security Violation: Path {:?} points outside workspace root {:?}",
                                p, root
                            )));
                        }
                        p.to_string_lossy().into_owned()
                    }
                    Err(e) => {
                        tracing::warn!("Failed to canonicalize path {:?}: {}", path, e);
                        path.to_string_lossy().into_owned()
                    }
                };

                let relative_path = match path.strip_prefix(&root) {
                    Ok(p) => p.to_string_lossy().into_owned().replace('\\', "/"),
                    Err(_) => path.to_string_lossy().into_owned().replace('\\', "/"),
                };

                let language = language_for_ext(ext).to_string();
                let is_test = is_test_path(&relative_path);

                let name = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown")
                    .to_string();

                let content = match std::fs::read_to_string(&path) {
                    Ok(content) => content,
                    Err(e) => {
                        return Err(AppError::InternalServerError(format!(
                            "Failed to read file {:?}: {}", path, e
                        )));
                    }
                };

                let mut hash_input = content.as_bytes().to_vec();
                hash_input.extend_from_slice(salt.as_bytes());
                let hash = md5::compute(&hash_input);
                let file_hash = format!("{:x}", hash);

                if let Some((cached_hash, cache_json)) = cache.get(&absolute_path) {
                    if cached_hash == &file_hash {
                        if let Ok(cached_data) = serde_json::from_str::<CachedFileData>(cache_json) {
                            let record = FileRecord {
                                absolute_path,
                                relative_path,
                                name,
                                language,
                                is_test,
                                symbols: cached_data.symbols,
                                refs: cached_data.refs,
                                imports: cached_data.imports,
                                file_hash,
                            };
                            return Ok((record, None));
                        }
                    }
                }

                let raw_record = RawFileRecord {
                    absolute_path: absolute_path.clone(),
                    relative_path: relative_path.clone(),
                    name: name.clone(),
                    language: language.clone(),
                    is_test,
                    content,
                    file_hash: file_hash.clone(),
                };

                let processed = processor.process(raw_record)?;

                let cached_data = CachedFileData {
                    symbols: processed.symbols.clone(),
                    refs: processed.refs.clone(),
                    imports: processed.imports.clone(),
                };

                let cache_json = serde_json::to_string(&cached_data).map_err(|e| {
                    AppError::InternalServerError(format!("Failed to serialize cache: {e}"))
                })?;

                Ok((processed, Some((absolute_path, file_hash, cache_json))))
            }
        )
        .collect();

    let mut files = Vec::new();
    let mut cache_updates = Vec::new();
    let mut files_present = Vec::new();

    for res in processed_results {
        match res {
            Ok((record, maybe_update)) => {
                files_present.push(record.absolute_path.clone());
                files.push(record);
                if let Some(update) = maybe_update {
                    cache_updates.push(update);
                }
            }
            Err(e) => {
                let err_str = e.to_string();
                if err_str.contains("Security Violation") {
                    tracing::warn!("{}", err_str);
                    continue;
                } else {
                    return Err(e);
                }
            }
        }
    }

    files.sort_by(|a, b| a.absolute_path.cmp(&b.absolute_path));

    let builder = GraphBuilder::new();
    let (mut nodes, edges) = builder.build(&files);

    let config_path = root.join(".code-review-graph").join("config.json");
    let rules = if config_path.exists() {
        std::fs::read_to_string(&config_path)
            .ok()
            .and_then(|content| serde_json::from_str::<Vec<CommunityRule>>(&content).ok())
            .unwrap_or_else(default_community_rules)
    } else {
        default_community_rules()
    };

    let metric_engine = MetricEngine::new();
    let (communities, risks, flows) = metric_engine.process(&mut nodes, &edges, &rules);

    Ok(GraphSnapshot {
        root,
        nodes,
        edges,
        risks,
        communities,
        flows,
        cache_updates,
        files_present,
    })
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

/// Extracts symbols, references, and imports using simple regex patterns.
///
/// ### Design Constraint & Limitation
/// This function processes code line-by-line using regular expressions. As a result,
/// it fundamentally cannot capture multi-line constructs (e.g., signatures, struct/class definitions
/// that span multiple lines, or macro calls spanning multiple lines).
/// For high-fidelity extraction, this should be upgraded to use tree-sitter or an AST parser.
fn lightweight_extract(
    ext: &str,
    content: &str,
) -> (Vec<SymbolRecord>, Vec<Reference>, Vec<String>) {
    let mut symbols = Vec::new();
    let mut refs = Vec::new();
    let mut imports = Vec::new();
    let patterns = match ext {
        "py" => vec![(&*PY_FUNC_RE, "func"), (&*PY_CLASS_RE, "class")],
        "sql" => vec![(&*SQL_CLASS_RE, "class")],
        "js" | "cjs" | "mjs" => vec![
            (&*JS_FUNC_RE, "func"),
            (&*JS_CLASS_RE, "class"),
            (&*JS_VAR_FUNC_RE, "func"),
        ],
        "ps1" | "sh" => vec![(&*SH_FUNC_RE, "func")],
        _ => Vec::new(),
    };
    for (line_idx, line) in content.lines().enumerate() {
        if line.len() > 1000 {
            continue;
        }
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
        .filter(|line| line.len() <= 1000)
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

fn build_communities(
    nodes: &[NodeRow],
    edges: &[EdgeRow],
    rules: &[CommunityRule],
) -> Vec<CommunityRow> {
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

        let target_node = visited.iter().max_by(|a, b| {
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
        let test_salt = uuid::Uuid::new_v4().to_string();
        let first = refresh_code_review_graph_db(
            dir.path().to_path_buf(),
            db.clone(),
            test_salt.clone(),
        )
        .await
        .unwrap();
        let second = refresh_code_review_graph_db(
            dir.path().to_path_buf(),
            db.clone(),
            test_salt,
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
        let test_salt = uuid::Uuid::new_v4().to_string();
        refresh_code_review_graph_db(dir.path().to_path_buf(), db.clone(), test_salt.clone())
            .await
            .unwrap();
        std::fs::remove_file(dir.path().join("b.rs")).unwrap();
        refresh_code_review_graph_db(dir.path().to_path_buf(), db.clone(), test_salt)
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
        let test_salt = uuid::Uuid::new_v4().to_string();
        refresh_code_review_graph_db(dir.path().to_path_buf(), db.clone(), test_salt)
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
        let test_salt = uuid::Uuid::new_v4().to_string();
        refresh_code_review_graph_db(dir.path().to_path_buf(), db.clone(), test_salt)
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

    #[test]
    fn test_build_flows_happy_path() {
        let nodes = vec![
            NodeRow {
                id: 1,
                kind: "Function".to_string(),
                name: "main".to_string(),
                qualified_name: "main".to_string(),
                file_path: "main.rs".to_string(),
                line_start: Some(1),
                line_end: Some(10),
                language: "rust".to_string(),
                parent_name: None,
                params: None,
                return_type: None,
                modifiers: None,
                is_test: false,
                file_hash: "hash".to_string(),
                extra: "{}".to_string(),
                signature: "fn main()".to_string(),
                community_id: None,
            },
            NodeRow {
                id: 2,
                kind: "Function".to_string(),
                name: "service_func".to_string(),
                qualified_name: "service_func".to_string(),
                file_path: "service.rs".to_string(),
                line_start: Some(1),
                line_end: Some(10),
                language: "rust".to_string(),
                parent_name: None,
                params: None,
                return_type: None,
                modifiers: None,
                is_test: false,
                file_hash: "hash".to_string(),
                extra: "{}".to_string(),
                signature: "fn service_func()".to_string(),
                community_id: None,
            },
            NodeRow {
                id: 3,
                kind: "Function".to_string(),
                name: "repo_func".to_string(),
                qualified_name: "repo_func".to_string(),
                file_path: "repo.rs".to_string(),
                line_start: Some(1),
                line_end: Some(10),
                language: "rust".to_string(),
                parent_name: None,
                params: None,
                return_type: None,
                modifiers: None,
                is_test: false,
                file_hash: "hash".to_string(),
                extra: "{}".to_string(),
                signature: "fn repo_func()".to_string(),
                community_id: None,
            },
        ];

        let edges = vec![
            EdgeRow {
                kind: "CALLS".to_string(),
                source_qualified: "main".to_string(),
                target_qualified: "service_func".to_string(),
                file_path: "main.rs".to_string(),
                line: 2,
                extra: "{}".to_string(),
            },
            EdgeRow {
                kind: "CALLS".to_string(),
                source_qualified: "service_func".to_string(),
                target_qualified: "repo_func".to_string(),
                file_path: "service.rs".to_string(),
                line: 5,
                extra: "{}".to_string(),
            },
        ];

        let risks = vec![
            RiskRow {
                node_id: 1,
                qualified_name: "main".to_string(),
                risk_score: 0.10,
                caller_count: 0,
                test_coverage: "untested".to_string(),
                security_relevant: false,
            },
            RiskRow {
                node_id: 2,
                qualified_name: "service_func".to_string(),
                risk_score: 0.20,
                caller_count: 1,
                test_coverage: "untested".to_string(),
                security_relevant: false,
            },
            RiskRow {
                node_id: 3,
                qualified_name: "repo_func".to_string(),
                risk_score: 0.30,
                caller_count: 1,
                test_coverage: "untested".to_string(),
                security_relevant: false,
            },
        ];

        let flows = build_flows(&nodes, &edges, &risks);
        assert_eq!(flows.len(), 1);
        let flow = &flows[0];
        assert_eq!(flow.name, "main");
        assert_eq!(flow.entry_point, "main");
        assert_eq!(flow.node_count, 3);
        assert_eq!(flow.file_count, 3);
        assert_eq!(flow.critical_path, vec!["main", "service_func", "repo_func"]);
    }

    #[test]
    fn test_build_flows_disconnected_and_circular() {
        let nodes = vec![
            NodeRow {
                id: 1,
                kind: "Function".to_string(),
                name: "main".to_string(),
                qualified_name: "main".to_string(),
                file_path: "/routes/main.rs".to_string(),
                line_start: Some(1),
                line_end: Some(10),
                language: "rust".to_string(),
                parent_name: None,
                params: None,
                return_type: None,
                modifiers: None,
                is_test: false,
                file_hash: "hash".to_string(),
                extra: "{}".to_string(),
                signature: "fn main()".to_string(),
                community_id: None,
            },
            NodeRow {
                id: 2,
                kind: "Function".to_string(),
                name: "service_func".to_string(),
                qualified_name: "service_func".to_string(),
                file_path: "/services/service.rs".to_string(),
                line_start: Some(1),
                line_end: Some(10),
                language: "rust".to_string(),
                parent_name: None,
                params: None,
                return_type: None,
                modifiers: None,
                is_test: false,
                file_hash: "hash".to_string(),
                extra: "{}".to_string(),
                signature: "fn service_func()".to_string(),
                community_id: None,
            },
        ];

        let edges = vec![
            EdgeRow {
                kind: "CALLS".to_string(),
                source_qualified: "main".to_string(),
                target_qualified: "service_func".to_string(),
                file_path: "/routes/main.rs".to_string(),
                line: 2,
                extra: "{}".to_string(),
            },
            EdgeRow {
                kind: "CALLS".to_string(),
                source_qualified: "service_func".to_string(),
                target_qualified: "main".to_string(),
                file_path: "/services/service.rs".to_string(),
                line: 5,
                extra: "{}".to_string(),
            },
        ];

        let risks = vec![
            RiskRow {
                node_id: 1,
                qualified_name: "main".to_string(),
                risk_score: 0.15,
                caller_count: 1,
                test_coverage: "untested".to_string(),
                security_relevant: false,
            },
            RiskRow {
                node_id: 2,
                qualified_name: "service_func".to_string(),
                risk_score: 0.15,
                caller_count: 1,
                test_coverage: "untested".to_string(),
                security_relevant: false,
            },
        ];

        let flows = build_flows(&nodes, &edges, &risks);
        assert_eq!(flows.len(), 2);
    }

    #[test]
    fn test_build_flows_empty_and_boundaries() {
        let flows = build_flows(&[], &[], &[]);
        assert!(flows.is_empty());

        let nodes = vec![NodeRow {
            id: 1,
            kind: "Function".to_string(),
            name: "main".to_string(),
            qualified_name: "main".to_string(),
            file_path: "main.rs".to_string(),
            line_start: Some(1),
            line_end: Some(10),
            language: "rust".to_string(),
            parent_name: None,
            params: None,
            return_type: None,
            modifiers: None,
            is_test: false,
            file_hash: "hash".to_string(),
            extra: "{}".to_string(),
            signature: "fn main()".to_string(),
            community_id: None,
        }];
        let risks = vec![RiskRow {
            node_id: 1,
            qualified_name: "main".to_string(),
            risk_score: 0.15,
            caller_count: 0,
            test_coverage: "untested".to_string(),
            security_relevant: false,
        }];
        let flows = build_flows(&nodes, &[], &risks);
        assert_eq!(flows.len(), 1);
        assert_eq!(flows[0].node_count, 1);
        assert_eq!(flows[0].file_count, 1);
        assert_eq!(flows[0].criticality, 0.035);

        let mut nodes = Vec::new();
        let mut edges = Vec::new();
        let mut risks = Vec::new();
        for i in 1..=10 {
            nodes.push(NodeRow {
                id: i,
                kind: "Function".to_string(),
                name: if i == 1 { "main".to_string() } else { format!("func_{i}") },
                qualified_name: format!("func_{i}"),
                file_path: format!("file_{i}.rs"),
                line_start: Some(1),
                line_end: Some(10),
                language: "rust".to_string(),
                parent_name: None,
                params: None,
                return_type: None,
                modifiers: None,
                is_test: false,
                file_hash: "hash".to_string(),
                extra: "{}".to_string(),
                signature: format!("fn func_{i}()"),
                community_id: None,
            });
            risks.push(RiskRow {
                node_id: i,
                qualified_name: format!("func_{i}"),
                risk_score: 0.15,
                caller_count: if i == 1 { 0 } else { 1 },
                test_coverage: "untested".to_string(),
                security_relevant: false,
            });
            if i < 10 {
                edges.push(EdgeRow {
                    kind: "CALLS".to_string(),
                    source_qualified: format!("func_{i}"),
                    target_qualified: format!("func_{}", i + 1),
                    file_path: format!("file_{i}.rs"),
                    line: 2,
                    extra: "{}".to_string(),
                });
            }
        }

        let flows = build_flows(&nodes, &edges, &risks);
        assert_eq!(flows.len(), 1);
        assert_eq!(flows[0].node_count, 7);
    }

    #[tokio::test]
    async fn test_file_scanner_symlinks_and_traversal() {
        let dir = tempdir().unwrap();
        let outside_dir = tempdir().unwrap();
        
        let root = dir.path().to_path_buf();
        let outside_file = outside_dir.path().join("outside.rs");
        std::fs::write(&outside_file, "fn dangerous() {}").unwrap();
        
        let inside_file = root.join("inside.rs");
        std::fs::write(&inside_file, "fn safe() {}").unwrap();
        
        let symlink_path = root.join("link_outside.rs");
        #[cfg(unix)]
        let _ = std::os::unix::fs::symlink(&outside_file, &symlink_path);
        #[cfg(windows)]
        let _ = std::os::windows::fs::symlink_file(&outside_file, &symlink_path);
        
        let test_salt = uuid::Uuid::new_v4().to_string();
        let snapshot_res = build_snapshot(root.clone(), &test_salt, std::collections::HashMap::new());
        if let Ok(snapshot) = snapshot_res {
            let files: Vec<_> = snapshot.nodes.iter().filter(|n| n.kind == "File").collect();
            for file_node in files {
                assert!(!file_node.file_path.contains("outside.rs"));
                assert!(!file_node.file_path.contains("link_outside.rs"));
            }
        }
    }
}

// Metadata: [graph_store]

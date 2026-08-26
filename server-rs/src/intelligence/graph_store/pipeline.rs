//! @docs ARCHITECTURE:Core:Intelligence
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / graph_store / pipeline
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Fault-tolerant parallel file ingestion, fail-closed security boundary verification, and atomic snapshot synthesis.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: none declared
//! - **Witness Tests**: `intelligence::graph_store::pipeline::tests`

use super::db;
use super::extract::lookup_processor;
use super::heuristics::{
    is_test_path, language_for_ext, match_targets, normalize_kind, qualified_symbol,
    tightest_symbol,
};
use super::metrics::MetricEngine;
use super::model::{
    default_community_rules, CachedFileData, CommunityRule, ConfigPayload, EdgeKind, EdgeRow,
    FileRecord, GraphDbRefreshSummary, GraphSnapshot, IdGenerator, NodeRow, RawFileRecord,
    SymbolKind,
};
use crate::error::AppError;
use crate::utils::parser::SymbolExtractor;
use rayon::prelude::*;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Arc;
use walkdir::WalkDir;

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

pub struct FileScanner {
    root: PathBuf,
}

impl FileScanner {
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }

    pub fn scan(&self) -> Vec<PathBuf> {
        let mut paths = Vec::new();
        let walker = WalkDir::new(&self.root).into_iter().filter_entry(|e| {
            let name = e.file_name().to_string_lossy();
            !crate::intelligence::EXCLUDED_DIRS.contains(&name.as_ref())
        });

        for entry in walker.filter_map(|e| e.ok()) {
            if paths.len() >= 10_000 {
                tracing::warn!(
                    "⚠️ [GraphStore] Hard limit of 10,000 discovered files reached. Skipping subsequent files."
                );
                break;
            }
            if !entry.path().is_file() {
                continue;
            }
            let path = entry.path();
            let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
            if !matches!(
                ext,
                "rs" | "ts" | "tsx" | "py" | "sql" | "js" | "cjs" | "mjs" | "ps1" | "sh"
            ) {
                continue;
            }
            match std::fs::metadata(path) {
                Ok(m) => {
                    if m.len() <= 2 * 1024 * 1024 {
                        paths.push(path.to_path_buf());
                    } else {
                        tracing::warn!(
                            "⚠️ [GraphStore] Skipping oversized file {:?} ({} bytes > 2MB)",
                            path,
                            m.len()
                        );
                    }
                }
                Err(e) => {
                    tracing::warn!("Failed to read metadata for {:?}: {}", path, e);
                }
            }
        }
        paths
    }
}

pub struct SymbolProcessor {
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

pub struct GraphBuilder {
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
                        kind: EdgeKind::Contains,
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
                        kind: EdgeKind::ImportsFrom,
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
                    let kind = if file.is_test {
                        EdgeKind::Tests
                    } else {
                        EdgeKind::Calls
                    };
                    edges.push(EdgeRow {
                        kind,
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

pub struct FileIngestOutcome {
    pub record: Option<FileRecord>,
    pub cache_update: Option<(String, String, String)>,
}

pub fn ingest_file(
    path: &Path,
    root: &Path,
    salt: &str,
    cache: &HashMap<String, (String, String)>,
    processor: &mut SymbolProcessor,
) -> Result<FileIngestOutcome, AppError> {
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");

    let absolute_path = match path.canonicalize() {
        Ok(p) => {
            if !p.starts_with(root) {
                tracing::warn!(
                    "🛡️ [GraphStore] Security Violation: Path {:?} points outside workspace root {:?}",
                    p, root
                );
                return Ok(FileIngestOutcome {
                    record: None,
                    cache_update: None,
                });
            }
            p.to_string_lossy().into_owned()
        }
        Err(e) => {
            tracing::warn!(
                "⚠️ [GraphStore] Failed to canonicalize path {:?}, skipping for boundary safety: {}",
                path, e
            );
            return Ok(FileIngestOutcome {
                record: None,
                cache_update: None,
            });
        }
    };

    let relative_path = match path.strip_prefix(root) {
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

    let content = match std::fs::read_to_string(path) {
        Ok(content) => content,
        Err(e) => {
            tracing::warn!(
                "⚠️ [GraphStore] Failed to read file {:?}, skipping: {}",
                path,
                e
            );
            return Ok(FileIngestOutcome {
                record: None,
                cache_update: None,
            });
        }
    };

    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    hasher.update(salt.as_bytes());
    let file_hash = hex::encode(hasher.finalize());

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
                return Ok(FileIngestOutcome {
                    record: Some(record),
                    cache_update: None,
                });
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

    let cache_json = serde_json::to_string(&cached_data)
        .map_err(|e| AppError::InternalServerError(format!("Failed to serialize cache: {e}")))?;

    Ok(FileIngestOutcome {
        record: Some(processed),
        cache_update: Some((absolute_path, file_hash, cache_json)),
    })
}

pub fn load_graph_config(root: &Path) -> (Vec<CommunityRule>, usize) {
    let config_path = root.join(".code-review-graph").join("config.json");
    if config_path.exists() {
        match std::fs::read_to_string(&config_path) {
            Ok(content) => match serde_json::from_str::<ConfigPayload>(&content) {
                Ok(ConfigPayload::LegacyRules(r)) => (r, 250),
                Ok(ConfigPayload::FullConfig(c)) => (c.community_rules, c.max_flows),
                Err(e) => {
                    tracing::warn!(
                        "⚠️ [GraphStore] Malformed .code-review-graph/config.json: {}. Using defaults.",
                        e
                    );
                    (default_community_rules(), 250)
                }
            },
            Err(e) => {
                tracing::warn!(
                    "⚠️ [GraphStore] Failed to read .code-review-graph/config.json: {}. Using defaults.",
                    e
                );
                (default_community_rules(), 250)
            }
        }
    } else {
        (default_community_rules(), 250)
    }
}

fn probe_git_output(root: &Path, args: &[&str]) -> Option<String> {
    let output = Command::new("git")
        .args(args)
        .current_dir(root)
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

pub fn build_snapshot(
    root: PathBuf,
    salt: &str,
    cache: HashMap<String, (String, String)>,
) -> Result<GraphSnapshot, AppError> {
    let git_branch = probe_git_output(&root, &["branch", "--show-current"]);
    let git_head_sha = probe_git_output(&root, &["rev-parse", "HEAD"]);

    let scanner = FileScanner::new(root.clone());
    let paths = scanner.scan();

    let processed_results: Vec<Result<FileIngestOutcome, AppError>> = paths
        .into_par_iter()
        .map_init(
            || SymbolProcessor::new(),
            |processor, path| ingest_file(&path, &root, salt, &cache, processor),
        )
        .collect();

    let mut files = Vec::new();
    let mut cache_updates = Vec::new();
    let mut files_present = Vec::new();

    for res in processed_results {
        match res {
            Ok(outcome) => {
                if let Some(record) = outcome.record {
                    files_present.push(record.absolute_path.clone());
                    files.push(record);
                }
                if let Some(update) = outcome.cache_update {
                    cache_updates.push(update);
                }
            }
            Err(e) => {
                tracing::warn!("⚠️ [GraphStore] Error processing file record: {}", e);
            }
        }
    }

    files.sort_by(|a, b| a.absolute_path.cmp(&b.absolute_path));

    let builder = GraphBuilder::new();
    let (mut nodes, edges) = builder.build(&files);

    let (rules, max_flows) = load_graph_config(&root);

    let metric_engine = MetricEngine::new();
    let (communities, risks, flows) = metric_engine.process(&mut nodes, &edges, &rules, max_flows);

    Ok(GraphSnapshot {
        root,
        git_branch,
        git_head_sha,
        nodes,
        edges,
        risks,
        communities,
        flows,
        cache_updates,
        files_present,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_ingest_file_security_boundary() {
        let root_dir = tempdir().unwrap();
        let outside_dir = tempdir().unwrap();

        let outside_file = outside_dir.path().join("outside.rs");
        std::fs::write(&outside_file, "fn dangerous() {}").unwrap();

        let mut processor = SymbolProcessor::new();
        let outcome = ingest_file(
            &outside_file,
            root_dir.path(),
            "salt",
            &HashMap::new(),
            &mut processor,
        )
        .unwrap();

        assert!(
            outcome.record.is_none(),
            "Outside file must be rejected by boundary guard"
        );
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
        let snapshot_res = build_snapshot(root.clone(), &test_salt, HashMap::new());
        assert!(
            snapshot_res.is_ok(),
            "build_snapshot must succeed gracefully skipping symlink escapes"
        );
        let snapshot = snapshot_res.unwrap();
        let files: Vec<_> = snapshot.nodes.iter().filter(|n| n.kind == "File").collect();
        for file_node in files {
            assert!(
                !file_node.file_path.contains("outside.rs"),
                "Outside file must not be indexed"
            );
            assert!(
                !file_node.file_path.contains("link_outside.rs"),
                "Symlink escaping root must not be indexed"
            );
        }
    }
}

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
//! - **Telemetry Targets**: `[FAIL]`, `[ADG-SG]`, `[OK]`, `[WARN]`
//! - **Witness Tests**: none declared

use crate::graph::{CodeSymbolGraph, SymbolNode};
use crate::path_utils::get_git_modified_files;
use crate::GraphQueryError;
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;

pub const BUILTIN_KEYWORDS: &[&str] = &[
    "true",
    "false",
    "any",
    "unwrap",
    "string",
    "number",
    "boolean",
    "void",
    "null",
    "undefined",
    "str",
    "u8",
    "u16",
    "u32",
    "u64",
    "usize",
    "i32",
    "i64",
    "f32",
    "f64",
    "Self",
    "self",
    "Ok",
    "Err",
    "Option",
    "Result",
    "Some",
    "None",
    "Arc",
    "Mutex",
    "State",
    "Body",
    "Request",
    "Response",
    "StatusCode",
    "Next",
    "axum",
    "tokio",
    "std",
    "env",
    "var",
    "cfg",
    "test",
    "tests",
    "Error",
    "props",
    "Props",
    "interface",
    "type",
    "const",
    "let",
    "function",
    "class",
    "import",
];

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct FileValidationFailure {
    pub file_path: String,
    pub missing_symbols: Vec<String>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct ValidationReport {
    pub schema_version: u32,
    pub strict: bool,
    pub has_errors: bool,
    pub total_files_audited: usize,
    pub total_symbols_checked: usize,
    pub failed_files_count: usize,
    pub failures: Vec<FileValidationFailure>,
    pub anomalies: Vec<String>,
    pub anomaly_count: usize,
}

pub fn extract_backticked_symbols(doc: &str) -> Vec<String> {
    let mut symbols = Vec::new();
    let mut in_code_fence = false;

    for line in doc.lines() {
        let trimmed = line.trim();
        let content = if let Some(stripped) = trimmed.strip_prefix("///") {
            stripped.trim()
        } else if let Some(stripped) = trimmed.strip_prefix("//!") {
            stripped.trim()
        } else if let Some(stripped) = trimmed.strip_prefix('*') {
            stripped.trim()
        } else {
            trimmed
        };

        if content.starts_with("```") || content.starts_with("~~~") {
            in_code_fence = !in_code_fence;
            continue;
        }

        if in_code_fence {
            continue;
        }

        let mut in_tick = false;
        let mut start_idx = 0;
        for (i, c) in content.char_indices() {
            if c == '`' {
                if in_tick {
                    let term = &content[start_idx..i];
                    let cleaned = term.trim();
                    let cleaned = cleaned.strip_suffix("()").unwrap_or(cleaned);
                    if !cleaned.is_empty() && !cleaned.contains('`') {
                        symbols.push(cleaned.to_string());
                    }
                    in_tick = false;
                } else {
                    in_tick = true;
                    start_idx = i + 1;
                }
            }
        }
    }

    let mut seen = HashSet::new();
    symbols.retain(|s| seen.insert(s.clone()));
    symbols
}

pub fn filter_symbol(s: &str, is_md: bool, whitelist: &HashSet<String>) -> Option<String> {
    let s = s.trim();
    let s_clean = s.strip_suffix("()").unwrap_or(s);
    if whitelist.contains(s_clean) {
        return None;
    }

    // Security defense: Reject null bytes, drive colons, root prefixes, absolute paths, and traversal
    if s_clean.contains('\0') || s_clean.contains(':') {
        return None;
    }
    if s_clean.starts_with('/') || s_clean.starts_with('\\') || Path::new(s_clean).is_absolute() {
        return None;
    }
    if s_clean.split(['/', '\\']).any(|seg| seg == "..") {
        return None;
    }

    if is_md {
        if s_clean.chars().any(|c| " ${}[]<>=+*\"'".contains(c)) {
            return None;
        }
        if s_clean.starts_with("http://") || s_clean.starts_with("https://") {
            return None;
        }
        if !s_clean.chars().any(|c| c.is_alphanumeric()) {
            return None;
        }
        Some(s_clean.to_string())
    } else {
        if s_clean.chars().any(|c| " /\\-:${}[]<>&|".contains(c)) {
            return None;
        }
        if !s_clean.is_empty()
            && (s_clean.chars().next().unwrap().is_alphabetic() || s_clean.starts_with('_'))
            && s_clean.chars().all(|c| c.is_alphanumeric() || c == '_')
        {
            return Some(s_clean.to_string());
        }
        None
    }
}

pub fn disk_candidate_exists(root: &Path, doc_dir: &Path, sym: &str) -> bool {
    let canonical_root = match root.canonicalize() {
        Ok(r) => r,
        Err(_) => root.to_path_buf(),
    };
    for base in [root, doc_dir] {
        let raw_target = base.join(sym);
        let normalized = crate::path_utils::lexical_normalize(&raw_target);
        let canonical_target = match normalized.canonicalize() {
            Ok(p) => p,
            Err(_) => normalized,
        };
        // Strict containment check and file check (prevents directory probing oracle)
        if canonical_target.starts_with(&canonical_root) && canonical_target.is_file() {
            return true;
        }
    }
    false
}

#[derive(Debug, Clone)]
pub struct DocGuardOptions<'a> {
    pub strict: bool,
    pub out: Option<&'a Path>,
    pub diff_only: bool,
    pub fix_enabled: bool,
}

pub fn validate_graph_docstrings(
    graph: &CodeSymbolGraph,
    root: &Path,
    options: DocGuardOptions,
) -> Result<(), GraphQueryError> {
    const GREEN: &str = "\x1b[92m";
    const RED: &str = "\x1b[91m";
    const YELLOW: &str = "\x1b[93m";
    const RESET: &str = "\x1b[0m";

    println!("🔍 Starting Active Documentation Guard - Symbol Graph Validator...");

    let mut whitelist = HashSet::new();
    for kw in BUILTIN_KEYWORDS {
        whitelist.insert(kw.to_string());
    }

    let globals_path = root.join(".agent/globals.json");
    if globals_path.exists() {
        match fs::read_to_string(&globals_path) {
            Ok(content) => match serde_json::from_str::<Vec<String>>(&content) {
                Ok(globals) => {
                    whitelist.extend(globals);
                }
                Err(e) => {
                    println!(
                        "{YELLOW}[WARN] Could not parse .agent/globals.json ({e}). Retaining builtin whitelist.{RESET}"
                    );
                }
            },
            Err(e) => {
                println!(
                    "{YELLOW}[WARN] Could not read .agent/globals.json ({e}). Retaining builtin whitelist.{RESET}"
                );
            }
        }
    }

    if let Ok(content) = fs::read_to_string(root.join("server-rs/.env.example")) {
        for line in content.lines() {
            let trimmed = line.trim();
            if !trimmed.is_empty() && !trimmed.starts_with('#') {
                if let Some(key) = trimmed.split('=').next() {
                    whitelist.insert(key.trim().to_string());
                }
            }
        }
    }

    let diff_files = if options.diff_only {
        match get_git_modified_files(root) {
            Ok(files) => {
                println!(
                    "ℹ️ [Diff Mode] Restricting validation to {} modified files from git",
                    files.len()
                );
                Some(files)
            }
            Err(e) => {
                println!(
                    "⚠️ [WARN] Failed to retrieve modified files from git ({}). Scanning all files.",
                    e
                );
                None
            }
        }
    } else {
        None
    };

    let mut total_files = 0;
    let mut total_symbols_checked = 0;
    let mut failed_files = 0;
    let mut failures = Vec::new();

    // Precompute global symbol names for O(1) membership checking
    let global_symbols: HashSet<&str> = graph
        .graph
        .node_indices()
        .map(|idx| graph.graph[idx].name.as_str())
        .collect();

    let mut file_to_nodes: std::collections::HashMap<String, Vec<&SymbolNode>> =
        std::collections::HashMap::new();
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

    let mut sorted_file_paths: Vec<_> = file_to_nodes.keys().cloned().collect();
    sorted_file_paths.sort();

    for real_path in &sorted_file_paths {
        let nodes = &file_to_nodes[real_path];
        if let Some(ref filter) = diff_files {
            if !filter.contains(real_path) {
                continue; // Skip unmodified file
            }
        }
        total_files += 1;
        let is_md = real_path.ends_with(".md");
        let full_path = root.join(real_path);
        let file_content = fs::read_to_string(&full_path).ok();
        let initial_hash = file_content.as_ref().map(|c| {
            let mut hasher = Sha256::new();
            hasher.update(c.as_bytes());
            hasher.finalize()
        });

        // Pre-tokenize file code body once per file for O(1) membership check (eliminating regex compile in inner loop)
        let file_code_tokens: HashSet<String> = if let Some(ref content) = file_content {
            let mut body = String::new();
            for line in content.lines() {
                let trimmed = line.trim();
                if !trimmed.starts_with("//")
                    && !trimmed.starts_with("/*")
                    && !trimmed.starts_with('*')
                {
                    body.push_str(line);
                    body.push('\n');
                }
            }
            body.split(|c: char| !c.is_alphanumeric() && c != '_')
                .filter(|s| !s.is_empty())
                .map(|s| s.to_string())
                .collect()
        } else {
            HashSet::new()
        };

        let mut file_dirty = false;
        let mut file_edits = Vec::new();

        let mut missing_symbols = Vec::new();
        let mut checked_symbols_in_file = 0;
        let mut seen_ranges = HashSet::new();

        for node in nodes {
            if let Some(ref doc) = node.docstring {
                if let Some(ref range) = node.docstring_range {
                    if !seen_ranges.insert((range.start_byte, range.end_byte)) {
                        continue;
                    }
                }
                let extracted = extract_backticked_symbols(doc);
                for raw_sym in extracted {
                    if let Some(sym) = filter_symbol(&raw_sym, is_md, &whitelist) {
                        checked_symbols_in_file += 1;
                        total_symbols_checked += 1;

                        let mut found = false;

                        if let Some((symbols, _)) = graph.repository.parse_cache.get(real_path) {
                            if symbols.iter().any(|s| s.name == sym) {
                                found = true;
                            }
                        }

                        if !found {
                            if let Some((_, refs)) = graph.repository.parse_cache.get(real_path) {
                                if refs.iter().any(|r| r.name == sym) {
                                    found = true;
                                }
                            }
                        }

                        if !found && global_symbols.contains(sym.as_str()) {
                            found = true;
                        }

                        if !found && (sym.contains('/') || sym.contains('\\') || sym.contains('.'))
                        {
                            let parent = full_path.parent().unwrap_or(root);
                            if disk_candidate_exists(root, parent, &sym) {
                                found = true;
                            }
                        }

                        if !found && file_code_tokens.contains(&sym) {
                            found = true;
                        }

                        if !found {
                            let mut best_suggestion = None;
                            if let Some((symbols, _)) = graph.repository.parse_cache.get(real_path)
                            {
                                for candidate in symbols {
                                    let sim = strsim::jaro_winkler(&sym, &candidate.name);
                                    if sim > 0.8 {
                                        match best_suggestion {
                                            Some((_, best_sim)) if sim > best_sim => {
                                                best_suggestion =
                                                    Some((candidate.name.as_str(), sim));
                                            }
                                            None => {
                                                best_suggestion =
                                                    Some((candidate.name.as_str(), sim));
                                            }
                                            _ => {}
                                        }
                                    }
                                }
                            }

                            if let Some((suggested, _)) = best_suggestion {
                                println!("   💡 Suggestion: Did you mean `{}`?", suggested);
                                if let Some(ref range) = node.docstring_range {
                                    file_edits.push((
                                        range.clone(),
                                        sym.clone(),
                                        suggested.to_string(),
                                    ));
                                }
                            }

                            missing_symbols.push(sym);
                        }
                    }
                }
            }
        }

        if options.fix_enabled && !file_edits.is_empty() {
            if let Some(mut content) = file_content.clone() {
                let current_bytes = fs::read(&full_path).unwrap_or_default();
                let mut hasher = Sha256::new();
                hasher.update(&current_bytes);
                let current_hash = hasher.finalize();

                if let Some(expected_hash) = initial_hash {
                    if current_hash != expected_hash {
                        println!(
                            "{YELLOW}[WARN] [ADG-SG] {real_path}: content changed on disk after scan; refusing to auto-fix.{RESET}"
                        );
                    } else {
                        file_edits.sort_by_key(|e| std::cmp::Reverse(e.0.start_byte));
                        let mut applied_occurrences: HashMap<String, usize> = HashMap::new();

                        for (range, sym, suggested) in &file_edits {
                            if range.start_byte < content.len()
                                && range.end_byte <= content.len()
                                && content.is_char_boundary(range.start_byte)
                                && content.is_char_boundary(range.end_byte)
                            {
                                let orig_doc = &content[range.start_byte..range.end_byte];
                                let target_backtick = format!("`{}`", sym);
                                let replacement_backtick = format!("`{}`", suggested);

                                let mut new_doc =
                                    orig_doc.replace(&target_backtick, &replacement_backtick);
                                new_doc = new_doc
                                    .replace(&format!("[[{}", sym), &format!("[[{}", suggested));

                                if new_doc != orig_doc {
                                    content
                                        .replace_range(range.start_byte..range.end_byte, &new_doc);
                                    file_dirty = true;
                                    *applied_occurrences.entry(sym.clone()).or_insert(0) += 1;
                                    println!(
                                        "   🔧 [FIXED] Replaced `{}` with `{}` in {}",
                                        sym, suggested, real_path
                                    );
                                }
                            }
                        }

                        if file_dirty {
                            let tmp_path = full_path.with_extension("docguard.tmp");
                            if let Err(e) = fs::write(&tmp_path, &content) {
                                println!(
                                    "⚠️ [ERROR] Failed to write auto-fixes to temporary file {}: {}",
                                    tmp_path.display(),
                                    e
                                );
                            } else if let Err(e) = fs::rename(&tmp_path, &full_path) {
                                println!(
                                    "⚠️ [ERROR] Failed to atomically rename auto-fixes to {}: {}",
                                    real_path, e
                                );
                                let _ = fs::remove_file(&tmp_path);
                            } else {
                                // Occurrence-aware removal: only drop the number of occurrences actually fixed
                                let mut remaining_missing = Vec::new();
                                for s in missing_symbols {
                                    if let Some(count) = applied_occurrences.get_mut(&s) {
                                        if *count > 0 {
                                            *count -= 1;
                                            continue;
                                        }
                                    }
                                    remaining_missing.push(s);
                                }
                                missing_symbols = remaining_missing;
                            }
                        }
                    }
                }
            }
        }

        if !missing_symbols.is_empty() {
            failed_files += 1;
            failures.push(FileValidationFailure {
                file_path: real_path.clone(),
                missing_symbols: missing_symbols.clone(),
            });
            let msg = if is_md {
                format!(
                    "Mismatched references in header not found in body or disk: {}",
                    missing_symbols
                        .iter()
                        .map(|x| format!("`{x}`"))
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            } else {
                format!(
                    "Mismatched symbols in header not found in code: {}",
                    missing_symbols
                        .iter()
                        .map(|x| format!("`{x}`"))
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            };
            println!("{RED}[FAIL]{RESET} [ADG-SG] {real_path}: {msg}");
        } else if checked_symbols_in_file > 0 {
            let msg = if is_md {
                format!("Verified {checked_symbols_in_file} references")
            } else {
                format!("Verified {checked_symbols_in_file} symbols")
            };
            println!("{GREEN}[OK]{RESET} [ADG-SG] {real_path}: {msg}");
        }
    }

    if options.diff_only && total_files == 0 {
        println!("{RED}⚠️ [FAIL] Diff mode active but 0 matching files in git changeset.{RESET}");
        return Err(GraphQueryError::Validation(
            "Diff mode matched 0 files in repository changeset — refusing to report false success"
                .to_string(),
        ));
    }

    println!("\n============================================================");
    println!("  ADG-SG AUDIT SUMMARY");
    println!("============================================================");
    println!("Total Files Audited: {total_files}");
    println!("Total Symbols Checked: {total_symbols_checked}");
    println!("Failed Files: {failed_files}");

    let anomalies = graph.find_anomalies();
    if !anomalies.is_empty() {
        println!("\n{RED}[FAIL] Found {} structural codebase anomalies (unused symbols/dead code):{RESET}", anomalies.len());
        for anomaly in &anomalies {
            println!("  - {anomaly}");
        }
    } else {
        println!("\n{GREEN}[OK] No structural codebase anomalies (dead code) detected.{RESET}");
    }

    let mut has_errors = false;
    if failed_files > 0 {
        println!("\n{RED}[FAIL] Symbol-to-Header Parity check failed. Please align header docstrings with code.{RESET}");
        has_errors = true;
    }
    if !anomalies.is_empty() {
        if options.strict {
            println!("\n{RED}[FAIL] Unused/dead code check failed.{RESET}");
            has_errors = true;
        } else {
            println!("\n{YELLOW}[WARN] Unused/dead code check failed (Warning Only). Run with --strict to enforce.{RESET}");
        }
    }

    if let Some(out_path) = options.out {
        let report = ValidationReport {
            schema_version: 1,
            strict: options.strict,
            has_errors,
            total_files_audited: total_files,
            total_symbols_checked,
            failed_files_count: failed_files,
            failures,
            anomaly_count: anomalies.len(),
            anomalies,
        };
        let content =
            serde_json::to_string_pretty(&report).map_err(GraphQueryError::Serialization)?;
        if let Some(parent) = out_path.parent() {
            fs::create_dir_all(parent).map_err(GraphQueryError::Io)?;
        }
        fs::write(out_path, content).map_err(GraphQueryError::Io)?;
        println!(
            "Saved structured validation report to: {}",
            out_path.display()
        );
    }

    if has_errors {
        return Err(GraphQueryError::Validation(
            "Symbol Gate validation failed".to_string(),
        ));
    } else {
        println!("\n{GREEN}[OK] Symbol Gate validation achieved!{RESET}\n");
    }

    Ok(())
}

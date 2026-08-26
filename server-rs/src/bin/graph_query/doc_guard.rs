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
use std::collections::HashSet;
use std::fs;
use std::path::Path;

#[derive(Debug, serde::Serialize)]
pub struct FileValidationFailure {
    pub file_path: String,
    pub missing_symbols: Vec<String>,
}

#[derive(Debug, serde::Serialize)]
pub struct ValidationReport {
    pub total_files_audited: usize,
    pub total_symbols_checked: usize,
    pub failed_files_count: usize,
    pub failures: Vec<FileValidationFailure>,
}

pub fn extract_backticked_symbols(doc: &str) -> Vec<String> {
    let mut symbols = Vec::new();
    let mut in_backticks = false;
    let mut start_idx = 0;
    for (i, c) in doc.char_indices() {
        if c == '`' {
            if in_backticks {
                let term = &doc[start_idx..i];
                let cleaned = term.trim();
                let cleaned = cleaned.strip_suffix("()").unwrap_or(cleaned);
                if !cleaned.is_empty() {
                    symbols.push(cleaned.to_string());
                }
                in_backticks = false;
            } else {
                in_backticks = true;
                start_idx = i + 1;
            }
        } else if c == '\n' {
            in_backticks = false;
        }
    }
    symbols
}

pub fn filter_symbol(s: &str, is_md: bool, whitelist: &HashSet<String>) -> Option<String> {
    let s = s.trim();
    let s_clean = s.strip_suffix("()").unwrap_or(s);
    if whitelist.contains(s_clean) {
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
    let globals_path = root.join(".agent/globals.json");
    match fs::read_to_string(&globals_path) {
        Ok(content) => match serde_json::from_str::<Vec<String>>(&content) {
            Ok(globals) => {
                whitelist.extend(globals);
            }
            Err(e) => {
                println!(
                    "{YELLOW}[WARN] Could not parse .agent/globals.json ({e}). Using fallback whitelist.{RESET}"
                );
            }
        },
        Err(e) => {
            if globals_path.exists() {
                println!(
                    "{YELLOW}[WARN] Could not read .agent/globals.json ({e}). Using fallback whitelist.{RESET}"
                );
            }
        }
    }

    if whitelist.is_empty() {
        for fallback in &[
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
        ] {
            whitelist.insert(fallback.to_string());
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
        let mut file_dirty = false;
        let mut file_edits = Vec::new();

        let mut missing_symbols = Vec::new();
        let mut checked_symbols_in_file = 0;

        for node in nodes {
            if let Some(ref doc) = node.docstring {
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
                            let rel_to_root = root.join(&sym);
                            let parent = full_path.parent().unwrap_or_else(|| Path::new(""));
                            let rel_to_file = parent.join(&sym);
                            if rel_to_root.exists() || rel_to_file.exists() {
                                found = true;
                            }
                        }

                        if !found {
                            if let Some(ref content) = file_content {
                                let mut code_body = String::new();
                                for line in content.lines() {
                                    let trimmed = line.trim();
                                    if !trimmed.starts_with("//")
                                        && !trimmed.starts_with("/*")
                                        && !trimmed.starts_with("*")
                                    {
                                        code_body.push_str(line);
                                        code_body.push('\n');
                                    }
                                }
                                let pattern = format!(r"\b{}\b", regex::escape(&sym));
                                if let Ok(re) = regex::Regex::new(&pattern) {
                                    if re.is_match(&code_body) {
                                        found = true;
                                    }
                                }
                            }
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

                            // Keep sym in missing_symbols until auto-fix writes successfully
                            missing_symbols.push(sym);
                        }
                    }
                }
            }
        }

        if options.fix_enabled && !file_edits.is_empty() {
            if let Some(mut content) = file_content.clone() {
                file_edits.sort_by_key(|e| std::cmp::Reverse(e.0.start_byte));
                let mut applied_fixed_symbols = HashSet::new();

                for (range, sym, suggested) in &file_edits {
                    if range.start_byte < content.len()
                        && range.end_byte <= content.len()
                        && content.is_char_boundary(range.start_byte)
                        && content.is_char_boundary(range.end_byte)
                    {
                        let orig_doc = &content[range.start_byte..range.end_byte];
                        let target_backtick = format!("`{}`", sym);
                        let replacement_backtick = format!("`{}`", suggested);

                        let mut new_doc = orig_doc.replace(&target_backtick, &replacement_backtick);
                        new_doc =
                            new_doc.replace(&format!("[[{}", sym), &format!("[[{}", suggested));

                        if new_doc != orig_doc {
                            content.replace_range(range.start_byte..range.end_byte, &new_doc);
                            file_dirty = true;
                            applied_fixed_symbols.insert(sym.clone());
                            println!(
                                "   🔧 [FIXED] Replaced `{}` with `{}` in {}",
                                sym, suggested, real_path
                            );
                        }
                    }
                }

                if file_dirty {
                    if let Err(e) = fs::write(&full_path, &content) {
                        println!(
                            "⚠️ [ERROR] Failed to write auto-fixes to {}: {}",
                            real_path, e
                        );
                    } else {
                        // Only remove symbols that were successfully fixed in the file
                        missing_symbols.retain(|s| !applied_fixed_symbols.contains(s));
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
        println!("⚠️ [WARN] Diff mode active but 0 matching files in git changeset.");
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

    if let Some(out_path) = options.out {
        let report = ValidationReport {
            total_files_audited: total_files,
            total_symbols_checked,
            failed_files_count: failed_files,
            failures,
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

    if has_errors {
        return Err(GraphQueryError::Validation(
            "Symbol Gate validation failed".to_string(),
        ));
    } else {
        println!("\n{GREEN}[OK] Symbol Gate validation achieved!{RESET}\n");
    }

    Ok(())
}

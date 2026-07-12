//! @docs ARCHITECTURE:CodeBaseIntelligence
//!
//! ### AI Assist Note
//! **Documentation Guard**: Validates backticked symbols inside docstrings against
//! the graph connections to enforce sync.
//!
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: Failed docstring validation, files missing from the git log.
//! - **Trace Scope**: `server-rs::bin::graph_query::doc_guard`

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

pub fn validate_graph_docstrings(
    graph: &CodeSymbolGraph,
    root: &Path,
    strict: bool,
    out: Option<&Path>,
    diff_only: bool,
    fix_enabled: bool,
) -> Result<(), GraphQueryError> {
    const GREEN: &str = "\x1b[92m";
    const RED: &str = "\x1b[91m";
    const YELLOW: &str = "\x1b[93m";
    const RESET: &str = "\x1b[0m";

    println!("🔍 Starting Active Documentation Guard - Symbol Graph Validator...");

    let mut whitelist = HashSet::new();
    if let Ok(content) = fs::read_to_string(root.join(".agent/globals.json")) {
        if let Ok(globals) = serde_json::from_str::<Vec<String>>(&content) {
            whitelist.extend(globals);
        }
    } else {
        println!(
            "{YELLOW}[WARN] Could not parse .agent/globals.json. Using fallback whitelist.{RESET}"
        );
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

    let diff_files = if diff_only {
        match get_git_modified_files(root) {
            Ok(files) => {
                println!(
                    "ℹ️ [Diff Mode] Restricting validation to {} modified files from git",
                    files.len()
                );
                Some(files)
            }
            Err(_) => {
                println!(
                    "⚠️ [WARN] Failed to retrieve modified files from git. Scanning all files."
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

    for (real_path, nodes) in &file_to_nodes {
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

                        if !found
                            && graph
                                .graph
                                .node_indices()
                                .any(|idx| graph.graph[idx].name == sym)
                        {
                            found = true;
                        }

                        if !found && (sym.contains('/') || sym.contains('\\') || sym.contains('.'))
                        {
                            let rel_to_root = root.join(&sym);
                            let rel_to_file = full_path.parent().unwrap().join(&sym);
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

                            if fix_enabled && best_suggestion.is_some() {
                                found = true;
                            }
                        }

                        if !found {
                            missing_symbols.push(sym);
                        }
                    }
                }
            }
        }

        if fix_enabled && !file_edits.is_empty() {
            if let Some(mut content) = file_content.clone() {
                file_edits.sort_by_key(|e| std::cmp::Reverse(e.0.start_byte));

                for (range, sym, suggested) in file_edits {
                    if range.start_byte < content.len() && range.end_byte <= content.len() {
                        let orig_doc = &content[range.start_byte..range.end_byte];
                        let target_backtick = format!("`{}`", sym);
                        let replacement_backtick = format!("`{}`", suggested);

                        let mut new_doc = orig_doc.replace(&target_backtick, &replacement_backtick);
                        new_doc =
                            new_doc.replace(&format!("[[{}", sym), &format!("[[{}", suggested));

                        if new_doc != orig_doc {
                            content.replace_range(range.start_byte..range.end_byte, &new_doc);
                            file_dirty = true;
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

    println!("\n============================================================");
    println!("  ADG-SG AUDIT SUMMARY");
    println!("============================================================");
    println!("Total Files Audited: {total_files}");
    println!("Total Symbols Checked: {total_symbols_checked}");
    println!("Failed Files: {failed_files}");

    if let Some(out_path) = out {
        let report = ValidationReport {
            total_files_audited: total_files,
            total_symbols_checked,
            failed_files_count: failed_files,
            failures,
        };
        let content =
            serde_json::to_string_pretty(&report).map_err(GraphQueryError::Serialization)?;
        if let Some(parent) = out_path.parent() {
            fs::create_dir_all(parent).map_err(|e| GraphQueryError::Io(e))?;
        }
        fs::write(out_path, content).map_err(|e| GraphQueryError::Io(e))?;
        println!(
            "Saved structured validation report to: {}",
            out_path.display()
        );
    }

    if failed_files > 0 {
        if strict {
            println!("\n{RED}[FAIL] Symbol Gate validation failed. Please align headers with code.{RESET}\n");
            return Err(GraphQueryError::Security(
                "Symbol Gate validation failed".to_string(),
            ));
        } else {
            println!("\n{YELLOW}[WARN] Symbol Gate validation failed (Warning Only). Run with --strict to enforce.{RESET}\n");
        }
    } else {
        println!("\n{GREEN}[OK] 100% Symbol-to-Header Parity Achieved!{RESET}\n");
    }

    Ok(())
}

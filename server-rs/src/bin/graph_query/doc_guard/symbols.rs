//! @docs ARCHITECTURE:CodeBaseIntelligence
//! ### AI Assist Note
//! Backticked symbol extraction, identifier validation, and whitelist management.
//! ### 🔍 Debugging & Observability
//! Symbol filtering prevents path traversal and suppresses non-code doc tokens.

use std::collections::HashSet;
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

fn extract_line_backticks(content: &str, symbols: &mut Vec<String>) {
    let mut in_tick = false;
    let mut start_idx = 0;
    for (i, c) in content.char_indices() {
        if c != '`' {
            continue;
        }
        if !in_tick {
            in_tick = true;
            start_idx = i + 1;
            continue;
        }
        let term = &content[start_idx..i];
        let cleaned = term.trim().strip_suffix("()").unwrap_or(term.trim());
        if !cleaned.is_empty() && !cleaned.contains('`') {
            symbols.push(cleaned.to_string());
        }
        in_tick = false;
    }
}

fn strip_doc_prefix(trimmed: &str) -> &str {
    if let Some(stripped) = trimmed.strip_prefix("///") {
        stripped.trim()
    } else if let Some(stripped) = trimmed.strip_prefix("//!") {
        stripped.trim()
    } else if let Some(stripped) = trimmed.strip_prefix('*') {
        stripped.trim()
    } else {
        trimmed
    }
}

pub fn extract_backticked_symbols(doc: &str) -> Vec<String> {
    let mut symbols = Vec::new();
    let mut in_code_fence = false;

    for line in doc.lines() {
        let content = strip_doc_prefix(line.trim());
        if content.starts_with("```") || content.starts_with("~~~") {
            in_code_fence = !in_code_fence;
            continue;
        }
        if in_code_fence {
            continue;
        }
        extract_line_backticks(content, &mut symbols);
    }

    let mut seen = HashSet::new();
    symbols.retain(|s| seen.insert(s.clone()));
    symbols
}

fn is_path_unsafe(s_clean: &str) -> bool {
    if s_clean.contains('\0') || s_clean.contains(':') {
        return true;
    }
    if s_clean.starts_with('/') || s_clean.starts_with('\\') || Path::new(s_clean).is_absolute() {
        return true;
    }
    s_clean.split(['/', '\\']).any(|seg| seg == "..")
}

fn filter_markdown_symbol(s: &str) -> Option<String> {
    if s.chars().any(|c| " ${}[]<>=+*\"'".contains(c))
        || s.starts_with("http://")
        || s.starts_with("https://")
        || !s.chars().any(|c| c.is_alphanumeric())
    {
        return None;
    }
    Some(s.to_string())
}

fn filter_code_symbol(s: &str) -> Option<String> {
    if s.chars().any(|c| " /\\-:${}[]<>&|".contains(c)) {
        return None;
    }
    let first_valid = s
        .chars()
        .next()
        .is_some_and(|c| c.is_alphabetic() || c == '_');
    if first_valid && s.chars().all(|c| c.is_alphanumeric() || c == '_') {
        return Some(s.to_string());
    }
    None
}

pub fn filter_symbol(s: &str, is_md: bool, whitelist: &HashSet<String>) -> Option<String> {
    let s_clean = s.trim().strip_suffix("()").unwrap_or(s.trim());
    if whitelist.contains(s_clean) || is_path_unsafe(s_clean) {
        return None;
    }
    if is_md {
        filter_markdown_symbol(s_clean)
    } else {
        filter_code_symbol(s_clean)
    }
}

pub fn disk_candidate_exists(root: &Path, doc_dir: &Path, sym: &str) -> bool {
    let canonical_root = root.canonicalize().unwrap_or_else(|_| root.to_path_buf());
    for base in [root, doc_dir] {
        let normalized = crate::path_utils::lexical_normalize(&base.join(sym));
        let canonical = normalized.canonicalize().unwrap_or(normalized);
        if canonical.starts_with(&canonical_root) && canonical.is_file() {
            return true;
        }
    }
    false
}

fn load_env_keys(root: &Path, whitelist: &mut HashSet<String>) {
    let Ok(content) = fs::read_to_string(root.join("server-rs/.env.example")) else {
        return;
    };
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        if let Some(key) = trimmed.split('=').next() {
            whitelist.insert(key.trim().to_string());
        }
    }
}

pub fn build_whitelist(root: &Path) -> HashSet<String> {
    let mut whitelist: HashSet<String> = HashSet::new();
    for kw in BUILTIN_KEYWORDS {
        whitelist.insert(kw.to_string());
    }

    let globals_path = root.join(".agent/globals.json");
    if let Ok(content) = fs::read_to_string(&globals_path) {
        if let Ok(globals) = serde_json::from_str::<Vec<String>>(&content) {
            whitelist.extend(globals);
        }
    }

    load_env_keys(root, &mut whitelist);
    whitelist
}

pub fn tokenize_code_body(content: &str) -> HashSet<String> {
    let mut body = String::new();
    for line in content.lines() {
        let trimmed = line.trim();
        if !trimmed.starts_with("//") && !trimmed.starts_with("/*") && !trimmed.starts_with('*') {
            body.push_str(line);
            body.push('\n');
        }
    }
    body.split(|c: char| !c.is_alphanumeric() && c != '_')
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .collect()
}

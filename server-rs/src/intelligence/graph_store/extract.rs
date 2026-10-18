//! @docs ARCHITECTURE:Core:Intelligence
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / graph_store / extract
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Robust AST-first and regex fallback parsing with bounded line limits.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: none declared
//! - **Witness Tests**: `intelligence::graph_store::extract::tests`

use super::model::SymbolRecord;
use crate::error::AppError;
use crate::utils::parser::{Reference, Symbol, SymbolExtractor, SymbolRange};
use once_cell::sync::Lazy;
use regex::Regex;
use std::path::Path;

static PY_FUNC_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^\s*def\s+([A-Za-z_][A-Za-z0-9_]*)").expect("valid PY_FUNC_RE"));
static PY_CLASS_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^\s*class\s+([A-Za-z_][A-Za-z0-9_]*)").expect("valid PY_CLASS_RE"));
static SQL_CLASS_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)^\s*CREATE\s+TABLE\s+(?:IF\s+NOT\s+EXISTS\s+)?([A-Za-z_][A-Za-z0-9_]*)")
        .expect("valid SQL_CLASS_RE")
});
static JS_FUNC_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^\s*(?:export\s+)?(?:async\s+)?function\s+([A-Za-z_][A-Za-z0-9_]*)")
        .expect("valid JS_FUNC_RE")
});
static JS_CLASS_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^\s*(?:export\s+)?class\s+([A-Za-z_][A-Za-z0-9_]*)").expect("valid JS_CLASS_RE")
});
static JS_VAR_FUNC_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^\s*(?:export\s+)?(?:const|let|var)\s+([A-Za-z_][A-Za-z0-9_]*)\s*=")
        .expect("valid JS_VAR_FUNC_RE")
});
static SH_FUNC_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^\s*(?:function\s+)?([A-Za-z_][A-Za-z0-9_-]*)\s*(?:\(\))?\s*\{")
        .expect("valid SH_FUNC_RE")
});
static IMPORT_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"(?i)(?:import|from|require|use)\s+["']?([A-Za-z0-9_./:-]+)"#)
        .expect("valid IMPORT_RE")
});
static CALL_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\b([A-Za-z_][A-Za-z0-9_]*)\s*\(").expect("valid CALL_RE"));

static RS_IMPORT_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^\s*use\s+([A-Za-z0-9_:]+)").expect("valid RS_IMPORT_RE"));
static OTHER_IMPORT_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"^\s*import\s+(?:.+?\s+from\s+)?["']?([A-Za-z0-9_@./-]+)"#)
        .expect("valid OTHER_IMPORT_RE")
});

const IGNORED_KEYWORDS: &[&str] = &[
    "if", "while", "for", "switch", "match", "catch", "else", "return", "sizeof", "typeof",
    "require",
];

pub trait LanguageProcessor: Send + Sync {
    fn extract(
        &self,
        extractor: &mut SymbolExtractor,
        path: &Path,
        content: &str,
    ) -> Result<(Vec<SymbolRecord>, Vec<Reference>, Vec<String>), AppError>;
}

pub struct AstProcessor;
impl LanguageProcessor for AstProcessor {
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

pub struct RegexProcessor;
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

pub fn lookup_processor(ext: &str) -> Option<&'static dyn LanguageProcessor> {
    static AST_PROC: AstProcessor = AstProcessor;
    static REGEX_PROC: RegexProcessor = RegexProcessor;

    match ext {
        "rs" | "ts" | "tsx" => Some(&AST_PROC),
        "py" | "sql" | "js" | "cjs" | "mjs" | "ps1" | "sh" => Some(&REGEX_PROC),
        // Unsupported extensions fall back to empty extraction
        _ => None,
    }
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
pub fn lightweight_extract(
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
                let name_str = name.as_str();
                if !IGNORED_KEYWORDS.contains(&name_str) {
                    refs.push(Reference {
                        name: name_str.to_string(),
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
    }
    (symbols, refs, imports)
}

pub fn extract_imports(ext: &str, content: &str) -> Vec<String> {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_python_lightweight_extract() {
        let py_code = r#"
import os
from math import sqrt

class Matrix:
    def solve():
        if (True):
            calculate()
"#;
        let (symbols, refs, imports) = lightweight_extract("py", py_code);
        assert_eq!(symbols.len(), 2);
        assert_eq!(symbols[0].name, "Matrix");
        assert_eq!(symbols[1].name, "solve");

        // `if` must be filtered from refs
        assert!(!refs.iter().any(|r| r.name == "if"));
        assert!(refs.iter().any(|r| r.name == "calculate"));
        assert_eq!(imports, vec!["os", "math", "sqrt"]);
    }
}

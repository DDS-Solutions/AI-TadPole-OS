//! @docs ARCHITECTURE:Core:Intelligence
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / graph_store / heuristics
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Deterministic heuristic classification, tokenized security relevance, and test boundary detection.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: none declared
//! - **Witness Tests**: `intelligence::graph_store::heuristics::tests`

use super::model::{FileRecord, NodeRow, SymbolKind, SymbolRecord};
use crate::utils::parser::Reference;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

pub fn normalize_kind(kind: &str, is_test: bool) -> SymbolKind {
    if is_test {
        return SymbolKind::Test;
    }
    match kind {
        "struct" | "enum" | "trait" | "class" | "interface" | "type" | "impl" => SymbolKind::Class,
        _ => SymbolKind::Function,
    }
}

pub fn language_for_ext(ext: &str) -> &'static str {
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

pub fn is_test_path(path: &str) -> bool {
    let lower = path.to_lowercase().replace('\\', "/");
    lower.contains(".test.")
        || lower.contains("_test.")
        || lower.contains("_tests.")
        || lower.contains("/tests/")
        || lower.ends_with("/tests.rs")
        || lower.ends_with(".tests.rs")
        || lower.ends_with("/test.rs")
}

pub fn is_security_relevant(node: &NodeRow) -> bool {
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
    let name_lower = node.name.to_lowercase();
    let path_lower = node.file_path.to_lowercase();
    let sig_lower = node.signature.to_lowercase();

    let haystack = format!("{name_lower} {path_lower} {sig_lower}");
    let tokens: HashSet<&str> = haystack
        .split(|c: char| !c.is_alphanumeric())
        .filter(|s| !s.is_empty())
        .collect();

    TERMS.iter().any(|term| tokens.contains(term))
}

pub fn qualified(file_path: &str, symbol: &str) -> String {
    format!("{file_path}::{symbol}")
}

pub fn qualified_symbol(file_path: &str, symbol: &SymbolRecord) -> String {
    format!(
        "{}@{}-{}",
        qualified(file_path, &symbol.name),
        symbol.line_start,
        symbol.line_end
    )
}

pub fn tightest_symbol<'a>(
    file: &'a FileRecord,
    reference: &Reference,
) -> Option<&'a SymbolRecord> {
    file.symbols
        .iter()
        .filter(|sym| {
            (reference.range.start_line as i64).saturating_add(1) >= sym.line_start
                && (reference.range.end_line as i64).saturating_add(1) <= sym.line_end
        })
        .min_by_key(|sym| sym.line_end - sym.line_start)
}

pub fn match_targets(name: &str, by_name: &HashMap<Arc<str>, Vec<Arc<str>>>) -> Vec<Arc<str>> {
    let direct = name.rsplit([':', '/', '.', '\\']).next().unwrap_or(name);
    by_name.get(direct).cloned().unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_test_path_boundaries() {
        assert!(is_test_path("server-rs/src/tests/auth.rs"));
        assert!(is_test_path("server-rs/src/auth.test.ts"));
        assert!(is_test_path("server-rs/src/auth_test.go"));
        assert!(is_test_path("server-rs/tests.rs"));
        assert!(is_test_path("server-rs\\sub\\tests.rs"));

        // Negative cases (false positives avoided)
        assert!(!is_test_path("server-rs/src/contests.rs"));
        assert!(!is_test_path("server-rs/src/protests.rs"));
        assert!(!is_test_path("server-rs/src/latest.rs"));
    }

    #[test]
    fn test_is_security_relevant_token_matching() {
        let make_node = |name: &str, path: &str, sig: &str| NodeRow {
            id: 1,
            kind: "Function".to_string(),
            name: name.to_string(),
            qualified_name: format!("{path}::{name}"),
            file_path: path.to_string(),
            line_start: Some(1),
            line_end: Some(5),
            language: "rust".to_string(),
            parent_name: None,
            params: None,
            return_type: None,
            modifiers: None,
            is_test: false,
            file_hash: "hash".to_string(),
            extra: "{}".to_string(),
            signature: sig.to_string(),
            community_id: None,
        };

        // Positive security hits
        assert!(is_security_relevant(&make_node(
            "validate_token",
            "src/auth.rs",
            "fn validate_token()"
        )));
        assert!(is_security_relevant(&make_node(
            "execute_cmd",
            "src/shell.rs",
            "fn execute_cmd()"
        )));
        assert!(is_security_relevant(&make_node(
            "open_db_pool",
            "src/db.rs",
            "fn open_db_pool()"
        )));

        // Negative cases (substring containment without word boundaries)
        assert!(!is_security_relevant(&make_node(
            "monkey",
            "src/animals.rs",
            "fn monkey()"
        )));
        assert!(!is_security_relevant(&make_node(
            "render_toolbar",
            "src/ui.rs",
            "fn render_toolbar()"
        )));
        assert!(!is_security_relevant(&make_node(
            "fdb_sync",
            "src/sync.rs",
            "fn fdb_sync()"
        )));
    }
}

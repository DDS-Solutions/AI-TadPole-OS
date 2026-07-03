//! High-fidelity Symbol Extraction - Tree-sitter
//!
//! Provides the semantic parsing backbone for the engine's codebase
//! awareness, extracting functions, structs, and traits from Rust and
//! TypeScript source files.
//!
//! @docs ARCHITECTURE:CodeBaseIntelligence
//!
//! ### AI Assist Note
//! **Symbol Extraction (Tree-sitter)**: Orchestrates the high-fidelity
//! semantic parsing for the Tadpole OS engine. Extracts functions,
//! structs, traits, and interfaces from **Rust** and **TypeScript**
//! source files using tree-sitter grammars. Features **In-Memory
//! Parsing**: all AST operations are performed without intermediate
//! disk writes. Note: For massive files (>10MB), parsing may consume
//! significant RAM; AI agents should favor targeted indexed lookups
//! provided by the CodeGraph rather than repeated raw file
//! re-parsing (PARSE-01).
//!
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: Tree-sitter grammar loading failure,
//!   unsupported language extensions, or query mismatch causing
//!   incomplete symbol extraction.
//! - **Trace Scope**: `server-rs::utils::parser`

use specta::Type;
use std::path::Path;
use streaming_iterator::StreamingIterator;
use tree_sitter::{Parser, Query, QueryCursor};
use std::collections::HashSet;
use once_cell::sync::Lazy;

static BUILTIN_GLOBALS: Lazy<HashSet<String>> = Lazy::new(|| {
    let json_str = include_str!("../../../.agent/globals.json");
    serde_json::from_str(json_str).unwrap_or_default()
});

/// A semantic code element extracted from source text.
#[derive(Debug, Clone, serde::Serialize, Type)]
pub struct Symbol {
    /// The unadorned name of the symbol (e.g., function name).
    pub name: String,
    /// The type of symbol (e.g., "struct", "func", "impl").
    pub kind: String,
    /// Exact byte and line coordinates in the source file.
    pub range: SymbolRange,
    /// The first line of the definition (e.g., `pub fn main()`).
    pub signature: String,
    /// The complete implementation body of the symbol.
    pub body: String,
}

/// A reference to another symbol (e.g., a function call or type usage).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Type)]
pub struct Reference {
    /// The name of the symbol being referenced.
    pub name: String,
    /// Exact byte and line coordinates of the reference.
    pub range: SymbolRange,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Type)]
pub struct SymbolRange {
    pub start_byte: usize,
    pub end_byte: usize,
    pub start_line: usize,
    pub end_line: usize,
}

pub struct SymbolExtractor {
    rust_parser: Parser,
    ts_parser: Parser,
}

impl Default for SymbolExtractor {
    fn default() -> Self {
        Self::new()
    }
}

impl SymbolExtractor {
    pub fn new() -> Self {
        let mut rust_parser = Parser::new();
        rust_parser
            .set_language(&tree_sitter_rust::LANGUAGE.into())
            .expect("Error loading Rust grammar");

        let mut ts_parser = Parser::new();
        ts_parser
            .set_language(&tree_sitter_typescript::LANGUAGE_TSX.into())
            .expect("Error loading TSX grammar");

        Self {
            rust_parser,
            ts_parser,
        }
    }

    pub fn extract_symbols(&mut self, path: &Path, content: &str) -> Vec<Symbol> {
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");

        let mut symbols = match ext {
            "rs" => self.extract_rust(content),
            "ts" | "tsx" => self.extract_typescript(content),
            "md" => self.extract_markdown(path, content),
            _ => Vec::new(),
        };

        if !symbols.is_empty() {
            symbols.push(Symbol {
                name: "__module__".to_string(),
                kind: "module".to_string(),
                range: SymbolRange {
                    start_byte: 0,
                    end_byte: content.len(),
                    start_line: 0,
                    end_line: content.lines().count(),
                },
                signature: "".to_string(),
                body: "".to_string(),
            });
            symbols.sort_by_key(|s| s.range.start_byte);
        }

        symbols
    }

    pub fn extract_references(&mut self, path: &Path, content: &str) -> Vec<Reference> {
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");

        match ext {
            "rs" => self.extract_rust_refs(content),
            "ts" | "tsx" => self.extract_typescript_refs(content),
            "md" => self.extract_markdown_refs(content),
            _ => Vec::new(),
        }
    }

    fn extract_markdown(&mut self, path: &Path, content: &str) -> Vec<Symbol> {
        let file_stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("unknown");
        let mut title = file_stem.to_string();
        for line in content.lines() {
            if line.trim().starts_with("title:") {
                let parts: Vec<&str> = line.splitn(2, ':').collect();
                if parts.len() == 2 {
                    let cleaned = parts[1].trim().trim_matches('"').trim_matches('\'').trim();
                    if !cleaned.is_empty() {
                        title = cleaned.to_string();
                    }
                }
                break;
            }
        }

        let total_lines = content.lines().count();
        vec![Symbol {
            name: file_stem.to_string(),
            kind: "wiki".to_string(),
            range: SymbolRange {
                start_byte: 0,
                end_byte: content.len(),
                start_line: 0,
                end_line: if total_lines > 0 { total_lines - 1 } else { 0 },
            },
            signature: format!("# {}", title),
            body: content.to_string(),
        }]
    }

    fn extract_markdown_refs(&mut self, content: &str) -> Vec<Reference> {
        let mut refs = Vec::new();
        let mut start_idx = 0;

        while let Some(open_idx) = content[start_idx..].find("[[") {
            let absolute_open = start_idx + open_idx;
            if let Some(close_idx) = content[absolute_open..].find("]]") {
                let absolute_close = absolute_open + close_idx;
                let inside = &content[absolute_open + 2..absolute_close];

                let target = inside
                    .split('|')
                    .next()
                    .unwrap_or("")
                    .split('#')
                    .next()
                    .unwrap_or("")
                    .trim();

                if !target.is_empty() {
                    let start_line = content[..absolute_open].lines().count().saturating_sub(1);
                    let end_line = content[..absolute_close + 2].lines().count().saturating_sub(1);
                    refs.push(Reference {
                        name: target.to_string(),
                        range: SymbolRange {
                            start_byte: absolute_open,
                            end_byte: absolute_close + 2,
                            start_line,
                            end_line,
                        },
                    });
                }
                start_idx = absolute_close + 2;
            } else {
                break;
            }
        }

        let mut in_backticks = false;
        let mut backtick_start = 0;
        let bytes = content.as_bytes();
        for (i, &b) in bytes.iter().enumerate() {
            if b == b'`' {
                if in_backticks {
                    let term = &content[backtick_start + 1..i];
                    if is_valid_code_identifier(term) {
                        let start_line = content[..backtick_start].lines().count().saturating_sub(1);
                        let end_line = content[..i + 1].lines().count().saturating_sub(1);
                        refs.push(Reference {
                            name: term.to_string(),
                            range: SymbolRange {
                                start_byte: backtick_start,
                                end_byte: i + 1,
                                start_line,
                                end_line,
                            },
                        });
                    }
                    in_backticks = false;
                } else {
                    in_backticks = true;
                    backtick_start = i;
                }
            }
        }

        refs
    }

    fn extract_rust(&mut self, content: &str) -> Vec<Symbol> {
        let tree = match self.rust_parser.parse(content, None) {
            Some(t) => t,
            None => return Vec::new(),
        };
        let query_str = r#"
            (function_item (identifier) @name) @func
            (struct_item name: (type_identifier) @name) @struct
            (impl_item type: (_) @name) @impl
            (enum_item name: (type_identifier) @name) @enum
            (trait_item name: (type_identifier) @name) @trait
        "#;

        let query = match Query::new(&tree_sitter_rust::LANGUAGE.into(), query_str) {
            Ok(q) => q,
            Err(_) => return Vec::new(),
        };
        self.query_symbols(content, &query, tree.root_node())
    }

    fn extract_typescript(&mut self, content: &str) -> Vec<Symbol> {
        let tree = match self.ts_parser.parse(content, None) {
            Some(t) => t,
            None => return Vec::new(),
        };
        let query_str = r#"
            (function_declaration name: (identifier) @name) @func
            (class_declaration name: (type_identifier) @name) @class
            (interface_declaration name: (type_identifier) @name) @interface
            (type_alias_declaration name: (type_identifier) @name) @type
            (method_definition name: (property_identifier) @name) @method
            (variable_declarator name: (identifier) @name value: [ (arrow_function) (function_expression) ]) @func
        "#;

        let query = match Query::new(&tree_sitter_typescript::LANGUAGE_TSX.into(), query_str) {
            Ok(q) => q,
            Err(_) => return Vec::new(),
        };
        self.query_symbols(content, &query, tree.root_node())
    }

    fn query_symbols(
        &self,
        content: &str,
        query: &Query,
        root_node: tree_sitter::Node,
    ) -> Vec<Symbol> {
        let mut cursor = QueryCursor::new();
        let mut matches = cursor.matches(query, root_node, content.as_bytes());
        let mut symbols = Vec::new();

        while let Some(m) = matches.next() {
            let mut name = String::new();
            let mut kind = String::new();
            let mut full_node = None;

            for capture in m.captures {
                let capture_name = query.capture_names()[capture.index as usize];
                if capture_name == "name" {
                    name = capture
                        .node
                        .utf8_text(content.as_bytes())
                        .unwrap_or("")
                        .to_string();
                } else {
                    kind = capture_name.to_string();
                    full_node = Some(capture.node);
                }
            }

            if let Some(node) = full_node {
                let range = SymbolRange {
                    start_byte: node.start_byte(),
                    end_byte: node.end_byte(),
                    start_line: node.start_position().row,
                    end_line: node.end_position().row,
                };

                let body = content[range.start_byte..range.end_byte].to_string();
                let signature = body.lines().next().unwrap_or("").to_string();

                symbols.push(Symbol {
                    name,
                    kind,
                    range,
                    signature,
                    body,
                });
            }
        }

        symbols
    }

    fn extract_rust_refs(&mut self, content: &str) -> Vec<Reference> {
        let tree = match self.rust_parser.parse(content, None) {
            Some(t) => t,
            None => return Vec::new(),
        };
        let query_str = r#"
            (call_expression
              function: [
                (identifier) @ref
                (field_expression field: (field_identifier) @ref)
              ]
            )
            (type_identifier) @ref
        "#;
        let query = match Query::new(&tree_sitter_rust::LANGUAGE.into(), query_str) {
            Ok(q) => q,
            Err(_) => return Vec::new(),
        };
        self.query_references(content, &query, tree.root_node())
    }

    fn extract_typescript_refs(&mut self, content: &str) -> Vec<Reference> {
        let tree = match self.ts_parser.parse(content, None) {
            Some(t) => t,
            None => return Vec::new(),
        };
        let query_str = r#"
            (call_expression
              function: [
                (identifier) @ref
                (member_expression property: (property_identifier) @ref)
              ]
            )
            (new_expression
              constructor: [
                (identifier) @ref
                (member_expression property: (property_identifier) @ref)
              ]
            )
            (pair value: (identifier) @ref)
            (type_identifier) @ref
            (jsx_opening_element name: [
              (identifier) @ref
              (member_expression object: (identifier) @ref property: (property_identifier) @ref)
            ])
            (jsx_self_closing_element name: [
              (identifier) @ref
              (member_expression object: (identifier) @ref property: (property_identifier) @ref)
            ])
            (import_specifier (identifier) @ref)
            (import_clause (identifier) @ref)
            (namespace_import (identifier) @ref)
            (jsx_expression (identifier) @ref)
            (arguments (identifier) @ref)
            (variable_declarator value: (identifier) @ref)
            (assignment_expression right: (identifier) @ref)
            (shorthand_property_identifier) @ref
            (member_expression object: (identifier) @ref)
        "#;
        let query = match Query::new(&tree_sitter_typescript::LANGUAGE_TSX.into(), query_str) {
            Ok(q) => q,
            Err(_) => return Vec::new(),
        };
        self.query_references(content, &query, tree.root_node())
    }

    fn query_references(
        &self,
        content: &str,
        query: &Query,
        root_node: tree_sitter::Node,
    ) -> Vec<Reference> {
        let mut cursor = QueryCursor::new();
        let mut matches = cursor.matches(query, root_node, content.as_bytes());
        let mut refs = Vec::new();

        while let Some(m) = matches.next() {
            for capture in m.captures {
                let name = capture
                    .node
                    .utf8_text(content.as_bytes())
                    .unwrap_or("")
                    .to_string();

                if BUILTIN_GLOBALS.contains(&name) {
                    continue;
                }

                let range = SymbolRange {
                    start_byte: capture.node.start_byte(),
                    end_byte: capture.node.end_byte(),
                    start_line: capture.node.start_position().row,
                    end_line: capture.node.end_position().row,
                };

                refs.push(Reference { name, range });
            }
        }
        refs
    }
}

fn is_valid_code_identifier(s: &str) -> bool {
    if s.is_empty() || s.len() > 100 {
        return false;
    }

    let mut chars = s.chars();
    if let Some(first) = chars.next() {
        if !first.is_ascii_alphabetic() && first != '_' {
            return false;
        }
    } else {
        return false;
    }

    for c in chars {
        if !c.is_ascii_alphanumeric() && c != '_' && c != ':' && c != '.' && c != '-' {
            return false;
        }
    }

    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_extract_rust_symbols() {
        let mut extractor = SymbolExtractor::new();
        let content = r#"
            /// A test struct
            pub struct TestStruct {
                pub field: String,
            }

            impl TestStruct {
                pub fn new() -> Self {
                    Self { field: "test".to_string() }
                }
            }

            fn top_level_func() {}
        "#;

        let mut file = NamedTempFile::new().unwrap();
        write!(file, "{}", content).unwrap();
        let path = file.path().to_owned();
        let _path_rs = path.with_extension("rs");

        let symbols = extractor.extract_rust(content);

        assert!(symbols
            .iter()
            .any(|s| s.name == "TestStruct" && s.kind == "struct"));
        assert!(symbols
            .iter()
            .any(|s| s.name == "TestStruct" && s.kind == "impl"));
        assert!(symbols
            .iter()
            .any(|s| s.name == "top_level_func" && s.kind == "func"));
    }

    #[test]
    fn test_extract_typescript_symbols() {
        let mut extractor = SymbolExtractor::new();
        let content = r#"
            export interface User {
                id: string;
            }

            class UserService {
                login(user: User) {
                    return true;
                }
            }

            function helper() {}

            const arrowFunc = () => {};
            const typedArrowFunc: React.FC = () => {};
            const fnExpr = function() {};
        "#;

        let symbols = extractor.extract_typescript(content);

        assert!(symbols
            .iter()
            .any(|s| s.name == "User" && s.kind == "interface"));
        assert!(symbols
            .iter()
            .any(|s| s.name == "UserService" && s.kind == "class"));
        assert!(symbols
            .iter()
            .any(|s| s.name == "login" && s.kind == "method"));
        assert!(symbols
            .iter()
            .any(|s| s.name == "helper" && s.kind == "func"));
        assert!(symbols
            .iter()
            .any(|s| s.name == "arrowFunc" && s.kind == "func"));
        assert!(symbols
            .iter()
            .any(|s| s.name == "typedArrowFunc" && s.kind == "func"));
        assert!(symbols
            .iter()
            .any(|s| s.name == "fnExpr" && s.kind == "func"));
    }

    #[test]
    fn test_extract_typescript_refs() {
        let mut extractor = SymbolExtractor::new();
        let content = r#"
            import { foo } from './a';
            export function bar() {
                foo();
                const element = <button onClick={handle_close} />;
                useEffect(handle_scroll, []);
                const x = fn_val;
                y = assigned_val;
                const obj = { shorthand_val };
                obj_val.method();
            }
        "#;
        let path = std::path::Path::new("test.tsx");
        let refs = extractor.extract_references(path, content);
        println!("Extracted refs: {:?}", refs);
        assert!(refs.iter().any(|r| r.name == "foo"));
        assert!(refs.iter().any(|r| r.name == "handle_close"));
        assert!(refs.iter().any(|r| r.name == "handle_scroll"));
        assert!(refs.iter().any(|r| r.name == "fn_val"));
        assert!(refs.iter().any(|r| r.name == "assigned_val"));
        assert!(refs.iter().any(|r| r.name == "shorthand_val"));
        assert!(refs.iter().any(|r| r.name == "obj_val"));
    }
}

// Metadata: [parser]

//! High-fidelity symbol extraction using Tree-sitter grammars.
//!
//! This utility provides the semantic parsing backbone for the engine's
//! codebase awareness, extracting functions, structs, and traits from
//! both Rust and TypeScript source files.

use std::path::Path;
use streaming_iterator::StreamingIterator;
use tree_sitter::{Parser, Query, QueryCursor};

/// A semantic code element extracted from source text.
#[derive(Debug, Clone, serde::Serialize)]
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

#[derive(Debug, Clone, serde::Serialize)]
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

        match ext {
            "rs" => self.extract_rust(content),
            "ts" | "tsx" => self.extract_typescript(content),
            _ => Vec::new(),
        }
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
    }
}

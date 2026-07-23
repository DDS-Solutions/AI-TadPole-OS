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

use once_cell::sync::Lazy;
use specta::Type;
use std::collections::HashSet;
use std::path::Path;
use streaming_iterator::StreamingIterator;
use tree_sitter::{Parser, Query, QueryCursor};

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
    /// Optional docstring associated with the symbol.
    pub docstring: Option<String>,
    /// Exact range of the associated docstring comments in the file.
    pub docstring_range: Option<SymbolRange>,
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
    #[specta(type = u32)]
    pub start_byte: usize,
    #[specta(type = u32)]
    pub end_byte: usize,
    #[specta(type = u32)]
    pub start_line: usize,
    #[specta(type = u32)]
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
            let (module_doc, module_doc_range) = match ext {
                "rs" => self
                    .rust_parser
                    .parse(content, None)
                    .and_then(|t| extract_module_comments(t.root_node(), content))
                    .map(|(doc, range)| (Some(doc), Some(range)))
                    .unwrap_or((None, None)),
                "ts" | "tsx" => self
                    .ts_parser
                    .parse(content, None)
                    .and_then(|t| extract_module_comments(t.root_node(), content))
                    .map(|(doc, range)| (Some(doc), Some(range)))
                    .unwrap_or((None, None)),
                _ => (None, None),
            };

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
                docstring: module_doc,
                docstring_range: module_doc_range,
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
        let file_stem = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown");
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
        let md_range = SymbolRange {
            start_byte: 0,
            end_byte: content.len(),
            start_line: 0,
            end_line: if total_lines > 0 { total_lines - 1 } else { 0 },
        };
        vec![Symbol {
            name: file_stem.to_string(),
            kind: "wiki".to_string(),
            range: md_range.clone(),
            signature: format!("# {}", title),
            body: content.to_string(),
            docstring: Some(content.to_string()),
            docstring_range: Some(md_range),
        }]
    }

    fn extract_markdown_refs(&mut self, content: &str) -> Vec<Reference> {
        let mut refs = Vec::new();
        let mut start_idx = 0;

        // 1. Extract [[WikiLink]] references
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
                    let end_line = content[..absolute_close + 2]
                        .lines()
                        .count()
                        .saturating_sub(1);
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

        // 2. Extract `code` backticked references using pulldown-cmark
        let parser = pulldown_cmark::Parser::new(content).into_offset_iter();
        for (event, range) in parser {
            if let pulldown_cmark::Event::Code(code_text) = event {
                let term = code_text.trim();
                if is_valid_code_identifier(term) {
                    let start_line = content[..range.start].lines().count().saturating_sub(1);
                    let end_line = content[..range.end].lines().count().saturating_sub(1);
                    refs.push(Reference {
                        name: term.to_string(),
                        range: SymbolRange {
                            start_byte: range.start,
                            end_byte: range.end,
                            start_line,
                            end_line,
                        },
                    });
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
                let comments_res = extract_preceding_comments(node, content);
                let (docstring, docstring_range) = match comments_res {
                    Some((doc, r)) => (Some(doc), Some(r)),
                    None => (None, None),
                };

                // Detect if this is a Rust function decorated with #[command] or #[tauri::command]
                let mut is_tauri = false;
                if kind == "func" {
                    // Case A: attribute_item is a child of function_item
                    let mut cursor = node.walk();
                    if cursor.goto_first_child() {
                        loop {
                            let child = cursor.node();
                            if child.kind() == "attribute_item" {
                                let attr_text = child.utf8_text(content.as_bytes()).unwrap_or("");
                                if attr_text.contains("command") {
                                    is_tauri = true;
                                    break;
                                }
                            }
                            if !cursor.goto_next_sibling() {
                                break;
                            }
                        }
                    }

                    // Case B: attribute_item is a preceding sibling
                    if !is_tauri {
                        let mut sibling = node;
                        while let Some(prev) = sibling.prev_sibling() {
                            let p_kind = prev.kind();
                            if p_kind == "attribute_item" {
                                let attr_text = prev.utf8_text(content.as_bytes()).unwrap_or("");
                                if attr_text.contains("command") {
                                    is_tauri = true;
                                    break;
                                }
                                sibling = prev;
                            } else if p_kind == "line_comment"
                                || p_kind == "block_comment"
                                || p_kind == "comment"
                            {
                                sibling = prev;
                            } else {
                                break;
                            }
                        }
                    }
                }

                if is_tauri {
                    symbols.push(Symbol {
                        name: format!("tauri_ipc:{}", name),
                        kind: "tauri_cmd".to_string(),
                        range: range.clone(),
                        signature: signature.clone(),
                        body: body.clone(),
                        docstring: docstring.clone(),
                        docstring_range: docstring_range.clone(),
                    });
                }

                symbols.push(Symbol {
                    name,
                    kind,
                    range,
                    signature,
                    body,
                    docstring,
                    docstring_range,
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
            (call_expression
              function: (identifier) @fn_name (#eq? @fn_name "invoke")
              arguments: (arguments (string) @invoke_ref)
            )
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
                let capture_name = query.capture_names()[capture.index as usize];
                if capture_name == "fn_name"
                    || capture_name == "path"
                    || capture_name == "attr_name"
                {
                    continue;
                }

                let mut name = capture
                    .node
                    .utf8_text(content.as_bytes())
                    .unwrap_or("")
                    .to_string();

                if capture_name == "invoke_ref" {
                    name = format!("tauri_ipc:{}", name.trim_matches(|c| c == '\'' || c == '"'));
                }

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

fn extract_preceding_comments(
    node: tree_sitter::Node,
    content: &str,
) -> Option<(String, SymbolRange)> {
    let mut comments = Vec::new();
    let mut current = node;
    let mut start_byte = node.start_byte();
    let mut end_byte = node.start_byte();
    let mut start_line = node.start_position().row;
    let mut end_line = node.start_position().row;
    let mut first = true;

    while let Some(prev) = current.prev_sibling() {
        let kind = prev.kind();
        if kind == "line_comment" || kind == "block_comment" || kind == "comment" {
            let gap_start = prev.end_byte();
            let gap_end = current.start_byte();
            if gap_start <= gap_end {
                let gap = &content[gap_start..gap_end];
                if gap.chars().all(|c| c.is_whitespace()) {
                    let text = prev.utf8_text(content.as_bytes()).unwrap_or("").trim_end();
                    comments.push(text.to_string());
                    if first {
                        end_byte = prev.end_byte();
                        end_line = prev.end_position().row;
                        first = false;
                    }
                    start_byte = prev.start_byte();
                    start_line = prev.start_position().row;
                    current = prev;
                    continue;
                }
            }
        }
        break;
    }
    if comments.is_empty() {
        None
    } else {
        comments.reverse();
        let range = SymbolRange {
            start_byte,
            end_byte,
            start_line,
            end_line,
        };
        Some((comments.join("\n"), range))
    }
}

fn extract_module_comments(
    root_node: tree_sitter::Node,
    content: &str,
) -> Option<(String, SymbolRange)> {
    let mut comments = Vec::new();
    let mut cursor = root_node.walk();
    let mut start_byte = 0;
    let mut end_byte = 0;
    let mut start_line = 0;
    let mut end_line = 0;
    let mut first = true;

    if cursor.goto_first_child() {
        loop {
            let child = cursor.node();
            let kind = child.kind();
            if kind == "line_comment" || kind == "block_comment" || kind == "comment" {
                let text = child.utf8_text(content.as_bytes()).unwrap_or("").trim_end();
                comments.push(text.to_string());
                if first {
                    start_byte = child.start_byte();
                    start_line = child.start_position().row;
                    first = false;
                }
                end_byte = child.end_byte();
                end_line = child.end_position().row;
            } else if kind != "empty" && !child.is_extra() {
                break;
            }
            if !cursor.goto_next_sibling() {
                break;
            }
        }
    }
    if comments.is_empty() {
        None
    } else {
        let range = SymbolRange {
            start_byte,
            end_byte,
            start_line,
            end_line,
        };
        Some((comments.join("\n"), range))
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

        let test_struct = symbols
            .iter()
            .find(|s| s.name == "TestStruct" && s.kind == "struct")
            .expect("TestStruct not found");
        assert_eq!(test_struct.docstring.as_deref(), Some("/// A test struct"));

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

    #[test]
    fn test_extract_markdown_references() {
        let mut extractor = SymbolExtractor::default();
        let content = "This is a [[WikiLink|Custom Label]] and a [[SimpleWiki]]. \
                       Here is some code `some_symbol` and another `another_one`. \
                       Inside a triple backtick code block:\n\
                       ```rust\n\
                       let ignored_symbol = 10;\n\
                       ```\n\
                       And some HTML comment <!-- `hidden_symbol` -->.";
        let path = std::path::Path::new("test.md");
        let refs = extractor.extract_references(path, content);

        let names: Vec<String> = refs.iter().map(|r| r.name.clone()).collect();
        assert!(names.contains(&"WikiLink".to_string()));
        assert!(names.contains(&"SimpleWiki".to_string()));
        assert!(names.contains(&"some_symbol".to_string()));
        assert!(names.contains(&"another_one".to_string()));

        // Symbols inside code blocks and HTML comments must NOT be extracted
        assert!(!names.contains(&"ignored_symbol".to_string()));
        assert!(!names.contains(&"hidden_symbol".to_string()));
    }

    #[test]
    fn test_tauri_command_validation() {
        let mut extractor = SymbolExtractor::default();

        // 1. Rust Tauri Command Symbol Extraction
        let rust_content = r#"
            #[tauri::command]
            fn get_neural_status() -> String {
                "active".to_string()
            }
            
            #[command]
            fn reboot_mesh() {}
            
            fn regular_helper() {}
        "#;
        let rust_path = std::path::Path::new("main.rs");
        let symbols = extractor.extract_symbols(rust_path, rust_content);
        let sym_names: Vec<String> = symbols.iter().map(|s| s.name.clone()).collect();
        assert!(sym_names.contains(&"tauri_ipc:get_neural_status".to_string()));
        assert!(sym_names.contains(&"tauri_ipc:reboot_mesh".to_string()));
        assert!(sym_names.contains(&"regular_helper".to_string()));
        assert!(!sym_names.contains(&"tauri_ipc:regular_helper".to_string()));

        // 2. TypeScript/TSX Tauri Invoke Reference Extraction
        let ts_content = r#"
            import { invoke } from '@tauri-apps/api';
            async fn run() {
                const status = await invoke("get_neural_status");
                await invoke('reboot_mesh', { force: true });
                regular_helper();
            }
        "#;
        let ts_path = std::path::Path::new("index.tsx");
        let refs = extractor.extract_references(ts_path, ts_content);
        let ref_names: Vec<String> = refs.iter().map(|r| r.name.clone()).collect();
        assert!(ref_names.contains(&"tauri_ipc:get_neural_status".to_string()));
        assert!(ref_names.contains(&"tauri_ipc:reboot_mesh".to_string()));
        assert!(ref_names.contains(&"regular_helper".to_string()));
        assert!(!ref_names.contains(&"invoke".to_string()));
    }
}

// Metadata: [parser]

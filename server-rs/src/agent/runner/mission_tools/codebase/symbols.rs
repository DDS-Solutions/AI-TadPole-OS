//! @docs ARCHITECTURE:Registry
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Agent Runner / codebase access
//!
//! ### AI Assist Note
//! - Keep repository path authorization separate from symbol parsing and graph queries.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: ToolExecutionError, path validation failures
//! - **Telemetry Targets**: none declared

use super::*;

use std::sync::OnceLock;

impl AgentRunner {
    /// Handles `list_file_symbols`: parses a file to list functions, classes, and variables.
    pub(crate) async fn handle_list_file_symbols(
        &self,
        ctx: &RunContext,
        fc: &crate::agent::types::ToolCall,
    ) -> Result<String, ToolExecutionError> {
        let path_str = require_path(ctx, &fc.args, "list_file_symbols")?;

        let final_path = match self.require_path_safety(ctx, &path_str).await {
            Ok(p) => p,
            Err(e) => return Err(ToolExecutionError::SecurityBlocked(e)),
        };

        match self
            .read_codebase_file_helper(ctx, &final_path, &path_str)
            .await
        {
            Ok(content) => {
                let symbols = self.extract_symbols(&content, &path_str);
                if symbols.is_empty() {
                    Ok(format!("(No recognizable symbols found in {})", path_str))
                } else {
                    let symbol_list = symbols.join("\n");
                    Ok(format!("(SYMBOLS IN {}):\n\n{}", path_str, symbol_list))
                }
            }
            Err(e) => Err(ToolExecutionError::ExecutionFailed(format!(
                "Symbol listing failed for {path_str}: {e}"
            ))),
        }
    }

    /// Handles `get_symbol_body`: extracts the implementation of a specific symbol from a file.
    pub(crate) async fn handle_get_symbol_body(
        &self,
        ctx: &RunContext,
        fc: &crate::agent::types::ToolCall,
    ) -> Result<String, ToolExecutionError> {
        let path_str = require_path(ctx, &fc.args, "get_symbol_body")?;
        let symbol_name = require_str(ctx, &fc.args, "symbol", "get_symbol_body")?;

        let final_path = match self.require_path_safety(ctx, &path_str).await {
            Ok(p) => p,
            Err(e) => return Err(ToolExecutionError::SecurityBlocked(e)),
        };

        match self
            .read_codebase_file_helper(ctx, &final_path, &path_str)
            .await
        {
            Ok(content) => {
                if let Some(body) = self.extract_symbol_body(&content, &symbol_name, &path_str) {
                    Ok(format!(
                        "(BODY OF SYMBOL '{}' IN {}):\n\n{}",
                        symbol_name, path_str, body
                    ))
                } else {
                    Ok(format!(
                        "(SYMBOL '{}' NOT FOUND in {})",
                        symbol_name, path_str
                    ))
                }
            }
            Err(e) => Err(ToolExecutionError::ExecutionFailed(format!(
                "Symbol read failed for {path_str}: {e}"
            ))),
        }
    }

    /// Internal helper: Extracts a list of symbols using regex patterns based on file extension.
    pub(crate) fn extract_symbols(&self, content: &str, path: &str) -> Vec<String> {
        let ext = std::path::Path::new(path)
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("");
        let mut symbols = Vec::new();

        static RUST_RE: OnceLock<regex::Regex> = OnceLock::new();
        static JS_TS_RE: OnceLock<regex::Regex> = OnceLock::new();
        static PYTHON_RE: OnceLock<regex::Regex> = OnceLock::new();
        static FALLBACK_RE: OnceLock<regex::Regex> = OnceLock::new();

        match ext {
            "rs" => {
                let re = RUST_RE.get_or_init(|| regex::Regex::new(r"(?m)^[ \t]*(?:pub(?:\(.*\))?\s+)?(?:async\s+)?(fn|struct|enum|trait|type|const|static)\s+([a-zA-Z_][a-zA-Z0-9_]*)").unwrap());
                for cap in re.captures_iter(content) {
                    symbols.push(format!("[{}] {}", &cap[1], &cap[2]));
                }
            }
            "ts" | "js" | "tsx" | "jsx" => {
                let re = JS_TS_RE.get_or_init(|| regex::Regex::new(r"(?m)^[ \t]*(?:export\s+)?(?:async\s+)?(function|class|type|interface|const|let|var)\s+([a-zA-Z_][a-zA-Z0-9_]*)").unwrap());
                for cap in re.captures_iter(content) {
                    symbols.push(format!("[{}] {}", &cap[1], &cap[2]));
                }
            }
            "py" => {
                let re = PYTHON_RE.get_or_init(|| {
                    regex::Regex::new(r"(?m)^[ \t]*(def|class)\s+([a-zA-Z_][a-zA-Z0-9_]*)").unwrap()
                });
                for cap in re.captures_iter(content) {
                    symbols.push(format!("[{}] {}", &cap[1], &cap[2]));
                }
            }
            _ => {
                // Fallback for unknown languages: search for common patterns
                let re = FALLBACK_RE.get_or_init(|| {
                    regex::Regex::new(
                        r"(?m)^[ \t]*(?:function|class|def|fn)\s+([a-zA-Z_][a-zA-Z0-9_]*)",
                    )
                    .unwrap()
                });
                for cap in re.captures_iter(content) {
                    symbols.push(format!("[symbol] {}", &cap[1]));
                }
            }
        }
        symbols
    }

    /// Internal helper: Extracts the body of a specific symbol.
    ///
    /// ### ⚠️ Python Limitations & Parser Assumptions
    /// - **Tabs vs. Spaces**: Indents are normalized (tabs counted as 4 spaces) to handle mixed spacing.
    /// - **Decorators & Multi-line defs**: Decorators above functions and multi-line parameter definitions
    ///   are not fully parsed, only the function body following the signature is evaluated.
    /// - **Docstrings at Column 0**: Multi-line docstrings starting at column 0 (unindented) inside
    ///   a function can trigger premature termination.
    pub(crate) fn extract_symbol_body(
        &self,
        content: &str,
        symbol: &str,
        path: &str,
    ) -> Option<String> {
        let lines: Vec<&str> = content.lines().collect();
        let ext = std::path::Path::new(path)
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("");

        let prefixes: &[&str] = match ext {
            "py" => &["def ", "async def ", "class "],
            "rs" => &[
                "fn ",
                "pub fn ",
                "pub(crate) fn ",
                "pub(super) fn ",
                "async fn ",
                "pub async fn ",
                "pub(crate) async fn ",
                "pub(super) async fn ",
                "unsafe fn ",
                "pub unsafe fn ",
                "pub(crate) unsafe fn ",
                "struct ",
                "pub struct ",
                "enum ",
                "pub enum ",
                "trait ",
                "pub trait ",
            ],
            _ => &[
                "function ",
                "async function ",
                "class ",
                "const ",
                "let ",
                "var ",
                "export function ",
                "export async function ",
                "export class ",
                "export const ",
                "export let ",
                "export var ",
                "export default function ",
                "export default class ",
            ],
        };
        let start_idx = lines.iter().position(|line| {
            prefixes.iter().any(|prefix| {
                let Some(rest) = line.trim_start().strip_prefix(*prefix) else {
                    return false;
                };
                let Some(rest) = rest.trim_start().strip_prefix(symbol) else {
                    return false;
                };
                rest.chars()
                    .next()
                    .map_or(true, |ch| !ch.is_ascii_alphanumeric() && ch != '_')
            })
        });

        if let Some(start) = start_idx {
            let mut body = Vec::new();
            let mut found_start = false;
            let mut indent_level = None;
            let mut parser_state = BraceCounterState::default();

            for line in &lines[start..] {
                body.push(*line);

                if ext == "py" {
                    // Python indentation-based blocks
                    // Normalize tabs to 4 spaces to prevent tabs/spaces mismatches
                    let current_indent: usize = line
                        .chars()
                        .take_while(|c| c.is_whitespace())
                        .map(|c| if c == '\t' { 4 } else { 1 })
                        .sum();

                    let trimmed = line.trim();
                    if !trimmed.is_empty()
                        && !trimmed.starts_with('#')
                        && !trimmed.starts_with("\"\"\"")
                        && !trimmed.starts_with("'''")
                    {
                        if let Some(level) = indent_level {
                            if current_indent <= level && body.len() > 1 {
                                // Block ended
                                body.pop();
                                break;
                            }
                        } else {
                            indent_level = Some(current_indent);
                        }
                    }
                } else {
                    // Brace-based blocks (RS, JS, TS)
                    count_braces_robust(line, &mut parser_state);

                    if parser_state.opened_brace_on_line {
                        found_start = true;
                    }

                    if found_start && parser_state.current_depth == 0 {
                        break;
                    }
                }
            }
            return Some(body.join("\n"));
        }
        None
    }
}

#[derive(Default, Clone)]
pub(super) struct BraceCounterState {
    pub(super) in_string: bool,
    pub(super) in_char: bool,
    pub(super) in_block_comment: bool,
    pub(super) escaped: bool,
    pub(super) raw_string_hashes: Option<usize>,
    pub(super) in_template_literal: bool,
    pub(super) template_literal_brace_depths: Vec<i32>,
    pub(super) current_depth: i32,
    pub(super) opened_brace_on_line: bool,
}

pub(super) fn count_braces_robust(line: &str, state: &mut BraceCounterState) {
    state.opened_brace_on_line = false;
    let mut chars = line.char_indices().peekable();

    while let Some((_, c)) = chars.next() {
        if state.in_block_comment {
            if c == '*' {
                if let Some((_, '/')) = chars.peek() {
                    chars.next();
                    state.in_block_comment = false;
                }
            }
            continue;
        }

        if state.raw_string_hashes.is_some() {
            if c == '"' {
                let n = state.raw_string_hashes.unwrap();
                let mut hash_count = 0;
                let mut temp_chars = chars.clone();
                while hash_count < n {
                    if let Some((_, '#')) = temp_chars.peek() {
                        temp_chars.next();
                        hash_count += 1;
                    } else {
                        break;
                    }
                }
                if hash_count == n {
                    for _ in 0..n {
                        chars.next();
                    }
                    state.raw_string_hashes = None;
                }
            }
            continue;
        }

        if state.in_string {
            if state.escaped {
                state.escaped = false;
            } else if c == '\\' {
                state.escaped = true;
            } else if c == '"' {
                state.in_string = false;
            }
            continue;
        }

        if state.in_char {
            if state.escaped {
                state.escaped = false;
            } else if c == '\\' {
                state.escaped = true;
            } else if c == '\'' {
                state.in_char = false;
            }
            continue;
        }

        if state.in_template_literal {
            if state.escaped {
                state.escaped = false;
            } else if c == '\\' {
                state.escaped = true;
            } else if c == '`' {
                state.in_template_literal = false;
            } else if c == '$' {
                if let Some((_, '{')) = chars.peek() {
                    chars.next();
                    state.current_depth += 1;
                    state.opened_brace_on_line = true;
                    if state.template_literal_brace_depths.len() < MAX_TEMPLATE_LITERAL_DEPTH {
                        state
                            .template_literal_brace_depths
                            .push(state.current_depth);
                    }
                    state.in_template_literal = false;
                }
            }
            continue;
        }

        // We are in normal code state
        state.escaped = false;

        // Check for line comments or block comments
        if c == '/' {
            if let Some((_, '/')) = chars.peek() {
                break; // Line comment ends parsing for this line
            } else if let Some((_, '*')) = chars.peek() {
                chars.next();
                state.in_block_comment = true;
                continue;
            }
        }

        // Check for Rust raw string start
        if c == 'r' {
            let mut hash_count = 0;
            let temp_chars = chars.clone();
            let mut found_raw_start = false;
            for (_, next_c) in temp_chars {
                if next_c == '#' {
                    hash_count += 1;
                } else if next_c == '"' {
                    found_raw_start = true;
                    break;
                } else {
                    break;
                }
            }
            if found_raw_start {
                for _ in 0..(hash_count + 1) {
                    chars.next();
                }
                state.raw_string_hashes = Some(hash_count);
                continue;
            }
        }

        // Normal triggers
        if c == '"' {
            state.in_string = true;
        } else if c == '\'' {
            state.in_char = true;
        } else if c == '`' {
            state.in_template_literal = true;
        } else if c == '{' {
            state.current_depth += 1;
            state.opened_brace_on_line = true;
        } else if c == '}' {
            state.current_depth -= 1;
            // Check if we just exited a template literal interpolation block
            if let Some(&depth) = state.template_literal_brace_depths.last() {
                if state.current_depth < depth {
                    state.template_literal_brace_depths.pop();
                    state.in_template_literal = true;
                }
            }
        }
    }
}

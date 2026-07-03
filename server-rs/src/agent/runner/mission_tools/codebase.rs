//! @docs ARCHITECTURE:Registry
//! @docs ARCHITECTURE:Runner
//! 
//! ### AI Assist Note
//! **Codebase Tools**: Codebase Navigation, path safety validation, and robust brace parsing.
//! This module implements high-fidelity logic for the Sovereign Reality layer.
//! Implements **Sovereignty Guard** (Oversight for codebase read/write/symbols) and **Breadcrumb Resolution** for ambiguous paths.
//! 
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: Runtime logic error, state desynchronization, or resource exhaustion.
//! - **Telemetry Link**: Search `[codebase]` in tracing logs.

use crate::agent::runner::{AgentRunner, RunContext};
use crate::agent::runner::tools::error::ToolExecutionError;
use std::sync::OnceLock;
use super::require_str;

const CODEBASE_READ_MAX_CHARS: usize = 30_000;
const BREADCRUMB_CAP: usize = 30;
const MAX_TEMPLATE_LITERAL_DEPTH: usize = 32;

impl AgentRunner {
    /// Handles `read_codebase_file`: allows reading files from the project root.
    ///
    /// ### 🛡️ Security Filter (Sovereign)
    /// - **Oversight**: Requires manual approval to access files outside the
    ///   mission sandbox.
    /// - **Credential Filter**: Blocks any files containing "key", "token", or
    ///   ".env" to prevent data leakage.
    /// - **Breadcrumb Resolution**: If a relative path is ambiguous, uses
    ///   the `RunContext` history to resolve the absolute path.
    pub(crate) async fn handle_read_codebase_file(
        &self,
        ctx: &RunContext,
        fc: &crate::agent::types::ToolCall,
    ) -> Result<String, ToolExecutionError> {
        let path_str = require_str(ctx, &fc.args, "path", "read_codebase_file")?;

        tracing::info!(
            "🔍 [Sovereignty] Agent {} requesting codebase read: {}",
            ctx.agent_id,
            path_str
        );

        self.broadcast_agent(
            ctx,
            &format!(
                "🔍 Oversight: wants to read codebase file: {}. Review required.",
                path_str
            ),
            "warning",
        );

        let approved = self
            .submit_oversight(
                crate::agent::types::ToolCallAudit {
                    id: uuid::Uuid::new_v4().to_string(),
                    agent_id: ctx.agent_id.clone(),
                    mission_id: Some(ctx.mission_id.clone()),
                    skill: "read_codebase_file".to_string(),
                    params: fc.args.clone(),
                    department: ctx.department.clone(),
                    description: format!(
                        "Reading codebase file for architectural analysis: {}",
                        path_str
                    ),
                    timestamp: chrono::Utc::now().to_rfc3339(),
                },
                Some(ctx.mission_id.clone()),
            )
            .await?;

        if !approved {
            return Ok("(Codebase read REJECTED by Oversight)".to_string());
        }

        let final_path = match self.require_path_safety(ctx, &path_str).await {
            Ok(p) => p,
            Err(e) => return Ok(e),
        };

        match self.read_codebase_file_helper(ctx, &final_path, &path_str).await {
            Ok(content) => {
                let truncated = self.safe_truncate(&content, CODEBASE_READ_MAX_CHARS);
                Ok(format!("(FILE CONTENT OF {}):\n\n{}", path_str, truncated))
            }
            Err(e) => Ok(format!("(CODEBASE READ FAILED for {}: {})", path_str, e)),
        }
    }

    /// Handles `list_file_symbols`: parses a file to list functions, classes, and variables.
    pub(crate) async fn handle_list_file_symbols(
        &self,
        ctx: &RunContext,
        fc: &crate::agent::types::ToolCall,
    ) -> Result<String, ToolExecutionError> {
        let path_str = require_str(ctx, &fc.args, "path", "list_file_symbols")?;

        self.broadcast_agent(
            ctx,
            &format!(
                "🔍 Oversight: wants to list symbols in codebase file: {}. Review required.",
                path_str
            ),
            "warning",
        );

        let approved = self
            .submit_oversight(
                crate::agent::types::ToolCallAudit {
                    id: uuid::Uuid::new_v4().to_string(),
                    agent_id: ctx.agent_id.clone(),
                    mission_id: Some(ctx.mission_id.clone()),
                    skill: "list_file_symbols".to_string(),
                    params: fc.args.clone(),
                    department: ctx.department.clone(),
                    description: format!(
                        "Listing codebase symbols for architectural analysis: {}",
                        path_str
                    ),
                    timestamp: chrono::Utc::now().to_rfc3339(),
                },
                Some(ctx.mission_id.clone()),
            )
            .await?;

        if !approved {
            return Ok("(List codebase symbols REJECTED by Oversight)".to_string());
        }

        let final_path = match self.require_path_safety(ctx, &path_str).await {
            Ok(p) => p,
            Err(e) => return Ok(e),
        };

        match self.read_codebase_file_helper(ctx, &final_path, &path_str).await {
            Ok(content) => {
                let symbols = self.extract_symbols(&content, &path_str);
                if symbols.is_empty() {
                    Ok(format!("(No recognizable symbols found in {})", path_str))
                } else {
                    let symbol_list = symbols.join("\n");
                    Ok(format!("(SYMBOLS IN {}):\n\n{}", path_str, symbol_list))
                }
            }
            Err(e) => Ok(format!("(LIST SYMBOLS FAILED for {}: {})", path_str, e)),
        }
    }

    /// Handles `get_symbol_body`: extracts the implementation of a specific symbol from a file.
    pub(crate) async fn handle_get_symbol_body(
        &self,
        ctx: &RunContext,
        fc: &crate::agent::types::ToolCall,
    ) -> Result<String, ToolExecutionError> {
        let path_str = require_str(ctx, &fc.args, "path", "get_symbol_body")?;
        let symbol_name = require_str(ctx, &fc.args, "symbol", "get_symbol_body")?;

        self.broadcast_agent(
            ctx,
            &format!(
                "🔍 Oversight: wants to retrieve symbol body of '{}' in: {}. Review required.",
                symbol_name, path_str
            ),
            "warning",
        );

        let approved = self
            .submit_oversight(
                crate::agent::types::ToolCallAudit {
                    id: uuid::Uuid::new_v4().to_string(),
                    agent_id: ctx.agent_id.clone(),
                    mission_id: Some(ctx.mission_id.clone()),
                    skill: "get_symbol_body".to_string(),
                    params: fc.args.clone(),
                    department: ctx.department.clone(),
                    description: format!(
                        "Extracting symbol implementation for architectural analysis: {} in {}",
                        symbol_name, path_str
                    ),
                    timestamp: chrono::Utc::now().to_rfc3339(),
                },
                Some(ctx.mission_id.clone()),
            )
            .await?;

        if !approved {
            return Ok("(Get codebase symbol body REJECTED by Oversight)".to_string());
        }

        let final_path = match self.require_path_safety(ctx, &path_str).await {
            Ok(p) => p,
            Err(e) => return Ok(e),
        };

        match self.read_codebase_file_helper(ctx, &final_path, &path_str).await {
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
            Err(e) => Ok(format!("(GET SYMBOL FAILED for {}: {})", path_str, e)),
        }
    }

    /// Centralized safety path validator and resolver helper
    pub(crate) async fn require_path_safety(
        &self,
        ctx: &RunContext,
        path_str: &str,
    ) -> Result<crate::utils::security::SafePath, String> {
        let sensitive_patterns = [".env", "key", "token", "credential", "secret", "private"];
        if sensitive_patterns
            .iter()
            .any(|p| path_str.to_lowercase().contains(p))
        {
            return Err(format!(
                "(SECURITY BLOCKED: Access to sensitive file '{}' is prohibited.)",
                path_str
            ));
        }

        let root = &ctx.workspace_root;
        let target_path = match crate::utils::security::validate_path(root, path_str) {
            Ok(p) => p,
            Err(e) => {
                return Err(format!("(SECURITY BLOCKED: {})", e));
            }
        };

        let mut final_path = target_path.clone();
        if tokio::fs::metadata(&final_path).await.is_err() {
            let breadcrumbs = ctx.last_accessed_files.lock();
            if let Some(resolved) = breadcrumbs.iter().find(|p| {
                let p_path = std::path::Path::new(p);
                let target = std::path::Path::new(path_str);
                p_path.ends_with(target)
            }) {
                tracing::info!(
                    "🧩 [Context] Resolved ambiguous codebase path '{}' to '{}' via breadcrumbs",
                    path_str,
                    resolved
                );
                final_path = crate::utils::security::SafePath::from_trusted(root.join(resolved));
            }
        }

        if tokio::fs::metadata(&final_path).await.is_err()
            && !path_str.contains('/')
            && !path_str.contains('\\')
        {
            let common_dirs = ["src", "src/agent", "server-rs/src", "server-rs/src/agent"];
            for dir in common_dirs {
                let alt_path = root.join(dir).join(path_str);
                if tokio::fs::metadata(&alt_path).await.is_ok() {
                    tracing::info!("🧩 [Context] Resolved ambiguous codebase path '{}' to '{:?}' via common-dirs", path_str, alt_path);
                    final_path = crate::utils::security::SafePath::from_trusted(alt_path);
                    break;
                }
            }
        }

        Ok(final_path)
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
                let re = PYTHON_RE.get_or_init(|| regex::Regex::new(r"(?m)^[ \t]*(def|class)\s+([a-zA-Z_][a-zA-Z0-9_]*)").unwrap());
                for cap in re.captures_iter(content) {
                    symbols.push(format!("[{}] {}", &cap[1], &cap[2]));
                }
            }
            _ => {
                // Fallback for unknown languages: search for common patterns
                let re = FALLBACK_RE.get_or_init(|| regex::Regex::new(
                    r"(?m)^[ \t]*(?:function|class|def|fn)\s+([a-zA-Z_][a-zA-Z0-9_]*)",
                ).unwrap());
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
    pub(crate) fn extract_symbol_body(&self, content: &str, symbol: &str, path: &str) -> Option<String> {
        let lines: Vec<&str> = content.lines().collect();
        let ext = std::path::Path::new(path)
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("");

        // Find the line where the symbol is defined
        let start_idx = match ext {
            "py" => lines.iter().position(|l| {
                l.contains(&format!("def {}", symbol)) || l.contains(&format!("class {}", symbol))
            }),
            "rs" => lines.iter().position(|l| {
                l.contains(&format!("fn {}", symbol))
                    || l.contains(&format!("struct {}", symbol))
                    || l.contains(&format!("enum {}", symbol))
                    || l.contains(&format!("trait {}", symbol))
            }),
            _ => lines.iter().position(|l| {
                l.contains(&format!("function {}", symbol))
                    || l.contains(&format!("class {}", symbol))
                    || l.contains(&format!("const {}", symbol))
            }),
        };

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
                    let current_indent: usize = line.chars()
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

                    if line.contains('{') {
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

    /// Read file and record breadcrumb
    pub(crate) async fn read_codebase_file_helper(
        &self,
        ctx: &RunContext,
        final_path: &std::path::Path,
        path_str: &str,
    ) -> Result<String, String> {
        match tokio::fs::read_to_string(final_path).await {
            Ok(content) => {
                self.record_breadcrumb(ctx, final_path, path_str);
                Ok(content)
            }
            Err(e) => Err(format!("{}", e)),
        }
    }

    /// Record path access breadcrumb
    pub(crate) fn record_breadcrumb(&self, ctx: &RunContext, final_path: &std::path::Path, path_str: &str) {
        let mut breadcrumbs = ctx.last_accessed_files.lock();
        let path_to_record = if final_path.is_absolute() {
            final_path.to_string_lossy().to_string()
        } else {
            path_str.to_string()
        };

        if !breadcrumbs.contains(&path_to_record) {
            breadcrumbs.push(path_to_record);
            if breadcrumbs.len() > BREADCRUMB_CAP {
                breadcrumbs.remove(0);
            }
        }
    }
}

#[derive(Default, Clone)]
struct BraceCounterState {
    in_string: bool,
    in_char: bool,
    in_block_comment: bool,
    escaped: bool,
    raw_string_hashes: Option<usize>,
    in_template_literal: bool,
    template_literal_brace_depths: Vec<i32>,
    current_depth: i32,
}

fn count_braces_robust(line: &str, state: &mut BraceCounterState) {
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
                    if state.template_literal_brace_depths.len() < MAX_TEMPLATE_LITERAL_DEPTH {
                        state.template_literal_brace_depths.push(state.current_depth);
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::runner::RunContext;
    use crate::state::AppState;
    use std::sync::Arc;

    async fn setup_test_runner() -> (AgentRunner, Arc<AppState>) {
        let state = Arc::new(AppState::new_minimal_mock().await);
        let runner = AgentRunner::new(state.clone());
        (runner, state)
    }

    #[test]
    fn test_count_braces_robust() {
        let mut state = BraceCounterState::default();
        count_braces_robust("fn hello() {}", &mut state);
        assert_eq!(state.current_depth, 0);

        let mut state = BraceCounterState::default();
        count_braces_robust("fn hello() {", &mut state);
        assert_eq!(state.current_depth, 1);

        let mut state = BraceCounterState::default();
        count_braces_robust("}", &mut state);
        assert_eq!(state.current_depth, -1);

        // String literals
        let mut state = BraceCounterState::default();
        count_braces_robust("let x = \"{ escaped brace }\";", &mut state);
        assert_eq!(state.current_depth, 0);

        // Char literals
        let mut state = BraceCounterState::default();
        count_braces_robust("let x = '{';", &mut state);
        assert_eq!(state.current_depth, 0);

        let mut state = BraceCounterState::default();
        count_braces_robust("let x = '}';", &mut state);
        assert_eq!(state.current_depth, 0);

        // Nested braces
        let mut state = BraceCounterState::default();
        count_braces_robust("let x = '{'; fn nested() {", &mut state);
        assert_eq!(state.current_depth, 1);

        // Comments
        let mut state = BraceCounterState::default();
        count_braces_robust("let x = \"}\"; // comment with {", &mut state);
        assert_eq!(state.current_depth, 0);

        // Block comments spanning lines
        let mut state = BraceCounterState::default();
        count_braces_robust("/* { brace in block comment */", &mut state);
        assert_eq!(state.current_depth, 0);
        assert!(!state.in_block_comment);

        let mut state = BraceCounterState::default();
        count_braces_robust("/* start comment", &mut state);
        assert_eq!(state.current_depth, 0);
        assert!(state.in_block_comment);
        count_braces_robust("brace { still in comment */ }", &mut state);
        assert_eq!(state.current_depth, -1); // only the trailing `}` should count
        assert!(!state.in_block_comment);

        // Rust Raw strings
        let mut state = BraceCounterState::default();
        count_braces_robust("let raw = r#\" { raw content } \"#;", &mut state);
        assert_eq!(state.current_depth, 0);

        // JS template literals with interpolation
        let mut state = BraceCounterState::default();
        count_braces_robust("let msg = `hello ${name}`;", &mut state);
        assert_eq!(state.current_depth, 0);

        let mut state = BraceCounterState::default();
        count_braces_robust("let msg = `nested ${ {a: '{'} }`;", &mut state);
        assert_eq!(state.current_depth, 0);

        // Multi-level template literals nesting
        let mut state = BraceCounterState::default();
        count_braces_robust("let s = `${ a ? `${b}` : c }`;", &mut state);
        assert_eq!(state.current_depth, 0);

        let mut state = BraceCounterState::default();
        count_braces_robust("let s = `a ${ `${'}'}` } b`;", &mut state);
        assert_eq!(state.current_depth, 0);
    }

    #[tokio::test]
    async fn test_extract_symbols_rust() {
        let (runner, _) = setup_test_runner().await;
        let content = r#"
            pub fn run() {}
            struct Data {}
            enum Kind {}
        "#;
        let symbols = runner.extract_symbols(content, "src/main.rs");
        assert_eq!(symbols.len(), 3);
        assert_eq!(symbols[0], "[fn] run");
        assert_eq!(symbols[1], "[struct] Data");
        assert_eq!(symbols[2], "[enum] Kind");
    }

    #[tokio::test]
    async fn test_extract_symbol_body_rust() {
        let (runner, _) = setup_test_runner().await;
        let content = r#"
            fn add(a: i32, b: i32) -> i32 {
                let sum = a + b;
                sum
            }
        "#;
        let body = runner.extract_symbol_body(content, "add", "src/main.rs");
        assert!(body.is_some());
        let body_str = body.unwrap();
        assert!(body_str.contains("fn add"));
        assert!(body_str.contains("let sum = a + b;"));
        assert!(body_str.contains("}"));
    }

    #[tokio::test]
    async fn test_extract_symbol_body_python_indentation() {
        let (runner, _) = setup_test_runner().await;
        let content = r#"
def calc(a, b):
    # This is a tab expanded indent
	val = a + b
	return val

def another_func():
    pass
"#;
        let body = runner.extract_symbol_body(content, "calc", "main.py");
        assert!(body.is_some());
        let body_str = body.unwrap();
        assert!(body_str.contains("def calc"));
        assert!(body_str.contains("val = a + b"));
        assert!(body_str.contains("return val"));
        assert!(!body_str.contains("def another_func"));
    }

    #[tokio::test]
    async fn test_require_path_safety() {
        let (runner, _) = setup_test_runner().await;
        let ctx = RunContext::default();
        
        // Prohibited sensitive paths
        let res = runner.require_path_safety(&ctx, ".env").await;
        assert!(res.is_err());
        assert!(res.unwrap_err().contains("SECURITY BLOCKED"));

        let res = runner.require_path_safety(&ctx, "prod.secret").await;
        assert!(res.is_err());
        assert!(res.unwrap_err().contains("SECURITY BLOCKED"));

        // Valid relative paths
        let res = runner.require_path_safety(&ctx, "src/agent/runner/mission_tools.rs").await;
        assert!(res.is_ok());
    }
}

// Metadata: [codebase]

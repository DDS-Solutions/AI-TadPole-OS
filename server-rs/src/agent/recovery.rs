//! @docs ARCHITECTURE:Agent
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Agent Runner / recovery
//! - **Primary Entrypoint**: `extract_tool_call`
//!
//! ### AI Assist Note
//! Recovery parses provider text into tool arguments; execution belongs to the runner.
//!
//! ### 🔍 Debugging & Observability
//! - **Telemetry Targets**: none declared

use once_cell::sync::Lazy;
use regex::Regex;

/// Regex for locating the start of a tool call tag (Groq/Llama 3 style)
static FUNCTION_START_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?s)<function=([a-zA-Z0-9_-]+)")
        .expect("Static tool-call start parser regex MUST be valid.")
});

/// Regex for locating code-based tool call blocks (<tool_code>, <tool_call>, <code>, or ```tool_call)
static CODE_TOOL_BLOCK_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?s)(?:<(?P<tag>tool_code|tool_call|code)>\s*(?P<xml_code>.*?)</(?P<closing_tag>tool_code|tool_call|code)>|```(?:tool_code|tool_call|tool|python)\s*(?P<fence_code>.*?)```)")
        .expect("Static code tool-call regex MUST be valid.")
});

/// Regex for extracting function invocation from code: name(args)
static CODE_FN_INVOCATION_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?s)([a-zA-Z0-9_-]+)\s*\(([\s\S]*)\)")
        .expect("Static code fn invocation regex MUST be valid.")
});

/// Regex for parsing Python-style keyword arguments: key = value
static PY_KWARG_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"(?s)([a-zA-Z0-9_]+)\s*=\s*(?:"""(?P<tq1>[\s\S]*?)"""|'''(?P<tq2>[\s\S]*?)'''|"(?P<dq>[^"\\]*(?:\\.[^"\\]*)*)"|'(?P<sq>[^'\\]*(?:\\.[^'\\]*)*)'|(?P<num>-?\d+(?:\.\d+)?)|(?P<bool>True|False|true|false))"#)
        .expect("Static python kwarg regex MUST be valid.")
});

/// Extracts a single tool call from a string, checking both Groq/Llama-style JSON tags
/// and code-block/Pythonic function call representations.
/// Returns Option<(function_name, arguments_json_string, raw_matched_segment_to_strip)>.
pub(crate) fn extract_tool_call(s: &str) -> Option<(String, String, String)> {
    if let Some(res) = extract_function_tag_call(s) {
        return Some(res);
    }
    extract_code_tool_call(s)
}

/// Extracts a function call formatted as `<function=name>{...}</function>`.
fn extract_function_tag_call(s: &str) -> Option<(String, String, String)> {
    let start_match = FUNCTION_START_REGEX.captures(s)?;
    let name = start_match.get(1)?.as_str().to_string();
    let match_start = start_match.get(0)?.start();
    let start_search_idx = start_match.get(0)?.end();

    // Find the first '{' after the tag start
    let brace_start = s[start_search_idx..].find('{')? + start_search_idx;

    let mut brace_count = 0;
    let mut brace_end = None;
    let mut in_string = false;
    let mut escaped = false;
    for (i, c) in s[brace_start..].char_indices() {
        if escaped {
            escaped = false;
            continue;
        }
        if c == '\\' && in_string {
            escaped = true;
            continue;
        }
        if c == '"' {
            in_string = !in_string;
            continue;
        }
        if !in_string {
            if c == '{' {
                brace_count += 1;
            } else if c == '}' {
                brace_count -= 1;
                if brace_count == 0 {
                    brace_end = Some(brace_start + i + 1);
                    break;
                }
            }
        }
    }

    let brace_end = brace_end?;
    let args_json = &s[brace_start..brace_end];

    // Find the end of the entire matched segment including optional </function> or >
    let mut match_end = brace_end;
    let lookahead = &s[brace_end..];
    if lookahead.starts_with("</function>") {
        match_end += "</function>".len();
    } else if lookahead.starts_with("</function>>") {
        match_end += "</function>>".len();
    } else if lookahead.starts_with('>') {
        match_end += 1;
    }

    // Also consume any trailing closed parenthesis commonly hallucinated
    if s[match_end..].starts_with(')') {
        match_end += 1;
    }

    let raw_match = &s[match_start..match_end];
    Some((name, args_json.to_string(), raw_match.to_string()))
}

/// Extracts a function call formatted inside `<tool_code>`, `<tool_call>`, or markdown code blocks.
fn extract_code_tool_call(s: &str) -> Option<(String, String, String)> {
    let block_cap = CODE_TOOL_BLOCK_REGEX.captures(s)?;
    if block_cap.name("tag").map(|tag| tag.as_str())
        != block_cap.name("closing_tag").map(|tag| tag.as_str())
    {
        return None;
    }
    let raw_match = block_cap.get(0)?.as_str().to_string();
    let code_body = block_cap
        .name("xml_code")
        .or_else(|| block_cap.name("fence_code"))?
        .as_str()
        .trim();

    let fn_cap = CODE_FN_INVOCATION_REGEX.captures(code_body)?;
    let name = fn_cap.get(1)?.as_str().trim().to_string();
    let args_raw = fn_cap.get(2)?.as_str().trim();

    let mut map = serde_json::Map::new();
    for cap in PY_KWARG_REGEX.captures_iter(args_raw) {
        if let Some(key_match) = cap.get(1) {
            let key = key_match.as_str().to_string();
            let val = if let Some(v) = cap.name("tq1").or_else(|| cap.name("tq2")) {
                serde_json::Value::String(v.as_str().trim().to_string())
            } else if let Some(v) = cap.name("dq") {
                let unescaped = v.as_str().replace(r#"\""#, r#"""#).replace(r"\n", "\n");
                serde_json::Value::String(unescaped)
            } else if let Some(v) = cap.name("sq") {
                let unescaped = v.as_str().replace(r"\'", "'").replace(r"\n", "\n");
                serde_json::Value::String(unescaped)
            } else if let Some(v) = cap.name("num") {
                if let Ok(i) = v.as_str().parse::<i64>() {
                    serde_json::Value::Number(i.into())
                } else if let Ok(f) = v.as_str().parse::<f64>() {
                    if let Some(n) = serde_json::Number::from_f64(f) {
                        serde_json::Value::Number(n)
                    } else {
                        serde_json::Value::Null
                    }
                } else {
                    serde_json::Value::Null
                }
            } else if let Some(v) = cap.name("bool") {
                serde_json::Value::Bool(v.as_str().to_lowercase() == "true")
            } else {
                serde_json::Value::Null
            };
            map.insert(key, val);
        }
    }

    // Fallback: positional string argument (e.g. func("""..."""))
    if map.is_empty() && !args_raw.is_empty() {
        let trimmed = args_raw.trim();
        if (trimmed.starts_with(r#""""#) && trimmed.ends_with(r#""""#))
            || (trimmed.starts_with("'''") && trimmed.ends_with("'''"))
        {
            if trimmed.len() >= 6 {
                let inner = &trimmed[3..trimmed.len() - 3].trim();
                map.insert(
                    "directive".to_string(),
                    serde_json::Value::String(inner.to_string()),
                );
            }
        } else if (trimmed.starts_with('"') && trimmed.ends_with('"'))
            || (trimmed.starts_with('\'') && trimmed.ends_with('\''))
        {
            if trimmed.len() >= 2 {
                let inner = &trimmed[1..trimmed.len() - 1].trim();
                map.insert(
                    "directive".to_string(),
                    serde_json::Value::String(inner.to_string()),
                );
            }
        } else if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(trimmed) {
            if parsed.is_object() {
                return Some((name, parsed.to_string(), raw_match));
            }
        }
    }

    let json_str = serde_json::to_string(&serde_json::Value::Object(map)).ok()?;
    Some((name, json_str, raw_match))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plain_and_empty_responses_have_no_recovered_tool() {
        for text in ["", "The task is complete.", "普通の応答です。"] {
            assert!(extract_tool_call(text).is_none());
        }
    }

    #[test]
    fn test_recovery_requires_matching_block_delimiters() {
        for tag in ["tool_code", "tool_call", "code"] {
            let text = format!("<{tag}>search(query=\"hello\")</{tag}>");
            let (name, args, raw) = extract_tool_call(&text).unwrap();
            assert_eq!(name, "search");
            assert_eq!(
                serde_json::from_str::<serde_json::Value>(&args).unwrap()["query"],
                "hello"
            );
            assert_eq!(raw, text);
        }
        for language in ["tool_code", "tool_call", "tool", "python"] {
            let text = format!("```{language}\nsearch(query=\"hello\")\n```");
            assert_eq!(extract_tool_call(&text).unwrap().0, "search");
        }
        for text in [
            "<tool_code>search(query=\"hello\")</tool_call>",
            "<code>search(query=\"hello\")</tool_code>",
            "<tool_call>search(query=\"hello\")```",
            "```tool_call\nsearch(query=\"hello\")</tool_call>",
            "<tool_code>search(query=\"hello\")",
            "```python\nsearch(query=\"hello\")",
        ] {
            assert!(extract_tool_call(text).is_none(), "accepted {text}");
        }
    }

    #[test]
    fn test_function_tag_can_be_stripped_to_plain_or_empty_text() {
        for remaining in ["", "\nDone."] {
            let text = format!("<function=search>{{\"query\":\"hello\"}}</function>{remaining}");
            let (_, _, raw) = extract_tool_call(&text).unwrap();
            let residual = text.replacen(&raw, "", 1);
            assert_eq!(residual, remaining);
            assert!(extract_tool_call(&residual).is_none());
        }
    }

    #[test]
    fn test_extract_tool_call_nested_json() {
        // Nested JSON extraction test
        let text = "<function=search>{\"query\": \"hello\", \"filters\": {\"category\": \"news\"}}</function>";
        let res = extract_tool_call(text);
        assert!(res.is_some());
        let (name, args_str, raw_match) = res.unwrap();
        assert_eq!(name, "search");
        assert_eq!(
            args_str,
            "{\"query\": \"hello\", \"filters\": {\"category\": \"news\"}}"
        );
        assert_eq!(raw_match, text);

        // Trailing parenthesis hallucinated test
        let text2 = "<function=write_file>({\"path\": \"test.txt\", \"content\": \"hello\"})";
        let res2 = extract_tool_call(text2);
        assert!(res2.is_some());
        let (name2, args_str2, raw_match2) = res2.unwrap();
        assert_eq!(name2, "write_file");
        assert_eq!(
            args_str2,
            "{\"path\": \"test.txt\", \"content\": \"hello\"}"
        );
        assert_eq!(raw_match2, text2);
    }

    #[test]
    fn test_extract_tool_call_with_braces_in_string() {
        let text = "<function=write_file>{\"path\": \"test.rs\", \"content\": \"fn main() { println!(\\\"hello\\\"); }\"}</function>";
        let res = extract_tool_call(text);
        assert!(res.is_some());
        let (name, args_str, _) = res.unwrap();
        assert_eq!(name, "write_file");
        assert_eq!(
            args_str,
            "{\"path\": \"test.rs\", \"content\": \"fn main() { println!(\\\"hello\\\"); }\"}"
        );
    }

    #[test]
    fn test_extract_code_tool_call_gemma_style() {
        let text = r#"
## Unified Technical Specification (UTS) Generated

<tool_code>
issue_alpha_directive(
    directive="""
    **OBJECTIVE**: Generate a Unified Technical Specification.
    Line 2 of objective.
    """,
    agent_id=2,
    agent_name="tadpole_alpha"
)
</tool_code>

> [!IMPORTANT]
> Mission status set to Spec Review.
"#;
        let res = extract_tool_call(text);
        assert!(res.is_some(), "Expected code tool call to be extracted");
        let (name, args_str, raw_match) = res.unwrap();
        assert_eq!(name, "issue_alpha_directive");
        assert!(raw_match.starts_with("<tool_code>"));
        assert!(raw_match.ends_with("</tool_code>"));

        let args: serde_json::Value = serde_json::from_str(&args_str).unwrap();
        assert!(args["directive"]
            .as_str()
            .unwrap()
            .contains("**OBJECTIVE**: Generate a Unified Technical Specification."));
        assert_eq!(args["agent_id"], 2);
        assert_eq!(args["agent_name"], "tadpole_alpha");
    }

    #[test]
    fn test_extract_code_tool_call_markdown_fence() {
        let text = r#"
```tool_call
spawn_subagent(
    target_agent_id="coder",
    message="Implement feature",
    priority=1
)
```
"#;
        let res = extract_tool_call(text);
        assert!(res.is_some());
        let (name, args_str, _) = res.unwrap();
        assert_eq!(name, "spawn_subagent");

        let args: serde_json::Value = serde_json::from_str(&args_str).unwrap();
        assert_eq!(args["target_agent_id"], "coder");
        assert_eq!(args["message"], "Implement feature");
        assert_eq!(args["priority"], 1);
    }
}

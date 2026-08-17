//! @docs ARCHITECTURE:Agent
//!
//! ### AI Assist Note
//! **Recovery Engine**: Centralizes LLM response repair, balanced-brace JSON parsing,
//! and hallucination stripping utilities.
//!
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: Improper balanced brace termination on extremely corrupted payloads.

use once_cell::sync::Lazy;
use regex::Regex;

/// Regex for locating the start of a tool call tag (Groq/Llama 3 style)
static FUNCTION_START_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?s)<function=([a-zA-Z0-9_-]+)")
        .expect("Static tool-call start parser regex MUST be valid.")
});

/// Extracts a single tool call from a string, using a balanced-brace counter to support nested JSON arguments.
/// Returns Option<(function_name, arguments_json_string, raw_matched_segment_to_strip)>.
pub(crate) fn extract_tool_call(s: &str) -> Option<(String, String, String)> {
    let start_match = FUNCTION_START_REGEX.captures(s)?;
    let name = start_match.get(1)?.as_str().to_string();
    let match_start = start_match.get(0)?.start();
    let start_search_idx = start_match.get(0)?.end();

    // Find the first '{' after the tag start
    let brace_start = s[start_search_idx..].find('{')? + start_search_idx;

    let mut brace_count = 0;
    let mut brace_end = None;
    for (i, c) in s[brace_start..].char_indices() {
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

#[cfg(test)]
mod tests {
    use super::*;

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
}

// Metadata: [recovery]

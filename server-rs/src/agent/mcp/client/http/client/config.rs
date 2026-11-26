//! @docs ARCHITECTURE:Registry:Mcp
//! ### AI Assist Note
//! HTTP client timeout configuration, bracketed IPv6 authority normalization, and header validation.
//! ### 🔍 Debugging & Observability
//! Enforces RFC 3986 bracketed authority formatting and validates transport-owned header invariants.

use crate::error::AppError;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClientTimeouts {
    pub discovery: std::time::Duration,
    pub tool: std::time::Duration,
}

impl ClientTimeouts {
    pub fn new(discovery: std::time::Duration, tool: std::time::Duration) -> Self {
        Self { discovery, tool }
    }
}

pub fn validate_and_format_authority(parsed_url: &reqwest::Url) -> String {
    let raw_host = parsed_url.host_str().unwrap_or("");
    let formatted_host = if raw_host.contains(':') && !raw_host.starts_with('[') {
        format!("[{raw_host}]")
    } else {
        raw_host.to_string()
    };
    if let Some(port) = parsed_url.port() {
        format!("{formatted_host}:{port}")
    } else {
        formatted_host
    }
}

pub fn build_initial_headers(
    headers: Option<&HashMap<String, String>>,
    server_name: &str,
) -> Result<reqwest::header::HeaderMap, AppError> {
    let mut header_map = reqwest::header::HeaderMap::new();
    header_map.insert(
        reqwest::header::ACCEPT,
        reqwest::header::HeaderValue::from_static("application/json, text/event-stream"),
    );
    header_map.insert(
        reqwest::header::CONTENT_TYPE,
        reqwest::header::HeaderValue::from_static("application/json"),
    );

    let Some(custom_headers) = headers else {
        return Ok(header_map);
    };

    for (key, val) in custom_headers {
        let k_lower = key.to_ascii_lowercase();
        if matches!(
            k_lower.as_str(),
            "host"
                | "content-type"
                | "accept"
                | "mcp-protocol-version"
                | "mcp-method"
                | "mcp-name"
                | "origin"
                | "mcp-session-id"
                | "last-event-id"
        ) || k_lower.starts_with("mcp-param-")
        {
            return Err(AppError::BadRequest(format!(
                "MCP server '{server_name}' may not configure transport-owned header '{key}'"
            )));
        }

        let name = reqwest::header::HeaderName::from_bytes(key.as_bytes()).map_err(|e| {
            AppError::BadRequest(format!(
                "Invalid header name '{key}' for MCP server '{server_name}': {e}"
            ))
        })?;
        let val_parsed = reqwest::header::HeaderValue::from_str(val).map_err(|e| {
            AppError::BadRequest(format!(
                "Invalid header value for '{key}' on MCP server '{server_name}': {e}"
            ))
        })?;
        header_map.insert(name, val_parsed);
    }

    Ok(header_map)
}

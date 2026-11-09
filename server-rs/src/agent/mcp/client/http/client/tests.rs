//! @docs ARCHITECTURE:Registry:Mcp
//! ### AI Assist Note
//! Unit tests for MCP HTTP client header management, bracketed host authorities, bounded eviction, and redaction.
//! ### 🔍 Debugging & Observability
//! Verifies fail-closed header validation, token leak prevention, and 1,024 operation binding eviction.

use super::*;
use crate::agent::mcp::client::http::MAX_OPERATION_BINDINGS;
use serde_json::json;
use std::collections::HashMap;

#[test]
fn test_mcp_http_client_creation() {
    let client = McpHttpClient::new(
        "remote-docs",
        "http://127.0.0.1:3000/mcp",
        None,
        Some(MCP_PROTOCOL_2026_07_28),
    )
    .unwrap();

    assert_eq!(client.server_name, "remote-docs");
    assert_eq!(client.url, "http://127.0.0.1:3000/mcp");
    assert_eq!(client.host_authority, "127.0.0.1:3000");
}

#[test]
fn test_mcp_http_client_header_population() {
    let mut headers = HashMap::new();
    headers.insert("Authorization".to_string(), "Bearer test-key".to_string());
    headers.insert("X-Custom-Header".to_string(), "CustomValue".to_string());

    let client = McpHttpClient::new(
        "remote-docs",
        "http://127.0.0.1:3000/mcp",
        Some(&headers),
        Some(MCP_PROTOCOL_2026_07_28),
    )
    .unwrap();

    assert_eq!(
        client
            .headers
            .get("Authorization")
            .and_then(|v| v.to_str().ok()),
        Some("Bearer test-key")
    );
    assert_eq!(
        client
            .headers
            .get("X-Custom-Header")
            .and_then(|v| v.to_str().ok()),
        Some("CustomValue")
    );
    assert_eq!(
        client
            .headers
            .get(reqwest::header::ACCEPT)
            .and_then(|v| v.to_str().ok()),
        Some("application/json, text/event-stream")
    );
    assert_eq!(
        client
            .headers
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok()),
        Some("application/json")
    );
}

#[test]
fn test_mcp_http_client_rejects_transport_owned_headers() {
    for name in [
        "Host",
        "Content-Type",
        "Accept",
        "MCP-Protocol-Version",
        "Mcp-Method",
        "Mcp-Name",
        "Mcp-Param-Test",
        "Origin",
        "Mcp-Session-Id",
        "Last-Event-ID",
    ] {
        let headers = HashMap::from([(name.to_string(), "unsafe".to_string())]);
        assert!(McpHttpClient::new(
            "remote-docs",
            "http://127.0.0.1:3000/mcp",
            Some(&headers),
            Some(MCP_PROTOCOL_2026_07_28),
        )
        .is_err());
    }
}

#[test]
fn test_mcp_http_client_rejects_invalid_header_name() {
    let mut headers = HashMap::new();
    headers.insert("Invalid Header Name!".to_string(), "value".to_string());

    let err = McpHttpClient::new(
        "bad-header",
        "https://mcp.example.com/api",
        Some(&headers),
        None,
    )
    .unwrap_err();

    assert!(matches!(err, AppError::BadRequest(_)));
}

#[test]
fn test_ipv6_host_authority_is_bracketed() {
    let client_port = McpHttpClient::new("ipv6-test", "http://[::1]:3000/mcp", None, None).unwrap();
    assert_eq!(client_port.host_authority, "[::1]:3000");

    let client_no_port = McpHttpClient::new("ipv6-test", "http://[::1]/mcp", None, None).unwrap();
    assert_eq!(client_no_port.host_authority, "[::1]");

    let client_v4 =
        McpHttpClient::new("ipv4-test", "http://127.0.0.1:3000/mcp", None, None).unwrap();
    assert_eq!(client_v4.host_authority, "127.0.0.1:3000");
}

#[test]
fn test_operation_binding_eviction_is_bounded() {
    let mut client =
        McpHttpClient::new("bound-test", "http://127.0.0.1:3000/mcp", None, None).unwrap();

    // Exercise production path directly through remember_operation
    for i in 0..(MAX_OPERATION_BINDINGS + 10) {
        let op_id = format!("00000000-0000-4000-8000-{:012x}", i);
        assert!(client
            .remember_operation(&op_id, "tool", &json!({"i": i}))
            .is_ok());
    }
    assert_eq!(client.operation_bindings.len(), MAX_OPERATION_BINDINGS);
}

#[test]
fn test_sanitize_raw_error_redacts_tokens() {
    let headers = HashMap::from([(
        "Authorization".to_string(),
        "Bearer super-secret-token".to_string(),
    )]);
    let client = McpHttpClient::new(
        "sec-test",
        "http://127.0.0.1:3000/mcp",
        Some(&headers),
        None,
    )
    .unwrap();
    let raw = crate::agent::mcp::client::jsonrpc::JsonRpcError {
        code: -32001,
        message: "invalid request for Bearer super-secret-token".to_string(),
        data: Some(json!({
            "echo": "super-secret-token",
            "nested": ["Bearer super-secret-token"]
        })),
    };
    let sanitized = client.sanitize_raw_error(&raw);
    assert!(!sanitized.message.contains("super-secret-token"));
    assert!(sanitized.message.contains("[REDACTED]"));
    let data_str = serde_json::to_string(&sanitized.data).unwrap();
    assert!(!data_str.contains("super-secret-token"));
    assert!(data_str.contains("[REDACTED]"));
}

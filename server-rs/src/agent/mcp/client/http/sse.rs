//! @docs ARCHITECTURE:Registry:Mcp
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Agent Runner / MCP HTTP SSE Parser
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Request-scoped SSE parsing only; server-initiated requests prohibited.
//! - `[Structural]` Stream line, event, notification, and byte limits strictly enforced.
//! - `[Structural]` No dependencies on McpHttpClient; pure stream and byte processor.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: `AppError::InfrastructureError`
//! - **Telemetry Targets**: none declared
//! - **Witness Tests**: `agent::mcp::client::http::sse::tests::*`

use crate::agent::mcp::client::jsonrpc::JsonRpcResponse;
use crate::error::{AppError, InfrastructureErrorKind, ProviderId};
use serde_json::Value;
use tracing::debug;

use super::limits::{
    MAX_RESPONSE_BODY_BYTES, MAX_SSE_EVENTS, MAX_SSE_LINE_BYTES, MAX_SSE_NOTIFICATIONS,
};

pub fn sse_protocol_error(server_name: &str, detail: &str) -> AppError {
    AppError::InfrastructureError {
        provider_id: ProviderId::Mcp,
        kind: InfrastructureErrorKind::ApiError,
        detail: format!("MCP SSE protocol error from '{}': {}", server_name, detail),
        help_link: None,
    }
}

pub fn parse_sse_event(
    payload: &str,
    server_name: &str,
    target_id: &Value,
) -> Result<Option<JsonRpcResponse>, AppError> {
    let value: Value = serde_json::from_str(payload)
        .map_err(|_| sse_protocol_error(server_name, "malformed SSE JSON-RPC event"))?;
    if value.get("jsonrpc").and_then(Value::as_str) != Some("2.0") {
        return Err(sse_protocol_error(
            server_name,
            "SSE event has invalid JSON-RPC version",
        ));
    }
    let has_method = value.get("method").is_some();
    let has_id = value.get("id").is_some();
    if has_method && has_id {
        return Err(sse_protocol_error(
            server_name,
            "independent server-to-client request is prohibited",
        ));
    }
    if has_method {
        if !value.get("method").is_some_and(Value::is_string) {
            return Err(sse_protocol_error(
                server_name,
                "SSE notification method is not a string",
            ));
        }
        debug!(
            "[MCP SSE] Notification received for request id={:?}",
            target_id
        );
        return Ok(None);
    }
    if value.get("id") != Some(target_id) {
        return Err(sse_protocol_error(
            server_name,
            "SSE response id does not match the request",
        ));
    }
    serde_json::from_value(value)
        .map(Some)
        .map_err(|_| sse_protocol_error(server_name, "invalid SSE JSON-RPC response"))
}

pub fn process_sse_line(
    line_bytes: &[u8],
    server_name: &str,
    current_data: &mut String,
    event_count: &mut usize,
    notification_count: &mut usize,
    target_id: &Value,
) -> Result<Option<JsonRpcResponse>, AppError> {
    if line_bytes.len() > MAX_SSE_LINE_BYTES {
        return Err(sse_protocol_error(server_name, "SSE line limit exceeded"));
    }
    let line = std::str::from_utf8(line_bytes)
        .map_err(|_| sse_protocol_error(server_name, "SSE line is not UTF-8"))?;
    let trimmed = line.trim_end_matches(&['\r', '\n'][..]);

    if trimmed.is_empty() {
        if !current_data.is_empty() {
            *event_count = event_count.saturating_add(1);
            if *event_count > MAX_SSE_EVENTS {
                return Err(sse_protocol_error(server_name, "SSE event limit exceeded"));
            }
            let payload = std::mem::take(current_data);
            match parse_sse_event(&payload, server_name, target_id)? {
                Some(response) => return Ok(Some(response)),
                None => {
                    *notification_count = notification_count.saturating_add(1);
                    if *notification_count > MAX_SSE_NOTIFICATIONS {
                        return Err(sse_protocol_error(
                            server_name,
                            "SSE notification limit exceeded",
                        ));
                    }
                }
            }
        }
    } else if trimmed.starts_with(':') {
        // SSE comment line - ignore
    } else if let Some(data) = trimmed.strip_prefix("data:") {
        let data_part = data.strip_prefix(' ').unwrap_or(data);
        if !current_data.is_empty() {
            current_data.push('\n');
        }
        current_data.push_str(data_part);
        if current_data.len() > MAX_RESPONSE_BODY_BYTES {
            return Err(sse_protocol_error(server_name, "SSE event limit exceeded"));
        }
    }

    Ok(None)
}

pub async fn parse_sse_stream(
    mut response: reqwest::Response,
    server_name: &str,
    target_id: &Value,
) -> Result<JsonRpcResponse, AppError> {
    let mut buffer = Vec::new();
    let mut current_data = String::new();
    let mut total_bytes = 0usize;
    let mut event_count = 0usize;
    let mut notification_count = 0usize;

    while let Some(chunk) = response
        .chunk()
        .await
        .map_err(|e| AppError::InfrastructureError {
            provider_id: ProviderId::Mcp,
            kind: InfrastructureErrorKind::NetworkError,
            detail: format!("Error reading SSE stream from '{}': {}", server_name, e),
            help_link: None,
        })?
    {
        total_bytes = total_bytes.saturating_add(chunk.len());
        if total_bytes > MAX_RESPONSE_BODY_BYTES {
            return Err(sse_protocol_error(
                server_name,
                "response body limit exceeded",
            ));
        }
        buffer.extend_from_slice(&chunk);

        while let Some(pos) = buffer.windows(1).position(|w| w == b"\n") {
            let line_bytes: Vec<u8> = buffer.drain(..=pos).collect();
            if let Some(response) = process_sse_line(
                &line_bytes,
                server_name,
                &mut current_data,
                &mut event_count,
                &mut notification_count,
                target_id,
            )? {
                return Ok(response);
            }
        }
        if buffer.len() > MAX_SSE_LINE_BYTES {
            return Err(sse_protocol_error(server_name, "SSE line limit exceeded"));
        }
    }

    // Process leftover buffer without trailing newline upon stream termination
    if !buffer.is_empty() {
        let leftover = std::mem::take(&mut buffer);
        if let Some(response) = process_sse_line(
            &leftover,
            server_name,
            &mut current_data,
            &mut event_count,
            &mut notification_count,
            target_id,
        )? {
            return Ok(response);
        }
    }

    if !current_data.is_empty() {
        if let Some(response) = parse_sse_event(&current_data, server_name, target_id)? {
            return Ok(response);
        }
    }

    Err(AppError::InfrastructureError {
        provider_id: ProviderId::Mcp,
        kind: InfrastructureErrorKind::ApiError,
        detail: format!(
            "SSE stream closed without final response for id={:?} from '{}'",
            target_id, server_name
        ),
        help_link: None,
    })
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_process_sse_line_handles_comments_and_data() {
        let mut current_data = String::new();
        let mut event_count = 0;
        let mut notification_count = 0;
        let target_id = json!(1);

        // Comment line ignored
        let res = process_sse_line(
            b": keepalive\n",
            "test-server",
            &mut current_data,
            &mut event_count,
            &mut notification_count,
            &target_id,
        )
        .unwrap();
        assert!(res.is_none());
        assert!(current_data.is_empty());

        // Data line buffered
        let res = process_sse_line(
            b"data: {\"jsonrpc\":\"2.0\",\"id\":1,\"result\":{\"ok\":true}}\n",
            "test-server",
            &mut current_data,
            &mut event_count,
            &mut notification_count,
            &target_id,
        )
        .unwrap();
        assert!(res.is_none());
        assert!(!current_data.is_empty());

        // Empty line dispatches
        let res = process_sse_line(
            b"\n",
            "test-server",
            &mut current_data,
            &mut event_count,
            &mut notification_count,
            &target_id,
        )
        .unwrap();
        assert!(res.is_some());
        assert_eq!(res.unwrap().result.unwrap()["ok"], true);
    }

    #[test]
    fn test_process_sse_line_rejects_oversized_lines() {
        let mut current_data = String::new();
        let mut event_count = 0;
        let mut notification_count = 0;
        let target_id = json!(1);

        let oversized = vec![b'x'; MAX_SSE_LINE_BYTES + 1];
        let err = process_sse_line(
            &oversized,
            "test-server",
            &mut current_data,
            &mut event_count,
            &mut notification_count,
            &target_id,
        );
        assert!(err.is_err());
    }

    #[test]
    fn test_process_sse_line_rejects_invalid_utf8() {
        let mut current_data = String::new();
        let mut event_count = 0;
        let mut notification_count = 0;
        let target_id = json!(1);

        let bad_utf8 = vec![0xFF, 0xFE, b'\n'];
        let err = process_sse_line(
            &bad_utf8,
            "test-server",
            &mut current_data,
            &mut event_count,
            &mut notification_count,
            &target_id,
        );
        assert!(err.is_err());
    }

    #[test]
    fn test_parse_sse_event_validations() {
        let target_id = json!(42);

        // Wrong ID rejected
        let payload = json!({"jsonrpc": "2.0", "id": 99, "result": {}}).to_string();
        assert!(parse_sse_event(&payload, "test", &target_id).is_err());

        // Independent server-to-client request rejected
        let request_payload = json!({"jsonrpc": "2.0", "id": 1, "method": "ping"}).to_string();
        assert!(parse_sse_event(&request_payload, "test", &target_id).is_err());

        // Notification returns Ok(None)
        let notif_payload = json!({"jsonrpc": "2.0", "method": "progress"}).to_string();
        assert!(parse_sse_event(&notif_payload, "test", &target_id)
            .unwrap()
            .is_none());
    }
}

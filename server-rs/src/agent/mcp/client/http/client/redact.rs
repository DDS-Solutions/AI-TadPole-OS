//! @docs ARCHITECTURE:Registry:Mcp
//! ### AI Assist Note
//! Redaction and sanitization of authorization tokens and credentials from MCP error payloads.
//! ### 🔍 Debugging & Observability
//! Disarms recursive token leaks up to 32 depth levels.

use crate::agent::mcp::client::jsonrpc::{JsonRpcError, JsonRpcResponse};
use serde_json::Value;

const MAX_REDACTION_DEPTH: usize = 32;

pub fn redact_json_value_bounded(value: &mut Value, secret: &str, depth: usize) {
    if depth > MAX_REDACTION_DEPTH {
        return;
    }
    match value {
        Value::String(s) => {
            if s.contains(secret) {
                *s = s.replace(secret, "[REDACTED]");
            }
        }
        Value::Array(arr) => {
            for item in arr {
                redact_json_value_bounded(item, secret, depth + 1);
            }
        }
        Value::Object(obj) => {
            for (_, val) in obj.iter_mut() {
                redact_json_value_bounded(val, secret, depth + 1);
            }
        }
        _ => {}
    }
}

pub fn redact_json_value(value: &mut Value, secret: &str) {
    redact_json_value_bounded(value, secret, 0);
}

pub fn sanitize_raw_error(error: &JsonRpcError, auth_header: Option<&str>) -> JsonRpcError {
    let mut sanitized = error.clone();
    let Some(secret) = auth_header else {
        return sanitized;
    };

    sanitized.message = sanitized.message.replace(secret, "[REDACTED]");
    if let Some(token) = secret.strip_prefix("Bearer ") {
        sanitized.message = sanitized.message.replace(token, "[REDACTED]");
    }
    if let Some(ref mut data) = sanitized.data {
        redact_json_value(data, secret);
        if let Some(token) = secret.strip_prefix("Bearer ") {
            redact_json_value(data, token);
        }
    }
    sanitized
}

pub fn safe_rpc_error(error: &JsonRpcError, auth_header: Option<&str>) -> String {
    let sanitized = sanitize_raw_error(error, auth_header);
    format!("Code {}: {}", sanitized.code, sanitized.message)
}

pub fn extract_rpc_error_from_body(body: &[u8], expected_id: &Value) -> Option<JsonRpcError> {
    let json_resp = serde_json::from_slice::<JsonRpcResponse>(body).ok()?;
    json_resp.validate_for(expected_id).ok()?;
    json_resp.error
}

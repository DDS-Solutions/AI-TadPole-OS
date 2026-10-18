//! @docs ARCHITECTURE:Registry:Mcp
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Agent Runner / MCP JSON-RPC
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Strict conformance to JSON-RPC 2.0 protocol representations.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: none declared
//! - **Witness Tests**: `client::jsonrpc::tests::*`

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

// JSON-RPC standard error codes
pub const JSONRPC_INVALID_REQUEST: i64 = -32600;
pub const JSONRPC_METHOD_NOT_FOUND: i64 = -32601;
pub const JSONRPC_INVALID_PARAMS: i64 = -32602;
pub const JSONRPC_INTERNAL_ERROR: i64 = -32603;
pub const JSONRPC_HEADER_MISMATCH: i64 = -32020;
pub const JSONRPC_UNSUPPORTED_PROTOCOL_VERSION: i64 = -32022;

#[derive(Debug, Serialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub id: Value,
    pub method: String,
    pub params: Value,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    pub id: Option<Value>,
    pub result: Option<Value>,
    pub error: Option<JsonRpcError>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct JsonRpcError {
    pub code: i64,
    pub message: String,
    #[serde(default)]
    pub data: Option<Value>,
}

impl JsonRpcError {
    pub fn is_unsupported_protocol_version(&self) -> bool {
        self.code == JSONRPC_UNSUPPORTED_PROTOCOL_VERSION
    }

    pub fn supported_versions(&self) -> Vec<String> {
        if let Some(ref data) = self.data {
            if let Some(versions) = data.get("supported").and_then(|v| v.as_array()) {
                return versions
                    .iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect();
            }
        }
        Vec::new()
    }

    pub fn requested_version(&self) -> Option<String> {
        self.data
            .as_ref()
            .and_then(|d| d.get("requested"))
            .and_then(|v| v.as_str().map(|s| s.to_string()))
    }
}

impl std::fmt::Display for JsonRpcError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Structured error data is deliberately retained on the value but omitted from
        // Display so bearer tokens or tool arguments echoed by a peer cannot reach logs.
        write!(f, "Code {}: {}", self.code, self.message)
    }
}

impl JsonRpcResponse {
    /// Validate the response envelope before exposing either the result or error.
    pub fn validate_for(&self, expected_id: &Value) -> Result<(), String> {
        if self.jsonrpc != "2.0" {
            return Err(format!(
                "invalid JSON-RPC version '{}'; expected '2.0'",
                self.jsonrpc
            ));
        }
        if self.id.as_ref() != Some(expected_id) {
            return Err("JSON-RPC response id does not match the request".to_string());
        }
        if self.result.is_some() == self.error.is_some() {
            return Err(
                "JSON-RPC response must contain exactly one of result or error".to_string(),
            );
        }
        Ok(())
    }
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct JsonRpcNotification {
    pub jsonrpc: String,
    pub method: String,
    pub params: Option<Value>,
}

pub const MCP_CLIENT_NAME: &str = "ai-tadpole-os";
pub const MCP_CLIENT_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Helper building the required namespaced `_meta` object per the 2026-07-28 specification.
pub fn make_meta(protocol_version: &str) -> Value {
    json!({
        "io.modelcontextprotocol/protocolVersion": protocol_version,
        "io.modelcontextprotocol/clientInfo": {
            "name": MCP_CLIENT_NAME,
            "version": MCP_CLIENT_VERSION
        },
        "io.modelcontextprotocol/clientCapabilities": {}
    })
}

/// Helper building `_meta` for tool calls including the governed `operation_id`.
pub fn make_tool_call_meta(protocol_version: &str, operation_id: Option<&str>) -> Value {
    let mut meta = make_meta(protocol_version);
    if let Some(op_id) = operation_id {
        if let Some(obj) = meta.as_object_mut() {
            obj.insert("operation_id".to_string(), json!(op_id));
        }
    }
    meta
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_make_meta_keys_conform_to_spec() {
        let meta = make_meta("2026-07-28");
        assert_eq!(
            meta.get("io.modelcontextprotocol/protocolVersion")
                .and_then(|v| v.as_str()),
            Some("2026-07-28")
        );
        let client_info = meta.get("io.modelcontextprotocol/clientInfo").unwrap();
        assert_eq!(
            client_info.get("name").and_then(|v| v.as_str()),
            Some("ai-tadpole-os")
        );
        assert!(meta
            .get("io.modelcontextprotocol/clientCapabilities")
            .is_some());
        assert!(meta.get("protocolVersion").is_none());
    }

    #[test]
    fn test_make_tool_call_meta_includes_operation_id() {
        let meta = make_tool_call_meta("2026-07-28", Some("op-1234-5678"));
        assert_eq!(
            meta.get("operation_id").and_then(|v| v.as_str()),
            Some("op-1234-5678")
        );
    }

    #[test]
    fn test_jsonrpc_error_inspection() {
        let err = JsonRpcError {
            code: JSONRPC_UNSUPPORTED_PROTOCOL_VERSION,
            message: "Unsupported protocol version".to_string(),
            data: Some(json!({
                "requested": "2025-11-25",
                "supported": ["2026-07-28"]
            })),
        };

        assert!(err.is_unsupported_protocol_version());
        assert_eq!(err.supported_versions(), vec!["2026-07-28"]);
        assert_eq!(err.requested_version().as_deref(), Some("2025-11-25"));
        assert!(!err.to_string().contains("2025-11-25"));
    }

    #[test]
    fn validates_jsonrpc_response_identity_and_exclusive_payload() {
        let good: JsonRpcResponse = serde_json::from_value(json!({
            "jsonrpc": "2.0", "id": 7, "result": {"ok": true}
        }))
        .unwrap();
        assert!(good.validate_for(&json!(7)).is_ok());

        let wrong_id: JsonRpcResponse = serde_json::from_value(json!({
            "jsonrpc": "2.0", "id": 8, "result": {}
        }))
        .unwrap();
        assert!(wrong_id.validate_for(&json!(7)).is_err());

        let both: JsonRpcResponse = serde_json::from_value(json!({
            "jsonrpc": "2.0", "id": 7, "result": {},
            "error": {"code": -32600, "message": "bad"}
        }))
        .unwrap();
        assert!(both.validate_for(&json!(7)).is_err());
    }
}

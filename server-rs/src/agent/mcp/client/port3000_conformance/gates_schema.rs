//! @docs ARCHITECTURE:Registry:Mcp
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Agent Runner / MCP Port 3000 Conformance - Schema Gates
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Exact JSON Schema validation rules for `x-mcp-header` parameters.
//! - `[Structural]` RFC 2047-style safe encoding for arbitrary / non-ASCII header values.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: schema validation errors on invalid `x-mcp-header` declarations.
//! - **Telemetry Targets**: none (pure in-memory validation).
//! - **Witness Tests**: `port3000_conformance::tests::gates_schema::test_gate_*`

use crate::agent::mcp::client::http::{
    encode_header_value_if_needed, extract_and_validate_tool_headers,
};
use base64::Engine;
use serde_json::json;

// ---------------------------------------------------------------------------
// Gate 12: Invalid x-mcp-header definitions excluded; valid values safely encoded
// ---------------------------------------------------------------------------
#[test]
fn test_gate_12_x_mcp_header_extraction_and_safe_encoding() {
    // Safe encoding for non-ASCII
    assert_eq!(
        encode_header_value_if_needed("utf8-💥"),
        format!(
            "=?base64?{}?=",
            base64::engine::general_purpose::STANDARD.encode("utf8-💥")
        )
    );

    // Safe encoding for leading/trailing whitespace
    assert!(encode_header_value_if_needed(" leading_space").starts_with("=?base64?"));

    // Valid schema
    let valid_schema = json!({
        "type": "object",
        "properties": {
            "scene_id": {
                "type": "string",
                "x-mcp-header": "Scene-Id"
            }
        }
    });
    let res = extract_and_validate_tool_headers("load_scene", &valid_schema).unwrap();
    assert_eq!(res.get("scene_id"), Some(&"Scene-Id".to_string()));

    // Invalid schema: $ref reference
    let invalid_ref = json!({
        "type": "object",
        "properties": {
            "bad_ref": {
                "$ref": "#/definitions/Scene",
                "x-mcp-header": "Scene-Ref"
            }
        }
    });
    assert!(extract_and_validate_tool_headers("bad_tool", &invalid_ref).is_err());
}

// ---------------------------------------------------------------------------
// Gate 15: Complete x-mcp-header schema rules
// ---------------------------------------------------------------------------
#[test]
fn test_gate_15_complete_x_mcp_header_schema_rules() {
    let nested = json!({
        "type": "object",
        "properties": {
            "scene": {
                "type": "object",
                "properties": {
                    "id": {"type": "string", "x-mcp-header": "Scene-Id"},
                    "enabled": {"type": "boolean", "x-mcp-header": "Scene-Enabled"},
                    "count": {"type": "integer", "x-mcp-header": "Scene-Count"}
                }
            }
        }
    });
    let bindings = extract_and_validate_tool_headers("nested", &nested).unwrap();
    assert_eq!(
        bindings.get("scene.id").map(String::as_str),
        Some("Scene-Id")
    );
    assert_eq!(
        bindings.get("scene.enabled").map(String::as_str),
        Some("Scene-Enabled")
    );

    for invalid in [
        json!({"type":"object","properties":{"v":{"type":"number","x-mcp-header":"Float"}}}),
        json!({"type":"object","properties":{"v":{"type":"string","if":{},"x-mcp-header":"Conditional"}}}),
        json!({"type":"object","properties":{"v":{"type":"string","not":{},"x-mcp-header":"Not"}}}),
        json!({"type":"object","properties":{"v":{"type":"array","items":{"type":"string","x-mcp-header":"Array"}}}}),
        json!({"type":"object","properties":{"v":{"allOf":[{"type":"string","x-mcp-header":"Composed"}]}}}),
        json!({"type":"object","patternProperties":{".*":{"type":"string","x-mcp-header":"Dynamic"}}}),
    ] {
        assert!(extract_and_validate_tool_headers("invalid", &invalid).is_err());
    }

    assert!(encode_header_value_if_needed("=?sentinel?=").starts_with("=?base64?"));
    assert_eq!(
        encode_header_value_if_needed("middle=?value"),
        "middle=?value"
    );
}

//! @docs ARCHITECTURE:Registry:Mcp
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Agent Runner / MCP HTTP Header & Schema Utilities
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Zero reqwest dependencies; pure schema and header string manipulation.
//! - `[Structural]` MCP Base64 sentinel `=?base64?...?=` encoding and RFC 9110 token validation.
//! - `[Structural]` SEP-2243 parameter header projection and static schema reachability.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: string error messages for schema walk failures
//! - **Telemetry Targets**: none declared
//! - **Witness Tests**: `agent::mcp::client::http::headers::tests::*`

use base64::Engine;
use serde_json::Value;
use std::collections::{HashMap, HashSet};

use super::limits::MAX_SAFE_INTEGER;

pub fn is_valid_header_token(name: &str) -> bool {
    !name.is_empty()
        && name.bytes().all(|byte| {
            byte.is_ascii_alphanumeric()
                || matches!(
                    byte,
                    b'!' | b'#'
                        | b'$'
                        | b'%'
                        | b'&'
                        | b'\''
                        | b'*'
                        | b'+'
                        | b'-'
                        | b'.'
                        | b'^'
                        | b'_'
                        | b'`'
                        | b'|'
                        | b'~'
                )
        })
}

pub fn encode_header_value_if_needed(value: &str) -> String {
    let needs_encoding = value
        .chars()
        .any(|c| c.is_control() || !c.is_ascii() || c == '\r' || c == '\n')
        || value.chars().next().is_some_and(char::is_whitespace)
        || value.chars().next_back().is_some_and(char::is_whitespace)
        || (value.starts_with("=?") && value.ends_with("?="));

    if needs_encoding {
        let b64 = base64::engine::general_purpose::STANDARD.encode(value.as_bytes());
        format!("=?base64?{}?=", b64)
    } else {
        value.to_string()
    }
}

pub fn extract_and_validate_tool_headers(
    tool_name: &str,
    schema: &Value,
) -> Result<HashMap<String, String>, String> {
    fn walk(
        tool_name: &str,
        node: &Value,
        path: &[String],
        forbidden_context: bool,
        bindings: &mut HashMap<String, String>,
        headers: &mut HashSet<String>,
    ) -> Result<(), String> {
        let Some(object) = node.as_object() else {
            return Ok(());
        };
        let locally_forbidden = forbidden_context
            || [
                "$ref",
                "allOf",
                "anyOf",
                "oneOf",
                "not",
                "if",
                "then",
                "else",
                "items",
                "contains",
                "prefixItems",
                "patternProperties",
                "propertyNames",
                "unevaluatedProperties",
                "unevaluatedItems",
            ]
            .iter()
            .any(|key| object.contains_key(*key));

        if let Some(annotation) = object.get("x-mcp-header") {
            let property_path = path.join(".");
            if path.is_empty() || locally_forbidden {
                return Err(format!(
                    "x-mcp-header on '{}' in tool '{}' is not statically reachable",
                    property_path, tool_name
                ));
            }
            let header = annotation
                .as_str()
                .ok_or_else(|| format!("x-mcp-header on '{}' must be a string", property_path))?;
            if !is_valid_header_token(header) {
                return Err(format!("x-mcp-header '{}' is not a valid token", header));
            }
            let property_type = object.get("type").and_then(Value::as_str).unwrap_or("");
            if !matches!(property_type, "string" | "integer" | "boolean") {
                return Err(format!(
                    "x-mcp-header on '{}' has unsupported type '{}'",
                    property_path, property_type
                ));
            }
            if !headers.insert(header.to_ascii_lowercase()) {
                return Err(format!(
                    "duplicate x-mcp-header '{}' in tool '{}'",
                    header, tool_name
                ));
            }
            bindings.insert(property_path, header.to_string());
        }

        if let Some(properties) = object.get("properties").and_then(Value::as_object) {
            for (name, property) in properties {
                let mut child_path = path.to_vec();
                child_path.push(name.clone());
                walk(
                    tool_name,
                    property,
                    &child_path,
                    locally_forbidden,
                    bindings,
                    headers,
                )?;
            }
        }

        for key in [
            "$defs",
            "definitions",
            "allOf",
            "anyOf",
            "oneOf",
            "not",
            "if",
            "then",
            "else",
            "items",
            "contains",
            "prefixItems",
            "dependentSchemas",
            "additionalProperties",
            "patternProperties",
            "propertyNames",
            "unevaluatedProperties",
            "unevaluatedItems",
        ] {
            if let Some(child) = object.get(key) {
                match child {
                    Value::Array(values) => {
                        for value in values {
                            walk(tool_name, value, path, true, bindings, headers)?;
                        }
                    }
                    Value::Object(values)
                        if matches!(
                            key,
                            "$defs" | "definitions" | "patternProperties" | "dependentSchemas"
                        ) =>
                    {
                        for value in values.values() {
                            walk(tool_name, value, path, true, bindings, headers)?;
                        }
                    }
                    _ => walk(tool_name, child, path, true, bindings, headers)?,
                }
            }
        }
        Ok(())
    }

    let mut bindings = HashMap::new();
    let mut headers = HashSet::new();
    walk(tool_name, schema, &[], false, &mut bindings, &mut headers)?;
    Ok(bindings)
}

pub fn value_at_property_path<'a>(value: &'a Value, path: &str) -> Option<&'a Value> {
    path.split('.')
        .try_fold(value, |current, part| current.as_object()?.get(part))
}

pub fn primitive_header_value(value: &Value) -> Option<String> {
    if let Some(string) = value.as_str() {
        return Some(string.to_string());
    }
    if let Some(boolean) = value.as_bool() {
        return Some(boolean.to_string());
    }
    if let Some(integer) = value.as_i64() {
        return ((-MAX_SAFE_INTEGER..=MAX_SAFE_INTEGER).contains(&integer))
            .then(|| integer.to_string());
    }
    if let Some(integer) = value
        .as_u64()
        .filter(|integer| *integer <= MAX_SAFE_INTEGER as u64)
    {
        return Some(integer.to_string());
    }
    if let Some(float) = value.as_f64() {
        if float.is_finite()
            && float.fract() == 0.0
            && float >= (-MAX_SAFE_INTEGER as f64)
            && float <= (MAX_SAFE_INTEGER as f64)
        {
            return Some((float as i64).to_string());
        }
    }
    None
}

pub fn hash_operation_binding(name: &str, arguments: &Value) -> String {
    use sha2::Digest;
    let mut hasher = sha2::Sha256::new();
    hasher.update(name.as_bytes());
    hasher.update(b":");
    hasher.update(arguments.to_string().as_bytes());
    hex::encode(hasher.finalize())
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_mcp_sentinel_base64_header_encoding() {
        let ascii_clean = "simple-token_123";
        assert_eq!(encode_header_value_if_needed(ascii_clean), ascii_clean);

        let with_spaces = "  padded  ";
        let encoded_spaces = encode_header_value_if_needed(with_spaces);
        assert!(encoded_spaces.starts_with("=?base64?"));
        assert!(encoded_spaces.ends_with("?="));

        let non_ascii = "Café_123";
        let encoded_non_ascii = encode_header_value_if_needed(non_ascii);
        assert!(encoded_non_ascii.starts_with("=?base64?"));
        assert!(encoded_non_ascii.ends_with("?="));

        let control_chars = "Line1\nLine2";
        let encoded_ctrl = encode_header_value_if_needed(control_chars);
        assert!(encoded_ctrl.starts_with("=?base64?"));
        assert!(encoded_ctrl.ends_with("?="));

        let raw = base64::engine::general_purpose::STANDARD
            .decode(&encoded_ctrl[9..encoded_ctrl.len() - 2])
            .unwrap();
        assert_eq!(String::from_utf8(raw).unwrap(), control_chars);
    }

    #[test]
    fn test_x_mcp_header_extraction_and_exclusion() {
        let valid_schema = json!({
            "type": "object",
            "properties": {
                "user_id": {
                    "type": "string",
                    "x-mcp-header": "X-User-Id"
                },
                "count": {
                    "type": "integer",
                    "x-mcp-header": "X-Count"
                },
                "nested_obj": {
                    "type": "object",
                    "properties": {
                        "name": { "type": "string" }
                    }
                }
            }
        });

        let extracted = extract_and_validate_tool_headers("valid_tool", &valid_schema).unwrap();
        assert_eq!(extracted.len(), 2);
        assert_eq!(extracted.get("user_id").unwrap(), "X-User-Id");
        assert_eq!(extracted.get("count").unwrap(), "X-Count");

        let invalid_token = json!({
            "type": "object",
            "properties": {
                "param": {
                    "type": "string",
                    "x-mcp-header": "Invalid Token Name!"
                }
            }
        });
        assert!(extract_and_validate_tool_headers("bad_token", &invalid_token).is_err());

        let duplicate_header = json!({
            "type": "object",
            "properties": {
                "p1": {
                    "type": "string",
                    "x-mcp-header": "Auth"
                },
                "p2": {
                    "type": "string",
                    "x-mcp-header": "auth"
                }
            }
        });
        assert!(extract_and_validate_tool_headers("dup_header", &duplicate_header).is_err());
    }

    #[test]
    fn test_header_projection_accepts_only_bounded_json_primitives() {
        assert_eq!(
            primitive_header_value(&json!("scene-1")),
            Some("scene-1".into())
        );
        assert_eq!(primitive_header_value(&json!(true)), Some("true".into()));
        assert_eq!(
            primitive_header_value(&json!(MAX_SAFE_INTEGER)),
            Some(MAX_SAFE_INTEGER.to_string())
        );
        assert_eq!(
            primitive_header_value(&json!(-MAX_SAFE_INTEGER)),
            Some((-MAX_SAFE_INTEGER).to_string())
        );
        assert_eq!(primitive_header_value(&json!(1.0)), Some("1".into()));
        assert_eq!(primitive_header_value(&json!(MAX_SAFE_INTEGER + 1)), None);
        assert_eq!(primitive_header_value(&json!(1.5)), None);
        assert_eq!(primitive_header_value(&Value::Null), None);
        assert_eq!(primitive_header_value(&json!(["unsafe"])), None);
        assert_eq!(primitive_header_value(&json!({"unsafe": true})), None);
    }

    #[test]
    fn test_hash_operation_binding_deterministic() {
        let h1 = hash_operation_binding("toolA", &json!({"a": 1}));
        let h2 = hash_operation_binding("toolA", &json!({"a": 1}));
        let h3 = hash_operation_binding("toolA", &json!({"a": 2}));
        assert_eq!(h1, h2);
        assert_ne!(h1, h3);
        assert_eq!(h1.len(), 64);
    }
}

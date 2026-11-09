//! @docs ARCHITECTURE:Registry:Mcp
//! ### AI Assist Note
//! Validation of MCP discovery responses, tools catalog schema, and execution results.
//! ### 🔍 Debugging & Observability
//! Enforces SEP-2243 quarantine policy and prevents malformed tool schema injection.

use crate::agent::mcp::client::MCP_PROTOCOL_2026_07_28;
use crate::error::{AppError, InfrastructureErrorKind, ProviderId};
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use tracing::warn;

use super::super::headers::{
    encode_header_value_if_needed, extract_and_validate_tool_headers, primitive_header_value,
    value_at_property_path,
};

pub fn build_projected_tool_headers(
    tool_param_headers: &HashMap<String, HashMap<String, String>>,
    tool_name: &str,
    arguments: &Value,
) -> Result<HashMap<String, String>, AppError> {
    let Some(param_map) = tool_param_headers.get(tool_name) else {
        return Ok(HashMap::new());
    };

    let mut extra_headers = HashMap::new();
    for (argument_path, header_name) in param_map {
        let Some(value) = value_at_property_path(arguments, argument_path) else {
            continue;
        };
        if value.is_null() {
            continue;
        }

        let value_string = primitive_header_value(value).ok_or_else(|| {
            AppError::BadRequest(format!(
                "MCP header-bound argument '{argument_path}' must be a string, boolean, or safe integer"
            ))
        })?;
        let encoded = encode_header_value_if_needed(&value_string);
        let header_key = format!("Mcp-Param-{header_name}");

        reqwest::header::HeaderName::from_bytes(header_key.as_bytes()).map_err(|e| {
            AppError::BadRequest(format!("Invalid projected header name '{header_key}': {e}"))
        })?;
        reqwest::header::HeaderValue::from_str(&encoded).map_err(|e| {
            AppError::BadRequest(format!(
                "Invalid projected header value for '{header_key}': {e}"
            ))
        })?;
        extra_headers.insert(header_key, encoded);
    }
    Ok(extra_headers)
}

pub fn validate_tool_call_result(result: &Value, server_name: &str) -> Result<(), AppError> {
    if result.get("resultType").and_then(Value::as_str) != Some("complete") {
        return Err(AppError::InfrastructureError {
            provider_id: ProviderId::Mcp,
            kind: InfrastructureErrorKind::ApiError,
            detail: format!("MCP server '{server_name}' returned invalid tools/call resultType"),
            help_link: None,
        });
    }
    if !result.get("content").is_some_and(Value::is_array) {
        return Err(AppError::InfrastructureError {
            provider_id: ProviderId::Mcp,
            kind: InfrastructureErrorKind::ApiError,
            detail: format!("MCP server '{server_name}' returned tools/call without content array"),
            help_link: None,
        });
    }
    if result
        .get("structuredContent")
        .is_some_and(|v| !v.is_object())
    {
        return Err(AppError::InfrastructureError {
            provider_id: ProviderId::Mcp,
            kind: InfrastructureErrorKind::ApiError,
            detail: format!("MCP server '{server_name}' returned non-object structuredContent"),
            help_link: None,
        });
    }
    if result.get("isError").is_some_and(|v| !v.is_boolean()) {
        return Err(AppError::InfrastructureError {
            provider_id: ProviderId::Mcp,
            kind: InfrastructureErrorKind::ApiError,
            detail: format!("MCP server '{server_name}' returned non-boolean isError"),
            help_link: None,
        });
    }
    Ok(())
}

pub fn validate_discovery_response(result: &Value, server_name: &str) -> Result<Value, AppError> {
    if result.get("resultType").and_then(Value::as_str) != Some("complete") {
        return Err(AppError::InfrastructureError {
            provider_id: ProviderId::Mcp,
            kind: InfrastructureErrorKind::ApiError,
            detail: format!(
                "MCP server '{server_name}' returned invalid server/discover resultType"
            ),
            help_link: None,
        });
    }
    let versions = result
        .get("supportedVersions")
        .and_then(Value::as_array)
        .ok_or_else(|| AppError::InfrastructureError {
            provider_id: ProviderId::Mcp,
            kind: InfrastructureErrorKind::ApiError,
            detail: format!("MCP server '{server_name}' omitted supportedVersions"),
            help_link: None,
        })?;
    if versions.iter().any(|v| !v.is_string())
        || !versions
            .iter()
            .any(|v| v.as_str() == Some(MCP_PROTOCOL_2026_07_28))
    {
        return Err(AppError::InfrastructureError {
            provider_id: ProviderId::Mcp,
            kind: InfrastructureErrorKind::ApiError,
            detail: format!("MCP server '{server_name}' did not offer required protocol version '{MCP_PROTOCOL_2026_07_28}'"),
            help_link: None,
        });
    }
    result
        .get("capabilities")
        .filter(|value| value.is_object())
        .cloned()
        .ok_or_else(|| AppError::InfrastructureError {
            provider_id: ProviderId::Mcp,
            kind: InfrastructureErrorKind::ApiError,
            detail: format!("MCP server '{server_name}' omitted valid capabilities"),
            help_link: None,
        })
}

fn process_single_tool(
    tool: Value,
    server_name: &str,
    tool_param_headers: &mut HashMap<String, HashMap<String, String>>,
    listed_tools: &mut HashSet<String>,
) -> Result<Option<Value>, AppError> {
    let Some(name) = tool.get("name").and_then(|n| n.as_str()) else {
        return Err(AppError::InfrastructureError {
            provider_id: ProviderId::Mcp,
            kind: InfrastructureErrorKind::ApiError,
            detail: format!("MCP server '{server_name}' returned a tool without a name"),
            help_link: None,
        });
    };

    let Some(schema) = tool.get("inputSchema").filter(|s| s.is_object()) else {
        warn!("[MCP HTTP] Excluded tool '{name}' because inputSchema is not an object");
        return Ok(None);
    };

    match extract_and_validate_tool_headers(name, schema) {
        Ok(headers_map) => {
            if !headers_map.is_empty() {
                tool_param_headers.insert(name.to_string(), headers_map);
            }
            listed_tools.insert(name.to_string());
            Ok(Some(tool))
        }
        Err(err_msg) => {
            warn!("[MCP HTTP] Excluded tool '{name}' due to invalid x-mcp-header: {err_msg}");
            Ok(None)
        }
    }
}

pub fn filter_and_quarantine_tools(
    raw_tools: Vec<Value>,
    server_name: &str,
    tool_param_headers: &mut HashMap<String, HashMap<String, String>>,
    listed_tools: &mut HashSet<String>,
) -> Result<Vec<Value>, AppError> {
    tool_param_headers.clear();
    listed_tools.clear();
    let mut valid_tools = Vec::new();
    let mut excluded_count = 0usize;

    for tool in raw_tools {
        match process_single_tool(tool, server_name, tool_param_headers, listed_tools)? {
            Some(valid) => valid_tools.push(valid),
            None => excluded_count += 1,
        }
    }

    if excluded_count > 0 {
        warn!(
            "[MCP HTTP] Catalog update for '{server_name}': {} tools active, {excluded_count} tools quarantined",
            valid_tools.len()
        );
    }

    Ok(valid_tools)
}

//! @docs ARCHITECTURE:Registry
//! 
//! ### AI Assist Note
//! **Core technical resource for the Tadpole OS Sovereign infrastructure.**
//! This module implements high-fidelity logic for the Sovereign Reality layer.
//! 
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: Runtime logic error, state desynchronization, or resource exhaustion.
//! - **Telemetry Link**: Search `[validation]` in tracing logs.

use crate::error::AppError;
use crate::agent::runner::tools::error::ToolExecutionError;

pub(crate) fn extract_path_from_args(
    args: &serde_json::Value,
    workspace_root: &std::path::Path,
) -> Result<Option<std::path::PathBuf>, ToolExecutionError> {
    if let serde_json::Value::Object(map) = args {
        let path_val = map
            .get("path")
            .or_else(|| map.get("file"))
            .or_else(|| map.get("target_path"))
            .or_else(|| map.get("SearchPath"))
            .or_else(|| map.get("DirectoryPath"))
            .or_else(|| map.get("AbsolutePath"));

        if let Some(v) = path_val {
            if let Some(path_str) = v.as_str() {
                let mut p_str = path_str;
                let root_str = workspace_root.to_string_lossy();
                if p_str.starts_with(&*root_str) {
                    p_str = &p_str[root_str.len()..];
                } else {
                    let p_lower = p_str.to_lowercase();
                    let root_lower = root_str.to_lowercase();
                    if p_lower.starts_with(&root_lower) {
                        p_str = &p_str[root_str.len()..];
                    }
                }
                let clean_path = p_str.trim_start_matches('/').trim_start_matches('\\');

                let safe_path = crate::utils::security::validate_path(workspace_root, clean_path)
                    .map_err(|e| {
                        ToolExecutionError::SecurityBlocked(format!(
                            "Path validation failed: {}",
                            e
                        ))
                    })?;
                return Ok(Some(safe_path.to_path_buf()));
            }
        }
    }
    Ok(None)
}

/// Validates dynamic or core tool arguments against their parameters JSON schema (properties and required fields).
pub(crate) fn validate_json_schema(
    schema: &serde_json::Value,
    args: &serde_json::Value,
) -> Result<(), AppError> {
    if schema.is_null() || !schema.is_object() {
        return Ok(());
    }

    let schema_obj = schema.as_object().unwrap();
    let args_obj = args.as_object().ok_or_else(|| {
        AppError::BadRequest("Arguments must be a JSON object".to_string())
    })?;

    // 1. Check required parameters
    if let Some(required) = schema_obj.get("required").and_then(|r| r.as_array()) {
        for req_val in required {
            if let Some(req_name) = req_val.as_str() {
                if !args_obj.contains_key(req_name) || args_obj.get(req_name).unwrap().is_null() {
                    return Err(AppError::BadRequest(format!(
                        "Missing required parameter: '{}'",
                        req_name
                    )));
                }
            }
        }
    }

    // 2. Check properties and types
    if let Some(properties) = schema_obj.get("properties").and_then(|p| p.as_object()) {
        for (prop_name, prop_def) in properties {
            if let Some(val) = args_obj.get(prop_name) {
                if val.is_null() {
                    continue;
                }

                if let Some(expected_type) = prop_def.get("type").and_then(|t| t.as_str()) {
                    match expected_type {
                        "string" => {
                            if !val.is_string() {
                                return Err(AppError::BadRequest(format!(
                                    "Parameter '{}' expects type 'string', got: {}",
                                    prop_name, val
                                )));
                            }
                        }
                        "integer" => {
                            let is_int = val.is_i64()
                                || val.is_u64()
                                || (val.is_number()
                                    && val.as_f64().map_or(false, |f| f.fract() == 0.0));
                            if !is_int {
                                return Err(AppError::BadRequest(format!(
                                    "Parameter '{}' expects type 'integer', got: {}",
                                    prop_name, val
                                )));
                            }
                        }
                        "number" => {
                            if !val.is_number() {
                                return Err(AppError::BadRequest(format!(
                                    "Parameter '{}' expects type 'number', got: {}",
                                    prop_name, val
                                )));
                            }
                        }
                        "boolean" => {
                            if !val.is_boolean() {
                                return Err(AppError::BadRequest(format!(
                                    "Parameter '{}' expects type 'boolean', got: {}",
                                    prop_name, val
                                )));
                            }
                        }
                        "array" => {
                            if !val.is_array() {
                                return Err(AppError::BadRequest(format!(
                                    "Parameter '{}' expects type 'array', got: {}",
                                    prop_name, val
                                )));
                            }
                        }
                        "object" => {
                            if !val.is_object() {
                                return Err(AppError::BadRequest(format!(
                                    "Parameter '{}' expects type 'object', got: {}",
                                    prop_name, val
                                )));
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    Ok(())
}

// Metadata: [validation]

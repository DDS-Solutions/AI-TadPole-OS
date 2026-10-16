//! @docs ARCHITECTURE:Registry
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Agent Runner / validation
//!
//! ### AI Assist Note
//! Resolve tool arguments before authorization and share their representation with execution.
//!
//! ### 🔍 Debugging & Observability
//! - **Telemetry Targets**: none declared

use crate::agent::runner::tools::error::ToolExecutionError;
use crate::error::AppError;

pub(crate) const PATH_ALIASES: &[&str] = &[
    "path",
    "filename",
    "file_name",
    "file",
    "target_path",
    "SearchPath",
    "DirectoryPath",
    "AbsolutePath",
    "dir",
];

pub(crate) fn extract_path_from_args(
    args: &serde_json::Value,
    workspace_root: &std::path::Path,
) -> Result<Option<std::path::PathBuf>, ToolExecutionError> {
    let mut resolved = None;
    for alias in PATH_ALIASES {
        if let Some(value) = args.get(*alias) {
            let raw = value.as_str().filter(|p| !p.is_empty()).ok_or_else(|| {
                ToolExecutionError::SecurityBlocked(format!(
                    "Path alias '{alias}' must be a nonempty string"
                ))
            })?;
            let path = crate::utils::security::validate_path(workspace_root, raw)
                .map_err(|e| {
                    ToolExecutionError::SecurityBlocked(format!("Path validation failed: {e}"))
                })?
                .to_path_buf();
            if resolved.as_ref().is_some_and(|previous| previous != &path) {
                return Err(ToolExecutionError::SecurityBlocked(
                    "Conflicting path aliases".to_string(),
                ));
            }
            resolved = Some(path);
        }
    }
    Ok(resolved)
}

pub(crate) fn extract_shell_command(
    args: &serde_json::Value,
) -> Result<(String, Vec<String>), AppError> {
    if args.get("envs").is_some() {
        return Err(AppError::Forbidden(
            "Caller-supplied process environment is prohibited".to_string(),
        ));
    }
    let (executable, arguments) = match (args.get("executable"), args.get("command")) {
        (Some(_), Some(_)) => {
            return Err(AppError::BadRequest(
                "Use either executable/args or command, not both".to_string(),
            ))
        }
        (Some(exe), None) => {
            let exe = exe
                .as_str()
                .filter(|s| !s.trim().is_empty())
                .ok_or_else(|| {
                    AppError::BadRequest("executable must be a nonempty string".to_string())
                })?;
            let arguments = match args.get("args") {
                None => Vec::new(),
                Some(values) => values
                    .as_array()
                    .ok_or_else(|| {
                        AppError::BadRequest("args must be an array of strings".to_string())
                    })?
                    .iter()
                    .map(|value| {
                        value.as_str().map(str::to_owned).ok_or_else(|| {
                            AppError::BadRequest("args must contain only strings".to_string())
                        })
                    })
                    .collect::<Result<Vec<_>, _>>()?,
            };
            (exe.to_string(), arguments)
        }
        (None, Some(command)) => {
            if args.get("args").is_some() {
                return Err(AppError::BadRequest("args requires executable".to_string()));
            }
            let command = command
                .as_str()
                .ok_or_else(|| AppError::BadRequest("command must be a string".to_string()))?;
            if command.contains(['\0', '\n', '\r']) {
                return Err(AppError::Forbidden(
                    "Process command contains control separators".to_string(),
                ));
            }
            let tokens = crate::utils::security::parse_command_tokens(command);
            let exe = tokens
                .first()
                .ok_or_else(|| AppError::BadRequest("command must not be empty".to_string()))?;
            (exe.clone(), tokens[1..].to_vec())
        }
        (None, None) => {
            return Err(AppError::BadRequest(
                "executable or command is required".to_string(),
            ))
        }
    };
    if std::iter::once(&executable)
        .chain(arguments.iter())
        .any(|value| value.contains(['\0', '\n', '\r']))
    {
        return Err(AppError::Forbidden(
            "Process arguments contain control separators".to_string(),
        ));
    }
    Ok((executable, arguments))
}

pub(crate) fn shell_command_display(executable: &str, args: &[String]) -> String {
    std::iter::once(executable)
        .chain(args.iter().map(String::as_str))
        .map(|part| format!("\"{}\"", part.replace('\\', "\\\\").replace('"', "\\\"")))
        .collect::<Vec<_>>()
        .join(" ")
}

pub(crate) fn shell_working_directory(
    args: &serde_json::Value,
    workspace_root: &std::path::Path,
    current_dir: Option<&std::path::Path>,
) -> Result<std::path::PathBuf, AppError> {
    let requested = if let Some(cwd) = args.get("cwd") {
        cwd.as_str()
            .ok_or_else(|| AppError::BadRequest("cwd must be a string".to_string()))?
            .to_string()
    } else {
        current_dir
            .map(|p| p.to_string_lossy().into_owned())
            .unwrap_or_else(|| ".".to_string())
    };
    let path = crate::utils::security::validate_path(workspace_root, &requested)?.to_path_buf();
    if !path.is_dir() {
        return Err(AppError::BadRequest(
            "cwd must be an existing workspace directory".to_string(),
        ));
    }
    Ok(path)
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
    let args_obj = args
        .as_object()
        .ok_or_else(|| AppError::BadRequest("Arguments must be a JSON object".to_string()))?;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_path_alias_conflicts_and_sibling_prefix_are_rejected() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().join("workspace");
        let sibling = tmp.path().join("workspace-other");
        std::fs::create_dir_all(&root).unwrap();
        std::fs::create_dir_all(&sibling).unwrap();
        assert!(extract_path_from_args(
            &serde_json::json!({"path":"allowed.txt", "filename":"restricted.txt"}),
            &root
        )
        .is_err());
        assert!(extract_path_from_args(
            &serde_json::json!({"path":null, "filename":"allowed.txt"}),
            &root
        )
        .is_err());
        assert!(extract_path_from_args(
            &serde_json::json!({"path":sibling.join("new.txt")}),
            &root
        )
        .is_err());
        let aliases = serde_json::json!({"path":"allowed.txt", "filename":"./allowed.txt"});
        assert!(extract_path_from_args(&aliases, &root)
            .unwrap()
            .unwrap()
            .ends_with("allowed.txt"));
    }

    #[test]
    fn test_shell_argument_forms_share_one_execution_representation() {
        for args in [
            serde_json::json!({"command":"git status"}),
            serde_json::json!({"executable":"git","args":["status"]}),
        ] {
            assert_eq!(
                extract_shell_command(&args).unwrap(),
                ("git".to_string(), vec!["status".to_string()])
            );
        }
        for args in [
            serde_json::json!({"command":"git status","executable":"npm","args":["test"]}),
            serde_json::json!({"command":"git status","args":["ignored"]}),
            serde_json::json!({"executable":"git","args":[1]}),
            serde_json::json!({"executable":"git","envs":{}}),
            serde_json::json!({"executable":"git\nstatus"}),
            serde_json::json!({"command":"git\nstatus"}),
        ] {
            assert!(extract_shell_command(&args).is_err());
        }
        let args = vec![
            "a file.txt".to_string(),
            "a\\b".to_string(),
            "a\"b".to_string(),
        ];
        let display = shell_command_display("git", &args);
        assert_eq!(
            crate::utils::security::parse_command_tokens(&display),
            [vec!["git".to_string()], args].concat()
        );
    }

    #[test]
    fn test_shell_cwd_is_resolved_inside_the_workspace() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().join("workspace");
        let child = root.join("subdir");
        let sibling = tmp.path().join("workspace-other");
        std::fs::create_dir_all(&child).unwrap();
        std::fs::create_dir_all(&sibling).unwrap();
        assert_eq!(
            shell_working_directory(&serde_json::json!({"cwd":"subdir"}), &root, None).unwrap(),
            child.canonicalize().unwrap()
        );
        assert!(shell_working_directory(&serde_json::json!({"cwd":sibling}), &root, None).is_err());
        assert!(shell_working_directory(
            &serde_json::json!({"cwd":"../workspace-other"}),
            &root,
            None
        )
        .is_err());
        assert!(
            shell_working_directory(&serde_json::json!({"cwd":"missing"}), &root, None).is_err()
        );
        // An explicit cwd also anchors a built-in cd target.
        std::fs::create_dir_all(child.join("nested")).unwrap();
        std::fs::create_dir_all(root.join("nested")).unwrap();
        let run_dir =
            shell_working_directory(&serde_json::json!({"cwd":"subdir"}), &root, None).unwrap();
        let destination = shell_working_directory(
            &serde_json::json!({"cwd":run_dir.join("nested")}),
            &root,
            None,
        )
        .unwrap();
        assert_eq!(destination, child.join("nested").canonicalize().unwrap());
        assert_ne!(destination, root.join("nested").canonicalize().unwrap());
    }

    #[test]
    fn test_extract_path_from_args_filename_alias() {
        let tmp = tempfile::tempdir().unwrap();
        let ws = tmp.path();

        let args = serde_json::json!({
            "filename": "subdir/test.txt"
        });
        let res = extract_path_from_args(&args, ws).unwrap();
        assert!(res.is_some());
        assert!(res
            .unwrap()
            .ends_with(std::path::Path::new("subdir").join("test.txt")));

        let args2 = serde_json::json!({
            "file_name": "another.txt"
        });
        let res2 = extract_path_from_args(&args2, ws).unwrap();
        assert!(res2.is_some());
        assert!(res2.unwrap().ends_with("another.txt"));

        // Path traversal should be blocked
        let bad_args = serde_json::json!({
            "filename": "../outside.txt"
        });
        let bad_res = extract_path_from_args(&bad_args, ws);
        assert!(bad_res.is_err());
    }
}

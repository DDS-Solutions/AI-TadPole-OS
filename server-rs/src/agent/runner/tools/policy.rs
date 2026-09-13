//! @docs ARCHITECTURE:Registry
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Agent Runner / policy
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Type-safe state handling and bounded execution without unhandled panics.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: none declared
//! - **Witness Tests**: none declared

use super::validation::extract_path_from_args;
use crate::agent::runner::tools::capability::Permission;
use crate::agent::runner::tools::error::ToolExecutionError;

pub(crate) fn resolve_required_permission(
    name: &str,
    args: &serde_json::Value,
    workspace_root: &std::path::Path,
    is_mutating: bool,
) -> Result<Permission, ToolExecutionError> {
    let get_workspace_str = || {
        let workspace_canonical = workspace_root
            .canonicalize()
            .unwrap_or_else(|_| workspace_root.to_path_buf());
        let mut ws_str = workspace_canonical
            .to_string_lossy()
            .to_string()
            .replace('\\', "/");
        if ws_str.starts_with("//?/") {
            ws_str = ws_str[4..].to_string();
        }
        ws_str
    };

    match name {
        "read_file" | "get_file_contents" | "read_codebase_file" | "list_files" | "grep_search"
        | "list_file_symbols" | "get_symbol_body" => {
            let path = extract_path_from_args(args, workspace_root)?
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(get_workspace_str);
            Ok(Permission::FileRead(path))
        }
        "write_file" | "delete_file" => {
            let path = extract_path_from_args(args, workspace_root)?
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(get_workspace_str);
            Ok(Permission::FileWrite(path))
        }
        "execute_shell" => {
            let cmd = args.get("command").and_then(|v| v.as_str()).unwrap_or("");
            Ok(Permission::ShellExecute(cmd.to_string()))
        }
        "spawn_subagent" | "recruit_specialist" => Ok(Permission::SpawnAgent),
        "request_model_switch" => Ok(Permission::ModelSwitch),
        "fetch_url" => {
            let url = args.get("url").and_then(|v| v.as_str()).unwrap_or("");
            Ok(Permission::NetworkFetch(url.to_string()))
        }
        _ => {
            if is_mutating {
                Err(ToolExecutionError::SecurityBlocked(format!(
                    "Security Violation: Mutating permission is denied by default for unknown tool '{}'",
                    name
                )))
            } else {
                // Least-privilege: unknown non-mutating tools get ToolExec only,
                // not workspace-wide FileRead. Actual security is enforced by
                // the CBS skill-gate and oversight pipeline.
                Ok(Permission::ToolExec(name.to_string()))
            }
        }
    }
}

pub fn is_cacheable_tool_name(name: &str) -> bool {
    matches!(
        name,
        "read_file"
            | "get_file_contents"
            | "read_codebase_file"
            | "list_files"
            | "grep_search"
            | "list_file_symbols"
            | "get_symbol_body"
    )
}

pub fn is_mutating_tool_name(name: &str) -> bool {
    let mutating_operations = [
        "write_file",
        "delete_file",
        "execute_shell",
        "synthesize_micro_script",
        "refactor_synthesized_skill",
        "spawn_subagent",
        "recruit_specialist",
        "archive_to_vault",
        "archive_to_global_vault",
        "add_mission_task",
        "update_mission_task",
        "store_knowledge",
        "update_working_memory",
        "restore_file_version",
        "create_or_update_file",
        "push_files",
        "create_repository",
        "create_issue",
        "create_pull_request",
        "fork_repository",
        "create_branch",
        "update_issue",
        "add_issue_comment",
        "create_pull_request_review",
        "merge_pull_request",
        "update_pull_request_branch",
        "propose_asset_transaction",
        "confirm_asset_transaction",
        "cancel_asset_transaction",
    ];

    if mutating_operations.contains(&name) {
        return true;
    }

    if let Some(rest) = name.strip_prefix("mcp_") {
        for op in &mutating_operations {
            if rest.ends_with(op)
                && (rest.len() == op.len() || rest[..rest.len() - op.len()].ends_with('_'))
            {
                return true;
            }
        }
    }

    false
}

/// Checks if an MCP-prefixed tool name maps to a dangerous operation.
/// This includes all mutating operations plus GitHub data-access endpoints
/// that could leak private repository data.
///
/// IMPORTANT: every mutating tool is also considered dangerous via the
/// `is_mutating_tool_name` check. This composition is intentional.
/// Add new dangerous operations to one of:
///   - `is_mutating_tool_name` (modifies workspace files)
///   - this function's `matches!` block (reads sensitive MCP data)
pub(crate) fn is_dangerous_mcp_operation(name: &str) -> bool {
    if !name.starts_with("mcp_") {
        return false;
    }

    // Already covers mutating ops
    if is_mutating_tool_name(name) {
        return true;
    }

    let dangerous_read_ops = [
        "search_repositories",
        "search_code",
        "search_users",
        "search_issues",
        "get_pull_request_files",
        "get_file_contents",
        "get_pull_request",
        "get_pull_request_comments",
        "get_pull_request_reviews",
        "get_pull_request_status",
        "get_issue",
    ];

    if let Some(rest) = name.strip_prefix("mcp_") {
        for op in &dangerous_read_ops {
            if rest.ends_with(op)
                && (rest.len() == op.len() || rest[..rest.len() - op.len()].ends_with('_'))
            {
                return true;
            }
        }
    }

    false
}

/// Checks if a native (non-MCP) tool name is a dangerous operation.
/// These tools don't modify workspace files (so they're not in `is_mutating_tool_name`)
/// but they carry security risk from prompt injection, data exfiltration,
/// cryptographic signing authorization, lateral movement, or unauthorized state changes.
pub(crate) fn is_dangerous_native_operation(name: &str) -> bool {
    matches!(
        name,
        "request_model_switch"
            | "fetch_url"
            | "run_integrity_check"
            | "resolve_x402_challenge"
            | "send_mission_directive"
            | "send_agent_envelope"
            | "issue_alpha_directive"
            | "spawn_subagent"
            | "recruit_specialist"
            | "script_builder"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mcp_underscore_server_mutation_classification() {
        assert!(is_mutating_tool_name("write_file"));
        assert!(is_mutating_tool_name("mcp_git_create_repository"));
        assert!(is_mutating_tool_name("mcp_my_fs_server_write_file"));
        assert!(is_mutating_tool_name("mcp_complex_nested_name_delete_file"));
        assert!(!is_mutating_tool_name("mcp_my_fs_server_read_file"));
        assert!(is_dangerous_mcp_operation(
            "mcp_github_org_search_repositories"
        ));
    }
}

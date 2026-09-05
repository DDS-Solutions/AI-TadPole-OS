//! @docs ARCHITECTURE:Runner
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Agent Runner / manifest
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Type-safe state handling and bounded execution without unhandled panics.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: `[manifest]`
//! - **Witness Tests**: tests::test_manifest_uniqueness, tests::test_manifest_schema_validity, tests::test_manifest_matches_dispatcher, tests::test_tool_security_classifications_pinned

use super::trait_tool::ToolDefinitionData;
use once_cell::sync::Lazy;

pub const MANIFEST_VERSION: &str = "1.3.0";

fn def(name: &str, description: &str, parameters: serde_json::Value) -> ToolDefinitionData {
    let is_mutating = crate::agent::runner::tools::policy::is_mutating_tool_name(name);
    let is_dangerous = is_mutating
        || crate::agent::runner::tools::policy::is_dangerous_native_operation(name)
        || matches!(
            name,
            "issue_alpha_directive"
                | "spawn_subagent"
                | "recruit_specialist"
                | "fetch_url"
                | "script_builder"
                | "request_model_switch"
                | "send_mission_directive"
                | "send_agent_envelope"
                | "resolve_x402_challenge"
        );
    let is_cacheable = crate::agent::runner::tools::policy::is_cacheable_tool_name(name);

    ToolDefinitionData {
        name: name.to_string(),
        description: description.to_string(),
        parameters,
        is_mutating,
        is_dangerous,
        is_cacheable,
    }
}

static CORE_MANIFEST: Lazy<Vec<ToolDefinitionData>> = Lazy::new(|| {
    tracing::debug!("[manifest] Loaded core tool manifest v{}", MANIFEST_VERSION);
    build_core_tool_manifest()
});

pub fn load_core_tool_manifest() -> Vec<ToolDefinitionData> {
    CORE_MANIFEST.clone()
}

pub fn get_core_tool_manifest() -> &'static [ToolDefinitionData] {
    &CORE_MANIFEST
}

fn build_core_tool_manifest() -> Vec<ToolDefinitionData> {
    vec![
        // --- Core Operational Tools ---
        def(
            "spawn_subagent",
            "Spawns one or more specialized sub-agents to handle tasks in parallel.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "agent_id": { "type": "string", "description": "The ID of the specialist agent to recruit." },
                    "agent_ids": { "type": "array", "items": { "type": "string" }, "description": "Optional: multiple specialist IDs.", "maxItems": 10 },
                    "message": { "type": "string", "description": "Instruction for the sub-agent(s)." },
                    "role": { "type": "string", "description": "Optional: override role (cannot exceed parent authority)." }
                },
                "required": ["message"]
            }),
        ),
        def(
            "issue_alpha_directive",
            "Delegates a strategic objective to the operations coordinator (COO / Alpha).",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "directive": { "type": "string", "description": "Strategic objective to delegate." }
                },
                "required": ["directive"]
            }),
        ),
        def(
            "share_finding",
            "Shares a key finding, insight, or data point with the rest of the swarm.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "topic": { "type": "string", "description": "Topic or category of the finding." },
                    "finding": { "type": "string", "description": "The insight or discovery text." }
                },
                "required": ["topic", "finding"]
            }),
        ),
        def(
            "send_mission_directive",
            "Directly delegates a specific task or instruction to another agent.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "agent_id": { "type": "string", "description": "Target agent ID." },
                    "instruction": { "type": "string", "description": "Specific mission instruction." }
                },
                "required": ["agent_id", "instruction"]
            }),
        ),
        def(
            "archive_to_global_vault",
            "Archives critical intelligence to the global vault.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "content": { "type": "string", "description": "Full document content to archive." },
                    "summary": { "type": "string", "description": "High-level summary for indexing." }
                },
                "required": ["content", "summary"]
            }),
        ),
        def(
            "search_global_vault",
            "Searches the swarm-wide global vault.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "query": { "type": "string", "description": "Search query keywords." }
                },
                "required": ["query"]
            }),
        ),
        def(
            "update_working_memory",
            "Updates your persistent structured working memory (scratchpad).",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "memory": { "type": "object", "description": "Key-value map of memory entries." }
                },
                "required": ["memory"]
            }),
        ),
        def(
            "complete_mission",
            "Signals that the mission objective has been achieved.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "final_report": { "type": "string", "description": "Final mission synthesis and delivery report." }
                },
                "required": ["final_report"]
            }),
        ),
        def(
            "add_mission_task",
            "Adds a new task to the mission backlog DAG.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "description": { "type": "string", "description": "Task description." },
                    "dependencies": { "type": "array", "items": { "type": "string" }, "description": "Task IDs that must complete first." }
                },
                "required": ["description"]
            }),
        ),
        def(
            "update_mission_task",
            "Updates the status of a task node in the mission DAG.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "task_id": { "type": "string", "description": "The unique task node ID." },
                    "status": { "type": "string", "enum": ["pending", "in_progress", "completed", "failed"], "description": "Updated task execution status." },
                    "findings": { "type": "string", "description": "Optional notes or findings from task execution." }
                },
                "required": ["task_id", "status"]
            }),
        ),
        def(
            "synthesize_micro_script",
            "Autonomously synthesizes a new Python micro-script tool for the swarm to use. Requires oversight approval.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "skill_name": { "type": "string", "description": "Short, unique identifier for the tool (snake_case)." },
                    "description": { "type": "string", "description": "What this tool does." },
                    "code": { "type": "string", "description": "The raw Python code for the tool." },
                    "schema": { "type": "object", "description": "The JSON schema defining the tool's parameters." }
                },
                "required": ["skill_name", "description", "code", "schema"]
            }),
        ),
        def(
            "refactor_synthesized_skill",
            "Refactors or updates an existing autonomously synthesized tool. Requires oversight approval.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "skill_name": { "type": "string", "description": "The identifier of the existing synthesized tool." },
                    "description": { "type": "string", "description": "Updated description (optional)." },
                    "code": { "type": "string", "description": "The new Python code for the tool." }
                },
                "required": ["skill_name", "code"]
            }),
        ),
        // --- Filesystem Tools ---
        def(
            "read_file",
            "Reads content from a file. Supporting line range slicing via start_line and end_line parameters.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "filename": { "type": "string", "description": "Path to the target file." },
                    "path": { "type": "string", "description": "Alternative alias for target file path." },
                    "start_line": { "type": "integer", "description": "Optional starting line number (1-indexed, inclusive). Defaults to 1." },
                    "end_line": { "type": "integer", "description": "Optional ending line number (1-indexed, inclusive). Use -1 to read to end. Defaults to -1." }
                },
                "required": ["filename"]
            }),
        ),
        def(
            "write_file",
            "Writes content to a file.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "filename": { "type": "string", "description": "Path to the target file." },
                    "content": { "type": "string", "description": "Content string to write." }
                },
                "required": ["filename", "content"]
            }),
        ),
        def(
            "list_files",
            "Lists directory contents.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "dir": { "type": "string", "description": "Optional directory path relative to workspace root. Defaults to root if omitted." }
                }
            }),
        ),
        def(
            "delete_file",
            "Deletes a file from the workspace.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "filename": { "type": "string", "description": "Path to the file to delete." }
                },
                "required": ["filename"]
            }),
        ),
        def(
            "read_codebase_file",
            "Reads a file from the central project codebase with safety filtering. Supports line range slicing via start_line and end_line parameters.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "Path to the file relative to workspace root (e.g. 'src/main.rs', 'Cargo.toml'). Access to credential/env files is prohibited." },
                    "start_line": { "type": "integer", "description": "Optional starting line number (1-indexed, inclusive). Defaults to 1." },
                    "end_line": { "type": "integer", "description": "Optional ending line number (1-indexed, inclusive). Use -1 to read to end. Defaults to -1." }
                },
                "required": ["path"]
            }),
        ),
        def(
            "list_file_symbols",
            "Parses a codebase file and lists all functions, structs, classes, and variables defined in it. Use to understand file structure before reading the full content.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "Path to the file relative to workspace root (e.g. 'src/main.rs')." }
                },
                "required": ["path"]
            }),
        ),
        def(
            "get_symbol_body",
            "Extracts the full implementation body of a named symbol (function, struct, class, enum) from a codebase file. More token-efficient than reading the whole file.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "Path to the file relative to workspace root." },
                    "symbol": { "type": "string", "description": "Exact name of the function, struct, class, or enum to extract." }
                },
                "required": ["path", "symbol"]
            }),
        ),
        def(
            "grep_search",
            "Performs a regex search across files.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "query": { "type": "string", "description": "Regular expression or literal search string." },
                    "path": { "type": "string", "description": "Optional directory or file path scope." }
                },
                "required": ["query"]
            }),
        ),
        // --- Advanced Tools ---
        def(
            "execute_shell",
            "Executes a direct process executable in the mission workspace. Accepts structured 'executable' and 'args' array or legacy single 'command' string without shell operators.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "executable": { "type": "string", "description": "Binary file to execute directly (e.g. 'cargo', 'git', 'npm', 'python')" },
                    "args": { "type": "array", "items": { "type": "string" }, "description": "List of command arguments passed directly to binary" },
                    "cwd": { "type": "string", "description": "Optional working directory override strictly relative to workspace root" },
                    "envs": { "type": "object", "additionalProperties": { "type": "string" }, "description": "Optional key-value map of environment variables" },
                    "command": { "type": "string", "description": "Legacy command string (must not contain compound operators like &&, ||, ;, |)" }
                },
                "required": ["executable"]
            }),
        ),
        def(
            "search_web",
            "Performs a web search.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "query": { "type": "string", "description": "Web search query." }
                },
                "required": ["query"]
            }),
        ),
        def(
            "fetch_url",
            "Retrieves the text content of a public URL.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "url": { "type": "string", "description": "Public HTTP/HTTPS URL to fetch." }
                },
                "required": ["url"]
            }),
        ),
        def(
            "get_agent_metrics",
            "Retrieves performance metrics for a specific agent.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "agent_id": { "type": "string", "description": "Target agent ID." }
                },
                "required": ["agent_id"]
            }),
        ),
        def(
            "script_builder",
            "Executes a batch of tool calls sequentially.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "allowed_tools": {
                        "type": "array",
                        "items": { "type": "string" },
                        "description": "List of tool names allowed to execute in this batch. Any tool in 'steps' not in this list will be rejected."
                    },
                    "steps": {
                        "type": "array",
                        "items": {
                            "type": "object",
                            "properties": {
                                "tool": { "type": "string", "description": "Name of the tool to execute." },
                                "params": { "type": "object", "description": "Tool parameters." }
                            },
                            "required": ["tool"]
                        },
                        "maxItems": 50
                    }
                },
                "required": ["steps", "allowed_tools"]
            }),
        ),
        def(
            "store_knowledge",
            "Write a permanent fact to the Institutional Knowledge Store. Use for: business SOPs, client facts, agent learning, decision patterns.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "text": { "type": "string", "description": "The knowledge to store (full sentence/paragraph, not just a keyword)" },
                    "topic": { "type": "string", "description": "Category: general | sop | agent_pattern | finance | sales | legal | payroll | pii | medical" },
                    "cluster_id": { "type": "string", "description": "Optional: scope to a specific cluster" },
                    "confidence": { "type": "number", "description": "Optional: confidence score (0.0 to 1.0), defaults to 1.0" },
                    "ttl_days": { "type": "integer", "description": "Optional: days until expiry. Omit/None for default (90 days for agents, permanent if human-confirmed)" }
                },
                "required": ["text", "topic"]
            }),
        ),
        def(
            "search_knowledge",
            "Semantic search across the Institutional Knowledge Store for cross-cluster and cross-restart intelligence.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "query": { "type": "string", "description": "Natural language query to search for" },
                    "topic": { "type": "string", "description": "Optional: pre-filter by specific topic" },
                    "limit": { "type": "integer", "description": "Optional: max results, default 5" }
                },
                "required": ["query"]
            }),
        ),
        def(
            "request_model_switch",
            "Requests a model switch to another configured model slot (planning, execution, default) and submits it to the oversight action ledger for human approval.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "slot": { "type": "string", "enum": ["planning", "execution", "default"], "description": "The target model slot to switch to. Must be 'planning', 'execution', or 'default'." }
                },
                "required": ["slot"]
            }),
        ),
        // --- A2A Transactional & Payment Tools ---
        def(
            "send_agent_envelope",
            "Dispatches a structured informational envelope (message) to another agent.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "target_agent_id": { "type": "string", "description": "The recipient agent ID." },
                    "instruction": { "type": "string", "description": "The message body / instructions." },
                    "artifacts": {
                        "type": "array",
                        "items": { "type": "string" },
                        "description": "Optional: list of file paths/artifacts to pass to the target agent."
                    }
                },
                "required": ["target_agent_id", "instruction"]
            }),
        ),
        def(
            "propose_asset_transaction",
            "Initiates the Prepare phase of an agent-to-agent asset transaction.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "seller_id": { "type": "string", "description": "The wallet address or agent ID of the seller." },
                    "amount": { "type": "integer", "minimum": 1, "description": "The positive u64 micro-USDC payment amount." },
                    "challenge_data": { "type": "string", "description": "Optional: challenge data matching the x402 challenge." },
                    "challenge_signature": { "type": "string", "description": "Optional: challenge signature hex matching the x402 challenge." }
                },
                "required": ["seller_id", "amount"]
            }),
        ),
        def(
            "confirm_asset_transaction",
            "Finalizes the Commit phase of an agent-to-agent asset transaction using a lock ID.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "lock_id": { "type": "string", "description": "The lock ID returned by the propose step." }
                },
                "required": ["lock_id"]
            }),
        ),
        def(
            "cancel_asset_transaction",
            "Rolls back a locked transaction, releasing locked funds.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "lock_id": { "type": "string", "description": "The lock ID to roll back." }
                },
                "required": ["lock_id"]
            }),
        ),
        def(
            "resolve_x402_challenge",
            "Resolves an HTTP 402 challenge by verifying the challenge structure against budget limits and signing with domain-separated transaction proof.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "challenge_data": { "type": "string", "description": "The structured challenge string issued by the seller." }
                },
                "required": ["challenge_data"]
            }),
        ),
        def(
            "archive_to_vault",
            "Archives critical research findings or notes to the local Markdown vault.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "filename": { "type": "string", "description": "Vault filename (e.g. 'findings.md')" },
                    "content": { "type": "string", "description": "Markdown content to archive" }
                },
                "required": ["filename", "content"]
            }),
        ),
        def(
            "restore_file_version",
            "Restores a workspace file to a target CAS revision version number.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "filename": { "type": "string", "description": "Path to the file to restore" },
                    "version_num": { "type": "integer", "description": "Revision version number to restore to" }
                },
                "required": ["filename", "version_num"]
            }),
        ),
        def(
            "get_file_history",
            "Retrieves revision history metadata for a workspace file from CAS.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "filename": { "type": "string", "description": "Path to the file" }
                },
                "required": ["filename"]
            }),
        ),
        def(
            "get_file_contents",
            "Alias for read_file. Reads content from a file in the mission workspace.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "filename": { "type": "string", "description": "Path to the target file." },
                    "start_line": { "type": "integer", "description": "Optional start line." },
                    "end_line": { "type": "integer", "description": "Optional end line." }
                },
                "required": ["filename"]
            }),
        ),
        def(
            "recruit_specialist",
            "Recruits a specialist agent subagent for a mission task.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "agent_id": { "type": "string", "description": "The target agent ID of the specialist." },
                    "message": { "type": "string", "description": "Task instructions for the specialist." }
                },
                "required": ["agent_id", "message"]
            }),
        ),
        def(
            "pin_mission",
            "Pins a mission to the workspace dashboard.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "mission_id": { "type": "string", "description": "Mission ID to pin." }
                },
                "required": ["mission_id"]
            }),
        ),
        def(
            "search_mission_knowledge",
            "Searches knowledge context specific to the current mission.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "query": { "type": "string", "description": "Search query." }
                },
                "required": ["query"]
            }),
        ),
        def(
            "propose_capability",
            "Proposes a new agent capability to the governance system.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "name": { "type": "string", "description": "Proposed capability name." },
                    "description": { "type": "string", "description": "Detailed specification of capability." }
                },
                "required": ["name", "description"]
            }),
        ),
        def(
            "request_peer_audit",
            "Requests a peer review/audit of code or artifacts from another agent.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "target_agent_id": { "type": "string", "description": "Agent ID requested to perform audit." },
                    "artifact_path": { "type": "string", "description": "File or artifact path to audit." }
                },
                "required": ["target_agent_id", "artifact_path"]
            }),
        ),
        def(
            "submit_peer_review",
            "Submits a peer review report for an audited artifact.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "audit_id": { "type": "string", "description": "Audit ID being responded to." },
                    "approved": { "type": "boolean", "description": "Whether the artifact is approved." },
                    "comments": { "type": "string", "description": "Audit findings and suggestions." }
                },
                "required": ["audit_id", "approved", "comments"]
            }),
        ),
        def(
            "query_financial_logs",
            "Queries financial telemetry and transaction audit logs.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "query": { "type": "string", "description": "Search query for financial audit logs." }
                },
                "required": ["query"]
            }),
        ),
        def(
            "notify_discord",
            "Sends a notification message to the configured Discord webhook channel.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "message": { "type": "string", "description": "Message content to send." }
                },
                "required": ["message"]
            }),
        ),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn test_manifest_uniqueness() {
        let manifest = load_core_tool_manifest();
        let mut names = HashSet::new();
        for tool in &manifest {
            assert!(
                names.insert(&tool.name),
                "Duplicate tool name found in manifest: {}",
                tool.name
            );
        }
        assert_eq!(names.len(), manifest.len());
    }

    #[test]
    fn test_manifest_schema_validity() {
        let manifest = load_core_tool_manifest();
        for tool in &manifest {
            assert!(!tool.name.trim().is_empty(), "Tool name cannot be empty");
            assert!(
                !tool.description.trim().is_empty(),
                "Tool description for '{}' cannot be empty",
                tool.name
            );
            assert!(
                tool.parameters.is_object(),
                "Parameters for '{}' must be a JSON Object",
                tool.name
            );

            let params = tool.parameters.as_object().unwrap();
            assert_eq!(
                params.get("type").and_then(|v| v.as_str()),
                Some("object"),
                "Schema type for '{}' must be 'object'",
                tool.name
            );

            if let Some(required) = params.get("required").and_then(|v| v.as_array()) {
                if let Some(props) = params.get("properties").and_then(|v| v.as_object()) {
                    for req_field in required {
                        let field_name = req_field.as_str().expect("Required item must be string");
                        assert!(
                            props.contains_key(field_name),
                            "Tool '{}' requires field '{}' which is missing from properties",
                            tool.name,
                            field_name
                        );
                    }
                } else {
                    panic!("Tool '{}' has 'required' but lacks 'properties'", tool.name);
                }
            }
        }
    }

    #[test]
    fn test_manifest_matches_dispatcher() {
        let manifest = load_core_tool_manifest();
        let dispatcher_tools: HashSet<&'static str> = [
            // 1. Mission Tools
            "share_finding",
            "complete_mission",
            "pin_mission",
            "search_mission_knowledge",
            "read_codebase_file",
            "propose_capability",
            "list_file_symbols",
            "get_symbol_body",
            "send_mission_directive",
            "request_peer_audit",
            "submit_peer_review",
            "archive_to_global_vault",
            "search_global_vault",
            "update_working_memory",
            "query_financial_logs",
            "store_knowledge",
            "search_knowledge",
            "request_model_switch",
            "send_agent_envelope",
            "propose_asset_transaction",
            "confirm_asset_transaction",
            "cancel_asset_transaction",
            "resolve_x402_challenge",
            "add_mission_task",
            "update_mission_task",
            // 2. Filesystem Tools
            "read_file",
            "get_file_contents",
            "write_file",
            "list_files",
            "delete_file",
            "grep_search",
            "archive_to_vault",
            "restore_file_version",
            "get_file_history",
            // 3. Swarm Tools
            "spawn_subagent",
            "issue_alpha_directive",
            "recruit_specialist",
            // 4. Metrics & External
            "get_agent_metrics",
            "notify_discord",
            "fetch_url",
            "script_builder",
            "search_web",
            "execute_shell",
            // 5. Evolution Tools
            "synthesize_micro_script",
            "refactor_synthesized_skill",
        ]
        .into_iter()
        .collect();

        for tool in &manifest {
            assert!(
                dispatcher_tools.contains(tool.name.as_str()),
                "Manifest tool '{}' is missing from categorical dispatcher handler mappings",
                tool.name
            );
        }

        for dispatcher_tool in &dispatcher_tools {
            assert!(
                manifest.iter().any(|m| m.name == *dispatcher_tool),
                "Dispatcher routes tool '{}' which is missing from load_core_tool_manifest()",
                dispatcher_tool
            );
        }
    }

    #[test]
    fn test_tool_security_classifications_pinned() {
        let manifest = load_core_tool_manifest();

        let expected_mutating: HashSet<&'static str> = [
            "write_file",
            "delete_file",
            "restore_file_version",
            "archive_to_vault",
            "archive_to_global_vault",
            "add_mission_task",
            "update_mission_task",
            "store_knowledge",
            "update_working_memory",
            "propose_asset_transaction",
            "confirm_asset_transaction",
            "cancel_asset_transaction",
            "execute_shell",
            "synthesize_micro_script",
            "refactor_synthesized_skill",
            "spawn_subagent",
            "recruit_specialist",
        ]
        .into_iter()
        .collect();

        let expected_dangerous: HashSet<&'static str> = [
            // Mutating tools
            "write_file",
            "delete_file",
            "restore_file_version",
            "archive_to_vault",
            "archive_to_global_vault",
            "add_mission_task",
            "update_mission_task",
            "store_knowledge",
            "update_working_memory",
            "propose_asset_transaction",
            "confirm_asset_transaction",
            "cancel_asset_transaction",
            "execute_shell",
            "synthesize_micro_script",
            "refactor_synthesized_skill",
            "spawn_subagent",
            "recruit_specialist",
            // Native dangerous read/delegation/signing tools
            "issue_alpha_directive",
            "fetch_url",
            "script_builder",
            "request_model_switch",
            "send_mission_directive",
            "send_agent_envelope",
            "resolve_x402_challenge",
        ]
        .into_iter()
        .collect();

        let expected_cacheable: HashSet<&'static str> = [
            "read_file",
            "get_file_contents",
            "read_codebase_file",
            "list_files",
            "grep_search",
            "list_file_symbols",
            "get_symbol_body",
        ]
        .into_iter()
        .collect();

        for tool in &manifest {
            let is_mut = tool.is_mutating;
            let is_dang = tool.is_dangerous;
            let is_cach = tool.is_cacheable;

            assert_eq!(
                is_mut,
                expected_mutating.contains(tool.name.as_str()),
                "Mutating classification mismatch for tool '{}'",
                tool.name
            );

            assert_eq!(
                is_dang,
                expected_dangerous.contains(tool.name.as_str()),
                "Dangerous classification mismatch for tool '{}'",
                tool.name
            );

            assert_eq!(
                is_cach,
                expected_cacheable.contains(tool.name.as_str()),
                "Cacheable classification mismatch for tool '{}'",
                tool.name
            );
        }
    }
}

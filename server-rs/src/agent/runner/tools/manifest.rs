//! @docs ARCHITECTURE:Runner
//!
//! ### AI Assist Note
//! **Tool Manifest**: Centralized repository of all core tool definitions.
//! Synchronizes Discovery (Synthesis) and Execution (Dispatcher).
//!
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: Runtime logic error, state desynchronization, or resource exhaustion.
//! - **Telemetry Link**: Search `[manifest]` in tracing logs.

use super::trait_tool::ToolDefinitionData;

pub fn load_core_tool_manifest() -> Vec<ToolDefinitionData> {
    vec![
        // --- Core Operational Tools ---
        ToolDefinitionData {
            name: "spawn_subagent".to_string(),
            description: "Spawns one or more specialized sub-agents to handle tasks in parallel."
                .to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "agent_id": { "type": "string", "description": "The ID of the specialist agent to recruit." },
                    "agent_ids": { "type": "array", "items": { "type": "string" }, "description": "Optional: multiple specialist IDs." },
                    "message": { "type": "string", "description": "Instruction for the sub-agent(s)." },
                    "role": { "type": "string", "description": "Optional: override role." }
                },
                "required": ["message"]
            }),
        },
        ToolDefinitionData {
            name: "issue_alpha_directive".to_string(),
            description: "Delegates a complex mission to Tadpole Alpha (the COO).".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "directive": { "type": "string", "description": "Strategic objective to delegate." }
                },
                "required": ["directive"]
            }),
        },
        ToolDefinitionData {
            name: "share_finding".to_string(),
            description: "Shares a key finding, insight, or data point with the rest of the swarm."
                .to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "topic": { "type": "string" },
                    "finding": { "type": "string" }
                },
                "required": ["topic", "finding"]
            }),
        },
        ToolDefinitionData {
            name: "send_mission_directive".to_string(),
            description: "Directly delegates a specific task or instruction to another agent."
                .to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "agent_id": { "type": "string" },
                    "instruction": { "type": "string" }
                },
                "required": ["agent_id", "instruction"]
            }),
        },
        ToolDefinitionData {
            name: "archive_to_global_vault".to_string(),
            description: "Archives critical intelligence to the global vault.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "content": { "type": "string" },
                    "summary": { "type": "string" }
                },
                "required": ["content", "summary"]
            }),
        },
        ToolDefinitionData {
            name: "search_global_vault".to_string(),
            description: "Searches the swarm-wide global vault.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "query": { "type": "string" }
                },
                "required": ["query"]
            }),
        },
        ToolDefinitionData {
            name: "synthesize_micro_script".to_string(),
            description: "Autonomously synthesizes a new micro-script (skill).".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "skill_name": { "type": "string" },
                    "description": { "type": "string" },
                    "code": { "type": "string" }
                },
                "required": ["skill_name", "description", "code"]
            }),
        },
        ToolDefinitionData {
            name: "refactor_synthesized_skill".to_string(),
            description: "Autonomously refactors or updates an existing micro-script (skill)."
                .to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "skill_name": { "type": "string" },
                    "new_description": { "type": "string" },
                    "new_code": { "type": "string" }
                },
                "required": ["skill_name", "new_code"]
            }),
        },
        ToolDefinitionData {
            name: "update_working_memory".to_string(),
            description: "Updates your persistent structured working memory (scratchpad)."
                .to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "memory": { "type": "object" }
                },
                "required": ["memory"]
            }),
        },
        ToolDefinitionData {
            name: "complete_mission".to_string(),
            description: "Signals that the mission objective has been achieved.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "final_report": { "type": "string" }
                },
                "required": ["final_report"]
            }),
        },
        ToolDefinitionData {
            name: "add_mission_task".to_string(),
            description: "Adds a new task to the mission backlog DAG.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "description": { "type": "string" },
                    "dependencies": { "type": "array", "items": { "type": "string" } }
                },
                "required": ["description"]
            }),
        },
        ToolDefinitionData {
            name: "update_mission_task".to_string(),
            description: "Updates the status of a task node in the mission DAG.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "task_id": { "type": "string" },
                    "status": { "type": "string", "enum": ["pending", "in_progress", "completed", "failed"] },
                    "findings": { "type": "string" }
                },
                "required": ["task_id", "status"]
            }),
        },
        ToolDefinitionData {
            name: "synthesize_micro_script".to_string(),
            description: "Autonomously synthesizes a new Python micro-script tool for the swarm to use. Requires oversight approval.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "skill_name": { "type": "string", "description": "Short, unique identifier for the tool (snake_case)." },
                    "description": { "type": "string", "description": "What this tool does." },
                    "code": { "type": "string", "description": "The raw Python code for the tool." },
                    "schema": { "type": "object", "description": "The JSON schema defining the tool's parameters." }
                },
                "required": ["skill_name", "description", "code", "schema"]
            }),
        },
        ToolDefinitionData {
            name: "refactor_synthesized_skill".to_string(),
            description: "Refactors or updates an existing autonomously synthesized tool. Requires oversight approval.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "skill_name": { "type": "string", "description": "The identifier of the existing synthesized tool." },
                    "description": { "type": "string", "description": "Updated description (optional)." },
                    "code": { "type": "string", "description": "The new Python code for the tool." }
                },
                "required": ["skill_name", "code"]
            }),
        },
        // --- Filesystem Tools ---
        ToolDefinitionData {
            name: "read_file".to_string(),
            description: "Reads content from a file. Supporting line range slicing via start_line and end_line parameters.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "filename": { "type": "string" },
                    "path": { "type": "string" },
                    "start_line": { "type": "integer", "description": "Optional starting line number (1-indexed, inclusive). Defaults to 1." },
                    "end_line": { "type": "integer", "description": "Optional ending line number (1-indexed, inclusive). Use -1 to read to end. Defaults to -1." }
                },
                "required": ["filename"]
            }),
        },
        ToolDefinitionData {
            name: "write_file".to_string(),
            description: "Writes content to a file.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "filename": { "type": "string" },
                    "content": { "type": "string" }
                },
                "required": ["filename", "content"]
            }),
        },
        ToolDefinitionData {
            name: "list_files".to_string(),
            description: "Lists directory contents.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "dir": { "type": "string" }
                }
            }),
        },
        ToolDefinitionData {
            name: "delete_file".to_string(),
            description: "Deletes a file from the workspace.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "filename": { "type": "string" }
                },
                "required": ["filename"]
            }),
        },
        ToolDefinitionData {
            name: "read_codebase_file".to_string(),
            description: "Reads a file from the central project codebase. Supporting line range slicing via start_line and end_line parameters. Requires Oversight approval.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "Path to the file relative to workspace root (e.g. 'src/main.rs', 'Cargo.toml'). Note: 'filename' is also accepted as an alias." },
                    "start_line": { "type": "integer", "description": "Optional starting line number (1-indexed, inclusive). Defaults to 1." },
                    "end_line": { "type": "integer", "description": "Optional ending line number (1-indexed, inclusive). Use -1 to read to end. Defaults to -1." }
                },
                "required": ["path"]
            }),
        },
        ToolDefinitionData {
            name: "list_file_symbols".to_string(),
            description: "Parses a codebase file and lists all functions, structs, classes, and variables defined in it. Use to understand file structure before reading the full content.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "Path to the file relative to workspace root (e.g. 'src/main.rs'). Note: 'filename' is also accepted as an alias." }
                },
                "required": ["path"]
            }),
        },
        ToolDefinitionData {
            name: "get_symbol_body".to_string(),
            description: "Extracts the full implementation body of a named symbol (function, struct, class, enum) from a codebase file. More token-efficient than reading the whole file.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "Path to the file relative to workspace root." },
                    "symbol": { "type": "string", "description": "Exact name of the function, struct, class, or enum to extract." }
                },
                "required": ["path", "symbol"]
            }),
        },
        ToolDefinitionData {
            name: "grep_search".to_string(),
            description: "Performs a regex search across files.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "query": { "type": "string" },
                    "path": { "type": "string" }
                },
                "required": ["query"]
            }),
        },
        // --- Advanced Tools ---
        ToolDefinitionData {
            name: "execute_shell".to_string(),
            description: "Executes a direct process executable in the mission workspace. Accepts structured 'executable' and 'args' array or legacy single 'command' string without shell operators.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "executable": { "type": "string", "description": "Binary file to execute directly (e.g. 'cargo', 'git', 'npm', 'python')" },
                    "args": { "type": "array", "items": { "type": "string" }, "description": "List of command arguments passed directly to binary" },
                    "cwd": { "type": "string", "description": "Optional working directory override relative to workspace root" },
                    "envs": { "type": "object", "description": "Optional key-value map of environment variables" },
                    "command": { "type": "string", "description": "Legacy command string (must not contain compound operators like &&, ||, ;, |)" }
                }
            }),
        },
        ToolDefinitionData {
            name: "search_web".to_string(),
            description: "Performs a web search.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "query": { "type": "string" }
                },
                "required": ["query"]
            }),
        },
        ToolDefinitionData {
            name: "fetch_url".to_string(),
            description: "Retrieves the text content of a public URL.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "url": { "type": "string" }
                },
                "required": ["url"]
            }),
        },
        ToolDefinitionData {
            name: "get_agent_metrics".to_string(),
            description: "Retrieves performance metrics for a specific agent.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "agent_id": { "type": "string" }
                },
                "required": ["agent_id"]
            }),
        },
        ToolDefinitionData {
            name: "script_builder".to_string(),
            description: "Executes a batch of tool calls sequentially.".to_string(),
            parameters: serde_json::json!({
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
                                "tool": { "type": "string" },
                                "params": { "type": "object" }
                            },
                            "required": ["tool"]
                        }
                    }
                },
                "required": ["steps", "allowed_tools"]
            }),
        },
        ToolDefinitionData {
            name: "store_knowledge".to_string(),
            description: "Write a permanent fact to the Institutional Knowledge Store. Use for: business SOPs, client facts, agent learning, decision patterns.".to_string(),
            parameters: serde_json::json!({
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
        },
        ToolDefinitionData {
            name: "search_knowledge".to_string(),
            description: "Semantic search across the Institutional Knowledge Store for cross-cluster and cross-restart intelligence.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "query": { "type": "string", "description": "Natural language query to search for" },
                    "topic": { "type": "string", "description": "Optional: pre-filter by specific topic" },
                    "limit": { "type": "integer", "description": "Optional: max results, default 5" }
                },
                "required": ["query"]
            }),
        },
        ToolDefinitionData {
            name: "request_model_switch".to_string(),
            description: "Requests a model switch to another configured model slot (planning, execution, default) and submits it to the oversight action ledger for human approval.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "slot": { "type": "string", "description": "The target model slot to switch to. Must be 'planning', 'execution', or 'default'." }
                },
                "required": ["slot"]
            }),
        },
        // --- A2A Transactional & Payment Tools ---
        ToolDefinitionData {
            name: "send_agent_envelope".to_string(),
            description: "Dispatches a structured informational envelope (message) to another agent.".to_string(),
            parameters: serde_json::json!({
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
        },
        ToolDefinitionData {
            name: "propose_asset_transaction".to_string(),
            description: "Initiates the Prepare phase of an agent-to-agent asset transaction.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "seller_id": { "type": "string", "description": "The wallet address or agent ID of the seller." },
                    "amount": { "type": "integer", "description": "The u64 micro-USDC payment amount." },
                    "challenge_data": { "type": "string", "description": "Optional: challenge data matching the x402 challenge." },
                    "challenge_signature": { "type": "string", "description": "Optional: challenge signature hex matching the x402 challenge." }
                },
                "required": ["seller_id", "amount"]
            }),
        },
        ToolDefinitionData {
            name: "confirm_asset_transaction".to_string(),
            description: "Finalizes the Commit phase of an agent-to-agent asset transaction using a lock ID.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "lock_id": { "type": "string", "description": "The lock ID returned by the propose step." }
                },
                "required": ["lock_id"]
            }),
        },
        ToolDefinitionData {
            name: "cancel_asset_transaction".to_string(),
            description: "Rolls back a locked transaction, releasing locked funds.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "lock_id": { "type": "string", "description": "The lock ID to roll back." }
                },
                "required": ["lock_id"]
            }),
        },
        ToolDefinitionData {
            name: "resolve_x402_challenge".to_string(),
            description: "Resolves an HTTP 402 challenge by signing the challenge data with the vault key.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "challenge_data": { "type": "string", "description": "The challenge string issued by the seller." }
                },
                "required": ["challenge_data"]
            }),
        },
    ]
}

// Metadata: [manifest]

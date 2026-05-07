//! @docs ARCHITECTURE:Runner
//!
//! ### Tool Manifest
//! Centralized repository of all core tool definitions.
//! Synchronizes Discovery (Synthesis) and Execution (Dispatcher).

use super::trait_tool::ToolDefinitionData;

pub fn load_core_tool_manifest() -> Vec<ToolDefinitionData> {
    vec![
        // --- Core Operational Tools ---
        ToolDefinitionData {
            name: "spawn_subagent".to_string(),
            description: "Spawns one or more specialized sub-agents to handle tasks in parallel.".to_string(),
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
            description: "Shares a key finding, insight, or data point with the rest of the swarm.".to_string(),
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
            description: "Directly delegates a specific task or instruction to another agent.".to_string(),
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
            description: "Autonomously refactors or updates an existing micro-script (skill).".to_string(),
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
            description: "Updates your persistent structured working memory (scratchpad).".to_string(),
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
        // --- Filesystem Tools ---
        ToolDefinitionData {
            name: "read_file".to_string(),
            description: "Reads content from a file.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "filename": { "type": "string" },
                    "path": { "type": "string" }
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
            description: "Reads a file from the central project codebase.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "filename": { "type": "string" }
                },
                "required": ["filename"]
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
            description: "Executes a terminal command in the mission workspace.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "command": { "type": "string" }
                },
                "required": ["command"]
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
                "required": ["steps"]
            }),
        },
    ]
}

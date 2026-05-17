use std::num::NonZeroUsize;
use lru::LruCache;
use once_cell::sync::Lazy;
use parking_lot::Mutex;
use crate::agent::runner::RunContext;
use crate::state::AppState;
use crate::agent::types::{ToolDefinition, FunctionDeclaration};
use crate::agent::runner::synthesis::fragments::has_file_system_capability;

static TOOL_CACHE: Lazy<Mutex<LruCache<String, ToolDefinition>>> =
    Lazy::new(|| Mutex::new(LruCache::new(NonZeroUsize::new(64).unwrap())));

pub async fn build_tools(ctx: &RunContext, state: &AppState) -> ToolDefinition {
    if !ctx.model_config.supports_native_tools() {
        return ToolDefinition { function_declarations: vec![] };
    }

    let mut sorted_skills = ctx.skills.clone();
    sorted_skills.sort();
    let cache_key = format!("{}:{}:{}", sorted_skills.join(","), ctx.safe_mode, ctx.agent_id);

    {
        let mut cache = TOOL_CACHE.lock();
        if let Some(cached) = cache.get(&cache_key) {
            return cached.clone();
        }
    }

    let mut function_declarations = Vec::new();

    // 1. Inject Registered Tools (Core, Filesystem, Advanced)
    let tool_list = state.registry.tool_registry.list_tools();
    for tool in tool_list {
        if state.resources.acl.is_tool_allowed(&ctx.agent_id, &ctx.role, ctx.authority_level, &tool.name) {
            // Specialized Skill Checks
            if tool.name == "execute_shell" && !(ctx.skills.contains(&"shell".to_string()) || ctx.skills.contains(&"terminal".to_string())) {
                continue;
            }

            // Safe Mode Restrictions (Mutation Block)
            if ctx.safe_mode && (tool.name == "write_file" || tool.name == "delete_file" || tool.name == "execute_shell" || tool.name == "synthesize_micro_script") {
                continue;
            }

            // Filesystem Capability Check
            if (tool.name == "read_file" || tool.name == "write_file" || tool.name == "list_files" || tool.name == "delete_file") && !has_file_system_capability(ctx) {
                continue;
            }

            function_declarations.push(FunctionDeclaration {
                name: tool.name.clone(),
                description: tool.description.clone(),
                parameters: tool.parameters.clone(),
            });
        }
    }

    // 2. Special Utility: Confidence Halting
    function_declarations.push(FunctionDeclaration {
        name: "set_confidence".to_string(),
        description: "Signals your current confidence in the answer. If score >= act_threshold, reasoning halts early.".to_string(),
        parameters: serde_json::json!({
            "type": "object",
            "properties": {
                "score": { "type": "number", "description": "Confidence score between 0.0 and 1.0." }
            },
            "required": ["score"]
        }),
    });

    // 4. Dynamic MCP Tools
    if !ctx.safe_mode {
        let mcp_tools = state.registry.mcp_host.list_tools(&ctx.skills, &state.registry.skills.skills).await;
        for tool in mcp_tools {
            if state.resources.acl.is_tool_allowed(&ctx.agent_id, &ctx.role, ctx.authority_level, &tool.name) {
                function_declarations.push(FunctionDeclaration {
                    name: tool.name,
                    description: tool.description,
                    parameters: tool.input_schema,
                });
            }
        }
    }

    let final_definition = ToolDefinition { function_declarations };
    TOOL_CACHE.lock().put(cache_key, final_definition.clone());
    final_definition
}

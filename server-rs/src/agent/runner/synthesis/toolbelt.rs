//! @docs ARCHITECTURE:Runner
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Agent Runner / toolbelt
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Type-safe state handling and bounded execution without unhandled panics.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: none declared
//! - **Witness Tests**: `toolbelt::tests::*`

use crate::agent::runner::synthesis::fragments::has_file_system_capability;
use crate::agent::runner::RunContext;
use crate::agent::types::{FunctionDeclaration, ToolDefinition};
use crate::state::AppState;
use lru::LruCache;
use once_cell::sync::Lazy;
use parking_lot::Mutex;
use std::num::NonZeroUsize;
use std::sync::atomic::{AtomicU64, Ordering};

static TOOL_CACHE: Lazy<Mutex<LruCache<String, ToolDefinition>>> =
    Lazy::new(|| Mutex::new(LruCache::new(NonZeroUsize::new(64).unwrap())));

static TOOL_REGISTRY_EPOCH: AtomicU64 = AtomicU64::new(1);

/// Increments the tool registry epoch to invalidate stale cached tool definitions across the runner.
pub fn invalidate_tool_cache() {
    TOOL_REGISTRY_EPOCH.fetch_add(1, Ordering::SeqCst);
    TOOL_CACHE.lock().clear();
}

pub async fn build_tools(ctx: &RunContext, state: &AppState) -> ToolDefinition {
    if !ctx.model_config.supports_native_tools() {
        return ToolDefinition {
            function_declarations: vec![],
        };
    }

    let epoch = TOOL_REGISTRY_EPOCH.load(Ordering::Relaxed);
    let mut sorted_skills = ctx.skills.clone();
    sorted_skills.sort();
    let mut sorted_mcp_tools = ctx.mcp_tools.clone();
    sorted_mcp_tools.sort();
    let cache_key = format!(
        "{}:{}:{}:{}:{}",
        epoch,
        sorted_skills.join(","),
        sorted_mcp_tools.join(","),
        ctx.safe_mode,
        ctx.agent_id
    );

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
        if state.resources.acl.is_tool_allowed(
            &ctx.agent_id,
            &ctx.role,
            ctx.authority_level,
            &tool.name,
        ) {
            // Specialized Shell Skill Check
            if tool.name == "execute_shell"
                && !(ctx.skills.contains(&"shell".to_string())
                    || ctx.skills.contains(&"terminal".to_string()))
            {
                continue;
            }

            // Safe Mode Restrictions (Block mutating and workflow tools in conversational mode)
            if ctx.safe_mode
                && (tool.name == "write_file"
                    || tool.name == "delete_file"
                    || tool.name == "execute_shell"
                    || tool.name == "synthesize_micro_script"
                    || tool.name == "spawn_subagent"
                    || tool.name == "send_mission_directive"
                    || tool.name == "complete_mission")
            {
                continue;
            }

            // Filesystem Capability Check
            if (tool.name == "read_file"
                || tool.name == "write_file"
                || tool.name == "list_files"
                || tool.name == "delete_file")
                && !has_file_system_capability(ctx)
            {
                continue;
            }

            function_declarations.push(FunctionDeclaration {
                name: tool.name.clone(),
                description: tool.description.clone(),
                parameters: tool.parameters.clone(),
            });
        }
    }

    // 2. Special Utility: Confidence Halting (only for non-safe autonomous mode)
    if !ctx.safe_mode {
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
    }

    // 3. Dynamic MCP Tools (Safe mode skips external MCP mutations)
    if !ctx.safe_mode {
        let mcp_tools = state
            .registry
            .mcp_host
            .list_tools_for_agent(&ctx.skills, &state.registry.skills.skills, &ctx.mcp_tools)
            .await;
        for tool in mcp_tools {
            if tool.source.starts_with("mcp:")
                && !crate::agent::mcp::is_mcp_tool_authorized(&ctx.mcp_tools, &tool.name)
            {
                continue;
            }
            if state.resources.acl.is_tool_allowed(
                &ctx.agent_id,
                &ctx.role,
                ctx.authority_level,
                &tool.name,
            ) {
                function_declarations.push(FunctionDeclaration {
                    name: tool.name,
                    description: tool.description,
                    parameters: tool.input_schema,
                });
            }
        }
    }

    let final_definition = ToolDefinition {
        function_declarations,
    };
    TOOL_CACHE.lock().put(cache_key, final_definition.clone());
    final_definition
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::AppState;

    #[tokio::test]
    async fn test_build_tools_safe_mode_blocks_mutations() {
        let state = AppState::new_minimal_mock().await;
        let mut ctx = RunContext::default();
        ctx.safe_mode = true;
        ctx.skills = vec!["filesystem".to_string(), "shell".to_string()];

        let tools = build_tools(&ctx, &state).await;
        for decl in &tools.function_declarations {
            assert_ne!(decl.name, "write_file");
            assert_ne!(decl.name, "delete_file");
            assert_ne!(decl.name, "execute_shell");
            assert_ne!(decl.name, "complete_mission");
            assert_ne!(decl.name, "set_confidence");
        }
    }
}

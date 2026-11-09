//! @docs ARCHITECTURE:Runner
//!
//! ### AI Assist Note
//! - Grouped execution context owns identity, mission state and environment; retain scope across callers.
//! - Advertise static tool permissions only when the execution capability would grant them.
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

use crate::agent::runner::synthesis::fragments::has_file_system_capability;
use crate::agent::runner::tools::{
    capability::Permission, policy::resolve_required_permission, ZeroTrustGuard,
};
use crate::agent::runner::RunContext;
use crate::agent::types::{FunctionDeclaration, ToolDefinition};
use crate::state::AppState;
use lru::LruCache;
use once_cell::sync::Lazy;
use parking_lot::Mutex;
use std::num::NonZeroUsize;
use std::sync::atomic::{AtomicU64, Ordering};

static TOOL_CACHE: Lazy<Mutex<LruCache<String, ToolDefinition>>> = Lazy::new(|| {
    Mutex::new(LruCache::new(
        NonZeroUsize::new(64).expect("64 is non-zero"),
    ))
});

static TOOL_REGISTRY_EPOCH: AtomicU64 = AtomicU64::new(1);

/// Increments the tool registry epoch to invalidate stale cached tool definitions across the runner.
pub fn invalidate_tool_cache() {
    TOOL_REGISTRY_EPOCH.fetch_add(1, Ordering::SeqCst);
    TOOL_CACHE.lock().clear();
}

fn build_tool_cache_key(ctx: &RunContext, state: &AppState) -> String {
    let epoch = TOOL_REGISTRY_EPOCH.load(Ordering::Relaxed);
    let dynamic_count = state
        .registry
        .tool_registry
        .dynamic_handlers
        .read()
        .map(|h| h.len())
        .unwrap_or(0);
    let mut sorted_skills = ctx.skills.clone();
    sorted_skills.sort();
    let mut sorted_mcp_tools = ctx.mcp_tools.clone();
    sorted_mcp_tools.sort();
    format!(
        "{}:{}:{}:{}:{}:{}:{}:{:?}",
        epoch,
        dynamic_count,
        sorted_skills.join(","),
        sorted_mcp_tools.join(","),
        ctx.safe_mode,
        ctx.identity.agent_id,
        ctx.identity.role,
        ctx.identity.authority_level
    )
}

fn is_blocked_in_safe_mode(tool_name: &str) -> bool {
    matches!(
        tool_name,
        "write_file"
            | "delete_file"
            | "execute_shell"
            | "synthesize_micro_script"
            | "spawn_subagent"
            | "send_mission_directive"
            | "complete_mission"
    )
}

fn requires_filesystem_skill(tool_name: &str) -> bool {
    matches!(
        tool_name,
        "read_file" | "write_file" | "list_files" | "delete_file"
    )
}

fn verify_static_tool_permission(
    ctx: &RunContext,
    token: &crate::agent::runner::tools::capability::CapabilityToken,
    tool_name: &str,
    is_mutating: bool,
) -> bool {
    let permission = resolve_required_permission(
        tool_name,
        &serde_json::json!({}),
        &ctx.env.workspace_root,
        is_mutating,
    );
    match permission {
        Ok(
            required @ (Permission::ToolExec(_) | Permission::SpawnAgent | Permission::ModelSwitch),
        ) => token.verify(&required),
        Ok(_) => true,
        Err(err) => {
            tracing::warn!(
                tool = %tool_name,
                error = %err,
                "Excluding tool declaration due to permission resolution error"
            );
            false
        }
    }
}

fn is_registered_tool_eligible(
    ctx: &RunContext,
    state: &AppState,
    token: &crate::agent::runner::tools::capability::CapabilityToken,
    tool: &crate::agent::runner::tools::trait_tool::ToolDefinitionData,
) -> bool {
    if !state.resources.acl.is_tool_allowed(
        &ctx.identity.agent_id,
        &ctx.identity.role,
        ctx.identity.authority_level,
        &tool.name,
    ) {
        return false;
    }
    if !verify_static_tool_permission(ctx, token, &tool.name, tool.is_mutating) {
        return false;
    }
    if tool.name == "execute_shell" && !(ctx.skills.iter().any(|s| s == "shell" || s == "terminal"))
    {
        return false;
    }
    if ctx.safe_mode && is_blocked_in_safe_mode(&tool.name) {
        return false;
    }
    if requires_filesystem_skill(&tool.name) && !has_file_system_capability(ctx) {
        return false;
    }
    true
}

fn confidence_tool_declaration() -> FunctionDeclaration {
    FunctionDeclaration {
        name: "set_confidence".to_string(),
        description: "Signals your current confidence in the answer. If score >= act_threshold, reasoning halts early.".to_string(),
        parameters: serde_json::json!({
            "type": "object",
            "properties": {
                "score": { "type": "number", "description": "Confidence score between 0.0 and 1.0." }
            },
            "required": ["score"]
        }),
    }
}

async fn append_mcp_declarations(
    ctx: &RunContext,
    state: &AppState,
    declarations: &mut Vec<FunctionDeclaration>,
) {
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
            &ctx.identity.agent_id,
            &ctx.identity.role,
            ctx.identity.authority_level,
            &tool.name,
        ) {
            declarations.push(FunctionDeclaration {
                name: tool.name,
                description: tool.description,
                parameters: tool.input_schema,
            });
        }
    }
}

pub async fn build_tools(ctx: &RunContext, state: &AppState) -> ToolDefinition {
    if ctx.identity.is_plane2() || !ctx.model_config.supports_native_tools() {
        return ToolDefinition {
            function_declarations: vec![],
        };
    }

    let cache_key = build_tool_cache_key(ctx, state);
    if let Some(cached) = TOOL_CACHE.lock().get(&cache_key).cloned() {
        return cached;
    }

    let token = ZeroTrustGuard::mint_token(
        &ctx.identity.agent_id,
        &ctx.mission_state.mission_id,
        ctx.identity.authority_level,
        ctx.allowed_files.as_deref(),
        Some(&ctx.skills),
        &ctx.env.workspace_root,
    );

    let mut function_declarations: Vec<FunctionDeclaration> = state
        .registry
        .tool_registry
        .list_tools()
        .into_iter()
        .filter(|tool| is_registered_tool_eligible(ctx, state, &token, tool))
        .map(|tool| FunctionDeclaration {
            name: tool.name,
            description: tool.description,
            parameters: tool.parameters,
        })
        .collect();

    if !ctx.safe_mode {
        function_declarations.push(confidence_tool_declaration());
        append_mcp_declarations(ctx, state, &mut function_declarations).await;
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

    #[tokio::test]
    async fn test_build_tools_advertises_archive_to_vault_for_specialist() {
        let state = AppState::new_minimal_mock().await;
        let mut ctx = RunContext::default();
        ctx.safe_mode = false;
        ctx.identity.authority_level = crate::agent::types::RoleAuthorityLevel::Specialist;
        ctx.skills = vec!["filesystem".to_string()];

        let tools = build_tools(&ctx, &state).await;
        let names: Vec<_> = tools
            .function_declarations
            .iter()
            .map(|d| d.name.as_str())
            .collect();
        assert!(
            names.contains(&"archive_to_vault"),
            "archive_to_vault must be advertised to specialists"
        );
    }

    #[tokio::test]
    async fn test_build_tools_hides_ungranted_script_builder() {
        let state = AppState::new_minimal_mock().await;
        let mut ctx = RunContext::default();
        ctx.identity.agent_id = "toolbelt-audit-worker".into();
        ctx.identity.authority_level = crate::agent::types::RoleAuthorityLevel::Specialist;
        ctx.skills = vec!["filesystem".into()];
        let tools = build_tools(&ctx, &state).await;
        assert!(tools
            .function_declarations
            .iter()
            .any(|d| d.name == "read_codebase_file"));
        assert!(!tools
            .function_declarations
            .iter()
            .any(|d| d.name == "script_builder"));

        ctx.skills.push("script_builder".into());
        let tools = build_tools(&ctx, &state).await;
        assert!(tools
            .function_declarations
            .iter()
            .any(|d| d.name == "script_builder"));
        ctx.skills.pop();
        let tools = build_tools(&ctx, &state).await;
        assert!(!tools
            .function_declarations
            .iter()
            .any(|d| d.name == "script_builder"));
    }
}

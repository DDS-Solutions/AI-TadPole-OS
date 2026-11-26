//! @docs ARCHITECTURE:State
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / State / Init Registry
//! - **Primary Entrypoints**: `build_registry_hub`, `spawn_skill_hot_reloader`
//!
//! ### 🔍 Debugging & Observability
//! - **Telemetry Targets**: `[Registries]`, `[Registry]`, `[Evolution]`

use crate::agent::types::{AgentHealth, AgentIdentity, AgentState, EngineAgent};
use crate::error::AppError;
use crate::state::hubs::comm::CommunicationHub;
use crate::state::hubs::reg::RegistryHub;
use dashmap::DashMap;
use sqlx::SqlitePool;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::broadcast;

/// Hydrates providers, models, agents, script skills, MCP host, tool plugins, and capability map.
pub async fn build_registry_hub(
    base_dir: &Path,
    pool: &SqlitePool,
    comms: &Arc<CommunicationHub>,
    event_tx: &broadcast::Sender<serde_json::Value>,
    permission_policy: Arc<crate::security::permissions::PermissionPolicy>,
) -> Result<Arc<RegistryHub>, AppError> {
    let (providers, models) = load_providers_and_models(base_dir).await;
    let agents = load_or_seed_agents(pool).await;

    tracing::info!("🛰️ [Registry] Initializing Script Skills Registry...");
    let raw_skills = crate::agent::script_skills::ScriptSkillsRegistry::new()
        .await
        .map_err(|e| {
            AppError::InternalServerError(format!(
                "Failed to initialize script skills registry: {e}"
            ))
        })?;
    let script_skills = Arc::new(raw_skills);

    tracing::info!("🛰️ [Registry] Loading Skill Manifests...");
    let skill_registry = Arc::new(crate::agent::skill_manifest::SkillRegistry::load_all());

    let mcp_config_opt = Some(base_dir.join(".agent").join("mcp_config.json"));
    tracing::info!(
        "🛰️ [Registry] Initializing MCP Host (Config: {:?})...",
        mcp_config_opt
    );
    let mcp_prompter = Arc::new(crate::security::permissions::AsyncOversightPrompter::new(
        pool.clone(),
        comms.clone(),
        Some(event_tx.clone()),
    ));
    let mcp_host = Arc::new(
        crate::agent::mcp::McpHost::new(event_tx.clone(), mcp_config_opt, permission_policy)
            .with_prompter(mcp_prompter),
    );

    let tool_registry = init_tool_registry(base_dir).await;
    let capability_map = build_capability_map(&agents);

    Ok(Arc::new(RegistryHub {
        agents,
        providers,
        provider_health: DashMap::new(),
        provider_failures: DashMap::new(),
        models,
        nodes: DashMap::new(),
        skills: script_skills,
        skill_registry,
        mcp_host,
        tool_registry,
        mission_backlogs: DashMap::new(),
        capability_map,
    }))
}

async fn load_providers_and_models(
    base_dir: &Path,
) -> (
    DashMap<String, crate::agent::types::ProviderConfig>,
    DashMap<String, crate::agent::types::ModelEntry>,
) {
    tracing::info!("📂 [Registries] Loading Providers and Models...");
    let providers = DashMap::new();
    for p in crate::agent::persistence::load_providers(base_dir).await {
        providers.insert(p.id.clone(), p);
    }

    let models = DashMap::new();
    for m in crate::agent::persistence::load_models(base_dir).await {
        models.insert(m.id.clone(), m);
    }
    (providers, models)
}

async fn load_or_seed_agents(pool: &SqlitePool) -> DashMap<String, EngineAgent> {
    tracing::info!("📂 [Registries] Loading Agents...");
    let agents_list = crate::agent::persistence::load_agents_db(pool)
        .await
        .unwrap_or_default();

    let agents = DashMap::new();
    for a in agents_list {
        agents.insert(a.identity.id.clone(), a);
    }

    let is_demo_mode = std::env::var("TADPOLE_DEMO_MODE")
        .map(|v| v == "true" || v == "1")
        .unwrap_or(false);
    if agents.is_empty() && is_demo_mode {
        for a in build_demo_agents() {
            agents.insert(a.identity.id.clone(), a);
        }
    }

    tracing::info!("✅ [Registries] Agents loaded (count: {}).", agents.len());
    agents
}

struct DemoAgentSpec<'a> {
    id: &'a str,
    name: &'a str,
    role: &'a str,
    department: &'a str,
    description: &'a str,
    category: &'a str,
    status: &'a str,
    turn: u32,
}

fn demo_agent(spec: DemoAgentSpec<'_>) -> EngineAgent {
    EngineAgent {
        identity: AgentIdentity {
            id: spec.id.to_string(),
            name: spec.name.to_string(),
            role: spec.role.to_string(),
            department: spec.department.to_string(),
            description: spec.description.to_string(),
            category: spec.category.to_string(),
            theme_color: None,
        },
        health: AgentHealth {
            status: spec.status.to_string(),
            ..Default::default()
        },
        state: AgentState {
            current_reasoning_turn: spec.turn,
            current_task: Some(spec.description.to_string()),
            ..Default::default()
        },
        ..Default::default()
    }
}

fn build_demo_agents() -> Vec<EngineAgent> {
    vec![
        demo_agent(DemoAgentSpec {
            id: "ag-1",
            name: "Master Swarm Router",
            role: "Router",
            department: "Core Subsystem",
            description: "Monitoring IPC channels & zero-trust bridge",
            category: "router",
            status: "RUNNING",
            turn: 142,
        }),
        demo_agent(DemoAgentSpec {
            id: "ag-2",
            name: "Continuity Scheduler",
            role: "Scheduler",
            department: "Automation",
            description: "Polling cron queue & state checkpoints",
            category: "scheduler",
            status: "RUNNING",
            turn: 89,
        }),
        demo_agent(DemoAgentSpec {
            id: "ag-3",
            name: "Vector RAG Indexer",
            role: "Search",
            department: "Knowledge",
            description: "Awaiting query embedding",
            category: "rag",
            status: "IDLE",
            turn: 450,
        }),
        demo_agent(DemoAgentSpec {
            id: "ag-4",
            name: "Oversight Ledger Gate",
            role: "Security",
            department: "Governance",
            description: "Listening for HITL signals from Android app",
            category: "oversight",
            status: "RUNNING",
            turn: 12,
        }),
    ]
}

async fn init_tool_registry(
    base_dir: &Path,
) -> Arc<crate::agent::runner::tools::registry::ToolRegistry> {
    tracing::info!("🛰️ [Registry] Initializing Tool Registry...");
    let dispatcher = crate::agent::runner::tools::dispatcher::Dispatcher::new();
    let tool_registry = Arc::new(dispatcher.registry);
    let plugins_dir = base_dir.join("plugins");
    let plugins = crate::agent::runner::tools::plugin::load_dynamic_plugins(&plugins_dir).await;
    tool_registry.reload_plugins(plugins);
    tool_registry
}

fn build_capability_map(
    agents: &DashMap<String, EngineAgent>,
) -> Arc<parking_lot::RwLock<crate::agent::capability_map::CapabilityMap>> {
    let mut cap_map = crate::agent::capability_map::CapabilityMap::new();
    for kv in agents.iter() {
        cap_map.update_agent(kv.value());
    }
    Arc::new(parking_lot::RwLock::new(cap_map))
}

/// Passive hot-reloading loop that watches workspace skills and dynamic tool plugins every 10s.
pub fn spawn_skill_hot_reloader(registry: Arc<RegistryHub>, base_dir: PathBuf) {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(10));
        loop {
            interval.tick().await;
            if let Err(e) = registry.skills.reload_all().await {
                tracing::error!("🚨 [Evolution] Passive hot-reload failure: {:?}", e);
            }
            let plugins_dir = base_dir.join("plugins");
            let plugins =
                crate::agent::runner::tools::plugin::load_dynamic_plugins(&plugins_dir).await;
            registry.tool_registry.reload_plugins(plugins);
        }
    });
}

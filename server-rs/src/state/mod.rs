//! Global Application State & Thread-Safe Context - The Sovereign State
//!
//! The `AppState` acts as the single source of truth for the swarm, managing
//! authenticated session maps, database pools, and real-time telemetry hubs.
//!
//! @docs ARCHITECTURE:State
//! @docs OPERATIONS_MANUAL:Governance
//!
//! ### AI Assist Note
//! **The Sovereign State**: Acts as the single source of truth for the swarm.
//! Manages the **Telemetry Hub**, **Agent Registry**, **Governance
//! Policy**, and **Resource Pool**. All asynchronous workers MUST hold
//! an `Arc<AppState>` to remain synchronized with the global swarm state.
//! Features a **Multi-Hub Architecture** to isolate concerns across
//! communication, security, and persistence.
//!
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: Double-locking `parking_lot::RwLock` in nested
//!   callbacks, DB pool exhaustion during high-concurrency bursts, or
//!   state corruption due to out-of-order event broadcasts.
//! - **Trace Scope**: `server-rs::state` (Search for `[Engine]` or `[State]` tags)

use anyhow::Context;
use dashmap::DashMap;
use parking_lot::RwLock;
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicUsize};
use std::sync::Arc;
use tokio::sync::{broadcast, OnceCell};

pub mod hubs;
pub mod mock;

use hubs::comm::CommunicationHub;
use hubs::gov::GovernanceHub;
use hubs::reg::RegistryHub;
use hubs::res::ResourceHub;
use hubs::sec::SecurityHub;

use crate::error::AppError;
use crate::types::SubsystemStatus;

/// The global application state shared across all routes via Axum State.
/// Decomposed into logical hubs for modularity.
pub struct AppState {
    /// Manages real-time communication channels (logs, events, telemetry, audio).
    pub comms: Arc<CommunicationHub>,
    /// Manages operational limits and policy settings.
    pub governance: Arc<GovernanceHub>,
    /// Manages entities like agents, providers, models, and skills.
    pub registry: Arc<RegistryHub>,
    /// Manages security features like auditing, budget enforcement, and scanning.
    pub security: Arc<SecurityHub>,
    /// Manages shared system resources (DB pool, HTTP client, file contexts).
    pub resources: Arc<ResourceHub>,
    /// Whether mirror mode is active (read from environment/config).
    pub mirror_mode: bool,
    /// Active drift alerts observed in mirror mode.
    pub drift_alerts: Arc<dashmap::DashMap<String, serde_json::Value>>,
    /// Global workspace root directory for data persistence.
    pub base_dir: std::path::PathBuf,
    /// Unique session ID generated at boot, used to correlate engine:boot / engine:shutdown events.
    pub session_id: String,
}


impl AppState {
    /// ### 🏁 Boot Sequence: Engine Initialization (new)
    /// Performs the synchronous and asynchronous orchestration required to bring
    /// the Tadpole OS engine online.
    ///
    /// ### 🧬 Initialization Stages
    /// 1. **Secret Loading**: Verifies the existence of `NEURAL_TOKEN`.
    /// 2. **Database Link**: Establishes the persistent SQLite connection pool.
    /// 3. **Hydration**: Rapidly loads providers, models, and agents from SQLite
    ///    into highly-concurrent `DashMap` registries.
    /// 4. **Capability Discovery**: Scans for dynamic Python/JS skills and
    ///    markdown workflows.
    /// 5. **Subsystem Assembly**: Initializes the `McpHost`, `BunkerCache`,
    ///    and `SecretRedactor`.
    pub async fn new() -> Result<Self, AppError> {
        dotenvy::dotenv().ok();

        let (tx, _) = broadcast::channel(1000);
        let (event_tx, _) = broadcast::channel(1000);
        let (audio_stream_tx, _) = broadcast::channel(5000);
        let (pulse_tx, _) = broadcast::channel(1000);
        let telemetry_tx = crate::telemetry::TELEMETRY_TX.clone();

        let base_dir = std::env::var("WORKSPACE_ROOT")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|_| {
                let current = std::env::current_dir().unwrap_or_default();
                if current.ends_with("server-rs") {
                    current.parent().unwrap_or(&current).to_path_buf()
                } else {
                    current
                }
            });

        tracing::info!("🏁 [Engine] Starting AppState initialization...");

        // Security: Load Neural Token (Mandatory, but relaxed for tests)
        tracing::info!("🔑 [Auth] Loading Neural Token...");
        let deploy_token = match std::env::var("NEURAL_ENGINE_ACCESS_TOKEN")
            .or_else(|_| std::env::var("NEURAL_TOKEN"))
        {
            Ok(token) => token.trim().to_string(),
            Err(_) if cfg!(test) => "ci-test-token-placeholder".to_string(),
            Err(_) => return Err(AppError::Unauthorized(
                "🚨 FATAL: NEURAL_TOKEN or NEURAL_ENGINE_ACCESS_TOKEN environment variable MUST be set for the engine to start.".to_string()
            )),
        };
        let is_production = crate::utils::security::is_production_env();
        let admin_token = match std::env::var("NEURAL_ADMIN_TOKEN")
            .or_else(|_| std::env::var("ADMIN_TOKEN"))
        {
            Ok(token) => token.trim().to_string(),
            Err(_) if cfg!(test) => "ci-test-admin-token-placeholder".to_string(),
            Err(_) if is_production => {
                return Err(AppError::Unauthorized(
                    "🚨 FATAL: NEURAL_ADMIN_TOKEN or ADMIN_TOKEN environment variable MUST be set in production."
                        .to_string(),
                ));
            }
            Err(_) => {
                tracing::warn!(
                    "⚠️ NEURAL_ADMIN_TOKEN is not configured. Falling back to NEURAL_TOKEN for local development only."
                );
                deploy_token.clone()
            }
        };
        if admin_token.is_empty() {
            return Err(AppError::Unauthorized(
                "🚨 FATAL: administrative token cannot be empty.".to_string(),
            ));
        }
        if is_production && admin_token == deploy_token {
            return Err(AppError::Unauthorized(
                "🚨 FATAL: NEURAL_ADMIN_TOKEN must differ from NEURAL_TOKEN in production."
                    .to_string(),
            ));
        }

        // Initialize DB
        let database_url = if cfg!(test) {
            "sqlite::memory:".to_string()
        } else {
            std::env::var("DATABASE_URL").unwrap_or_else(|_| {
                let db_path = base_dir.join("data").join("tadpole.db");
                format!("sqlite:{}", db_path.display())
            })
        };

        tracing::info!("🗄️ [Database] Connecting to: {}", database_url);
        let pool = match crate::db::init_db(&database_url).await {
            Ok(p) => {
                tracing::info!("✅ [Database] Pool established successfully.");
                p
            }
            Err(e) => {
                tracing::error!(
                    "🚨 [Database] FATAL: Failed to initialize database pool at {}: {:?}",
                    database_url,
                    e
                );
                return Err(AppError::from(e));
            }
        };

        // Load Registries
        tracing::info!("📂 [Registries] Loading Providers and Models...");
        let providers_list = crate::agent::persistence::load_providers(&base_dir).await;
        let providers = DashMap::new();
        for p in providers_list {
            providers.insert(p.id.clone(), p);
        }

        let models_list = crate::agent::persistence::load_models(&base_dir).await;
        let models = DashMap::new();
        for m in models_list {
            models.insert(m.id.clone(), m);
        }

        tracing::info!("📂 [Registries] Loading Agents...");
        let agents_list = crate::agent::persistence::load_agents_db(&pool)
            .await
            .unwrap_or_default();

        let agents = DashMap::new();
        for a in agents_list {
            agents.insert(a.identity.id.clone(), a);
        }
        if agents.is_empty() {
            let default_agents = vec![
                crate::agent::types::EngineAgent {
                    identity: crate::agent::types::AgentIdentity {
                        id: "ag-1".to_string(),
                        name: "Master Swarm Router".to_string(),
                        role: "Router".to_string(),
                        department: "Core Subsystem".to_string(),
                        description: "Monitoring IPC channels & zero-trust bridge".to_string(),
                        category: "router".to_string(),
                        theme_color: None,
                    },
                    health: crate::agent::types::AgentHealth {
                        status: "RUNNING".to_string(),
                        ..Default::default()
                    },
                    state: crate::agent::types::AgentState {
                        current_reasoning_turn: 142,
                        current_task: Some("Monitoring IPC channels & zero-trust bridge".to_string()),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                crate::agent::types::EngineAgent {
                    identity: crate::agent::types::AgentIdentity {
                        id: "ag-2".to_string(),
                        name: "Continuity Scheduler".to_string(),
                        role: "Scheduler".to_string(),
                        department: "Automation".to_string(),
                        description: "Polling cron queue & state checkpoints".to_string(),
                        category: "scheduler".to_string(),
                        theme_color: None,
                    },
                    health: crate::agent::types::AgentHealth {
                        status: "RUNNING".to_string(),
                        ..Default::default()
                    },
                    state: crate::agent::types::AgentState {
                        current_reasoning_turn: 89,
                        current_task: Some("Polling cron queue & state checkpoints".to_string()),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                crate::agent::types::EngineAgent {
                    identity: crate::agent::types::AgentIdentity {
                        id: "ag-3".to_string(),
                        name: "Vector RAG Indexer".to_string(),
                        role: "Search".to_string(),
                        department: "Knowledge".to_string(),
                        description: "Awaiting query embedding".to_string(),
                        category: "rag".to_string(),
                        theme_color: None,
                    },
                    health: crate::agent::types::AgentHealth {
                        status: "IDLE".to_string(),
                        ..Default::default()
                    },
                    state: crate::agent::types::AgentState {
                        current_reasoning_turn: 450,
                        current_task: Some("Awaiting query embedding".to_string()),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                crate::agent::types::EngineAgent {
                    identity: crate::agent::types::AgentIdentity {
                        id: "ag-4".to_string(),
                        name: "Oversight Ledger Gate".to_string(),
                        role: "Security".to_string(),
                        department: "Governance".to_string(),
                        description: "Listening for HITL signals from Android app".to_string(),
                        category: "oversight".to_string(),
                        theme_color: None,
                    },
                    health: crate::agent::types::AgentHealth {
                        status: "RUNNING".to_string(),
                        ..Default::default()
                    },
                    state: crate::agent::types::AgentState {
                        current_reasoning_turn: 12,
                        current_task: Some("Listening for HITL signals from Android app".to_string()),
                        ..Default::default()
                    },
                    ..Default::default()
                },
            ];
            for a in default_agents {
                agents.insert(a.identity.id.clone(), a);
            }
        }
        tracing::info!("✅ [Registries] Agents loaded (count: {}).", agents.len());

        tracing::info!("🚀 [Engines] Initializing HTTP Client...");
        let http_client = Arc::new(
            reqwest::Client::builder()
                .user_agent("TadpoleOS/1.1.352")
                .pool_max_idle_per_host(100) // 🛡️ [Hardening] Increase pool for 10+ agent swarms
                .pool_idle_timeout(std::time::Duration::from_secs(60))
                .connect_timeout(std::time::Duration::from_secs(10))
                .timeout(std::time::Duration::from_secs(900))
                .tcp_nodelay(true)
                .tcp_keepalive(Some(std::time::Duration::from_secs(60))) // 🛡️ [Hardening] Prevent stale drops
                .build()
                .context("Failed to build HTTP client")?,
        );
        let audio_cache_path = base_dir.join("data").join("audio_cache.db");
        tracing::info!(
            "🚀 [Engines] Initializing Audio Cache at {}...",
            audio_cache_path.display()
        );
        let audio_cache = match crate::agent::audio_cache::BunkerCache::new(
            audio_cache_path.clone(),
        )
        .await
        {
            Ok(cache) => Arc::new(cache),
            Err(e) => {
                tracing::warn!("⚠️ [Engines] Audio Cache failed to initialize at {}: {:?}. Falling back to no-op mode.", audio_cache_path.display(), e);
                Arc::new(crate::agent::audio_cache::BunkerCache::new_noop().await)
            }
        };

        let secret_redactor = Arc::new(crate::secret_redactor::SecretRedactor::from_env());

        // Assemble Hubs
        tracing::info!("💠 [Hubs] Assembling Communication Hub...");
        let comms = Arc::new(CommunicationHub {
            tx: tx.clone(),
            event_tx: event_tx.clone(),
            telemetry_tx,
            audio_stream_tx,
            pulse_tx,
            oversight_queue: DashMap::new(),
            oversight_resolvers: DashMap::new(),
            active_runners: DashMap::new(),
            recent_requests: DashMap::new(),
            runner_semaphore: tokio::sync::Semaphore::new(
                std::env::var("MAX_CONCURRENT_RUNNERS")
                    .ok()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(20),
            ),
            event_sequence: std::sync::atomic::AtomicU64::new(0),
        });

        tracing::info!("💠 [Hubs] Assembling Governance Hub...");
        let governance = Arc::new(GovernanceHub {
            auto_approve_safe_skills: AtomicBool::new(
                std::env::var("AUTO_APPROVE_SAFE_SKILLS")
                    .map(|s| s == "true")
                    .unwrap_or(true),
            ),
            max_agents: AtomicU32::new(
                std::env::var("MAX_AGENTS")
                    .map(|s| s.parse().unwrap_or(50))
                    .unwrap_or(50),
            ),
            max_clusters: AtomicU32::new(
                std::env::var("MAX_CLUSTERS")
                    .map(|s| s.parse().unwrap_or(10))
                    .unwrap_or(10),
            ),
            max_swarm_depth: AtomicU32::new(
                std::env::var("MAX_SWARM_DEPTH")
                    .map(|s| s.parse().unwrap_or(5))
                    .unwrap_or(5),
            ),
            max_task_length: AtomicUsize::new(
                std::env::var("MAX_TASK_LENGTH")
                    .map(|s| s.parse().unwrap_or(32768))
                    .unwrap_or(32768),
            ),
            default_budget_usd: RwLock::new(
                std::env::var("DEFAULT_AGENT_BUDGET_USD")
                    .map(|s| s.parse().unwrap_or(1.0))
                    .unwrap_or(1.0),
            ),
            default_model: RwLock::new(
                std::env::var("DEFAULT_INTELLIGENCE_MODEL")
                    .unwrap_or_else(|_| "gemini-1.5-pro".to_string()),
            ),
            default_provider: RwLock::new(
                std::env::var("DEFAULT_PROVIDER")
                    .unwrap_or_else(|_| "google".to_string()),
            ),
            active_agents: AtomicU32::new(0),
            recruit_count: AtomicU32::new(0),
            tpm_accumulator: AtomicUsize::new(0),
            privacy_mode: AtomicBool::new(
                std::env::var("PRIVACY_MODE")
                    .map(|s| s.to_lowercase() == "true")
                    .unwrap_or(false),
            ),
            failover_amber_threshold: AtomicU32::new(
                std::env::var("FAILOVER_AMBER_THRESHOLD")
                    .map(|s| s.parse().unwrap_or(3))
                    .unwrap_or(3),
            ),
            failover_red_threshold: AtomicU32::new(
                std::env::var("FAILOVER_RED_THRESHOLD")
                    .map(|s| s.parse().unwrap_or(5))
                    .unwrap_or(5),
            ),
            failover_max_attempts: AtomicU32::new(
                std::env::var("FAILOVER_MAX_ATTEMPTS")
                    .map(|s| s.parse().unwrap_or(3))
                    .unwrap_or(3),
            ),
            provider_timeout_secs: AtomicU32::new(
                std::env::var("PROVIDER_TIMEOUT_SECS")
                    .ok()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(60),
            ),
            null_providers_test_mode: AtomicBool::new(
                std::env::var("TADPOLE_NULL_PROVIDERS")
                    .map(|s| s == "true")
                    .unwrap_or(false),
            ),
            deprecated_routes: RwLock::new({
                let mut m = std::collections::HashMap::new();
                m.insert(
                    "/infra/providers".to_string(),
                    (
                        "Fri, 01 Jan 2027 23:59:59 GMT".to_string(),
                        "<https://docs.tadpole.so/api/v2/providers>; rel=\"alternate\"".to_string(),
                    ),
                );
                m
            }),
            cluster_privacy_policies: dashmap::DashMap::new(),
        });

        // 🛡️ [Governance] Hydrate from persisted settings if present
        let settings_path = base_dir.join("data").join("governance_settings.json");
        if settings_path.exists() {
            if let Ok(data) = std::fs::read_to_string(&settings_path) {
                if let Ok(persisted) = serde_json::from_str::<
                    crate::routes::oversight::OversightSettingsPayload,
                >(&data)
                {
                    tracing::info!(
                        "🛡️ [Governance] Found persisted settings, applying overrides..."
                    );
                    governance.auto_approve_safe_skills.store(
                        persisted.auto_approve_safe_skills,
                        std::sync::atomic::Ordering::Relaxed,
                    );
                    if let Some(val) = persisted.privacy_mode {
                        governance
                            .privacy_mode
                            .store(val, std::sync::atomic::Ordering::Relaxed);
                    }
                    if let Some(ref policies) = persisted.cluster_privacy_policies {
                        for (cid, mode) in policies {
                            governance.cluster_privacy_policies.insert(cid.clone(), *mode);
                        }
                    }
                    if let Some(val) = persisted.max_agents {
                        governance
                            .max_agents
                            .store(val, std::sync::atomic::Ordering::Relaxed);
                    }
                    if let Some(val) = persisted.max_clusters {
                        governance
                            .max_clusters
                            .store(val, std::sync::atomic::Ordering::Relaxed);
                    }
                    if let Some(val) = persisted.max_swarm_depth {
                        governance
                            .max_swarm_depth
                            .store(val, std::sync::atomic::Ordering::Relaxed);
                    }
                    if let Some(val) = persisted.max_task_length {
                        governance
                            .max_task_length
                            .store(val, std::sync::atomic::Ordering::Relaxed);
                    }
                    if let Some(val) = persisted.default_budget_usd {
                        *governance.default_budget_usd.write() = val;
                    }
                    if let Some(val) = persisted.default_model {
                        *governance.default_model.write() = val;
                    }
                    if let Some(val) = persisted.default_provider {
                        *governance.default_provider.write() = val;
                    }
                }
            }
        }

        let mcp_config_path = base_dir.join(".agent").join("mcp_config.json");
        let mcp_config_opt = if mcp_config_path.exists() {
            Some(mcp_config_path)
        } else {
            None
        };

        tracing::info!("🛰️ [Registry] Initializing Script Skills Registry...");
        let script_skills = Arc::new(
            crate::agent::script_skills::ScriptSkillsRegistry::new()
                .await
                .map_err(|e| {
                    AppError::InternalServerError(format!(
                        "Failed to initialize script skills registry: {}",
                        e
                    ))
                })?,
        );

        tracing::info!("🛰️ [Registry] Loading Skill Manifests...");
        let skill_registry = Arc::new(crate::agent::skill_manifest::SkillRegistry::load_all());

        let permission_policy = Arc::new(crate::security::permissions::PermissionPolicy::new(
            pool.clone(),
        ));

        tracing::info!(
            "🛰️ [Registry] Initializing MCP Host (Config: {:?})...",
            mcp_config_opt
        );
        let mcp_host = Arc::new(crate::agent::mcp::McpHost::new(
            event_tx.clone(),
            mcp_config_opt,
            permission_policy.clone(),
        ));

        tracing::info!("🛰️ [Registry] Initializing Hooks Manager...");
        let hooks = Arc::new(crate::agent::hooks::HooksManager::new(
            std::path::Path::new("data"),
        ));

        tracing::info!("🛰️ [Registry] Initializing Tool Registry...");
        let dispatcher = crate::agent::runner::tools::dispatcher::Dispatcher::new();
        let tool_registry = Arc::new(dispatcher.registry);

        // Load dynamic plugins at startup
        let plugins_dir = base_dir.join("plugins");
        let plugins = crate::agent::runner::tools::plugin::load_dynamic_plugins(&plugins_dir).await;
        tool_registry.reload_plugins(plugins);

        let registry = Arc::new(RegistryHub {
            agents: agents.clone(),
            providers,
            provider_health: DashMap::new(),
            provider_failures: DashMap::new(),
            models,
            nodes: DashMap::new(),
            skills: script_skills,
            skill_registry,
            mcp_host,
            hooks,
            tool_registry,
            mission_backlogs: DashMap::new(),
        });

        let system_monitor = Arc::new(crate::security::monitoring::SecurityMonitor::new());
        let budget_guard = Arc::new(crate::security::metering::BudgetGuard::new(pool.clone(), system_monitor.clone()));

        let security = Arc::new(SecurityHub {
            audit_trail: Arc::new(
                crate::security::audit::MerkleAuditTrail::new(pool.clone())
                    .expect("Failed to initialize audit trail"),
            ),
            budget_guard,
            shell_scanner: Arc::new(crate::security::scanner::ShellScanner::new(
                secret_redactor.clone(),
            )),
            secret_redactor,
            system_monitor,
            permission_policy,
            deploy_token,
            admin_token,
            // C-03: Load pinned oversight public key once at startup.
            // In production, this should be set via OVERSIGHT_PUBLIC_KEY env var.
            oversight_public_key: {
                let key = std::env::var("OVERSIGHT_PUBLIC_KEY").ok();
                if let Some(ref k) = key {
                    let fingerprint = if k.len() >= 8 { &k[..8] } else { k };
                    tracing::info!(
                        "🔑 [Security] Oversight public key pinned (fingerprint: {}...)",
                        fingerprint
                    );
                } else {
                    let is_production = std::env::var("TADPOLE_ENV")
                        .or_else(|_| std::env::var("ENV"))
                        .map(|v| v.eq_ignore_ascii_case("production"))
                        .unwrap_or(false);
                    if is_production {
                        tracing::error!("🚨 SECURITY: OVERSIGHT_PUBLIC_KEY is not set in production! Oversight decisions will be rejected until a pinned key is configured.");
                    } else {
                        tracing::warn!("⚠️ OVERSIGHT_PUBLIC_KEY is not configured. Oversight signatures are verified but NOT pinned to an authorized operator.");
                    }
                }
                key
            },
        });

        let mirror_mode = std::env::var("MIRROR_MODE")
            .map(|s| s.to_lowercase() == "true")
            .unwrap_or(false);

        let state = Self {
            comms,
            governance,
            registry,
            security,
            resources: Arc::new(ResourceHub {
                pool: pool.clone(),
                http_client,
                audio_engine: OnceCell::new(),
                audio_cache,
                code_graph: OnceCell::new(),
                symbol_graph: OnceCell::new(),
                obfuscation_salt: crate::intelligence::graph::derive_stable_salt(&base_dir),
                identity_context: OnceCell::new(),
                memory_context: OnceCell::new(),
                #[cfg(feature = "vector-memory")]
                swarm_vault: OnceCell::new(),
                #[cfg(feature = "vector-memory")]
                knowledge_store: OnceCell::new(),
                rate_limiters: DashMap::new(),
                initialization_registry: DashMap::new(),
                hardware_profiler: Arc::new(crate::system::profiler::HardwareProfiler::new()),
                blueprint_cache: OnceCell::new(),
                acl: Arc::new(crate::services::acl_service::AclService),
                renderer: Arc::new(crate::agent::runner::prompt_renderer::PromptRenderer),
                base_dir: base_dir.clone(),
                tool_cache: Arc::new(parking_lot::Mutex::new(
                    crate::agent::runner::tools::cache::SharedToolCache::new(),
                )),
                conflict_manager: Arc::new(crate::security::conflict::ConflictManager::new()),
                payment_router: create_payment_router(pool.clone()),
            }),
            mirror_mode,
            drift_alerts: Arc::new(dashmap::DashMap::new()),
            base_dir,
            session_id: uuid::Uuid::new_v4().to_string(),
        };

        // 🧬 [Evolution] Passive Hot-Reloading Loop
        // Monitors the workspace for autonomously generated skills and workflows.
        let registry_handle = state.registry.clone();
        let base_dir_clone = state.base_dir.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(10));
            loop {
                interval.tick().await;
                if let Err(e) = registry_handle.skills.reload_all().await {
                    tracing::error!("🚨 [Evolution] Passive hot-reload failure: {:?}", e);
                }
                let plugins_dir = base_dir_clone.join("plugins");
                let plugins =
                    crate::agent::runner::tools::plugin::load_dynamic_plugins(&plugins_dir).await;
                registry_handle.tool_registry.reload_plugins(plugins);
            }
        });

        // Initial Statuses
        state
            .resources
            .set_subsystem_status("Database", SubsystemStatus::Ready);
        state
            .resources
            .set_subsystem_status("Agents", SubsystemStatus::Ready);
        state
            .resources
            .set_subsystem_status("MCP", SubsystemStatus::Ready);
        state
            .resources
            .set_subsystem_status("Network", SubsystemStatus::NotStarted);
        state
            .resources
            .set_subsystem_status("CodeGraph", SubsystemStatus::NotStarted);
        state
            .resources
            .set_subsystem_status("Audio", SubsystemStatus::NotStarted);

        Ok(state)
    }

    /// ### 📡 Observability: System Broadcast (broadcast_sys)
    /// Publishes a high-priority system event to all connected telemetry
    /// consumers (WebSockets, OTel exporters).
    ///
    /// ### 🛡️ Neural Shield: Secret Redaction
    /// Automatically performs in-flight redaction of the log message using
    /// industry-standard regex patterns to prevent accidental leakage of
    /// API keys, tokens, or PII.
    pub fn broadcast_sys(&self, text: &str, severity: &str, mission_id: Option<String>) {
        let safe_text = self.security.secret_redactor.redact(text);
        let entry = crate::types::LogEntry::new("System", &safe_text, severity, mission_id);
        let _ = self.comms.tx.send(entry);
    }

    /// Helper to broadcast an agent-sourced log with identity metadata.
    pub fn broadcast_agent(
        &self,
        text: &str,
        severity: &str,
        mission_id: Option<String>,
        agent_id: &str,
        agent_name: &str,
    ) {
        let safe_text = self.security.secret_redactor.redact(text);
        let mut entry = crate::types::LogEntry::new("Agent", &safe_text, severity, mission_id);
        entry.agent_id = Some(agent_id.to_string());
        entry.agent_name = Some(agent_name.to_string());
        let _ = self.comms.tx.send(entry);
    }

    /// Helper to broadcast an arbitrary Engine event.
    pub fn emit_event(&self, event: serde_json::Value) {
        let mut full_event = event;
        let seq = self
            .comms
            .event_sequence
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);

        if let Some(obj) = full_event.as_object_mut() {
            obj.insert("_seq".to_string(), serde_json::json!(seq));
        }

        let _ = self.comms.event_tx.send(full_event);
    }

    /// Returns the overall health state of the Swarm Engine based on critical subsystems.
    pub fn health_state(&self) -> crate::types::SystemHealthState {
        let mut critical_subsystems = vec![
            "Database",
            "Agents",
            "MCP",
            "Heartbeat",
            "SecurityEviction",
            "PrivacyGuard",
            "BudgetFlush",
        ];

        let has_network = self
            .resources
            .initialization_registry
            .contains_key("Network");
        let has_codegraph = self
            .resources
            .initialization_registry
            .contains_key("CodeGraph");

        if has_network {
            critical_subsystems.push("Network");
            critical_subsystems.push("SwarmPulse");
        }
        if has_codegraph {
            critical_subsystems.push("CodeGraph");
            critical_subsystems.push("CodeGraphDbRefresh");
        }

        let mut warming = false;
        for sub in critical_subsystems {
            match self.resources.initialization_registry.get(sub) {
                Some(status) => match status.value() {
                    crate::types::SubsystemStatus::Failed(_) => {
                        return crate::types::SystemHealthState::Degraded;
                    }
                    crate::types::SubsystemStatus::Warming(_)
                    | crate::types::SubsystemStatus::NotStarted => {
                        warming = true;
                    }
                    crate::types::SubsystemStatus::Ready => {}
                },
                None => {
                    warming = true;
                }
            }
        }

        if warming {
            crate::types::SystemHealthState::Warming
        } else {
            crate::types::SystemHealthState::Ready
        }
    }

    /// ### ⏳ Governance: Oversight Synchronization (yield_phase_transition)
    /// Forces the current agent mission thread to yield execution back to the
    /// Tokio scheduler.
    ///
    /// ### 🧬 Rationale: Resource Fairness & Interception
    /// 1. **Scheduler Fairness**: Prevents long-running "Think Loops" or
    ///    heavy RAG retrievals from starving other mission branches.
    /// 2. **Interception Window**: Provides a deterministic point where the
    ///    `SecurityHub` can inject external pause/stop signals (e.g., from
    ///    the User Oversight UI) before the next phase begins.
    pub async fn yield_phase_transition(&self, agent_id: &str, phase: &str) {
        tracing::debug!(
            "⏳ [Oversight] Agent {} yielding at boundary: {}",
            agent_id,
            phase
        );

        // Emits a phase transition telemetry event for UI tracking
        self.emit_event(serde_json::json!({
            "type": "agent:phase_transition",
            "agent_id": agent_id,
            "phase": phase
        }));

        // Explicitly suspend the task to allow other scheduler components
        // (like the monitoring loops) to execute.
        tokio::task::yield_now().await;
    }

    /// Persists all current agent states to the database in a single transaction.
    /// Batched to avoid N individual round-trips (was the #1 shutdown bottleneck).
    pub async fn save_agents(&self) {
        // Batch all saves into a single transaction (1 fsync vs N fsyncs)
        match self.resources.pool.begin().await {
            Ok(mut tx) => {
                for mut entry in self.registry.agents.iter_mut() {
                    let agent = entry.value_mut();
                    if let Err(err) =
                        crate::agent::persistence::save_agent_db_in_tx(&mut tx, agent).await
                    {
                        tracing::error!(
                            agent_id = %agent.identity.id,
                            error = %err,
                            "❌ [State] Failed to persist agent during batched save_agents"
                        );
                    }
                }
                if let Err(err) = tx.commit().await {
                    tracing::error!(
                        error = %err,
                        "❌ [State] Failed to commit agent batch transaction"
                    );
                }
            }
            Err(err) => {
                tracing::error!(
                    error = %err,
                    "❌ [State] Failed to begin agent batch transaction — falling back to individual saves"
                );
                // Fallback: individual saves (degraded but functional)
                for mut entry in self.registry.agents.iter_mut() {
                    let agent = entry.value_mut();
                    if let Err(err) =
                        crate::agent::persistence::save_agent_db(&self.resources.pool, agent).await
                    {
                        tracing::error!(
                            agent_id = %agent.identity.id,
                            error = %err,
                            "❌ [State] Failed to persist agent during fallback save_agents"
                        );
                    }
                }
            }
        }
    }

    /// Persists all provider configurations to disk.
    pub async fn save_providers(&self) {
        let providers_vec: Vec<crate::agent::types::ProviderConfig> = self
            .registry
            .providers
            .iter()
            .map(|kv| kv.value().clone())
            .collect();
        if let Err(e) =
            crate::agent::persistence::save_providers(&self.base_dir, providers_vec).await
        {
            tracing::error!("❌ [State] Failed to persist providers to disk: {:?}", e);
        }
    }

    /// Persists all model metadata to disk.
    pub async fn save_models(&self) {
        let models_vec: Vec<crate::agent::types::ModelEntry> = self
            .registry
            .models
            .iter()
            .map(|kv| kv.value().clone())
            .collect();
        if let Err(e) = crate::agent::persistence::save_models(&self.base_dir, models_vec).await {
            tracing::error!("❌ [State] Failed to persist models to disk: {:?}", e);
        }
    }

    /// Persists governance settings to data/governance_settings.json.
    pub fn save_governance_settings(&self) {
        let settings_path = self.base_dir.join("data").join("governance_settings.json");
        let payload = crate::routes::oversight::OversightSettingsPayload {
            auto_approve_safe_skills: self
                .governance
                .auto_approve_safe_skills
                .load(std::sync::atomic::Ordering::Relaxed),
            privacy_mode: Some(
                self.governance
                    .privacy_mode
                    .load(std::sync::atomic::Ordering::Relaxed),
            ),
            cluster_privacy_policies: Some(
                self.governance
                    .cluster_privacy_policies
                    .iter()
                    .map(|kv| (kv.key().clone(), *kv.value()))
                    .collect(),
            ),
            max_agents: Some(
                self.governance
                    .max_agents
                    .load(std::sync::atomic::Ordering::Relaxed),
            ),
            max_clusters: Some(
                self.governance
                    .max_clusters
                    .load(std::sync::atomic::Ordering::Relaxed),
            ),
            max_swarm_depth: Some(
                self.governance
                    .max_swarm_depth
                    .load(std::sync::atomic::Ordering::Relaxed),
            ),
            max_task_length: Some(
                self.governance
                    .max_task_length
                    .load(std::sync::atomic::Ordering::Relaxed),
            ),
            default_budget_usd: Some(*self.governance.default_budget_usd.read()),
            default_model: Some(self.governance.default_model.read().clone()),
            default_provider: Some(self.governance.default_provider.read().clone()),
        };

        if let Some(parent) = settings_path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }

        if let Ok(json_str) = serde_json::to_string_pretty(&payload) {
            if let Err(e) = std::fs::write(&settings_path, json_str) {
                tracing::error!(
                    "🚨 [Governance] Failed to persist settings to {}: {:?}",
                    settings_path.display(),
                    e
                );
            } else {
                tracing::info!(
                    "🛡️ [Governance] Settings persisted successfully to {}",
                    settings_path.display()
                );
            }
        }
    }

    /// Flushes all volatile buffers to persistent storage.
    ///
    /// ### 💾 Persistence Guarantee
    /// Aggregates agent registry states, model updates, and budget meter logs
    /// into a batched transaction. This is the primary safety valve for
    /// graceful engine shutdowns.
    pub async fn flush_all(&self) {
        tracing::info!(
            "💾 [System] Flushing all volatile buffers and registries to persistence..."
        );

        // 1. Persist Registries
        self.save_agents().await;
        self.save_providers().await;
        self.save_models().await;

        // 2. Flush Telemetry & Metering
        if let Err(e) = self.security.budget_guard.flush_to_db().await {
            tracing::error!("🚨 [System] Failed to flush budget data: {}", e);
        }
    }
}

pub(crate) fn create_payment_router(
    pool: sqlx::SqlitePool,
) -> std::sync::Arc<crate::agent::runner::a2a_router::PaymentRouter> {
    let dev_adapter = std::sync::Arc::new(crate::agent::runner::a2a_router::LocalMockAdapter);
    let staging_adapter = std::sync::Arc::new(crate::agent::runner::a2a_router::LocalMockAdapter);

    let web3_rpc = std::env::var("A2A_WEB3_RPC_URL").unwrap_or_default();
    let web3_vault = std::env::var("A2A_WEB3_VAULT_ADDRESS").unwrap_or_default();

    let prod_adapter: std::sync::Arc<dyn crate::agent::runner::a2a_router::A2APaymentAdapter> =
        if !web3_rpc.is_empty() && !web3_vault.is_empty() {
            std::sync::Arc::new(crate::agent::runner::a2a_router::L3HybridAdapter::new(
                web3_rpc, web3_vault,
            ))
        } else {
            std::sync::Arc::new(crate::agent::runner::a2a_router::LocalMockAdapter)
        };

    std::sync::Arc::new(crate::agent::runner::a2a_router::PaymentRouter::new(
        pool,
        dev_adapter,
        staging_adapter,
        prod_adapter,
    ))
}

// Metadata: [mod]

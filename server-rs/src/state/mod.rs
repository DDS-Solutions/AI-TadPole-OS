//! @docs ARCHITECTURE:State
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / mod
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Type-safe state handling and bounded execution without unhandled panics.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: none declared
//! - **Witness Tests**: none declared

use dashmap::DashMap;
use parking_lot::RwLock;
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicUsize};
use std::sync::Arc;
use tokio::sync::{broadcast, OnceCell};

pub mod hubs;
pub mod init;
#[cfg(test)]
pub mod mock;
pub mod persistence;

pub use init::create_payment_router;

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

        let channels = init::init_channels();
        let tx = channels.tx;
        let event_tx = channels.event_tx;
        let audio_stream_tx = channels.audio_stream_tx;
        let pulse_tx = channels.pulse_tx;
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

        let security_tokens = init::load_security_tokens()?;
        let deploy_token = security_tokens.deploy_token;
        let admin_token = security_tokens.admin_token;

        let pool = init::init_database_pool(&base_dir).await?;

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
            let is_demo_mode = std::env::var("TADPOLE_DEMO_MODE")
                .map(|v| v == "true" || v == "1")
                .unwrap_or(false);
            if is_demo_mode {
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
                            current_task: Some(
                                "Monitoring IPC channels & zero-trust bridge".to_string(),
                            ),
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
                            current_task: Some(
                                "Polling cron queue & state checkpoints".to_string(),
                            ),
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
                            current_task: Some(
                                "Listening for HITL signals from Android app".to_string(),
                            ),
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                ];
                for a in default_agents {
                    agents.insert(a.identity.id.clone(), a);
                }
            }
        }
        tracing::info!("✅ [Registries] Agents loaded (count: {}).", agents.len());

        let http_client = init::init_http_client()?;
        let audio_cache = init::init_audio_cache(&base_dir).await;

        let secret_redactor = Arc::new(crate::secret_redactor::SecretRedactor::from_env());

        // Assemble Hubs
        let max_concurrent_runners = std::env::var("MAX_CONCURRENT_RUNNERS")
            .ok()
            .and_then(|s| s.parse::<u32>().ok())
            .filter(|limit| *limit > 0)
            .unwrap_or(10);
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
            runner_semaphore: tokio::sync::Semaphore::new(max_concurrent_runners as usize),
            event_sequence: std::sync::atomic::AtomicU64::new(0),
        });

        tracing::info!("💠 [Hubs] Assembling Governance Hub...");
        let governance = Arc::new(GovernanceHub {
            auto_approve_safe_skills: AtomicBool::new(
                std::env::var("AUTO_APPROVE_SAFE_SKILLS")
                    .map(|s| s == "true")
                    .unwrap_or(false),
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
                std::env::var("DEFAULT_PROVIDER").unwrap_or_else(|_| "google".to_string()),
            ),
            active_agents: AtomicU32::new(0),
            max_concurrent_runners: AtomicU32::new(max_concurrent_runners),
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
                    if let Some(val) = persisted.auto_approve_safe_skills {
                        governance
                            .auto_approve_safe_skills
                            .store(val, std::sync::atomic::Ordering::Relaxed);
                    }
                    if let Some(val) = persisted.privacy_mode {
                        governance
                            .privacy_mode
                            .store(val, std::sync::atomic::Ordering::Relaxed);
                    }
                    if let Some(ref policies) = persisted.cluster_privacy_policies {
                        for (cid, mode) in policies {
                            governance
                                .cluster_privacy_policies
                                .insert(cid.clone(), *mode);
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

        // 🛡️ [Task 1.5: Fail-Fast Tool Config Validator]
        init::validate_tool_configurations(&registry)?;

        let system_monitor = Arc::new(crate::security::monitoring::SecurityMonitor::new());
        let budget_guard = Arc::new(crate::security::metering::BudgetGuard::new(
            pool.clone(),
            system_monitor.clone(),
        ));

        let security = Arc::new(SecurityHub {
            conflict_manager: Arc::new(crate::security::conflict::ConflictManager::new()),
            audit_trail: Arc::new(
                crate::security::audit::MerkleAuditTrail::new(pool.clone()).map_err(|e| {
                    AppError::InternalServerError(format!(
                        "Failed to initialize audit trail: {}",
                        e
                    ))
                })?,
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
            oversight_public_key: init::load_oversight_public_key(),
        });

        // ── Boot Audit Ledger Integrity Verification ────────────────
        init::verify_boot_audit_ledger(security.audit_trail.clone());

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
                workflow_active_runs: Arc::new(DashMap::new()),
                workflow_concurrency_semaphore: Arc::new(tokio::sync::Semaphore::new(10)),
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
}

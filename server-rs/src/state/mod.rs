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
use sqlx::SqlitePool;
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicUsize};
use std::sync::Arc;
use tokio::sync::{broadcast, OnceCell};

pub mod hubs;

use hubs::comm::CommunicationHub;
use hubs::gov::GovernanceHub;
use hubs::reg::RegistryHub;
use hubs::res::ResourceHub;
use hubs::sec::SecurityHub;

use crate::agent::types::EngineAgent;
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
    /// Global workspace root directory for data persistence.
    pub base_dir: std::path::PathBuf,
}

impl AppState {
    /// Mock constructor for unit tests
    #[allow(dead_code)]
    pub async fn new_mock() -> Self {
        Self::new_mock_ext(false).await
    }

    /// Lighter mock constructor that skips database seeding and non-essential subsystems.
    /// Ideal for high-performance unit tests.
    #[allow(dead_code)]
    pub async fn new_minimal_mock() -> Self {
        Self::new_mock_ext(true).await
    }

    async fn new_mock_ext(minimal: bool) -> Self {
        let (tx, _) = tokio::sync::broadcast::channel(1000);
        let (event_tx, _) = tokio::sync::broadcast::channel(1000);
        let (audio_stream_tx, _) = tokio::sync::broadcast::channel(5000);
        let (telemetry_tx, _) = tokio::sync::broadcast::channel(1000);
        let (pulse_tx, _) = tokio::sync::broadcast::channel(1000);

        // SEC: Use unique temporary directories for test base_dir to avoid shared-state 
        // race conditions in parallel unit tests (INFRA-01).
        let test_id = uuid::Uuid::new_v4().to_string();
        let base_dir = std::env::temp_dir().join(format!("tadpole-test-{}", test_id));
        std::fs::create_dir_all(&base_dir).ok();

        if minimal {
            std::env::set_var("SKIP_DB_SEED", "true");
        } else {
            std::env::remove_var("SKIP_DB_SEED");
        }

        let database_url = "sqlite::memory:".to_string();
        let pool = crate::db::init_db(&database_url)
            .await
            .map_err(|e| {
                eprintln!("🚨 CRITICAL: Failed to init test DB: {:?}", e);
                e
            })
            .unwrap_or_else(|_| panic!("Failed to initialize in-memory test database"));

        let comms = Arc::new(CommunicationHub {
            tx,
            event_tx,
            telemetry_tx: telemetry_tx.clone(),
            audio_stream_tx,
            pulse_tx,
            oversight_queue: DashMap::new(),
            oversight_resolvers: DashMap::new(),
            active_runners: DashMap::new(),
            event_sequence: std::sync::atomic::AtomicU64::new(0),
        });

        let governance = Arc::new(GovernanceHub {
            auto_approve_safe_skills: std::sync::atomic::AtomicBool::new(true),
            max_agents: std::sync::atomic::AtomicU32::new(10),
            max_clusters: std::sync::atomic::AtomicU32::new(5),
            max_swarm_depth: std::sync::atomic::AtomicU32::new(3),
            max_task_length: std::sync::atomic::AtomicUsize::new(4096),
            default_budget_usd: parking_lot::RwLock::new(0.50),
            active_agents: std::sync::atomic::AtomicU32::new(0),
            recruit_count: std::sync::atomic::AtomicU32::new(0),
            tpm_accumulator: std::sync::atomic::AtomicUsize::new(0),
            privacy_mode: std::sync::atomic::AtomicBool::new(false),
        });

        let permission_policy = Arc::new(crate::security::permissions::PermissionPolicy::new(
            pool.clone(),
        ));

        let registry = Arc::new(RegistryHub {
            agents: DashMap::new(),
            providers: DashMap::new(),
            models: DashMap::new(),
            nodes: DashMap::new(),
            skills: Arc::new(crate::agent::script_skills::ScriptSkillsRegistry::mock(base_dir.clone())),
            skill_registry: Arc::new(crate::agent::skill_manifest::SkillRegistry::new()),
            mcp_host: Arc::new(crate::agent::mcp::McpHost::new(
                telemetry_tx,
                None,
                permission_policy.clone(),
            )),
            hooks: Arc::new(crate::agent::hooks::HooksManager::new(
                &base_dir,
            )),
        });

        let security = Arc::new(SecurityHub {
            audit_trail: Arc::new(crate::security::audit::MerkleAuditTrail::mock()),
            budget_guard: Arc::new(crate::security::metering::BudgetGuard::mock()),
            shell_scanner: Arc::new(crate::security::scanner::ShellScanner::mock()),
            secret_redactor: Arc::new(crate::secret_redactor::SecretRedactor::noop()),
            system_monitor: Arc::new(crate::security::monitoring::SecurityMonitor::new()),
            permission_policy,
            deploy_token: "test-token".to_string(),
        });

        let state = Self {
            comms,
            governance,
            registry,
            security,
            resources: Arc::new(ResourceHub {
                pool,
                http_client: Arc::new(reqwest::Client::new()),
                audio_engine: tokio::sync::OnceCell::new(),
                audio_cache: Arc::new(crate::agent::audio_cache::BunkerCache::mock()),
                code_graph: tokio::sync::OnceCell::new(),
                identity_context: tokio::sync::OnceCell::new(),
                memory_context: tokio::sync::OnceCell::new(),
                #[cfg(feature = "vector-memory")]
                vector_memory: tokio::sync::OnceCell::new(),
                rate_limiters: DashMap::new(),
                initialization_registry: DashMap::new(),
                hardware_profiler: Arc::new(crate::system::profiler::HardwareProfiler::new()),
            }),
            base_dir,
        };
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
    }

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
        let deploy_token = match std::env::var("NEURAL_TOKEN") {
            Ok(token) => token.trim().to_string(),
            Err(_) if cfg!(test) => "ci-test-token-placeholder".to_string(),
            Err(_) => return Err(AppError::Unauthorized(
                "🚨 FATAL: NEURAL_TOKEN environment variable MUST be set for the engine to start.".to_string()
            )),
        };

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
        tracing::info!("✅ [Registries] Agents loaded (count: {}).", agents.len());

        tracing::info!("🚀 [Engines] Initializing HTTP Client...");
        let http_client = Arc::new(
            reqwest::Client::builder()
                .user_agent("TadpoleOS/1.1.38")
                .pool_max_idle_per_host(10)
                .pool_idle_timeout(std::time::Duration::from_secs(60))
                .connect_timeout(std::time::Duration::from_secs(10))
                .timeout(std::time::Duration::from_secs(120))
                .tcp_nodelay(true)
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
            active_agents: AtomicU32::new(0),
            recruit_count: AtomicU32::new(0),
            tpm_accumulator: AtomicUsize::new(0),
            privacy_mode: AtomicBool::new(false),
        });

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
                .map_err(|e| AppError::InternalServerError(format!("Failed to initialize script skills registry: {}", e)))?,
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

        let registry = Arc::new(RegistryHub {
            agents: agents.clone(),
            providers,
            models,
            nodes: DashMap::new(),
            skills: script_skills,
            skill_registry,
            mcp_host,
            hooks,
        });

        let security = Arc::new(SecurityHub {
            audit_trail: Arc::new(crate::security::audit::MerkleAuditTrail::new(pool.clone())),
            budget_guard: Arc::new(crate::security::metering::BudgetGuard::new(pool.clone())),
            shell_scanner: Arc::new(crate::security::scanner::ShellScanner::new(
                secret_redactor.clone(),
            )),
            secret_redactor,
            system_monitor: Arc::new(crate::security::monitoring::SecurityMonitor::new()),
            permission_policy,
            deploy_token,
        });

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
                identity_context: OnceCell::new(),
                memory_context: OnceCell::new(),
                #[cfg(feature = "vector-memory")]
                vector_memory: OnceCell::new(),
                rate_limiters: DashMap::new(),
                initialization_registry: DashMap::new(),
                hardware_profiler: Arc::new(crate::system::profiler::HardwareProfiler::new()),
            }),
            base_dir,
        };

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
        let seq = self.comms.event_sequence.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        
        if let Some(obj) = full_event.as_object_mut() {
            obj.insert("_seq".to_string(), serde_json::json!(seq));
        }
        
        let _ = self.comms.event_tx.send(full_event);
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
        let agents_vec: Vec<EngineAgent> = self
            .registry
            .agents
            .iter()
            .map(|kv| kv.value().clone())
            .collect();

        // Batch all saves into a single transaction (1 fsync vs N fsyncs)
        match self.resources.pool.begin().await {
            Ok(mut tx) => {
                for agent in &agents_vec {
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
                for agent in &agents_vec {
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
        let _ = crate::agent::persistence::save_providers(&self.base_dir, providers_vec).await;
    }

    /// Persists all model metadata to disk.
    pub async fn save_models(&self) {
        let models_vec: Vec<crate::agent::types::ModelEntry> = self
            .registry
            .models
            .iter()
            .map(|kv| kv.value().clone())
            .collect();
        let _ = crate::agent::persistence::save_models(&self.base_dir, models_vec).await;
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

impl Default for AppState {
    /// Creates a mock version of the application state for testing purposes.
    fn default() -> Self {
        let (tx, _) = broadcast::channel(1);
        let (event_tx, _) = broadcast::channel(1);
        let (audio_stream_tx, _) = broadcast::channel(1);
        let (telemetry_tx, _) = broadcast::channel(1);
        let (pulse_tx, _) = broadcast::channel(1);

        let pool = SqlitePool::connect_lazy("sqlite::memory:").unwrap_or_else(|_| {
            panic!("🚨 CRITICAL: Failed to connect to lazy in-memory SQLite pool for Default AppState");
        });

        let comms = Arc::new(CommunicationHub {
            tx: tx.clone(),
            event_tx,
            telemetry_tx,
            audio_stream_tx,
            pulse_tx,
            oversight_queue: DashMap::new(),
            oversight_resolvers: DashMap::new(),
            active_runners: DashMap::new(),
            event_sequence: std::sync::atomic::AtomicU64::new(0),
        });

        let governance = Arc::new(GovernanceHub {
            auto_approve_safe_skills: AtomicBool::new(true),
            max_agents: AtomicU32::new(50),
            max_clusters: AtomicU32::new(10),
            max_swarm_depth: AtomicU32::new(5),
            max_task_length: AtomicUsize::new(32768),
            default_budget_usd: RwLock::new(1.0),
            active_agents: AtomicU32::new(0),
            recruit_count: AtomicU32::new(0),
            tpm_accumulator: AtomicUsize::new(0),
            privacy_mode: AtomicBool::new(false),
        });

        let permission_policy = Arc::new(crate::security::permissions::PermissionPolicy::new(
            pool.clone(),
        ));

        let registry = Arc::new(RegistryHub {
            agents: DashMap::new(),
            providers: DashMap::new(),
            models: DashMap::new(),
            nodes: DashMap::new(),
            skills: Arc::new(crate::agent::script_skills::ScriptSkillsRegistry::mock(std::path::PathBuf::from("tmp"))),
            skill_registry: Arc::new(crate::agent::skill_manifest::SkillRegistry::new()),
            mcp_host: Arc::new(crate::agent::mcp::McpHost::new(
                broadcast::channel(1).0,
                None,
                permission_policy.clone(),
            )),
            hooks: Arc::new(crate::agent::hooks::HooksManager::new(
                &std::path::PathBuf::from("tmp"),
            )),
        });

        let security = Arc::new(SecurityHub {
            audit_trail: Arc::new(crate::security::audit::MerkleAuditTrail::mock()),
            budget_guard: Arc::new(crate::security::metering::BudgetGuard::mock()),
            shell_scanner: Arc::new(crate::security::scanner::ShellScanner::mock()),
            secret_redactor: Arc::new(crate::secret_redactor::SecretRedactor::noop()),
            system_monitor: Arc::new(crate::security::monitoring::SecurityMonitor::new()),
            permission_policy,
            deploy_token: "test".into(),
        });

        let resources = Arc::new(ResourceHub {
            pool: pool.clone(),
            http_client: Arc::new(reqwest::Client::new()),
            audio_engine: OnceCell::new(),
            audio_cache: Arc::new(crate::agent::audio_cache::BunkerCache::mock()),
            code_graph: OnceCell::new(),
            identity_context: OnceCell::new(),
            memory_context: OnceCell::new(),
            #[cfg(feature = "vector-memory")]
            vector_memory: OnceCell::new(),
            rate_limiters: DashMap::new(),
            initialization_registry: DashMap::new(),
            hardware_profiler: Arc::new(crate::system::profiler::HardwareProfiler::new()),
        });

        Self {
            comms,
            governance,
            registry,
            security,
            resources,
            base_dir: std::path::PathBuf::from("data"),
        }
    }
}

// Metadata: [mod]

// Metadata: [mod]

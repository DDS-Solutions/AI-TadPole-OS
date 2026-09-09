//! @docs ARCHITECTURE:Core
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / mock
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Type-safe state handling and bounded execution without unhandled panics.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: none declared
//! - **Witness Tests**: none declared

use super::create_payment_router;
use super::hubs::comm::CommunicationHub;
use super::hubs::gov::GovernanceHub;
use super::hubs::reg::RegistryHub;
use super::hubs::res::ResourceHub;
use super::hubs::sec::SecurityHub;
use super::AppState;
use crate::types::SubsystemStatus;
use dashmap::DashMap;
use parking_lot::RwLock;
use sqlx::SqlitePool;
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicUsize};
use std::sync::Arc;
use tokio::sync::{broadcast, OnceCell};

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

    /// Creates an AppState using a provided pool. Useful for testing.
    #[allow(dead_code)]
    pub async fn with_pool(pool: SqlitePool) -> Self {
        std::env::set_var(
            "WORKFLOW_ENCRYPTION_KEY",
            "temporary_fallback_encryption_key_32_bytes_long!",
        );
        let (tx, _) = tokio::sync::broadcast::channel(1000);
        let (event_tx, _) = tokio::sync::broadcast::channel(1000);
        let (audio_stream_tx, _) = tokio::sync::broadcast::channel(5000);
        let (telemetry_tx, _) = tokio::sync::broadcast::channel(1000);
        let (pulse_tx, _) = tokio::sync::broadcast::channel(1000);

        let base_dir = std::env::temp_dir().join(format!("tadpole-test-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&base_dir).ok();

        let comms = Arc::new(CommunicationHub {
            tx,
            event_tx,
            telemetry_tx: telemetry_tx.clone(),
            audio_stream_tx,
            pulse_tx,
            oversight_queue: DashMap::new(),
            oversight_resolvers: DashMap::new(),
            active_runners: DashMap::new(),
            recent_requests: DashMap::new(),
            runner_semaphore: tokio::sync::Semaphore::new(20),
            event_sequence: std::sync::atomic::AtomicU64::new(0),
        });

        let governance = Arc::new(GovernanceHub {
            auto_approve_safe_skills: std::sync::atomic::AtomicBool::new(true),
            max_agents: std::sync::atomic::AtomicU32::new(10),
            max_clusters: std::sync::atomic::AtomicU32::new(5),
            max_swarm_depth: std::sync::atomic::AtomicU32::new(3),
            max_task_length: std::sync::atomic::AtomicUsize::new(4096),
            default_budget_usd: parking_lot::RwLock::new(0.50),
            default_model: parking_lot::RwLock::new("gemini-1.5-pro".to_string()),
            default_provider: parking_lot::RwLock::new("google".to_string()),
            active_agents: std::sync::atomic::AtomicU32::new(0),
            max_concurrent_runners: std::sync::atomic::AtomicU32::new(10),
            recruit_count: std::sync::atomic::AtomicU32::new(0),
            tpm_accumulator: std::sync::atomic::AtomicUsize::new(0),
            privacy_mode: std::sync::atomic::AtomicBool::new(false),
            failover_amber_threshold: std::sync::atomic::AtomicU32::new(3),
            failover_red_threshold: std::sync::atomic::AtomicU32::new(5),
            failover_max_attempts: std::sync::atomic::AtomicU32::new(3),
            provider_timeout_secs: std::sync::atomic::AtomicU32::new(60),
            null_providers_test_mode: std::sync::atomic::AtomicBool::new(false),
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

        let permission_policy = Arc::new(crate::security::permissions::PermissionPolicy::new(
            pool.clone(),
        ));

        let agents_list = crate::agent::persistence::load_agents_db(&pool)
            .await
            .unwrap_or_default();
        let agents = DashMap::new();
        for a in agents_list {
            agents.insert(a.identity.id.clone(), a);
        }

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

        let registry = Arc::new(RegistryHub {
            agents,
            providers,
            provider_health: DashMap::new(),
            provider_failures: DashMap::new(),
            models,
            nodes: DashMap::new(),
            skills: Arc::new(crate::agent::script_skills::ScriptSkillsRegistry::mock(
                base_dir.clone(),
            )),
            skill_registry: Arc::new(crate::agent::skill_manifest::SkillRegistry::new()),
            mcp_host: Arc::new(crate::agent::mcp::McpHost::new(
                telemetry_tx,
                None,
                permission_policy.clone(),
            )),
            hooks: Arc::new(crate::agent::hooks::HooksManager::new(&base_dir)),
            tool_registry: Arc::new(
                crate::agent::runner::tools::dispatcher::Dispatcher::new().registry,
            ),
            mission_backlogs: dashmap::DashMap::new(),
        });

        let system_monitor = Arc::new(crate::security::monitoring::SecurityMonitor::new());
        let budget_guard = Arc::new(crate::security::metering::BudgetGuard::new(
            pool.clone(),
            system_monitor.clone(),
        ));

        let security = Arc::new(SecurityHub {
            audit_trail: Arc::new(
                crate::security::audit::MerkleAuditTrail::mock_async()
                    .await
                    .expect("Failed to initialize mock audit trail"),
            ),
            budget_guard,
            shell_scanner: Arc::new(crate::security::scanner::ShellScanner::mock()),
            secret_redactor: Arc::new(crate::secret_redactor::SecretRedactor::noop()),
            system_monitor,
            permission_policy,
            deploy_token: "test-token".to_string(),
            admin_token: "test-admin-token".to_string(),
            oversight_public_key: None,
        });

        let mirror_mode = std::env::var("MIRROR_MODE")
            .map(|s| s.to_lowercase() == "true")
            .unwrap_or(false);

        Self {
            comms,
            governance,
            registry,
            security,
            resources: Arc::new(ResourceHub {
                pool: pool.clone(),
                http_client: Arc::new(reqwest::Client::new()),
                audio_engine: tokio::sync::OnceCell::new(),
                audio_cache: Arc::new(crate::agent::audio_cache::BunkerCache::mock()),
                code_graph: tokio::sync::OnceCell::new(),
                symbol_graph: tokio::sync::OnceCell::new(),
                obfuscation_salt: crate::intelligence::graph::derive_stable_salt(&base_dir),
                identity_context: tokio::sync::OnceCell::new(),
                memory_context: tokio::sync::OnceCell::new(),
                #[cfg(feature = "vector-memory")]
                swarm_vault: tokio::sync::OnceCell::new(),
                #[cfg(feature = "vector-memory")]
                knowledge_store: tokio::sync::OnceCell::new(),
                rate_limiters: DashMap::new(),
                initialization_registry: DashMap::new(),
                hardware_profiler: Arc::new(crate::system::profiler::HardwareProfiler::new()),
                blueprint_cache: tokio::sync::OnceCell::new(),
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
        }
    }

    async fn new_mock_ext(minimal: bool) -> Self {
        std::env::set_var(
            "WORKFLOW_ENCRYPTION_KEY",
            "temporary_fallback_encryption_key_32_bytes_long!",
        );
        let (tx, _) = tokio::sync::broadcast::channel(1000);
        let (event_tx, _) = tokio::sync::broadcast::channel(1000);
        let (audio_stream_tx, _) = tokio::sync::broadcast::channel(5000);
        let (telemetry_tx, _) = tokio::sync::broadcast::channel(1000);
        let (pulse_tx, _) = tokio::sync::broadcast::channel(1000);

        let test_id = uuid::Uuid::new_v4().to_string();
        let base_dir = std::env::temp_dir().join(format!("tadpole-test-{}", test_id));
        std::fs::create_dir_all(&base_dir).ok();

        let db_path = base_dir.join("test.db");
        let mut database_url = format!("sqlite:{}", db_path.display());
        if minimal {
            database_url.push_str("?skip_seed=true");
        }
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
            recent_requests: DashMap::new(),
            runner_semaphore: tokio::sync::Semaphore::new(20),
            event_sequence: std::sync::atomic::AtomicU64::new(0),
        });

        let governance = Arc::new(GovernanceHub {
            auto_approve_safe_skills: AtomicBool::new(true),
            max_agents: AtomicU32::new(10),
            max_clusters: AtomicU32::new(5),
            max_swarm_depth: AtomicU32::new(3),
            max_task_length: AtomicUsize::new(4096),
            default_budget_usd: parking_lot::RwLock::new(0.50),
            default_model: parking_lot::RwLock::new("gemini-1.5-pro".to_string()),
            default_provider: parking_lot::RwLock::new("google".to_string()),
            active_agents: std::sync::atomic::AtomicU32::new(0),
            max_concurrent_runners: std::sync::atomic::AtomicU32::new(10),
            recruit_count: std::sync::atomic::AtomicU32::new(0),
            tpm_accumulator: std::sync::atomic::AtomicUsize::new(0),
            privacy_mode: std::sync::atomic::AtomicBool::new(false),
            failover_amber_threshold: std::sync::atomic::AtomicU32::new(3),
            failover_red_threshold: std::sync::atomic::AtomicU32::new(5),
            failover_max_attempts: std::sync::atomic::AtomicU32::new(3),
            provider_timeout_secs: std::sync::atomic::AtomicU32::new(60),
            null_providers_test_mode: std::sync::atomic::AtomicBool::new(false),
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

        let permission_policy = Arc::new(crate::security::permissions::PermissionPolicy::new(
            pool.clone(),
        ));

        let agents_list = crate::agent::persistence::load_agents_db(&pool)
            .await
            .unwrap_or_default();
        let agents = DashMap::new();
        for a in agents_list {
            agents.insert(a.identity.id.clone(), a);
        }

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

        let registry = Arc::new(RegistryHub {
            agents,
            providers,
            provider_health: DashMap::new(),
            provider_failures: DashMap::new(),
            models,
            nodes: DashMap::new(),
            skills: Arc::new(crate::agent::script_skills::ScriptSkillsRegistry::mock(
                base_dir.clone(),
            )),
            skill_registry: Arc::new(crate::agent::skill_manifest::SkillRegistry::new()),
            mcp_host: Arc::new(crate::agent::mcp::McpHost::new(
                telemetry_tx,
                None,
                permission_policy.clone(),
            )),
            hooks: Arc::new(crate::agent::hooks::HooksManager::new(&base_dir)),
            tool_registry: Arc::new(
                crate::agent::runner::tools::dispatcher::Dispatcher::new().registry,
            ),
            mission_backlogs: dashmap::DashMap::new(),
        });

        let system_monitor = Arc::new(crate::security::monitoring::SecurityMonitor::new());
        let budget_guard = Arc::new(crate::security::metering::BudgetGuard::new(
            pool.clone(),
            system_monitor.clone(),
        ));

        let security = Arc::new(SecurityHub {
            audit_trail: Arc::new(
                crate::security::audit::MerkleAuditTrail::mock_async()
                    .await
                    .expect("Failed to initialize mock audit trail"),
            ),
            budget_guard,
            shell_scanner: Arc::new(crate::security::scanner::ShellScanner::mock()),
            secret_redactor: Arc::new(crate::secret_redactor::SecretRedactor::noop()),
            system_monitor,
            permission_policy,
            deploy_token: "test-token".to_string(),
            admin_token: "test-admin-token".to_string(),
            oversight_public_key: None,
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
                http_client: Arc::new(reqwest::Client::new()),
                audio_engine: tokio::sync::OnceCell::new(),
                audio_cache: Arc::new(crate::agent::audio_cache::BunkerCache::mock()),
                code_graph: tokio::sync::OnceCell::new(),
                symbol_graph: tokio::sync::OnceCell::new(),
                obfuscation_salt: crate::intelligence::graph::derive_stable_salt(&base_dir),
                identity_context: tokio::sync::OnceCell::new(),
                memory_context: tokio::sync::OnceCell::new(),
                #[cfg(feature = "vector-memory")]
                swarm_vault: tokio::sync::OnceCell::new(),
                #[cfg(feature = "vector-memory")]
                knowledge_store: tokio::sync::OnceCell::new(),
                rate_limiters: DashMap::new(),
                initialization_registry: DashMap::new(),
                hardware_profiler: Arc::new(crate::system::profiler::HardwareProfiler::new()),
                blueprint_cache: tokio::sync::OnceCell::new(),
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
            panic!(
                "🚨 CRITICAL: Failed to connect to lazy in-memory SQLite pool for Default AppState"
            );
        });

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
            runner_semaphore: tokio::sync::Semaphore::new(20),
            event_sequence: std::sync::atomic::AtomicU64::new(0),
        });

        let governance = Arc::new(GovernanceHub {
            auto_approve_safe_skills: AtomicBool::new(true),
            max_agents: AtomicU32::new(50),
            max_clusters: AtomicU32::new(10),
            max_swarm_depth: AtomicU32::new(5),
            max_task_length: AtomicUsize::new(4096),
            default_budget_usd: parking_lot::RwLock::new(0.50),
            default_model: parking_lot::RwLock::new("gemini-1.5-pro".to_string()),
            default_provider: parking_lot::RwLock::new("google".to_string()),
            active_agents: AtomicU32::new(0),
            max_concurrent_runners: AtomicU32::new(20),
            recruit_count: AtomicU32::new(0),
            tpm_accumulator: AtomicUsize::new(0),
            privacy_mode: AtomicBool::new(false),
            failover_amber_threshold: AtomicU32::new(3),
            failover_red_threshold: AtomicU32::new(5),
            failover_max_attempts: AtomicU32::new(3),
            provider_timeout_secs: AtomicU32::new(60),
            null_providers_test_mode: AtomicBool::new(false),
            deprecated_routes: RwLock::new(std::collections::HashMap::new()),
            cluster_privacy_policies: dashmap::DashMap::new(),
        });

        let permission_policy = Arc::new(crate::security::permissions::PermissionPolicy::new(
            pool.clone(),
        ));

        let registry = Arc::new(RegistryHub {
            agents: DashMap::new(),
            providers: DashMap::new(),
            provider_health: DashMap::new(),
            provider_failures: DashMap::new(),
            models: DashMap::new(),
            nodes: DashMap::new(),
            skills: Arc::new(crate::agent::script_skills::ScriptSkillsRegistry::mock(
                std::path::PathBuf::from("tmp"),
            )),
            skill_registry: Arc::new(crate::agent::skill_manifest::SkillRegistry::new()),
            tool_registry: Arc::new(
                crate::agent::runner::tools::dispatcher::Dispatcher::new().registry,
            ),
            mcp_host: Arc::new(crate::agent::mcp::McpHost::new(
                event_tx.clone(),
                None,
                permission_policy.clone(),
            )),
            hooks: Arc::new(crate::agent::hooks::HooksManager::new(
                &std::path::PathBuf::from("tmp"),
            )),
            mission_backlogs: DashMap::new(),
        });

        let system_monitor = Arc::new(crate::security::monitoring::SecurityMonitor::new());

        let security = Arc::new(SecurityHub {
            audit_trail: Arc::new(
                crate::security::audit::MerkleAuditTrail::mock()
                    .expect("Failed to initialize mock audit trail"),
            ),
            budget_guard: Arc::new(crate::security::metering::BudgetGuard::mock()),
            shell_scanner: Arc::new(crate::security::scanner::ShellScanner::mock()),
            secret_redactor: Arc::new(crate::secret_redactor::SecretRedactor::noop()),
            system_monitor,
            permission_policy,
            deploy_token: "test".into(),
            admin_token: "test-admin-token".to_string(),
            oversight_public_key: None,
        });

        let resources = Arc::new(ResourceHub {
            pool: pool.clone(),
            http_client: Arc::new(reqwest::Client::new()),
            audio_engine: OnceCell::new(),
            audio_cache: Arc::new(crate::agent::audio_cache::BunkerCache::mock()),
            code_graph: OnceCell::new(),
            symbol_graph: OnceCell::new(),
            obfuscation_salt: crate::intelligence::graph::derive_stable_salt(
                &std::path::PathBuf::from("data"),
            ),
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
            base_dir: std::path::PathBuf::from("data"),
            tool_cache: Arc::new(parking_lot::Mutex::new(
                crate::agent::runner::tools::cache::SharedToolCache::new(),
            )),
            conflict_manager: Arc::new(crate::security::conflict::ConflictManager::new()),
            payment_router: create_payment_router(pool.clone()),
            workflow_active_runs: Arc::new(DashMap::new()),
            workflow_concurrency_semaphore: Arc::new(tokio::sync::Semaphore::new(10)),
        });

        Self {
            comms,
            governance,
            registry,
            security,
            resources,
            mirror_mode: false,
            drift_alerts: Arc::new(dashmap::DashMap::new()),
            base_dir: std::path::PathBuf::from("data"),
            session_id: uuid::Uuid::new_v4().to_string(),
        }
    }
}

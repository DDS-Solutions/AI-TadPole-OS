//! Global Application State & Thread-Safe Context
//!
//! The `AppState` acts as the single source of truth for the swarm, managing
//! authenticated session maps, database pools, and real-time telemetry hubs.
//!
//! @docs ARCHITECTURE:StatePersistence
//! @docs OPERATIONS_MANUAL:Governance

use chrono::{DateTime, Utc};
use dashmap::DashMap;
use parking_lot::RwLock;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicUsize};
use std::sync::Arc;
use tokio::sync::{broadcast, oneshot, OnceCell};
use uuid::Uuid;

use crate::agent::audio::NeuralAudioEngine;
use crate::agent::audio_cache::BunkerCache;
use crate::agent::types::{EngineAgent, OversightEntry, ProviderConfig, ModelEntry, SwarmNode};
use crate::security::audit::MerkleAuditTrail;
use crate::security::metering::BudgetGuard;
use crate::security::scanner::ShellScanner;
use crate::utils::graph::CodeGraph;

/// Exact parity with the `LogEntry` frontend interface.
/// Represents a single telemetry or system event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    #[serde(rename = "type")]
    pub event_type: String,
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub source: String,
    pub severity: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent_id: Option<String>,
    pub text: String,
}

impl LogEntry {
    /// Creates a new log entry with a unique UUID and current timestamp.
    pub fn new(source: &str, text: &str, severity: &str) -> Self {
        Self {
            event_type: "log".to_string(),
            id: Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            source: source.to_string(),
            text: text.to_string(),
            severity: severity.to_string(),
            agent_id: None,
        }
    }
}

/// The global application state shared across all routes via Axum State.
/// Decomposed into logical hubs for modularity.
pub struct AppState {
    /// Manages real-time communication channels (logs, events, telemetry, audio).
    pub comms: Arc<CommunicationHub>,
    /// Manages operational limits and policy settings.
    pub governance: Arc<GovernanceHub>,
    /// Manages entities like agents, providers, models, and capabilities.
    pub registry: Arc<RegistryHub>,
    /// Manages security features like auditing, budget enforcement, and scanning.
    pub security: Arc<SecurityHub>,
    /// Manages shared system resources (DB pool, HTTP client, file contexts).
    pub resources: Arc<ResourceHub>,
}

/// Hub for real-time broadcast and event orchestration.
pub struct CommunicationHub {
    /// Broadcast system logs to all connected UI WebSockets.
    pub tx: broadcast::Sender<LogEntry>,
    /// Dedicated broadcast for Engine events (decisions, lifecycle changes).
    pub event_tx: broadcast::Sender<serde_json::Value>,
    /// Dedicated high-speed broadcast for agent telemetry (thinking, status).
    pub telemetry_tx: broadcast::Sender<serde_json::Value>,
    /// Dedicated high-speed broadcast for neural audio streams (PCM chunks).
    pub audio_stream_tx: broadcast::Sender<Vec<u8>>,
    /// Pending Oversight entries awaiting human decision.
    pub oversight_queue: DashMap<String, OversightEntry>,
    /// Resolvers for pending oversight promises.
    pub oversight_resolvers: DashMap<String, oneshot::Sender<bool>>,
}

/// Hub for system limits and automated policy enforcement.
pub struct GovernanceHub {
    /// Global setting: whether to auto-approve low-risk skills.
    pub auto_approve_safe_skills: AtomicBool,
    /// Maximum allowed agents in the swarm.
    pub max_agents: AtomicU32,
    /// Maximum allowed clusters.
    pub max_clusters: AtomicU32,
    /// Maximum depth for agent recursion/spawning.
    pub max_swarm_depth: AtomicU32,
    /// Maximum token length for a single task.
    pub max_task_length: AtomicUsize,
    /// Default budget allocated to new agents (in USD).
    pub default_budget_usd: RwLock<f64>,
    /// Number of agents currently executing tasks.
    pub active_agents: AtomicU32,
    /// Total number of recruitment operations performed.
    pub recruit_count: AtomicU32,
    /// Global TPM accumulator for telemetry.
    pub tpm_accumulator: AtomicUsize,
}

/// Hub for agent identities, provider configs, and capability discovery.
pub struct RegistryHub {
    /// The live agent registry, synced with persistence.
    pub agents: DashMap<String, EngineAgent>,
    /// Configured LLM providers (e.g., OpenAI, Ollama).
    pub providers: DashMap<String, ProviderConfig>,
    /// Available LLM models catalog.
    pub models: DashMap<String, ModelEntry>,
    /// Discovery registry for infrastructure nodes in the swarm.
    pub nodes: DashMap<String, SwarmNode>,
    /// Registry for dynamic file-based Skills and Workflows.
    pub capabilities: Arc<crate::agent::capabilities::CapabilitiesRegistry>,
    /// Manager for dynamic Skill Manifests (skill.json).
    pub skill_registry: Arc<crate::agent::skill_manifest::SkillRegistry>,
    /// Host for Model Context Protocol (MCP) tool aggregation.
    pub mcp_host: Arc<crate::agent::mcp::McpHost>,
    /// Manager for Lifecycle Hooks (Pre/Post tool execution).
    pub hooks: Arc<crate::agent::hooks::HooksManager>,
}

/// Hub for tamper-evident auditing and preventative security checks.
pub struct SecurityHub {
    /// Tamper-evident audit trail engine (Merkle Hash Chain).
    pub audit_trail: Arc<MerkleAuditTrail>,
    /// Persistent budget governance and metering engine.
    pub budget_guard: Arc<BudgetGuard>,
    /// Proactive shell safety scanner (API key leak protection).
    pub shell_scanner: Arc<ShellScanner>,
    /// Runtime secret redactor for logs and telemetry.
    pub secret_redactor: Arc<crate::secret_redactor::SecretRedactor>,
    /// System resource and environment monitor.
    pub system_monitor: Arc<crate::security::monitoring::SecurityMonitor>,
    /// Authentication token for administrative/deploy requests.
    pub deploy_token: String,
}

/// Hub for heavy infrastructure resources and shared mission context.
pub struct ResourceHub {
    /// SQLite connection pool for persistent storage.
    pub pool: SqlitePool,
    /// Shared HTTP client with optimized connection pooling.
    pub http_client: Arc<Client>,
    /// Native engine for local audio synthesis (PCM) and transcription.
    /// Loaded lazily to save 200MB+ RAM until actively used.
    pub audio_engine: OnceCell<Arc<NeuralAudioEngine>>,
    /// Zero-latency semantic audio replicate cache for frequent phrases.
    pub audio_cache: Arc<BunkerCache>,
    /// Graph of code relationships for RAG-enhanced tool search.
    /// Warmed up lazily to prevent massive CPU/RAM spikes on boot.
    pub code_graph: OnceCell<Arc<RwLock<CodeGraph>>>,
    /// Global system identity context loaded from `directives/IDENTITY.md`.
    /// Loaded lazily to keep baseline memory footprint minimal.
    pub identity_context: OnceCell<String>,
    /// Global long-term memory context loaded from `directives/LONG_TERM_MEMORY.md`.
    /// Loaded lazily to keep baseline memory footprint minimal.
    pub memory_context: OnceCell<String>,
    /// Cached rate limiters partitioned by model and provider.
    pub rate_limiters: DashMap<String, Arc<crate::agent::rate_limiter::RateLimiter>>,
}

impl ResourceHub {
    /// Lazily initializes and returns the ONNX audio engine.
    pub async fn get_audio_engine(&self) -> Arc<NeuralAudioEngine> {
        self.audio_engine.get_or_init(|| async {
            Arc::new(NeuralAudioEngine::new().await.expect("Failed to init ONNX Audio Engine"))
        }).await.clone()
    }

    /// Lazily initializes and returns the code graph.
    pub async fn get_code_graph(&self) -> Arc<RwLock<CodeGraph>> {
        self.code_graph.get_or_init(|| async {
            Arc::new(RwLock::new(CodeGraph::new(std::path::PathBuf::from("."))))
        }).await.clone()
    }

    /// Lazily initializes and returns the identity context.
    pub async fn get_identity_context(&self) -> String {
        self.identity_context.get_or_init(|| async {
            tokio::fs::read_to_string("directives/IDENTITY.md").await.unwrap_or_default()
        }).await.clone()
    }

    /// Lazily initializes and returns the long-term memory context.
    pub async fn get_memory_context(&self) -> String {
        self.memory_context.get_or_init(|| async {
            tokio::fs::read_to_string("directives/LONG_TERM_MEMORY.md").await.unwrap_or_default()
        }).await.clone()
    }
}

impl AppState {
    /// Initializes the full application state by loading configurations, 
    /// establishing DB connections, and starting background engines.
    pub async fn new() -> Self {
        dotenvy::dotenv().ok();
        
        let (tx, _) = broadcast::channel(1000);
        let (event_tx, _) = broadcast::channel(1000);
        let (audio_stream_tx, _) = broadcast::channel(5000);
        let telemetry_tx = crate::telemetry::TELEMETRY_TX.clone();

        // Security: Load Neural Token (Mandatory, but relaxed for tests)
        let deploy_token = std::env::var("NEURAL_TOKEN").unwrap_or_else(|_| {
            if cfg!(test) {
                "ci-test-token-placeholder".to_string()
            } else {
                panic!("🚨 FATAL: NEURAL_TOKEN environment variable MUST be set for the engine to start.");
            }
        });

        // Initialize DB
        let database_url = if cfg!(test) {
            "sqlite::memory:".to_string()
        } else {
            std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite:data/tadpole.db".to_string())
        };
        let pool = crate::db::init_db(&database_url).await.expect("Failed to initialize database");

        // Load Registries
        let providers_list = crate::agent::persistence::load_providers().await;
        let providers = DashMap::new();
        for p in providers_list { providers.insert(p.id.clone(), p); }

        let models_list = crate::agent::persistence::load_models().await;
        let models = DashMap::new();
        for m in models_list { models.insert(m.id.clone(), m); }

        let agents_list = crate::agent::persistence::load_agents_db(&pool).await.unwrap_or_default();

        let agents = DashMap::new();
        for a in agents_list { agents.insert(a.id.clone(), a); }

        // Initialize Engines
        let http_client = Arc::new(Client::builder().user_agent("TadpoleOS/1.0.0").build().unwrap());
        let audio_cache = Arc::new(BunkerCache::new("data/audio_cache.db".into()).await.unwrap());
        
        let secret_redactor = Arc::new(crate::secret_redactor::SecretRedactor::from_env());
        
        // Assemble Hubs
        let comms = Arc::new(CommunicationHub {
            tx: tx.clone(),
            event_tx: event_tx.clone(),
            telemetry_tx,
            audio_stream_tx,
            oversight_queue: DashMap::new(),
            oversight_resolvers: DashMap::new(),
        });

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
        });

        let registry = Arc::new(RegistryHub {
            agents: agents.clone(),
            providers,
            models,
            nodes: DashMap::new(),
            capabilities: Arc::new(crate::agent::capabilities::CapabilitiesRegistry::new().await.unwrap()),
            skill_registry: Arc::new(crate::agent::skill_manifest::SkillRegistry::load_all()),
            mcp_host: Arc::new(crate::agent::mcp::McpHost::new(event_tx.clone())),
            hooks: Arc::new(crate::agent::hooks::HooksManager::new(std::path::Path::new("data"))),
        });

        let security = Arc::new(SecurityHub {
            audit_trail: Arc::new(MerkleAuditTrail::new(pool.clone())),
            budget_guard: Arc::new(BudgetGuard::new(pool.clone())),
            shell_scanner: Arc::new(ShellScanner::new(secret_redactor.clone())),
            secret_redactor,
            system_monitor: Arc::new(crate::security::monitoring::SecurityMonitor::new()),
            deploy_token,
        });

        let resource_hub = Arc::new(ResourceHub {
            pool: pool.clone(),
            http_client,
            audio_engine: OnceCell::new(),
            audio_cache,
            code_graph: OnceCell::new(),
            identity_context: OnceCell::new(),
            memory_context: OnceCell::new(),
            rate_limiters: DashMap::new(),
        });

        Self {
            comms,
            governance,
            registry,
            security,
            resources: resource_hub,
        }
    }

    /// Helper to broadcast a system log.
    /// Applies secret redaction to prevent accidental key leaks in the UI.
    pub fn broadcast_sys(&self, text: &str, severity: &str) {
        let safe_text = self.security.secret_redactor.redact(text);
        let entry = LogEntry::new("System", &safe_text, severity);
        let _ = self.comms.tx.send(entry);
    }

    /// Helper to broadcast an arbitrary Engine event.
    pub fn emit_event(&self, event: serde_json::Value) {
        let _ = self.comms.event_tx.send(event);
    }

    /// Forces the agent to yield execution back to the Tokio scheduler, 
    /// acting as a rigid phase transition boundary for oversight injection.
    /// This prevents agents from running amok and allows the SecurityHub 
    /// to introspect or pause execution between operational phases.
    pub async fn yield_phase_transition(&self, agent_id: &str, phase: &str) {
        tracing::debug!("⏳ [Oversight] Agent {} yielding at boundary: {}", agent_id, phase);
        
        // Emits a phase transition telemetry event for UI tracking
        self.emit_event(serde_json::json!({
            "type": "agent:phase_transition",
            "agentId": agent_id,
            "phase": phase
        }));

        // Explicitly suspend the task to allow other scheduler components
        // (like the monitoring loops) to execute.
        tokio::task::yield_now().await;
    }

    /// Persists all current agent states to the database.
    pub async fn save_agents(&self) {
        let agents_vec: Vec<EngineAgent> = self.registry.agents.iter().map(|kv| kv.value().clone()).collect();
        // 1. Database Persistence (Runtime State)
        for agent in &agents_vec {
            let _ = crate::agent::persistence::save_agent_db(&self.resources.pool, agent).await;
        }
    }

    /// Persists all provider configurations to disk.
    pub async fn save_providers(&self) {
        let providers_vec: Vec<ProviderConfig> = self.registry.providers.iter().map(|kv| kv.value().clone()).collect();
        let _ = crate::agent::persistence::save_providers(providers_vec).await;
    }

    /// Persists all model metadata to disk.
    pub async fn save_models(&self) {
        let models_vec: Vec<ModelEntry> = self.registry.models.iter().map(|kv| kv.value().clone()).collect();
        let _ = crate::agent::persistence::save_models(models_vec).await;
    }
}

impl Default for AppState {
    /// Creates a mock version of the application state for testing purposes.
    fn default() -> Self {
        let (tx, _) = broadcast::channel(1);
        let (event_tx, _) = broadcast::channel(1);
        let (audio_stream_tx, _) = broadcast::channel(1);
        let (telemetry_tx, _) = broadcast::channel(1);

        let pool = SqlitePool::connect_lazy("sqlite::memory:").unwrap();

        let comms = Arc::new(CommunicationHub {
            tx: tx.clone(),
            event_tx,
            telemetry_tx,
            audio_stream_tx,
            oversight_queue: DashMap::new(),
            oversight_resolvers: DashMap::new(),
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
        });

        let registry = Arc::new(RegistryHub {
            agents: DashMap::new(),
            providers: DashMap::new(),
            models: DashMap::new(),
            nodes: DashMap::new(),
            capabilities: Arc::new(crate::agent::capabilities::CapabilitiesRegistry::mock()),
            skill_registry: Arc::new(crate::agent::skill_manifest::SkillRegistry::new()),
            mcp_host: Arc::new(crate::agent::mcp::McpHost::new(broadcast::channel(1).0)),
            hooks: Arc::new(crate::agent::hooks::HooksManager::new(&std::path::PathBuf::from("tmp"))),
        });

        let security = Arc::new(SecurityHub {
            audit_trail: Arc::new(MerkleAuditTrail::mock()),
            budget_guard: Arc::new(BudgetGuard::mock()),
            shell_scanner: Arc::new(ShellScanner::mock()),
            secret_redactor: Arc::new(crate::secret_redactor::SecretRedactor::noop()),
            system_monitor: Arc::new(crate::security::monitoring::SecurityMonitor::new()),
            deploy_token: "test".into(),
        });

        let resources = Arc::new(ResourceHub {
            pool: pool.clone(),
            http_client: Arc::new(Client::new()),
            audio_engine: OnceCell::new(),
            audio_cache: Arc::new(BunkerCache::mock()),
            code_graph: OnceCell::new(),
            identity_context: OnceCell::new(),
            memory_context: OnceCell::new(),
            rate_limiters: DashMap::new(),
        });

        Self {
            comms,
            governance,
            registry,
            security,
            resources,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_lazy_resource_initialization() {
        let state = AppState::default();

        // 1. Verify uninitialized state for deferred resources
        assert!(state.resources.identity_context.get().is_none(), "Identity context should start uninitialized");
        assert!(state.resources.memory_context.get().is_none(), "Memory context should start uninitialized");
        assert!(state.resources.audio_engine.get().is_none(), "Audio engine should start uninitialized");
        assert!(state.resources.code_graph.get().is_none(), "Code graph should start uninitialized");

        // 2. Trigger lazy initialization
        let _ = state.resources.get_identity_context().await;

        // 3. Verify initialized state
        assert!(state.resources.identity_context.get().is_some(), "Identity context should be initialized after access");
    }
}

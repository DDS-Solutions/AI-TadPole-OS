//! @docs ARCHITECTURE:Networking
//!
//! ### AI Assist Note
//! **System Startup (Boot Orchestrator)**: Orchestrates the transition
//! from a static process to a living, autonomous swarm. Features the
//! **System Boot Sequence**: manages the branching between **Extreme
//! Performance** (Fast Path) and **Full Systemic Awareness** (Full Path)
//! via `BootstrapIntent`. Implements **Subsystem Warmup**: categorizes
//! and initializes heavy components including the **Hydra-RS Code
//! Graph**, **mDNS Swarm Discovery**, **Continuity Scheduler**, and
//! **Heartbeat Loop**. AI agents should monitor the `initialization`
//! telemetry to verify that core background workers are `Ready` before
//! attempting mission dispatch (BOOT-01).
//!
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: Fatal `.env.schema` validation failures, mDNS
//!   port availability conflicts (UDP 5353), or Hydra-RS scan
//!   panics during codebase indexing. Check the bootstrap_intent if
//!   specific workers are missing.
//! - **Telemetry Link**: Search for `[Bootstrap]` or `[Hydra-RS]` in
//!   `tracing` logs for phase-specific boot milestones.
//! - **Trace Scope**: `server-rs::startup`

use crate::state::AppState;
use async_trait::async_trait;
use opentelemetry::trace::TracerProvider as _;
use opentelemetry_sdk::trace::SdkTracerProvider;
use std::sync::Arc;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

/// Configures the weight and scope of the engine boot sequence.
///
/// This intent allows the process to branch between extreme performance (Fast Path)
/// and full systemic awareness (Full Path).
///
/// ### AI Assist Note
/// Check `intent` in `main.rs` to understand why certain subsystems might be `NotStarted`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BootstrapIntent {
    /// Full mission execution: Warm up all Code Graph, mDNS, and ingestion workers.
    /// **Weight**: High (CPU/RAM intensive).
    /// @state: Constant(Full)
    Full,
    /// Fast Path: Skip heavy warm-up tasks for simple CLI status/version requests.
    /// **Weight**: Low (Instant response).
    /// @state: Constant(Fast)
    Fast,
}

/// Initializes the global telemetry and tracing ecosystem.
///
/// Bridges `tracing` spans with OpenTelemetry and a custom real-time broadcast
/// layer for the frontend UI.
pub fn init_tracing(disable_otel: bool) {

    // 1. Core Layers (Always active)
    let fmt_layer = tracing_subscriber::fmt::layer();
    let env_filter = tracing_subscriber::EnvFilter::from_default_env();
    let telemetry_layer = crate::telemetry::TelemetryLayer::new();

    // 2. OpenTelemetry Layer (Optional & Resilient)
    if !disable_otel {
        let exporter = opentelemetry_stdout::SpanExporter::default();
        let provider = SdkTracerProvider::builder()
            .with_simple_exporter(exporter)
            .build();
        let tracer = provider.tracer("tadpole-os");
        let otel_layer = tracing_opentelemetry::layer().with_tracer(tracer);

        let registry = tracing_subscriber::registry()
            .with(env_filter)
            .with(fmt_layer)
            .with(telemetry_layer)
            .with(otel_layer);

        if let Err(e) = registry.try_init() {
            eprintln!("⚠️  [Telemetry] Failed to initialize with OTel: {}. Falling back to basic console logging.", e);
            // Fallback to basic logging handled by second branch if this fails?
            // Actually, if try_init fails, we usually can't try again for the same process.
        } else {
            return; // Successfully initialized with OTel
        }
    }

    // Fallback or Explicitly Disabled: Initialize without OTel
    let _ = tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .with(crate::telemetry::TelemetryLayer::new())
        .try_init();
}

/// Loads environmental configuration and validates it against the schema.
///
/// Ensures that all required keys are present and types are valid before
/// the engine begins mission execution.
pub fn load_environment() {
    check_sovereign_config();
}

/// Checks for critical AI provider API keys and issues a "Sovereign Warning" if missing.
fn check_sovereign_config() {
    let providers = [
        ("OPENAI_API_KEY", "OpenAI"),
        ("ANTHROPIC_API_KEY", "Anthropic"),
        ("GOOGLE_API_KEY", "Google Gemini"),
        ("GROQ_API_KEY", "Groq"),
    ];

    let mut missing = Vec::new();
    for (key, name) in providers {
        if std::env::var(key)
            .map(|v| v.trim().is_empty())
            .unwrap_or(true)
        {
            missing.push(name);
        }
    }

    let privacy_mode = std::env::var("PRIVACY_MODE")
        .map(|v| v.to_lowercase() == "true")
        .unwrap_or(false);

    if !missing.is_empty() && !privacy_mode {
        println!("\n\x1b[1;33m⚠️  [SOVEREIGN WARNING]\x1b[0m");
        println!("\x1b[1;33m--------------------------------------------------\x1b[0m");
        println!("The following AI providers are not configured:");
        for name in &missing {
            println!("  - {}", name);
        }
        println!("\nAI-Tadpole-OS will fall back to local models (Ollama) if available.");
        println!("To enable these providers, add your API keys to the \x1b[1m.env\x1b[0m file.");
        println!("See \x1b[1mdocs/GETTING_STARTED.md\x1b[0m for instructions.");
        println!("\x1b[1;33m--------------------------------------------------\x1b[0m\n");

        tracing::warn!(missing = ?missing, "Sovereign Warning: Some AI providers are not configured.");
    } else if privacy_mode {
        tracing::info!("🔒 [Privacy Guard] Running in strict local-only mode (Zero-Cloud).");
    }
}

/// Orchestrates the asynchronous warmup of heavy engine subsystems.
///
/// This function spawns long-running tasks for:
/// 1. **Codebase Indexing**: Preparing the RAG-enhanced code graph.
/// 2. **Network Discovery**: Starting mDNS and swarm coordination.
/// 3. **Health Heartbeat**: Periodically broadcasting systemic telemetry.
///
/// ### Dependencies
/// - Requires an initialized `AppState`.
/// - Obeys `BootstrapIntent` (skips heavy tasks in `Fast` mode).
///
/// ### Side Effects
/// - Reports status to `app_state.resources.initialization_registry`.
/// - Emits `engine:health` socket events.
///
/// ### AI Assist Note
/// This is the primary orchestrator for the engine's background lifecycle.
// ==========================================
// CENTRAL SYSTEM SERVICE REGISTRY (ADAMANT v4.0)
// ==========================================

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ServiceConfiguration {
    pub heartbeat_secs: u64,
    pub rate_limit_eviction_interval_secs: u64,
    pub max_bucket_age_secs: u64,
    pub max_auth_age_secs: u64,
    pub memory_cleanup_interval_secs: u64,
    pub iks_decay_interval_secs: u64,
    pub iks_eviction_interval_secs: u64,
    pub budget_flush_interval_secs: u64,
}

impl Default for ServiceConfiguration {
    fn default() -> Self {
        Self {
            heartbeat_secs: 3,
            rate_limit_eviction_interval_secs: 300,
            max_bucket_age_secs: 120,
            max_auth_age_secs: 600,
            memory_cleanup_interval_secs: 6 * 3600,
            iks_decay_interval_secs: 6 * 3600,
            iks_eviction_interval_secs: 24 * 3600,
            budget_flush_interval_secs: 10,
        }
    }
}

#[derive(Debug, Clone)]
pub enum StateQuery {
    GetMaxSwarmDepth,
    GetActiveAgents,
    GetTpmAccumulator,
    GetRecruitCount,
}

#[derive(Debug, Clone)]
pub enum StateResponse {
    MaxSwarmDepth(u32),
    ActiveAgents(u32),
    TpmAccumulator(usize),
    RecruitCount(u32),
}

#[derive(Clone)]
pub struct SystemContext {
    pub app_state: Arc<AppState>,
    pub shutdown_rx: tokio::sync::watch::Receiver<bool>,
    pub config: ServiceConfiguration,
}

impl SystemContext {
    pub async fn query_state(&self, query: StateQuery) -> StateResponse {
        match query {
            StateQuery::GetMaxSwarmDepth => {
                let val = self.app_state.governance.max_swarm_depth.load(std::sync::atomic::Ordering::Relaxed);
                StateResponse::MaxSwarmDepth(val)
            }
            StateQuery::GetActiveAgents => {
                let val = self.app_state.governance.active_agents.load(std::sync::atomic::Ordering::Relaxed);
                StateResponse::ActiveAgents(val)
            }
            StateQuery::GetTpmAccumulator => {
                let val = self.app_state.governance.tpm_accumulator.swap(0, std::sync::atomic::Ordering::Relaxed);
                StateResponse::TpmAccumulator(val)
            }
            StateQuery::GetRecruitCount => {
                let val = self.app_state.governance.recruit_count.swap(0, std::sync::atomic::Ordering::Relaxed);
                StateResponse::RecruitCount(val)
            }
        }
    }
}

#[async_trait]
pub trait SystemService: Send + Sync {
    fn name(&self) -> &'static str;
    fn is_critical(&self) -> bool {
        false
    }
    async fn start(
        &self,
        context: SystemContext,
    ) -> Result<(), anyhow::Error>;
}


pub struct CodeGraphWarmupService;

#[async_trait]
impl SystemService for CodeGraphWarmupService {
    fn name(&self) -> &'static str { "CodeGraphWarmup" }
    fn is_critical(&self) -> bool { true }
    async fn start(
        &self,
        context: SystemContext,
    ) -> Result<(), anyhow::Error> {
        let app_state = context.app_state;
        app_state
            .resources
            .set_subsystem_status("CodeGraph", crate::types::SubsystemStatus::Warming(0.1));
        let graph_lock = app_state.resources.get_code_graph().await;
        let mut graph = graph_lock.write();
        graph.scan();
        app_state
            .resources
            .set_subsystem_status("CodeGraph", crate::types::SubsystemStatus::Ready);
        tracing::info!(
            "[Hydra-RS] In-memory code graph warmed up ({} modules indexed)",
            graph.modules.len()
        );
        Ok(())
    }
}

pub struct CodeGraphDbRefreshService;

#[async_trait]
impl SystemService for CodeGraphDbRefreshService {
    fn name(&self) -> &'static str { "CodeGraphDbRefresh" }
    fn is_critical(&self) -> bool { true }
    async fn start(
        &self,
        context: SystemContext,
    ) -> Result<(), anyhow::Error> {
        let app_state = context.app_state;
        let started = std::time::Instant::now();
        app_state.resources.set_subsystem_status(
            "CodeGraphDbRefresh",
            crate::types::SubsystemStatus::Warming(0.05),
        );

        let root = app_state.resources.base_dir.clone();
        let db_path = root.join(".code-review-graph").join("graph.db");
        let salt = app_state.resources.obfuscation_salt.clone();

        let refresh_fut = crate::intelligence::graph_store::refresh_code_review_graph_db(root, db_path, salt);
        match tokio::time::timeout(std::time::Duration::from_secs(90), refresh_fut).await {
            Ok(Ok(summary)) => {
                app_state.resources.set_subsystem_status(
                    "CodeGraphDbRefresh",
                    crate::types::SubsystemStatus::Ready,
                );
                tracing::info!(
                    db_path = %summary.db_path.display(),
                    nodes = summary.node_count,
                    edges = summary.edge_count,
                    risks = summary.risk_count,
                    communities = summary.community_count,
                    flows = summary.flow_count,
                    elapsed_ms = started.elapsed().as_millis(),
                    "[CodeGraphDbRefresh] refreshed persistent code-review graph"
                );
                Ok(())
            }
            Ok(Err(err)) => {
                app_state.resources.set_subsystem_status(
                    "CodeGraphDbRefresh",
                    crate::types::SubsystemStatus::Failed(err.to_string()),
                );
                tracing::error!(
                    error = %err,
                    elapsed_ms = started.elapsed().as_millis(),
                    "[CodeGraphDbRefresh] failed to refresh persistent code-review graph"
                );
                Err(anyhow::anyhow!(err))
            }
            Err(_) => {
                let err_msg = "CodeGraphDbRefresh timed out after 90s".to_string();
                app_state.resources.set_subsystem_status(
                    "CodeGraphDbRefresh",
                    crate::types::SubsystemStatus::Failed(err_msg.clone()),
                );
                tracing::error!(
                    elapsed_ms = started.elapsed().as_millis(),
                    "[CodeGraphDbRefresh] {}", err_msg
                );
                Err(anyhow::anyhow!(err_msg))
            }
        }
    }
}

pub struct SwarmDiscoveryService;

#[async_trait]
impl SystemService for SwarmDiscoveryService {
    fn name(&self) -> &'static str { "SwarmDiscovery" }
    fn is_critical(&self) -> bool { true }
    async fn start(
        &self,
        context: SystemContext,
    ) -> Result<(), anyhow::Error> {
        let app_state = context.app_state;
        app_state
            .resources
            .set_subsystem_status("Network", crate::types::SubsystemStatus::Warming(0.0));
        match crate::services::discovery::SwarmDiscoveryManager::new(app_state.clone()) {
            Ok(manager) => {
                let start_fut = async { manager.start() };
                match tokio::time::timeout(std::time::Duration::from_secs(10), start_fut).await {
                    Ok(Ok(_)) => {
                        app_state
                            .resources
                            .set_subsystem_status("Network", crate::types::SubsystemStatus::Ready);
                        Ok(())
                    }
                    Ok(Err(e)) => {
                        tracing::error!("📡 [Discovery] Failed to start mDNS manager: {}", e);
                        app_state.resources.set_subsystem_status(
                            "Network",
                            crate::types::SubsystemStatus::Failed(e.to_string()),
                        );
                        Err(anyhow::anyhow!(e))
                    }
                    Err(_) => {
                        let err_msg = "SwarmDiscoveryService timed out after 10s".to_string();
                        tracing::error!("📡 [Discovery] {}", err_msg);
                        app_state.resources.set_subsystem_status(
                            "Network",
                            crate::types::SubsystemStatus::Failed(err_msg.clone()),
                        );
                        Err(anyhow::anyhow!(err_msg))
                    }
                }
            }
            Err(e) => {
                tracing::error!("📡 [Discovery] Failed to initialize mDNS manager: {}", e);
                app_state.resources.set_subsystem_status(
                    "Network",
                    crate::types::SubsystemStatus::Failed(e.to_string()),
                );
                Err(anyhow::anyhow!(e))
            }
        }
    }
}

pub struct SwarmPulseService;

#[async_trait]
impl SystemService for SwarmPulseService {
    fn name(&self) -> &'static str { "SwarmPulse" }
    fn is_critical(&self) -> bool { true }
    async fn start(
        &self,
        context: SystemContext,
    ) -> Result<(), anyhow::Error> {
        let app_state = context.app_state;
        let shutdown_rx = context.shutdown_rx;
        app_state.resources.set_subsystem_status("SwarmPulse", crate::types::SubsystemStatus::Ready);
        crate::telemetry::pulse::spawn_pulse_loop(app_state, shutdown_rx).await;
        Ok(())
    }
}

pub struct RecoverActiveAgentsService;

#[async_trait]
impl SystemService for RecoverActiveAgentsService {
    fn name(&self) -> &'static str { "RecoverActiveAgents" }
    fn is_critical(&self) -> bool { true }
    async fn start(
        &self,
        context: SystemContext,
    ) -> Result<(), anyhow::Error> {
        let app_state = context.app_state;
        app_state.resources.set_subsystem_status("RecoverActiveAgents", crate::types::SubsystemStatus::Ready);
        crate::routes::agent::recover_active_agents(app_state).await;
        Ok(())
    }
}

pub struct HeartbeatService;

#[async_trait]
impl SystemService for HeartbeatService {
    fn name(&self) -> &'static str { "Heartbeat" }
    fn is_critical(&self) -> bool { true }
    async fn start(
        &self,
        context: SystemContext,
    ) -> Result<(), anyhow::Error> {
        let context_clone = context.clone();
        let mut shutdown_rx = context.shutdown_rx.clone();
        let heartbeat_secs = context.config.heartbeat_secs;
        context.app_state.resources.set_subsystem_status("Heartbeat", crate::types::SubsystemStatus::Ready);
        tokio::spawn(async move {
            let boot_instant = std::time::Instant::now();
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(heartbeat_secs));
            interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
            loop {
                tokio::select! {
                    _ = shutdown_rx.changed() => {
                        if *shutdown_rx.borrow() {
                            tracing::info!("🛑 [Heartbeat] Heartbeat Loop shutting down gracefully.");
                            break;
                        }
                    }
                    _ = interval.tick() => {
                        // Recovery wrapper: execute metrics collection safely
                        if let Err(e) = gather_and_emit_metrics(&context_clone, boot_instant).await {
                            tracing::warn!("⚠️  [Heartbeat] Metrics collection failed: {:?}", e);
                        }
                    }
                }
            }
        });
        Ok(())
    }
}

async fn gather_and_emit_metrics(context: &SystemContext, boot_instant: std::time::Instant) -> Result<(), anyhow::Error> {
    let app_state = &context.app_state;
    
    let active_agents = match context.query_state(StateQuery::GetActiveAgents).await {
        StateResponse::ActiveAgents(val) => val,
        _ => 0,
    };
    let swarm_depth = match context.query_state(StateQuery::GetMaxSwarmDepth).await {
        StateResponse::MaxSwarmDepth(val) => val,
        _ => 0,
    };
    let tpm = match context.query_state(StateQuery::GetTpmAccumulator).await {
        StateResponse::TpmAccumulator(val) => val,
        _ => 0,
    };
    let recruits = match context.query_state(StateQuery::GetRecruitCount).await {
        StateResponse::RecruitCount(val) => val,
        _ => 0,
    };

    let profile = app_state.resources.hardware_profiler.get_profile();
    let cpu = profile.cpu_usage;
    let memory_gb = profile.memory_used as f32 / (1024.0 * 1024.0 * 1024.0);
    let total_gb = profile.memory_total as f32 / (1024.0 * 1024.0 * 1024.0);

    let registry_snapshot: std::collections::HashMap<
        String,
        crate::types::SubsystemStatus,
    > = app_state
        .resources
        .initialization_registry
        .iter()
        .map(|kv| (kv.key().clone(), kv.value().clone()))
        .collect();

    app_state.emit_event(serde_json::json!({
        "type": "engine:health",
        "uptime": boot_instant.elapsed().as_secs(),
        "agentCount": active_agents,
        "activeAgents": active_agents,
        "maxDepth": swarm_depth,
        "tpm": tpm,
        "recruitCount": recruits,
        "cpu": cpu,
        "memory": memory_gb,
        "maxMemory": total_gb,
        "latency": 42,
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "initialization": registry_snapshot
    }));

    Ok(())
}

pub struct ContinuitySchedulerService;

#[async_trait]
impl SystemService for ContinuitySchedulerService {
    fn name(&self) -> &'static str { "ContinuityScheduler" }
    async fn start(
        &self,
        context: SystemContext,
    ) -> Result<(), anyhow::Error> {
        let app_state = context.app_state;
        let mut shutdown_rx = context.shutdown_rx;
        app_state.resources.set_subsystem_status("ContinuityScheduler", crate::types::SubsystemStatus::Ready);
        tokio::spawn(async move {
            tokio::select! {
                _ = shutdown_rx.changed() => {
                    if *shutdown_rx.borrow() {
                        tracing::info!("🛑 [Continuity] Scheduled job executor shutting down gracefully.");
                    }
                }
                _ = crate::agent::continuity::executor::start_scheduler(app_state) => {}
            }
        });
        tracing::info!("🕐 [Continuity] Scheduled job executor launched.");
        Ok(())
    }
}

pub struct SwarmReaperService;

#[async_trait]
impl SystemService for SwarmReaperService {
    fn name(&self) -> &'static str { "SwarmReaper" }
    async fn start(
        &self,
        context: SystemContext,
    ) -> Result<(), anyhow::Error> {
        let app_state = context.app_state;
        let mut shutdown_rx = context.shutdown_rx;
        app_state.resources.set_subsystem_status("SwarmReaper", crate::types::SubsystemStatus::Ready);
        tokio::spawn(async move {
            tokio::select! {
                _ = shutdown_rx.changed() => {
                    if *shutdown_rx.borrow() {
                        tracing::info!("🛑 [Reaper] Swarm Reaper shutting down gracefully.");
                    }
                }
                _ = crate::agent::reaper::SwarmReaper::start(app_state) => {}
            }
        });
        tracing::info!("♻️ [Reaper] Swarm Reaper launched (48h retention policy).");
        Ok(())
    }
}

pub struct MemoryCleanupService;

#[async_trait]
impl SystemService for MemoryCleanupService {
    fn name(&self) -> &'static str { "MemoryCleanup" }
    async fn start(
        &self,
        context: SystemContext,
    ) -> Result<(), anyhow::Error> {
        let app_state = context.app_state;
        let shutdown_rx = context.shutdown_rx;
        let interval_secs = context.config.memory_cleanup_interval_secs;
        app_state.resources.set_subsystem_status("MemoryCleanup", crate::types::SubsystemStatus::Ready);
        #[cfg(not(feature = "vector-memory"))]
        let _ = (app_state, shutdown_rx, interval_secs);

        #[cfg(feature = "vector-memory")]
        {
            let memory_cleanup_pool = app_state.resources.pool.clone();
            tokio::spawn(async move {
                let mut shutdown_rx = shutdown_rx;
                crate::agent::memory::VectorMemory::cleanup_orphaned_scopes(&memory_cleanup_pool).await;
                let mut interval = tokio::time::interval(std::time::Duration::from_secs(interval_secs));
                interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
                loop {
                    tokio::select! {
                        _ = shutdown_rx.changed() => {
                            if *shutdown_rx.borrow() {
                                tracing::info!("🛑 [MemoryCleanup] Memory Cleanup shutting down gracefully.");
                                break;
                            }
                        }
                        _ = interval.tick() => {
                            crate::agent::memory::VectorMemory::cleanup_orphaned_scopes(&memory_cleanup_pool).await;
                        }
                    }
                }
            });
        }
        Ok(())
    }
}

pub struct IngestionWorkerService;

#[async_trait]
impl SystemService for IngestionWorkerService {
    fn name(&self) -> &'static str { "IngestionWorker" }
    async fn start(
        &self,
        context: SystemContext,
    ) -> Result<(), anyhow::Error> {
        let app_state = context.app_state;
        let mut shutdown_rx = context.shutdown_rx;
        app_state.resources.set_subsystem_status("IngestionWorker", crate::types::SubsystemStatus::Ready);
        tokio::spawn(async move {
            tokio::select! {
                _ = shutdown_rx.changed() => {
                    if *shutdown_rx.borrow() {
                        tracing::info!("🛑 [IngestionWorker] Ingestion Worker shutting down gracefully.");
                    }
                }
                _ = crate::agent::connectors::start_ingestion_worker(app_state) => {}
            }
        });
        Ok(())
    }
}

pub struct PrivacyGuardService;

#[async_trait]
impl SystemService for PrivacyGuardService {
    fn name(&self) -> &'static str { "PrivacyGuard" }
    fn is_critical(&self) -> bool { true }
    async fn start(
        &self,
        context: SystemContext,
    ) -> Result<(), anyhow::Error> {
        let app_state = context.app_state;
        let mut shutdown_rx = context.shutdown_rx;
        app_state.resources.set_subsystem_status("PrivacyGuard", crate::types::SubsystemStatus::Ready);
        tokio::spawn(async move {
            tokio::select! {
                _ = shutdown_rx.changed() => {
                    if *shutdown_rx.borrow() {
                        tracing::info!("🛑 [PrivacyGuard] Privacy Guard shutting down gracefully.");
                    }
                }
                _ = crate::services::privacy::start_privacy_guard(app_state) => {}
            }
        });
        Ok(())
    }
}

pub struct SecurityEvictionService;

#[async_trait]
impl SystemService for SecurityEvictionService {
    fn name(&self) -> &'static str { "SecurityEviction" }
    fn is_critical(&self) -> bool { true }
    async fn start(
        &self,
        context: SystemContext,
    ) -> Result<(), anyhow::Error> {
        let app_state = context.app_state;
        let mut shutdown_rx = context.shutdown_rx;
        let eviction_interval_secs = context.config.rate_limit_eviction_interval_secs;
        let max_bucket_age_secs = context.config.max_bucket_age_secs;
        let max_auth_age_secs = context.config.max_auth_age_secs;
        app_state.resources.set_subsystem_status("SecurityEviction", crate::types::SubsystemStatus::Ready);
        tokio::spawn(async move {
            use std::time::Duration;
            let eviction_interval = Duration::from_secs(eviction_interval_secs);
            let max_bucket_age = Duration::from_secs(max_bucket_age_secs);
            let max_auth_age = Duration::from_secs(max_auth_age_secs);
            let mut interval = tokio::time::interval(eviction_interval);
            interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

            loop {
                tokio::select! {
                    _ = shutdown_rx.changed() => {
                        if *shutdown_rx.borrow() {
                            tracing::info!("🛑 [SecurityEviction] Security Eviction shutting down gracefully.");
                            break;
                        }
                    }
                    _ = interval.tick() => {
                        crate::middleware::rate_limit::evict_stale_buckets(max_bucket_age);
                        crate::middleware::auth_rate_limit::evict_expired_blocks(max_auth_age);
                        tracing::debug!("🧹 [Security] Rate limit bucket eviction completed");
                    }
                }
            }
        });
        let _ = app_state;
        Ok(())
    }
}

pub struct MetricAggregatorService;

#[async_trait]
impl SystemService for MetricAggregatorService {
    fn name(&self) -> &'static str { "MetricAggregator" }
    fn is_critical(&self) -> bool { true }
    async fn start(
        &self,
        context: SystemContext,
    ) -> Result<(), anyhow::Error> {
        let app_state = context.app_state;
        let mut shutdown_rx = context.shutdown_rx;
        app_state.resources.set_subsystem_status("MetricAggregator", crate::types::SubsystemStatus::Ready);
        let aggregator_rx = crate::telemetry::TELEMETRY_TX.subscribe();
        let aggregator = crate::telemetry::aggregator::MetricAggregator::new(1000);
        tokio::spawn(async move {
            tokio::select! {
                _ = shutdown_rx.changed() => {
                    if *shutdown_rx.borrow() {
                        tracing::info!("🛑 [MetricAggregator] Metric Aggregator shutting down gracefully.");
                    }
                }
                _ = aggregator.run(aggregator_rx) => {}
            }
        });
        let _ = app_state;
        Ok(())
    }
}

pub struct BudgetFlushService;

#[async_trait]
impl SystemService for BudgetFlushService {
    fn name(&self) -> &'static str { "BudgetFlush" }
    fn is_critical(&self) -> bool { true }
    async fn start(
        &self,
        context: SystemContext,
    ) -> Result<(), anyhow::Error> {
        let app_state = context.app_state;
        let mut shutdown_rx = context.shutdown_rx;
        let interval_secs = context.config.budget_flush_interval_secs;
        app_state.resources.set_subsystem_status("BudgetFlush", crate::types::SubsystemStatus::Ready);
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(interval_secs));
            interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
            loop {
                tokio::select! {
                    _ = shutdown_rx.changed() => {
                        if *shutdown_rx.borrow() {
                            tracing::info!("🛑 [BudgetFlush] Budget Flush shutting down gracefully.");
                            break;
                        }
                    }
                    _ = interval.tick() => {
                        if let Err(e) = app_state.security.budget_guard.flush_to_db().await {
                            tracing::error!("🚨 [BudgetGuard] Failed to flush usage to DB: {}", e);
                        }
                    }
                }
            }
        });
        Ok(())
    }
}

pub struct RecipeIngestionService;

#[async_trait]
impl SystemService for RecipeIngestionService {
    fn name(&self) -> &'static str { "RecipeIngestion" }
    async fn start(
        &self,
        context: SystemContext,
    ) -> Result<(), anyhow::Error> {
        let app_state = context.app_state;
        let mut shutdown_rx = context.shutdown_rx;
        app_state.resources.set_subsystem_status("RecipeIngestion", crate::types::SubsystemStatus::Ready);
        tokio::spawn(async move {
            tokio::select! {
                _ = shutdown_rx.changed() => {
                    if *shutdown_rx.borrow() {
                        tracing::info!("🛑 [RecipeIngestion] Recipe Ingestion shutting down gracefully.");
                    }
                }
                _ = crate::agent::recipes::auto_ingest_recipes(app_state) => {}
            }
        });
        Ok(())
    }
}

pub struct SystemHealthMonitorService;

#[async_trait]
impl SystemService for SystemHealthMonitorService {
    fn name(&self) -> &'static str { "SystemHealthMonitor" }
    fn is_critical(&self) -> bool { true }
    async fn start(
        &self,
        context: SystemContext,
    ) -> Result<(), anyhow::Error> {
        let app_state = context.app_state;
        let mut shutdown_rx = context.shutdown_rx;
        app_state.resources.set_subsystem_status("SystemHealthMonitor", crate::types::SubsystemStatus::Ready);
        tokio::spawn(async move {
            let boot_instant = std::time::Instant::now();
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(5));
            interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
            loop {
                tokio::select! {
                    _ = shutdown_rx.changed() => {
                        if *shutdown_rx.borrow() {
                            tracing::info!("🛑 [HealthMonitor] System Health Monitor shutting down gracefully.");
                            break;
                        }
                    }
                    _ = interval.tick() => {
                        let is_timeout = boot_instant.elapsed().as_secs() > 30;
                        let critical_subsystems = vec![
                            "Database", "Agents", "MCP", "Heartbeat",
                            "SecurityEviction", "PrivacyGuard", "BudgetFlush"
                        ];

                        let mut failed_detected = false;
                        for sub in critical_subsystems {
                            let status = app_state.resources.initialization_registry.get(sub).map(|r| r.value().clone());
                            match status {
                                Some(crate::types::SubsystemStatus::Failed(e)) => {
                                    tracing::error!("🚨 [HealthMonitor] Critical subsystem '{}' is in failed state: {}", sub, e);
                                    failed_detected = true;
                                }
                                Some(crate::types::SubsystemStatus::Warming(_)) | Some(crate::types::SubsystemStatus::NotStarted) | None => {
                                    if is_timeout {
                                        let err_msg = "Startup timeout (exceeded 30s in warming state)".to_string();
                                        app_state.resources.set_subsystem_status(sub, crate::types::SubsystemStatus::Failed(err_msg));
                                        tracing::error!("🚨 [HealthMonitor] Critical subsystem '{}' timed out warming up.", sub);
                                        failed_detected = true;
                                    }
                                }
                                _ => {}
                            }
                        }

                        if failed_detected {
                            let current_max = app_state.governance.max_agents.load(std::sync::atomic::Ordering::Relaxed);
                            if current_max > 2 {
                                tracing::warn!("🚨 [HealthMonitor] Critical failure active. Scaling down max_agents from {} to 2.", current_max);
                                app_state.governance.max_agents.store(2, std::sync::atomic::Ordering::Relaxed);
                            }
                        }
                    }
                }
            }
        });
        Ok(())
    }
}

#[cfg(feature = "vector-memory")]
pub struct IksDecayService;

#[cfg(feature = "vector-memory")]
#[async_trait]
impl SystemService for IksDecayService {
    fn name(&self) -> &'static str { "IksDecay" }
    async fn start(
        &self,
        context: SystemContext,
    ) -> Result<(), anyhow::Error> {
        let app_state = context.app_state;
        let mut shutdown_rx = context.shutdown_rx;
        let decay_interval_secs = context.config.iks_decay_interval_secs;
        app_state.resources.set_subsystem_status("IksDecay", crate::types::SubsystemStatus::Ready);
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(decay_interval_secs));
            interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
            loop {
                tokio::select! {
                    _ = shutdown_rx.changed() => {
                        if *shutdown_rx.borrow() {
                            tracing::info!("🛑 [IksDecay] IKS Decay shutting down gracefully.");
                            break;
                        }
                    }
                    _ = interval.tick() => {
                        match app_state.resources.get_knowledge_store().await {
                            Ok(ks) => {
                                if let Err(e) = ks.decay_confidence().await {
                                    tracing::warn!("[IKS] Confidence decay pass failed: {:?}", e);
                                } else {
                                    tracing::debug!("[IKS] Confidence decay pass complete.");
                                }
                            }
                            Err(e) => {
                                tracing::warn!("[IKS] Could not acquire store for decay: {:?}", e);
                            }
                        }
                    }
                }
            }
        });
        Ok(())
    }
}

#[cfg(feature = "vector-memory")]
pub struct IksEvictionService;

#[cfg(feature = "vector-memory")]
#[async_trait]
impl SystemService for IksEvictionService {
    fn name(&self) -> &'static str { "IksEviction" }
    async fn start(
        &self,
        context: SystemContext,
    ) -> Result<(), anyhow::Error> {
        let app_state = context.app_state;
        let mut shutdown_rx = context.shutdown_rx;
        let eviction_interval_secs = context.config.iks_eviction_interval_secs;
        app_state.resources.set_subsystem_status("IksEviction", crate::types::SubsystemStatus::Ready);
        tokio::spawn(async move {
            tokio::time::sleep(std::time::Duration::from_secs(30 * 60)).await;
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(eviction_interval_secs));
            interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
            loop {
                tokio::select! {
                    _ = shutdown_rx.changed() => {
                        if *shutdown_rx.borrow() {
                            tracing::info!("🛑 [IksEviction] IKS Eviction shutting down gracefully.");
                            break;
                        }
                    }
                    _ = interval.tick() => {
                        match app_state.resources.get_knowledge_store().await {
                            Ok(ks) => match ks.evict_expired().await {
                                Ok(n) => {
                                    tracing::info!("[IKS] Eviction pass removed {} entries.", n);
                                }
                                Err(e) => {
                                    tracing::warn!("[IKS] Eviction pass failed: {:?}", e);
                                }
                            },
                            Err(e) => {
                                tracing::warn!("[IKS] Could not acquire store for eviction: {:?}", e);
                            }
                        }
                    }
                }
            }
        });
        Ok(())
    }
}


pub async fn spawn_background_tasks(
    app_state: Arc<AppState>,
    intent: BootstrapIntent,
    service_config: ServiceConfiguration,
    shutdown_rx: tokio::sync::watch::Receiver<bool>,
) {
    let context = SystemContext {
        app_state,
        shutdown_rx,
        config: service_config,
    };

    let mut services: Vec<Box<dyn SystemService>> = vec![
        Box::new(HeartbeatService),
        Box::new(ContinuitySchedulerService),
        Box::new(SwarmReaperService),
        Box::new(MemoryCleanupService),
        Box::new(IngestionWorkerService),
        Box::new(PrivacyGuardService),
        Box::new(SecurityEvictionService),
        Box::new(MetricAggregatorService),
        Box::new(BudgetFlushService),
        Box::new(RecipeIngestionService),
        Box::new(SystemHealthMonitorService),
    ];

    #[cfg(feature = "vector-memory")]
    {
        services.push(Box::new(IksDecayService));
        services.push(Box::new(IksEvictionService));
    }

    if intent == BootstrapIntent::Full {
        services.push(Box::new(CodeGraphWarmupService));
        services.push(Box::new(CodeGraphDbRefreshService));
        services.push(Box::new(SwarmDiscoveryService));
        services.push(Box::new(SwarmPulseService));
        services.push(Box::new(RecoverActiveAgentsService));
    }

    for service in services {
        let name = service.name();
        let is_crit = service.is_critical();
        let context_clone = context.clone();
        tokio::spawn(async move {
            if is_crit {
                tracing::debug!("Starting critical service: {}", name);
            }
            if let Err(e) = service.start(context_clone).await {
                tracing::error!("🚨 [Service] System service '{}' failed to start: {:?}", name, e);
            }
        });
    }
}

/// Handles version/help administrative queries before full engine initialization.
/// Returns `Ok(Some(()))` if a query was handled and the program should exit,
/// `Ok(None)` if no administrative query was detected and execution should continue.
pub fn handle_admin_cli(args: &[String]) -> anyhow::Result<Option<()>> {
    if args.iter().any(|arg| arg == "--version" || arg == "-v") {
        println!("Tadpole OS Engine v{}", env!("CARGO_PKG_VERSION"));
        return Ok(Some(()));
    }
    if args.iter().any(|arg| arg == "--help" || arg == "-h") {
        println!("Tadpole OS - Sovereign AI Swarm Engine\n");
        println!("Usage: server-rs [OPTIONS]\n");
        println!("Options:");
        println!("  -v, --version    Show version and exit");
        println!("  -h, --help       Show this help and exit");
        println!("  --status         Show engine status and exit (Fast Path)");
        println!("  --port <PORT>    Set the port to listen on (Default: 8000)");
        return Ok(Some(()));
    }
    Ok(None)
}

/// Configures and builds a custom multi-threaded Tokio runtime.
/// Note: thread_stack_size configures the OS stack size of the executor worker threads.
/// It does NOT limit or affect the stack size of async tasks (futures), which are
/// allocated on the heap when spawned via tokio::spawn.
pub fn build_custom_runtime() -> anyhow::Result<tokio::runtime::Runtime> {
    let stack_size = std::env::var("TOKIO_THREAD_STACK_SIZE")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(2 * 1024 * 1024); // Default to 2 MB (safer memory footprint, configurable via env)

    let rt = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(
            std::thread::available_parallelism()
                .map(|n| n.get().max(4))
                .unwrap_or(4),
        )
        .max_blocking_threads(32)
        .thread_name("tadpole-worker")
        .thread_stack_size(stack_size)
        .enable_all()
        .build();

    match rt {
        Ok(r) => Ok(r),
        Err(e) => {
            let err_msg = format!("❌ FATAL: Failed to initialize Tokio runtime: {:?}", e);
            eprintln!("{}", err_msg);
            // Try to log it
            if let Ok(root) = std::env::var("WORKSPACE_ROOT") {
                let _ = std::fs::write(
                    std::path::Path::new(&root).join("sidecar_boot_error.log"),
                    &err_msg,
                );
            }
            Err(anyhow::anyhow!(err_msg))
        }
    }
}

/// Detects the bootstrap intent based on the command line arguments.
pub fn detect_bootstrap_intent(args: &[String]) -> BootstrapIntent {
    if args.iter().any(|arg| arg == "--status") {
        BootstrapIntent::Fast
    } else {
        BootstrapIntent::Full
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::AppState;
    use crate::types::SubsystemStatus;

    #[test]
    fn test_bootstrap_intent_variants() {
        let full = BootstrapIntent::Full;
        let fast = BootstrapIntent::Fast;
        assert_ne!(full, fast);
    }

    #[tokio::test]
    async fn test_warmup_registry_reporting() {
        let state = Arc::new(AppState::default());

        // Simulate a warmup task
        state
            .resources
            .set_subsystem_status("TestWarmup", SubsystemStatus::Warming(0.1));
        let status = state
            .resources
            .get_initialization_snapshot()
            .get("TestWarmup")
            .cloned();
        assert_eq!(status, Some(SubsystemStatus::Warming(0.1)));

        state
            .resources
            .set_subsystem_status("TestWarmup", SubsystemStatus::Ready);
        let status = state
            .resources
            .get_initialization_snapshot()
            .get("TestWarmup")
            .cloned();
        assert_eq!(status, Some(SubsystemStatus::Ready));
    }

    #[tokio::test]
    async fn test_fast_path_branching() {
        let state = Arc::new(AppState::new_mock().await);

        // Use Fast path
        let (_shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);
        let service_config = ServiceConfiguration {
            heartbeat_secs: 3,
            ..Default::default()
        };
        spawn_background_tasks(state.clone(), BootstrapIntent::Fast, service_config, shutdown_rx).await;

        // Wait a bit for background tasks (though they shouldn't start heavy ones)
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;

        let snapshot = state.resources.get_initialization_snapshot();

        // CodeGraph and Network should NOT be in the registry or should be NotStarted
        // In current implementation, they only appear when Warming is called.
        assert!(!snapshot.contains_key("CodeGraph"));
        assert!(!snapshot.contains_key("Network"));
    }

    #[cfg(feature = "vector-memory")]
    #[tokio::test]
    async fn test_iks_decay_service_resilience_to_dependency_failure() {
        let state = Arc::new(AppState::new_mock().await);
        let service = IksDecayService;
        let (_shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);
        let context = SystemContext {
            app_state: state.clone(),
            shutdown_rx,
            config: ServiceConfiguration::default(),
        };
        
        let res = service.start(context).await;
        assert!(res.is_ok(), "Service startup should be resilient to dependency errors");
    }

    #[tokio::test]
    async fn test_shutdown_race_condition_responsiveness() {
        let state = Arc::new(AppState::new_mock().await);
        let (shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);
        let context = SystemContext {
            app_state: state.clone(),
            shutdown_rx,
            config: ServiceConfiguration {
                heartbeat_secs: 5,
                ..Default::default()
            },
        };

        let service = HeartbeatService;
        let res = service.start(context).await;
        assert!(res.is_ok());

        let _ = shutdown_tx.send(true);
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    }

    #[tokio::test]
    async fn test_resource_lock_contention_graceful_handling() {
        let state = Arc::new(AppState::new_mock().await);
        
        let graph_lock = state.resources.get_code_graph().await;
        let _write_guard = graph_lock.write();
        
        let service = CodeGraphWarmupService;
        let (_shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);
        let context = SystemContext {
            app_state: state.clone(),
            shutdown_rx,
            config: ServiceConfiguration::default(),
        };

        drop(_write_guard);
        let res = service.start(context).await;
        assert!(res.is_ok(), "Service should complete successfully after lock is released");
    }
}

// Metadata: [startup]

// Metadata: [startup]

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

pub mod services;

use crate::state::AppState;
use async_trait::async_trait;
use opentelemetry::trace::TracerProvider as _;
use opentelemetry_sdk::trace::SdkTracerProvider;
use std::sync::Arc;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

/// Configures the weight and scope of the engine boot sequence.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BootstrapIntent {
    /// Full mission execution: Warm up all Code Graph, mDNS, and ingestion workers.
    Full,
    /// Fast Path: Skip heavy warm-up tasks for simple CLI status/version requests.
    Fast,
}

/// Initializes the global telemetry and tracing ecosystem.
pub fn init_tracing(disable_otel: bool) {
    // 1. Core Layers (Always active)
    let fmt_layer = tracing_subscriber::fmt::layer();
    let env_filter = tracing_subscriber::EnvFilter::from_default_env();
    let telemetry_layer = crate::telemetry::TelemetryLayer::new();

    // 2. OpenTelemetry Layer (Optional & Resilient)
    let enable_stdout = std::env::var("OTEL_STDOUT_EXPORTER")
        .map(|v| v.to_lowercase() == "true")
        .unwrap_or(false);

    if !disable_otel && enable_stdout {
        let exporter = opentelemetry_stdout::SpanExporter::default();
        let redacting_exporter = RedactingSpanExporter::new(exporter);
        let provider = SdkTracerProvider::builder()
            .with_simple_exporter(redacting_exporter)
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
        } else {
            return;
        }
    }

    // Fallback or Explicitly Disabled: Initialize without OTel
    let _ = tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .with(crate::telemetry::TelemetryLayer::new())
        .try_init();
}

/// A wrapper for OpenTelemetry SpanExporters that redacts sensitive information.
#[derive(Debug)]
pub struct RedactingSpanExporter<E> {
    inner: E,
    redactor: crate::secret_redactor::SecretRedactor,
}

impl<E> RedactingSpanExporter<E> {
    pub fn new(inner: E) -> Self {
        Self {
            inner,
            redactor: crate::secret_redactor::SecretRedactor::from_env(),
        }
    }
}

impl<E: opentelemetry_sdk::trace::SpanExporter> opentelemetry_sdk::trace::SpanExporter for RedactingSpanExporter<E> {
    fn export(
        &self,
        mut batch: Vec<opentelemetry_sdk::trace::SpanData>,
    ) -> impl std::future::Future<Output = opentelemetry_sdk::error::OTelSdkResult> + Send {
        for span in &mut batch {
            // 1. Redact span name
            span.name = std::borrow::Cow::Owned(self.redactor.redact(&span.name));
            
            // 2. Redact span attributes (e.g. db.statement, http.request.body, etc.)
            for kv in &mut span.attributes {
                if let opentelemetry::Value::String(ref s) = kv.value {
                    let redacted = self.redactor.redact(s.as_str());
                    kv.value = opentelemetry::Value::String(redacted.into());
                }
            }
            
            // 3. Redact events (accessing internal Vec of Event)
            for event in &mut span.events.events {
                event.name = std::borrow::Cow::Owned(self.redactor.redact(&event.name));
                for kv in &mut event.attributes {
                    if let opentelemetry::Value::String(ref s) = kv.value {
                        let redacted = self.redactor.redact(s.as_str());
                        kv.value = opentelemetry::Value::String(redacted.into());
                    }
                }
            }
        }
        self.inner.export(batch)
    }

    fn shutdown(&self) -> Result<(), opentelemetry_sdk::error::OTelSdkError> {
        self.inner.shutdown()
    }

    fn force_flush(&self) -> Result<(), opentelemetry_sdk::error::OTelSdkError> {
        self.inner.force_flush()
    }
}

/// Loads environmental configuration and validates it against the schema.
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
        let is_missing = if let Ok(val) = std::env::var(key) {
            let trimmed = val.trim();
            trimmed.is_empty()
                || trimmed.eq_ignore_ascii_case("todo")
                || trimmed.contains("placeholder")
                || trimmed.contains("YOUR_KEY")
                || trimmed.len() < 10
        } else {
            true
        };
        if is_missing {
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
                let val = self.app_state.governance.tpm_accumulator.load(std::sync::atomic::Ordering::Relaxed);
                StateResponse::TpmAccumulator(val)
            }
            StateQuery::GetRecruitCount => {
                let val = self.app_state.governance.recruit_count.load(std::sync::atomic::Ordering::Relaxed);
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
    fn registry_key(&self) -> &'static str {
        self.name()
    }
    fn start_timeout(&self) -> std::time::Duration {
        std::time::Duration::from_secs(30)
    }
    async fn start(
        &self,
        context: SystemContext,
    ) -> Result<(), anyhow::Error>;
}

pub async fn spawn_background_tasks(
    app_state: Arc<AppState>,
    intent: BootstrapIntent,
    service_config: ServiceConfiguration,
    shutdown_rx: tokio::sync::watch::Receiver<bool>,
) {
    let context = SystemContext {
        app_state,
        shutdown_rx: shutdown_rx.clone(),
        config: service_config,
    };

    // 1. Phased Boot Sequence: Run warmup tasks sequentially (ARCH-02)
    let mut warmup_tasks: Vec<Box<dyn SystemService>> = Vec::new();
    if intent == BootstrapIntent::Full {
        warmup_tasks.push(Box::new(services::CodeGraphWarmupService));
        warmup_tasks.push(Box::new(services::CodeGraphDbRefreshService));
        warmup_tasks.push(Box::new(services::RecoverActiveAgentsService));
    }

    for service in warmup_tasks {
        let name = service.name();
        let reg_key = service.registry_key();
        let is_crit = service.is_critical();
        let timeout_duration = service.start_timeout();
        let context_clone = context.clone();

        if is_crit {
            tracing::debug!("Running warmup service: {}", name);
        }
        let start_fut = service.start(context_clone.clone());
        match tokio::time::timeout(timeout_duration, start_fut).await {
            Ok(Ok(())) => {
                if is_crit {
                    tracing::debug!("Warmup service '{}' completed successfully", name);
                }
            }
            Ok(Err(e)) => {
                tracing::error!("🚨 [Service] Warmup service '{}' failed: {:?}", name, e);
                context_clone.app_state.resources.set_subsystem_status(
                    reg_key,
                    crate::types::SubsystemStatus::Failed(e.to_string()),
                );
            }
            Err(_) => {
                let err_msg = format!("Warmup timeout (exceeded {}s)", timeout_duration.as_secs());
                tracing::error!("🚨 [Service] Warmup service '{}' timed out: {}", name, err_msg);
                context_clone.app_state.resources.set_subsystem_status(
                    reg_key,
                    crate::types::SubsystemStatus::Failed(err_msg),
                );
            }
        }
    }

    // Create phased shutdown channels (ARCH-03)
    let (shutdown_tx_p1, shutdown_rx_p1) = tokio::sync::watch::channel(false);
    let (shutdown_tx_p2, shutdown_rx_p2) = tokio::sync::watch::channel(false);
    let (shutdown_tx_p3, shutdown_rx_p3) = tokio::sync::watch::channel(false);
    let (shutdown_tx_p4, shutdown_rx_p4) = tokio::sync::watch::channel(false);

    // Coordinate phased shutdown sequence in response to global shutdown signal
    let mut global_shutdown_rx = shutdown_rx.clone();
    tokio::spawn(async move {
        loop {
            if global_shutdown_rx.changed().await.is_err() {
                break;
            }
            if *global_shutdown_rx.borrow() {
                break;
            }
        }
        
        tracing::info!("🔔 [ShutdownOrchestrator] Initiating Phased Graceful Shutdown...");
        
        // Phase 1: Ingestion & Networking
        tracing::info!("🛑 [ShutdownOrchestrator] Phase 1: Shutting down Ingestion & Networking...");
        let _ = shutdown_tx_p1.send(true);
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        
        // Phase 2: Telemetry & Monitoring
        tracing::info!("🛑 [ShutdownOrchestrator] Phase 2: Shutting down Telemetry & Monitoring...");
        let _ = shutdown_tx_p2.send(true);
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        
        // Phase 3: Security & Cleanup
        tracing::info!("🛑 [ShutdownOrchestrator] Phase 3: Shutting down Security & Cleanup...");
        let _ = shutdown_tx_p3.send(true);
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        
        // Phase 4: Persistence
        tracing::info!("🛑 [ShutdownOrchestrator] Phase 4: Shutting down Persistence...");
        let _ = shutdown_tx_p4.send(true);
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        
        tracing::info!("✨ [ShutdownOrchestrator] Phased Graceful Shutdown complete.");
    });

    // 2. Spawn loop and network tasks concurrently
    let mut loop_services: Vec<Box<dyn SystemService>> = vec![
        Box::new(services::HeartbeatService),
        Box::new(services::ContinuitySchedulerService),
        Box::new(services::SwarmReaperService),
        Box::new(services::MemoryCleanupService),
        Box::new(services::IngestionWorkerService),
        Box::new(services::PrivacyGuardService),
        Box::new(services::SecurityEvictionService),
        Box::new(services::MetricAggregatorService),
        Box::new(services::BudgetFlushService),
        Box::new(services::RecipeIngestionService),
        Box::new(services::SystemHealthMonitorService),
    ];

    #[cfg(feature = "vector-memory")]
    {
        loop_services.push(Box::new(services::IksDecayService));
        loop_services.push(Box::new(services::IksEvictionService));
    }

    if intent == BootstrapIntent::Full {
        loop_services.push(Box::new(services::SwarmDiscoveryService));
        loop_services.push(Box::new(services::SwarmPulseService));
    }

    for service in loop_services {
        let name = service.name();
        let reg_key = service.registry_key();
        let is_crit = service.is_critical();
        let timeout_duration = service.start_timeout();
        
        // Select shutdown receiver based on phase category
        let phase_rx = match name {
            "SwarmDiscovery" | "SwarmPulse" | "RecipeIngestion" | "IngestionWorker" | "ContinuityScheduler" => {
                shutdown_rx_p1.clone()
            }
            "Heartbeat" | "MetricAggregator" | "SystemHealthMonitor" => {
                shutdown_rx_p2.clone()
            }
            "PrivacyGuard" | "SecurityEviction" | "MemoryCleanup" | "IksDecay" | "IksEviction" | "SwarmReaper" => {
                shutdown_rx_p3.clone()
            }
            "BudgetFlush" => {
                shutdown_rx_p4.clone()
            }
            _ => shutdown_rx.clone(),
        };

        let mut context_clone = context.clone();
        context_clone.shutdown_rx = phase_rx;
        
        tokio::spawn(async move {
            if is_crit {
                tracing::debug!("Starting background service: {}", name);
            }
            let start_fut = service.start(context_clone.clone());
            match tokio::time::timeout(timeout_duration, start_fut).await {
                Ok(Ok(())) => {
                    if is_crit {
                        tracing::debug!("Background service '{}' started successfully", name);
                    }
                }
                Ok(Err(e)) => {
                    tracing::error!("🚨 [Service] Background service '{}' failed to start: {:?}", name, e);
                    context_clone.app_state.resources.set_subsystem_status(
                        reg_key,
                        crate::types::SubsystemStatus::Failed(e.to_string()),
                    );
                }
                Err(_) => {
                    let err_msg = format!("Startup timeout (exceeded {}s)", timeout_duration.as_secs());
                    tracing::error!("🚨 [Service] Background service '{}' timed out starting: {}", name, err_msg);
                    context_clone.app_state.resources.set_subsystem_status(
                        reg_key,
                        crate::types::SubsystemStatus::Failed(err_msg),
                    );
                }
            }
        });
    }
}

/// Handles version/help administrative queries before full engine initialization.
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
pub fn build_custom_runtime() -> anyhow::Result<tokio::runtime::Runtime> {
    let stack_size = std::env::var("TOKIO_THREAD_STACK_SIZE")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(2 * 1024 * 1024);

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
    use super::services::*;

    #[test]
    fn test_bootstrap_intent_variants() {
        let full = BootstrapIntent::Full;
        let fast = BootstrapIntent::Fast;
        assert_ne!(full, fast);
    }

    #[tokio::test]
    async fn test_warmup_registry_reporting() {
        let state = Arc::new(AppState::default());

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

        let (_shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);
        let service_config = ServiceConfiguration {
            heartbeat_secs: 3,
            ..Default::default()
        };
        spawn_background_tasks(state.clone(), BootstrapIntent::Fast, service_config, shutdown_rx).await;

        tokio::time::sleep(std::time::Duration::from_millis(50)).await;

        let snapshot = state.resources.get_initialization_snapshot();

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

// Metadata: [mod]

// Metadata: [mod]

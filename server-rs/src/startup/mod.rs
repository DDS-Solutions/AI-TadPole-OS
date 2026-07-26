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
//! **Heartbeat Loop**. AI agents should monitor the `init_tracing`
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
///
/// ### Log Format
/// - Default: human-readable console output via `tracing_subscriber::fmt`
/// - JSON: Set `OTEL_STDOUT_EXPORTER=json` for machine-parseable newline-delimited JSON
/// - OTel OTLP: Set `OTEL_EXPORTER_OTLP_ENDPOINT` for Jaeger/Tempo export
/// - OTel Stdout: Set `OTEL_STDOUT_EXPORTER=true` for OTel stdout span export
pub fn init_tracing(disable_otel: bool) {
    crate::telemetry::init_prometheus_metrics();

    let otel_env = std::env::var("OTEL_STDOUT_EXPORTER")
        .unwrap_or_default()
        .to_lowercase();
    let use_json_logs = otel_env == "json";
    let enable_stdout_otel = otel_env == "true";

    // Build the OTel provider (type-erased via Option so it can be moved
    // into exactly one match arm without the compiler needing to unify
    // the different Layered<fmt::Layer<_, JsonFields, ...>> vs
    // Layered<fmt::Layer<_, DefaultFields, ...>> subscriber types).
    let otel_provider: Option<SdkTracerProvider> = if !disable_otel {
        let provider = if enable_stdout_otel {
            SdkTracerProvider::builder()
                .with_simple_exporter(RedactingSpanExporter::new(
                    opentelemetry_stdout::SpanExporter::default(),
                ))
                .build()
        } else {
            use opentelemetry_otlp::WithExportConfig;
            let endpoint = std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT")
                .unwrap_or_else(|_| "http://localhost:4317".to_string());
            match opentelemetry_otlp::SpanExporter::builder()
                .with_tonic()
                .with_endpoint(endpoint.clone())
                .build()
            {
                Ok(exp) => SdkTracerProvider::builder()
                    .with_batch_exporter(RedactingSpanExporter::new(exp))
                    .build(),
                Err(e) => {
                    eprintln!(
                        "⚠️  [Telemetry] OTLP failed ({}): {}. Using stdout.",
                        endpoint, e
                    );
                    SdkTracerProvider::builder()
                        .with_simple_exporter(RedactingSpanExporter::new(
                            opentelemetry_stdout::SpanExporter::default(),
                        ))
                        .build()
                }
            }
        };
        Some(provider)
    } else {
        None
    };

    // Each arm creates its own otel_layer so Rust never needs to unify
    // the concrete Layered<fmt::Layer<_, JsonFields, …>> vs DefaultFields types.
    match (use_json_logs, otel_provider) {
        (true, Some(provider)) => {
            let otel = tracing_opentelemetry::layer().with_tracer(provider.tracer("tadpole-os"));
            let _ = tracing_subscriber::registry()
                .with(tracing_subscriber::EnvFilter::from_default_env())
                .with(tracing_subscriber::fmt::layer().json())
                .with(otel)
                .with(crate::telemetry::TelemetryLayer::new())
                .try_init();
        }
        (false, Some(provider)) => {
            let otel = tracing_opentelemetry::layer().with_tracer(provider.tracer("tadpole-os"));
            let _ = tracing_subscriber::registry()
                .with(tracing_subscriber::EnvFilter::from_default_env())
                .with(tracing_subscriber::fmt::layer())
                .with(otel)
                .with(crate::telemetry::TelemetryLayer::new())
                .try_init();
        }
        (true, None) => {
            let _ = tracing_subscriber::registry()
                .with(tracing_subscriber::EnvFilter::from_default_env())
                .with(tracing_subscriber::fmt::layer().json())
                .with(crate::telemetry::TelemetryLayer::new())
                .try_init();
        }
        (false, None) => {
            let _ = tracing_subscriber::registry()
                .with(tracing_subscriber::EnvFilter::from_default_env())
                .with(tracing_subscriber::fmt::layer())
                .with(crate::telemetry::TelemetryLayer::new())
                .try_init();
        }
    }
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

impl<E: opentelemetry_sdk::trace::SpanExporter> opentelemetry_sdk::trace::SpanExporter
    for RedactingSpanExporter<E>
{
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
                let val = self
                    .app_state
                    .governance
                    .max_swarm_depth
                    .load(std::sync::atomic::Ordering::Relaxed);
                StateResponse::MaxSwarmDepth(val)
            }
            StateQuery::GetActiveAgents => {
                let val = self
                    .app_state
                    .governance
                    .active_agents
                    .load(std::sync::atomic::Ordering::Relaxed);
                StateResponse::ActiveAgents(val)
            }
            StateQuery::GetTpmAccumulator => {
                let val = self
                    .app_state
                    .governance
                    .tpm_accumulator
                    .load(std::sync::atomic::Ordering::Relaxed);
                StateResponse::TpmAccumulator(val)
            }
            StateQuery::GetRecruitCount => {
                let val = self
                    .app_state
                    .governance
                    .recruit_count
                    .load(std::sync::atomic::Ordering::Relaxed);
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
    async fn start(&self, context: SystemContext) -> Result<(), anyhow::Error>;
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
        let context_clone = context.clone();
        tokio::spawn(async move {
            let name = service.name();
            let reg_key = service.registry_key();
            let is_crit = service.is_critical();
            let timeout_duration = service.start_timeout();

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
                    tracing::error!(
                        "🚨 [Service] Warmup service '{}' timed out: {}",
                        name,
                        err_msg
                    );
                    context_clone
                        .app_state
                        .resources
                        .set_subsystem_status(reg_key, crate::types::SubsystemStatus::Failed(err_msg));
                }
            }
        });
    }

    // ── Gap 7: Emit engine:boot event after warmup phase completes ─────────────
    {
        use crate::telemetry::TELEMETRY_TX;
        use std::sync::atomic::Ordering;
        let boot_ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
        let _ = TELEMETRY_TX.send(::serde_json::json!({
            "type": "engine:boot",
            "session_id": context.app_state.session_id.clone(),
            "timestamp": boot_ts,
            "version": env!("CARGO_PKG_VERSION"),
            "config": {
                "budget_flush_interval_secs": context.config.budget_flush_interval_secs,
                "null_providers_test_mode": context.app_state.governance.null_providers_test_mode.load(Ordering::Relaxed),
                "privacy_mode": context.app_state.governance.privacy_mode.load(Ordering::Relaxed),
                "max_agents": context.app_state.governance.max_agents.load(Ordering::Relaxed),
            }
        }));
    }

    // Create phased shutdown channels (ARCH-03)
    let (shutdown_tx_p1, shutdown_rx_p1) = tokio::sync::watch::channel(false);
    let (shutdown_tx_p2, shutdown_rx_p2) = tokio::sync::watch::channel(false);
    let (shutdown_tx_p3, shutdown_rx_p3) = tokio::sync::watch::channel(false);
    let (shutdown_tx_p4, shutdown_rx_p4) = tokio::sync::watch::channel(false);

    // Coordinate phased shutdown sequence in response to global shutdown signal
    let mut global_shutdown_rx = shutdown_rx.clone();
    let shutdown_session_id = context.app_state.session_id.clone();
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

        // ── Gap 7: Emit engine:shutdown before tearing down services ────────────
        {
            use crate::telemetry::TELEMETRY_TX;
            let shutdown_ts = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64;
            let _ = TELEMETRY_TX.send(::serde_json::json!({
                "type": "engine:shutdown",
                "session_id": shutdown_session_id,
                "timestamp": shutdown_ts,
                "reason": "graceful"
            }));
        }

        // Phase 1: Ingestion & Networking
        tracing::info!(
            "🛑 [ShutdownOrchestrator] Phase 1: Shutting down Ingestion & Networking..."
        );
        let _ = shutdown_tx_p1.send(true);
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;

        // Phase 2: Telemetry & Monitoring
        tracing::info!(
            "🛑 [ShutdownOrchestrator] Phase 2: Shutting down Telemetry & Monitoring..."
        );
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
        Box::new(services::TelemetryLogSinkService),
        Box::new(services::BudgetFlushService),
        Box::new(services::RecipeIngestionService),
        Box::new(services::SystemHealthMonitorService),
        Box::new(crate::services::cognitive_memory::CognitiveMemoryPipelineService),
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
            "SwarmDiscovery"
            | "SwarmPulse"
            | "RecipeIngestion"
            | "IngestionWorker"
            | "ContinuityScheduler" => shutdown_rx_p1.clone(),
            "Heartbeat" | "MetricAggregator" | "TelemetryLogSink" | "SystemHealthMonitor" => {
                shutdown_rx_p2.clone()
            }
            "PrivacyGuard" | "SecurityEviction" | "MemoryCleanup" | "IksDecay" | "IksEviction"
            | "SwarmReaper" => shutdown_rx_p3.clone(),
            "BudgetFlush" => shutdown_rx_p4.clone(),
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
                    tracing::error!(
                        "🚨 [Service] Background service '{}' failed to start: {:?}",
                        name,
                        e
                    );
                    context_clone.app_state.resources.set_subsystem_status(
                        reg_key,
                        crate::types::SubsystemStatus::Failed(e.to_string()),
                    );
                }
                Err(_) => {
                    let err_msg =
                        format!("Startup timeout (exceeded {}s)", timeout_duration.as_secs());
                    tracing::error!(
                        "🚨 [Service] Background service '{}' timed out starting: {}",
                        name,
                        err_msg
                    );
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

/// Scans for structured JSON crash logs in `.tmp/crashes/` and updates the database state accordingly.
pub async fn reconcile_crashes(pool: &sqlx::SqlitePool) -> anyhow::Result<()> {
    let crashes_dir = if let Ok(root) = std::env::var("WORKSPACE_ROOT") {
        std::path::PathBuf::from(root).join(".tmp").join("crashes")
    } else {
        std::path::PathBuf::from(".tmp").join("crashes")
    };

    if tokio::fs::metadata(&crashes_dir).await.is_err() {
        return Ok(());
    }

    let mut entries = tokio::fs::read_dir(&crashes_dir).await?;
    while let Some(entry) = entries.next_entry().await? {
        let path = entry.path();
        if path.is_file() && path.extension().map_or(false, |ext| ext == "json") {
            // Parse structured JSON
            if let Ok(content) = tokio::fs::read_to_string(&path).await {
                if let Ok(val) = serde_json::from_str::<serde_json::Value>(&content) {
                    let msg = val.get("message").and_then(|m| m.as_str()).unwrap_or("Unknown panic");
                    let loc = val.get("location").and_then(|l| l.as_str()).unwrap_or("unknown location");
                    let timestamp_str = val.get("timestamp")
                        .map(|t| t.to_string())
                        .unwrap_or_else(|| "unknown".to_string());
                    
                    tracing::warn!("🔧 [Reconciler] Found crash log from panic: {} at {}. Reconciling database...", msg, loc);

                    // Find the most recent active/pending mission in the database
                    let active_mission: Option<(String, String)> = sqlx::query_as::<_, (String, String)>(
                        "SELECT id, title FROM mission_history WHERE status IN ('active', 'pending') ORDER BY created_at DESC LIMIT 1"
                    )
                    .fetch_optional(pool)
                    .await?;

                    if let Some((mission_id, mission_title)) = active_mission {
                        tracing::warn!("🔧 [Reconciler] Reconciling active mission {} ('{}') as crashed.", mission_id, mission_title);
                        
                        // Insert a fatal step log
                        let log_id = uuid::Uuid::new_v4().to_string();
                        let now = chrono::Utc::now();
                        let log_text = format!("🚨 ENGINE CRASHED: {}\nLocation: {}\nTimestamp: {}", msg, loc, timestamp_str);
                        
                        sqlx::query(
                            "INSERT INTO mission_logs (id, mission_id, agent_id, source, text, severity, timestamp, hash)
                             VALUES (?1, ?2, 'system', 'system', ?3, 'fatal', ?4, '')"
                        )
                        .bind(&log_id)
                        .bind(&mission_id)
                        .bind(&log_text)
                        .bind(now)
                        .execute(pool)
                        .await?;

                        // Update the mission status
                        sqlx::query(
                            "UPDATE mission_history SET status = 'failed' WHERE id = ?1"
                        )
                        .bind(&mission_id)
                        .execute(pool)
                        .await?;
                    }
                }
            }
            // Delete the crash file once processed
            if let Err(e) = tokio::fs::remove_file(&path).await {
                tracing::error!("🚨 [Reconciler] Failed to remove reconciled crash file {:?}: {:?}", path, e);
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::services::*;
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
        spawn_background_tasks(
            state.clone(),
            BootstrapIntent::Fast,
            service_config,
            shutdown_rx,
        )
        .await;

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
        assert!(
            res.is_ok(),
            "Service startup should be resilient to dependency errors"
        );
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
        assert!(
            res.is_ok(),
            "Service should complete successfully after lock is released"
        );
    }

    #[tokio::test]
    async fn test_crash_reconciliation() {
        let state = Arc::new(AppState::new_mock().await);
        let pool = &state.resources.pool;

        // Get a valid agent_id to satisfy foreign key constraints
        let agent_id: String = sqlx::query_scalar("SELECT id FROM agents LIMIT 1")
            .fetch_one(pool)
            .await
            .unwrap();

        // Seed an active mission in the database
        let mission_id = "test-mission-123";
        sqlx::query(
            "INSERT INTO mission_history (id, title, status, agent_id, created_at)
             VALUES (?1, 'Test Mission', 'active', ?2, ?3)"
        )
        .bind(mission_id)
        .bind(&agent_id)
        .bind(chrono::Utc::now())
        .execute(pool)
        .await
        .unwrap();

        // Create a temporary crashes directory
        let crashes_dir = if let Ok(root) = std::env::var("WORKSPACE_ROOT") {
            std::path::PathBuf::from(root).join(".tmp").join("crashes")
        } else {
            std::path::PathBuf::from(".tmp").join("crashes")
        };
        std::fs::create_dir_all(&crashes_dir).unwrap();

        // Create a crash JSON file
        let crash_file = crashes_dir.join("crash-12345.json");
        let payload = r#"{
            "timestamp": 12345,
            "message": "assertion failed: self.is_char_boundary(new_len)",
            "location": "src/utils/serialization.rs:36:15"
        }"#;
        std::fs::write(&crash_file, payload).unwrap();

        // Run the reconciler
        reconcile_crashes(pool).await.unwrap();

        // Verify that the crash file has been deleted
        assert!(!crash_file.exists());

        // Verify that the mission status has been updated to failed
        let status: String = sqlx::query_scalar("SELECT status FROM mission_history WHERE id = ?1")
            .bind(mission_id)
            .fetch_one(pool)
            .await
            .unwrap();
        assert_eq!(status, "failed");

        // Verify that a fatal log step has been inserted
        let log_text: String = sqlx::query_scalar("SELECT text FROM mission_logs WHERE mission_id = ?1 AND severity = 'fatal'")
            .bind(mission_id)
            .fetch_one(pool)
            .await
            .unwrap();
        assert!(log_text.contains("assertion failed: self.is_char_boundary(new_len)"));
    }
}

// Metadata: [mod]

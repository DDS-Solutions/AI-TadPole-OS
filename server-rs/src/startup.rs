use crate::state::AppState;
use opentelemetry::trace::TracerProvider as _;
use opentelemetry_sdk::trace::TracerProvider;
use std::sync::Arc;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

/// Initializes the global telemetry and tracing ecosystem.
/// 
/// Bridges `tracing` spans with OpenTelemetry and a custom real-time broadcast
/// layer for the frontend UI.
pub fn init_tracing() {
    // 1. OTel stdout exporter (Local debugging of Spans)
    let exporter = opentelemetry_stdout::SpanExporter::default();

    let provider = TracerProvider::builder()
        .with_simple_exporter(exporter)
        .build();
    let tracer = provider.tracer("tadpole-os");

    // 2. Composable layers
    let telemetry_layer = crate::telemetry::TelemetryLayer::new();
    let otel_layer = tracing_opentelemetry::layer().with_tracer(tracer);

    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .with(tracing_subscriber::fmt::layer())
        .with(otel_layer)
        .with(telemetry_layer)
        .init();
}

/// Loads environmental configuration and validates it against the schema.
/// 
/// Ensures that all required keys are present and types are valid before 
/// the engine begins mission execution.
pub fn load_environment() {
    if dotenvy::dotenv().is_err() {
        tracing::warn!("No .env file found. Relying on system environment variables.");
    }

    // SEC-02: Validate environment against schema
    if let Err(e) = crate::env_schema::validate_and_report(std::path::Path::new(".env.schema")) {
        tracing::error!("🚨 [EnvSchema] Validation failed: {}", e);
        if !cfg!(debug_assertions) {
            panic!("{}", e);
        }
    }
}

/// Spawns the engine's long-running background tasks.
/// 
/// Includes:
/// 1. Warm-up of the in-memory code graph.
/// 2. Real-time system heartbeat (Internal diagnostics).
/// 3. Autonomous mission continuity scheduler.
/// 4. Orphaned memory scope hygiene (GC).
pub async fn spawn_background_tasks(app_state: Arc<AppState>) {
    // 1. Hydra-RS: Initial Code Scan
    {
        let graph_lock = app_state.resources.get_code_graph().await;
        let mut graph = graph_lock.write();
        graph.scan();
        tracing::info!(
            "[Hydra-RS] In-memory code graph warmed up ({} modules indexed)",
            graph.modules.len()
        );
    }

    // 2. Launch Heartbeat Loop to drive UI presence
    let heartbeat_state = app_state.clone();
    let heartbeat_secs: u64 = std::env::var("HEARTBEAT_INTERVAL_SECS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(3);

    tokio::spawn(async move {
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(heartbeat_secs)).await;

            let active_agents = heartbeat_state
                .governance
                .active_agents
                .load(std::sync::atomic::Ordering::Relaxed);
            let swarm_depth = heartbeat_state
                .governance
                .max_swarm_depth
                .load(std::sync::atomic::Ordering::Relaxed);
            let tpm = heartbeat_state
                .governance
                .tpm_accumulator
                .swap(0, std::sync::atomic::Ordering::Relaxed);
            let recruits = heartbeat_state
                .governance
                .recruit_count
                .swap(0, std::sync::atomic::Ordering::Relaxed);

            heartbeat_state.emit_event(serde_json::json!({
                "type": "engine:health",
                "uptime": 0,
                "agentCount": active_agents,
                "activeAgents": active_agents,
                "maxDepth": swarm_depth,
                "tpm": tpm,
                "recruitCount": recruits,
                "timestamp": chrono::Utc::now().to_rfc3339()
            }));
        }
    });

    // 3. Launch Continuity Scheduler (autonomous scheduled missions)
    let continuity_state = app_state.clone();
    tokio::spawn(crate::agent::continuity::executor::start_scheduler(
        continuity_state,
    ));
    tracing::info!("🕐 [Continuity] Scheduled job executor launched.");

    // 4. Launch Orphaned Scope Cleanup
    let memory_cleanup_pool = app_state.resources.pool.clone();
    tokio::spawn(async move {
        // Run cleanup immediately on startup, then every 6 hours
        crate::agent::memory::cleanup_orphaned_scopes(&memory_cleanup_pool).await;
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(6 * 3600)).await;
            crate::agent::memory::cleanup_orphaned_scopes(&memory_cleanup_pool).await;
        }
    });
}

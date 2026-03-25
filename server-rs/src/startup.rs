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

    // 5. Swarm Discovery (mDNS)
    let discovery_state = app_state.clone();
    tokio::spawn(async move {
        match crate::services::discovery::SwarmDiscoveryManager::new(discovery_state) {
            Ok(manager) => {
                if let Err(e) = manager.start() {
                    tracing::error!("📡 [Discovery] Failed to start mDNS manager: {}", e);
                }
            }
            Err(e) => tracing::error!("📡 [Discovery] Failed to initialize mDNS manager: {}", e),
        }
    });

    // 6. Privacy Guard (Air-Gap Monitor)
    let privacy_state = app_state.clone();
    tokio::spawn(crate::services::privacy::start_privacy_guard(privacy_state));

    // 7. Launch Debounced Token Usage Flush (every 10 seconds)
    let flush_state = app_state.clone();
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(10)).await;
            if let Err(e) = flush_state.security.budget_guard.flush_to_db().await {
                tracing::error!("🚨 [BudgetGuard] Failed to flush usage to DB: {}", e);
            }
        }
    });
}

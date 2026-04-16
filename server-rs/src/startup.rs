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
//!   panics during codebase indexing. Check the `bootstrap_intent` if
//!   specific workers are missing.
//! - **Telemetry Link**: Search for `[Bootstrap]` or `[Hydra-RS]` in
//!   `tracing` logs for phase-specific boot milestones.
//! - **Trace Scope**: `server-rs::startup`

use crate::state::AppState;
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
pub fn init_tracing() {
    // 0. Privacy/Performance toggle: Disable telemetry if requested
    let disable_otel = std::env::var("DISABLE_TELEMETRY").as_deref() == Ok("true");

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
pub async fn spawn_background_tasks(app_state: Arc<AppState>, intent: BootstrapIntent) {
    // 1. Hydra-RS: Initial Code Scan (Optional in Fast Path)
    if intent == BootstrapIntent::Full {
        let state_for_graph = app_state.clone();
        tokio::spawn(async move {
            state_for_graph
                .resources
                .set_subsystem_status("CodeGraph", crate::types::SubsystemStatus::Warming(0.1));
            let graph_lock = state_for_graph.resources.get_code_graph().await;
            let mut graph = graph_lock.write();
            graph.scan();
            state_for_graph
                .resources
                .set_subsystem_status("CodeGraph", crate::types::SubsystemStatus::Ready);
            tracing::info!(
                "[Hydra-RS] In-memory code graph warmed up ({} modules indexed)",
                graph.modules.len()
            );
        });
    }

    // 2. Launch Heartbeat Loop to drive UI presence
    let heartbeat_state = app_state.clone();
    let heartbeat_secs: u64 = std::env::var("HEARTBEAT_INTERVAL_SECS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(3);

    tokio::spawn(async move {
        let boot_instant = std::time::Instant::now();
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

            let registry_snapshot: std::collections::HashMap<
                String,
                crate::types::SubsystemStatus,
            > = heartbeat_state
                .resources
                .initialization_registry
                .iter()
                .map(|kv| (kv.key().clone(), kv.value().clone()))
                .collect();

            heartbeat_state.emit_event(serde_json::json!({
                "type": "engine:health",
                "uptime": boot_instant.elapsed().as_secs(),
                "agentCount": active_agents,
                "activeAgents": active_agents,
                "maxDepth": swarm_depth,
                "tpm": tpm,
                "recruitCount": recruits,
                "timestamp": chrono::Utc::now().to_rfc3339(),
                "initialization": registry_snapshot
            }));
        }
    });

    // 3. Launch Continuity Scheduler (autonomous scheduled missions)
    let continuity_state = app_state.clone();
    tokio::spawn(crate::agent::continuity::executor::start_scheduler(
        continuity_state,
    ));
    tracing::info!("🕐 [Continuity] Scheduled job executor launched.");

    // 4. Launch Orphaned Scope Cleanup (Neural Memory Only)
    #[cfg(feature = "vector-memory")]
    {
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

    // 5. Ingestion Worker (Phase 2 SME Enhancement)
    let ingestion_state = app_state.clone();
    tokio::spawn(crate::agent::connectors::start_ingestion_worker(
        ingestion_state,
    ));

    // 6. Swarm Discovery (mDNS) (Skip in Fast Path)
    if intent == BootstrapIntent::Full {
        let discovery_state = app_state.clone();
        tokio::spawn(async move {
            discovery_state
                .resources
                .set_subsystem_status("Network", crate::types::SubsystemStatus::Warming(0.0));
            match crate::services::discovery::SwarmDiscoveryManager::new(discovery_state.clone()) {
                Ok(manager) => {
                    if let Err(e) = manager.start() {
                        tracing::error!("📡 [Discovery] Failed to start mDNS manager: {}", e);
                        discovery_state.resources.set_subsystem_status(
                            "Network",
                            crate::types::SubsystemStatus::Failed(e.to_string()),
                        );
                    } else {
                        discovery_state
                            .resources
                            .set_subsystem_status("Network", crate::types::SubsystemStatus::Ready);
                    }
                }
                Err(e) => {
                    tracing::error!("📡 [Discovery] Failed to initialize mDNS manager: {}", e);
                    discovery_state.resources.set_subsystem_status(
                        "Network",
                        crate::types::SubsystemStatus::Failed(e.to_string()),
                    );
                }
            }
        });
    }

    // 7. Privacy Guard (Air-Gap Monitor)
    let privacy_state = app_state.clone();
    tokio::spawn(crate::services::privacy::start_privacy_guard(privacy_state));

    // 8. Rate Limit Bucket Eviction (prevents unbounded memory growth under bot traffic)
    tokio::spawn(async {
        use std::time::Duration;
        let eviction_interval = Duration::from_secs(300); // Every 5 minutes
        let max_bucket_age = Duration::from_secs(120);    // Evict entries idle for 2+ minutes
        let max_auth_age = Duration::from_secs(600);      // Matches BLOCK_DURATION

        loop {
            tokio::time::sleep(eviction_interval).await;

            // Evict stale rate limit buckets
            crate::middleware::rate_limit::evict_stale_buckets(max_bucket_age);

            // Evict expired auth failure records
            crate::middleware::auth_rate_limit::evict_expired_blocks(max_auth_age);

            tracing::debug!("🧹 [Security] Rate limit bucket eviction completed");
        }
    });

    // 9. Launch Telemetry Metric Aggregator (p95 Metrics)
    let aggregator_rx = crate::telemetry::TELEMETRY_TX.subscribe();
    let aggregator = crate::telemetry::aggregator::MetricAggregator::new(1000); // Window of 1000 spans
    tokio::spawn(aggregator.run(aggregator_rx));

    // 10. Launch Debounced Token Usage Flush (every 10 seconds)
    let flush_state = app_state.clone();
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(10)).await;
            if let Err(e) = flush_state.security.budget_guard.flush_to_db().await {
                tracing::error!("🚨 [BudgetGuard] Failed to flush usage to DB: {}", e);
            }
        }
    });

    // 11. Launch High-Speed Swarm Pulse (100ms) - Only in Full Mode
    if intent == BootstrapIntent::Full {
        let pulse_state = app_state.clone();
        tokio::spawn(crate::telemetry::pulse::spawn_pulse_loop(pulse_state));
    }

    // 12. Declarative Swarm Recipes (Auto-provision standard swarms)
    let recipes_state = app_state.clone();
    tokio::spawn(crate::agent::recipes::auto_ingest_recipes(recipes_state));
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
        spawn_background_tasks(state.clone(), BootstrapIntent::Fast).await;

        // Wait a bit for background tasks (though they shouldn't start heavy ones)
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;

        let snapshot = state.resources.get_initialization_snapshot();

        // CodeGraph and Network should NOT be in the registry or should be NotStarted
        // In current implementation, they only appear when Warming is called.
        assert!(!snapshot.contains_key("CodeGraph"));
        assert!(!snapshot.contains_key("Network"));
    }
}

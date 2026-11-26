//! @docs ARCHITECTURE:State
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / State / Init Governance
//! - **Primary Entrypoints**: `resolve_workspace_root`, `parse_max_concurrent_runners`, `build_comms_hub`, `build_governance_hub`, `hydrate_governance_settings`
//!
//! ### 🔍 Debugging & Observability
//! - **Telemetry Targets**: `[Hubs]`, `[Governance]`

use super::channels::BroadcastChannels;
use crate::routes::oversight::OversightSettingsPayload;
use crate::state::hubs::comm::CommunicationHub;
use crate::state::hubs::gov::GovernanceHub;
use dashmap::DashMap;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;

pub fn resolve_workspace_root() -> PathBuf {
    std::env::var("WORKSPACE_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            let current = std::env::current_dir().unwrap_or_default();
            if current.ends_with("server-rs") {
                current.parent().unwrap_or(&current).to_path_buf()
            } else {
                current
            }
        })
}

pub fn parse_max_concurrent_runners() -> u32 {
    std::env::var("MAX_CONCURRENT_RUNNERS")
        .ok()
        .and_then(|s| s.parse::<u32>().ok())
        .filter(|limit| *limit > 0)
        .unwrap_or(10)
}

pub fn build_comms_hub(
    channels: &BroadcastChannels,
    max_concurrent_runners: u32,
) -> Arc<CommunicationHub> {
    tracing::info!("💠 [Hubs] Assembling Communication Hub...");
    Arc::new(CommunicationHub {
        tx: channels.tx.clone(),
        event_tx: channels.event_tx.clone(),
        telemetry_tx: crate::telemetry::TELEMETRY_TX.clone(),
        audio_stream_tx: channels.audio_stream_tx.clone(),
        pulse_tx: channels.pulse_tx.clone(),
        oversight_queue: DashMap::new(),
        oversight_resolvers: DashMap::new(),
        active_runners: DashMap::new(),
        active_executions: DashMap::new(),
        execution_generations: DashMap::new(),
        stopped_agents: dashmap::DashSet::new(),
        engine_stopped: AtomicBool::new(false),
        execution_gate: parking_lot::Mutex::new(()),
        settings_write_gate: parking_lot::Mutex::new(()),
        execution_control: tokio::sync::Mutex::new(()),
        agent_write_gate: tokio::sync::Mutex::new(()),
        execution_changed: tokio::sync::Notify::new(),
        recent_requests: DashMap::new(),
        runner_semaphore: tokio::sync::Semaphore::new(max_concurrent_runners as usize),
        event_sequence: AtomicU64::new(0),
    })
}

fn env_u32(key: &str, default: u32) -> u32 {
    std::env::var(key)
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(default)
}

fn env_bool(key: &str) -> bool {
    std::env::var(key)
        .map(|s| s.eq_ignore_ascii_case("true"))
        .unwrap_or(false)
}

fn default_deprecated_routes() -> HashMap<String, (String, String)> {
    let mut routes = HashMap::new();
    routes.insert(
        "/infra/providers".to_string(),
        (
            "Fri, 01 Jan 2027 23:59:59 GMT".to_string(),
            "<https://docs.tadpole.so/api/v2/providers>; rel=\"alternate\"".to_string(),
        ),
    );
    routes
}

pub fn build_governance_hub(max_concurrent_runners: u32) -> Arc<GovernanceHub> {
    tracing::info!("💠 [Hubs] Assembling Governance Hub...");
    let max_task_length = std::env::var("MAX_TASK_LENGTH")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(32768);
    let default_budget = std::env::var("DEFAULT_AGENT_BUDGET_USD")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(1.0);
    let default_model = std::env::var("DEFAULT_INTELLIGENCE_MODEL")
        .unwrap_or_else(|_| "gemini-1.5-pro".to_string());
    let default_provider =
        std::env::var("DEFAULT_PROVIDER").unwrap_or_else(|_| "google".to_string());

    Arc::new(GovernanceHub {
        auto_approve_safe_skills: AtomicBool::new(
            std::env::var("AUTO_APPROVE_SAFE_SKILLS")
                .map(|s| s == "true")
                .unwrap_or(false),
        ),
        max_agents: AtomicU32::new(env_u32("MAX_AGENTS", 50)),
        max_clusters: AtomicU32::new(env_u32("MAX_CLUSTERS", 10)),
        max_swarm_depth: AtomicU32::new(env_u32("MAX_SWARM_DEPTH", 5)),
        max_task_length: AtomicUsize::new(max_task_length),
        default_budget_usd: RwLock::new(default_budget),
        default_model: RwLock::new(default_model),
        default_provider: RwLock::new(default_provider),
        active_agents: AtomicU32::new(0),
        max_concurrent_runners: AtomicU32::new(max_concurrent_runners),
        recruit_count: AtomicU32::new(0),
        tpm_accumulator: AtomicUsize::new(0),
        privacy_mode: AtomicBool::new(env_bool("PRIVACY_MODE")),
        failover_amber_threshold: AtomicU32::new(env_u32("FAILOVER_AMBER_THRESHOLD", 3)),
        failover_red_threshold: AtomicU32::new(env_u32("FAILOVER_RED_THRESHOLD", 5)),
        failover_max_attempts: AtomicU32::new(env_u32("FAILOVER_MAX_ATTEMPTS", 3)),
        provider_timeout_secs: AtomicU32::new(env_u32("PROVIDER_TIMEOUT_SECS", 60)),
        null_providers_test_mode: AtomicBool::new(
            std::env::var("TADPOLE_NULL_PROVIDERS")
                .map(|s| s == "true")
                .unwrap_or(false),
        ),
        deprecated_routes: RwLock::new(default_deprecated_routes()),
        cluster_privacy_policies: DashMap::new(),
    })
}

/// Hydrates GovernanceHub and CommunicationHub dispatch state from `data/governance_settings.json`.
pub fn hydrate_governance_settings(
    base_dir: &Path,
    comms: &CommunicationHub,
    governance: &GovernanceHub,
) {
    let settings_path = base_dir.join("data").join("governance_settings.json");
    if !settings_path.exists() {
        return;
    }
    let Ok(data) = std::fs::read_to_string(&settings_path) else {
        return;
    };
    let Ok(persisted) = serde_json::from_str::<OversightSettingsPayload>(&data) else {
        return;
    };

    tracing::info!("🛡️ [Governance] Found persisted settings, applying overrides...");
    apply_persisted_governance(comms, governance, persisted);
}

fn apply_persisted_governance(
    comms: &CommunicationHub,
    governance: &GovernanceHub,
    persisted: OversightSettingsPayload,
) {
    comms.engine_stopped.store(
        persisted.dispatch_paused.unwrap_or(false),
        Ordering::Release,
    );

    if let Some(val) = persisted.auto_approve_safe_skills {
        governance
            .auto_approve_safe_skills
            .store(val, Ordering::Relaxed);
    }

    apply_privacy_overrides(
        governance,
        persisted.privacy_mode,
        persisted.cluster_privacy_policies,
    );

    if let Some(val) = persisted.max_agents {
        governance.max_agents.store(val, Ordering::Relaxed);
    }
    if let Some(val) = persisted.max_clusters {
        governance.max_clusters.store(val, Ordering::Relaxed);
    }
    if let Some(val) = persisted.max_swarm_depth {
        governance.max_swarm_depth.store(val, Ordering::Relaxed);
    }
    if let Some(val) = persisted.max_task_length {
        governance.max_task_length.store(val, Ordering::Relaxed);
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

fn apply_privacy_overrides(
    governance: &GovernanceHub,
    privacy_mode: Option<bool>,
    cluster_policies: Option<HashMap<String, bool>>,
) {
    if std::env::var("PRIVACY_MODE").is_ok() {
        tracing::info!(
            "🛡️ [Governance] Preserving PRIVACY_MODE from environment: {}",
            governance.privacy_mode.load(Ordering::Relaxed)
        );
    } else if let Some(val) = privacy_mode {
        governance.privacy_mode.store(val, Ordering::Relaxed);
    }

    if let Some(policies) = cluster_policies {
        for (cid, mode) in policies {
            governance.cluster_privacy_policies.insert(cid, mode);
        }
    }
}

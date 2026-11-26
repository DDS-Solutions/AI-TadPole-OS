//! @docs ARCHITECTURE:State
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / State / Init Resources
//! - **Primary Entrypoints**: `build_security_hub`, `build_resource_hub`, `set_initial_subsystem_statuses`
//!
//! ### 🔍 Debugging & Observability
//! - **Telemetry Targets**: none declared

use super::security::{load_oversight_public_key, SecurityTokens};
use super::services::create_payment_router;
use crate::error::AppError;
use crate::state::hubs::res::ResourceHub;
use crate::state::hubs::sec::SecurityHub;
use crate::types::SubsystemStatus;
use dashmap::DashMap;
use sqlx::SqlitePool;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::OnceCell;

pub fn build_security_hub(
    pool: &SqlitePool,
    secret_redactor: Arc<crate::secret_redactor::SecretRedactor>,
    permission_policy: Arc<crate::security::permissions::PermissionPolicy>,
    tokens: SecurityTokens,
) -> Result<Arc<SecurityHub>, AppError> {
    let system_monitor = Arc::new(crate::security::monitoring::SecurityMonitor::new());
    let budget_guard = Arc::new(crate::security::metering::BudgetGuard::new(
        pool.clone(),
        system_monitor.clone(),
    ));
    let audit_trail = Arc::new(
        crate::security::audit::MerkleAuditTrail::new(pool.clone()).map_err(|e| {
            AppError::InternalServerError(format!("Failed to initialize audit trail: {}", e))
        })?,
    );
    let shell_scanner = Arc::new(crate::security::scanner::ShellScanner::new(
        secret_redactor.clone(),
    ));

    Ok(Arc::new(SecurityHub {
        conflict_manager: Arc::new(crate::security::conflict::ConflictManager::new()),
        audit_trail,
        budget_guard,
        shell_scanner,
        secret_redactor,
        system_monitor,
        permission_policy,
        deploy_token: tokens.deploy_token,
        admin_token: tokens.admin_token,
        // C-03: Load pinned oversight public key once at startup.
        oversight_public_key: load_oversight_public_key(),
    }))
}

pub fn build_resource_hub(
    base_dir: &Path,
    pool: &SqlitePool,
    http_client: Arc<reqwest::Client>,
    audio_cache: Arc<crate::agent::audio_cache::BunkerCache>,
) -> Arc<ResourceHub> {
    Arc::new(ResourceHub {
        pool: pool.clone(),
        http_client,
        audio_engine: OnceCell::new(),
        audio_cache,
        code_graph: OnceCell::new(),
        symbol_graph: OnceCell::new(),
        obfuscation_salt: crate::intelligence::graph::derive_stable_salt(base_dir),
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
        base_dir: base_dir.to_path_buf(),
        tool_cache: Arc::new(parking_lot::Mutex::new(
            crate::agent::runner::tools::cache::SharedToolCache::new(),
        )),
        conflict_manager: Arc::new(crate::security::conflict::ConflictManager::new()),
        payment_router: create_payment_router(pool.clone()),
        workflow_active_runs: Arc::new(DashMap::new()),
        workflow_concurrency_semaphore: Arc::new(tokio::sync::Semaphore::new(10)),
    })
}

pub fn set_initial_subsystem_statuses(resources: &ResourceHub) {
    resources.set_subsystem_status("Database", SubsystemStatus::Ready);
    resources.set_subsystem_status("Agents", SubsystemStatus::Ready);
    resources.set_subsystem_status("MCP", SubsystemStatus::Ready);
    resources.set_subsystem_status("Network", SubsystemStatus::NotStarted);
    resources.set_subsystem_status("CodeGraph", SubsystemStatus::NotStarted);
    resources.set_subsystem_status("Audio", SubsystemStatus::NotStarted);
}

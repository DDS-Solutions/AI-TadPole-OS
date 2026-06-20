//! @docs ARCHITECTURE:State
//!
//! ### AI Assist Note
//! **Infrastructure Engine**: Manages thread-safe access to heavy system
//! resources. Features **Lazy-Loaded Subsystems** (ONNX Audio, Vector
//! Memory, Code Graph) via `OnceCell` to minimize initial startup
//! footprint. Tracks warmup progress in the **Initialization Registry**
//! for real-time telemetry broadcasts.
//!
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: 200MB+ RAM spikes during ONNX model loading,
//!   LanceDB connection timeout, or deadlocks in the `CodeGraph`
//!   (parking_lot::RwLock) during workspace-wide scans.
//! - **Trace Scope**: `server-rs::state::hubs::res`
//! - **Telemetry Link**: Search for `engine:health` or `subsystem:status` logs.

use std::sync::Arc;

use crate::agent::audio::NeuralAudioEngine;
use crate::agent::audio_cache::BunkerCache;
#[cfg(feature = "vector-memory")]
use crate::agent::memory::VectorMemory;
use crate::agent::rate_limiter::RateLimiter;
use crate::intelligence::graph::CodeSymbolGraph;
use crate::types::SubsystemStatus;
use crate::utils::graph::CodeGraph;
use dashmap::DashMap;
use parking_lot::RwLock;
use reqwest::Client;
use sqlx::SqlitePool;
use tokio::sync::OnceCell;

/// Hub for heavy infrastructure resources and shared mission context.
///
/// This hub manages thread-safe access to core engines and persistence layers.
/// To minimize startup footprint, heavy components are loaded lazily via `OnceCell`.
///
/// ### AI Assist Note
/// This is the primary point of failure for dependency-related bugs.
/// Always check the `initialization_registry` or subsystem getters for `Ready` status.
///
/// @docs ARCHITECTURE:ResourceHub
pub struct ResourceHub {
    /// SQLite connection pool for persistent storage.
    pub pool: SqlitePool,
    /// Shared HTTP client with optimized connection pooling.
    pub http_client: Arc<Client>,
    /// Native engine for local audio synthesis (PCM) and transcription.
    /// @state: Deferred (Loaded lazily to save 200MB+ RAM)
    pub audio_engine: OnceCell<Arc<NeuralAudioEngine>>,
    /// Zero-latency semantic audio replicate cache for frequent phrases.
    #[allow(dead_code)]
    pub audio_cache: Arc<BunkerCache>,
    /// Graph of code relationships for RAG-enhanced tool search.
    /// @state: Deferred (Warmed up lazily to prevent CPU/RAM spikes)
    pub code_graph: OnceCell<Arc<RwLock<CodeGraph>>>,
    /// Symbol-level Knowledge Graph.
    /// @state: Deferred
    pub symbol_graph: OnceCell<Arc<RwLock<CodeSymbolGraph>>>,
    /// Dynamic boot-time cryptographically secure salt. Used for path obfuscation.
    pub obfuscation_salt: String,
    /// Global system identity context loaded from `directives/IDENTITY.md`.
    /// @state: Deferred
    pub identity_context: OnceCell<String>,
    /// Global long-term memory context loaded from `directives/LONG_TERM_MEMORY.md`.
    /// @state: Deferred
    pub memory_context: OnceCell<String>,
    /// Global swarm-wide knowledge vault for cross-mission intelligence.
    /// @state: Deferred
    #[cfg(feature = "vector-memory")]
    pub swarm_vault: OnceCell<Arc<VectorMemory>>,
    /// Persistent cross-cluster Institutional Knowledge Store.
    /// Durable, curated facts that survive restarts and cluster migrations.
    /// @state: Deferred
    #[cfg(feature = "vector-memory")]
    pub knowledge_store: OnceCell<Arc<crate::agent::knowledge_store::KnowledgeStore>>,
    /// Cached rate limiters partitioned by model and provider.
    pub rate_limiters: DashMap<String, Arc<RateLimiter>>,
    /// Tracks the initialization status of all subsystems (Phase 3).
    /// @telemetry: engine:health broadcasts this registry.
    pub initialization_registry: DashMap<String, SubsystemStatus>,
    /// System hardware profiler for sovereign compute telemetry.
    pub hardware_profiler: Arc<crate::system::profiler::HardwareProfiler>,
    /// Global Access Control List service for tool governance.
    pub acl: Arc<dyn crate::agent::runner::service_traits::AclServiceTrait>,
    /// System prompt template renderer.
    pub renderer: Arc<dyn crate::agent::runner::service_traits::PromptRendererTrait>,
    /// Base directory for relative path resolution.
    pub base_dir: std::path::PathBuf,
    /// Shared cache for read-only tool results
    pub tool_cache: Arc<parking_lot::Mutex<crate::agent::runner::tools::cache::SharedToolCache>>,
    /// Registry for conflict locks during concurrent file operations
    pub conflict_manager: Arc<crate::security::conflict::ConflictManager>,
    /// Cached codebase blueprint for System 2 indexing.
    pub blueprint_cache: tokio::sync::OnceCell<Arc<crate::services::blueprint_service::Blueprint>>,
}

impl ResourceHub {
    /// Updates the status of a specific subsystem.
    ///
    /// ### Side Effects
    /// 1. Updates the internal `initialization_registry` DashMap.
    /// 2. Subsequent `engine:health` heartbeats will reflect this new state.
    ///
    /// ### AI Assist Note
    /// This is the primary method for reporting warmup progress.
    pub fn set_subsystem_status(&self, name: &str, status: SubsystemStatus) {
        self.initialization_registry
            .insert(name.to_string(), status);
    }

    #[allow(dead_code)]
    pub fn get_initialization_snapshot(
        &self,
    ) -> std::collections::HashMap<String, SubsystemStatus> {
        self.initialization_registry
            .iter()
            .map(|entry| (entry.key().clone(), entry.value().clone()))
            .collect()
    }

    /// Lazily initializes and returns the ONNX audio engine.
    #[allow(dead_code)]
    pub async fn get_audio_engine(&self) -> Result<Arc<NeuralAudioEngine>, crate::error::AppError> {
        if self.audio_engine.get().is_none() {
            self.set_subsystem_status("Audio", SubsystemStatus::Warming(0.5));
        }
        let engine = self
            .audio_engine
            .get_or_try_init(|| async {
                let e = NeuralAudioEngine::new().await?;
                Ok::<_, crate::error::AppError>(Arc::new(e))
            })
            .await?;
        self.set_subsystem_status("Audio", SubsystemStatus::Ready);
        Ok(engine.clone())
    }

    /// Lazily initializes and returns the code graph.
    pub async fn get_code_graph(&self) -> Arc<RwLock<CodeGraph>> {
        if self.code_graph.get().is_none() {
            self.set_subsystem_status("CodeGraph", SubsystemStatus::Warming(0.0));
        }
        let graph = self
            .code_graph
            .get_or_init(|| async {
                Arc::new(RwLock::new(CodeGraph::new(std::path::PathBuf::from("."))))
            })
            .await;
        // Note: graph.scan() still needs to be called to be truly "Ready"
        graph.clone()
    }

    /// Lazily initializes and returns the symbol-level knowledge graph.
    pub async fn get_symbol_graph(&self) -> Arc<RwLock<CodeSymbolGraph>> {
        if self.symbol_graph.get().is_none() {
            self.set_subsystem_status("CodeGraph", SubsystemStatus::Warming(0.0));
        }
        let graph = self
            .symbol_graph
            .get_or_init(|| async {
                Arc::new(RwLock::new(CodeSymbolGraph::new(self.base_dir.clone())))
            })
            .await;
        graph.clone()
    }

    /// Lazily initializes and returns the identity context.
    pub async fn get_identity_context(&self) -> Result<String, crate::error::AppError> {
        if self.identity_context.get().is_none() {
            self.set_subsystem_status("Identity", SubsystemStatus::Warming(0.0));
        }
        let identity = self
            .identity_context
            .get_or_try_init(|| async {
                let path = self.base_dir.join("directives/IDENTITY.md");
                tokio::fs::read_to_string(path)
                    .await
                    .map_err(crate::error::AppError::Io)
            })
            .await?;
        self.set_subsystem_status("Identity", SubsystemStatus::Ready);
        Ok(identity.clone())
    }

    /// Lazily initializes and returns the long-term memory context.
    pub async fn get_memory_context(&self) -> Result<String, crate::error::AppError> {
        if self.memory_context.get().is_none() {
            self.set_subsystem_status("Memory", SubsystemStatus::Warming(0.0));
        }
        let memory = self
            .memory_context
            .get_or_try_init(|| async {
                let path = self.base_dir.join("directives/LONG_TERM_MEMORY.md");
                tokio::fs::read_to_string(path)
                    .await
                    .map_err(crate::error::AppError::Io)
            })
            .await?;
        self.set_subsystem_status("Memory", SubsystemStatus::Ready);
        Ok(memory.clone())
    }

    /// Lazily initializes and returns the swarm knowledge vault (Phase 3).
    #[cfg(feature = "vector-memory")]
    pub async fn get_swarm_vault(&self) -> Result<Arc<VectorMemory>, crate::error::AppError> {
        if self.swarm_vault.get().is_none() {
            self.set_subsystem_status("SwarmVault", SubsystemStatus::Warming(0.0));
        }
        let memory = self
            .swarm_vault
            .get_or_try_init(|| async {
                let v = VectorMemory::connect("data/swarm/global_vault", "vault_entries").await?;
                Ok::<_, crate::error::AppError>(Arc::new(v))
            })
            .await?;
        self.set_subsystem_status("SwarmVault", SubsystemStatus::Ready);
        Ok(memory.clone())
    }

    /// Lazily initializes and returns the Institutional Knowledge Store.
    #[cfg(feature = "vector-memory")]
    pub async fn get_knowledge_store(
        &self,
    ) -> Result<Arc<crate::agent::knowledge_store::KnowledgeStore>, crate::error::AppError> {
        if self.knowledge_store.get().is_none() {
            self.set_subsystem_status("KnowledgeStore", SubsystemStatus::Warming(0.0));
        }
        let ks = self
            .knowledge_store
            .get_or_try_init(|| async {
                let store = crate::agent::knowledge_store::KnowledgeStore::new(self.pool.clone());
                Ok::<_, crate::error::AppError>(Arc::new(store))
            })
            .await?;
        self.set_subsystem_status("KnowledgeStore", SubsystemStatus::Ready);
        Ok(ks.clone())
    }

    /// Lazily initializes and returns the codebase blueprint.
    pub async fn get_blueprint(
        &self,
        workspace_root: std::path::PathBuf,
    ) -> Result<Arc<crate::services::blueprint_service::Blueprint>, crate::error::AppError> {
        if self.blueprint_cache.get().is_none() {
            self.set_subsystem_status("Blueprint", SubsystemStatus::Warming(0.0));
        }
        let blueprint = self
            .blueprint_cache
            .get_or_try_init(|| async {
                let blueprint_svc =
                    crate::services::blueprint_service::BlueprintService::new(workspace_root);
                let blueprint = blueprint_svc.scan_workspace().await.map_err(|e| {
                    crate::error::AppError::InternalServerError(format!(
                        "Workspace blueprint scan failed: {}",
                        e
                    ))
                })?;
                Ok::<_, crate::error::AppError>(Arc::new(blueprint))
            })
            .await?;
        self.set_subsystem_status("Blueprint", SubsystemStatus::Ready);
        Ok(blueprint.clone())
    }
}

// Metadata: [res]

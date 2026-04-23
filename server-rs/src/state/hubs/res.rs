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
    /// Global system identity context loaded from `directives/IDENTITY.md`.
    /// @state: Deferred
    pub identity_context: OnceCell<String>,
    /// Global long-term memory context loaded from `directives/LONG_TERM_MEMORY.md`.
    /// @state: Deferred
    pub memory_context: OnceCell<String>,
    /// High-performance vector storage engine for long-term RAG.
    /// @state: Deferred
    #[cfg(feature = "vector-memory")]
    pub vector_memory: OnceCell<Arc<VectorMemory>>,
    /// Cached rate limiters partitioned by model and provider.
    pub rate_limiters: DashMap<String, Arc<RateLimiter>>,
    /// Tracks the initialization status of all subsystems (Phase 3).
    /// @telemetry: engine:health broadcasts this registry.
    pub initialization_registry: DashMap<String, SubsystemStatus>,
    /// System hardware profiler for sovereign compute telemetry.
    pub hardware_profiler: Arc<crate::system::profiler::HardwareProfiler>,
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
    pub async fn get_audio_engine(&self) -> Arc<NeuralAudioEngine> {
        if self.audio_engine.get().is_none() {
            self.set_subsystem_status("Audio", SubsystemStatus::Warming(0.5));
        }
        let engine = self
            .audio_engine
            .get_or_init(|| async {
                let e = NeuralAudioEngine::new()
                    .await
                    .expect("Failed to init ONNX Audio Engine");
                Arc::new(e)
            })
            .await;
        self.set_subsystem_status("Audio", SubsystemStatus::Ready);
        engine.clone()
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

    /// Lazily initializes and returns the identity context.
    pub async fn get_identity_context(&self) -> String {
        if self.identity_context.get().is_none() {
            self.set_subsystem_status("Identity", SubsystemStatus::Warming(0.0));
        }
        let identity = self
            .identity_context
            .get_or_init(|| async {
                tokio::fs::read_to_string("directives/IDENTITY.md")
                    .await
                    .unwrap_or_default()
            })
            .await;
        self.set_subsystem_status("Identity", SubsystemStatus::Ready);
        identity.clone()
    }

    /// Lazily initializes and returns the long-term memory context.
    pub async fn get_memory_context(&self) -> String {
        if self.memory_context.get().is_none() {
            self.set_subsystem_status("Memory", SubsystemStatus::Warming(0.0));
        }
        let memory = self
            .memory_context
            .get_or_init(|| async {
                tokio::fs::read_to_string("directives/LONG_TERM_MEMORY.md")
                    .await
                    .unwrap_or_default()
            })
            .await;
        self.set_subsystem_status("Memory", SubsystemStatus::Ready);
        memory.clone()
    }

    /// Lazily initializes and returns the vector memory engine (Phase 3).
    #[cfg(feature = "vector-memory")]
    pub async fn get_vector_memory(&self) -> Arc<VectorMemory> {
        if self.vector_memory.get().is_none() {
            self.set_subsystem_status("VectorMemory", SubsystemStatus::Warming(0.0));
        }
        let memory = self
            .vector_memory
            .get_or_init(|| async {
                Arc::new(
                    VectorMemory::connect("data/memory/global", "memories")
                        .await
                        .expect("Failed to init LanceDB"),
                )
            })
            .await;
        self.set_subsystem_status("VectorMemory", SubsystemStatus::Ready);
        memory.clone()
    }
}

// Metadata: [res]

// Metadata: [res]

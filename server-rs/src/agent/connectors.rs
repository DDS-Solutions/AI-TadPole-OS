//! @docs ARCHITECTURE:Networking
//!
//! ### AI Assist Note
//! **SME Connectors**: Orchestrates external data ingestion and
//! synchronization for the swarm. Implements the **FileSystem Connector**
//! with layout-aware parsing and automated chunking. Features an
//! **Asynchronous Ingestion Worker** that polls sync manifests and
//! updates **Vector Memory** (LanceDB) with fresh embeddings.
//! Ensures **Incremental Sync** by tracking `last_sync_at` timestamps.
//!
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: Permission denied on source directories,
//!   embedding provider rate limits (429) during batch ingestion,
//!   or LanceDB table lock contention.
//! - **Trace Scope**: `server-rs::agent::connectors`

use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use walkdir::WalkDir;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IngestionItem {
    pub id: String,
    pub content: String,
    pub metadata: serde_json::Value,
    pub updated_at: DateTime<Utc>,
}

/// ### 🧩 Protocol: ConnectorTrait
/// Defines the behavior for sovereign data sources (FS, Slack, Notion, etc.)
/// that ingest external knowledge into the swarm's memory palace.
#[async_trait]
#[allow(dead_code)]
pub trait ConnectorTrait: Send + Sync {
    /// Unique identifier for this connector instance (e.g. agent_id + source)
    fn id(&self) -> &str;

    /// Type of source (slack, notion, fs)
    fn source_type(&self) -> &str;

    /// ### 📡 Synchronization Cycle: fetch_new_items
    /// Fetches new or updated items since the last sync time.
    /// Implementation must be incremental to prevent redundant embedding 
    /// costs (LMT-04).
    async fn fetch_new_items(&self, last_sync: Option<DateTime<Utc>>)
        -> Result<Vec<IngestionItem>>;
}

pub struct FsConnector {
    #[allow(dead_code)]
    id: String,
    base_path: PathBuf,
}

impl FsConnector {
    pub fn new(id: String, path: &str) -> Self {
        Self {
            id,
            base_path: PathBuf::from(path),
        }
    }
}

#[async_trait]
impl ConnectorTrait for FsConnector {
    fn id(&self) -> &str {
        &self.id
    }
    fn source_type(&self) -> &str {
        "fs"
    }

    /// ### 📂 Ingestion Logic: Recursive File Crawl
    /// Walks the target directory and identifies files changed since 
    /// `last_sync`. 
    /// 
    /// ### 🛡️ Sector Defense: Format Filtering
    /// To prevent noise and binary corruption in the vector memory, we 
    /// strictly filter for plain-text/semantic formats (MD, TXT, PDF, CSV).
    async fn fetch_new_items(
        &self,
        last_sync: Option<DateTime<Utc>>,
    ) -> Result<Vec<IngestionItem>> {
        let mut items = Vec::new();

        if !self.base_path.exists() {
            return Ok(items);
        }

        for entry in WalkDir::new(&self.base_path)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if entry.file_type().is_file() {
                let path = entry.path();
                let extension = path.extension().and_then(|s| s.to_str()).unwrap_or("");

                // Only ingest relevant SME formats
                if !["txt", "md", "pdf", "csv"].contains(&extension) {
                    continue;
                }

                let metadata = entry.metadata()?;
                let modified: DateTime<Utc> = metadata.modified()?.into();

                // ### 🔄 Efficiency: Incremental Filter
                // Compares filesystem `mtime` with the registry's `last_sync_at` 
                // to avoid expensive re-parsing of static assets.
                if let Some(last) = last_sync {
                    if modified <= last {
                        continue;
                    }
                }

                // ### 🧠 Logic: Layout-Aware Parsing (Phase 4)
                // Dispatches the file to a specialized parser (e.g., Markdown 
                // block-aware or PDF structure-aware) to maintain context 
                // boundaries during chunking.
                match crate::agent::parser::parse_file(path) {
                    Ok(doc) => {
                        // Use structured chunks for better embedding quality.
                        // Standard chunk size of 1500 characters optimized for 
                        // modern embedding windows like Gemini text-embedding-004.
                        let chunks = doc.to_chunks(1500);
                        for (ci, chunk) in chunks.into_iter().enumerate() {
                            items.push(IngestionItem {
                                id: format!("{}#chunk-{}", path.to_string_lossy(), ci),
                                content: chunk,
                                metadata: serde_json::json!({
                                    "path": path.to_string_lossy(),
                                    "format": doc.format,
                                    "chunk_index": ci,
                                }),
                                updated_at: modified,
                            });
                        }
                    }
                    Err(e) => {
                        tracing::warn!(
                            "⚠️ [FsConnector] Failed to parse {}: {}",
                            path.display(),
                            e
                        );
                    }
                }
            }
        }

        Ok(items)
    }
}

/// The background worker that periodically polls all sync manifests.
pub async fn start_ingestion_worker(state: std::sync::Arc<crate::state::AppState>) {
    let interval_mins: u64 = std::env::var("SME_SYNC_INTERVAL_MINS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(30);

    tracing::info!(
        "🚀 [IngestionWorker] Started with {}m interval.",
        interval_mins
    );

    loop {
        if let Err(e) = run_ingestion_cycle(&state).await {
            tracing::error!("🚨 [IngestionWorker] Cycle failed: {}", e);
        }
        tokio::time::sleep(std::time::Duration::from_secs(interval_mins * 60)).await;
    }
}

/// ### 🤖 Orchestration: run_ingestion_cycle
/// Single atomic cycle that reconciles all syncing manifests with their 
/// physical sources.
/// 
/// ### 🧬 Workflow: Embedding Pipeline
/// 1. Locates active sync manifests in SQLite.
/// 2. Resolves the appropriate `ConnectorTrait` implementation (e.g., FsConnector).
/// 3. Fetches batch delta items from the source.
/// 4. Resolves an embedding provider (ARC-01) shared with the agent's LLM context.
/// 5. Populates the `VectorMemory` (LanceDB) with chunked knowledge nodes.
/// 6. Updates the physical ledger to mark completion and record the new sync high-water mark.
async fn run_ingestion_cycle(state: &crate::state::AppState) -> Result<()> {
    let pool = &state.resources.pool;
    let manifests = crate::agent::persistence::load_sync_manifests(pool).await?;

    for manifest in manifests {
        if manifest.status == "syncing" {
            continue; // Skip if already in progress (prevents Race Conditions)
        }

        // Lookup agent from DashMap registry
        let agent = match state.registry.agents.get(&manifest.agent_id) {
            Some(entry) => entry.value().clone(),
            None => continue,
        };

        tracing::info!(
            "📂 [IngestionWorker] Syncing {} for agent {}",
            manifest.source_uri,
            agent.identity.id
        );
        crate::agent::persistence::update_sync_status(pool, &manifest.id, "syncing").await?;

        let connector: Box<dyn ConnectorTrait> = match manifest.source_type.as_str() {
            "fs" => Box::new(FsConnector::new(manifest.id.clone(), &manifest.source_uri)),
            _ => {
                tracing::warn!("Unsupported connector type: {}", manifest.source_type);
                continue;
            }
        };

        match connector.fetch_new_items(Some(manifest.last_sync_at)).await {
            Ok(items) => {
                if items.is_empty() {
                    crate::agent::persistence::update_sync_status(pool, &manifest.id, "idle")
                        .await?;
                    continue;
                }

                // Resolve an embedding provider from the agent's model config
                let latest_update = {
                    #[cfg(feature = "vector-memory")]
                    {
                        let client = (*state.resources.http_client).clone();
                        let provider = resolve_embedding_provider(&agent, client);
                        let mut latest_update = manifest.last_sync_at;

                        let memory_path = format!("data/memory/{}/knowledge.lance", agent.identity.id);
                        match crate::agent::memory::VectorMemory::connect(&memory_path, "memories")
                            .await
                        {
                            Ok(mem) => {
                                for item in items {
                                    match provider.embed(&item.content).await {
                                        Ok(vec) => {
                                            // Atomically insert memory into Vector Space
                                            let _ = mem
                                                .add_memory(
                                                    &item.id,
                                                    &item.content,
                                                    "sync-cycle",
                                                    vec,
                                                )
                                                .await;
                                            if item.updated_at > latest_update {
                                                latest_update = item.updated_at;
                                            }
                                        }
                                        Err(e) => {
                                            tracing::warn!(
                                                "⚠️ [IngestionWorker] Embedding failed for {}: {}",
                                                item.id,
                                                e
                                            );
                                        }
                                    }
                                }
                            }
                            Err(e) => {
                                tracing::error!(
                                    "❌ [IngestionWorker] Failed to connect VectorMemory for {}: {}",
                                    agent.identity.id,
                                    e
                                );
                                crate::agent::persistence::update_sync_status(
                                    pool,
                                    &manifest.id,
                                    "error",
                                )
                                .await?;
                                continue;
                            }
                        }

                        latest_update
                    }

                    #[cfg(not(feature = "vector-memory"))]
                    {
                        tracing::warn!("⚠️ [IngestionWorker] Vector Memory feature is disabled. Skipping SME data ingestion for agent {}.", agent.identity.id);
                        // We still update the manifest to 'idle' to avoid infinite 'syncing' state
                        manifest.last_sync_at
                    }
                };

                crate::agent::persistence::complete_sync(pool, &manifest.id, latest_update).await?;
                tracing::info!(
                    "✅ [IngestionWorker] Completed sync for manifest {}",
                    manifest.id
                );
            }
            Err(e) => {
                tracing::error!(
                    "❌ [IngestionWorker] Sync failed for {}: {}",
                    manifest.id,
                    e
                );
                crate::agent::persistence::update_sync_status(pool, &manifest.id, "error").await?;
            }
        }
    }

    Ok(())
}

/// Resolves a lightweight embedding provider from agent config.
/// Uses the public `LlmProvider` trait to avoid depending on runner-private types.
#[cfg(feature = "vector-memory")]
fn resolve_embedding_provider(
    agent: &crate::agent::types::EngineAgent,
    client: reqwest::Client,
) -> Box<dyn crate::agent::provider_trait::LlmProvider> {
    let provider_name = agent.models.model.provider.to_string();
    let model_config = agent.models.model.clone();

    match provider_name.as_str() {
        "google" | "gemini" => {
            let key = model_config
                .api_key
                .clone()
                .or_else(|| std::env::var("GOOGLE_API_KEY").ok());
            match key {
                Some(k) => Box::new(crate::agent::gemini::GeminiProvider::new(
                    client,
                    k,
                    model_config,
                )),
                None => {
                    tracing::warn!(
                        "⚠️ [IngestionWorker] No GOOGLE_API_KEY for embedding — using NullProvider"
                    );
                    Box::new(crate::agent::null_provider::NullProvider::new(
                        &agent.identity.id,
                        crate::agent::null_provider::NullReason::MissingApiKey {
                            env_var: "GOOGLE_API_KEY",
                        },
                    ))
                }
            }
        }
        "openai" => {
            let key = model_config
                .api_key
                .clone()
                .or_else(|| std::env::var("OPENAI_API_KEY").ok());
            match key {
                Some(k) => Box::new(crate::agent::openai::OpenAIProvider::new(
                    client,
                    k,
                    model_config,
                )),
                None => {
                    tracing::warn!(
                        "⚠️ [IngestionWorker] No OPENAI_API_KEY for embedding — using NullProvider"
                    );
                    Box::new(crate::agent::null_provider::NullProvider::new(
                        &agent.identity.id,
                        crate::agent::null_provider::NullReason::MissingApiKey {
                            env_var: "OPENAI_API_KEY",
                        },
                    ))
                }
            }
        }
        _ => {
            // Fallback: try Gemini with env key
            let key = std::env::var("GOOGLE_API_KEY").ok();
            match key {
                Some(k) => Box::new(crate::agent::gemini::GeminiProvider::new(
                    client,
                    k,
                    model_config,
                )),
                None => {
                    tracing::warn!(
                        "⚠️ [IngestionWorker] No API key for embedding — using NullProvider"
                    );
                    Box::new(crate::agent::null_provider::NullProvider::new(
                        &agent.identity.id,
                        crate::agent::null_provider::NullReason::MissingApiKey {
                            env_var: "GOOGLE_API_KEY",
                        },
                    ))
                }
            }
        }
    }
}

// Metadata: [connectors]

// Metadata: [connectors]

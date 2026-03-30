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

#[async_trait]
#[allow(dead_code)]
pub trait ConnectorTrait: Send + Sync {
    /// Unique identifier for this connector instance (e.g. agent_id + source)
    fn id(&self) -> &str;

    /// Type of source (slack, notion, fs)
    fn source_type(&self) -> &str;

    /// Fetches new or updated items since the last sync time.
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

                if let Some(last) = last_sync {
                    if modified <= last {
                        continue;
                    }
                }

                // Phase 4: Use layout-aware parser for all supported formats
                match crate::agent::parser::parse_file(path) {
                    Ok(doc) => {
                        // Use structured chunks for better embedding quality
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

async fn run_ingestion_cycle(state: &crate::state::AppState) -> Result<()> {
    let pool = &state.resources.pool;
    let manifests = crate::agent::persistence::load_sync_manifests(pool).await?;

    for manifest in manifests {
        if manifest.status == "syncing" {
            continue; // Skip if already in progress
        }

        // Lookup agent from DashMap registry
        let agent = match state.registry.agents.get(&manifest.agent_id) {
            Some(entry) => entry.value().clone(),
            None => continue,
        };

        tracing::info!(
            "📂 [IngestionWorker] Syncing {} for agent {}",
            manifest.source_uri,
            agent.id
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

                // Resolve memory path for this agent
                let memory_path = format!("data/memory/{}/knowledge.lance", agent.id);
                let mem =
                    match crate::agent::memory::VectorMemory::connect(&memory_path, "memories")
                        .await
                    {
                        Ok(m) => m,
                        Err(e) => {
                            tracing::error!(
                                "❌ [IngestionWorker] Failed to connect VectorMemory for {}: {}",
                                agent.id,
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
                    };

                // Resolve an embedding provider from the agent's model config
                let client = (*state.resources.http_client).clone();
                let provider = resolve_embedding_provider(&agent, client);

                let mut latest_update = manifest.last_sync_at;

                for item in items {
                    match provider.embed(&item.content).await {
                        Ok(vec) => {
                            let _ = mem
                                .add_memory(&item.id, &item.content, "sync-cycle", vec)
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
fn resolve_embedding_provider(
    agent: &crate::agent::types::EngineAgent,
    client: reqwest::Client,
) -> Box<dyn crate::agent::provider_trait::LlmProvider> {
    let provider_name = agent.model.provider.to_lowercase();
    let model_config = agent.model.clone();

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
                        &agent.id,
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
                        &agent.id,
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
                        &agent.id,
                        crate::agent::null_provider::NullReason::MissingApiKey {
                            env_var: "GOOGLE_API_KEY",
                        },
                    ))
                }
            }
        }
    }
}

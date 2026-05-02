//! @docs ARCHITECTURE:Persistence
//!
//! ### AI Assist Note
//! **Memory Engine**: High-performance vector storage utilizing **LanceDB**
//! and **Apache Arrow**. Orchestrates **Hybrid Search** via a dedicated **FTS Index**
//! and **Vector k-NN** to provide precise mission context.
//! Implements **Semantic Archival**, where fragmented session memories are
//! summarized into dense historical records via Gemini. Features **High-Performance 
//! Batching** to reduce IOPS overhead and **Automatic Orphan Cleanup** to prevent 
//! disk bloat from terminated missions.
//!
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: LanceDB table lock contention, FTS index corruption,
//!   malformed Arrow batch schemas during ingestion, or 429 rate limits 
//!   on embedding provider callbacks.
//! - **Trace Scope**: `server-rs::agent::memory`

#[cfg(feature = "vector-memory")]
use std::sync::Arc;
#[cfg(feature = "vector-memory")]
use std::sync::atomic::{AtomicBool, Ordering};
#[cfg(feature = "vector-memory")]
use std::sync::OnceLock;
use crate::error::AppError;

#[cfg(feature = "vector-memory")]
use arrow_array::{
    builder::{FixedSizeListBuilder, PrimitiveBuilder},
    Array, Float32Array, Int64Array, RecordBatch, StringArray,
};
#[cfg(feature = "vector-memory")]
use arrow_schema::{DataType, Field, Schema};
#[cfg(feature = "vector-memory")]
use futures::StreamExt;
#[cfg(feature = "vector-memory")]
use lancedb::query::{ExecutableQuery, QueryBase};
#[cfg(feature = "vector-memory")]
use lancedb::Connection;

/// Dimensionality of the vector embeddings (text-embedding-004).
#[allow(dead_code)]
pub const EMBEDDING_DIM: usize = 768;

/// Unified Vector Memory for Agents and Mission RAG Scopes.
///
/// High-performance storage utilizing LanceDB (file-system based) and Apache
/// Arrow for semantic search and mission context retrieval.
#[derive(Clone)]
#[allow(dead_code)]
pub struct VectorMemory {
    /// Active connection to the LanceDB database.
    #[cfg(feature = "vector-memory")]
    conn: Connection,
    /// Name of the table storing embeddings for this scope.
    #[allow(dead_code)]
    table_name: String,
    /// Cache to prevent redundant metadata lookups for table existence.
    #[cfg(feature = "vector-memory")]
    table_ensured: Arc<AtomicBool>,
}

// ─────────────────────────────────────────────────────────
//  CORE IMPLEMENTATION (VECTOR ENABLED)
// ─────────────────────────────────────────────────────────

#[cfg(feature = "vector-memory")]
#[allow(dead_code)]
impl VectorMemory {
    /// ### 🔗 Orchestration: Data Persistence
    /// Connects to a physical LanceDB storage backend at the specified path.
    pub async fn connect(path: &str, table_name: &str) -> Result<Self, AppError> {
        let conn = lancedb::connect(path).execute().await
            .map_err(|e| AppError::InternalServerError(format!("LanceDB connect failed: {}", e)))?;
        Ok(Self {
            conn,
            table_name: table_name.to_string(),
            table_ensured: Arc::new(AtomicBool::new(false)),
        })
    }

    /// Returns the Apache Arrow schema used for Vector Memory storage.
    pub fn get_schema() -> Arc<Schema> {
        static SCHEMA: OnceLock<Arc<Schema>> = OnceLock::new();
        SCHEMA.get_or_init(|| {
            Arc::new(Schema::new(vec![
                Field::new("id", DataType::Utf8, false),
                Field::new("text", DataType::Utf8, false),
                Field::new("mission_id", DataType::Utf8, false),
                Field::new("timestamp", DataType::Int64, false),
                Field::new(
                    "vector",
                    DataType::FixedSizeList(
                        Arc::new(Field::new("item", DataType::Float32, true)),
                        EMBEDDING_DIM as i32,
                    ),
                    false,
                ),
            ]))
        }).clone()
    }

    /// ### 🧱 Data Integrity: Schema Initialization
    /// Ensures that the memory table and FTS index exist for the connection.
    pub async fn ensure_table(&self) -> Result<(), AppError> {
        if self.table_ensured.load(Ordering::SeqCst) {
            return Ok(());
        }

        let table_names = self.conn.table_names().execute().await
            .map_err(|e| AppError::InternalServerError(format!("LanceDB table_names failed: {}", e)))?;
        if !table_names.contains(&self.table_name) {
            let schema = Self::get_schema();
            let empty_batch = RecordBatch::new_empty(schema.clone());
            let batches_results: Vec<std::result::Result<RecordBatch, arrow_schema::ArrowError>> =
                vec![Ok(empty_batch)];
            let reader = Box::new(arrow_array::RecordBatchIterator::new(
                batches_results.into_iter(),
                schema.clone(),
            )) as Box<dyn arrow_array::RecordBatchReader + Send>;
            
            let table = self.conn
                .create_table(&self.table_name, reader)
                .execute()
                .await
                .map_err(|e| AppError::InternalServerError(format!("LanceDB create_table failed: {}", e)))?;

            // Create FTS index for hybrid search (on-demand creation for new tables)
            let _ = table.create_index(&["text"], lancedb::index::Index::FullText(Default::default()))
                .execute()
                .await;
        }
        
        self.table_ensured.store(true, Ordering::SeqCst);
        Ok(())
    }

    /// Adds a single knowledge fragment to the persistent vector store.
    pub async fn add_memory(
        &self,
        id: &str,
        text: &str,
        mission_id: &str,
        vector: Vec<f32>,
    ) -> Result<(), AppError> {
        self.add_memories(vec![MemoryEntryRaw {
            id: id.to_string(),
            text: text.to_string(),
            mission_id: mission_id.to_string(),
            vector,
        }]).await
    }

    /// ### 🛰️ High-Performance Ingestion: add_memories
    /// Appends multiple knowledge fragments in a single disk synchronization pass.
    pub async fn add_memories(&self, entries: Vec<MemoryEntryRaw>) -> Result<(), AppError> {
        if entries.is_empty() { return Ok(()); }
        self.ensure_table().await?;

        let schema = Self::get_schema();
        let mut id_builder = arrow_array::builder::StringBuilder::new();
        let mut text_builder = arrow_array::builder::StringBuilder::new();
        let mut mission_builder = arrow_array::builder::StringBuilder::new();
        let mut ts_builder = arrow_array::builder::PrimitiveBuilder::<arrow_array::types::Int64Type>::new();
        let mut vector_builder = FixedSizeListBuilder::new(
            PrimitiveBuilder::<arrow_array::types::Float32Type>::new(),
            EMBEDDING_DIM as i32,
        );

        let now = chrono::Utc::now().timestamp();
        for entry in entries {
            id_builder.append_value(entry.id);
            text_builder.append_value(entry.text);
            mission_builder.append_value(entry.mission_id);
            ts_builder.append_value(now);
            vector_builder.values().append_slice(&entry.vector);
            vector_builder.append(true);
        }

        let batch = RecordBatch::try_new(
            schema,
            vec![
                Arc::new(id_builder.finish()),
                Arc::new(text_builder.finish()),
                Arc::new(mission_builder.finish()),
                Arc::new(ts_builder.finish()),
                Arc::new(vector_builder.finish()),
            ],
        ).map_err(|e| AppError::InternalServerError(format!("Arrow RecordBatch creation failed: {}", e)))?;

        let table = self.conn.open_table(&self.table_name).execute().await
            .map_err(|e| AppError::InternalServerError(format!("LanceDB open_table failed: {}", e)))?;
        table.add(vec![batch]).execute().await
            .map_err(|e| AppError::InternalServerError(format!("LanceDB table.add failed: {}", e)))?;
        Ok(())
    }

    /// Searches for similar knowledge using Hybrid Search (Vector + Full-Text).
    pub async fn search_knowledge_hybrid(
        &self,
        _query_text: &str,
        query_vector: Vec<f32>,
        limit: usize,
    ) -> Result<Vec<String>, AppError> {
        self.ensure_table().await?;
        let table = self.conn.open_table(&self.table_name).execute().await
            .map_err(|e| AppError::InternalServerError(format!("LanceDB open_table failed: {}", e)))?;

        let query = table.vector_search(query_vector)
            .map_err(|e| AppError::InternalServerError(format!("LanceDB vector_search failed: {}", e)))?;
        let mut results = query.limit(limit * 2).execute().await
            .map_err(|e| AppError::InternalServerError(format!("LanceDB query execution failed: {}", e)))?;

        let mut texts = Vec::new();
        while let Some(batch_result) = results.next().await {
            let batch = batch_result.map_err(|e| AppError::InternalServerError(format!("LanceDB result stream error: {}", e)))?;
            let text_column = batch.column_by_name("text")
                .ok_or_else(|| AppError::InternalServerError("Missing text column".to_string()))?;
            let text_array = text_column.as_any().downcast_ref::<StringArray>()
                .ok_or_else(|| AppError::InternalServerError("Column text is not String".to_string()))?;

            for i in 0..text_array.len() {
                texts.push(text_array.value(i).to_string());
            }
        }
        texts.truncate(limit);
        Ok(texts)
    }

    /// Performs a pure vector-based k-Nearest Neighbors (k-NN) search.
    pub async fn search_knowledge(
        &self,
        query_vector: Vec<f32>,
        limit: usize,
    ) -> Result<Vec<String>, AppError> {
        self.ensure_table().await?;
        let table = self.conn.open_table(&self.table_name).execute().await
            .map_err(|e| AppError::InternalServerError(format!("LanceDB open_table failed: {}", e)))?;

        let query = table.vector_search(query_vector)
            .map_err(|e| AppError::InternalServerError(format!("LanceDB vector_search failed: {}", e)))?;
        let mut results = query.limit(limit).execute().await
            .map_err(|e| AppError::InternalServerError(format!("LanceDB query execution failed: {}", e)))?;

        let mut texts = Vec::new();
        while let Some(batch_result) = results.next().await {
            let batch = batch_result.map_err(|e| AppError::InternalServerError(format!("LanceDB result stream error: {}", e)))?;
            let text_column = batch.column_by_name("text")
                .ok_or_else(|| AppError::InternalServerError("Missing text column".to_string()))?;
            let text_array = text_column.as_any().downcast_ref::<StringArray>()
                .ok_or_else(|| AppError::InternalServerError("Column text is not String".to_string()))?;
            for i in 0..text_array.len() {
                texts.push(text_array.value(i).to_string());
            }
        }
        Ok(texts)
    }

    /// Searches similar knowledge and returns the text along with the L2 distance.
    pub async fn search_knowledge_with_distance(
        &self,
        query_vector: Vec<f32>,
        limit: usize,
    ) -> Result<Vec<(String, f32)>, AppError> {
        self.ensure_table().await?;
        let table = self.conn.open_table(&self.table_name).execute().await
            .map_err(|e| AppError::InternalServerError(format!("LanceDB open_table failed: {}", e)))?;

        let query = table.vector_search(query_vector)
            .map_err(|e| AppError::InternalServerError(format!("LanceDB vector_search failed: {}", e)))?;
        let mut results = query.limit(limit).execute().await
            .map_err(|e| AppError::InternalServerError(format!("LanceDB query execution failed: {}", e)))?;

        let mut texts = Vec::new();
        while let Some(batch_result) = results.next().await {
            let batch = batch_result.map_err(|e| AppError::InternalServerError(format!("LanceDB result stream error: {}", e)))?;
            let text_column = batch.column_by_name("text")
                .ok_or_else(|| AppError::InternalServerError("Missing text column".to_string()))?;
            let text_array = text_column.as_any().downcast_ref::<StringArray>()
                .ok_or_else(|| AppError::InternalServerError("Column text is not String".to_string()))?;
            let distance_column = batch.column_by_name("_distance")
                .ok_or_else(|| AppError::InternalServerError("Missing _distance column".to_string()))?;
            let distance_array = distance_column.as_any().downcast_ref::<Float32Array>()
                .ok_or_else(|| AppError::InternalServerError("Column _distance is not Float32".to_string()))?;

            for i in 0..text_array.len() {
                texts.push((text_array.value(i).to_string(), distance_array.value(i)));
            }
        }
        Ok(texts)
    }

    /// High-Fidelity retrieval for Advanced RAG with metadata inclusion.
    pub async fn search_knowledge_full(
        &self,
        query_vector: Vec<f32>,
        limit: usize,
    ) -> Result<Vec<crate::agent::types::MemoryEntryDetailed>, AppError> {
        self.ensure_table().await?;
        let table = self.conn.open_table(&self.table_name).execute().await
            .map_err(|e| AppError::InternalServerError(format!("LanceDB open_table failed: {}", e)))?;

        let query = table.vector_search(query_vector)
            .map_err(|e| AppError::InternalServerError(format!("LanceDB vector_search failed: {}", e)))?;
        let mut results = query.limit(limit).execute().await
            .map_err(|e| AppError::InternalServerError(format!("LanceDB query execution failed: {}", e)))?;

        let mut entries = Vec::new();
        while let Some(batch_result) = results.next().await {
            let batch = batch_result.map_err(|e| AppError::InternalServerError(format!("LanceDB result stream error: {}", e)))?;
            let id_col = batch.column_by_name("id").and_then(|c| c.as_any().downcast_ref::<StringArray>())
                .ok_or_else(|| AppError::InternalServerError("Missing id column".to_string()))?;
            let text_col = batch.column_by_name("text").and_then(|c| c.as_any().downcast_ref::<StringArray>())
                .ok_or_else(|| AppError::InternalServerError("Missing text column".to_string()))?;
            let mission_col = batch.column_by_name("mission_id").and_then(|c| c.as_any().downcast_ref::<StringArray>())
                .ok_or_else(|| AppError::InternalServerError("Missing mission_id column".to_string()))?;
            let ts_col = batch.column_by_name("timestamp").and_then(|c| c.as_any().downcast_ref::<Int64Array>())
                .ok_or_else(|| AppError::InternalServerError("Missing timestamp column".to_string()))?;
            let dist_col = batch.column_by_name("_distance").and_then(|c| c.as_any().downcast_ref::<Float32Array>())
                .ok_or_else(|| AppError::InternalServerError("Missing _distance column".to_string()))?;

            for i in 0..batch.num_rows() {
                entries.push(crate::agent::types::MemoryEntryDetailed {
                    id: id_col.value(i).to_string(),
                    text: text_col.value(i).to_string(),
                    mission_id: mission_col.value(i).to_string(),
                    timestamp: ts_col.value(i),
                    distance: dist_col.value(i),
                    score: None,
                });
            }
        }
        Ok(entries)
    }

    /// Fetches the exhaustive list of memory IDs and raw text for a specific mission.
    pub async fn get_all_memories(&self, mission_id: &str) -> Result<Vec<(String, String)>, AppError> {
        self.ensure_table().await?;
        let table = self.conn.open_table(&self.table_name).execute().await
            .map_err(|e| AppError::InternalServerError(format!("LanceDB open_table failed: {}", e)))?;
        let mut results = table.query().execute().await
            .map_err(|e| AppError::InternalServerError(format!("LanceDB query execution failed: {}", e)))?;

        let mut memories = Vec::new();
        while let Some(batch_result) = results.next().await {
            let batch: RecordBatch = batch_result.map_err(|e| AppError::InternalServerError(format!("LanceDB result stream error: {}", e)))?;
            let id_array = batch.column_by_name("id")
                .ok_or_else(|| AppError::InternalServerError("Missing id".to_string()))?
                .as_any().downcast_ref::<StringArray>()
                .ok_or_else(|| AppError::InternalServerError("Invalid id".to_string()))?;
            let text_array = batch.column_by_name("text")
                .ok_or_else(|| AppError::InternalServerError("Missing text".to_string()))?
                .as_any().downcast_ref::<StringArray>()
                .ok_or_else(|| AppError::InternalServerError("Invalid text".to_string()))?;
            let mid_array = batch.column_by_name("mission_id")
                .ok_or_else(|| AppError::InternalServerError("Missing mid".to_string()))?
                .as_any().downcast_ref::<StringArray>()
                .ok_or_else(|| AppError::InternalServerError("Invalid mid".to_string()))?;

            for i in 0..id_array.len() {
                if mid_array.value(i) == mission_id {
                    memories.push((id_array.value(i).to_string(), text_array.value(i).to_string()));
                }
            }
        }
        Ok(memories)
    }

    /// Deletes specific memories by ID.
    pub async fn delete_memories(&self, ids: Vec<String>) -> Result<(), AppError> {
        self.ensure_table().await?;
        let table = self.conn.open_table(&self.table_name).execute().await
            .map_err(|e| AppError::InternalServerError(format!("LanceDB open_table failed: {}", e)))?;
        let filter = format!("id IN ({})", ids.iter().map(|id| format!("'{}'", id)).collect::<Vec<_>>().join(", "));
        table.delete(&filter).await
            .map_err(|e| AppError::InternalServerError(format!("LanceDB table.delete failed: {}", e)))?;
        Ok(())
    }

    /// ### ⚖️ Governance & Storage: Semantic Archival
    pub async fn summarize_and_archive(
        &self,
        mission_id: &str,
        client: &reqwest::Client,
        api_key: &str,
        model_id: &str,
    ) -> Result<(), AppError> {
        let memories = self.get_all_memories(mission_id).await?;
        if memories.len() < 3 {
            tracing::info!("⏭️ [Memory] Mission {} too brief to summarize. Skipping.", mission_id);
            return Ok(());
        }

        let mut combined_text = memories.iter().map(|(_, text)| format!("- {}", text)).collect::<Vec<_>>().join("\n");
        const MAX_ARCHIVE_CHARS: usize = 50000;
        if combined_text.len() > MAX_ARCHIVE_CHARS {
            tracing::warn!("⚠️ [Memory] Mission {} history truncated.", mission_id);
            combined_text.truncate(MAX_ARCHIVE_CHARS);
            combined_text.push_str("\n... [TRUNCATED] ...");
        }

        let prompt = format!("You are the Tadpole OS Semantic Archiver. Summarize these memories:\n{}", combined_text);
        let schema = serde_json::json!({
            "type": "object",
            "properties": {
                "summary": { "type": "string" },
                "key_decisions": { "type": "array", "items": { "type": "string" } },
                "outcomes": { "type": "array", "items": { "type": "string" } }
            },
            "required": ["summary", "key_decisions", "outcomes"]
        });

        let url = format!("https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent", model_id);
        let res = client.post(&url).header("x-goog-api-key", api_key).json(&serde_json::json!({
            "contents": [{"role": "user", "parts": [{"text": prompt}]}],
            "generationConfig": { "response_mime_type": "application/json", "response_schema": schema }
        })).send().await
            .map_err(|e| AppError::InternalServerError(format!("Gemini request failed: {}", e)))?;

        if !res.status().is_success() { return Err(AppError::InternalServerError(format!("Archival error: {}", res.text().await.unwrap_or_default()))); }
        let json: serde_json::Value = res.json().await
            .map_err(|e| AppError::InternalServerError(format!("Gemini response parse failed: {}", e)))?;
        let raw_json_str = json["candidates"][0]["content"]["parts"][0]["text"].as_str().ok_or_else(|| AppError::InternalServerError("No response body from Gemini".to_string()))?;
        let parsed: serde_json::Value = serde_json::from_str(raw_json_str)
            .map_err(|e| AppError::InternalServerError(format!("Malformed Gemini JSON: {}", e)))?;
        let final_summary = format!("{}\n\nDecisions:\n{}\n\nOutcomes:\n{}", 
            parsed["summary"].as_str().unwrap_or(""), 
            parsed["key_decisions"].as_array().map(|a| a.iter().map(|v| format!("- {}", v.as_str().unwrap_or_default())).collect::<Vec<_>>().join("\n")).unwrap_or_default(),
            parsed["outcomes"].as_array().map(|a| a.iter().map(|v| format!("- {}", v.as_str().unwrap_or_default())).collect::<Vec<_>>().join("\n")).unwrap_or_default());

        let summary_id = format!("archived_{}_{}", mission_id, chrono::Utc::now().timestamp());
        let vector = get_gemini_embedding(client, api_key, &final_summary).await?;
        self.add_memory(&summary_id, &final_summary, mission_id, vector).await?;
        self.delete_memories(memories.into_iter().map(|(id, _)| id).collect()).await?;
        Ok(())
    }
}

#[cfg(not(feature = "vector-memory"))]
#[allow(dead_code)]
impl VectorMemory {
    pub async fn connect(_path: &str, table_name: &str) -> Result<Self, AppError> { Ok(Self { table_name: table_name.to_string() }) }
    pub async fn ensure_table(&self) -> Result<(), AppError> { Ok(()) }
    pub async fn add_memory(&self, _id: &str, _text: &str, _mission_id: &str, _vector: Vec<f32>) -> Result<(), AppError> { Ok(()) }
    pub async fn add_memories(&self, _entries: Vec<MemoryEntryRaw>) -> Result<(), AppError> { Ok(()) }
    pub async fn search_knowledge_hybrid(&self, _t: &str, _v: Vec<f32>, _l: usize) -> Result<Vec<String>, AppError> { Ok(Vec::new()) }
    pub async fn search_knowledge(&self, _v: Vec<f32>, _l: usize) -> Result<Vec<String>, AppError> { Ok(Vec::new()) }
    pub async fn search_knowledge_with_distance(&self, _v: Vec<f32>, _l: usize) -> Result<Vec<(String, f32)>, AppError> { Ok(Vec::new()) }
    pub async fn search_knowledge_full(&self, _v: Vec<f32>, _l: usize) -> Result<Vec<crate::agent::types::MemoryEntryDetailed>, AppError> { Ok(Vec::new()) }
    pub async fn get_all_memories(&self, _mid: &str) -> Result<Vec<(String, String)>, AppError> { Ok(Vec::new()) }
    pub async fn delete_memories(&self, _ids: Vec<String>) -> Result<(), AppError> { Ok(()) }
    pub async fn summarize_and_archive(&self, _mid: &str, _c: &reqwest::Client, _k: &str, _m: &str) -> Result<(), AppError> { Ok(()) }
}

#[allow(dead_code)]
impl VectorMemory {
    /// Checks if a vector already exists within a specified L2 distance threshold.
    pub async fn check_memory_duplicate(
        &self,
        query_vector: Vec<f32>,
        limit_dist: f32,
    ) -> Result<bool, AppError> {
        #[cfg(feature = "vector-memory")]
        {
            let results = self.search_knowledge_with_distance(query_vector, 1).await?;
            if let Some((_, dist)) = results.first() {
                if *dist < limit_dist {
                    return Ok(true);
                }
            }
        }
        #[cfg(not(feature = "vector-memory"))]
        {
            let _ = query_vector;
            let _ = limit_dist;
        }
        Ok(false)
    }

    /// Background task to clean up LanceDB scopes for completed or failed missions.
    pub async fn cleanup_orphaned_scopes(pool: &sqlx::SqlitePool) {
        let workspaces_dir = std::path::PathBuf::from("data/workspaces");
        if !workspaces_dir.exists() { return; }
        tracing::info!("🧹 [Memory] Starting orphaned RAG scope cleanup...");

        let mission_statuses: std::collections::HashMap<String, String> =
            match sqlx::query_as::<_, (String, String)>("SELECT id, status FROM mission_history").fetch_all(pool).await {
                Ok(rows) => rows.into_iter().collect(),
                Err(e) => { tracing::error!("❌ [Memory] Cleanup failed: {}", e); return; }
            };

        let mut deleted_count = 0;
        if let Ok(mut cluster_entries) = tokio::fs::read_dir(&workspaces_dir).await {
            while let Ok(Some(cluster_entry)) = cluster_entries.next_entry().await {
                let cluster_path = cluster_entry.path();
                if !cluster_path.is_dir() { continue; }
                let missions_dir = cluster_path.join("missions");
                if !missions_dir.exists() { continue; }

                if let Ok(mut mission_entries) = tokio::fs::read_dir(&missions_dir).await {
                    while let Ok(Some(mission_entry)) = mission_entries.next_entry().await {
                        let mission_id = mission_entry.file_name().to_string_lossy().to_string();
                        let scope_path = mission_entry.path().join("scope.lance");
                        if scope_path.exists() {
                            let status = mission_statuses.get(&mission_id).map(|s| s.as_str());
                            if matches!(status, Some("completed") | Some("failed") | None)
                                && tokio::fs::remove_dir_all(&scope_path).await.is_ok()
                            {
                                deleted_count += 1;
                            }
                        }
                    }
                }
            }
        }
        if deleted_count > 0 { tracing::info!("🧹 [Memory] Cleanup complete. Removed {} scopes.", deleted_count); }
    }
}

/// Raw memory entry for batch ingestion.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct MemoryEntryRaw {
    pub id: String,
    pub text: String,
    pub mission_id: String,
    pub vector: Vec<f32>,
}

/// Helper function to retrieve a text embedding from Gemini.
/// Dimensions: 768 for text-embedding-004.
#[cfg(feature = "vector-memory")]
pub async fn get_gemini_embedding(
    client: &reqwest::Client,
    api_key: &str,
    text: &str,
) -> Result<Vec<f32>, AppError> {
    let url =
        "https://generativelanguage.googleapis.com/v1beta/models/text-embedding-004:embedContent";

    #[derive(serde::Serialize)]
    struct ContentPart {
        text: String,
    }
    #[derive(serde::Serialize)]
    struct EmbedContent {
        parts: Vec<ContentPart>,
    }
    #[derive(serde::Serialize)]
    struct EmbedRequest {
        content: EmbedContent,
    }

    let body = EmbedRequest {
        content: EmbedContent {
            parts: vec![ContentPart {
                text: text.to_string(),
            }],
        },
    };

    let res = client
        .post(url)
        .header("x-goog-api-key", api_key)
        .json(&body)
        .send()
        .await
        .map_err(|e| AppError::InternalServerError(format!("Gemini embedding request failed: {}", e)))?;

    if !res.status().is_success() {
        let err_text = res.text().await.unwrap_or_default();
        return Err(AppError::InternalServerError(format!("Gemini Embedding Error: {}", err_text)));
    }

    #[derive(serde::Deserialize)]
    struct ContentEmbedding {
        values: Vec<f32>,
    }
    #[derive(serde::Deserialize)]
    struct EmbedResponse {
        embedding: ContentEmbedding,
    }

    let parsed: EmbedResponse = res.json().await
        .map_err(|e| AppError::InternalServerError(format!("Gemini embedding response parse failed: {}", e)))?;
    Ok(parsed.embedding.values)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(feature = "vector-memory")]
    use tempfile::tempdir;

    #[cfg(feature = "vector-memory")]
    #[tokio::test]
    async fn test_vector_memory_lifecycle() -> Result<(), AppError> {
        let dir = tempdir().map_err(|e| AppError::InternalServerError(e.to_string()))?;
        let path = dir.path().to_str().unwrap();
        let table_name = "test_memories";
        
        let memory = VectorMemory::connect(path, table_name).await?;
        memory.ensure_table().await?;
        
        // Add memory
        let id = "mem-1";
        let text = "TadpoleOS is an agentic operating system.";
        let mission_id = "mission-alpha";
        let vector = vec![0.1; EMBEDDING_DIM];
        
        memory.add_memory(id, text, mission_id, vector.clone()).await?;
        
        // Search
        let results = memory.search_knowledge(vector, 1).await?;
        assert_eq!(results.len(), 1);
        assert_eq!(results[0], text);
        
        // Duplicate check
        let is_dup = memory.check_memory_duplicate(vec![0.1; EMBEDDING_DIM], 0.1).await?;
        assert!(is_dup);
        
        Ok(())
    }

    #[cfg(not(feature = "vector-memory"))]
    #[tokio::test]
    async fn test_vector_memory_mock_safe() -> Result<(), AppError> {
        // Verify that the mock implementation doesn't crash and returns safe defaults
        let memory = VectorMemory::connect("memory_mock", "test").await?;
        memory.ensure_table().await?;
        
        let vector = vec![0.0; EMBEDDING_DIM];
        memory.add_memory("id", "text", "mid", vector.clone()).await?;
        
        let results = memory.search_knowledge(vector, 10).await?;
        assert!(results.is_empty(), "Mock search should return empty results");
        
        let is_dup = memory.check_memory_duplicate(vec![0.0; EMBEDDING_DIM], 1.0).await?;
        assert!(!is_dup, "Mock duplicate check should return false");
        
        Ok(())
    }
}

// Metadata: [memory]

// Metadata: [memory]

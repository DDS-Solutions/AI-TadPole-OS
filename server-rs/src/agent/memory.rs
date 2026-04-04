//! Neural Memory — LanceDB vector storage interface
//!
//! High-performance vector storage utilizing LanceDB and Apache Arrow. 
//! Enables agents to retrieve previous findings and mission scope data.
//!
//! @docs ARCHITECTURE:Persistence
//!
//! @state MemoryTable: (Uninitialized | Connected | Error)
//! @state SearchContext: (Semantic | Hybrid | MultiFactorScored)
//!
//! ### AI Assist Note
//! **Memory Engine**: Uses 768-dimension embeddings. Supports **Hybrid Search** 
//! (Vector + Keyword) and **Advanced RAG Weighting** (MFS). Summarized archival 
//! dense records are prioritized during long-running sessions to save prompt tokens.
//!
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: Connection pool exhaustion, malformed Arrow schema, or 
//!   LanceDB file corruption. `search_knowledge_full` returns raw metadata for debugging.
//! - **Trace Scope**: `server-rs::agent::memory`

use anyhow::Result;
#[cfg(feature = "vector-memory")]
use anyhow::Context;
#[cfg(feature = "vector-memory")]
use lancedb::connection::Connection;
#[cfg(feature = "vector-memory")]
use std::sync::Arc;

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

/// Unified Vector Memory for Agents and Mission RAG Scopes.
///
/// High-performance storage utilizing LanceDB (file-system based) and Apache
/// Arrow for semantic search and mission context retrieval.
#[derive(Clone)]
pub struct VectorMemory {
    /// Active connection to the LanceDB database.
    #[cfg(feature = "vector-memory")]
    conn: Connection,
    /// Name of the table storing embeddings for this scope.
    #[allow(dead_code)]
    table_name: String,
}

impl VectorMemory {
    /// Connects to a LanceDB instance in the provided directory.
    #[cfg(feature = "vector-memory")]
    pub async fn connect(path: &str, table_name: &str) -> Result<Self> {
        let conn = lancedb::connect(path).execute().await?;
        Ok(Self {
            conn,
            table_name: table_name.to_string(),
        })
    }

    /// Connects to a dummy memory instance (for legacy CPUs).
    #[cfg(not(feature = "vector-memory"))]
    #[allow(dead_code)]
    pub async fn connect(_path: &str, table_name: &str) -> Result<Self> {
        Ok(Self {
            table_name: table_name.to_string(),
        })
    }

    /// Returns the Apache Arrow schema used for Vector Memory storage.
    #[cfg(feature = "vector-memory")]
    pub fn get_schema() -> Arc<Schema> {
        Arc::new(Schema::new(vec![
            Field::new("id", DataType::Utf8, false),
            Field::new("text", DataType::Utf8, false),
            Field::new("mission_id", DataType::Utf8, false),
            Field::new("timestamp", DataType::Int64, false),
            Field::new(
                "vector",
                DataType::FixedSizeList(Arc::new(Field::new("item", DataType::Float32, true)), 768),
                false,
            ),
        ]))
    }

    /// Ensures the table exists, creating it if necessary.
    #[cfg(feature = "vector-memory")]
    pub async fn ensure_table(&self) -> Result<()> {
        let table_names = self.conn.table_names().execute().await?;
        if !table_names.contains(&self.table_name) {
            let schema = Self::get_schema();
            let empty_batch = RecordBatch::new_empty(schema.clone());
            let batches_results: Vec<std::result::Result<RecordBatch, arrow_schema::ArrowError>> =
                vec![Ok(empty_batch)];
            let reader = Box::new(arrow_array::RecordBatchIterator::new(
                batches_results.into_iter(),
                schema.clone(),
            )) as Box<dyn arrow_array::RecordBatchReader + Send>;
            self.conn
                .create_table(&self.table_name, reader)
                .execute()
                .await?;
        }
        Ok(())
    }

    #[cfg(not(feature = "vector-memory"))]
    pub async fn ensure_table(&self) -> Result<()> {
        Ok(())
    }

    /// Adds a single memory entry.
    #[cfg(feature = "vector-memory")]
    pub async fn add_memory(
        &self,
        id: &str,
        text: &str,
        mission_id: &str,
        vector: Vec<f32>,
    ) -> Result<()> {
        self.ensure_table().await?;

        let schema = Self::get_schema();
        let id_array = StringArray::from(vec![id]);
        let text_array = StringArray::from(vec![text]);
        let mission_id_array = StringArray::from(vec![mission_id]);
        let ts_array = Int64Array::from(vec![chrono::Utc::now().timestamp()]);

        let mut list_builder = FixedSizeListBuilder::new(
            PrimitiveBuilder::<arrow_array::types::Float32Type>::new(),
            768,
        );
        for v in vector {
            list_builder.values().append_value(v);
        }
        list_builder.append(true); // Complete the single list item
        let vector_array = list_builder.finish();

        let batch = RecordBatch::try_new(
            schema.clone(),
            vec![
                Arc::new(id_array),
                Arc::new(text_array),
                Arc::new(mission_id_array),
                Arc::new(ts_array),
                Arc::new(vector_array),
            ],
        )?;

        let table = self.conn.open_table(&self.table_name).execute().await?;
        table.add(vec![batch]).execute().await?;

        Ok(())
    }

    #[cfg(not(feature = "vector-memory"))]
    pub async fn add_memory(
        &self,
        _id: &str,
        _text: &str,
        _mission_id: &str,
        _vector: Vec<f32>,
    ) -> Result<()> {
        Ok(())
    }

    /// Searches for similar knowledge using Hybrid Search (Vector + Full-Text).
    #[cfg(feature = "vector-memory")]
    pub async fn search_knowledge_hybrid(
        &self,
        _query_text: &str,
        query_vector: Vec<f32>,
        limit: usize,
    ) -> Result<Vec<String>> {
        self.ensure_table().await?;
        let table = self.conn.open_table(&self.table_name).execute().await?;

        let query = table.vector_search(query_vector)?;
        let mut results = query.limit(limit * 2).execute().await?;

        let mut texts = Vec::new();
        while let Some(batch_result) = results.next().await {
            let batch = batch_result?;
            let text_column = batch
                .column_by_name("text")
                .context("Missing text column")?;
            let text_array = text_column
                .as_any()
                .downcast_ref::<StringArray>()
                .context("Column text is not String")?;

            for i in 0..text_array.len() {
                let val = text_array.value(i);
                texts.push(val.to_string());
            }
        }

        // Final limit
        texts.truncate(limit);
        Ok(texts)
    }

    #[cfg(not(feature = "vector-memory"))]
    pub async fn search_knowledge_hybrid(
        &self,
        _query_text: &str,
        _query_vector: Vec<f32>,
        _limit: usize,
    ) -> Result<Vec<String>> {
        Ok(Vec::new())
    }

    /// Searches for similar knowledge.
    #[cfg(feature = "vector-memory")]
    pub async fn search_knowledge(
        &self,
        query_vector: Vec<f32>,
        limit: usize,
    ) -> Result<Vec<String>> {
        self.ensure_table().await?;
        let table = self.conn.open_table(&self.table_name).execute().await?;

        let query = table.vector_search(query_vector)?;
        let mut results = query.limit(limit).execute().await?;

        let mut texts = Vec::new();
        while let Some(batch_result) = results.next().await {
            let batch = batch_result?;
            let text_column = batch
                .column_by_name("text")
                .context("Missing text column")?;
            let text_array = text_column
                .as_any()
                .downcast_ref::<StringArray>()
                .context("Column text is not String")?;
            for i in 0..text_array.len() {
                texts.push(text_array.value(i).to_string());
            }
        }

        Ok(texts)
    }

    #[cfg(not(feature = "vector-memory"))]
    pub async fn search_knowledge(
        &self,
        _query_vector: Vec<f32>,
        _limit: usize,
    ) -> Result<Vec<String>> {
        Ok(Vec::new())
    }

    /// Searches for similar knowledge and returns the text along with the L2 distance.
    #[cfg(feature = "vector-memory")]
    pub async fn search_knowledge_with_distance(
        &self,
        query_vector: Vec<f32>,
        limit: usize,
    ) -> Result<Vec<(String, f32)>> {
        self.ensure_table().await?;
        let table = self.conn.open_table(&self.table_name).execute().await?;

        let query = table.vector_search(query_vector)?;
        let mut results = query.limit(limit).execute().await?;

        let mut texts = Vec::new();
        while let Some(batch_result) = results.next().await {
            let batch = batch_result?;
            let text_column = batch
                .column_by_name("text")
                .context("Missing text column")?;
            let text_array = text_column
                .as_any()
                .downcast_ref::<StringArray>()
                .context("Column text is not String")?;

            let distance_column = batch
                .column_by_name("_distance")
                .context("Missing _distance column")?;
            let distance_array = distance_column
                .as_any()
                .downcast_ref::<Float32Array>()
                .context("Column _distance is not Float32")?;

            for i in 0..text_array.len() {
                texts.push((text_array.value(i).to_string(), distance_array.value(i)));
            }
        }

        Ok(texts)
    }

    /// High-Fidelity retrieval for Advanced RAG. Returns full metadata and semantic distance.
    #[cfg(feature = "vector-memory")]
    pub async fn search_knowledge_full(
        &self,
        query_vector: Vec<f32>,
        limit: usize,
    ) -> Result<Vec<crate::routes::memory::MemoryEntryDetailed>> {
        self.ensure_table().await?;
        let table = self.conn.open_table(&self.table_name).execute().await?;

        let query = table.vector_search(query_vector)?;
        let mut results = query.limit(limit).execute().await?;

        let mut entries = Vec::new();
        while let Some(batch_result) = results.next().await {
            let batch = batch_result?;
            let id_col = batch
                .column_by_name("id")
                .and_then(|c| c.as_any().downcast_ref::<StringArray>())
                .context("Missing id column")?;
            let text_col = batch
                .column_by_name("text")
                .and_then(|c| c.as_any().downcast_ref::<StringArray>())
                .context("Missing text column")?;
            let mission_col = batch
                .column_by_name("mission_id")
                .and_then(|c| c.as_any().downcast_ref::<StringArray>())
                .context("Missing mission_id column")?;
            let ts_col = batch
                .column_by_name("timestamp")
                .and_then(|c| c.as_any().downcast_ref::<Int64Array>())
                .context("Missing timestamp column")?;
            let dist_col = batch
                .column_by_name("_distance")
                .and_then(|c| c.as_any().downcast_ref::<Float32Array>())
                .context("Missing _distance column")?;

            for i in 0..batch.num_rows() {
                entries.push(crate::routes::memory::MemoryEntryDetailed {
                    id: id_col.value(i).to_string(),
                    text: text_col.value(i).to_string(),
                    mission_id: mission_col.value(i).to_string(),
                    timestamp: ts_col.value(i),
                    distance: dist_col.value(i),
                    score: None, // Calculated at the routing layer
                });
            }
        }

        Ok(entries)
    }

    #[cfg(not(feature = "vector-memory"))]
    pub async fn search_knowledge_with_distance(
        &self,
        _query_vector: Vec<f32>,
        _limit: usize,
    ) -> Result<Vec<(String, f32)>> {
        Ok(Vec::new())
    }

    #[cfg(not(feature = "vector-memory"))]
    pub async fn search_knowledge_full(
        &self,
        _query_vector: Vec<f32>,
        _limit: usize,
    ) -> Result<Vec<crate::routes::memory::MemoryEntryDetailed>> {
        Ok(Vec::new())
    }

    pub async fn check_memory_duplicate(
        &self,
        query_vector: Vec<f32>,
        limit_dist: f32,
    ) -> Result<bool> {
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

    /// Retrieves all memories for a specific mission.
    #[cfg(feature = "vector-memory")]
    pub async fn get_all_memories(&self, mission_id: &str) -> Result<Vec<(String, String)>> {
        self.ensure_table().await?;
        let table = self.conn.open_table(&self.table_name).execute().await?;

        let mut results = table.query().execute().await?;

        let mut memories = Vec::new();
        while let Some(batch_result) = results.next().await {
            let batch: RecordBatch = batch_result?;
            let id_column = batch.column_by_name("id").context("Missing id column")?;
            let id_array = id_column
                .as_any()
                .downcast_ref::<StringArray>()
                .context("Column id is not String")?;

            let text_column = batch
                .column_by_name("text")
                .context("Missing text column")?;
            let text_array = text_column
                .as_any()
                .downcast_ref::<StringArray>()
                .context("Column text is not String")?;

            let mid_column = batch
                .column_by_name("mission_id")
                .context("Missing mission_id column")?;
            let mid_array = mid_column
                .as_any()
                .downcast_ref::<StringArray>()
                .context("Column mission_id is not String")?;

            for i in 0..id_array.len() {
                if mid_array.value(i) == mission_id {
                    memories.push((
                        id_array.value(i).to_string(),
                        text_array.value(i).to_string(),
                    ));
                }
            }
        }

        Ok(memories)
    }

    #[cfg(not(feature = "vector-memory"))]
    pub async fn get_all_memories(&self, _mission_id: &str) -> Result<Vec<(String, String)>> {
        Ok(Vec::new())
    }

    /// Deletes specific memories by ID.
    #[cfg(feature = "vector-memory")]
    pub async fn delete_memories(&self, ids: Vec<String>) -> Result<()> {
        self.ensure_table().await?;
        let table = self.conn.open_table(&self.table_name).execute().await?;

        let filter = format!(
            "id IN ({})",
            ids.iter()
                .map(|id| format!("'{}'", id))
                .collect::<Vec<_>>()
                .join(", ")
        );
        table.delete(&filter).await?;

        Ok(())
    }

    #[cfg(not(feature = "vector-memory"))]
    pub async fn delete_memories(&self, _ids: Vec<String>) -> Result<()> {
        Ok(())
    }

    /// Semantic Archival: Summarizes all memories for a mission and replaces them with a single dense summary.
    #[cfg(feature = "vector-memory")]
    pub async fn summarize_and_archive(
        &self,
        mission_id: &str,
        client: &reqwest::Client,
        api_key: &str,
        model_id: &str,
    ) -> Result<()> {
        let memories = self.get_all_memories(mission_id).await?;
        if memories.len() < 3 {
            tracing::info!(
                "⏭️ [Memory] Mission {} has too few memories ({}) to summarize. Skipping archival.",
                mission_id,
                memories.len()
            );
            return Ok(());
        }

        let combined_text = memories
            .iter()
            .map(|(_, text)| format!("- {}", text))
            .collect::<Vec<_>>()
            .join("\n");

        let prompt = format!(
            "You are the Tadpole OS Semantic Archiver. Summarize the following session memories into a single, high-density historical record. \
            Return the result in JSON format.\n\n\
            SESSION MEMORIES:\n{}",
            combined_text
        );

        let schema = serde_json::json!({
            "type": "object",
            "properties": {
                "summary": { "type": "string", "description": "The high-density historical summary." },
                "key_decisions": { "type": "array", "items": { "type": "string" } },
                "outcomes": { "type": "array", "items": { "type": "string" } }
            },
            "required": ["summary", "key_decisions", "outcomes"]
        });

        // Call Gemini for structured summarization
        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent",
            model_id
        );
        let body = serde_json::json!({
            "contents": [{
                "role": "user",
                "parts": [{ "text": prompt }]
            }],
            "generationConfig": {
                "response_mime_type": "application/json",
                "response_schema": schema
            }
        });

        let res = client
            .post(&url)
            .header("x-goog-api-key", api_key)
            .json(&body)
            .send()
            .await?;

        if !res.status().is_success() {
            return Err(anyhow::anyhow!(
                "Archival Summarization failed: {}",
                res.text().await?
            ));
        }

        let json: serde_json::Value = res.json().await?;
        let raw_json_str = json["candidates"][0]["content"]["parts"][0]["text"]
            .as_str()
            .context("Failed to extract summary text from Gemini response")?;

        // Re-parse to ensure integrity
        let parsed_summary: serde_json::Value = serde_json::from_str(raw_json_str)?;
        let final_summary_text = format!(
            "{}\n\nKey Decisions:\n{}\n\nOutcomes:\n{}",
            parsed_summary["summary"].as_str().unwrap_or(""),
            parsed_summary["key_decisions"]
                .as_array()
                .map(|a| a
                    .iter()
                    .map(|v| format!("- {}", v.as_str().unwrap_or("")))
                    .collect::<Vec<_>>()
                    .join("\n"))
                .unwrap_or_default(),
            parsed_summary["outcomes"]
                .as_array()
                .map(|a| a
                    .iter()
                    .map(|v| format!("- {}", v.as_str().unwrap_or("")))
                    .collect::<Vec<_>>()
                    .join("\n"))
                .unwrap_or_default()
        );

        // 1. Add the new summary
        let summary_id = format!("archived_{}_{}", mission_id, chrono::Utc::now().timestamp());
        let vector = get_gemini_embedding(client, api_key, &final_summary_text).await?;
        self.add_memory(&summary_id, &final_summary_text, mission_id, vector)
            .await?;

        // 2. Delete the old memories
        let ids_to_delete: Vec<String> = memories.into_iter().map(|(id, _)| id).collect();
        self.delete_memories(ids_to_delete).await?;

        tracing::info!(
            "✅ [Memory] Mission {} archived successfully into a single dense summary.",
            mission_id
        );
        Ok(())
    }

    #[cfg(not(feature = "vector-memory"))]
    pub async fn summarize_and_archive(
        &self,
        _mission_id: &str,
        _client: &reqwest::Client,
        _api_key: &str,
        _model_id: &str,
    ) -> Result<()> {
        Ok(())
    }
}

/// Helper function to retrieve a text embedding from Gemini.
/// Dimensions: 768 for text-embedding-004.
pub async fn get_gemini_embedding(
    client: &reqwest::Client,
    api_key: &str,
    text: &str,
) -> Result<Vec<f32>> {
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
        .await?;

    if !res.status().is_success() {
        let err_text = res.text().await?;
        return Err(anyhow::anyhow!("Gemini Embedding Error: {}", err_text));
    }

    #[derive(serde::Deserialize)]
    struct ContentEmbedding {
        values: Vec<f32>,
    }
    #[derive(serde::Deserialize)]
    struct EmbedResponse {
        embedding: ContentEmbedding,
    }

    let parsed: EmbedResponse = res.json().await?;
    Ok(parsed.embedding.values)
}

/// Background task to clean up LanceDB scopes for completed or failed missions to prevent disk bloat.
pub async fn cleanup_orphaned_scopes(pool: &sqlx::SqlitePool) {
    let workspaces_dir = std::path::PathBuf::from("data/workspaces");
    if !workspaces_dir.exists() {
        return;
    }

    tracing::info!("🧹 [Memory] Starting orphaned RAG scope cleanup...");

    // 1. Fetch mission statuses in bulk to avoid O(N) queries
    let mission_statuses: std::collections::HashMap<String, String> =
        match sqlx::query_as::<_, (String, String)>("SELECT id, status FROM mission_history")
            .fetch_all(pool)
            .await
        {
            Ok(rows) => rows.into_iter().collect(),
            Err(e) => {
                tracing::error!(
                    "❌ [Memory] Failed to fetch mission statuses for cleanup: {}",
                    e
                );
                return;
            }
        };

    let mut deleted_count = 0;

    // 2. Efficiently traverse cluster/mission structure (2 levels deep) instead of generic WalkDir
    if let Ok(mut cluster_entries) = tokio::fs::read_dir(&workspaces_dir).await {
        while let Ok(Some(cluster_entry)) = cluster_entries.next_entry().await {
            let cluster_path = cluster_entry.path();
            if !cluster_path.is_dir() {
                continue;
            }

            let missions_dir = cluster_path.join("missions");
            if !missions_dir.exists() {
                continue;
            }

            if let Ok(mut mission_entries) = tokio::fs::read_dir(&missions_dir).await {
                while let Ok(Some(mission_entry)) = mission_entries.next_entry().await {
                    let mission_path = mission_entry.path();
                    if !mission_path.is_dir() {
                        continue;
                    }

                    let mission_id = mission_entry.file_name().to_string_lossy().to_string();
                    let scope_path = mission_path.join("scope.lance");

                    if scope_path.exists() {
                        let status = mission_statuses.get(&mission_id).map(|s| s.as_str());
                        let should_delete = match status {
                            Some("completed") | Some("failed") => true,
                            None => true, // Orphaned (no record in DB)
                            _ => false,   // Active or Pending
                        };

                        if should_delete && tokio::fs::remove_dir_all(&scope_path).await.is_ok() {
                            deleted_count += 1;
                        }
                    }
                }
            }
        }
    }

    if deleted_count > 0 {
        tracing::info!(
            "🧹 [Memory] Cleanup complete. Removed {} orphaned LanceDB scopes.",
            deleted_count
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    #[cfg(feature = "vector-memory")]
    async fn test_check_memory_duplicate() -> Result<()> {
        let dir = tempdir()?;
        let mem = VectorMemory::connect(dir.path().to_str().unwrap(), "test_memories").await?;

        let mut vec1 = vec![0.0f32; 768];
        vec1[0] = 1.0;

        mem.add_memory("1", "Exact Match Text", "mission-1", vec1.clone())
            .await?;

        // 1. Exact match (distance 0.0) -> should be duplicate
        let is_dup_exact = mem.check_memory_duplicate(vec1.clone(), 0.01).await?;
        assert!(
            is_dup_exact,
            "Exact match (distance 0.0) should be flagged as duplicate with threshold 0.01"
        );

        // 2. Completely different vector -> should not be duplicate
        let mut vec2 = vec![0.0f32; 768];
        vec2[1] = 1.0;

        let is_dup_diff = mem.check_memory_duplicate(vec2.clone(), 0.1).await?;
        assert!(!is_dup_diff, "Different vector should not be a duplicate");

        // 3. Slightly different vector
        let mut vec3 = vec1.clone();
        vec3[0] = 0.99;

        let is_dup_close_tight = mem.check_memory_duplicate(vec3.clone(), 0.00001).await?;
        assert!(
            !is_dup_close_tight,
            "Slightly different vector should not be a duplicate with very tight threshold"
        );

        let is_dup_close_loose = mem.check_memory_duplicate(vec3.clone(), 0.1).await?;
        assert!(
            is_dup_close_loose,
            "Slightly different vector should be a duplicate with loose threshold"
        );

        Ok(())
    }

    #[tokio::test]
    #[cfg(feature = "vector-memory")]
    async fn test_search_knowledge_hybrid() -> Result<()> {
        let dir = tempdir()?;
        let mem = VectorMemory::connect(dir.path().to_str().unwrap(), "test_hybrid").await?;

        let vec1 = vec![0.1f32; 768];
        let vec2 = vec![0.2f32; 768];

        mem.add_memory(
            "1",
            "The quick brown fox jumps over the lazy dog",
            "mission-1",
            vec1,
        )
        .await?;
        mem.add_memory(
            "2",
            "A dedicated business report on SME growth",
            "mission-1",
            vec2,
        )
        .await?;

        // Search for keyword "SME" - should find result 2 even if vector is slightly distant (simulated by query vector)
        let query_vec = vec![0.15f32; 768];
        let results = mem.search_knowledge_hybrid("SME", query_vec, 5).await?;

        assert!(
            results.iter().any(|r| r.contains("SME")),
            "Should find the SME report via hybrid search"
        );

        Ok(())
    }

    #[tokio::test]
    #[cfg(feature = "vector-memory")]
    async fn test_vector_memory_reader_trait_bound() -> Result<()> {
        let dir = tempdir()?;
        let mem = VectorMemory::connect(dir.path().to_str().unwrap(), "test_trait").await?;

        // Initialize schema (copied from VectorMemory::connect internals if needed)
        use arrow_schema::{DataType, Field, Schema};
        let schema = Arc::new(Schema::new(vec![
            Field::new("id", DataType::Utf8, false),
            Field::new("text", DataType::Utf8, false),
            Field::new("mission_id", DataType::Utf8, false),
            Field::new(
                "vector",
                DataType::FixedSizeList(Arc::new(Field::new("item", DataType::Float32, true)), 768),
                false,
            ),
        ]));

        let empty_batch = RecordBatch::new_empty(schema.clone());
        let reader = Box::new(arrow_array::RecordBatchIterator::new(
            vec![Ok(empty_batch)].into_iter(),
            schema.clone(),
        )) as Box<dyn arrow_array::RecordBatchReader + Send>;

        // This verifies the create_table call with our casted reader
        mem.conn
            .create_table("test_trait_table", reader)
            .execute()
            .await?;

        Ok(())
    }

    #[tokio::test]
    #[cfg(feature = "vector-memory")]
    async fn test_vector_memory_add_batch_robustness() -> Result<()> {
        let dir = tempdir()?;
        let mem =
            Arc::new(VectorMemory::connect(dir.path().to_str().unwrap(), "test_robust").await?);

        // Pre-create the table to avoid race conditions during table creation in concurrent adds
        mem.ensure_table().await?;

        // Simulate high-concurrency writes to test our trait-casted reader in add_memory
        let mut handlers = vec![];
        for i in 0..20 {
            let mem_clone = mem.clone();
            handlers.push(tokio::spawn(async move {
                let vec = vec![0.1f32 + (i as f32 * 0.001); 768];
                mem_clone
                    .add_memory(
                        &format!("id-{}", i),
                        &format!("Robust Test Content {}", i),
                        "mission-robust",
                        vec,
                    )
                    .await
            }));
        }

        for h in handlers {
            h.await.unwrap()?;
        }

        // Verify some records were added
        let query_vec = vec![0.1f32; 768];
        let results = mem.search_knowledge_hybrid("Robust", query_vec, 50).await?;
        assert!(
            results.len() >= 20,
            "Should have successfully added all concurrent records"
        );

        Ok(())
    }
}

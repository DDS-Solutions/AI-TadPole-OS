use anyhow::{Context, Result};
#[cfg(feature = "vector-memory")]
use lancedb::connection::Connection;
use std::sync::Arc;

#[cfg(feature = "vector-memory")]
use arrow_array::{
    builder::{FixedSizeListBuilder, PrimitiveBuilder},
    Array, Float32Array, Int64Array, RecordBatch, StringArray,
};
#[cfg(feature = "vector-memory")]
use arrow_schema::{DataType, Field, Schema};
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
            let reader =
                arrow_array::RecordBatchIterator::new(batches_results.into_iter(), schema.clone());
            self.conn
                .create_table(&self.table_name, Box::new(reader))
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
        // ... (implementation omitted for brevity in replace call, but I'll provide the full block in a real edit)
        // Actually I should provide the full block to keep it correct.
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
        let batches_results: Vec<std::result::Result<RecordBatch, arrow_schema::ArrowError>> =
            vec![Ok(batch)];
        let reader =
            arrow_array::RecordBatchIterator::new(batches_results.into_iter(), schema.clone());
        table.add(Box::new(reader)).execute().await?;

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
        // No-op for legacy builds
        Ok(())
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

    #[cfg(not(feature = "vector-memory"))]
    pub async fn search_knowledge_with_distance(
        &self,
        _query_vector: Vec<f32>,
        _limit: usize,
    ) -> Result<Vec<(String, f32)>> {
        Ok(Vec::new())
    }

    /// Checks if a functionally identical memory already exists based on a distance threshold.
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
    pub async fn summarize_and_archive(
        &self,
        mission_id: &str,
        client: &reqwest::Client,
        api_key: &str,
        model_id: &str,
    ) -> Result<()> {
        #[cfg(feature = "vector-memory")]
        {
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
                Focus on key decisions, findings, and outcomes. Keep the summary under 500 words but ensure no critical insight is lost.\n\n\
                SESSION MEMORIES:\n{}\n\nFINAL ARCHIVAL SUMMARY:",
                combined_text
            );

            // Call Gemini for summarization
            let url = format!(
                "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent",
                model_id
            );
            let body = serde_json::json!({
                "contents": [{
                    "role": "user",
                    "parts": [{ "text": prompt }]
                }]
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
            let summary = json["candidates"][0]["content"]["parts"][0]["text"]
                .as_str()
                .context("Failed to extract summary text from Gemini response")?;

            // 1. Add the new summary
            let summary_id = format!("archived_{}_{}", mission_id, chrono::Utc::now().timestamp());
            let vector = get_gemini_embedding(client, api_key, summary).await?;
            self.add_memory(&summary_id, summary, mission_id, vector)
                .await?;

            // 2. Delete the old memories
            let ids_to_delete: Vec<String> = memories.into_iter().map(|(id, _)| id).collect();
            self.delete_memories(ids_to_delete).await?;

            tracing::info!(
                "✅ [Memory] Mission {} archived successfully into a single dense summary.",
                mission_id
            );
        }
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
}

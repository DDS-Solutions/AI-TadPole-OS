//! @docs ARCHITECTURE:Networking
//! @docs OPERATIONS_MANUAL:MemorySearch
//!
//! ### AI Assist Note
//! **Memory Semantic Bridge**: Orchestrates the REST surface for
//! **Long-term Memory** (RAG) and vector lookups. Features **Tiered
//! Affinity**: contextual boosting of results based on active
//! Mission-ID. Implements **Multi-Factor Scoring (MFS)**: advanced
//! result ranking combining semantic distance, temporal recency,
//! and mission relevance. Note: This module interacts with
//! LanceDB/Arrow for high-speed retrieval; SQL-injection risks in
//! predicate filters are mitigated via `escape_lancedb_string_literal`
//! (MEM-01).
//!
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: 404/500 if `vector-memory` feature is disabled,
//!   LanceDB table lock contention during concurrent writes, or
//!   embedding dimension mismatches on provider switching.
//! - **Trace Scope**: `server-rs::routes::memory`

use crate::agent::memory::VectorMemory;
use crate::agent::types::MemoryEntryDetailed;
use crate::error::AppError;
use crate::state::AppState;
#[cfg(feature = "vector-memory")]
use arrow_array::{Int64Array, StringArray};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use futures::StreamExt;
#[cfg(feature = "vector-memory")]
use lancedb::query::{ExecutableQuery, QueryBase};
use serde::Serialize;
use std::sync::Arc;

/// Escapes single quotes for safe embedding in LanceDB/DataFusion string literals.
fn escape_lancedb_string_literal(value: &str) -> String {
    value.replace('\'', "''")
}

/// Resolves the canonical workspaces root from app state.
fn workspaces_root(base_dir: &std::path::Path) -> Result<std::path::PathBuf, AppError> {
    crate::utils::security::validate_path(base_dir, "data/workspaces")
}

/// Robustly resolves the path to an agent's memory store by scanning workspaces.
fn resolve_agent_memory_path(base_dir: &std::path::Path, agent_id: &str) -> Option<std::path::PathBuf> {
    let _safe_agent_id = crate::utils::security::sanitize_id(agent_id);
    let workspaces_dir = workspaces_root(base_dir).ok()?;

    if !workspaces_dir.exists() {
        return None;
    }

    // Single-pass resolution using recursive searching or shallow walking
    if let Ok(entries) = std::fs::read_dir(&workspaces_dir) {
        for entry in entries.flatten() {
            let entry_path = entry.path();
            if entry_path.is_dir() {
                let agents_base = entry_path.join("agents");
                if let Ok(valid_agent_dir) = crate::utils::security::validate_path(&agents_base, &_safe_agent_id) {
                    let potential_path = valid_agent_dir.join("memory.lance");
                    if potential_path.exists() {
                        return Some(potential_path);
                    }
                }
            }
        }
    }
    None
}

/// Lists all valid agent memory paths across all clusters and workspaces.
fn list_all_memory_paths(base_dir: &std::path::Path) -> Vec<std::path::PathBuf> {
    let mut paths = Vec::new();
    let workspaces_dir = match workspaces_root(base_dir) {
        Ok(d) => d,
        Err(_) => return paths,
    };

    if !workspaces_dir.exists() {
        return paths;
    }

    if let Ok(workspace_entries) = std::fs::read_dir(&workspaces_dir) {
        for workspace in workspace_entries.flatten() {
            let agents_dir = workspace.path().join("agents");
            if agents_dir.exists() {
                if let Ok(agent_entries) = std::fs::read_dir(&agents_dir) {
                    for agent in agent_entries.flatten() {
                        let db_path = agent.path().join("memory.lance");
                        if db_path.exists() {
                            paths.push(db_path);
                        }
                    }
                }
            }
        }
    }
    paths
}

/// A single entry retrieved from the agent's long-term vector database.
#[derive(Serialize)]
pub struct MemoryEntry {
    /// Semantic Row ID.
    pub id: String,
    /// The actual text content of the memory.
    pub text: String,
    /// Originating mission ID.
    pub mission_id: String,
    /// Unix timestamp of creation.
    pub timestamp: i64,
}

#[cfg(feature = "vector-memory")]
impl TryFrom<&arrow_array::RecordBatch> for Vec<MemoryEntry> {
    type Error = AppError;

    fn try_from(batch: &arrow_array::RecordBatch) -> Result<Self, Self::Error> {
        use arrow_array::{Int64Array, StringArray};
        
        let id_col = batch
            .column_by_name("id")
            .and_then(|c| c.as_any().downcast_ref::<StringArray>())
            .ok_or_else(|| AppError::InternalServerError("Missing 'id' column in memory batch".to_string()))?;
            
        let text_col = batch
            .column_by_name("text")
            .and_then(|c| c.as_any().downcast_ref::<StringArray>())
            .ok_or_else(|| AppError::InternalServerError("Missing 'text' column in memory batch".to_string()))?;
            
        let mission_col = batch
            .column_by_name("mission_id")
            .and_then(|c| c.as_any().downcast_ref::<StringArray>())
            .ok_or_else(|| AppError::InternalServerError("Missing 'mission_id' column in memory batch".to_string()))?;
            
        let ts_col = batch
            .column_by_name("timestamp")
            .and_then(|c| c.as_any().downcast_ref::<Int64Array>())
            .ok_or_else(|| AppError::InternalServerError("Missing 'timestamp' column in memory batch".to_string()))?;

        let mut entries = Vec::with_capacity(batch.num_rows());
        for i in 0..batch.num_rows() {
            entries.push(MemoryEntry {
                id: id_col.value(i).to_string(),
                text: text_col.value(i).to_string(),
                mission_id: mission_col.value(i).to_string(),
                timestamp: ts_col.value(i),
            });
        }
        Ok(entries)
    }
}

/// Standardized response for agent memory retrieval.
#[derive(Serialize)]
pub struct MemoryResponse {
    /// Request status ("success", "error").
    pub status: String,
    /// List of retrieved memory entries.
    pub entries: Vec<MemoryEntry>,
}


/// Request payload for persisting new semantic memories.
#[derive(serde::Deserialize)]
pub struct SaveMemoryRequest {
    /// The raw text content to be vectorized and stored.
    pub text: String,
}

/// GET /v1/agents/:agent_id/memories
///
/// Retrieves semantic memories for a specific agent by scanning its local
/// workspace's LanceDB vector store.
///
/// @docs API_REFERENCE:GetAgentMemory
pub async fn get_agent_memory(
    Path(agent_id): Path<String>,
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, AppError> {
    #[cfg(not(feature = "vector-memory"))]
    {
        let _ = &state;
        return Ok((
            StatusCode::OK,
            Json(MemoryResponse {
                status: "success".to_string(),
                entries: vec![],
            }),
        ));
    }

    #[cfg(feature = "vector-memory")]
    {
        let memory_path = resolve_agent_memory_path(&state.base_dir, &agent_id).ok_or_else(|| {
            AppError::NotFound(format!("Memory for agent {} not found", agent_id))
        })?;

        let path_str = memory_path.to_string_lossy().to_string();

        match VectorMemory::connect(&path_str, "memories").await {
            Ok(memory) => {
                if let Err(e) = memory.ensure_table().await {
                    tracing::warn!("Table not found or error: {}", e);
                    return Ok((
                        StatusCode::OK,
                        Json(MemoryResponse {
                            status: "success".to_string(),
                            entries: vec![],
                        }),
                    ));
                }

                if let Ok(conn) = lancedb::connect(&format!("file://{}", path))
                    .execute()
                    .await
                {
                    if let Ok(table) = conn.open_table("memories").execute().await {
                        if let Ok(mut results) = table.query().limit(100).execute().await {
                            let mut entries = Vec::new();
                            while let Some(batch_result) = results.next().await {
                                if let Ok(batch) = batch_result {
                                    if let Ok(mut batch_entries) = Vec::<MemoryEntry>::try_from(&batch) {
                                        entries.append(&mut batch_entries);
                                    }
                                }
                            }
                            entries.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
                            return Ok((
                                StatusCode::OK,
                                Json(MemoryResponse {
                                    status: "success".to_string(),
                                    entries,
                                }),
                            ));
                        }
                    }
                }
                Err(AppError::InternalServerError(
                    "Failed to query memory store".to_string(),
                ))
            }
            Err(e) => {
                tracing::error!("Failed to connect to memory: {}", e);
                Err(AppError::InternalServerError(format!(
                    "Failed to connect to memory: {}",
                    e
                )))
            }
        }
    }
}

/// DELETE /v1/agents/:agent_id/memories/:row_id
pub async fn delete_agent_memory(
    Path((agent_id, row_id)): Path<(String, String)>,
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, AppError> {
    #[cfg(not(feature = "vector-memory"))]
    {
        let _ = &state;
        return Ok((
            StatusCode::OK,
            Json(serde_json::json!({"status": "success"})),
        ));
    }

    #[cfg(feature = "vector-memory")]
    {
        let memory_path = resolve_agent_memory_path(&state.base_dir, &agent_id).ok_or_else(|| {
            AppError::NotFound(format!("Memory for agent {} not found", agent_id))
        })?;

        let path_str = memory_path.to_string_lossy().to_string();

        if let Ok(conn) = lancedb::connect(&format!("file://{}", path_str))
            .execute()
            .await
        {
            if let Ok(table) = conn.open_table("memories").execute().await {
                let escaped_row_id = escape_lancedb_string_literal(&row_id);
                let predicate = format!("id = '{}'", escaped_row_id);
                table.delete(&predicate).await.map_err(|e| {
                    AppError::InternalServerError(format!("Failed to delete record: {}", e))
                })?;
            }
        }

        Ok((
            StatusCode::OK,
            Json(serde_json::json!({"status": "success"})),
        ))
    }
}

/// POST /v1/agents/:agent_id/memories
#[cfg(feature = "vector-memory")]
pub async fn save_agent_memory(
    Path(agent_id): Path<String>,
    State(state): State<Arc<AppState>>,
    Json(payload): Json<SaveMemoryRequest>,
) -> Result<impl IntoResponse, AppError> {
    let agent_ctx = state
        .registry
        .agents
        .get(&agent_id)
        .ok_or_else(|| AppError::NotFound(format!("Agent {} not found", agent_id)))?;

    // Use unified provider abstraction
    let runner = crate::agent::runner::AgentRunner::new(state.clone());
    let provider = runner.resolve_provider(
        &agent_ctx.resolve_provider_context(state.base_dir.clone()),
        (*state.resources.http_client).clone(),
    );
    let embedding = provider
        .embed(&payload.text)
        .await
        .map_err(|e| AppError::InternalServerError(format!("Embedding failed: {}", e)))?;

    // Sanitize agent_id for path safety
    let safe_id = crate::utils::security::sanitize_id(&agent_id);
    // Find or create agent memory path
    let cluster_id = std::env::var("CLUSTER_ID").unwrap_or_else(|_| "default".to_string());
    let _safe_cluster_id = crate::utils::security::sanitize_id(&cluster_id);

    // Security (Alert #25): Anchor the path construction and validate against workspace boundaries
    let agents_base = workspaces_root(&state.base_dir)?
        .join(&_safe_cluster_id)
        .join("agents");

    let agent_dir = crate::utils::security::validate_path(&agents_base, &safe_id)
        .map_err(|e| AppError::BadRequest(format!("Invalid agent path: {}", e)))?;

    tokio::fs::create_dir_all(&agent_dir)
        .await
        .map_err(|e| AppError::InternalServerError(format!("Failed to create directory: {}", e)))?;
    let path = agent_dir.join("memory.lance");
    let path_str = path.to_string_lossy().to_string();

    match VectorMemory::connect(&path_str, "memories").await {
        Ok(memory) => {
            let id = uuid::Uuid::new_v4().to_string();
            memory
                .add_memory(&id, &payload.text, "manual", embedding)
                .await
                .map_err(|e| {
                    AppError::InternalServerError(format!("Failed to persist memory: {}", e))
                })?;

            Ok((
                StatusCode::OK,
                Json(serde_json::json!({"status": "success", "id": id})),
            ))
        }
        Err(e) => {
            tracing::error!("Failed to connect to memory: {}", e);
            Err(AppError::InternalServerError(
                "Failed to connect to memory store".to_string(),
            ))
        }
    }
}

/// POST /v1/agents/:agent_id/memories (Fallback)
#[cfg(not(feature = "vector-memory"))]
pub async fn save_agent_memory(
    Path(_agent_id): Path<String>,
    State(_state): State<Arc<AppState>>,
    Json(_payload): Json<SaveMemoryRequest>,
) -> Result<impl IntoResponse, AppError> {
    Ok((
        StatusCode::OK,
        Json(serde_json::json!({"status": "success", "id": "placeholder"})),
    ))
}

#[derive(serde::Deserialize)]
pub struct SearchRequest {
    pub query: String,
    /// Optional mission ID to provide contextual boost (affinity Tier 1).
    pub mission_id: Option<String>,
}

/// GET /v1/search/memory?query=...&mission_id=...
#[cfg(feature = "vector-memory")]
pub async fn global_search(
    State(state): State<Arc<AppState>>,
    axum::extract::Query(params): axum::extract::Query<SearchRequest>,
) -> Result<impl IntoResponse, AppError> {
    if params.query.is_empty() {
        return Ok((
            StatusCode::OK,
            Json(serde_json::json!({
                "status": "success",
                "entries": []
            })),
        ));
    }

    // Resolve a provider for embedding
    let runner = crate::agent::runner::AgentRunner::new(state.clone());
    let provider = if let Some(agent_ctx) = state.registry.agents.iter().next() {
        runner.resolve_provider(
            &agent_ctx.resolve_provider_context(state.base_dir.clone()),
            (*state.resources.http_client).clone(),
        )
    } else {
        return Err(AppError::InternalServerError(
            "No agents configured for search".to_string(),
        ));
    };

    let query_vec = provider
        .embed(&params.query)
        .await
        .map_err(|e| AppError::InternalServerError(format!("Search embedding failed: {}", e)))?;

    let mut all_results = Vec::new();
    let scoring_config = crate::types::rag_scoring::ScoringConfig::default();

    // Iterate across workspaces
    {
        let memory_paths = list_all_memory_paths(&state.base_dir);
        for db_path in memory_paths {
            if let Ok(mem) = VectorMemory::connect(&db_path.to_string_lossy(), "memories").await {
                if let Ok(hits) = mem.search_knowledge_full(query_vec.clone(), 5).await {
                    // Process and score hits
                    for mut hit in hits {
                        let mfs = crate::types::rag_scoring::calculate_mfs(
                            hit.distance,
                            &hit.mission_id,
                            params.mission_id.as_deref(),
                            hit.timestamp,
                            &scoring_config,
                        );
                        hit.score = Some(mfs);
                        all_results.push(hit);
                    }
                }
            }
        }
    }

    // Sort by final score descending
    all_results.sort_by(|a, b| {
        let score_a = a.score.as_ref().map(|s| s.final_score).unwrap_or(0.0);
        let score_b = b.score.as_ref().map(|s| s.final_score).unwrap_or(0.0);
        score_b
            .partial_cmp(&score_a)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    // Limit to top 20 global results
    all_results.truncate(20);

    Ok((
        StatusCode::OK,
        Json(serde_json::json!({
            "status": "success",
            "entries": all_results
        })),
    ))
}

/// GET /v1/search/memory (Fallback)
#[cfg(not(feature = "vector-memory"))]
pub async fn global_search(
    State(_state): State<Arc<AppState>>,
    axum::extract::Query(_params): axum::extract::Query<SearchRequest>,
) -> Result<impl IntoResponse, AppError> {
    Ok((
        StatusCode::OK,
        Json(MemoryResponse {
            status: "success".to_string(),
            entries: vec![],
        }),
    ))
}

#[cfg(test)]
mod tests {
    use super::{escape_lancedb_string_literal, workspaces_root};
    use std::path::PathBuf;

    #[test]
    fn escapes_single_quotes_for_predicate_safety() {
        assert_eq!(escape_lancedb_string_literal("abc"), "abc");
        assert_eq!(escape_lancedb_string_literal("a'b"), "a''b");
        assert_eq!(
            escape_lancedb_string_literal("x' OR 1=1 --"),
            "x'' OR 1=1 --"
        );
    }

    #[test]
    fn workspaces_root_anchors_under_base_dir() {
        let base_dir = PathBuf::from("workspace-root");
        // validate_path expects absolute path if base is not absolute, 
        // so we use a mock absolute base for the test.
        #[cfg(not(windows))]
        let base_dir = PathBuf::from("/tmp/workspace-root");
        #[cfg(windows)]
        let base_dir = PathBuf::from("C:\\workspace-root");
        
        let expected = base_dir.join("data").join("workspaces");
        assert_eq!(workspaces_root(&base_dir).unwrap(), expected);
    }
}

// Metadata: [memory]

// Metadata: [memory]

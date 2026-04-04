//! Long-term Memory — RAG and vector search API
//!
//! Provides the primary interface for semantic retrieval and context persistence. 
//! Interacts with LanceDB/Arrow for high-speed vector lookups.
//!
//! @docs ARCHITECTURE:Networking
//! @docs OPERATIONS_MANUAL:MemorySearch
//!
//! @state SearchPipeline: (Semantic | MFS-Prioritized)
//!
//! ### AI Assist Note
//! **Feature Gating**: All endpoints lead to LanceDB/Arrow. Use `global_search` 
//! with a `mission_id` for Tier 1 context affinity. Memory operations can be 
//! heavy; use the `TelemetricLayer` to observe retrieval latency.
//!
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: 404 if `vector-memory` feature is disabled, LanceDB 
//!   table lock errors, or embedding provider timeouts.
//! - **Trace Scope**: `server-rs::routes::memory`

use crate::agent::memory::VectorMemory;
use crate::routes::error::AppError;
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
fn workspaces_root(base_dir: &std::path::Path) -> std::path::PathBuf {
    base_dir.join("data").join("workspaces")
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

/// Standardized response for agent memory retrieval.
#[derive(Serialize)]
pub struct MemoryResponse {
    /// Request status ("success", "error").
    pub status: String,
    /// List of retrieved memory entries.
    pub entries: Vec<MemoryEntry>,
}

/// Enhanced memory entry with scoring metadata for Advanced RAG.
#[derive(Serialize, Clone)]
pub struct MemoryEntryDetailed {
    pub id: String,
    pub text: String,
    pub mission_id: String,
    pub timestamp: i64,
    /// Raw semantic distance from vector search.
    pub distance: f32,
    /// Final calculated Multi-Factor Score.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub score: Option<crate::types::rag_scoring::RagScore>,
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
        // Security (Alert #27, #23): Sanitize agent_id and validate path to prevent traversal
        let _safe_agent_id = crate::utils::security::sanitize_id(&agent_id);
        let workspaces_dir = workspaces_root(&state.base_dir);
        let mut db_path = None;

        if workspaces_dir.exists() {
            if let Ok(entries) = std::fs::read_dir(&workspaces_dir) {
                for entry in entries.flatten() {
                    let entry_path = entry.path();
                    if entry_path.is_dir() {
                        let agents_base = entry_path.join("agents");
                        if let Ok(valid_agent_dir) =
                            crate::utils::security::validate_path(&agents_base, &_safe_agent_id)
                        {
                            let potential_path = valid_agent_dir.join("memory.lance");
                            if potential_path.exists() {
                                db_path = Some(potential_path.to_string_lossy().to_string());
                                break;
                            }
                        }
                    }
                }
            }
        }

        let path = db_path.ok_or_else(|| {
            AppError::NotFound(format!("Memory for agent {} not found", agent_id))
        })?;

        match VectorMemory::connect(&path, "memories").await {
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
                                    let id_col = batch
                                        .column_by_name("id")
                                        .and_then(|c| c.as_any().downcast_ref::<StringArray>());
                                    let text_col = batch
                                        .column_by_name("text")
                                        .and_then(|c| c.as_any().downcast_ref::<StringArray>());
                                    let mission_col = batch
                                        .column_by_name("mission_id")
                                        .and_then(|c| c.as_any().downcast_ref::<StringArray>());
                                    let ts_col = batch
                                        .column_by_name("timestamp")
                                        .and_then(|c| c.as_any().downcast_ref::<Int64Array>());

                                    if let (Some(ids), Some(texts), Some(missions), Some(tss)) =
                                        (id_col, text_col, mission_col, ts_col)
                                    {
                                        for i in 0..batch.num_rows() {
                                            entries.push(MemoryEntry {
                                                id: ids.value(i).to_string(),
                                                text: texts.value(i).to_string(),
                                                mission_id: missions.value(i).to_string(),
                                                timestamp: tss.value(i),
                                            });
                                        }
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
        // Security (Alert #28, #24): Sanitize agent_id and validate path to prevent traversal
        let _safe_agent_id = crate::utils::security::sanitize_id(&agent_id);
        let workspaces_dir = workspaces_root(&state.base_dir);
        let mut db_path = None;

        if workspaces_dir.exists() {
            if let Ok(entries) = std::fs::read_dir(&workspaces_dir) {
                for entry in entries.flatten() {
                    let entry_path = entry.path();
                    if entry_path.is_dir() {
                        let agents_base = entry_path.join("agents");
                        if let Ok(valid_agent_dir) =
                            crate::utils::security::validate_path(&agents_base, &_safe_agent_id)
                        {
                            let potential_path = valid_agent_dir.join("memory.lance");
                            if potential_path.exists() {
                                db_path = Some(potential_path.to_string_lossy().to_string());
                                break;
                            }
                        }
                    }
                }
            }
        }

        let path = db_path.ok_or_else(|| {
            AppError::NotFound(format!("Memory for agent {} not found", agent_id))
        })?;

        if let Ok(conn) = lancedb::connect(&format!("file://{}", path))
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
    let agents_base = workspaces_root(&state.base_dir)
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
        let workspaces_dir = workspaces_root(&state.base_dir);
        if workspaces_dir.exists() {
            if let Ok(workspace_entries) = std::fs::read_dir(&workspaces_dir) {
                for workspace in workspace_entries.flatten() {
                    let workspace_path = workspace.path();
                    let agents_dir = workspace_path.join("agents");

                    if agents_dir.exists() {
                        if let Ok(agent_entries) = std::fs::read_dir(&agents_dir) {
                            for agent in agent_entries.flatten() {
                                let agent_path = agent.path();
                                let db_path = agent_path.join("memory.lance");

                                if db_path.exists() && db_path.starts_with(&agents_dir) {
                                    if let Ok(mem) = VectorMemory::connect(
                                        &db_path.to_string_lossy(),
                                        "memories",
                                    )
                                    .await
                                    {
                                        if let Ok(mut hits) =
                                            mem.search_knowledge_full(query_vec.clone(), 5).await
                                        {
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
                        }
                    }
                }
            }
        }
    }

    // Sort by final score descending
    all_results.sort_by(|a, b| {
        let score_a = a.score.as_ref().map(|s| s.final_score).unwrap_or(0.0);
        let score_b = b.score.as_ref().map(|s| s.final_score).unwrap_or(0.0);
        score_b.partial_cmp(&score_a).unwrap_or(std::cmp::Ordering::Equal)
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
        let expected = base_dir.join("data").join("workspaces");
        assert_eq!(workspaces_root(&base_dir), expected);
    }
}

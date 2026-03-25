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
pub async fn get_agent_memory(Path(agent_id): Path<String>) -> Result<impl IntoResponse, AppError> {
    #[cfg(not(feature = "vector-memory"))]
    {
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
        let workspaces_dir = std::path::PathBuf::from("data/workspaces");
        let mut db_path = None;

        if workspaces_dir.exists() {
            if let Ok(entries) = std::fs::read_dir(&workspaces_dir) {
                for entry in entries.flatten() {
                    if entry.path().is_dir() {
                        let potential_path = entry
                            .path()
                            .join("agents")
                            .join(&agent_id)
                            .join("memory.lance");
                        if potential_path.exists() {
                            db_path = Some(potential_path.to_string_lossy().to_string());
                            break;
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
) -> Result<impl IntoResponse, AppError> {
    #[cfg(not(feature = "vector-memory"))]
    {
        return Ok((
            StatusCode::OK,
            Json(serde_json::json!({"status": "success"})),
        ));
    }

    #[cfg(feature = "vector-memory")]
    {
        let workspaces_dir = std::path::PathBuf::from("data/workspaces");
        let mut db_path = None;

        if workspaces_dir.exists() {
            if let Ok(entries) = std::fs::read_dir(&workspaces_dir) {
                for entry in entries.flatten() {
                    if entry.path().is_dir() {
                        let potential_path = entry
                            .path()
                            .join("agents")
                            .join(&agent_id)
                            .join("memory.lance");
                        if potential_path.exists() {
                            db_path = Some(potential_path.to_string_lossy().to_string());
                            break;
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
                table
                    .delete(&format!("id = '{}'", row_id))
                    .await
                    .map_err(|e| {
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
        &agent_ctx.resolve_provider_context(),
        (*state.resources.http_client).clone(),
    );
    let embedding = provider
        .embed(&payload.text)
        .await
        .map_err(|e| AppError::InternalServerError(format!("Embedding failed: {}", e)))?;

    // Sanitize agent_id for path safety
    let safe_agent_id = crate::utils::security::sanitize_id(&agent_id);

    // Find or create agent memory path
    let cluster_id = std::env::var("CLUSTER_ID").unwrap_or_else(|_| "default".to_string());
    let safe_cluster_id = crate::utils::security::sanitize_id(&cluster_id);

    let path = format!(
        "data/workspaces/{}/agents/{}/memory.lance",
        safe_cluster_id, safe_agent_id
    );
    let _ = tokio::fs::create_dir_all(format!(
        "data/workspaces/{}/agents/{}",
        safe_cluster_id, safe_agent_id
    ))
    .await;

    match VectorMemory::connect(&path, "memories").await {
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

#[derive(serde::Deserialize)]
pub struct SearchRequest {
    pub query: String,
    #[allow(dead_code)] // Deserialized for future agent-scoped search filtering
    pub agent_id: Option<String>,
}

/// GET /v1/search/memory?query=...
pub async fn global_search(
    State(state): State<Arc<AppState>>,
    axum::extract::Query(params): axum::extract::Query<SearchRequest>,
) -> Result<impl IntoResponse, AppError> {
    if params.query.is_empty() {
        return Ok((
            StatusCode::OK,
            Json(MemoryResponse {
                status: "success".to_string(),
                entries: vec![],
            }),
        ));
    }

    // Resolve a provider for embedding (use default or first agent)
    let runner = crate::agent::runner::AgentRunner::new(state.clone());
    let provider = if let Some(agent_ctx) = state.registry.agents.iter().next() {
        runner.resolve_provider(
            &agent_ctx.resolve_provider_context(),
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

    // Iterate across workspaces
    #[cfg(feature = "vector-memory")]
    {
        let workspaces_dir = std::path::PathBuf::from("data/workspaces");
        if workspaces_dir.exists() {
            if let Ok(workspace_entries) = std::fs::read_dir(&workspaces_dir) {
                for workspace in workspace_entries.flatten() {
                    let agents_dir = workspace.path().join("agents");
                    if agents_dir.exists() {
                        if let Ok(agent_entries) = std::fs::read_dir(&agents_dir) {
                            for agent in agent_entries.flatten() {
                                let db_path = agent.path().join("memory.lance");
                                if db_path.exists() {
                                    if let Ok(mem) = VectorMemory::connect(
                                        &db_path.to_string_lossy(),
                                        "memories",
                                    )
                                    .await
                                    {
                                        if let Ok(hits) =
                                            mem.search_knowledge(query_vec.clone(), 3).await
                                        {
                                            for hit in hits {
                                                all_results.push(MemoryEntry {
                                                    id: uuid::Uuid::new_v4().to_string(),
                                                    text: hit,
                                                    mission_id: "global-search".to_string(),
                                                    timestamp: chrono::Utc::now().timestamp(),
                                                });
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

    Ok((
        StatusCode::OK,
        Json(MemoryResponse {
            status: "success".to_string(),
            entries: all_results,
        }),
    ))
}

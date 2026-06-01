//! @docs ARCHITECTURE:Gateways
//! 
//! ### AI Assist Note
//! **! Intelligence Layer Routes — Code Graph & Blast Radius Analysis**
//! This module implements high-fidelity logic for the Sovereign Reality layer.
//! 
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: Runtime logic error, state desynchronization, or resource exhaustion.
//! - **Telemetry Link**: Search `[intelligence]` in tracing logs.

//!   Intelligence Layer Routes — Code Graph & Blast Radius Analysis
//!
//! @docs ARCHITECTURE:Intelligence
//!
//! ### AI Assist Note
//! **Intelligence Router**: Provides RESTful access to the system's 
//! semantic knowledge graph. Enables the frontend to visualize code 
//! interdependencies and perform real-time impact analysis (MOD-03).

use axum::{
    extract::{Query, State},
    Json,
};
use std::sync::Arc;
use serde::Deserialize;
use crate::state::AppState;
use crate::error::AppError;
use crate::intelligence::graph::{CodeSymbolGraph, SymbolNode};

use parking_lot::RwLock;

async fn get_built_symbol_graph(
    state: &Arc<AppState>,
) -> Result<Arc<RwLock<CodeSymbolGraph>>, AppError> {
    let graph_lock = state.resources.get_symbol_graph().await;

    let is_empty = {
        let guard = graph_lock.read();
        guard.index.is_empty()
    };

    if is_empty {
        let lock_clone = Arc::clone(&graph_lock);
        let salt = state.resources.obfuscation_salt.clone();
        tokio::task::spawn_blocking(move || {
            let mut graph = lock_clone.write();
            if graph.index.is_empty() {
                graph.build(&salt);
            }
        })
        .await
        .map_err(|e| AppError::InternalServerError(format!("Graph build thread panicked: {}", e)))?;
    }

    Ok(graph_lock)
}

#[derive(Deserialize)]
pub struct BlastRadiusQuery {
    pub name: String,
    pub path: String,
}

/// GET /v1/intelligence/graph
///
/// Returns the full high-fidelity symbol graph for visualization.
///
/// @docs API_REFERENCE:GetCodeGraph
pub async fn get_code_graph(
    State(state): State<Arc<AppState>>,
) -> Result<Json<serde_json::Value>, AppError> {
    let graph_lock = get_built_symbol_graph(&state).await?;

    let lock_clone = Arc::clone(&graph_lock);


    let (nodes, edges, anomalies) = tokio::task::spawn_blocking(move || {
        let guard = lock_clone.read();
        
        let mut nodes = Vec::new();
        let mut edges = Vec::new();

        for idx in guard.graph.node_indices() {
            if let Some(node) = guard.graph.node_weight(idx) {
                nodes.push(node.clone());
            }
        }

        use petgraph::visit::EdgeRef;
        for edge in guard.graph.edge_references() {
            let source = &guard.graph[edge.source()];
            let target = &guard.graph[edge.target()];
            edges.push(serde_json::json!({
                "source": format!("{}:{}", source.path, source.name),
                "target": format!("{}:{}", target.path, target.name),
            }));
        }

        let anomalies = guard.find_anomalies();

        (nodes, edges, anomalies)
    })
    .await
    .map_err(|e| AppError::InternalServerError(format!("Graph processing thread panicked: {}", e)))?;

    Ok(Json(serde_json::json!({
        "nodes": nodes,
        "links": edges,
        "anomalies": anomalies,
    })))
}

/// GET /v1/intelligence/blast-radius
///
/// Calculates the downstream impact of changing a specific symbol.
///
/// @docs API_REFERENCE:GetBlastRadius
pub async fn get_blast_radius(
    State(state): State<Arc<AppState>>,
    Query(query): Query<BlastRadiusQuery>,
) -> Result<Json<Vec<SymbolNode>>, AppError> {
    // 🛡️ [Path Traversal Hardening] Verify input resides within workspace boundary
    let workspace_root = &state.resources.base_dir;
    
    // Convert Windows backward slashes to forward slashes for unified traversal protection
    let normalized_path = query.path.replace('\\', "/");
    let combined = workspace_root.join(&normalized_path);
    
    let is_safe = if let Ok(canonical) = combined.canonicalize() {
        let ws_root_canonical = workspace_root.canonicalize().unwrap_or_else(|_| workspace_root.to_path_buf());
        canonical.starts_with(ws_root_canonical)
    } else {
        // Fallback boundary validation for raw query values
        !normalized_path.contains("..") && !normalized_path.starts_with('/') && !normalized_path.contains(':')
    };

    if !is_safe {
        return Err(AppError::BadRequest("Invalid path boundary: potential path traversal detected".to_string()));
    }

    let graph_lock = get_built_symbol_graph(&state).await?;
    let lock_clone = Arc::clone(&graph_lock);
    let query_path = query.path.clone();
    let query_name = query.name.clone();

    let affected = tokio::task::spawn_blocking(move || {
        let guard = lock_clone.read();
        
        // Reverse-resolve the physical raw path from the obfuscated path sent by the frontend client (O(1) lookup!)
        let raw_path = guard.obfuscated_to_real_path.get(&query_path).cloned().unwrap_or(query_path);

        guard.calculate_blast_radius(&query_name, &raw_path)
    })
    .await
    .map_err(|e| AppError::InternalServerError(format!("Blast radius processing thread panicked: {}", e)))?;

    Ok(Json(affected))
}


// Metadata: [intelligence]

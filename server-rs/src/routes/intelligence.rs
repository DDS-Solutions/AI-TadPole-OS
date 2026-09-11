//! @docs ARCHITECTURE:Gateways
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / HTTP Routes / intelligence
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Type-safe state handling and bounded execution without unhandled panics.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: `AppError::BadRequest`, `AppError::InternalServerError`
//! - **Telemetry Targets**: none declared
//! - **Witness Tests**: `intelligence::tests::test_path_boundary_safety`

use crate::error::AppError;
use crate::intelligence::graph::{CodeSymbolGraph, SymbolNode};
use crate::state::AppState;
use axum::{
    extract::{Query, State},
    Json,
};
use parking_lot::RwLock;
use serde::Deserialize;
use std::sync::Arc;
use tokio::sync::Mutex;

static BUILD_GUARD: Mutex<()> = Mutex::const_new(());
pub const DEFAULT_MAX_BLAST_DEPTH: usize = 50;

async fn get_built_symbol_graph(
    state: &Arc<AppState>,
) -> Result<Arc<RwLock<CodeSymbolGraph>>, AppError> {
    let graph_lock = state.resources.get_symbol_graph().await;

    let is_empty = {
        let guard = graph_lock.read();
        guard.index.is_empty()
    };

    if is_empty {
        let _guard = BUILD_GUARD.lock().await;

        let still_empty = {
            let guard = graph_lock.read();
            guard.index.is_empty()
        };

        if still_empty {
            let lock_clone = Arc::clone(&graph_lock);
            let salt = state.resources.obfuscation_salt.clone();
            tokio::task::spawn_blocking(move || -> Result<(), AppError> {
                let mut graph = lock_clone.write();
                if graph.index.is_empty() {
                    graph.build(&salt)?;
                }
                Ok(())
            })
            .await
            .map_err(|e| {
                AppError::InternalServerError(format!("Graph build worker failed: {}", e))
            })??;
        }
    }

    Ok(graph_lock)
}

#[derive(Debug, Deserialize)]
pub struct BlastRadiusQuery {
    pub name: String,
    pub path: String,
    pub max_depth: Option<usize>,
}

/// GET /v1/intelligence/graph
///
/// Returns the full high-fidelity symbol graph for visualization.
///
/// @docs API_REFERENCE:GetCodeGraph
#[tracing::instrument(skip(state), name = "intelligence::get_code_graph")]
pub async fn get_code_graph(
    State(state): State<Arc<AppState>>,
) -> Result<Json<serde_json::Value>, AppError> {
    let graph_lock = get_built_symbol_graph(&state).await?;
    let lock_clone = Arc::clone(&graph_lock);

    let (nodes, edges, anomalies) = tokio::task::spawn_blocking(move || {
        let guard = lock_clone.read();

        let mut nodes = Vec::with_capacity(guard.graph.node_count());
        let mut edges = Vec::with_capacity(guard.graph.edge_count());

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
    .map_err(|e| AppError::InternalServerError(format!("Graph processing worker failed: {}", e)))?;

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
#[tracing::instrument(skip(state), name = "intelligence::get_blast_radius")]
pub async fn get_blast_radius(
    State(state): State<Arc<AppState>>,
    Query(query): Query<BlastRadiusQuery>,
) -> Result<Json<Vec<SymbolNode>>, AppError> {
    // 🛡️ [Path Boundary Validation] Verify input does not contain traversal escapes
    let workspace_root = &state.resources.base_dir;
    let normalized_path = query.path.replace('\\', "/");
    let combined = workspace_root.join(&normalized_path);

    let is_safe = if let Ok(canonical) = combined.canonicalize() {
        let ws_root_canonical = workspace_root
            .canonicalize()
            .unwrap_or_else(|_| workspace_root.to_path_buf());
        canonical.starts_with(ws_root_canonical)
    } else {
        !normalized_path.contains("..")
            && !normalized_path.starts_with('/')
            && !normalized_path.contains(':')
    };

    if !is_safe {
        return Err(AppError::BadRequest(
            "Invalid path boundary: potential path traversal detected".to_string(),
        ));
    }

    let graph_lock = get_built_symbol_graph(&state).await?;
    let lock_clone = Arc::clone(&graph_lock);
    let query_path = query.path.clone();
    let query_name = query.name.clone();
    let depth = query
        .max_depth
        .unwrap_or(DEFAULT_MAX_BLAST_DEPTH)
        .clamp(1, 100);

    let affected = tokio::task::spawn_blocking(move || {
        let guard = lock_clone.read();

        // Check if query is obfuscated token or real path
        let resolved_path = guard
            .obfuscated_to_real_path
            .get(&query_path)
            .cloned()
            .unwrap_or_else(|| query_path.clone());

        guard.calculate_blast_radius(&query_name, &resolved_path, depth)
    })
    .await
    .map_err(|e| {
        AppError::InternalServerError(format!("Blast radius computation worker failed: {}", e))
    })?;

    Ok(Json(affected))
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_path_boundary_safety() {
        let temp = tempfile::tempdir().unwrap();
        let base = temp.path().to_path_buf();

        let malicious = "../../../etc/shadow";
        assert!(crate::utils::security::validate_path(&base, malicious).is_err());
    }
}

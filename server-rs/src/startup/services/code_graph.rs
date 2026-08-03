//! @docs ARCHITECTURE:Core
//!
//! ### AI Assist Note
//! **Core technical resource for the Tadpole OS Sovereign infrastructure.**
//! This module implements high-fidelity logic for the Sovereign Reality layer.
//!
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: Runtime logic error, state desynchronization, or resource exhaustion.
//! - **Telemetry Link**: Search `[code_graph]` in tracing logs.

use crate::startup::{SystemContext, SystemService};
use async_trait::async_trait;

pub struct CodeGraphWarmupService;

#[async_trait]
impl SystemService for CodeGraphWarmupService {
    fn name(&self) -> &'static str {
        "CodeGraphWarmup"
    }
    fn is_critical(&self) -> bool {
        true
    }
    fn registry_key(&self) -> &'static str {
        "CodeGraph"
    }
    async fn start(&self, context: SystemContext) -> Result<(), anyhow::Error> {
        let app_state = context.app_state;
        app_state
            .resources
            .set_subsystem_status("CodeGraph", crate::types::SubsystemStatus::Warming(0.1));

        // Lock-free Code Graph Warmup (REL-03 optimization)
        let graph_lock = app_state.resources.get_code_graph().await;
        let root = {
            let g = graph_lock.read();
            g.root.clone()
        };

        // Perform the scan on a temporary local instance without holding the write lock
        let mut temp_graph = crate::utils::graph::CodeGraph::new(root);
        temp_graph.scan();

        let module_count = temp_graph.modules.len();

        // Swap modules inside the write lock for a sub-millisecond lock duration
        {
            let mut graph = graph_lock.write();
            graph.modules = temp_graph.modules;
        }

        app_state
            .resources
            .set_subsystem_status("CodeGraph", crate::types::SubsystemStatus::Ready);
        tracing::info!(
            "[Hydra-RS] In-memory code graph warmed up ({} modules indexed)",
            module_count
        );
        Ok(())
    }
}

pub struct CodeGraphDbRefreshService;

#[async_trait]
impl SystemService for CodeGraphDbRefreshService {
    fn name(&self) -> &'static str {
        "CodeGraphDbRefresh"
    }
    fn is_critical(&self) -> bool {
        true
    }
    fn start_timeout(&self) -> std::time::Duration {
        std::time::Duration::from_secs(300)
    }
    async fn start(&self, context: SystemContext) -> Result<(), anyhow::Error> {
        let app_state = context.app_state;
        let started = std::time::Instant::now();
        app_state.resources.set_subsystem_status(
            "CodeGraphDbRefresh",
            crate::types::SubsystemStatus::Warming(0.05),
        );

        let root = app_state.resources.base_dir.clone();
        let db_path = root.join(".code-review-graph").join("graph.db");
        let salt = app_state.resources.obfuscation_salt.clone();

        let refresh_fut =
            crate::intelligence::graph_store::refresh_code_review_graph_db(root, db_path, salt);
        match tokio::time::timeout(std::time::Duration::from_secs(300), refresh_fut).await {
            Ok(Ok(summary)) => {
                app_state.resources.set_subsystem_status(
                    "CodeGraphDbRefresh",
                    crate::types::SubsystemStatus::Ready,
                );
                tracing::info!(
                    db_path = %summary.db_path.display(),
                    nodes = summary.node_count,
                    edges = summary.edge_count,
                    risks = summary.risk_count,
                    communities = summary.community_count,
                    flows = summary.flow_count,
                    elapsed_ms = started.elapsed().as_millis(),
                    "[CodeGraphDbRefresh] refreshed persistent code-review graph"
                );
                Ok(())
            }
            Ok(Err(err)) => {
                app_state.resources.set_subsystem_status(
                    "CodeGraphDbRefresh",
                    crate::types::SubsystemStatus::Failed(err.to_string()),
                );
                tracing::error!(
                    error = %err,
                    elapsed_ms = started.elapsed().as_millis(),
                    "[CodeGraphDbRefresh] failed to refresh persistent code-review graph"
                );
                Err(anyhow::anyhow!(err))
            }
            Err(_) => {
                let err_msg = "CodeGraphDbRefresh timed out after 300s".to_string();
                app_state.resources.set_subsystem_status(
                    "CodeGraphDbRefresh",
                    crate::types::SubsystemStatus::Failed(err_msg.clone()),
                );
                tracing::error!(
                    elapsed_ms = started.elapsed().as_millis(),
                    "[CodeGraphDbRefresh] {}",
                    err_msg
                );
                Err(anyhow::anyhow!(err_msg))
            }
        }
    }
}

// Metadata: [code_graph]

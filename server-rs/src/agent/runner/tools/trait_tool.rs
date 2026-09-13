//! @docs ARCHITECTURE:Runner
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Agent Runner / trait_tool
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Type-safe state handling and bounded execution without unhandled panics.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: none declared
//! - **Witness Tests**: none declared

use super::error::ToolExecutionError;
use crate::agent::types::TokenUsage;
use std::sync::Arc;

use serde_json::Value;

/// Data-driven tool definition for deduplication and discovery.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct ToolDefinitionData {
    pub name: String,
    pub description: String,
    pub parameters: Value,
    #[serde(default)]
    pub is_mutating: bool,
    #[serde(default)]
    pub is_dangerous: bool,
    #[serde(default)]
    pub is_cacheable: bool,
}

/// A lightweight, isolated view of the application state for tool execution.
/// Prevents tools from accessing unrelated runner internals.
pub struct ToolContext {
    pub mission_id: String,
    pub agent_id: String,
    pub workspace_root: std::path::PathBuf,
    pub fs_adapter: crate::adapter::filesystem::FilesystemAdapter,
    pub state: Arc<crate::state::AppState>,
}

#[async_trait::async_trait]
pub trait Tool: Send + Sync {
    /// Returns the metadata for this tool (name, description, parameters).
    fn metadata(&self) -> ToolDefinitionData;

    /// The unique identifier for the tool (e.g., "read_file").
    fn name(&self) -> &str;

    /// Checks if the tool outputs are cacheable.
    fn is_cacheable(&self) -> bool {
        false
    }

    /// Checks if the tool mutates state or files in the workspace.
    fn is_mutating(&self) -> bool {
        false
    }

    /// Checks if the tool performs dangerous operations (e.g. command execution, writes, external networks).
    fn is_dangerous(&self) -> bool {
        false
    }

    /// Executes the tool with the provided isolated context and arguments.
    async fn execute(
        &self,
        ctx: &ToolContext,
        args: Value,
        usage: &mut Option<TokenUsage>,
    ) -> Result<String, ToolExecutionError>;
}

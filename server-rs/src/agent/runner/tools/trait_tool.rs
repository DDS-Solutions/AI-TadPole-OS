//! @docs ARCHITECTURE:Runner
//! 
//! ### AI Assist Note
//! **Tool Trait Architecture**: Decouples tool execution from the `AgentRunner` 
//! to prevent "God Object" bloat. Implements **Isolated Contexts** to ensure 
//! tools only have access to the specific resources authorized for their scope.

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

    /// Executes the tool with the provided isolated context and arguments.
    async fn execute(
        &self,
        ctx: &ToolContext,
        args: Value,
        usage: &mut Option<TokenUsage>,
    ) -> Result<String, ToolExecutionError>;
}

// Metadata: [trait]

//! @docs ARCHITECTURE:Agent
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Agent Runner / registry
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Type-safe state handling and bounded execution without unhandled panics.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: none declared
//! - **Witness Tests**: none declared

use crate::agent::mcp::{McpResult, McpToolHub};
use async_trait::async_trait;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::warn;

#[async_trait]
pub trait ToolHandler: Send + Sync {
    /// Executes the tool with the given arguments.
    async fn execute(
        &self,
        args: Value,
        workspace_root: std::path::PathBuf,
    ) -> Result<McpResult, crate::error::AppError>;

    /// Returns the tool's registration metadata (name, schema, etc).
    fn metadata(&self) -> McpToolHub;
}

struct RegisteredTool {
    cached_metadata: McpToolHub,
    handler: Arc<dyn ToolHandler>,
}

pub struct McpRegistry {
    tools: HashMap<String, RegisteredTool>,
}

impl McpRegistry {
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
        }
    }

    /// Registers a tool handler, caching its metadata and warning if overwriting an existing tool
    pub fn register(&mut self, handler: Arc<dyn ToolHandler>) {
        let meta = handler.metadata();
        if self.tools.contains_key(&meta.name) {
            warn!(
                tool_name = %meta.name,
                "⚠️ [MCP] Duplicate tool registration '{}' - overriding previous handler",
                meta.name
            );
        }
        self.tools.insert(
            meta.name.clone(),
            RegisteredTool {
                cached_metadata: meta,
                handler,
            },
        );
    }

    /// Unregisters a tool handler by name
    pub fn unregister(&mut self, name: &str) -> Option<Arc<dyn ToolHandler>> {
        self.tools.remove(name).map(|r| r.handler)
    }

    pub fn get(&self, name: &str) -> Option<Arc<dyn ToolHandler>> {
        self.tools.get(name).map(|r| r.handler.clone())
    }

    pub fn list_all(&self) -> Vec<McpToolHub> {
        self.tools
            .values()
            .map(|r| r.cached_metadata.clone())
            .collect()
    }
}

impl Default for McpRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    struct DummyHandler {
        name: String,
    }

    #[async_trait]
    impl ToolHandler for DummyHandler {
        async fn execute(
            &self,
            _args: Value,
            _root: std::path::PathBuf,
        ) -> Result<McpResult, crate::error::AppError> {
            Ok(McpResult::Raw("dummy".to_string()))
        }
        fn metadata(&self) -> McpToolHub {
            McpToolHub {
                name: self.name.clone(),
                description: "Dummy tool".to_string(),
                input_schema: json!({}),
                source: "test".to_string(),
                stats: crate::agent::mcp::McpToolStats::default(),
                category: "test".to_string(),
            }
        }
    }

    #[test]
    fn test_registry_registration_and_retrieval() {
        let mut registry = McpRegistry::new();
        let h = Arc::new(DummyHandler {
            name: "test_tool".to_string(),
        });

        registry.register(h.clone());

        let retrieved = registry.get("test_tool").expect("Tool should be found");
        assert_eq!(retrieved.metadata().name, "test_tool");
    }

    #[test]
    fn test_registry_list_all_and_unregister() {
        let mut registry = McpRegistry::new();
        registry.register(Arc::new(DummyHandler {
            name: "tool1".to_string(),
        }));
        registry.register(Arc::new(DummyHandler {
            name: "tool2".to_string(),
        }));

        let list = registry.list_all();
        assert_eq!(list.len(), 2);
        assert!(list.iter().any(|t| t.name == "tool1"));
        assert!(list.iter().any(|t| t.name == "tool2"));

        let removed = registry.unregister("tool1");
        assert!(removed.is_some());
        assert!(registry.get("tool1").is_none());
        assert_eq!(registry.list_all().len(), 1);
    }
}

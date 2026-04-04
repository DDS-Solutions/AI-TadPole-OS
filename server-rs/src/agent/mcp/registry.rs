//! Modular Tool Registry — Discovery and dispatch for MCP tools
//!
//! Provides a standardized way to register and execute tools,
//! decoupling the schema definition from the underlying logic.
//!
//! @docs ARCHITECTURE:Agent

use crate::agent::mcp::{McpResult, McpToolHub};
use async_trait::async_trait;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;

#[async_trait]
pub trait ToolHandler: Send + Sync {
    /// Executes the tool with the given arguments.
    async fn execute(
        &self,
        args: Value,
        workspace_root: std::path::PathBuf,
    ) -> anyhow::Result<McpResult>;

    /// Returns the tool's registration metadata (name, schema, etc).
    fn metadata(&self) -> McpToolHub;
}

pub struct McpRegistry {
    tools: HashMap<String, Arc<dyn ToolHandler>>,
}

impl McpRegistry {
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
        }
    }

    pub fn register(&mut self, handler: Arc<dyn ToolHandler>) {
        let meta = handler.metadata();
        self.tools.insert(meta.name.clone(), handler);
    }

    pub fn get(&self, name: &str) -> Option<Arc<dyn ToolHandler>> {
        self.tools.get(name).cloned()
    }

    pub fn list_all(&self) -> Vec<McpToolHub> {
        self.tools.values().map(|h| h.metadata()).collect()
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
        ) -> anyhow::Result<McpResult> {
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
    fn test_registry_list_all() {
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
    }
}

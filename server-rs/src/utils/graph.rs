//! [RAG] Architectural graph analysis and codebase structure mapping.
//!
//! Scans the local filesystem to build a hierarchical map of modules,
//! extracting descriptions from READMEs for semantic tool discovery.
//!
//! @docs ARCHITECTURE:RAGSystems
//!
//! ### AI Assist Note
//! **Code Graph Utility**: Orchestrates the architectural analysis
//! and codebase structure mapping for the Tadpole OS engine.
//! Builds a hierarchical map of modules by scanning the local
//! filesystem and extracting semantic metadata from breadcrumbs and
//! READMEs. Features **Deterministic Scanning Boundaries**: `scan()`
//! is restricted to `max_depth(3)` and ignores infrastructure
//! directories (`target`, `node_modules`) to ensure sub-second
//! topological discovery for the RAG system (GRAPH-01).
//!
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: Filesystem permission issues during scan,
//!   malformed READMEs causing parsing failures, or depth-limited
//!   scans missing deep-nested mission-specific modules.
//! - **Trace Scope**: `server-rs::utils::graph`

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Debug, Clone, serde::Serialize)]
pub struct ModuleInfo {
    pub path: String,
    pub name: String,
    pub description: String,
    pub children: Vec<String>,
}

pub struct CodeGraph {
    pub modules: HashMap<String, ModuleInfo>,
    pub root: PathBuf,
}

impl CodeGraph {
    pub fn new(root: PathBuf) -> Self {
        Self {
            modules: HashMap::new(),
            root,
        }
    }

    /// Performs a recursive scan of the root directory up to a fixed depth.
    ///
    /// Identifies directories as modules, skips ignored paths (node_modules, target),
    /// and attempts to derive a meaningful description for each module found.
    pub fn scan(&mut self) {
        let mut modules = HashMap::new();

        for entry in WalkDir::new(&self.root)
            .max_depth(3)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if path.is_dir() {
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    if name.starts_with('.') || name == "node_modules" || name == "target" {
                        continue;
                    }

                    let rel_path = path
                        .strip_prefix(&self.root)
                        .unwrap_or(path)
                        .to_string_lossy()
                        .to_string()
                        .replace('\\', "/");
                    let description = self.extract_description(path);

                    modules.insert(
                        rel_path.clone(),
                        ModuleInfo {
                            path: rel_path,
                            name: name.to_string(),
                            description,
                            children: Vec::new(),
                        },
                    );
                }
            }
        }

        // Fill children
        let paths: Vec<String> = modules.keys().cloned().collect();
        for path_str in &paths {
            let path = Path::new(path_str);
            let children: Vec<String> = paths
                .iter()
                .filter(|p_str| {
                    let p = Path::new(p_str);
                    p.parent() == Some(path)
                })
                .cloned()
                .collect();

            if let Some(m) = modules.get_mut(path_str) {
                m.children = children;
            }
        }

        self.modules = modules;
    }

    fn extract_description(&self, path: &Path) -> String {
        // Look for README.md or mod.rs/mod.ts comments
        let readme = path.join("README.md");
        if readme.exists() {
            if let Ok(content) = std::fs::read_to_string(readme) {
                return content
                    .lines()
                    .next()
                    .unwrap_or("")
                    .replace("#", "")
                    .trim()
                    .to_string();
            }
        }

        "".to_string()
    }

    pub fn generate_summary(&self) -> String {
        let mut summary = String::from("### Project Architecture Map\n\n");
        let mut sorted_paths: Vec<_> = self.modules.keys().collect();
        sorted_paths.sort();

        for path in sorted_paths {
            if path.is_empty() {
                continue;
            }
            let m = &self.modules[path];
            let indent = "  ".repeat(path.chars().filter(|&c| c == '/').count());
            summary.push_str(&format!("{}- **{}**: {}\n", indent, m.name, m.description));
        }

        summary
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_code_graph_scan() {
        let dir = tempdir().unwrap();
        let root = dir.path();

        // Create dummy structure
        fs::create_dir_all(root.join("src/agent")).unwrap();
        fs::create_dir_all(root.join("src/utils")).unwrap();
        fs::write(
            root.join("src/agent/README.md"),
            "# Agent Module\nDescription here",
        )
        .unwrap();

        let mut graph = CodeGraph::new(root.to_path_buf());
        graph.scan();

        assert!(graph.modules.contains_key("src"));
        assert!(graph.modules.contains_key("src/agent"));
        assert!(graph.modules.contains_key("src/utils"));

        let agent_info = graph.modules.get("src/agent").unwrap();
        assert_eq!(agent_info.name, "agent");
        assert_eq!(agent_info.description, "Agent Module");

        let src_info = graph.modules.get("src").unwrap();
        assert!(src_info.children.contains(&"src/agent".to_string()));
        assert!(src_info.children.contains(&"src/utils".to_string()));
    }

    #[test]
    fn test_generate_summary() {
        let mut modules = HashMap::new();
        modules.insert(
            "src".to_string(),
            ModuleInfo {
                path: "src".to_string(),
                name: "src".to_string(),
                description: "Root source".to_string(),
                children: vec!["src/agent".to_string()],
            },
        );
        modules.insert(
            "src/agent".to_string(),
            ModuleInfo {
                path: "src/agent".to_string(),
                name: "agent".to_string(),
                description: "Agent logic".to_string(),
                children: vec![],
            },
        );

        let graph = CodeGraph {
            modules,
            root: PathBuf::from("/"),
        };

        let summary = graph.generate_summary();
        assert!(summary.contains("- **src**: Root source"));
        assert!(summary.contains("  - **agent**: Agent logic"));
    }
}

// Metadata: [graph]

// Metadata: [graph]

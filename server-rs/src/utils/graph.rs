//! @docs ARCHITECTURE:RAGSystems
//!
//! ### AI Context Alignment
//! - **Subsystem**: Frontend Utilities / graph
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Type-safe state handling and bounded execution without unhandled panics.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: none declared
//! - **Witness Tests**: none declared

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleInfo {
    pub path: String,
    pub name: String,
    pub description: String,
    pub children: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphSummary {
    pub text: String,
    pub hot_paths: Vec<String>,
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

        for entry_res in WalkDir::new(&self.root).max_depth(3) {
            let entry = match entry_res {
                Ok(e) => e,
                Err(e) => {
                    tracing::warn!("Failed to read directory during graph scan: {}", e);
                    continue;
                }
            };

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

                    if rel_path.is_empty() {
                        continue;
                    }

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
                    .trim_start_matches('#')
                    .trim()
                    .to_string();
            }
        }

        "".to_string()
    }

    pub fn generate_summary(&self) -> GraphSummary {
        let mut text = String::from("### Project Architecture Map\n\n");
        let mut sorted_paths: Vec<_> = self.modules.keys().collect();
        sorted_paths.sort();

        let mut hot_paths = Vec::new();

        for path in sorted_paths {
            if path.is_empty() {
                continue;
            }
            let m = &self.modules[path];
            let indent = "  ".repeat(path.chars().filter(|&c| c == '/').count());
            text.push_str(&format!("{}- **{}**: {}\n", indent, m.name, m.description));

            // Heuristic: modules with descriptions or many children are "hot"
            if !m.description.is_empty() || m.children.len() > 2 {
                hot_paths.push(path.clone());
            }
        }

        GraphSummary { text, hot_paths }
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
        fs::create_dir_all(root.join("src/csharp")).unwrap();
        fs::create_dir_all(root.join("node_modules/pkg")).unwrap();
        fs::create_dir_all(root.join(".hidden/stuff")).unwrap();
        fs::write(
            root.join("src/agent/README.md"),
            "# Agent Module\nDescription here",
        )
        .unwrap();
        fs::write(
            root.join("src/csharp/README.md"),
            "## C# Tooling Architecture\nDetails here",
        )
        .unwrap();

        let mut graph = CodeGraph::new(root.to_path_buf());
        graph.scan();

        assert!(!graph.modules.contains_key(""));
        assert!(graph.modules.contains_key("src"));
        assert!(graph.modules.contains_key("src/agent"));
        assert!(graph.modules.contains_key("src/utils"));
        assert!(graph.modules.contains_key("src/csharp"));
        assert!(!graph.modules.contains_key("node_modules"));
        assert!(!graph.modules.contains_key(".hidden"));

        let agent_info = graph.modules.get("src/agent").unwrap();
        assert_eq!(agent_info.name, "agent");
        assert_eq!(agent_info.description, "Agent Module");

        let csharp_info = graph.modules.get("src/csharp").unwrap();
        assert_eq!(csharp_info.description, "C# Tooling Architecture");

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
        assert!(summary.text.contains("- **src**: Root source"));
        assert!(summary.text.contains("  - **agent**: Agent logic"));
        assert!(summary.hot_paths.contains(&"src".to_string()));
    }
}

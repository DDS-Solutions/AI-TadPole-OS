//! @docs ARCHITECTURE:Core:Intelligence
//!
//! ### AI Assist Note
//! **Markdown Memory Graph Engine**: Parses Markdown documents (`.agent/memory/`, `directives/`, `docs/`)
//! into a Multi-Parent Directed Acyclic Graph (DAG) using `petgraph`.
//! Differentiates **Inclusion Links** (standalone line links `[Child](child.md)` forming parent->child hierarchy)
//! from **Cross-References** (inline links and `[[WikiLinks]]`).
//! Features single-pass shared file scanning across multiple root directories and multi-path breadcrumb tracking.
//!
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: Circular inclusion references, unreadable files, or missing relative target files.
//! - **Telemetry Link**: Search `[markdown_graph]` in tracing logs.

use petgraph::graph::{DiGraph, NodeIndex};
use petgraph::visit::EdgeRef;
use petgraph::Direction;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LinkType {
    /// Standalone line link: parent includes child structurally
    Inclusion,
    /// Inline link or cross-reference within narrative text
    CrossReference,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarkdownNode {
    pub id: String,
    pub title: String,
    pub path: PathBuf,
    pub relative_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkEdge {
    pub link_type: LinkType,
    pub target_href: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphExport {
    pub nodes: Vec<MarkdownNode>,
    pub edges: Vec<GraphExportEdge>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphExportEdge {
    pub source: String,
    pub target: String,
    pub link_type: LinkType,
}

/// Shared struct for single-pass file parsing across Graph and BM25 search
#[derive(Debug, Clone)]
pub struct ParsedFileData {
    pub path: PathBuf,
    pub normalized_key: String,
    pub relative_path: String,
    pub title: String,
    pub content: String,
    pub links: Vec<(String, bool)>, // (href, is_standalone)
}

pub struct MarkdownMemoryGraph {
    graph: DiGraph<MarkdownNode, LinkEdge>,
    key_to_node: HashMap<String, NodeIndex>,
}

impl Default for MarkdownMemoryGraph {
    fn default() -> Self {
        Self::new()
    }
}

impl MarkdownMemoryGraph {
    pub fn new() -> Self {
        Self {
            graph: DiGraph::new(),
            key_to_node: HashMap::new(),
        }
    }

    /// Reads all Markdown files across multiple root directories in a SINGLE disk pass.
    pub fn parse_root_directories<P: AsRef<Path>>(root_dirs: &[P]) -> Vec<ParsedFileData> {
        let mut parsed_files = Vec::new();

        for root_ref in root_dirs {
            let root = root_ref.as_ref();
            if !root.exists() {
                continue;
            }

            let walker = ignore::WalkBuilder::new(root)
                .hidden(false)
                .git_ignore(true)
                .build();

            for entry in walker.flatten() {
                let path = entry.path();
                if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("md") {
                    if let Ok(content) = std::fs::read_to_string(path) {
                        let rel_path = path
                            .strip_prefix(root)
                            .unwrap_or(path)
                            .to_string_lossy()
                            .replace('\\', "/");

                        let file_stem = path
                            .file_stem()
                            .and_then(|s| s.to_str())
                            .unwrap_or("file");

                        let title = extract_title_from_content(&content, file_stem);
                        let links = extract_links(&content);
                        let normalized_key = normalize_path_str(path);

                        parsed_files.push(ParsedFileData {
                            path: path.to_path_buf(),
                            normalized_key,
                            relative_path: rel_path,
                            title,
                            content,
                            links,
                        });
                    }
                }
            }
        }

        parsed_files
    }

    /// Constructs the graph from pre-parsed file data (Zero additional I/O).
    pub fn build_from_parsed_files(parsed_files: &[ParsedFileData]) -> Self {
        let mut builder = Self::new();

        // Pass 1: Add nodes
        for file_data in parsed_files {
            let node_id = file_data.relative_path.clone();
            let node = MarkdownNode {
                id: node_id.clone(),
                title: file_data.title.clone(),
                path: file_data.path.clone(),
                relative_path: file_data.relative_path.clone(),
            };

            let idx = builder.graph.add_node(node);
            builder.key_to_node.insert(file_data.normalized_key.clone(), idx);
            builder.key_to_node.insert(file_data.relative_path.clone(), idx);
        }

        // Pass 2: Add edges
        for file_data in parsed_files {
            let source_idx = match builder.key_to_node.get(&file_data.normalized_key).copied() {
                Some(idx) => idx,
                None => continue,
            };

            let parent_dir = file_data.path.parent().unwrap_or(&file_data.path);

            for (href, is_standalone) in &file_data.links {
                let link_type = if *is_standalone {
                    LinkType::Inclusion
                } else {
                    LinkType::CrossReference
                };

                let resolved_path = parent_dir.join(href);
                let target_key = normalize_path_str(&resolved_path);

                let target_idx = builder
                    .key_to_node
                    .get(&target_key)
                    .or_else(|| builder.key_to_node.get(href.trim_start_matches("./")))
                    .copied();

                if let Some(t_idx) = target_idx {
                    builder.graph.add_edge(source_idx, t_idx, LinkEdge {
                        link_type,
                        target_href: href.clone(),
                    });
                }
            }
        }

        builder
    }

    /// Convenience builder from a single directory.
    pub fn build_from_directory<P: AsRef<Path>>(root_dir: P) -> std::io::Result<Self> {
        let parsed = Self::parse_root_directories(&[root_dir]);
        Ok(Self::build_from_parsed_files(&parsed))
    }

    /// Convenience builder from multiple directories.
    #[allow(dead_code)]
    pub fn build_from_directories<P: AsRef<Path>>(root_dirs: &[P]) -> Self {
        let parsed = Self::parse_root_directories(root_dirs);
        Self::build_from_parsed_files(&parsed)
    }

    /// Gets ancestor paths as structured lists of titles (`Vec<Vec<String>>`).
    pub fn get_ancestor_paths<P: AsRef<Path>>(&self, target_path: P) -> Vec<Vec<String>> {
        let key = normalize_path_str(target_path.as_ref());
        let target_idx = match self.key_to_node.get(&key).copied() {
            Some(idx) => idx,
            None => return Vec::new(),
        };

        let mut visited = HashSet::new();
        let mut current_path = Vec::new();
        let mut all_paths = Vec::new();

        self.traverse_ancestors(target_idx, &mut visited, &mut current_path, &mut all_paths);

        all_paths
            .into_iter()
            .map(|chain| {
                chain
                    .into_iter()
                    .map(|idx| self.graph[idx].title.clone())
                    .collect()
            })
            .collect()
    }

    /// Helper that formats ancestor paths into readable string breadcrumbs.
    pub fn get_ancestor_breadcrumbs<P: AsRef<Path>>(&self, target_path: P) -> Vec<String> {
        self.get_ancestor_paths(target_path)
            .into_iter()
            .map(|path| path.join(" > "))
            .collect()
    }

    fn traverse_ancestors(
        &self,
        current: NodeIndex,
        visited: &mut HashSet<NodeIndex>,
        current_path: &mut Vec<NodeIndex>,
        all_paths: &mut Vec<Vec<NodeIndex>>,
    ) {
        current_path.push(current);
        visited.insert(current);

        let mut parents = Vec::new();
        for edge in self.graph.edges_directed(current, Direction::Incoming) {
            if edge.weight().link_type == LinkType::Inclusion {
                parents.push(edge.source());
            }
        }

        if parents.is_empty() {
            let mut reversed = current_path.clone();
            reversed.reverse();
            all_paths.push(reversed);
        } else {
            for parent in parents {
                if !visited.contains(&parent) {
                    self.traverse_ancestors(parent, visited, current_path, all_paths);
                }
            }
        }

        current_path.pop();
        visited.remove(&current);
    }

    /// Exports graph nodes and edges for JSON API serialization (e.g. Neural Map UI).
    pub fn export_graph(&self) -> GraphExport {
        let mut nodes = Vec::new();
        for idx in self.graph.node_indices() {
            nodes.push(self.graph[idx].clone());
        }

        let mut edges = Vec::new();
        for edge in self.graph.edge_indices() {
            let (source_idx, target_idx) = self.graph.edge_endpoints(edge).unwrap();
            let weight = &self.graph[edge];
            edges.push(GraphExportEdge {
                source: self.graph[source_idx].id.clone(),
                target: self.graph[target_idx].id.clone(),
                link_type: weight.link_type.clone(),
            });
        }

        GraphExport { nodes, edges }
    }
}

/// Cross-platform path normalization string.
fn normalize_path_str<P: AsRef<Path>>(path: P) -> String {
    path.as_ref()
        .to_string_lossy()
        .trim()
        .replace('\\', "/")
        .to_lowercase()
}

/// Single-pass title extraction from in-memory content.
fn extract_title_from_content(content: &str, file_stem: &str) -> String {
    for line in content.lines() {
        let trimmed = line.trim();
        if let Some(heading) = trimmed.strip_prefix("# ") {
            return heading.trim().to_string();
        }
    }
    file_stem.to_string()
}

/// Robust link extraction supporting standard Markdown `[Title](target.md)` and `[[WikiLinks]]`.
fn extract_links(content: &str) -> Vec<(String, bool)> {
    let mut results = Vec::new();

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        let is_standalone = is_standalone_link_line(trimmed);

        // 1. Check for WikiLinks `[[WikiLinkTarget]]` or `[[Target|Title]]`
        let mut offset = 0;
        while let Some(start_wiki) = trimmed[offset..].find("[[") {
            let actual_start = offset + start_wiki;
            if let Some(end_wiki) = trimmed[actual_start + 2..].find("]]") {
                let wiki_content = &trimmed[actual_start + 2..actual_start + 2 + end_wiki];
                let target = wiki_content.split('|').next().unwrap_or(wiki_content).trim();
                let href = if target.ends_with(".md") {
                    target.to_string()
                } else {
                    format!("{}.md", target)
                };
                results.push((href, is_standalone));
                offset = actual_start + 2 + end_wiki + 2;
                continue;
            }
            offset = actual_start + 2;
        }

        // 2. Check for standard Markdown links `[Title](target.md)`
        offset = 0;
        while let Some(start_bracket) = trimmed[offset..].find('[') {
            let actual_start = offset + start_bracket;
            if actual_start > 0 && &trimmed[actual_start - 1..actual_start] == "[" {
                offset = actual_start + 1;
                continue;
            }
            if let Some(close_bracket) = trimmed[actual_start..].find(']') {
                let actual_close = actual_start + close_bracket;
                if trimmed[actual_close..].starts_with("](") {
                    if let Some(close_paren) = trimmed[actual_close + 2..].find(')') {
                        let raw_href = &trimmed[actual_close + 2..actual_close + 2 + close_paren];
                        let clean_href = raw_href.trim();
                        if is_valid_relative_target(clean_href) {
                            results.push((clean_href.to_string(), is_standalone));
                        }
                        offset = actual_close + 2 + close_paren + 1;
                        continue;
                    }
                }
            }
            offset = actual_start + 1;
        }
    }

    results
}

fn is_standalone_link_line(line: &str) -> bool {
    let s = line
        .strip_prefix('-')
        .or_else(|| line.strip_prefix('*'))
        .unwrap_or(line)
        .trim();

    let trimmed_end = s.trim_end();

    (trimmed_end.starts_with('[') && trimmed_end.ends_with(')'))
        || (trimmed_end.starts_with("[[") && trimmed_end.ends_with("]]"))
}

fn is_valid_relative_target(href: &str) -> bool {
    !href.starts_with("http://")
        && !href.starts_with("https://")
        && !href.starts_with('#')
        && !href.is_empty()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_links_robust() {
        let content = r#"
# Memory Index

- [User Preferences](user-preferences.md )   
- [[Project Conventions]]

See [Inline Note](inline.md) for extra details.
"#;
        let links = extract_links(content);
        assert_eq!(links.len(), 3);
        assert_eq!(links[0], ("user-preferences.md".to_string(), true));
        assert_eq!(links[1], ("Project Conventions.md".to_string(), true));
        assert_eq!(links[2], ("inline.md".to_string(), false));
    }
}

// Metadata: [markdown_graph]

//! @docs ARCHITECTURE:Services:Memory
//!
//! ### AI Assist Note
//! **Zero-Embedding BM25 Lexical Search Engine**: Indexes `.agent/memory/`, `directives/`, and `docs/`
//! in pure Rust using `bm25` and pre-calculated term frequencies.
//! Features **Single-Pass Shared Disk I/O ($O(N)$)**, **Pre-Calculated Term Frequencies ($O(1)$ query allocations)**,
//! **Unified Multi-Directory Graph Merging**, **Optimized Snippet Slicing**, and **Thundering-Herd Lock Protection**.
//!
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: Disk read errors, malformed encoding, or cache eviction races.
//! - **Telemetry Link**: Search `[bm25_memory]` in tracing logs.

use crate::intelligence::markdown_graph::{MarkdownMemoryGraph, ParsedFileData};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bm25SearchResult {
    pub file_path: String,
    pub relative_path: String,
    pub title: String,
    pub score: f32,
    pub snippet: String,
    pub breadcrumbs: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct IndexedDocument {
    #[allow(dead_code)]
    pub id: String,
    pub title: String,
    pub path: PathBuf,
    pub relative_path: String,
    pub content: String,
    pub term_count: usize,
    /// Pre-calculated term frequencies to eliminate inner query loop HashMap allocations ($O(1)$)
    pub term_frequencies: HashMap<String, usize>,
}

struct CacheEntry {
    index: Arc<Bm25MemoryIndex>,
    timestamp: Instant,
}

pub struct Bm25MemoryEngine {
    root_dirs: Vec<PathBuf>,
    cache: RwLock<Option<CacheEntry>>,
    ttl: Duration,
}

impl Bm25MemoryEngine {
    pub fn new(root_dirs: Vec<PathBuf>) -> Self {
        Self {
            root_dirs,
            cache: RwLock::new(None),
            ttl: Duration::from_secs(5), // 5-second TTL cache as confirmed by user
        }
    }

    /// Performs BM25 search over indexed Markdown files, returning top-k ranked results with breadcrumbs.
    pub fn search(&self, query: &str, top_k: usize) -> Vec<Bm25SearchResult> {
        let index = self.get_or_build_index();
        index.query(query, top_k)
    }

    /// Double-checked locking to prevent thundering herd rebuild races
    fn get_or_build_index(&self) -> Arc<Bm25MemoryIndex> {
        // First check under read lock
        if let Ok(read_guard) = self.cache.read() {
            if let Some(entry) = read_guard.as_ref() {
                if entry.timestamp.elapsed() < self.ttl {
                    return entry.index.clone();
                }
            }
        }

        // Acquire write lock (blocks secondary threads to prevent thundering herd)
        let mut write_guard = match self.cache.write() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };

        // Second check inside write lock
        if let Some(entry) = write_guard.as_ref() {
            if entry.timestamp.elapsed() < self.ttl {
                return entry.index.clone();
            }
        }

        // Rebuild index in a single pass over disk I/O
        let new_index = Arc::new(Bm25MemoryIndex::build_from_root_directories(
            &self.root_dirs,
        ));

        *write_guard = Some(CacheEntry {
            index: new_index.clone(),
            timestamp: Instant::now(),
        });

        new_index
    }
}

pub struct Bm25MemoryIndex {
    documents: Vec<IndexedDocument>,
    graph: MarkdownMemoryGraph,
    avg_doc_length: f32,
    doc_freqs: HashMap<String, usize>,
}

impl Bm25MemoryIndex {
    /// Builds both Graph and BM25 index in a SINGLE DISK READ PASS ($O(N)$ total I/O).
    pub fn build_from_root_directories(root_dirs: &[PathBuf]) -> Self {
        // Single disk pass across all root directories
        let parsed_files = MarkdownMemoryGraph::parse_root_directories(root_dirs);

        // Build unified multi-directory memory graph
        let graph = MarkdownMemoryGraph::build_from_parsed_files(&parsed_files);

        // Index documents with PRE-CALCULATED term frequencies
        Self::build_from_parsed_files(parsed_files, graph)
    }

    pub fn build_from_parsed_files(
        parsed_files: Vec<ParsedFileData>,
        graph: MarkdownMemoryGraph,
    ) -> Self {
        let mut documents = Vec::with_capacity(parsed_files.len());
        let mut total_length = 0;
        let mut doc_freqs: HashMap<String, usize> = HashMap::new();

        for file_data in parsed_files {
            let terms = tokenize(&file_data.content);
            let term_count = terms.len();
            total_length += term_count;

            // Pre-calculate term frequencies per document
            let mut term_frequencies: HashMap<String, usize> = HashMap::new();
            for term in terms {
                *term_frequencies.entry(term).or_insert(0) += 1;
            }

            // Track global document frequency per term for IDF
            for term in term_frequencies.keys() {
                *doc_freqs.entry(term.clone()).or_insert(0) += 1;
            }

            documents.push(IndexedDocument {
                id: file_data.relative_path.clone(),
                title: file_data.title,
                path: file_data.path,
                relative_path: file_data.relative_path,
                content: file_data.content,
                term_count,
                term_frequencies,
            });
        }

        let avg_doc_length = if !documents.is_empty() {
            total_length as f32 / documents.len() as f32
        } else {
            1.0
        };

        Self {
            documents,
            graph,
            avg_doc_length,
            doc_freqs,
        }
    }

    pub fn query(&self, query_str: &str, top_k: usize) -> Vec<Bm25SearchResult> {
        let query_terms = tokenize(query_str);
        if query_terms.is_empty() || self.documents.is_empty() {
            return Vec::new();
        }

        const K1: f32 = 1.2;
        const B: f32 = 0.75;
        let num_docs = self.documents.len() as f32;

        let mut scored_results: Vec<(f32, &IndexedDocument)> = self
            .documents
            .iter()
            .map(|doc| {
                let doc_len = doc.term_count as f32;
                let mut score = 0.0f32;

                for q_term in &query_terms {
                    // ZERO ALLOCATION LOOKUP: Pre-calculated term frequency
                    if let Some(&tf) = doc.term_frequencies.get(q_term) {
                        let df = *self.doc_freqs.get(q_term).unwrap_or(&1) as f32;
                        let idf = ((num_docs - df + 0.5) / (df + 0.5) + 1.0).ln();
                        let tf_f32 = tf as f32;

                        let numerator = tf_f32 * (K1 + 1.0);
                        let denominator =
                            tf_f32 + K1 * (1.0 - B + B * (doc_len / self.avg_doc_length));

                        score += idf * (numerator / denominator);
                    }
                }

                (score, doc)
            })
            .filter(|(score, _)| *score > 0.01)
            .collect();

        // Sort descending by BM25 score
        scored_results.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
        scored_results.truncate(top_k);

        scored_results
            .into_iter()
            .map(|(score, doc)| {
                let breadcrumbs = self.graph.get_ancestor_breadcrumbs(&doc.path);
                let snippet = extract_snippet(&doc.content, &query_terms);

                Bm25SearchResult {
                    file_path: doc.path.to_string_lossy().to_string(),
                    relative_path: doc.relative_path.clone(),
                    title: doc.title.clone(),
                    score,
                    snippet,
                    breadcrumbs,
                }
            })
            .collect()
    }
}

fn tokenize(text: &str) -> Vec<String> {
    text.to_lowercase()
        .split(|c: char| !c.is_alphanumeric())
        .filter(|s| s.len() > 1)
        .map(|s| s.to_string())
        .collect()
}

/// Fast snippet extraction using string index slicing on in-memory content
fn extract_snippet(content: &str, query_terms: &[String]) -> String {
    let content_lower = content.to_lowercase();
    for q in query_terms {
        if let Some(idx) = content_lower.find(q) {
            let start = idx.saturating_sub(40);
            let end = (idx + q.len() + 160).min(content.len());

            // Adjust to valid UTF-8 character boundaries
            let mut valid_start = start;
            while !content.is_char_boundary(valid_start) && valid_start > 0 {
                valid_start -= 1;
            }
            let mut valid_end = end;
            while !content.is_char_boundary(valid_end) && valid_end < content.len() {
                valid_end += 1;
            }

            let slice = &content[valid_start..valid_end].trim();
            return format!("...{}...", slice.replace('\n', " "));
        }
    }

    content.lines().take(2).collect::<Vec<_>>().join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn test_bm25_search() {
        let dir = tempdir().unwrap();
        let file1_path = dir.path().join("doc1.md");
        let file2_path = dir.path().join("doc2.md");

        let mut f1 = File::create(&file1_path).unwrap();
        writeln!(
            f1,
            "# Tailwind Rules\nUse Tailwind CSS v4 container queries."
        )
        .unwrap();

        let mut f2 = File::create(&file2_path).unwrap();
        writeln!(
            f2,
            "# Rust Performance\nRust Tokio axum server optimization."
        )
        .unwrap();

        let engine = Bm25MemoryEngine::new(vec![dir.path().to_path_buf()]);
        let results = engine.search("tailwind", 5);

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].title, "Tailwind Rules");
        assert!(results[0].snippet.contains("Tailwind"));
    }
}

// Metadata: [bm25_memory]

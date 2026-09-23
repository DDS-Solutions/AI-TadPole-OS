//! @docs ARCHITECTURE:Services:Memory
//!
//! ### AI Context Alignment
//! - **Subsystem**: Frontend Service Layer / bm25_memory
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Asynchronous service calls normalize response envelopes and propagate typed errors.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: none declared
//! - **Witness Tests**: none declared

use crate::intelligence::markdown_graph::{MarkdownMemoryGraph, ParsedFileData};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::{Arc, Mutex, RwLock};
use std::time::{Duration, Instant, SystemTime};

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
    pub title: String,
    pub path: PathBuf,
    pub relative_path: String,
    pub content: String,
    pub term_count: usize,
    /// Pre-calculated term frequencies to accelerate query evaluation
    pub term_frequencies: HashMap<String, usize>,
}

struct CacheEntry {
    index: Arc<Bm25MemoryIndex>,
    timestamp: Instant,
    last_mtime: SystemTime,
}

/// Recursively inspects all files and directories across root paths to find the most recent modification time.
/// Note: Inspecting directory mtimes is mandatory to detect file creations, deletions, and renames.
fn get_latest_file_mtime(dirs: &[PathBuf]) -> SystemTime {
    let mut latest = SystemTime::UNIX_EPOCH;
    for root in dirs {
        if !root.exists() {
            continue;
        }
        for entry in walkdir::WalkDir::new(root)
            .follow_links(false)
            .into_iter()
            .flatten()
        {
            if let Ok(meta) = entry.metadata() {
                if let Ok(modified) = meta.modified() {
                    if modified > latest {
                        latest = modified;
                    }
                }
            }
        }
    }
    latest
}

pub struct Bm25MemoryEngine {
    root_dirs: Vec<PathBuf>,
    cache: RwLock<Option<CacheEntry>>,
    build_lock: Mutex<()>,
    ttl: Duration,
}

impl Bm25MemoryEngine {
    pub fn new(root_dirs: Vec<PathBuf>) -> Self {
        Self {
            root_dirs,
            cache: RwLock::new(None),
            build_lock: Mutex::new(()),
            ttl: Duration::from_secs(30),
        }
    }

    pub fn with_ttl(mut self, ttl: Duration) -> Self {
        self.ttl = ttl;
        self
    }

    /// Performs BM25 search over indexed Markdown files, returning top-k ranked results with breadcrumbs.
    pub fn search(&self, query: &str, top_k: usize) -> Vec<Bm25SearchResult> {
        let index = self.get_or_build_index();
        index.query(query, top_k)
    }

    /// Serves cached index or serializes rebuilds with double-checked locking to eliminate thundering herds.
    fn get_or_build_index(&self) -> Arc<Bm25MemoryIndex> {
        // Fast path 1: check under read lock for valid TTL
        {
            let read_guard = self.cache.read().unwrap_or_else(|p| p.into_inner());
            if let Some(entry) = read_guard.as_ref() {
                if entry.timestamp.elapsed() < self.ttl {
                    return entry.index.clone();
                }
            }
        }

        // Fast path 2: Check recursive file & directory mtimes
        let current_mtime = get_latest_file_mtime(&self.root_dirs);

        {
            let mut write_guard = self.cache.write().unwrap_or_else(|p| p.into_inner());
            if let Some(entry) = write_guard.as_mut() {
                if entry.timestamp.elapsed() < self.ttl {
                    return entry.index.clone();
                }
                if current_mtime <= entry.last_mtime {
                    entry.timestamp = Instant::now();
                    return entry.index.clone();
                }
            }
        }

        // Acquire build lock to serialize rebuilds (Anti-Thundering Herd)
        let _build_guard = self.build_lock.lock().unwrap_or_else(|p| p.into_inner());

        // Double-check cache under build_lock in case another thread just completed the rebuild
        {
            let read_guard = self.cache.read().unwrap_or_else(|p| p.into_inner());
            if let Some(entry) = read_guard.as_ref() {
                if entry.timestamp.elapsed() < self.ttl || current_mtime <= entry.last_mtime {
                    return entry.index.clone();
                }
            }
        }

        // Rebuild index from disk
        let start = Instant::now();
        let fresh_mtime = get_latest_file_mtime(&self.root_dirs);
        let new_index = Arc::new(Bm25MemoryIndex::build_from_root_directories(
            &self.root_dirs,
        ));

        tracing::debug!(
            elapsed_ms = start.elapsed().as_millis(),
            doc_count = new_index.documents.len(),
            "[bm25_memory] Rebuilt memory index from disk"
        );

        // Store in cache
        {
            let mut write_guard = self.cache.write().unwrap_or_else(|p| p.into_inner());
            *write_guard = Some(CacheEntry {
                index: new_index.clone(),
                timestamp: Instant::now(),
                last_mtime: fresh_mtime,
            });
        }

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
    /// Builds both Graph and BM25 index in a single disk read pass ($O(N)$ total I/O).
    pub fn build_from_root_directories(root_dirs: &[PathBuf]) -> Self {
        let parsed_files = MarkdownMemoryGraph::parse_root_directories(root_dirs);
        let graph = MarkdownMemoryGraph::build_from_parsed_files(&parsed_files);
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

            let mut term_frequencies: HashMap<String, usize> = HashMap::new();
            for term in terms {
                *term_frequencies.entry(term).or_insert(0) += 1;
            }

            for term in term_frequencies.keys() {
                *doc_freqs.entry(term.clone()).or_insert(0) += 1;
            }

            documents.push(IndexedDocument {
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
        let raw_query_terms = tokenize(query_str);
        if raw_query_terms.is_empty() || self.documents.is_empty() || top_k == 0 {
            return Vec::new();
        }

        // Deduplicate query terms to avoid score multiplication on repeated words
        let mut unique_query_terms: Vec<String> = Vec::new();
        let mut seen = HashSet::new();
        for term in raw_query_terms {
            if seen.insert(term.clone()) {
                unique_query_terms.push(term);
            }
        }

        // Standard Okapi BM25 parameters:
        // k1 = 1.2 (term frequency saturation limit)
        // b = 0.75 (document length normalization penalty)
        const K1: f32 = 1.2;
        const B: f32 = 0.75;
        let num_docs = self.documents.len() as f32;

        let mut scored_results: Vec<(f32, &IndexedDocument)> = self
            .documents
            .iter()
            .map(|doc| {
                let doc_len = doc.term_count as f32;
                let mut score = 0.0f32;

                for q_term in &unique_query_terms {
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

        // Sort descending with deterministic secondary tie-breaking
        scored_results.sort_by(|a, b| {
            b.0.partial_cmp(&a.0)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.1.relative_path.cmp(&b.1.relative_path))
        });
        scored_results.truncate(top_k);

        scored_results
            .into_iter()
            .map(|(score, doc)| {
                let breadcrumbs = self.graph.get_ancestor_breadcrumbs(&doc.path);
                let snippet = extract_snippet(&doc.content, &unique_query_terms);

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

/// Tokenizer: extracts lowercase alphanumeric tokens with char length > 1.
fn tokenize(text: &str) -> Vec<String> {
    text.to_lowercase()
        .split(|c: char| !c.is_alphanumeric())
        .filter(|s| s.chars().count() > 1)
        .map(|s| s.to_string())
        .collect()
}

/// Unicode-safe, case-insensitive snippet extraction directly indexing the original content.
fn extract_snippet(content: &str, query_terms: &[String]) -> String {
    for q in query_terms {
        if q.is_empty() {
            continue;
        }
        if let Some((match_start, match_end)) = find_case_insensitive_match(content, q) {
            let start = match_start.saturating_sub(40);
            let end = (match_end + 160).min(content.len());

            // Adjust to valid UTF-8 character boundaries
            let mut valid_start = start;
            while !content.is_char_boundary(valid_start) && valid_start > 0 {
                valid_start -= 1;
            }
            let mut valid_end = end;
            while !content.is_char_boundary(valid_end) && valid_end < content.len() {
                valid_end += 1;
            }

            let slice = content[valid_start..valid_end].trim();
            return format!("...{}...", slice.replace('\n', " "));
        }
    }

    content.lines().take(2).collect::<Vec<_>>().join(" ")
}

/// Finds case-insensitive substring boundaries matching on character indices.
fn find_case_insensitive_match(haystack: &str, needle: &str) -> Option<(usize, usize)> {
    if needle.is_empty() || haystack.is_empty() {
        return None;
    }

    // Fast path: ASCII
    if needle.is_ascii() && haystack.is_ascii() {
        let needle_lower = needle.to_ascii_lowercase();
        let needle_bytes = needle_lower.as_bytes();
        let haystack_bytes = haystack.as_bytes();

        if needle_bytes.len() > haystack_bytes.len() {
            return None;
        }

        for (i, window) in haystack_bytes.windows(needle_bytes.len()).enumerate() {
            if window
                .iter()
                .zip(needle_bytes)
                .all(|(h, n)| h.to_ascii_lowercase() == *n)
            {
                return Some((i, i + needle.len()));
            }
        }
        return None;
    }

    // Unicode path matching char boundaries
    let needle_chars: Vec<char> = needle.to_lowercase().chars().collect();
    for (start_byte, _) in haystack.char_indices() {
        let mut haystack_chars = haystack[start_byte..].chars();
        let mut matched = true;
        let mut end_byte = start_byte;

        for &n_char in &needle_chars {
            if let Some(h_char) = haystack_chars.next() {
                let mut h_lower = h_char.to_lowercase();
                if h_lower.next() != Some(n_char) || h_lower.next().is_some() {
                    matched = false;
                    break;
                }
                end_byte += h_char.len_utf8();
            } else {
                matched = false;
                break;
            }
        }

        if matched {
            return Some((start_byte, end_byte));
        }
    }

    None
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

    #[test]
    fn test_bm25_cache_reuse_when_unmodified() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("doc.md");
        let mut f = File::create(&file_path).unwrap();
        writeln!(f, "# Cache Test\nDeterministic caching verification.").unwrap();

        let engine = Bm25MemoryEngine::new(vec![dir.path().to_path_buf()]);

        let idx1 = engine.get_or_build_index();
        let idx2 = engine.get_or_build_index();

        // Arc pointers must be identical (cache hit)
        assert!(Arc::ptr_eq(&idx1, &idx2));
    }

    #[test]
    fn test_invalidates_on_content_edit() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("doc.md");
        {
            let mut f = File::create(&file_path).unwrap();
            writeln!(f, "# Initial Title\nInitial content alpha.").unwrap();
        }

        let engine = Bm25MemoryEngine::new(vec![dir.path().to_path_buf()])
            .with_ttl(Duration::from_millis(10));

        let res1 = engine.search("alpha", 5);
        assert_eq!(res1.len(), 1);

        // Sleep briefly so filesystem mtime advances
        std::thread::sleep(Duration::from_millis(50));

        // Overwrite file content with new keyword
        {
            let mut f = File::create(&file_path).unwrap();
            writeln!(f, "# Updated Title\nUpdated content beta.").unwrap();
        }

        // Wait for TTL to expire
        std::thread::sleep(Duration::from_millis(20));

        let res2 = engine.search("beta", 5);
        assert_eq!(
            res2.len(),
            1,
            "Must invalidate cache and find new term on file edit"
        );
        assert_eq!(res2[0].title, "Updated Title");
    }

    #[test]
    fn test_invalidates_on_file_deletion() {
        let dir = tempdir().unwrap();
        let file1_path = dir.path().join("doc1.md");
        let file2_path = dir.path().join("doc2.md");

        {
            let mut f1 = File::create(&file1_path).unwrap();
            writeln!(f1, "# First\nPersistent document content.").unwrap();
            let mut f2 = File::create(&file2_path).unwrap();
            writeln!(f2, "# Ephemeral\nEphemeral document to be deleted.").unwrap();
        }

        let engine = Bm25MemoryEngine::new(vec![dir.path().to_path_buf()])
            .with_ttl(Duration::from_millis(10));

        let res1 = engine.search("ephemeral", 5);
        assert_eq!(res1.len(), 1);

        // Sleep briefly so filesystem directory mtime advances
        std::thread::sleep(Duration::from_millis(50));

        // Delete second file
        std::fs::remove_file(&file2_path).unwrap();

        // Wait for TTL to expire
        std::thread::sleep(Duration::from_millis(20));

        let res2 = engine.search("ephemeral", 5);
        assert_eq!(
            res2.len(),
            0,
            "Must invalidate cache when a file is deleted from directory"
        );
    }

    #[test]
    fn test_invalidates_on_file_rename() {
        let dir = tempdir().unwrap();
        let file1_path = dir.path().join("old_name.md");
        let file2_path = dir.path().join("new_name.md");

        {
            let mut f1 = File::create(&file1_path).unwrap();
            writeln!(f1, "# Renamed Document\nImportant architectural concepts.").unwrap();
        }

        let engine = Bm25MemoryEngine::new(vec![dir.path().to_path_buf()])
            .with_ttl(Duration::from_millis(10));

        let res1 = engine.search("architectural", 5);
        assert_eq!(res1.len(), 1);
        assert_eq!(res1[0].relative_path, "old_name.md");

        std::thread::sleep(Duration::from_millis(50));

        // Rename file
        std::fs::rename(&file1_path, &file2_path).unwrap();

        std::thread::sleep(Duration::from_millis(20));

        let res2 = engine.search("architectural", 5);
        assert_eq!(res2.len(), 1);
        assert_eq!(res2[0].relative_path, "new_name.md");
    }

    #[test]
    fn test_empty_corpus_and_edge_cases() {
        let dir = tempdir().unwrap();
        let engine = Bm25MemoryEngine::new(vec![dir.path().to_path_buf()]);

        // Empty corpus
        let res = engine.search("anything", 5);
        assert!(res.is_empty());

        // Empty query
        let res2 = engine.search("", 5);
        assert!(res2.is_empty());

        // top_k = 0
        let res3 = engine.search("anything", 0);
        assert!(res3.is_empty());
    }

    #[test]
    fn test_unicode_and_duplicate_query_terms() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("unicode.md");
        {
            let mut f = File::create(&file_path).unwrap();
            writeln!(
                f,
                "# Sınıflandırma ve Türkçe Başlık\nTürkçe karakterler test ediliyor ve snippet çıkarılıyor."
            )
            .unwrap();
        }

        let engine = Bm25MemoryEngine::new(vec![dir.path().to_path_buf()]);

        // Query with duplicated term
        let res1 = engine.search("türkçe türkçe", 5);
        assert_eq!(res1.len(), 1);
        assert!(res1[0].snippet.contains("Türkçe"));

        // Query with ASCII term from Unicode document
        let res2 = engine.search("karakterler", 5);
        assert_eq!(res2.len(), 1);
        assert!(res2[0].snippet.contains("karakterler"));

        // Query with Turkish characters
        let res3 = engine.search("başlık", 5);
        assert_eq!(res3.len(), 1);
        assert_eq!(res3[0].title, "Sınıflandırma ve Türkçe Başlık");
    }

    #[test]
    fn test_thundering_herd_concurrency() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("concurrent.md");
        {
            let mut f = File::create(&file_path).unwrap();
            writeln!(f, "# High Concurrency\nParallel load testing.").unwrap();
        }

        let engine = Arc::new(Bm25MemoryEngine::new(vec![dir.path().to_path_buf()]));

        let mut handles = Vec::new();
        for _ in 0..10 {
            let engine_clone = Arc::clone(&engine);
            handles.push(std::thread::spawn(move || {
                let res = engine_clone.search("concurrency", 5);
                assert_eq!(res.len(), 1);
            }));
        }

        for handle in handles {
            handle.join().unwrap();
        }
    }
}

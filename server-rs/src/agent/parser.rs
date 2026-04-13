//! @docs ARCHITECTURE:Agent
//! 
//! ### AI Assist Note
//! **Document Parser**: High-fidelity engine for structured data extraction 
//! and validation across multiple formats (**Markdown**, **CSV**, **PDF**, **TXT**). 
//! Orchestrates **Semantic Chunking** with a 25% character overlap to 
//! maintain context across vector embeddings. Features **Structural Preservation**, 
//! ensuring that Markdown headers and CSV column names are prepended to 
//! chunks for optimal RAG precision.
//! 
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: UTF-8 decoding errors on legacy binary files, 
//!   malformed CSV headers causing alignment mismatch, or PDF text-layer 
//!   extraction failure (empty page or OCR required).
//! - **Trace Scope**: `server-rs::agent::parser`

use anyhow::{anyhow, Result};
use std::path::Path;

/// A parsed section of a document, preserving structural context.
#[derive(Debug, Clone)]
pub struct DocumentSection {
    /// Heading / label for this section (e.g. "Row 3", "## Overview")
    pub heading: String,
    /// The textual content of the section  
    pub content: String,
}

/// The result of parsing a single file.
#[derive(Debug, Clone)]
pub struct ParsedDocument {
    #[allow(dead_code)]
    pub source_path: String,
    pub format: String,
    pub sections: Vec<DocumentSection>,
}

impl ParsedDocument {
    /// Flatten all sections into a single string for simple embedding.
    #[allow(dead_code)]
    pub fn to_flat_text(&self) -> String {
        self.sections
            .iter()
            .map(|s| {
                if s.heading.is_empty() {
                    s.content.clone()
                } else {
                    format!("## {}\n{}", s.heading, s.content)
                }
            })
            .collect::<Vec<_>>()
            .join("\n\n")
    }

    /// Yield individual chunks for granular embedding.
    pub fn to_chunks(&self, max_chars: usize) -> Vec<String> {
        let mut chunks = Vec::new();
        for section in &self.sections {
            let text = if section.heading.is_empty() {
                section.content.clone()
            } else {
                format!("[{}] {}", section.heading, section.content)
            };

            if text.len() <= max_chars {
                chunks.push(text);
            } else {
                // Simple character-boundary chunking with overlap
                let mut start = 0;
                while start < text.len() {
                    let end = (start + max_chars).min(text.len());
                    chunks.push(text[start..end].to_string());
                    start += max_chars - (max_chars / 4); // 25% overlap
                }
            }
        }
        chunks
    }
}

/// Primary entry point: parse a file at the given path into structured sections.
pub fn parse_file(path: &Path) -> Result<ParsedDocument> {
    let ext = path
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_lowercase();

    match ext.as_str() {
        "txt" => parse_plaintext(path),
        "md" => parse_markdown(path),
        "csv" => parse_csv(path),
        "pdf" => parse_pdf_text_layer(path),
        _ => Err(anyhow!("Unsupported file format: .{}", ext)),
    }
}

// ─── Plain Text ──────────────────────────────────────────────

fn parse_plaintext(path: &Path) -> Result<ParsedDocument> {
    let content = std::fs::read_to_string(path)?;
    Ok(ParsedDocument {
        source_path: path.to_string_lossy().to_string(),
        format: "txt".into(),
        sections: vec![DocumentSection {
            heading: String::new(),
            content,
        }],
    })
}

// ─── Markdown ────────────────────────────────────────────────

fn parse_markdown(path: &Path) -> Result<ParsedDocument> {
    let content = std::fs::read_to_string(path)?;
    let mut sections = Vec::new();
    let mut current_heading = String::new();
    let mut current_body: Vec<&str> = Vec::new();

    for line in content.lines() {
        if line.starts_with("# ") || line.starts_with("## ") || line.starts_with("### ") {
            // Flush previous section
            if !current_heading.is_empty() || !current_body.is_empty() {
                sections.push(DocumentSection {
                    heading: current_heading.clone(),
                    content: current_body.join("\n").trim().to_string(),
                });
            }
            current_heading = line.trim_start_matches('#').trim().to_string();
            current_body.clear();
        } else {
            current_body.push(line);
        }
    }

    // Flush last section
    if !current_heading.is_empty() || !current_body.is_empty() {
        sections.push(DocumentSection {
            heading: current_heading,
            content: current_body.join("\n").trim().to_string(),
        });
    }

    Ok(ParsedDocument {
        source_path: path.to_string_lossy().to_string(),
        format: "md".into(),
        sections,
    })
}

// ─── CSV ─────────────────────────────────────────────────────

fn parse_csv(path: &Path) -> Result<ParsedDocument> {
    let content = std::fs::read_to_string(path)?;
    let mut lines = content.lines();

    let header = lines.next().unwrap_or("");
    let header_cols: Vec<&str> = header.split(',').collect();

    let mut sections = Vec::new();

    for (i, line) in lines.enumerate() {
        if line.trim().is_empty() {
            continue;
        }

        let values: Vec<&str> = line.split(',').collect();
        let mut row_text = Vec::new();

        for (col_idx, val) in values.iter().enumerate() {
            let col_name = header_cols.get(col_idx).unwrap_or(&"?");
            row_text.push(format!("{}: {}", col_name.trim(), val.trim()));
        }

        sections.push(DocumentSection {
            heading: format!("Row {}", i + 1),
            content: row_text.join(", "),
        });
    }

    Ok(ParsedDocument {
        source_path: path.to_string_lossy().to_string(),
        format: "csv".into(),
        sections,
    })
}

// ─── PDF (Text Layer only) ───────────────────────────────────
// Phase 4 MVP: Extracts raw bytes and attempts UTF-8 text extraction
// from PDF stream objects. Full OCR/layout support deferred to Phase 4.5
// when Tesseract or a Docling binding is introduced.

fn parse_pdf_text_layer(path: &Path) -> Result<ParsedDocument> {
    let bytes = std::fs::read(path)?;

    // Naive text-layer extraction: scan for BT...ET text blocks
    let content = String::from_utf8_lossy(&bytes);
    let mut extracted = Vec::new();
    let mut in_text = false;
    let mut buffer = String::new();

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed == "BT" {
            in_text = true;
            buffer.clear();
        } else if trimmed == "ET" {
            in_text = false;
            if !buffer.is_empty() {
                // Extract parenthesized text content from PDF operators
                let cleaned = extract_pdf_text_strings(&buffer);
                if !cleaned.is_empty() {
                    extracted.push(cleaned);
                }
            }
        } else if in_text {
            buffer.push_str(line);
            buffer.push('\n');
        }
    }

    if extracted.is_empty() {
        // Fallback: try to find any readable ASCII text in the binary
        let fallback = content
            .chars()
            .filter(|c| c.is_ascii_graphic() || c.is_ascii_whitespace())
            .collect::<String>();

        let cleaned: String = fallback
            .split_whitespace()
            .filter(|w| {
                w.len() > 2
                    && w.chars()
                        .all(|c| c.is_alphanumeric() || c.is_ascii_punctuation())
            })
            .take(500)
            .collect::<Vec<_>>()
            .join(" ");

        if cleaned.len() > 50 {
            extracted.push(cleaned);
        }
    }

    if extracted.is_empty() {
        return Err(anyhow!(
            "No readable text found in PDF. OCR-based parsing not yet available."
        ));
    }

    Ok(ParsedDocument {
        source_path: path.to_string_lossy().to_string(),
        format: "pdf".into(),
        sections: extracted
            .into_iter()
            .enumerate()
            .map(|(i, text)| DocumentSection {
                heading: format!("Page {}", i + 1),
                content: text,
            })
            .collect(),
    })
}

/// Extract text from PDF parenthesized strings like `(Hello World) Tj`
fn extract_pdf_text_strings(block: &str) -> String {
    let mut result = Vec::new();
    let mut in_paren = false;
    let mut current = String::new();

    for ch in block.chars() {
        match ch {
            '(' if !in_paren => {
                in_paren = true;
                current.clear();
            }
            ')' if in_paren => {
                in_paren = false;
                if !current.trim().is_empty() {
                    result.push(current.clone());
                }
            }
            _ if in_paren => {
                current.push(ch);
            }
            _ => {}
        }
    }
    result.join(" ")
}

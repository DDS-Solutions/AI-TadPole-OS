//! @docs ARCHITECTURE:Agent
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Agent Runner / parser
//!
//! ### AI Assist Note
//! - Document chunks use Unicode character limits and retain 25% overlap.
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Type-safe state handling and bounded execution without unhandled panics.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: none declared
//! - **Witness Tests**: `tests::chunks_preserve_unicode_and_character_limits`, `tests::chunks_handle_zero_limit_and_empty_sections`

use crate::error::AppError;
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

    /// Yield individual chunks for granular embedding. A zero limit yields no chunks.
    pub fn to_chunks(&self, max_chars: usize) -> Vec<String> {
        let mut chunks = Vec::new();
        if max_chars == 0 {
            return chunks;
        }
        let step = max_chars - max_chars / 4;
        for section in &self.sections {
            let text = if section.heading.is_empty() {
                section.content.clone()
            } else {
                format!("[{}] {}", section.heading, section.content)
            };

            if text.len() <= max_chars {
                chunks.push(text);
            } else {
                let mut remaining = text.as_str();
                while !remaining.is_empty() {
                    let end = remaining
                        .char_indices()
                        .nth(max_chars)
                        .map_or(remaining.len(), |(offset, _)| offset);
                    chunks.push(remaining[..end].to_string());
                    if end == remaining.len() {
                        break;
                    }
                    let next_start = remaining
                        .char_indices()
                        .nth(step)
                        .map_or(end, |(offset, _)| offset);
                    remaining = &remaining[next_start..];
                }
            }
        }
        chunks
    }
}

/// Primary entry point: parse a file at the given path into structured sections.
pub fn parse_file(path: &Path) -> Result<ParsedDocument, AppError> {
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
        _ => Err(AppError::BadRequest(format!(
            "Unsupported file format: .{}",
            ext
        ))),
    }
}

// ─── Plain Text ──────────────────────────────────────────────

fn parse_plaintext(path: &Path) -> Result<ParsedDocument, AppError> {
    let content = std::fs::read_to_string(path).map_err(AppError::Io)?;
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

fn parse_markdown(path: &Path) -> Result<ParsedDocument, AppError> {
    let content = std::fs::read_to_string(path).map_err(AppError::Io)?;
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

fn parse_csv(path: &Path) -> Result<ParsedDocument, AppError> {
    let content = std::fs::read_to_string(path).map_err(AppError::Io)?;
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

fn parse_pdf_text_layer(path: &Path) -> Result<ParsedDocument, AppError> {
    let bytes = std::fs::read(path).map_err(AppError::Io)?;

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
        return Err(AppError::BadRequest(
            "No readable text found in PDF. OCR-based parsing not yet available.".to_string(),
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

#[cfg(test)]
mod tests {
    use super::*;

    fn document(content: &str) -> ParsedDocument {
        ParsedDocument {
            source_path: "test.txt".into(),
            format: "txt".into(),
            sections: vec![DocumentSection {
                heading: String::new(),
                content: content.into(),
            }],
        }
    }

    #[test]
    fn chunks_preserve_unicode_and_character_limits() {
        let doc = document("é中🦀abc日本語");
        let chunks = doc.to_chunks(4);
        assert_eq!(chunks, vec!["é中🦀a", "abc日", "日本語"]);
        assert!(chunks.iter().all(|chunk| chunk.chars().count() <= 4));
        assert_eq!(document("é中🦀").to_chunks(1), vec!["é", "中", "🦀"]);
    }

    #[test]
    fn chunks_keep_overlap_without_repeating_the_final_tail() {
        assert_eq!(document("abcdefg").to_chunks(4), vec!["abcd", "defg"]);
        assert_eq!(document("abcd").to_chunks(4), vec!["abcd"]);
        assert_eq!(document("abc").to_chunks(usize::MAX), vec!["abc"]);
    }

    #[test]
    fn chunks_handle_zero_limit_and_empty_sections() {
        assert!(document("hello").to_chunks(0).is_empty());
        assert_eq!(document("").to_chunks(4), vec![""]);
        assert!(document("").to_chunks(0).is_empty());
    }

    #[test]
    fn chunks_include_section_headings_in_the_limit() {
        let mut doc = document("text");
        doc.sections[0].heading = "中".into();
        assert_eq!(doc.to_chunks(4), vec!["[中] ", " tex", "xt"]);
    }
}

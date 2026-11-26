//! @docs ARCHITECTURE:Agent
//! ### AI Assist Note
//! Catalog data structures, key normalization, and untrusted text sanitization wrappers.
//! ### 🔍 Debugging & Observability
//! Enforces metadata allowlists and sanitizes text before LLM prompt injection.

use crate::agent::sanitizer::Sanitizer;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub const PUBLIC_METADATA_HEADERS: &[&str] = &[
    "sku",
    "upc",
    "brand",
    "manufacturer",
    "model",
    "model_number",
    "product_code",
];
pub const INTERNAL_METADATA_KEYS: &[&str] = &["source", "qb_id"];

pub fn is_public_metadata_key(key: &str) -> bool {
    PUBLIC_METADATA_HEADERS
        .iter()
        .any(|allowed| allowed.eq_ignore_ascii_case(key.trim()))
}

pub fn is_allowed_stored_metadata_key(key: &str) -> bool {
    is_public_metadata_key(key)
        || INTERNAL_METADATA_KEYS
            .iter()
            .any(|allowed| allowed.eq_ignore_ascii_case(key.trim()))
}

pub fn sanitize_catalog_value(raw: &str, max_chars: usize) -> String {
    Sanitizer::sanitize_for_prompt(raw)
        .chars()
        .filter(|&c| !c.is_control() || c == '\n' || c == '\t')
        .take(max_chars)
        .collect()
}

/// Normalized case-folded deduplication key
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct CatalogKey {
    pub title_norm: String,
    pub category_norm: String,
}

impl CatalogKey {
    pub fn new(title: &str, category: &str) -> Self {
        Self {
            title_norm: title.trim().to_lowercase(),
            category_norm: category.trim().to_lowercase(),
        }
    }
}

/// Catalog item representation with normalized attributes and pricing
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CatalogItem {
    pub id: String,
    pub title: String,
    pub category: String,
    pub description: String,
    pub price_usd: Option<f64>,
    pub metadata: HashMap<String, String>,
}

/// Intermediate item representation prior to ID assignment and deduplication
#[derive(Debug, Clone)]
pub struct CatalogItemDraft {
    pub id_prefix: &'static str,
    pub title: String,
    pub category: String,
    pub description: String,
    pub price_usd: Option<f64>,
    pub metadata: HashMap<String, String>,
}

/// Structural sanitization wrapper for user-supplied catalog text entering LLM prompts
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UntrustedText(String);

impl UntrustedText {
    pub fn sanitize(raw: &str, max_chars: usize) -> Self {
        let filtered: String = Sanitizer::sanitize_for_prompt(raw)
            .chars()
            .filter(|&c| !c.is_control() || c == '\n' || c == '\t')
            .take(max_chars)
            .collect();
        let sanitized = Self::disarm_markdown_breaks(&filtered);
        Self(sanitized)
    }

    fn disarm_markdown_breaks(content: &str) -> String {
        let mut lines = Vec::new();
        for line in content.lines() {
            let trimmed = line.trim_start();
            if trimmed.starts_with('#') || trimmed.starts_with("```") || trimmed.starts_with("~~~")
            {
                lines.push(format!("  {}", line));
            } else {
                lines.push(line.to_string());
            }
        }
        lines.join("\n")
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

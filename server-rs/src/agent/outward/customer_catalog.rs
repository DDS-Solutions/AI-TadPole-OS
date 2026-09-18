//! @docs ARCHITECTURE:Agent
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Agent Runner / customer_catalog
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
use std::fs;
use std::path::Path;
use tracing::{debug, info, warn};

use super::outward_gateway::DEFAULT_MODEL_PROFILE;
use crate::error::AppError;

pub const MAX_CATALOG_ITEMS: usize = 50_000;
pub const MAX_TITLE_LEN: usize = 200;
pub const MAX_CATEGORY_LEN: usize = 100;
pub const MAX_DESCRIPTION_LEN: usize = 2000;
pub const DEFAULT_MAX_CONTEXT_ITEMS: usize = 50;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CatalogItem {
    pub id: String,
    pub title: String,
    pub category: String,
    pub description: String,
    pub price_usd: Option<f64>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomerCatalog {
    pub business_name: String,
    pub items: Vec<CatalogItem>,
    pub default_model_profile: String,
}

#[derive(Debug, Clone)]
pub(crate) struct CatalogItemDraft {
    id_prefix: &'static str,
    title: String,
    category: String,
    description: String,
    price_usd: Option<f64>,
    metadata: HashMap<String, String>,
}

impl CustomerCatalog {
    pub fn new(business_name: impl Into<String>) -> Self {
        let name = business_name.into();
        info!(
            "[CustomerCatalog] Initializing Customer Knowledge Catalog for: {}",
            name
        );
        Self {
            business_name: name,
            items: Vec::new(),
            default_model_profile: DEFAULT_MODEL_PROFILE.to_string(),
        }
    }

    /// Helper to parse and sanitize price values, rejecting negative, NaN, or non-finite values.
    fn parse_price_value(raw: &str, row_number: usize) -> Result<Option<f64>, AppError> {
        let cleaned = raw
            .trim()
            .trim_start_matches('$')
            .trim_start_matches('€')
            .trim_start_matches('£')
            .replace(',', "")
            .trim()
            .to_string();

        if cleaned.is_empty() {
            return Ok(None);
        }

        let val = cleaned.parse::<f64>().map_err(|_| {
            AppError::BadRequest(format!(
                "CSV row {row_number} contains an invalid price format: '{raw}'"
            ))
        })?;

        if !val.is_finite() || val < 0.0 {
            return Err(AppError::BadRequest(format!(
                "CSV row {row_number} price must be a non-negative finite number, got {val}"
            )));
        }

        Ok(Some(val))
    }

    /// Deterministic item ID generation based on content hash
    fn generate_item_id(prefix: &str, title: &str, category: &str) -> String {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(
            format!(
                "{}:{}",
                title.trim().to_lowercase(),
                category.trim().to_lowercase()
            )
            .as_bytes(),
        );
        let hex = hex::encode(hasher.finalize());
        format!("{}-{}", prefix, &hex[..10])
    }

    /// Parse CSV with RFC 4180 compliance (supporting embedded newlines and quotes)
    /// and dynamic, case-insensitive header column mapping.
    pub(crate) fn parse_csv(csv_data: &str) -> Result<Vec<CatalogItemDraft>, AppError> {
        let mut rdr = csv::ReaderBuilder::new()
            .has_headers(true)
            .flexible(true)
            .trim(csv::Trim::All)
            .from_reader(csv_data.as_bytes());

        let headers = rdr
            .headers()
            .map_err(|e| AppError::BadRequest(format!("Failed to read CSV headers: {e}")))?
            .clone();

        if headers.is_empty() {
            return Err(AppError::BadRequest(
                "CSV import is missing a header row".to_string(),
            ));
        }

        let mut title_idx = None;
        let mut category_idx = None;
        let mut desc_idx = None;
        let mut price_idx = None;
        let mut extra_indices = Vec::new();

        for (idx, name) in headers.iter().enumerate() {
            let norm = name.trim().to_lowercase();
            match norm.as_str() {
                "title" | "name" | "item" | "product" | "item_name" => title_idx = Some(idx),
                "category" | "type" | "group" | "item_type" => category_idx = Some(idx),
                "description" | "desc" | "details" => desc_idx = Some(idx),
                "price" | "price_usd" | "unitprice" | "unit_price" | "cost" => {
                    price_idx = Some(idx)
                }
                _ => extra_indices.push((idx, name.trim().to_string())),
            }
        }

        let title_pos = title_idx.ok_or_else(|| {
            AppError::BadRequest("CSV header missing required 'title' or 'name' column".to_string())
        })?;
        let cat_pos = category_idx.ok_or_else(|| {
            AppError::BadRequest(
                "CSV header missing required 'category' or 'type' column".to_string(),
            )
        })?;
        let desc_pos = desc_idx.ok_or_else(|| {
            AppError::BadRequest("CSV header missing required 'description' column".to_string())
        })?;

        let mut drafts = Vec::new();
        for (row_idx, result) in rdr.records().enumerate() {
            let record = result.map_err(|e| {
                AppError::BadRequest(format!("CSV row {} parse error: {}", row_idx + 2, e))
            })?;
            let row_number = row_idx + 2;

            let title = record.get(title_pos).unwrap_or("").trim();
            let category = record.get(cat_pos).unwrap_or("").trim();
            let description = record.get(desc_pos).unwrap_or("").trim();

            if title.is_empty() || category.is_empty() || description.is_empty() {
                return Err(AppError::BadRequest(format!(
                    "CSV row {row_number} requires non-empty title, category, and description fields"
                )));
            }

            let price_usd = if let Some(p_pos) = price_idx {
                if let Some(raw_p) = record.get(p_pos) {
                    Self::parse_price_value(raw_p, row_number)?
                } else {
                    None
                }
            } else {
                None
            };

            let mut metadata = HashMap::new();
            for (idx, col_name) in &extra_indices {
                if let Some(val) = record.get(*idx) {
                    if !val.trim().is_empty() {
                        metadata.insert(col_name.clone(), val.trim().to_string());
                    }
                }
            }

            let capped_title: String = title.chars().take(MAX_TITLE_LEN).collect();
            let capped_cat: String = category.chars().take(MAX_CATEGORY_LEN).collect();
            let capped_desc: String = description.chars().take(MAX_DESCRIPTION_LEN).collect();

            drafts.push(CatalogItemDraft {
                id_prefix: "item",
                title: capped_title,
                category: capped_cat,
                description: capped_desc,
                price_usd,
                metadata,
            });
        }

        if drafts.is_empty() {
            return Err(AppError::BadRequest(
                "CSV import contains no data rows".to_string(),
            ));
        }

        Ok(drafts)
    }

    /// Parse QuickBooks JSON outside the catalog lock.
    pub(crate) fn parse_quickbooks_json(json_str: &str) -> Result<Vec<CatalogItemDraft>, AppError> {
        let parsed_array = serde_json::from_str::<Vec<serde_json::Value>>(json_str)
            .map_err(|error| AppError::BadRequest(format!("Invalid QuickBooks JSON: {error}")))?;
        if parsed_array.is_empty() {
            return Err(AppError::BadRequest(
                "QuickBooks import contains no items".to_string(),
            ));
        }

        parsed_array
            .into_iter()
            .enumerate()
            .map(|(index, value)| {
                if !value.is_object() {
                    return Err(AppError::BadRequest(format!(
                        "QuickBooks item {} must be a JSON object",
                        index + 1
                    )));
                }

                let title = match value.get("Name").or_else(|| value.get("title")) {
                    Some(field) if field.as_str().map(|s| !s.trim().is_empty()).unwrap_or(false) => {
                        field.as_str().unwrap().trim().to_string()
                    }
                    _ => {
                        warn!(
                            item_index = index + 1,
                            "[CustomerCatalog] Missing 'Name' in QuickBooks item, defaulting to 'QB Item'"
                        );
                        "QB Item".to_string()
                    }
                };

                let category = value
                    .get("Type")
                    .or_else(|| value.get("category"))
                    .and_then(|field| field.as_str())
                    .unwrap_or("QuickBooks")
                    .trim()
                    .to_string();

                let description = value
                    .get("Description")
                    .or_else(|| value.get("description"))
                    .and_then(|field| field.as_str())
                    .unwrap_or("QuickBooks Imported Item")
                    .trim()
                    .to_string();

                // Accept UnitPrice as either f64 or numeric string (e.g. "125.00")
                let price_usd = if let Some(field) = value.get("UnitPrice").or_else(|| value.get("price")) {
                    if let Some(num) = field.as_f64() {
                        if !num.is_finite() || num < 0.0 {
                            return Err(AppError::BadRequest(format!(
                                "QuickBooks item {} contains an invalid price: {}",
                                index + 1,
                                num
                            )));
                        }
                        Some(num)
                    } else if let Some(str_val) = field.as_str() {
                        Self::parse_price_value(str_val, index + 1)?
                    } else {
                        return Err(AppError::BadRequest(format!(
                            "QuickBooks item {} price is not a valid number",
                            index + 1
                        )));
                    }
                } else {
                    None
                };

                let mut metadata = HashMap::new();
                metadata.insert("source".to_string(), "quickbooks".to_string());

                let capped_title: String = title.chars().take(MAX_TITLE_LEN).collect();
                let capped_cat: String = category.chars().take(MAX_CATEGORY_LEN).collect();
                let capped_desc: String = description.chars().take(MAX_DESCRIPTION_LEN).collect();

                Ok(CatalogItemDraft {
                    id_prefix: "qb-item",
                    title: capped_title,
                    category: capped_cat,
                    description: capped_desc,
                    price_usd,
                    metadata,
                })
            })
            .collect()
    }

    /// Apply already validated drafts with dedup and global item capacity bounds.
    pub(crate) fn ingest_drafts(&mut self, drafts: Vec<CatalogItemDraft>) -> usize {
        let mut added = 0;
        for draft in drafts {
            if self.items.len() >= MAX_CATALOG_ITEMS {
                warn!(
                    "[CustomerCatalog] Maximum catalog item capacity reached ({})",
                    MAX_CATALOG_ITEMS
                );
                break;
            }

            let title_lower = draft.title.trim().to_lowercase();
            let cat_lower = draft.category.trim().to_lowercase();

            // Dedup on (title, category)
            if let Some(existing) = self.items.iter_mut().find(|i| {
                i.title.trim().to_lowercase() == title_lower
                    && i.category.trim().to_lowercase() == cat_lower
            }) {
                existing.description = draft.description;
                existing.price_usd = draft.price_usd;
                existing.metadata = draft.metadata;
                debug!(title = %draft.title, "[CustomerCatalog] Updated existing catalog item");
            } else {
                let id = Self::generate_item_id(draft.id_prefix, &draft.title, &draft.category);
                self.items.push(CatalogItem {
                    id,
                    title: draft.title,
                    category: draft.category,
                    description: draft.description,
                    price_usd: draft.price_usd,
                    metadata: draft.metadata,
                });
                added += 1;
            }
        }
        added
    }

    /// Ingest raw CSV content into catalog items with quote-aware parsing.
    pub fn ingest_csv(&mut self, csv_data: &str) -> Result<usize, AppError> {
        info!("[CustomerCatalog] Ingesting CSV catalog data for SMB");
        let drafts = Self::parse_csv(csv_data)?;
        let added = self.ingest_drafts(drafts);
        info!(
            "[CustomerCatalog] Successfully processed {} CSV items (added/updated)",
            added
        );
        Ok(added)
    }

    /// Ingest QuickBooks product/invoice JSON objects.
    pub fn ingest_quickbooks_json(&mut self, json_str: &str) -> Result<usize, AppError> {
        info!("[CustomerCatalog] Ingesting QuickBooks JSON data");
        let drafts = Self::parse_quickbooks_json(json_str)?;
        let added = self.ingest_drafts(drafts);
        info!(
            "[CustomerCatalog] Successfully processed {} QuickBooks items (added/updated)",
            added
        );
        Ok(added)
    }

    /// Generate clean, bounded LLM-ready context snippet for local models (Max 50 items)
    pub fn to_llm_context(&self) -> String {
        let max_items = DEFAULT_MAX_CONTEXT_ITEMS;
        let mut context = format!("# Customer Catalog: {}\n\n", self.business_name);

        let display_items = self.items.iter().take(max_items);
        for item in display_items {
            context.push_str(&format!(
                "- **{}** [{}] - {}\n  Description: {}\n  Price: {}\n",
                item.title,
                item.category,
                item.id,
                item.description,
                item.price_usd
                    .map_or("N/A".to_string(), |p| format!("${:.2}", p))
            ));
        }

        if self.items.len() > max_items {
            context.push_str(&format!(
                "\n*Note: Catalog truncated ({}/{} total items shown). Use category search for full listings.*\n",
                max_items,
                self.items.len()
            ));
        }

        context
    }

    /// Generate bounded category-filtered LLM context snippet for targeted queries
    pub fn to_category_context(&self, category: &str) -> String {
        let max_items = DEFAULT_MAX_CONTEXT_ITEMS;
        let mut context = format!(
            "# Customer Catalog: {} (Category: {})\n\n",
            self.business_name, category
        );

        let category_lower = category.to_lowercase();
        let matching_items: Vec<&CatalogItem> = self
            .items
            .iter()
            .filter(|i| i.category.to_lowercase().contains(&category_lower))
            .collect();

        for item in matching_items.iter().take(max_items) {
            context.push_str(&format!(
                "- **{}** [{}] - {}\n  Description: {}\n  Price: {}\n",
                item.title,
                item.category,
                item.id,
                item.description,
                item.price_usd
                    .map_or("N/A".to_string(), |p| format!("${:.2}", p))
            ));
        }

        if matching_items.len() > max_items {
            context.push_str(&format!(
                "\n*Note: Category listings truncated ({}/{} items shown).*\n",
                max_items,
                matching_items.len()
            ));
        }

        context
    }

    /// Search catalog using keyword relevance scoring across title, category, description, and metadata
    pub fn search_catalog(&self, query: &str, top_k: usize) -> Vec<CatalogItem> {
        if query.trim().is_empty() {
            return self.items.iter().take(top_k).cloned().collect();
        }

        let query_terms: Vec<String> = query
            .to_lowercase()
            .split_whitespace()
            .map(|s| s.to_string())
            .collect();

        let mut scored_items: Vec<(usize, &CatalogItem)> = self
            .items
            .iter()
            .filter_map(|item| {
                let title_lower = item.title.to_lowercase();
                let category_lower = item.category.to_lowercase();
                let desc_lower = item.description.to_lowercase();

                let mut score = 0;
                for term in &query_terms {
                    if title_lower.contains(term) {
                        score += 5;
                    }
                    if category_lower.contains(term) {
                        score += 3;
                    }
                    if desc_lower.contains(term) {
                        score += 2;
                    }
                    for (k, v) in &item.metadata {
                        if k.to_lowercase().contains(term) || v.to_lowercase().contains(term) {
                            score += 1;
                        }
                    }
                }

                if score > 0 {
                    Some((score, item))
                } else {
                    None
                }
            })
            .collect();

        scored_items.sort_by_key(|b| std::cmp::Reverse(b.0));
        scored_items
            .into_iter()
            .take(top_k)
            .map(|(_, item)| item.clone())
            .collect()
    }

    /// Save Customer Catalog to local JSON file atomically (write to temp file then rename)
    pub fn save_to_file(&self, path: &Path) -> std::io::Result<()> {
        info!(
            "[CustomerCatalog] Saving catalog atomically to file: {:?}",
            path
        );
        let content = serde_json::to_string_pretty(self)?;

        let tmp_path = path.with_extension("tmp");
        fs::write(&tmp_path, content)?;
        fs::rename(&tmp_path, path)
    }

    /// Load Customer Catalog from local JSON file
    pub fn load_from_file(path: &Path) -> std::io::Result<Self> {
        info!("[CustomerCatalog] Loading catalog from file: {:?}", path);
        let content = fs::read_to_string(path)?;
        let catalog: Self = serde_json::from_str(&content)?;
        Ok(catalog)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_customer_catalog_csv_ingestion_with_header_reordering() {
        let mut catalog = CustomerCatalog::new("Test SMB Business");
        // Reordered columns + currency formatting
        let csv_data = "Price,Description,Title,Category,SKU\n$19.99,Premium widget,Widget A,Hardware,W-001\n\"$1,299.00\",Hourly repair,Service B,Labor,S-002";
        let count = catalog.ingest_csv(csv_data).unwrap();
        assert_eq!(count, 2);
        assert_eq!(catalog.items.len(), 2);
        assert_eq!(catalog.items[0].title, "Widget A");
        assert_eq!(catalog.items[0].category, "Hardware");
        assert_eq!(catalog.items[0].price_usd, Some(19.99));
        assert_eq!(
            catalog.items[0].metadata.get("SKU").map(|s| s.as_str()),
            Some("W-001")
        );
        assert_eq!(catalog.items[1].price_usd, Some(1299.00));
    }

    #[test]
    fn test_customer_catalog_quoted_csv_with_embedded_newlines() {
        let mut catalog = CustomerCatalog::new("Hardware Store");
        let csv_data = "Title,Category,Description,Price\n\"Hammer, 16oz\",Tools,\"Heavy duty\nClaw hammer\nWith rubber grip\",$14.99";
        let count = catalog.ingest_csv(csv_data).unwrap();
        assert_eq!(count, 1);
        assert_eq!(catalog.items[0].title, "Hammer, 16oz");
        assert!(catalog.items[0]
            .description
            .contains("Heavy duty\nClaw hammer"));
        assert_eq!(catalog.items[0].price_usd, Some(14.99));
    }

    #[test]
    fn test_customer_catalog_quickbooks_string_price_support() {
        let mut catalog = CustomerCatalog::new("QuickBooks SMB");
        let qb_json = r#"[
            {"Name": "Consulting Hour", "Type": "Service", "Description": "1 hr business audit", "UnitPrice": "$125.00"},
            {"Name": "Software License", "Type": "Digital", "Description": "Annual seat", "UnitPrice": 299.0}
        ]"#;
        let count = catalog.ingest_quickbooks_json(qb_json).unwrap();
        assert_eq!(count, 2);
        assert_eq!(catalog.items[0].price_usd, Some(125.0));
        assert_eq!(catalog.items[1].price_usd, Some(299.0));
    }

    #[test]
    fn test_customer_catalog_atomic_file_persistence() {
        let mut catalog = CustomerCatalog::new("Persistent SMB");
        catalog
            .ingest_csv("Title,Category,Description,Price\nNail,Hardware,Steel nail,0.10")
            .unwrap();

        let temp_dir = tempfile::tempdir().unwrap();
        let file_path = temp_dir.path().join("catalog.json");

        catalog.save_to_file(&file_path).unwrap();
        let loaded = CustomerCatalog::load_from_file(&file_path).unwrap();

        assert_eq!(loaded.business_name, "Persistent SMB");
        assert_eq!(loaded.items.len(), 1);
        assert_eq!(loaded.items[0].title, "Nail");
    }

    #[test]
    fn test_customer_catalog_rejects_negative_or_nan_prices() {
        let mut catalog = CustomerCatalog::new("Price Validation SMB");

        assert!(catalog
            .ingest_csv("Title,Category,Description,Price\nItem A,Cat,Desc,-19.99")
            .is_err());
        assert!(catalog
            .ingest_csv("Title,Category,Description,Price\nItem B,Cat,Desc,NaN")
            .is_err());
    }

    #[test]
    fn test_customer_catalog_to_llm_context_no_model_leakage() {
        let mut catalog = CustomerCatalog::new("Acme Hardware");
        catalog
            .ingest_csv("Title,Category,Description,Price\nHammer,Tools,Heavy duty hammer,12.50")
            .unwrap();
        let ctx = catalog.to_llm_context();
        assert!(ctx.contains("Acme Hardware"));
        assert!(!ctx.contains("Model Profile:"));
        assert!(ctx.contains("Hammer"));
        assert!(ctx.contains("$12.50"));
    }
}

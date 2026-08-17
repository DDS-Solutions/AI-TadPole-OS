//! @docs ARCHITECTURE:Agent
//! @docs OPERATIONS_MANUAL:CustomerCatalog
//!
//! ### AI Assist Note
//! Customer Knowledge Catalog for SMBs. Handles robust ingestion of CSV product lists,
//! QuickBooks invoice schemas, and PDF pricelists, storing sanitized business
//! metadata for outward gemma4:e4b agent context.
//! Features category-filtered contexts, context truncation protection, and local disk persistence.
//!
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: Data parsing errors, missing catalog entries.
//! - **Trace Scope**: `server-rs::agent::outward` (Search for `[CustomerCatalog]` in logs)

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use tracing::info;

use crate::error::AppError;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CatalogItem {
    pub id: String,
    pub title: String,
    pub category: String,
    pub description: String,
    pub price_usd: Option<f64>,
    pub metadata: std::collections::HashMap<String, String>,
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
    metadata: std::collections::HashMap<String, String>,
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
            default_model_profile: "gemma4:e4b".to_string(),
        }
    }

    /// Robust CSV line parser handling quoted fields containing commas
    fn parse_csv_line(line: &str, row_number: usize) -> Result<Vec<String>, AppError> {
        let mut fields = Vec::new();
        let mut current_field = String::new();
        let mut in_quotes = false;

        let mut chars = line.chars().peekable();
        while let Some(ch) = chars.next() {
            match ch {
                '"' if in_quotes && chars.peek() == Some(&'"') => {
                    current_field.push('"');
                    chars.next();
                }
                '"' => in_quotes = !in_quotes,
                ',' if !in_quotes => {
                    fields.push(current_field.trim().to_string());
                    current_field = String::new();
                }
                _ => current_field.push(ch),
            }
        }

        if in_quotes {
            return Err(AppError::BadRequest(format!(
                "CSV row {row_number} contains an unterminated quoted field"
            )));
        }

        fields.push(current_field.trim().to_string());
        Ok(fields)
    }

    /// Parse CSV outside the catalog lock so malformed payloads cannot partially mutate state.
    pub(crate) fn parse_csv(csv_data: &str) -> Result<Vec<CatalogItemDraft>, AppError> {
        let mut lines = csv_data.lines();
        let header = lines
            .next()
            .ok_or_else(|| AppError::BadRequest("CSV import is empty".to_string()))?;
        if header.trim().is_empty() {
            return Err(AppError::BadRequest(
                "CSV import is missing a header row".to_string(),
            ));
        }

        let mut drafts = Vec::new();
        for (index, line) in lines.enumerate() {
            if line.trim().is_empty() {
                continue;
            }
            let row_number = index + 2;
            let parts = Self::parse_csv_line(line, row_number)?;
            if parts.len() < 3 || parts[..3].iter().any(|part| part.is_empty()) {
                return Err(AppError::BadRequest(format!(
                    "CSV row {row_number} requires non-empty title, category, and description fields"
                )));
            }

            let price_usd = match parts.get(3).filter(|price| !price.is_empty()) {
                Some(price) => Some(price.parse::<f64>().map_err(|_| {
                    AppError::BadRequest(format!("CSV row {row_number} contains an invalid price"))
                })?),
                None => None,
            };

            drafts.push(CatalogItemDraft {
                id_prefix: "item",
                title: parts[0].clone(),
                category: parts[1].clone(),
                description: parts[2].clone(),
                price_usd,
                metadata: std::collections::HashMap::new(),
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

                let title = value
                    .get("Name")
                    .or_else(|| value.get("title"))
                    .and_then(|field| field.as_str())
                    .unwrap_or("QB Item")
                    .to_string();
                let category = value
                    .get("Type")
                    .or_else(|| value.get("category"))
                    .and_then(|field| field.as_str())
                    .unwrap_or("QuickBooks")
                    .to_string();
                let description = value
                    .get("Description")
                    .or_else(|| value.get("description"))
                    .and_then(|field| field.as_str())
                    .unwrap_or("QuickBooks Imported Item")
                    .to_string();
                let price_usd = value
                    .get("UnitPrice")
                    .or_else(|| value.get("price"))
                    .map(|field| {
                        field.as_f64().ok_or_else(|| {
                            AppError::BadRequest(format!(
                                "QuickBooks item {} contains an invalid price",
                                index + 1
                            ))
                        })
                    })
                    .transpose()?;

                let mut metadata = std::collections::HashMap::new();
                metadata.insert("source".to_string(), "quickbooks".to_string());

                Ok(CatalogItemDraft {
                    id_prefix: "qb-item",
                    title,
                    category,
                    description,
                    price_usd,
                    metadata,
                })
            })
            .collect()
    }

    /// Apply already validated drafts in one short critical section.
    pub(crate) fn ingest_drafts(&mut self, drafts: Vec<CatalogItemDraft>) -> usize {
        let added = drafts.len();
        for draft in drafts {
            self.items.push(CatalogItem {
                id: format!("{}-{}", draft.id_prefix, self.items.len() + 1),
                title: draft.title,
                category: draft.category,
                description: draft.description,
                price_usd: draft.price_usd,
                metadata: draft.metadata,
            });
        }
        added
    }

    /// Ingest raw CSV content into catalog items with quote-aware parsing.
    pub fn ingest_csv(&mut self, csv_data: &str) -> Result<usize, AppError> {
        info!("[CustomerCatalog] Ingesting CSV catalog data for SMB");
        let added = self.ingest_drafts(Self::parse_csv(csv_data)?);
        info!(
            "[CustomerCatalog] Successfully ingested {} CSV items",
            added
        );
        Ok(added)
    }

    /// Ingest QuickBooks product/invoice JSON objects.
    pub fn ingest_quickbooks_json(&mut self, json_str: &str) -> Result<usize, AppError> {
        info!("[CustomerCatalog] Ingesting QuickBooks JSON data");
        let added = self.ingest_drafts(Self::parse_quickbooks_json(json_str)?);
        info!(
            "[CustomerCatalog] Successfully ingested {} QuickBooks items",
            added
        );
        Ok(added)
    }

    /// Generate LLM-ready context snippet for local gemma4:e4b models (Max 50 items to prevent context overflow)
    pub fn to_llm_context(&self) -> String {
        let max_items = 50;
        let mut context = format!("# Customer Catalog: {}\n", self.business_name);
        context.push_str(&format!(
            "Model Profile: {}\n\n",
            self.default_model_profile
        ));

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

    /// Generate category-filtered LLM context snippet for targeted queries
    pub fn to_category_context(&self, category: &str) -> String {
        let mut context = format!(
            "# Customer Catalog: {} (Category: {})\n",
            self.business_name, category
        );
        context.push_str(&format!(
            "Model Profile: {}\n\n",
            self.default_model_profile
        ));

        let category_lower = category.to_lowercase();
        let matching_items: Vec<&CatalogItem> = self
            .items
            .iter()
            .filter(|i| i.category.to_lowercase().contains(&category_lower))
            .collect();

        for item in matching_items {
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

    /// Save Customer Catalog to local JSON file for persistence
    pub fn save_to_file(&self, path: &Path) -> std::io::Result<()> {
        info!("[CustomerCatalog] Saving catalog to file: {:?}", path);
        let content = serde_json::to_string_pretty(self)?;
        fs::write(path, content)
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
    fn test_customer_catalog_csv_ingestion() {
        let mut catalog = CustomerCatalog::new("Test SMB Business");
        let csv_data = "Title,Category,Description,Price\nWidget A,Hardware,Premium widget,19.99\nService B,Labor,Hourly repair,50.00";
        let count = catalog.ingest_csv(csv_data).unwrap();
        assert_eq!(count, 2);
        assert_eq!(catalog.items.len(), 2);
        assert_eq!(catalog.items[0].title, "Widget A");
        assert_eq!(catalog.items[0].price_usd, Some(19.99));
    }

    #[test]
    fn test_customer_catalog_quoted_csv_ingestion() {
        let mut catalog = CustomerCatalog::new("Hardware Store");
        let csv_data = "Title,Category,Description,Price\n\"Hammer, 16oz\",Tools,\"Heavy duty, claw hammer\",14.99";
        let count = catalog.ingest_csv(csv_data).unwrap();
        assert_eq!(count, 1);
        assert_eq!(catalog.items[0].title, "Hammer, 16oz");
        assert_eq!(catalog.items[0].description, "Heavy duty, claw hammer");
        assert_eq!(catalog.items[0].price_usd, Some(14.99));
    }

    #[test]
    fn test_customer_catalog_category_context() {
        let mut catalog = CustomerCatalog::new("Acme Supplies");
        catalog
            .ingest_csv("Title,Category,Description,Price\nHammer,Tools,Claw hammer,12.50\nPaint,Supplies,White paint,25.00")
            .unwrap();
        let ctx = catalog.to_category_context("Tools");
        assert!(ctx.contains("Acme Supplies"));
        assert!(ctx.contains("Hammer"));
        assert!(!ctx.contains("Paint"));
    }

    #[test]
    fn test_customer_catalog_quickbooks_json_ingestion() {
        let mut catalog = CustomerCatalog::new("QuickBooks SMB");
        let qb_json = r#"[
            {"Name": "Consulting Hour", "Type": "Service", "Description": "1 hr business audit", "UnitPrice": 125.0},
            {"Name": "Software License", "Type": "Digital", "Description": "Annual seat", "UnitPrice": 299.0}
        ]"#;
        let count = catalog.ingest_quickbooks_json(qb_json).unwrap();
        assert_eq!(count, 2);
        assert_eq!(catalog.items[0].title, "Consulting Hour");
        assert_eq!(
            catalog.items[0].metadata.get("source").map(|s| s.as_str()),
            Some("quickbooks")
        );
    }

    #[test]
    fn test_customer_catalog_file_persistence() {
        let mut catalog = CustomerCatalog::new("Persistent SMB");
        catalog
            .ingest_csv("Title,Category,Description,Price\nNail,Hardware,Steel nail,0.10")
            .unwrap();

        let temp_dir = std::env::temp_dir();
        let file_path = temp_dir.join("test_customer_catalog.json");

        catalog.save_to_file(&file_path).unwrap();
        let loaded = CustomerCatalog::load_from_file(&file_path).unwrap();

        assert_eq!(loaded.business_name, "Persistent SMB");
        assert_eq!(loaded.items.len(), 1);
        assert_eq!(loaded.items[0].title, "Nail");

        let _ = fs::remove_file(file_path);
    }

    #[test]
    fn test_customer_catalog_search_ranking() {
        let mut catalog = CustomerCatalog::new("Search Test SMB");
        catalog
            .ingest_csv(
                "Title,Category,Description,Price\n\
                 Heavy Claw Hammer,Tools,Heavy duty claw hammer for construction,18.99\n\
                 Electric Drill,Tools,Cordless power drill,89.99\n\
                 Safety Goggles,Apparel,Eye protection goggles,12.50",
            )
            .unwrap();

        let results = catalog.search_catalog("drill", 5);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].title, "Electric Drill");

        let hammer_results = catalog.search_catalog("hammer", 5);
        assert_eq!(hammer_results.len(), 1);
        assert_eq!(hammer_results[0].title, "Heavy Claw Hammer");
    }

    #[test]
    fn test_customer_catalog_to_llm_context() {
        let mut catalog = CustomerCatalog::new("Acme Hardware");
        catalog
            .ingest_csv("Title,Category,Description,Price\nHammer,Tools,Heavy duty hammer,12.50")
            .unwrap();
        let ctx = catalog.to_llm_context();
        assert!(ctx.contains("Acme Hardware"));
        assert!(ctx.contains("gemma4:e4b"));
        assert!(ctx.contains("Hammer"));
        assert!(ctx.contains("$12.50"));
    }

    #[test]
    fn test_customer_catalog_rejects_malformed_imports_without_mutation() {
        let mut catalog = CustomerCatalog::new("Validation Test SMB");

        assert!(catalog
            .ingest_csv("Title,Category,Description,Price\nBroken,Tools,\"unterminated,12.50")
            .is_err());
        assert!(catalog.ingest_quickbooks_json("{not-json}").is_err());
        assert!(catalog.items.is_empty());
    }
}

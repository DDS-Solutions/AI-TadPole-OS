//! @docs ARCHITECTURE:Agent
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Agent Runner / customer_catalog
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Type-safe state handling and bounded execution without unhandled panics.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: `AppError::BadRequest`
//! - **Telemetry Targets**: none declared
//! - **Witness Tests**: `agent::outward::customer_catalog::tests::*`

pub mod context;
pub mod parsers;
pub mod persistence;
pub mod search;
#[cfg(test)]
mod tests;
pub mod types;

pub use types::{CatalogItem, CatalogItemDraft, CatalogKey, UntrustedText};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::path::Path;
use tracing::{debug, info, warn};

use super::{
    CatalogLimits, IngestReport, PriceLocale, RowError, DEFAULT_MODEL_PROFILE,
    MAX_BUSINESS_NAME_LEN,
};
use crate::error::AppError;
use types::{is_allowed_stored_metadata_key, sanitize_catalog_value};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomerCatalog {
    pub business_name: String,
    pub items: Vec<CatalogItem>,
    pub default_model_profile: String,
    #[serde(skip)]
    pub(crate) index: HashMap<CatalogKey, usize>,
}

fn update_existing_item(existing: &mut CatalogItem, draft: CatalogItemDraft) {
    existing.description = draft.description;
    if draft.price_usd.is_some() {
        existing.price_usd = draft.price_usd;
    }
    for (k, v) in draft.metadata {
        existing.metadata.insert(k, v);
    }
    debug!(title = ?draft.title, "[CustomerCatalog] Updated existing catalog item");
}

fn sanitize_draft(draft: &mut CatalogItemDraft, limits: &CatalogLimits) {
    draft.title = sanitize_catalog_value(&draft.title, limits.max_title_len);
    draft.category = sanitize_catalog_value(&draft.category, limits.max_category_len);
    draft.description = sanitize_catalog_value(&draft.description, limits.max_description_len);
    draft
        .metadata
        .retain(|key, _| is_allowed_stored_metadata_key(key));
    for value in draft.metadata.values_mut() {
        *value = sanitize_catalog_value(value, limits.max_metadata_value_len);
    }
}

impl CustomerCatalog {
    pub fn new(business_name: impl Into<String>) -> Self {
        let name = business_name.into();
        let sanitized_name = UntrustedText::sanitize(&name, MAX_BUSINESS_NAME_LEN)
            .as_str()
            .to_string();
        info!(
            "[CustomerCatalog] Initializing Customer Knowledge Catalog for: {}",
            sanitized_name
        );
        let mut catalog = Self {
            business_name: sanitized_name,
            items: Vec::new(),
            default_model_profile: DEFAULT_MODEL_PROFILE.to_string(),
            index: HashMap::new(),
        };
        catalog.rebuild_index();
        catalog
    }

    /// Rebuilds the fast O(1) deduplication lookup index
    pub fn rebuild_index(&mut self) {
        self.index.clear();
        for (idx, item) in self.items.iter().enumerate() {
            let key = CatalogKey::new(&item.title, &item.category);
            self.index.insert(key, idx);
        }
    }

    /// Helper to parse and sanitize price values with locale disambiguation
    pub fn parse_price_value(
        raw: &str,
        row_number: usize,
        locale: PriceLocale,
    ) -> Result<Option<f64>, AppError> {
        parsers::parse_price_value(raw, row_number, locale)
    }

    /// Deterministic item ID generation based on content hash
    pub fn generate_item_id(id_prefix: &str, title: &str, category: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(title.trim().to_lowercase().as_bytes());
        hasher.update(b"::");
        hasher.update(category.trim().to_lowercase().as_bytes());
        let result = hasher.finalize();
        format!("{}-{:.10}", id_prefix, hex::encode(result))
    }

    fn add_new_item(&mut self, draft: CatalogItemDraft, key: CatalogKey) {
        let id = Self::generate_item_id(draft.id_prefix, &draft.title, &draft.category);
        let idx = self.items.len();
        self.items.push(CatalogItem {
            id,
            title: draft.title,
            category: draft.category,
            description: draft.description,
            price_usd: draft.price_usd,
            metadata: draft.metadata,
        });
        self.index.insert(key, idx);
    }

    /// Apply drafts with O(1) deduplication and global capacity bounds
    pub(crate) fn ingest_drafts_with_limits(
        &mut self,
        drafts: Vec<CatalogItemDraft>,
        errors: Vec<RowError>,
        limits: &CatalogLimits,
    ) -> IngestReport {
        if self.index.is_empty() && !self.items.is_empty() {
            self.rebuild_index();
        }

        let mut added = 0;
        let mut updated = 0;
        let mut skipped = 0;
        let mut capacity_exceeded = false;

        for mut draft in drafts {
            sanitize_draft(&mut draft, limits);
            let key = CatalogKey::new(&draft.title, &draft.category);

            if let Some(&existing_idx) = self.index.get(&key) {
                update_existing_item(&mut self.items[existing_idx], draft);
                updated += 1;
                continue;
            }
            if self.items.len() >= limits.max_items {
                warn!(
                    "[CustomerCatalog] Maximum item capacity reached ({})",
                    limits.max_items
                );
                capacity_exceeded = true;
                skipped += 1;
                continue;
            }
            self.add_new_item(draft, key);
            added += 1;
        }

        IngestReport {
            added,
            updated,
            skipped: skipped + errors.len(),
            capacity_exceeded,
            errors,
        }
    }

    /// Ingest raw CSV content into catalog items with quote-aware parsing
    pub fn ingest_csv(&mut self, csv_data: &str) -> Result<IngestReport, AppError> {
        self.ingest_csv_with_limits(csv_data, &CatalogLimits::default())
    }

    pub fn ingest_csv_with_limits(
        &mut self,
        csv_data: &str,
        limits: &CatalogLimits,
    ) -> Result<IngestReport, AppError> {
        info!("[CustomerCatalog] Ingesting CSV catalog data for SMB");
        let (drafts, errors) = parsers::parse_csv_with_limits(csv_data, limits)?;
        let report = self.ingest_drafts_with_limits(drafts, errors, limits);
        info!(
            "[CustomerCatalog] Processed CSV items: added={}, updated={}, skipped={}",
            report.added, report.updated, report.skipped
        );
        Ok(report)
    }

    /// Ingest QuickBooks product/invoice JSON objects
    pub fn ingest_quickbooks_json(&mut self, json_str: &str) -> Result<IngestReport, AppError> {
        self.ingest_quickbooks_json_with_limits(json_str, &CatalogLimits::default())
    }

    pub fn ingest_quickbooks_json_with_limits(
        &mut self,
        json_str: &str,
        limits: &CatalogLimits,
    ) -> Result<IngestReport, AppError> {
        info!("[CustomerCatalog] Ingesting QuickBooks JSON data");
        let (drafts, errors) = parsers::parse_quickbooks_json_with_limits(json_str, limits)?;
        let report = self.ingest_drafts_with_limits(drafts, errors, limits);
        info!(
            "[CustomerCatalog] Processed QuickBooks items: added={}, updated={}, skipped={}",
            report.added, report.updated, report.skipped
        );
        Ok(report)
    }

    /// Generate clean, bounded, sanitized LLM-ready context snippet with markdown breakout fencing
    pub fn to_llm_context(&self) -> String {
        self.to_llm_context_with_limits(&CatalogLimits::default())
    }

    pub fn to_llm_context_with_limits(&self, limits: &CatalogLimits) -> String {
        context::to_llm_context_with_limits(&self.business_name, &self.items, limits)
    }

    /// Generate filtered category context snippet
    pub fn to_category_context(&self, category: &str) -> String {
        context::to_category_context(&self.items, category)
    }

    /// Keyword search across catalog items with bounded scoring and ranking
    pub fn search_catalog(&self, query: &str, top_k: usize) -> Vec<CatalogItem> {
        self.search_catalog_with_limits(query, top_k, &CatalogLimits::default())
    }

    pub fn search_catalog_with_limits(
        &self,
        query: &str,
        top_k: usize,
        limits: &CatalogLimits,
    ) -> Vec<CatalogItem> {
        search::search_catalog_with_limits(&self.items, query, top_k, limits)
    }

    /// Save Customer Catalog to local JSON file atomically with durability sync
    pub fn save_to_file(&self, path: &Path) -> std::io::Result<()> {
        persistence::save_catalog_to_file(self, path)
    }

    /// Load Customer Catalog from local JSON file with structural invariant validation
    pub fn load_from_file(path: &Path) -> Result<Self, AppError> {
        persistence::load_catalog_from_file(path)
    }

    /// Validate catalog against agreed invariant boundaries
    pub fn validate(&self, limits: &CatalogLimits) -> Result<(), AppError> {
        persistence::validate_catalog(self, limits)
    }
}

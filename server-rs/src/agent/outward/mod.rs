//! @docs ARCHITECTURE:Agent
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / Agent Runner / mod
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Type-safe state handling and bounded execution without unhandled panics.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: `AppError::BadRequest`
//! - **Telemetry Targets**: none declared
//! - **Witness Tests**: `agent::outward::customer_catalog::tests::*`, `agent::outward::outward_gateway::tests::*`

pub mod customer_catalog;
pub mod outward_gateway;

use crate::error::AppError;
use serde::{Deserialize, Serialize};

// ── Shared System Boundaries & Constant Defaults ─────────────────────────
pub const DEFAULT_A2A_PROTOCOL_VERSION: &str = "0.2.0";
pub const DEFAULT_MODEL_PROFILE: &str = "gemma4:e4b";
pub const DEFAULT_MAX_CONTEXT_ITEMS: usize = 50;
pub const MAX_CATALOG_ITEMS: usize = 50_000;
pub const MAX_TITLE_LEN: usize = 200;
pub const MAX_CATEGORY_LEN: usize = 100;
pub const MAX_DESCRIPTION_LEN: usize = 2000;
pub const MAX_CONTEXT_CHARS: usize = 8000;
pub const MAX_SEARCH_RESULTS: usize = 25;
pub const MAX_METADATA_COLUMNS: usize = 32;
pub const MAX_METADATA_VALUE_LEN: usize = 512;
pub const MAX_SEARCH_QUERY_LEN: usize = 256;
pub const MAX_BUSINESS_NAME_LEN: usize = 120;

/// Validated local inference model profiles
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ModelProfile {
    #[serde(rename = "gemma4:e4b")]
    Gemma4E4B,
    #[serde(rename = "gemma4:e8b")]
    Gemma4E8B,
    #[serde(rename = "gemma4:full")]
    Gemma4Full,
}

impl ModelProfile {
    pub const ALLOWED: &'static [&'static str] = &["gemma4:e4b", "gemma4:e8b", "gemma4:full"];

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Gemma4E4B => "gemma4:e4b",
            Self::Gemma4E8B => "gemma4:e8b",
            Self::Gemma4Full => "gemma4:full",
        }
    }

    pub fn parse(s: &str) -> Result<Self, AppError> {
        match s.trim() {
            "gemma4:e4b" => Ok(Self::Gemma4E4B),
            "gemma4:e8b" => Ok(Self::Gemma4E8B),
            "gemma4:full" => Ok(Self::Gemma4Full),
            other => Err(AppError::BadRequest(format!(
                "Unsupported model profile: '{other}'. Allowed profiles: {}",
                Self::ALLOWED.join(", ")
            ))),
        }
    }
}

impl Default for ModelProfile {
    fn default() -> Self {
        Self::Gemma4E4B
    }
}

/// Price parsing locale formatting specification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum PriceLocale {
    /// '.' = decimal separator, ',' = thousands separator (e.g. US/UK $1,299.00)
    #[default]
    #[serde(rename = "dot_decimal")]
    DotDecimal,
    /// ',' = decimal separator, '.' = thousands separator (e.g. EU €1.299,00 or 19,99)
    #[serde(rename = "comma_decimal")]
    CommaDecimal,
}

/// Bounded limits and thresholds for catalog operations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct CatalogLimits {
    pub max_items: usize,
    pub max_title_len: usize,
    pub max_category_len: usize,
    pub max_description_len: usize,
    pub max_context_items: usize,
    pub max_context_chars: usize,
    pub max_search_results: usize,
    pub max_metadata_columns: usize,
    pub max_metadata_value_len: usize,
    pub max_search_query_len: usize,
    pub max_business_name_len: usize,
    pub price_locale: PriceLocale,
}

impl Default for CatalogLimits {
    fn default() -> Self {
        Self {
            max_items: MAX_CATALOG_ITEMS,
            max_title_len: MAX_TITLE_LEN,
            max_category_len: MAX_CATEGORY_LEN,
            max_description_len: MAX_DESCRIPTION_LEN,
            max_context_items: DEFAULT_MAX_CONTEXT_ITEMS,
            max_context_chars: MAX_CONTEXT_CHARS,
            max_search_results: MAX_SEARCH_RESULTS,
            max_metadata_columns: MAX_METADATA_COLUMNS,
            max_metadata_value_len: MAX_METADATA_VALUE_LEN,
            max_search_query_len: MAX_SEARCH_QUERY_LEN,
            max_business_name_len: MAX_BUSINESS_NAME_LEN,
            price_locale: PriceLocale::default(),
        }
    }
}

/// Structured ingestion execution report with non-fatal row tracking
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct IngestReport {
    pub added: usize,
    pub updated: usize,
    pub skipped: usize,
    pub capacity_exceeded: bool,
    pub errors: Vec<RowError>,
}

impl IngestReport {
    pub fn total_processed(&self) -> usize {
        self.added + self.updated
    }
}

/// Individual row failure descriptor
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RowError {
    pub row_number: usize,
    pub message: String,
}

#[allow(unused_imports)]
pub use customer_catalog::{CatalogItem, CatalogKey, CustomerCatalog, UntrustedText};
#[allow(unused_imports)]
pub use outward_gateway::{A2aAgentCard, A2aSkill, BusinessProfile, OutwardGateway};

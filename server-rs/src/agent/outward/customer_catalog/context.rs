//! @docs ARCHITECTURE:Agent
//! ### AI Assist Note
//! Formats bounded LLM prompt context blocks with structural markdown fencing.
//! ### 🔍 Debugging & Observability
//! Disarms markdown headers and applies hard context truncation indicators.

use super::super::{
    CatalogLimits, DEFAULT_MAX_CONTEXT_ITEMS, MAX_CATEGORY_LEN, MAX_DESCRIPTION_LEN, MAX_TITLE_LEN,
};
use super::types::{CatalogItem, UntrustedText};

fn format_catalog_entry(item: &CatalogItem, limits: &CatalogLimits) -> String {
    let safe_title = UntrustedText::sanitize(&item.title, limits.max_title_len);
    let safe_cat = UntrustedText::sanitize(&item.category, limits.max_category_len);
    let safe_desc = UntrustedText::sanitize(&item.description, limits.max_description_len);
    let price_str = item
        .price_usd
        .map_or("N/A".to_string(), |p| format!("${p:.2}"));

    format!(
        "- **{}** [{}] - {}\n  Description: {}\n  Price: {}\n",
        safe_title.as_str(),
        safe_cat.as_str(),
        item.id,
        safe_desc.as_str(),
        price_str
    )
}

pub fn to_llm_context_with_limits(
    business_name: &str,
    items: &[CatalogItem],
    limits: &CatalogLimits,
) -> String {
    let safe_name = UntrustedText::sanitize(business_name, limits.max_business_name_len);
    let mut context = format!(
        "# Customer Catalog: {}\n\n<!-- BEGIN_CATALOG_DATA -->\n",
        safe_name.as_str()
    );

    let mut total_chars = context.len();
    let mut items_rendered = 0;

    for item in items.iter().take(limits.max_context_items) {
        let entry = format_catalog_entry(item, limits);
        if total_chars + entry.len() > limits.max_context_chars {
            break;
        }
        context.push_str(&entry);
        total_chars += entry.len();
        items_rendered += 1;
    }

    context.push_str("<!-- END_CATALOG_DATA -->\n");
    if items.len() > items_rendered && !items.is_empty() {
        context.push_str(&format!(
            "\n*Note: Catalog truncated ({items_rendered}/{} total items shown due to bounded context limits).*\n",
            items.len()
        ));
    }
    context
}

pub fn to_category_context(items: &[CatalogItem], category: &str) -> String {
    let trimmed = category.trim();
    if trimmed.is_empty() {
        return "# Customer Catalog (Category: empty)\n\n<!-- BEGIN_CATALOG_DATA -->\n<!-- END_CATALOG_DATA -->\n".into();
    }

    let cat_lower = trimmed.to_lowercase();
    let safe_cat_query = UntrustedText::sanitize(trimmed, MAX_CATEGORY_LEN);
    let mut context = format!(
        "# Customer Catalog (Category: {})\n\n<!-- BEGIN_CATALOG_DATA -->\n",
        safe_cat_query.as_str()
    );

    for item in items
        .iter()
        .filter(|i| i.category.trim().to_lowercase() == cat_lower)
        .take(DEFAULT_MAX_CONTEXT_ITEMS)
    {
        let safe_title = UntrustedText::sanitize(&item.title, MAX_TITLE_LEN);
        let safe_desc = UntrustedText::sanitize(&item.description, MAX_DESCRIPTION_LEN);
        let price_str = item
            .price_usd
            .map_or("N/A".to_string(), |p| format!("${p:.2}"));
        context.push_str(&format!(
            "- **{}** - {}\n  Description: {}\n  Price: {}\n",
            safe_title.as_str(),
            item.id,
            safe_desc.as_str(),
            price_str
        ));
    }

    context.push_str("<!-- END_CATALOG_DATA -->\n");
    context
}

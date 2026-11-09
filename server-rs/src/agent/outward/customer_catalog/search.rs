//! @docs ARCHITECTURE:Agent
//! ### AI Assist Note
//! Keyword relevance scoring, result clamping, and public metadata projections.
//! ### 🔍 Debugging & Observability
//! Filters non-public metadata attributes and clamps search output bounds.

use super::super::CatalogLimits;
use super::types::{is_public_metadata_key, sanitize_catalog_value, CatalogItem};

struct ItemSearchFields<'a> {
    title: &'a str,
    cat: &'a str,
    desc: &'a str,
}

fn score_single_term(
    term: &str,
    fields: &ItemSearchFields<'_>,
    item: &CatalogItem,
    limits: &CatalogLimits,
) -> usize {
    let mut score = 0;
    if fields.title.contains(term) {
        score += 10;
    }
    if fields.cat.contains(term) {
        score += 5;
    }
    if fields.desc.contains(term) {
        score += 2;
    }
    for (k, v) in item
        .metadata
        .iter()
        .filter(|(k, _)| is_public_metadata_key(k))
        .take(limits.max_metadata_columns)
    {
        let safe_key = k.trim().to_lowercase();
        let safe_value = sanitize_catalog_value(v, limits.max_metadata_value_len).to_lowercase();
        if safe_key.contains(term) || safe_value.contains(term) {
            score += 1;
        }
    }
    score
}

fn score_item(item: &CatalogItem, query_terms: &[String], limits: &CatalogLimits) -> usize {
    let title = sanitize_catalog_value(&item.title, limits.max_title_len).to_lowercase();
    let cat = sanitize_catalog_value(&item.category, limits.max_category_len).to_lowercase();
    let desc = sanitize_catalog_value(&item.description, limits.max_description_len).to_lowercase();
    let fields = ItemSearchFields {
        title: &title,
        cat: &cat,
        desc: &desc,
    };

    let mut total_score = 0;
    for term in query_terms {
        total_score += score_single_term(term, &fields, item, limits);
    }
    total_score
}

fn project_public_item(item: &CatalogItem, limits: &CatalogLimits) -> CatalogItem {
    let mut public = item.clone();
    public.id = sanitize_catalog_value(&public.id, 128);
    public.title = sanitize_catalog_value(&public.title, limits.max_title_len);
    public.category = sanitize_catalog_value(&public.category, limits.max_category_len);
    public.description = sanitize_catalog_value(&public.description, limits.max_description_len);
    public.metadata.retain(|key, _| is_public_metadata_key(key));
    public.metadata.retain(|_, value| {
        *value = sanitize_catalog_value(value, limits.max_metadata_value_len);
        !value.is_empty()
    });
    public
}

pub fn search_catalog_with_limits(
    items: &[CatalogItem],
    query: &str,
    top_k: usize,
    limits: &CatalogLimits,
) -> Vec<CatalogItem> {
    let trimmed = query.trim();
    if trimmed.is_empty() || trimmed.chars().count() > limits.max_search_query_len {
        return Vec::new();
    }

    let clamped_k = top_k.clamp(1, limits.max_search_results);
    let query_terms: Vec<String> = trimmed
        .split_whitespace()
        .map(|t| t.to_lowercase())
        .collect();
    if query_terms.is_empty() {
        return Vec::new();
    }

    let mut scored: Vec<(usize, &CatalogItem)> = items
        .iter()
        .filter_map(|item| {
            let score = score_item(item, &query_terms, limits);
            if score > 0 {
                Some((score, item))
            } else {
                None
            }
        })
        .collect();

    scored.sort_by_key(|a| std::cmp::Reverse(a.0));
    scored
        .into_iter()
        .take(clamped_k)
        .map(|(_, item)| project_public_item(item, limits))
        .collect()
}

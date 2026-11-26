//! @docs ARCHITECTURE:Agent
//! ### AI Assist Note
//! Format-specific CSV and QuickBooks catalog parsers with non-fatal row fault isolation.
//! ### 🔍 Debugging & Observability
//! Disambiguates European comma decimals and captures structural row errors.

use std::collections::HashMap;

use super::super::{CatalogLimits, PriceLocale, RowError};
use super::types::{is_public_metadata_key, sanitize_catalog_value, CatalogItemDraft};
use crate::error::AppError;

fn check_dot_decimal_ambiguity(
    stripped: &str,
    row_number: usize,
    raw: &str,
) -> Result<(), AppError> {
    if !stripped.contains(',') || stripped.contains('.') {
        return Ok(());
    }
    let parts: Vec<&str> = stripped.split(',').collect();
    if parts.len() == 2 && (parts[1].len() == 1 || parts[1].len() == 2) {
        return Err(AppError::BadRequest(format!(
            "CSV row {row_number}: '{raw}' is ambiguous between decimal and thousands separators. Set price_locale=comma_decimal for European-format files."
        )));
    }
    Ok(())
}

pub fn clean_price_string(
    stripped: &str,
    row_number: usize,
    locale: PriceLocale,
    raw: &str,
) -> Result<String, AppError> {
    match locale {
        PriceLocale::DotDecimal => {
            check_dot_decimal_ambiguity(stripped, row_number, raw)?;
            Ok(stripped.replace(',', ""))
        }
        PriceLocale::CommaDecimal => Ok(stripped.replace('.', "").replace(',', ".")),
    }
}

pub fn validate_non_negative_finite(
    val: f64,
    row_number: usize,
    context: &str,
) -> Result<f64, AppError> {
    if !val.is_finite() || val < 0.0 || (val == 0.0 && val.is_sign_negative()) {
        return Err(AppError::BadRequest(format!(
            "{context} row {row_number} price must be a non-negative finite number, got {val}"
        )));
    }
    Ok(if val == 0.0 { 0.0 } else { val })
}

pub fn parse_price_value(
    raw: &str,
    row_number: usize,
    locale: PriceLocale,
) -> Result<Option<f64>, AppError> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Ok(None);
    }
    if trimmed.starts_with('(') && trimmed.ends_with(')') {
        return Err(AppError::BadRequest(format!(
            "CSV row {row_number} price must be a non-negative finite number, got negative accounting notation: '{trimmed}'"
        )));
    }

    let stripped = trimmed.trim_start_matches(['$', '€', '£']).trim();
    if stripped.is_empty() {
        return Ok(None);
    }

    let cleaned = clean_price_string(stripped, row_number, locale, raw)?;
    let val = cleaned.parse::<f64>().map_err(|_| {
        AppError::BadRequest(format!(
            "CSV row {row_number} contains an invalid price format: '{raw}'"
        ))
    })?;

    let normalized = validate_non_negative_finite(val, row_number, "CSV")?;
    Ok(Some(normalized))
}

pub struct CsvColumnMap {
    pub title_pos: usize,
    pub cat_pos: usize,
    pub desc_pos: usize,
    pub price_idx: Option<usize>,
    pub extra_indices: Vec<(usize, String)>,
}

fn is_title_header(h: &str) -> bool {
    matches!(h, "title" | "name" | "item" | "product")
}
fn is_category_header(h: &str) -> bool {
    matches!(h, "category" | "type" | "group")
}
fn is_desc_header(h: &str) -> bool {
    matches!(h, "description" | "desc" | "details")
}
fn is_price_header(h: &str) -> bool {
    matches!(h, "price" | "price_usd" | "unit_price")
}

struct ColumnTracker {
    title: Option<usize>,
    category: Option<usize>,
    description: Option<usize>,
    price: Option<usize>,
    extra: Vec<(usize, String)>,
}

fn assign_standard_column(h: &str, idx: usize, t: &mut ColumnTracker) -> bool {
    if is_title_header(h) && t.title.is_none() {
        t.title = Some(idx);
        true
    } else if is_category_header(h) && t.category.is_none() {
        t.category = Some(idx);
        true
    } else if is_desc_header(h) && t.description.is_none() {
        t.description = Some(idx);
        true
    } else if is_price_header(h) && t.price.is_none() {
        t.price = Some(idx);
        true
    } else {
        false
    }
}

impl CsvColumnMap {
    pub fn from_headers(
        headers: &csv::StringRecord,
        limits: &CatalogLimits,
    ) -> Result<Self, AppError> {
        let mut t = ColumnTracker {
            title: None,
            category: None,
            description: None,
            price: None,
            extra: Vec::new(),
        };

        for (idx, header) in headers.iter().enumerate() {
            let h = header.trim().to_lowercase();
            if !assign_standard_column(&h, idx, &mut t)
                && is_public_metadata_key(header)
                && t.extra.len() < limits.max_metadata_columns
            {
                t.extra.push((idx, header.trim().to_string()));
            }
        }

        let title_pos = t.title.ok_or_else(|| {
            AppError::BadRequest("CSV missing required 'title' or 'name' column".into())
        })?;
        let cat_pos = t.category.ok_or_else(|| {
            AppError::BadRequest("CSV missing required 'category' or 'type' column".into())
        })?;
        let desc_pos = t.description.ok_or_else(|| {
            AppError::BadRequest("CSV missing required 'description' or 'details' column".into())
        })?;

        Ok(Self {
            title_pos,
            cat_pos,
            desc_pos,
            price_idx: t.price,
            extra_indices: t.extra,
        })
    }
}

fn parse_single_csv_record(
    record: &csv::StringRecord,
    col_map: &CsvColumnMap,
    row_number: usize,
    limits: &CatalogLimits,
) -> Result<CatalogItemDraft, RowError> {
    let title_raw = record.get(col_map.title_pos).unwrap_or("").trim();
    let cat_raw = record.get(col_map.cat_pos).unwrap_or("").trim();
    let desc_raw = record.get(col_map.desc_pos).unwrap_or("").trim();

    if title_raw.is_empty() || cat_raw.is_empty() || desc_raw.is_empty() {
        return Err(RowError {
            row_number,
            message: "Required fields (title, category, description) cannot be empty".to_string(),
        });
    }

    let price_usd = match col_map.price_idx.and_then(|idx| record.get(idx)) {
        Some(p_str) => {
            parse_price_value(p_str, row_number, limits.price_locale).map_err(|e| RowError {
                row_number,
                message: format!("{e}"),
            })?
        }
        None => None,
    };

    let mut metadata = HashMap::new();
    for (idx, col_name) in &col_map.extra_indices {
        if let Some(val) = record.get(*idx) {
            let v = val.trim();
            if !v.is_empty() {
                metadata.insert(
                    col_name.clone(),
                    sanitize_catalog_value(v, limits.max_metadata_value_len),
                );
            }
        }
    }

    Ok(CatalogItemDraft {
        id_prefix: "item",
        title: sanitize_catalog_value(title_raw, limits.max_title_len),
        category: sanitize_catalog_value(cat_raw, limits.max_category_len),
        description: sanitize_catalog_value(desc_raw, limits.max_description_len),
        price_usd,
        metadata,
    })
}

pub fn parse_csv_with_limits(
    csv_data: &str,
    limits: &CatalogLimits,
) -> Result<(Vec<CatalogItemDraft>, Vec<RowError>), AppError> {
    let clean_csv = csv_data.strip_prefix('\u{feff}').unwrap_or(csv_data);
    let mut reader = csv::ReaderBuilder::new()
        .flexible(true)
        .trim(csv::Trim::All)
        .from_reader(clean_csv.as_bytes());

    let headers = reader
        .headers()
        .map_err(|e| AppError::BadRequest(format!("Failed to parse CSV headers: {e}")))?;
    let col_map = CsvColumnMap::from_headers(headers, limits)?;

    let mut drafts = Vec::new();
    let mut errors = Vec::new();

    for (row_idx, result) in reader.records().enumerate() {
        let row_number = row_idx + 2;
        let record = match result {
            Ok(rec) => rec,
            Err(e) => {
                errors.push(RowError {
                    row_number,
                    message: format!("Malformed CSV row format: {e}"),
                });
                continue;
            }
        };
        match parse_single_csv_record(&record, &col_map, row_number, limits) {
            Ok(draft) => drafts.push(draft),
            Err(err) => errors.push(err),
        }
    }

    if drafts.is_empty() && errors.is_empty() {
        return Err(AppError::BadRequest(
            "CSV file contains no data rows".to_string(),
        ));
    }
    Ok((drafts, errors))
}

fn extract_qb_title(obj: &serde_json::Map<String, serde_json::Value>, row_number: usize) -> String {
    let candidate = obj
        .get("Name")
        .or_else(|| obj.get("DisplayName"))
        .or_else(|| obj.get("title"))
        .and_then(|v| v.as_str())
        .filter(|s| !s.trim().is_empty());
    if let Some(title) = candidate {
        return title.to_string();
    }
    if let Some(sku) = obj.get("Sku").and_then(|v| v.as_str()) {
        format!("QB Item ({sku})")
    } else if let Some(id) = obj.get("Id").and_then(|v| v.as_str()) {
        format!("QB Item ({id})")
    } else {
        format!("QB Item #{row_number}")
    }
}

fn extract_qb_price(
    obj: &serde_json::Map<String, serde_json::Value>,
    row_number: usize,
    locale: PriceLocale,
) -> Result<Option<f64>, String> {
    let Some(p_val) = obj.get("UnitPrice").or_else(|| obj.get("price")) else {
        return Ok(None);
    };
    if let Some(p_num) = p_val.as_f64() {
        if !p_num.is_finite() || p_num < 0.0 || (p_num == 0.0 && p_num.is_sign_negative()) {
            return Err(format!(
                "QuickBooks item price must be non-negative, got {p_num}"
            ));
        }
        return Ok(Some(p_num));
    }
    if let Some(p_str) = p_val.as_str() {
        return parse_price_value(p_str, row_number, locale).map_err(|e| format!("{e}"));
    }
    Ok(None)
}

fn parse_single_qb_item(
    item: serde_json::Value,
    row_number: usize,
    limits: &CatalogLimits,
) -> Result<CatalogItemDraft, RowError> {
    let Some(obj) = item.as_object() else {
        return Err(RowError {
            row_number,
            message: "QuickBooks array element is not an object".into(),
        });
    };

    let title = extract_qb_title(obj, row_number);
    let category = obj
        .get("Type")
        .or_else(|| obj.get("category"))
        .and_then(|v| v.as_str())
        .filter(|s| !s.trim().is_empty())
        .unwrap_or("General");
    let description = obj
        .get("Description")
        .or_else(|| obj.get("description"))
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let price_usd =
        extract_qb_price(obj, row_number, limits.price_locale).map_err(|msg| RowError {
            row_number,
            message: msg,
        })?;

    let mut metadata = HashMap::new();
    metadata.insert("source".to_string(), "quickbooks".to_string());
    if let Some(id_val) = obj.get("Id").and_then(|v| v.as_str()) {
        metadata.insert("qb_id".to_string(), id_val.to_string());
    }

    Ok(CatalogItemDraft {
        id_prefix: "item",
        title: title.chars().take(limits.max_title_len).collect(),
        category: category.chars().take(limits.max_category_len).collect(),
        description: description
            .chars()
            .take(limits.max_description_len)
            .collect(),
        price_usd,
        metadata,
    })
}

pub fn parse_quickbooks_json_with_limits(
    json_str: &str,
    limits: &CatalogLimits,
) -> Result<(Vec<CatalogItemDraft>, Vec<RowError>), AppError> {
    let items_val: Vec<serde_json::Value> = serde_json::from_str(json_str)
        .map_err(|e| AppError::BadRequest(format!("Invalid QuickBooks JSON: {e}")))?;

    let mut drafts = Vec::new();
    let mut errors = Vec::new();

    for (idx, item) in items_val.into_iter().enumerate() {
        match parse_single_qb_item(item, idx + 1, limits) {
            Ok(draft) => drafts.push(draft),
            Err(err) => errors.push(err),
        }
    }

    Ok((drafts, errors))
}

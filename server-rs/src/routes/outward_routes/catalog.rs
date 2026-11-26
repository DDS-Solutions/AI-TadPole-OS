//! @docs ARCHITECTURE:OutwardGateway
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / HTTP Routes / outward_routes::catalog
//!
//! ### 🔍 Debugging & Observability
//! - **Telemetry Targets**: `[OutwardRoutes]`

use super::state::{
    atomic_write_json, company_catalog_path, company_workspaces_dir, validate_practice_company_key,
    ApiResponse, OutwardAppState,
};
use crate::agent::business::types::Offering;
use crate::agent::outward::{CatalogItem, CustomerCatalog, IngestReport};
use crate::error::AppError;
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Json, Response},
};
use serde::Deserialize;
use std::collections::HashMap;

const MAX_IMPORT_PAYLOAD_LEN: usize = 512 * 1024;

#[derive(Debug, Deserialize)]
pub struct SearchQueryParams {
    pub q: Option<String>,
    pub limit: Option<usize>,
    #[serde(default)]
    pub company_key: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ImportCatalogPayload {
    pub csv_content: Option<String>,
    pub qb_json_content: Option<String>,
    #[serde(default)]
    pub company_key: Option<String>,
}

/// GET /a2a/v1/catalog/search
/// Searches the public customer catalog with a bounded result limit. No bearer token is required.
/// @docs API_REFERENCE:SearchOutwardCatalog
pub async fn search_catalog_handler(
    State(state): State<OutwardAppState>,
    Query(params): Query<SearchQueryParams>,
    req: axum::extract::Request,
) -> Result<Response, AppError> {
    let client_ip = crate::middleware::extract_client_ip(&req);
    if !state.rate_limiter.check_and_record(&client_ip) {
        return Err(AppError::RateLimit(
            "Capped at 60 requests per minute".to_string(),
        ));
    }

    let query_str = params.q.unwrap_or_default();
    let limit = params.limit.unwrap_or(20).clamp(1, 100);

    let results = match params.company_key {
        Some(ref company_key) => {
            validate_practice_company_key(company_key)?;
            search_company_catalog(&state, company_key, &query_str, limit).await
        }
        None => search_global_catalog(&state, &query_str, limit)?,
    };

    Ok((
        StatusCode::OK,
        Json(ApiResponse {
            success: true,
            message: format!("Retrieved {} matching items", results.len()),
            data: Some(results),
        }),
    )
        .into_response())
}

fn search_global_catalog(
    state: &OutwardAppState,
    query_str: &str,
    limit: usize,
) -> Result<Vec<CatalogItem>, AppError> {
    let catalog = state.catalog.lock().map_err(|_| {
        AppError::InternalServerError("Customer catalog state lock is poisoned".to_string())
    })?;
    Ok(catalog.search_catalog(query_str, limit))
}

async fn search_company_catalog(
    state: &OutwardAppState,
    company_key: &str,
    query_str: &str,
    limit: usize,
) -> Vec<CatalogItem> {
    let catalog_path = company_catalog_path(state, company_key);
    let Ok(raw) = tokio::fs::read_to_string(&catalog_path).await else {
        return Vec::new();
    };
    let Some(items) = parse_company_catalog_items(&raw, company_key) else {
        return Vec::new();
    };

    let mut isolated = CustomerCatalog::new(company_key);
    isolated.items = items;
    isolated.rebuild_index();
    isolated.search_catalog(query_str, limit)
}

fn parse_company_catalog_items(raw: &str, company_key: &str) -> Option<Vec<CatalogItem>> {
    if let Ok(items) = serde_json::from_str::<Vec<CatalogItem>>(raw) {
        return Some(items);
    }

    let offerings = serde_json::from_str::<Vec<Offering>>(raw).ok()?;
    let prefix = format!("{company_key}-");
    let mapped = offerings
        .into_iter()
        .map(|o| offering_to_catalog_item(o, &prefix))
        .collect();
    Some(mapped)
}

fn offering_to_catalog_item(offering: Offering, prefix: &str) -> CatalogItem {
    let mut metadata = HashMap::new();
    metadata.insert("reference".to_string(), offering.display_reference);
    metadata.insert("currency".to_string(), offering.currency);
    CatalogItem {
        id: offering.id,
        title: offering
            .title
            .strip_prefix(prefix)
            .unwrap_or(&offering.title)
            .to_string(),
        category: "Catalog".to_string(),
        description: offering.description,
        price_usd: Some(offering.price_minor as f64 / 100.0),
        metadata,
    }
}

/// POST /a2a/v1/catalog/import
/// Validates and atomically imports CSV or QuickBooks JSON payloads up to 512 KiB.
/// @docs API_REFERENCE:ImportOutwardCatalog
pub async fn import_catalog_handler(
    State(state): State<OutwardAppState>,
    Json(payload): Json<ImportCatalogPayload>,
) -> Result<Response, AppError> {
    validate_import_payload(&payload)?;

    let csv_opt = payload.csv_content.as_deref();
    let qb_opt = payload.qb_json_content.as_deref();

    let combined_report = match payload.company_key.as_deref() {
        Some(company_key) => import_company_catalog(&state, company_key, csv_opt, qb_opt).await?,
        None => import_global_catalog(&state, csv_opt, qb_opt)?,
    };

    let has_warnings = !combined_report.errors.is_empty() || combined_report.capacity_exceeded;
    let message = format_import_message(&combined_report, has_warnings);

    tracing::info!(
        "[OutwardRoutes] Processed catalog import: added={}, updated={}, skipped={}, errors={}, capacity_exceeded={}",
        combined_report.added,
        combined_report.updated,
        combined_report.skipped,
        combined_report.errors.len(),
        combined_report.capacity_exceeded
    );

    Ok((
        StatusCode::OK,
        Json(ApiResponse {
            success: !has_warnings,
            message,
            data: Some(combined_report),
        }),
    )
        .into_response())
}

fn validate_import_payload(payload: &ImportCatalogPayload) -> Result<(), AppError> {
    if payload.csv_content.as_ref().map_or(0, String::len) > MAX_IMPORT_PAYLOAD_LEN
        || payload.qb_json_content.as_ref().map_or(0, String::len) > MAX_IMPORT_PAYLOAD_LEN
    {
        return Err(AppError::BadRequest(
            "Import payload exceeds maximum allowed size of 512KB".to_string(),
        ));
    }

    if payload.csv_content.is_none() && payload.qb_json_content.is_none() {
        return Err(AppError::BadRequest(
            "Import requires csv_content or qb_json_content".to_string(),
        ));
    }
    Ok(())
}

/// FR-6 / NFR-1 Isolation: company-scoped imports run in an isolated catalog instance
/// and NEVER read from or mutate the global singleton `state.catalog`.
async fn import_company_catalog(
    state: &OutwardAppState,
    company_key: &str,
    csv_opt: Option<&str>,
    qb_opt: Option<&str>,
) -> Result<IngestReport, AppError> {
    validate_practice_company_key(company_key)?;

    let mut isolated_catalog = CustomerCatalog::new(company_key);
    let report = ingest_into_catalog(&mut isolated_catalog, csv_opt, qb_opt)?;

    if let Some(workspaces_dir) = company_workspaces_dir(state) {
        let offerings = build_company_offerings(company_key, &isolated_catalog.items);
        let catalog_path = workspaces_dir.join(format!("{company_key}-catalog.json"));
        atomic_write_json(&catalog_path, &offerings, "company catalog").await?;
    }
    Ok(report)
}

fn import_global_catalog(
    state: &OutwardAppState,
    csv_opt: Option<&str>,
    qb_opt: Option<&str>,
) -> Result<IngestReport, AppError> {
    let mut catalog = state.catalog.lock().map_err(|_| {
        AppError::InternalServerError("Customer catalog state lock is poisoned".to_string())
    })?;

    // Parse all supplied formats into a draft catalog and commit only if all succeed
    let mut draft_catalog = catalog.clone();
    let report = ingest_into_catalog(&mut draft_catalog, csv_opt, qb_opt)?;
    *catalog = draft_catalog;
    Ok(report)
}

fn ingest_into_catalog(
    catalog: &mut CustomerCatalog,
    csv_opt: Option<&str>,
    qb_opt: Option<&str>,
) -> Result<IngestReport, AppError> {
    let mut report = IngestReport::default();
    if let Some(csv_data) = csv_opt {
        merge_ingest_report(&mut report, catalog.ingest_csv(csv_data)?);
    }
    if let Some(qb_data) = qb_opt {
        merge_ingest_report(&mut report, catalog.ingest_quickbooks_json(qb_data)?);
    }
    Ok(report)
}

fn merge_ingest_report(target: &mut IngestReport, rep: IngestReport) {
    target.added += rep.added;
    target.updated += rep.updated;
    target.skipped += rep.skipped;
    target.capacity_exceeded |= rep.capacity_exceeded;
    target.errors.extend(rep.errors);
}

fn build_company_offerings(company_key: &str, items: &[CatalogItem]) -> Vec<Offering> {
    let now = chrono::Utc::now();
    let now_str = now.to_rfc3339();
    let fresh_str = (now + chrono::Duration::days(90)).to_rfc3339();

    items
        .iter()
        .enumerate()
        .map(|(idx, item)| Offering {
            id: uuid::Uuid::new_v4().to_string(),
            display_reference: format!("{}-Offering-Cat-{:03}", company_key, idx + 1),
            title: format!("{}-{}", company_key, item.title),
            description: if item.description.is_empty() {
                format!("{} ({})", item.title, item.category)
            } else {
                item.description.clone()
            },
            price_minor: item
                .price_usd
                .map(|p| (p * 100.0).round() as i64)
                .unwrap_or(0),
            currency: "USD".to_string(),
            source_id: format!("{}-Source-Catalog-{:03}", company_key, idx + 1),
            source_updated_at: now_str.clone(),
            fresh_until: fresh_str.clone(),
        })
        .collect()
}

fn format_import_message(report: &IngestReport, has_warnings: bool) -> String {
    if !has_warnings {
        return format!(
            "Successfully processed catalog import ({} added, {} updated, {} skipped)",
            report.added, report.updated, report.skipped
        );
    }
    let cap_suffix = if report.capacity_exceeded {
        ", capacity exceeded"
    } else {
        ""
    };
    format!(
        "Catalog import completed with warnings ({} added, {} updated, {} skipped, {} errors{})",
        report.added,
        report.updated,
        report.skipped,
        report.errors.len(),
        cap_suffix
    )
}

//! @docs ARCHITECTURE:OutwardGateway
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / HTTP Routes / outward_routes
//!
//! ### 🔍 Debugging & Observability
//! - **Telemetry Targets**: `[OutwardRoutes]`

use crate::agent::outward::{CustomerCatalog, IngestReport, ModelProfile, OutwardGateway};
use crate::error::AppError;
use axum::{
    extract::{DefaultBodyLimit, Query, State},
    http::StatusCode,
    response::{IntoResponse, Json, Response},
    routing::{get, post, put},
    Router,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// Maximum distinct IPs tracked concurrently in rate limiter memory
const MAX_TRACKED_IPS: usize = 10_000;

/// Length limits on incoming profile fields to prevent memory bloat
const MAX_BUSINESS_NAME_LEN: usize = 256;
const MAX_DESCRIPTION_LEN: usize = 4096;
const MAX_MODEL_PROFILE_LEN: usize = 64;
const MAX_CONTACT_FIELD_LEN: usize = 1024;
const MAX_INFO_CARD_COUNT: usize = 100;
const MAX_INFO_CARD_TITLE_LEN: usize = 256;
const MAX_INFO_CARD_CONTENT_LEN: usize = 16384;

/// In-memory IP Fixed-Window Counter Rate Limiter (Default: 60 requests per 60-second window per IP)
#[derive(Debug, Clone)]
pub struct IpRateLimiter {
    max_requests: u32,
    window_duration: Duration,
    records: Arc<Mutex<HashMap<String, (u32, Instant)>>>,
}

impl IpRateLimiter {
    pub fn new(max_requests: u32, window_secs: u64) -> Self {
        Self {
            max_requests,
            window_duration: Duration::from_secs(window_secs),
            records: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Check if IP address is allowed to proceed. Returns true if request is permitted.
    pub fn check_and_record(&self, ip: &str) -> bool {
        if self.max_requests == 0 {
            return false;
        }

        let Ok(mut guard) = self.records.lock() else {
            return false;
        };
        let now = Instant::now();

        // Memory DoS Protection: Hard cap with opportunistic cleanup and oldest-entry eviction
        if guard.len() >= MAX_TRACKED_IPS {
            let window = self.window_duration;
            guard.retain(|_, (_, start)| now.duration_since(*start) < window);

            if guard.len() >= MAX_TRACKED_IPS {
                let mut by_age: Vec<(String, Instant)> =
                    guard.iter().map(|(k, (_, s))| (k.clone(), *s)).collect();
                by_age.sort_by_key(|(_, s)| *s);
                let evict_count = (guard.len() / 5).max(1); // evict oldest 20%
                for (key, _) in by_age.into_iter().take(evict_count) {
                    guard.remove(&key);
                }
            }
        }

        if let Some((count, window_start)) = guard.get_mut(ip) {
            if now.duration_since(*window_start) >= self.window_duration {
                *count = 1;
                *window_start = now;
                true
            } else if *count < self.max_requests {
                *count += 1;
                true
            } else {
                false
            }
        } else {
            guard.insert(ip.to_string(), (1, now));
            true
        }
    }
}

/// Persisted FAQ / Policy Info Card for outward knowledge base
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PersistedInfoCard {
    pub id: String,
    pub title: String,
    pub content: String,
    pub category: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub updated_at: String,
}

/// Persisted state for outward company agent card
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistedOutwardState {
    pub business_name: String,
    pub description: String,
    pub model_profile: String,
    pub skills: Vec<crate::agent::outward::A2aSkill>,
    #[serde(default)]
    pub address: Option<String>,
    #[serde(default)]
    pub hours: Option<String>,
    #[serde(default)]
    pub support_email: Option<String>,
    #[serde(default)]
    pub support_phone: Option<String>,
    #[serde(default)]
    pub return_policy: Option<String>,
    #[serde(default)]
    pub info_cards: Vec<PersistedInfoCard>,
}

/// Shared Outward Gateway Service State
#[derive(Clone)]
pub struct OutwardAppState {
    pub gateway: Arc<Mutex<OutwardGateway>>,
    pub catalog: Arc<Mutex<CustomerCatalog>>,
    pub info_cards: Arc<Mutex<Vec<PersistedInfoCard>>>,
    pub rate_limiter: IpRateLimiter,
    pub storage_path: Option<PathBuf>,
}

impl OutwardAppState {
    pub fn new(business_name: &str) -> Self {
        Self::new_with_storage(business_name, None)
    }

    pub fn new_with_storage(business_name: &str, storage_path: Option<PathBuf>) -> Self {
        let port = std::env::var("PORT").unwrap_or_else(|_| "8000".to_string());
        let card_url = std::env::var("TADPOLE_AGENT_CARD_URL").unwrap_or_else(|_| {
            format!("http://localhost:{}/a2a/v1/company-agent-card.json", port)
        });

        let mut gateway = OutwardGateway::new(business_name, card_url);
        let mut catalog = CustomerCatalog::new(business_name);
        let mut info_cards = Vec::new();

        if let Some(ref path) = storage_path {
            if path.exists() {
                match std::fs::read_to_string(path) {
                    Ok(content) => match serde_json::from_str::<PersistedOutwardState>(&content) {
                        Ok(persisted) => {
                            gateway.update_business_profile(
                                &persisted.business_name,
                                &persisted.description,
                            );
                            if let Err(e) = gateway.set_model_profile(&persisted.model_profile) {
                                tracing::warn!(
                                    "[OutwardRoutes] Failed to apply persisted model profile: {}",
                                    e
                                );
                            }
                            gateway.update_skills(persisted.skills);
                            if persisted.address.is_some() || persisted.hours.is_some() {
                                gateway
                                    .update_hours_and_location(persisted.address, persisted.hours);
                            }
                            if persisted.support_email.is_some()
                                || persisted.support_phone.is_some()
                            {
                                gateway.update_support_contact(
                                    persisted.support_email,
                                    persisted.support_phone,
                                );
                            }
                            if let Some(policy) = persisted.return_policy {
                                gateway.update_return_policy(Some(policy));
                            }
                            catalog.business_name = persisted.business_name;
                            catalog.default_model_profile = persisted.model_profile;
                            info_cards = persisted.info_cards;
                            tracing::info!(
                                "[OutwardRoutes] Loaded persisted outward profile from {:?}",
                                path
                            );
                        }
                        Err(e) => {
                            tracing::error!(
                                "[OutwardRoutes] Failed to deserialize outward state file at {:?}: {}. Using default profile.",
                                path,
                                e
                            );
                        }
                    },
                    Err(e) => {
                        tracing::error!(
                            "[OutwardRoutes] Failed to read outward state file at {:?}: {}. Using default profile.",
                            path,
                            e
                        );
                    }
                }
            }
        }

        Self {
            gateway: Arc::new(Mutex::new(gateway)),
            catalog: Arc::new(Mutex::new(catalog)),
            info_cards: Arc::new(Mutex::new(info_cards)),
            rate_limiter: IpRateLimiter::new(60, 60), // 60 requests per minute
            storage_path,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct SearchQueryParams {
    pub q: Option<String>,
    pub limit: Option<usize>,
}

#[derive(Debug, Deserialize)]
pub struct ImportCatalogPayload {
    pub csv_content: Option<String>,
    pub qb_json_content: Option<String>,
    #[serde(default)]
    pub company_key: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateProfilePayload {
    pub business_name: Option<String>,
    pub description: Option<String>,
    pub model_profile: Option<String>,
    pub skills: Option<Vec<crate::agent::outward::A2aSkill>>,
    pub address: Option<String>,
    pub hours: Option<String>,
    pub support_email: Option<String>,
    pub support_phone: Option<String>,
    pub return_policy: Option<String>,
    #[serde(default)]
    pub info_cards: Option<Vec<PersistedInfoCard>>,
}

#[derive(Debug, Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub message: String,
    pub data: Option<T>,
}

/// GET /a2a/v1/company-agent-card.json
/// Returns the public A2A agent card. No bearer token is required; requests are IP rate limited.
/// @docs API_REFERENCE:GetOutwardAgentCard
pub async fn get_agent_card_handler(
    State(state): State<OutwardAppState>,
    req: axum::extract::Request,
) -> Result<Response, AppError> {
    let client_ip = crate::middleware::extract_client_ip(&req);

    if !state.rate_limiter.check_and_record(&client_ip) {
        return Err(AppError::RateLimit(
            "Capped at 60 requests per minute".to_string(),
        ));
    }

    let (card, model_profile) = {
        let gateway = state.gateway.lock().map_err(|_| {
            AppError::InternalServerError("Outward gateway state lock is poisoned".to_string())
        })?;
        (
            gateway.get_agent_card().clone(),
            gateway.get_model_profile().to_string(),
        )
    };
    let info_cards = state
        .info_cards
        .lock()
        .map_err(|_| {
            AppError::InternalServerError("Outward info cards state lock is poisoned".to_string())
        })?
        .clone();

    let mut value = serde_json::to_value(&card).map_err(|e| {
        AppError::InternalServerError(format!("Failed to serialize agent card: {}", e))
    })?;
    if let Some(obj) = value.as_object_mut() {
        obj.insert(
            "model_profile".to_string(),
            serde_json::Value::String(model_profile),
        );
        obj.insert(
            "info_cards".to_string(),
            serde_json::to_value(info_cards).unwrap_or(serde_json::Value::Array(Vec::new())),
        );
    }

    Ok((StatusCode::OK, Json(value)).into_response())
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

    let catalog = state.catalog.lock().map_err(|_| {
        AppError::InternalServerError("Customer catalog state lock is poisoned".to_string())
    })?;
    let query_str = params.q.unwrap_or_default();
    let limit = params.limit.unwrap_or(20).clamp(1, 100);

    let results = catalog.search_catalog(&query_str, limit);

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

/// POST /a2a/v1/catalog/import
/// Validates and atomically imports CSV or QuickBooks JSON payloads up to 512 KiB.
/// @docs API_REFERENCE:ImportOutwardCatalog
pub async fn import_catalog_handler(
    State(state): State<OutwardAppState>,
    Json(payload): Json<ImportCatalogPayload>,
) -> Result<Response, AppError> {
    let max_payload_len = 512 * 1024; // 512KB limit per import payload
    if payload.csv_content.as_ref().map_or(0, |s| s.len()) > max_payload_len
        || payload.qb_json_content.as_ref().map_or(0, |s| s.len()) > max_payload_len
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

    let csv_opt = payload.csv_content;
    let qb_opt = payload.qb_json_content;

    let (combined_report, catalog_items_snapshot) = {
        let mut catalog = state.catalog.lock().map_err(|_| {
            AppError::InternalServerError("Customer catalog state lock is poisoned".to_string())
        })?;

        // F19: Parse all supplied formats into a draft catalog and commit only if all succeed
        let mut draft_catalog = catalog.clone();
        let mut combined_report = IngestReport::default();
        if let Some(ref csv_data) = csv_opt {
            let rep = draft_catalog.ingest_csv(csv_data)?;
            combined_report.added += rep.added;
            combined_report.updated += rep.updated;
            combined_report.skipped += rep.skipped;
            combined_report.capacity_exceeded |= rep.capacity_exceeded;
            combined_report.errors.extend(rep.errors);
        }
        if let Some(ref qb_data) = qb_opt {
            let rep = draft_catalog.ingest_quickbooks_json(qb_data)?;
            combined_report.added += rep.added;
            combined_report.updated += rep.updated;
            combined_report.skipped += rep.skipped;
            combined_report.capacity_exceeded |= rep.capacity_exceeded;
            combined_report.errors.extend(rep.errors);
        }

        // Atomically commit draft catalog only after all formats validate successfully
        let snapshot = draft_catalog.items.clone();
        *catalog = draft_catalog;
        (combined_report, snapshot)
    };

    if let Some(ref company_key) = payload.company_key {
        if matches!(company_key.as_str(), "Mock-1-Product" | "Mock-2-Service") {
            if let Some(ref storage_path) = state.storage_path {
                if let Some(data_dir) = storage_path.parent() {
                    let workspaces_dir = data_dir.join("business").join("workspaces");
                    if let Err(e) = tokio::fs::create_dir_all(&workspaces_dir).await {
                        tracing::warn!(
                            "[OutwardRoutes] Failed to create workspaces directory {:?}: {}",
                            workspaces_dir,
                            e
                        );
                    } else {
                        let now = chrono::Utc::now();
                        let fresh_until = now + chrono::Duration::days(365 * 10);
                        let now_str = now.to_rfc3339();
                        let fresh_str = fresh_until.to_rfc3339();
                        let offerings: Vec<crate::agent::business::types::Offering> =
                            catalog_items_snapshot
                                .iter()
                                .enumerate()
                                .map(|(idx, item)| crate::agent::business::types::Offering {
                                    id: uuid::Uuid::new_v4().to_string(),
                                    display_reference: format!(
                                        "{}-Offering-Cat-{:03}",
                                        company_key,
                                        idx + 1
                                    ),
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
                                    source_id: format!(
                                        "{}-Source-Catalog-{:03}",
                                        company_key,
                                        idx + 1
                                    ),
                                    source_updated_at: now_str.clone(),
                                    fresh_until: fresh_str.clone(),
                                })
                                .collect();
                        if let Ok(json_str) = serde_json::to_string_pretty(&offerings) {
                            let catalog_path =
                                workspaces_dir.join(format!("{}-catalog.json", company_key));
                            if let Err(e) =
                                tokio::fs::write(&catalog_path, json_str.as_bytes()).await
                            {
                                tracing::warn!(
                                    "[OutwardRoutes] Failed to export workspace catalog to {:?}: {}",
                                    catalog_path,
                                    e
                                );
                            }
                        }
                    }
                }
            }
        }
    }

    let has_warnings = !combined_report.errors.is_empty() || combined_report.capacity_exceeded;
    let success = !has_warnings;
    let message = if has_warnings {
        format!(
            "Catalog import completed with warnings ({} added, {} updated, {} skipped, {} errors{})",
            combined_report.added,
            combined_report.updated,
            combined_report.skipped,
            combined_report.errors.len(),
            if combined_report.capacity_exceeded {
                ", capacity exceeded"
            } else {
                ""
            }
        )
    } else {
        format!(
            "Successfully processed catalog import ({} added, {} updated, {} skipped)",
            combined_report.added, combined_report.updated, combined_report.skipped
        )
    };

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
            success,
            message,
            data: Some(combined_report),
        }),
    )
        .into_response())
}

/// PUT /a2a/v1/profile
/// Updates the outward business profile, model profile, advertised skills, and persisted FAQ/Policy info_cards.
/// @docs API_REFERENCE:UpdateOutwardProfile
pub async fn update_profile_handler(
    State(state): State<OutwardAppState>,
    Json(payload): Json<UpdateProfilePayload>,
) -> Result<Response, AppError> {
    // 1. Validate field boundaries upfront
    if let Some(ref name) = payload.business_name {
        if name.len() > MAX_BUSINESS_NAME_LEN {
            return Err(AppError::BadRequest(format!(
                "business_name exceeds maximum length of {} characters",
                MAX_BUSINESS_NAME_LEN
            )));
        }
    }
    if let Some(ref desc) = payload.description {
        if desc.len() > MAX_DESCRIPTION_LEN {
            return Err(AppError::BadRequest(format!(
                "description exceeds maximum length of {} characters",
                MAX_DESCRIPTION_LEN
            )));
        }
    }
    if let Some(ref mprofile) = payload.model_profile {
        if mprofile.len() > MAX_MODEL_PROFILE_LEN {
            return Err(AppError::BadRequest(format!(
                "model_profile exceeds maximum length of {} characters",
                MAX_MODEL_PROFILE_LEN
            )));
        }
        ModelProfile::parse(mprofile)?;
    }
    for (field_name, val) in [
        ("address", &payload.address),
        ("hours", &payload.hours),
        ("support_email", &payload.support_email),
        ("support_phone", &payload.support_phone),
        ("return_policy", &payload.return_policy),
    ] {
        if let Some(v) = val {
            if v.len() > MAX_CONTACT_FIELD_LEN {
                return Err(AppError::BadRequest(format!(
                    "{} exceeds maximum length of {} characters",
                    field_name, MAX_CONTACT_FIELD_LEN
                )));
            }
        }
    }
    if let Some(cards) = &payload.info_cards {
        if cards.len() > MAX_INFO_CARD_COUNT {
            return Err(AppError::BadRequest(format!(
                "Cannot persist more than {} outward info cards",
                MAX_INFO_CARD_COUNT
            )));
        }
        for card in cards {
            if card.title.len() > MAX_INFO_CARD_TITLE_LEN {
                return Err(AppError::BadRequest(format!(
                    "info card title exceeds maximum length of {} characters",
                    MAX_INFO_CARD_TITLE_LEN
                )));
            }
            if card.content.len() > MAX_INFO_CARD_CONTENT_LEN {
                return Err(AppError::BadRequest(format!(
                    "info card content exceeds maximum length of {} characters",
                    MAX_INFO_CARD_CONTENT_LEN
                )));
            }
        }
    }

    // 2. Server-side stamp updated_at for info cards if omitted
    let stamped_info_cards = payload.info_cards.map(|cards| {
        let now_str = chrono::Utc::now().to_rfc3339();
        cards
            .into_iter()
            .map(|mut card| {
                if card.updated_at.is_empty() {
                    card.updated_at = now_str.clone();
                }
                card
            })
            .collect::<Vec<_>>()
    });

    // 3. Read current state under lock and compute the single source of truth merged state
    let (current_state, current_cards) = {
        let gw = state.gateway.lock().map_err(|_| {
            AppError::InternalServerError("Outward gateway state lock is poisoned".to_string())
        })?;
        let prof = gw.profile();
        let card = gw.get_agent_card();
        let cards = state.info_cards.lock().map_err(|_| {
            AppError::InternalServerError("Outward info cards state lock is poisoned".to_string())
        })?;
        (
            PersistedOutwardState {
                business_name: prof.name.clone(),
                description: prof.description.clone(),
                model_profile: gw.get_model_profile().to_string(),
                skills: card.skills.clone(),
                address: prof.address.clone(),
                hours: prof.hours.clone(),
                support_email: prof.support_email.clone(),
                support_phone: prof.support_phone.clone(),
                return_policy: prof.return_policy.clone(),
                info_cards: cards.clone(),
            },
            cards.clone(),
        )
    };

    let new_name = payload.business_name.unwrap_or(current_state.business_name);
    let new_desc = payload.description.unwrap_or(current_state.description);
    let new_model = payload.model_profile.unwrap_or(current_state.model_profile);
    let new_skills = payload.skills.unwrap_or(current_state.skills);
    let new_addr = payload.address.or(current_state.address);
    let new_hours = payload.hours.or(current_state.hours);
    let new_email = payload.support_email.or(current_state.support_email);
    let new_phone = payload.support_phone.or(current_state.support_phone);
    let new_policy = payload.return_policy.or(current_state.return_policy);
    let new_info_cards = stamped_info_cards.unwrap_or(current_cards);

    let merged_state = PersistedOutwardState {
        business_name: new_name,
        description: new_desc,
        model_profile: new_model,
        skills: new_skills,
        address: new_addr,
        hours: new_hours,
        support_email: new_email,
        support_phone: new_phone,
        return_policy: new_policy,
        info_cards: new_info_cards,
    };

    // 4. Persist to disk atomically using temporary file + rename with cleanup on error
    if let Some(ref storage_path) = state.storage_path {
        if let Some(parent) = storage_path.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .map_err(AppError::Io)?;
        }
        let json_str = serde_json::to_string_pretty(&merged_state).map_err(|e| {
            AppError::InternalServerError(format!("Failed to serialize outward profile: {}", e))
        })?;

        let tmp_path = storage_path.with_extension(format!("tmp.{}", uuid::Uuid::new_v4()));
        if let Err(e) = tokio::fs::write(&tmp_path, json_str.as_bytes()).await {
            let _ = tokio::fs::remove_file(&tmp_path).await;
            return Err(AppError::Io(e));
        }
        if let Err(e) = tokio::fs::rename(&tmp_path, storage_path).await {
            let _ = tokio::fs::remove_file(&tmp_path).await;
            return Err(AppError::Io(e));
        }
    }

    // 5. Commit merged state into memory under locks
    {
        let mut stored_cards = state.info_cards.lock().map_err(|_| {
            AppError::InternalServerError("Outward info cards state lock is poisoned".to_string())
        })?;
        *stored_cards = merged_state.info_cards.clone();
    }

    let card = {
        let mut gateway = state.gateway.lock().map_err(|_| {
            AppError::InternalServerError("Outward gateway state lock is poisoned".to_string())
        })?;

        gateway.update_business_profile(&merged_state.business_name, &merged_state.description);
        gateway.update_hours_and_location(merged_state.address.clone(), merged_state.hours.clone());
        gateway.update_support_contact(
            merged_state.support_email.clone(),
            merged_state.support_phone.clone(),
        );
        gateway.update_return_policy(merged_state.return_policy.clone());
        gateway.set_model_profile(&merged_state.model_profile)?;
        gateway.update_skills(merged_state.skills.clone());

        gateway.get_agent_card().clone()
    };

    {
        let mut catalog = state.catalog.lock().map_err(|_| {
            AppError::InternalServerError("Customer catalog state lock is poisoned".to_string())
        })?;
        catalog.business_name = merged_state.business_name.clone();
        catalog.default_model_profile = merged_state.model_profile.clone();
    }

    Ok((
        StatusCode::OK,
        Json(ApiResponse {
            success: true,
            message: "Outward Gateway profile updated successfully".to_string(),
            data: Some(card),
        }),
    )
        .into_response())
}

/// Public discovery routes: `/a2a/v1/company-agent-card.json`, `/a2a/v1/catalog/search` (IP Rate Limited)
pub fn public_outward_router(app_state: OutwardAppState) -> Router {
    Router::new()
        .route(
            "/a2a/v1/company-agent-card.json",
            get(get_agent_card_handler),
        )
        .route("/a2a/v1/catalog/search", get(search_catalog_handler))
        .layer(DefaultBodyLimit::max(256 * 1024))
        .with_state(app_state)
}

/// Administrative profile mutation & ingestion routes: `/a2a/v1/catalog/import`, `/a2a/v1/profile`
fn protected_outward_router(app_state: OutwardAppState) -> Router {
    Router::new()
        .route("/a2a/v1/catalog/import", post(import_catalog_handler))
        .route("/a2a/v1/profile", put(update_profile_handler))
        .layer(DefaultBodyLimit::max(1024 * 1024))
        .with_state(app_state)
}

/// Combined outward routes. Discovery remains public; administrative routes require a valid bearer token.
pub fn outward_router(
    app_state: OutwardAppState,
    auth_state: Arc<crate::state::AppState>,
) -> Router {
    let protected = protected_outward_router(app_state.clone()).route_layer(
        axum::middleware::from_fn_with_state(auth_state, crate::middleware::auth::validate_token),
    );
    public_outward_router(app_state).merge(protected)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ip_rate_limiter_enforcement() {
        let limiter = IpRateLimiter::new(3, 60); // Allow max 3 requests
        let ip = "10.0.0.1";

        assert!(limiter.check_and_record(ip)); // 1
        assert!(limiter.check_and_record(ip)); // 2
        assert!(limiter.check_and_record(ip)); // 3
        assert!(!limiter.check_and_record(ip)); // 4th blocked
    }

    #[test]
    fn test_ip_rate_limiter_zero_limit() {
        let limiter = IpRateLimiter::new(0, 60);
        assert!(!limiter.check_and_record("127.0.0.1"));
    }

    #[test]
    fn test_ip_rate_limiter_memory_cleanup_and_hard_cap() {
        let limiter = IpRateLimiter::new(60, 60);
        // Insert 12,000 distinct IPs into rate limiter map
        for i in 0..12_000 {
            limiter.check_and_record(&format!("192.168.{}.{}", i / 256, i % 256));
        }

        // Memory DoS Protection: map must strictly not exceed MAX_TRACKED_IPS
        let map_len = limiter.records.lock().unwrap().len();
        assert!(
            map_len <= MAX_TRACKED_IPS,
            "Limiter map len {} exceeded MAX_TRACKED_IPS {}",
            map_len,
            MAX_TRACKED_IPS
        );
    }

    #[tokio::test]
    async fn test_import_rejects_malformed_payload_without_catalog_mutation() {
        let state = OutwardAppState::new("Import Validation Test");
        let result = import_catalog_handler(
            State(state.clone()),
            Json(ImportCatalogPayload {
                csv_content: None,
                qb_json_content: Some("{not-json}".to_string()),
                company_key: None,
            }),
        )
        .await;

        assert!(matches!(result, Err(AppError::BadRequest(_))));
        assert!(state.catalog.lock().unwrap().items.is_empty());
    }

    #[tokio::test]
    async fn test_profile_persistence_and_reload() {
        let temp_file =
            std::env::temp_dir().join(format!("test_outward_{}.json", uuid::Uuid::new_v4()));
        let state = OutwardAppState::new_with_storage("Initial SMB", Some(temp_file.clone()));

        let payload = UpdateProfilePayload {
            business_name: Some("Updated SMB Hardware".to_string()),
            description: Some("Durable description".to_string()),
            model_profile: Some("gemma4:e4b".to_string()),
            skills: None,
            address: Some("42 Harbor Road".to_string()),
            hours: Some("Mon-Sat 8-8".to_string()),
            support_email: Some("help@smb.io".to_string()),
            support_phone: None,
            return_policy: Some("30 days refund".to_string()),
            info_cards: Some(vec![PersistedInfoCard {
                id: "card-1".to_string(),
                title: "Return Policy".to_string(),
                content: "30 days refund".to_string(),
                category: "policy".to_string(),
                tags: vec!["returns".to_string()],
                updated_at: "2026-04-12T10:00:00Z".to_string(),
            }]),
        };

        let res = update_profile_handler(State(state), Json(payload)).await;
        assert!(res.is_ok());
        assert!(temp_file.exists());

        // Reload fresh state from disk
        let reloaded =
            OutwardAppState::new_with_storage("Default Fallback", Some(temp_file.clone()));
        let gateway = reloaded.gateway.lock().unwrap();
        assert_eq!(gateway.get_agent_card().name, "Updated SMB Hardware");
        assert_eq!(gateway.profile().description, "Durable description");
        assert_eq!(gateway.profile().address.as_deref(), Some("42 Harbor Road"));
        assert_eq!(gateway.profile().hours.as_deref(), Some("Mon-Sat 8-8"));
        assert_eq!(
            gateway.profile().support_email.as_deref(),
            Some("help@smb.io")
        );
        let cards = reloaded.info_cards.lock().unwrap();
        assert_eq!(cards.len(), 1);
        assert_eq!(cards[0].title, "Return Policy");

        // Cleanup
        let _ = std::fs::remove_file(temp_file);
    }

    #[tokio::test]
    async fn test_profile_partial_update_preserves_merged_fields_in_memory_and_disk() {
        let temp_file =
            std::env::temp_dir().join(format!("test_outward_merge_{}.json", uuid::Uuid::new_v4()));
        let state = OutwardAppState::new_with_storage("Merge SMB", Some(temp_file.clone()));

        // Step 1: Initial full setup
        let init_payload = UpdateProfilePayload {
            business_name: Some("Merge Test Corp".to_string()),
            description: Some("Base description".to_string()),
            model_profile: Some("gemma4:e4b".to_string()),
            skills: None,
            address: Some("123 Main Street".to_string()),
            hours: Some("9am - 5pm".to_string()),
            support_email: Some("contact@mergetest.com".to_string()),
            support_phone: Some("+1-555-0199".to_string()),
            return_policy: Some("14 days".to_string()),
            info_cards: None,
        };
        let res1 = update_profile_handler(State(state.clone()), Json(init_payload)).await;
        assert!(res1.is_ok());

        // Step 2: Partial update — only provide support_email and address (hours and phone are None)
        let partial_payload = UpdateProfilePayload {
            business_name: None,
            description: None,
            model_profile: None,
            skills: None,
            address: Some("456 New Road".to_string()),
            hours: None, // Must preserve "9am - 5pm" in memory and disk!
            support_email: Some("newemail@mergetest.com".to_string()),
            support_phone: None, // Must preserve "+1-555-0199" in memory and disk!
            return_policy: None,
            info_cards: None,
        };
        let res2 = update_profile_handler(State(state.clone()), Json(partial_payload)).await;
        assert!(res2.is_ok());

        // Verify In-Memory State preserves previously set hours and support_phone
        {
            let gw = state.gateway.lock().unwrap();
            let prof = gw.profile();
            assert_eq!(prof.address.as_deref(), Some("456 New Road"));
            assert_eq!(
                prof.hours.as_deref(),
                Some("9am - 5pm"),
                "In-memory hours must not be wiped out by partial address update"
            );
            assert_eq!(
                prof.support_email.as_deref(),
                Some("newemail@mergetest.com")
            );
            assert_eq!(
                prof.support_phone.as_deref(),
                Some("+1-555-0199"),
                "In-memory phone must not be wiped out by partial email update"
            );
        }

        // Verify Reloaded Disk State also preserves hours and support_phone
        let reloaded = OutwardAppState::new_with_storage("Fallback", Some(temp_file.clone()));
        {
            let gw = reloaded.gateway.lock().unwrap();
            let prof = gw.profile();
            assert_eq!(prof.address.as_deref(), Some("456 New Road"));
            assert_eq!(prof.hours.as_deref(), Some("9am - 5pm"));
            assert_eq!(
                prof.support_email.as_deref(),
                Some("newemail@mergetest.com")
            );
            assert_eq!(prof.support_phone.as_deref(), Some("+1-555-0199"));
        }

        // Cleanup
        let _ = std::fs::remove_file(temp_file);
    }

    #[tokio::test]
    async fn test_update_profile_rejects_oversized_fields() {
        let state = OutwardAppState::new("Oversized Validation SMB");

        // 1. Oversized business name (> 256)
        let bad_name_payload = UpdateProfilePayload {
            business_name: Some("A".repeat(300)),
            description: None,
            model_profile: None,
            skills: None,
            address: None,
            hours: None,
            support_email: None,
            support_phone: None,
            return_policy: None,
            info_cards: None,
        };
        let res = update_profile_handler(State(state.clone()), Json(bad_name_payload)).await;
        assert!(matches!(res, Err(AppError::BadRequest(_))));

        // 2. Oversized info card content (> 16384)
        let bad_card_payload = UpdateProfilePayload {
            business_name: None,
            description: None,
            model_profile: None,
            skills: None,
            address: None,
            hours: None,
            support_email: None,
            support_phone: None,
            return_policy: None,
            info_cards: Some(vec![PersistedInfoCard {
                id: "card-huge".to_string(),
                title: "Valid Title".to_string(),
                content: "X".repeat(20_000),
                category: "policy".to_string(),
                tags: vec![],
                updated_at: "".to_string(),
            }]),
        };
        let res2 = update_profile_handler(State(state), Json(bad_card_payload)).await;
        assert!(matches!(res2, Err(AppError::BadRequest(_))));
    }

    #[tokio::test]
    async fn test_import_response_honesty_on_ingest_errors() {
        let state = OutwardAppState::new("Honesty Test SMB");
        // A CSV with a header and an invalid row (price is not numeric)
        let csv_with_bad_price = "title,category,price,description\nGood Item,General,19.99,Desc\nBad Item,General,not-a-number,Desc\n";

        let payload = ImportCatalogPayload {
            csv_content: Some(csv_with_bad_price.to_string()),
            qb_json_content: None,
            company_key: None,
        };
        let res = import_catalog_handler(State(state), Json(payload)).await;
        assert!(res.is_ok());
        let response = res.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }
}

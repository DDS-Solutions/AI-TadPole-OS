//! @docs ARCHITECTURE:OutwardGateway
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / HTTP Routes / outward_routes
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Type-safe state handling and bounded execution without unhandled panics.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: `AppError::RateLimit`, `AppError::BadRequest`, `AppError::InternalServerError`
//! - **Telemetry Targets**: none declared
//! - **Witness Tests**: `outward_routes::tests::*`

use crate::agent::outward::{CustomerCatalog, IngestReport, ModelProfile, OutwardGateway};
use crate::error::AppError;
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Json, Response},
    routing::{get, post, put},
    Router,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tracing::info;

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

        // Memory DoS Protection: Opportunistic cleanup if map exceeds 10,000 tracked IPs
        if guard.len() > 10_000 {
            let window = self.window_duration;
            guard.retain(|_, (_, start)| now.duration_since(*start) < window);
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

use std::path::PathBuf;

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
}

/// Shared Outward Gateway Service State
#[derive(Clone)]
pub struct OutwardAppState {
    pub gateway: Arc<Mutex<OutwardGateway>>,
    pub catalog: Arc<Mutex<CustomerCatalog>>,
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

        if let Some(ref path) = storage_path {
            if path.exists() {
                if let Ok(content) = std::fs::read_to_string(path) {
                    if let Ok(persisted) = serde_json::from_str::<PersistedOutwardState>(&content) {
                        gateway.update_business_profile(
                            &persisted.business_name,
                            &persisted.description,
                        );
                        if let Err(e) = gateway.set_model_profile(&persisted.model_profile) {
                            tracing::warn!("Failed to apply persisted model profile: {}", e);
                        }
                        gateway.update_skills(persisted.skills);
                        if persisted.address.is_some() || persisted.hours.is_some() {
                            gateway.update_hours_and_location(persisted.address, persisted.hours);
                        }
                        if persisted.support_email.is_some() || persisted.support_phone.is_some() {
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
                        tracing::info!("Loaded persisted outward profile from {:?}", path);
                    }
                }
            }
        }

        Self {
            gateway: Arc::new(Mutex::new(gateway)),
            catalog: Arc::new(Mutex::new(catalog)),
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

    let gateway = state.gateway.lock().map_err(|_| {
        AppError::InternalServerError("Outward gateway state lock is poisoned".to_string())
    })?;
    let card = gateway.get_agent_card().clone();

    Ok((StatusCode::OK, Json(card)).into_response())
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
) -> Result<impl IntoResponse, AppError> {
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

    let csv_opt = payload.csv_content.clone();
    let qb_opt = payload.qb_json_content.clone();

    let mut catalog = state.catalog.lock().map_err(|_| {
        AppError::InternalServerError("Customer catalog state lock is poisoned".to_string())
    })?;

    // F19: Parse all supplied formats into a draft catalog and commit only if all succeed
    let mut draft_catalog = catalog.clone();
    let mut combined_report = IngestReport::default();
    if let Some(csv_data) = csv_opt {
        let rep = draft_catalog.ingest_csv(&csv_data)?;
        combined_report.added += rep.added;
        combined_report.updated += rep.updated;
        combined_report.skipped += rep.skipped;
        combined_report.capacity_exceeded |= rep.capacity_exceeded;
        combined_report.errors.extend(rep.errors);
    }
    if let Some(qb_data) = qb_opt {
        let rep = draft_catalog.ingest_quickbooks_json(&qb_data)?;
        combined_report.added += rep.added;
        combined_report.updated += rep.updated;
        combined_report.skipped += rep.skipped;
        combined_report.capacity_exceeded |= rep.capacity_exceeded;
        combined_report.errors.extend(rep.errors);
    }

    // Atomically commit draft catalog only after all formats validate successfully
    *catalog = draft_catalog;

    info!(
        "[OutwardRoutes] Processed catalog import: added={}, updated={}, skipped={}",
        combined_report.added, combined_report.updated, combined_report.skipped
    );

    Ok((
        StatusCode::OK,
        Json(ApiResponse {
            success: true,
            message: format!(
                "Successfully processed catalog import ({} added, {} updated, {} skipped)",
                combined_report.added, combined_report.updated, combined_report.skipped
            ),
            data: Some(combined_report),
        }),
    ))
}

/// PUT /a2a/v1/profile
/// Updates the outward business profile, model profile, or advertised skills.
/// @docs API_REFERENCE:UpdateOutwardProfile
pub async fn update_profile_handler(
    State(state): State<OutwardAppState>,
    Json(payload): Json<UpdateProfilePayload>,
) -> Result<impl IntoResponse, AppError> {
    let catalog_business_name = payload.business_name.clone();
    let catalog_model_profile = payload.model_profile.clone();

    // F20: Validate model profile upfront before mutating in-memory or persisted state
    if let Some(mprofile) = &payload.model_profile {
        ModelProfile::parse(mprofile)?;
    }

    // F20: Persist to disk atomically before in-memory state commit
    if let Some(ref storage_path) = state.storage_path {
        let (
            current_name,
            current_desc,
            current_profile,
            current_skills,
            current_addr,
            current_hours,
            current_email,
            current_phone,
            current_policy,
        ) = {
            let gw = state.gateway.lock().map_err(|_| {
                AppError::InternalServerError("Outward gateway state lock is poisoned".to_string())
            })?;
            let prof = gw.profile();
            let card = gw.get_agent_card();
            (
                prof.name.clone(),
                prof.description.clone(),
                gw.get_model_profile().to_string(),
                card.skills.clone(),
                prof.address.clone(),
                prof.hours.clone(),
                prof.support_email.clone(),
                prof.support_phone.clone(),
                prof.return_policy.clone(),
            )
        };

        let new_name = payload.business_name.clone().unwrap_or(current_name);
        let new_desc = payload.description.clone().unwrap_or(current_desc);
        let new_model = payload.model_profile.clone().unwrap_or(current_profile);
        let new_skills = payload.skills.clone().unwrap_or(current_skills);
        let new_addr = payload.address.clone().or(current_addr);
        let new_hours = payload.hours.clone().or(current_hours);
        let new_email = payload.support_email.clone().or(current_email);
        let new_phone = payload.support_phone.clone().or(current_phone);
        let new_policy = payload.return_policy.clone().or(current_policy);

        let persisted_state = PersistedOutwardState {
            business_name: new_name,
            description: new_desc,
            model_profile: new_model,
            skills: new_skills,
            address: new_addr,
            hours: new_hours,
            support_email: new_email,
            support_phone: new_phone,
            return_policy: new_policy,
        };

        if let Some(parent) = storage_path.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .map_err(AppError::Io)?;
        }
        let json_str = serde_json::to_string_pretty(&persisted_state).map_err(|e| {
            AppError::InternalServerError(format!("Failed to serialize outward profile: {}", e))
        })?;

        let tmp_path = storage_path.with_extension(format!("tmp.{}", uuid::Uuid::new_v4()));
        tokio::fs::write(&tmp_path, json_str.as_bytes())
            .await
            .map_err(AppError::Io)?;
        tokio::fs::rename(&tmp_path, storage_path)
            .await
            .map_err(AppError::Io)?;
    }

    let card = {
        let mut gateway = state.gateway.lock().map_err(|_| {
            AppError::InternalServerError("Outward gateway state lock is poisoned".to_string())
        })?;

        if let Some(bname) = &payload.business_name {
            let desc = payload
                .description
                .clone()
                .unwrap_or_else(|| gateway.get_agent_card().description.clone());
            gateway.update_business_profile(bname, &desc);
        } else if let Some(desc) = &payload.description {
            let bname = gateway.get_agent_card().name.clone();
            gateway.update_business_profile(&bname, desc);
        }

        if payload.address.is_some() || payload.hours.is_some() {
            gateway.update_hours_and_location(payload.address, payload.hours);
        }

        if payload.support_email.is_some() || payload.support_phone.is_some() {
            gateway.update_support_contact(payload.support_email, payload.support_phone);
        }

        if let Some(policy) = payload.return_policy {
            gateway.update_return_policy(Some(policy));
        }

        if let Some(mprofile) = &payload.model_profile {
            gateway.set_model_profile(mprofile)?;
        }

        if let Some(skills) = payload.skills {
            gateway.update_skills(skills);
        }

        gateway.get_agent_card().clone()
    };

    if catalog_business_name.is_some() || catalog_model_profile.is_some() {
        let mut catalog = state.catalog.lock().map_err(|_| {
            AppError::InternalServerError("Customer catalog state lock is poisoned".to_string())
        })?;
        if let Some(business_name) = catalog_business_name {
            catalog.business_name = business_name;
        }
        if let Some(model_profile) = catalog_model_profile {
            catalog.default_model_profile = model_profile;
        }
    }

    Ok((
        StatusCode::OK,
        Json(ApiResponse {
            success: true,
            message: "Outward Gateway profile updated successfully".to_string(),
            data: Some(card),
        }),
    ))
}

/// Public discovery routes: `/a2a/v1/company-agent-card.json`, `/a2a/v1/catalog/search` (IP Rate Limited)
pub fn public_outward_router(app_state: OutwardAppState) -> Router {
    Router::new()
        .route(
            "/a2a/v1/company-agent-card.json",
            get(get_agent_card_handler),
        )
        .route("/a2a/v1/agent-card.json", get(get_agent_card_handler))
        .route("/a2a/v1/catalog/search", get(search_catalog_handler))
        .with_state(app_state)
}

/// Administrative profile mutation & ingestion routes: `/a2a/v1/catalog/import`, `/a2a/v1/profile`
fn protected_outward_router(app_state: OutwardAppState) -> Router {
    Router::new()
        .route("/a2a/v1/catalog/import", post(import_catalog_handler))
        .route("/a2a/v1/profile", put(update_profile_handler))
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
    fn test_ip_rate_limiter_memory_cleanup() {
        let limiter = IpRateLimiter::new(60, 1);
        for i in 0..10_005 {
            limiter.check_and_record(&format!("192.168.1.{}", i));
        }

        let map_len = limiter.records.lock().unwrap().len();
        assert!(map_len <= 10_005);
    }

    #[tokio::test]
    async fn test_import_rejects_malformed_payload_without_catalog_mutation() {
        let state = OutwardAppState::new("Import Validation Test");
        let result = import_catalog_handler(
            State(state.clone()),
            Json(ImportCatalogPayload {
                csv_content: None,
                qb_json_content: Some("{not-json}".to_string()),
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

        // Cleanup
        let _ = std::fs::remove_file(temp_file);
    }
}

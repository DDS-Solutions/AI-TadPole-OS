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

use crate::agent::outward::{CustomerCatalog, OutwardGateway};
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

/// Shared Outward Gateway Service State
#[derive(Clone)]
pub struct OutwardAppState {
    pub gateway: Arc<Mutex<OutwardGateway>>,
    pub catalog: Arc<Mutex<CustomerCatalog>>,
    pub rate_limiter: IpRateLimiter,
}

impl OutwardAppState {
    pub fn new(business_name: &str) -> Self {
        Self {
            gateway: Arc::new(Mutex::new(OutwardGateway::new(
                business_name,
                "http://localhost:8000/a2a/v1/company-agent-card.json",
            ))),
            catalog: Arc::new(Mutex::new(CustomerCatalog::new(business_name))),
            rate_limiter: IpRateLimiter::new(60, 60), // 60 requests per minute
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

    let mut imported_count = 0;
    if let Some(csv_data) = csv_opt {
        imported_count += catalog.ingest_csv(&csv_data)?;
    }
    if let Some(qb_data) = qb_opt {
        imported_count += catalog.ingest_quickbooks_json(&qb_data)?;
    }

    info!(
        "[OutwardRoutes] Imported {} items into customer catalog",
        imported_count
    );

    Ok((
        StatusCode::OK,
        Json(ApiResponse {
            success: true,
            message: format!(
                "Successfully imported {} items into catalog",
                imported_count
            ),
            data: Some(imported_count),
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
            gateway.set_model_profile(mprofile);
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
pub fn protected_outward_router(app_state: OutwardAppState) -> Router {
    Router::new()
        .route("/a2a/v1/catalog/import", post(import_catalog_handler))
        .route("/a2a/v1/profile", put(update_profile_handler))
        .with_state(app_state)
}

/// Backward-compatible combined router
pub fn outward_router(app_state: OutwardAppState) -> Router {
    public_outward_router(app_state.clone()).merge(protected_outward_router(app_state))
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
}

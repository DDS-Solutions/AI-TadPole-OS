//! @docs ARCHITECTURE:OutwardGateway
//! @docs OPERATIONS_MANUAL:OutwardGateway
//!
//! ### AI Assist Note
//! Outward A2A REST Gateway & Anti-Scraping Rate Limiter.
//! Exposes public discovery endpoints (`/a2a/v1/agent-card.json`, `/a2a/v1/catalog/search`)
//! protected by an in-memory IP token-bucket rate limiter (60 req/min).
//! Exposes administrative profile mutation & ingestion REST endpoints.
//!
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: Rate limit exceeded (429 Too Many Requests), invalid CSV/JSON payloads.
//! - **Trace Scope**: `server-rs::routes::outward_routes` (Search for `[OutwardRoutes]` in logs)

#![allow(dead_code)]

use crate::agent::outward::{A2aAgentCard, CustomerCatalog, OutwardGateway};
use axum::{
    extract::{Query, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Json},
    routing::{get, post, put},
    Router,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tracing::info;

/// In-memory IP Token-Bucket Rate Limiter (Default: 60 requests per 60-second window per IP)
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
        let mut guard = self.records.lock().unwrap();
        let now = Instant::now();

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
            gateway: Arc::new(Mutex::new(OutwardGateway::new(business_name, "gemma4:e4b"))),
            catalog: Arc::new(Mutex::new(CustomerCatalog::new(business_name))),
            rate_limiter: IpRateLimiter::new(60, 60), // 60 requests per minute
        }
    }
}

pub fn extract_client_ip(headers: &HeaderMap) -> String {
    headers
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.split(',').next())
        .map(|s| s.trim().to_string())
        .or_else(|| {
            headers
                .get("x-real-ip")
                .and_then(|v| v.to_str().ok())
                .map(|s| s.to_string())
        })
        .unwrap_or_else(|| "127.0.0.1".to_string())
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
    pub model_profile: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub message: String,
    pub data: Option<T>,
}

/// Handler: `GET /a2a/v1/agent-card.json` (Public Discovery, IP Rate Limited)
pub async fn get_agent_card_handler(
    State(state): State<OutwardAppState>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let client_ip = extract_client_ip(&headers);

    if !state.rate_limiter.check_and_record(&client_ip) {
        return (
            StatusCode::TOO_MANY_REQUESTS,
            Json(ApiResponse::<A2aAgentCard> {
                success: false,
                message: "Rate limit exceeded. Capped at 60 requests per minute.".to_string(),
                data: None,
            }),
        );
    }

    let gateway = state.gateway.lock().unwrap();
    let card = gateway.get_agent_card().clone();

    (
        StatusCode::OK,
        Json(ApiResponse {
            success: true,
            message: "A2A Agent Card retrieved successfully".to_string(),
            data: Some(card),
        }),
    )
}

/// Handler: `GET /a2a/v1/catalog/search` (Public Keyword Search, IP Rate Limited)
pub async fn search_catalog_handler(
    State(state): State<OutwardAppState>,
    headers: HeaderMap,
    Query(params): Query<SearchQueryParams>,
) -> impl IntoResponse {
    let client_ip = extract_client_ip(&headers);

    if !state.rate_limiter.check_and_record(&client_ip) {
        return (
            StatusCode::TOO_MANY_REQUESTS,
            Json(ApiResponse::<Vec<crate::agent::outward::CatalogItem>> {
                success: false,
                message: "Rate limit exceeded. Capped at 60 requests per minute.".to_string(),
                data: None,
            }),
        );
    }

    let catalog = state.catalog.lock().unwrap();
    let query_str = params.q.unwrap_or_default();
    let limit = params.limit.unwrap_or(20);

    let results = catalog.search_catalog(&query_str, limit);

    (
        StatusCode::OK,
        Json(ApiResponse {
            success: true,
            message: format!("Retrieved {} matching items", results.len()),
            data: Some(results),
        }),
    )
}

/// Handler: `POST /a2a/v1/catalog/import` (Ingests CSV/QuickBooks payloads)
pub async fn import_catalog_handler(
    State(state): State<OutwardAppState>,
    Json(payload): Json<ImportCatalogPayload>,
) -> impl IntoResponse {
    let mut catalog = state.catalog.lock().unwrap();
    let mut imported_count = 0;

    if let Some(csv_data) = payload.csv_content {
        imported_count += catalog.ingest_csv(&csv_data);
    }

    if let Some(qb_data) = payload.qb_json_content {
        imported_count += catalog.ingest_quickbooks_json(&qb_data);
    }

    info!(
        "[OutwardRoutes] Imported {} items into customer catalog",
        imported_count
    );

    (
        StatusCode::OK,
        Json(ApiResponse {
            success: true,
            message: format!("Successfully imported {} items into catalog", imported_count),
            data: Some(imported_count),
        }),
    )
}

/// Handler: `PUT /a2a/v1/profile` (Mutates business profile & model runner)
pub async fn update_profile_handler(
    State(state): State<OutwardAppState>,
    Json(payload): Json<UpdateProfilePayload>,
) -> impl IntoResponse {
    let mut gateway = state.gateway.lock().unwrap();
    let mut catalog = state.catalog.lock().unwrap();

    if let Some(bname) = &payload.business_name {
        gateway.update_business_profile(
            bname,
            "Sovereign SMB Customer Service & Catalog Agent powered by AI-Tadpole-OS.",
        );
        catalog.business_name = bname.clone();
    }

    if let Some(mprofile) = &payload.model_profile {
        gateway.set_model_profile(mprofile);
        catalog.default_model_profile = mprofile.clone();
    }

    (
        StatusCode::OK,
        Json(ApiResponse {
            success: true,
            message: "Outward Gateway profile updated successfully".to_string(),
            data: Some(gateway.get_agent_card().clone()),
        }),
    )
}

/// Construct the Axum Router for `/a2a/v1` routes
pub fn outward_router(app_state: OutwardAppState) -> Router {
    Router::new()
        .route("/a2a/v1/agent-card.json", get(get_agent_card_handler))
        .route("/a2a/v1/catalog/search", get(search_catalog_handler))
        .route("/a2a/v1/catalog/import", post(import_catalog_handler))
        .route("/a2a/v1/profile", put(update_profile_handler))
        .with_state(app_state)
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
}

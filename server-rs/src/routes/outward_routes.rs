//! @docs ARCHITECTURE:OutwardGateway
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / HTTP Routes / outward_routes
//!
//! ### 🔍 Debugging & Observability
//! - **Telemetry Targets**: none declared

mod agent_card;
mod catalog;
mod profile;
mod rate_limit;
mod state;
#[cfg(test)]
mod tests;

use agent_card::get_agent_card_handler;
use catalog::{import_catalog_handler, search_catalog_handler};
use profile::update_profile_handler;
pub use state::OutwardAppState;

use axum::{
    extract::DefaultBodyLimit,
    routing::{get, post, put},
    Router,
};
use std::sync::Arc;

/// Public discovery routes: `/a2a/v1/company-agent-card.json`, `/a2a/v1/catalog/search` (IP Rate Limited).
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

/// Administrative profile mutation & ingestion routes: `/a2a/v1/catalog/import`, `/a2a/v1/profile`.
fn protected_outward_router(app_state: OutwardAppState) -> Router {
    Router::new()
        .route("/a2a/v1/catalog/import", post(import_catalog_handler))
        .route("/a2a/v1/profile", put(update_profile_handler))
        .layer(DefaultBodyLimit::max(1024 * 1024))
        .with_state(app_state)
}

/// Combined outward routes. Discovery remains public; administrative routes require an Admin bearer token.
pub fn outward_router(
    app_state: OutwardAppState,
    auth_state: Arc<crate::state::AppState>,
) -> Router {
    let protected = protected_outward_router(app_state.clone())
        .route_layer(axum::middleware::from_fn(
            crate::middleware::auth::require_admin_middleware,
        ))
        .route_layer(axum::middleware::from_fn_with_state(
            auth_state,
            crate::middleware::auth::validate_token,
        ));
    public_outward_router(app_state).merge(protected)
}

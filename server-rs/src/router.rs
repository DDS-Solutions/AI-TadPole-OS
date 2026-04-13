//! @docs ARCHITECTURE:Networking
//! 
//! ### AI Assist Note
//! **Route Assembly (Gateway Orchestrator)**: Orchestrates the 
//! high-speed mapping of REST and WebSocket handlers for the Tadpole 
//! OS engine. Features **Sovereign Security Policy Enforcement**: 
//! wraps protected `/v1` routes in `NEURAL_TOKEN` validation, 
//! brute-force prevention, and rate-limiting middleware. Implements 
//! **Static Asset Serving & SPA Fallback**: serves the React frontend 
//! from the `dist/` directory with automated `index.html` resolution 
//! for client-side routing. Includes **Feature-Gated Vector Memory**: 
//! routes are conditionally compiled (`vector-memory`) to ensure 
//! compatibility with diverse hardware targets (ROUT-01).
//! 
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: CORS origin mismatches causing frontend XHR 
//!   failures, 401 Unauthorized due to incorrect Bearer tokens, or 
//!   501 Not Implemented for disabled feature-gated memory routes.
//! - **Telemetry Link**: Search for `[Router]` or `[Auth]` in 
//!   `tracing` logs for request-id tracking and permission denials.
//! - **Trace Scope**: `server-rs::router`

use crate::state::AppState;
use crate::{middleware, routes};
use axum::{
    routing::{get, post, put},
    Router,
};
use std::sync::Arc;
use tower_http::services::{ServeDir, ServeFile};

/// Creates and configures the main Axum router, integrating all sub-routes,
/// middlewares (Auth, CORS, State), and static asset servings.
pub fn create_router(app_state: Arc<AppState>) -> Router {
    // 1. Configure CORS
    let cors = middleware::cors::create_cors_layer();

    // 2. Build Protected API Routes
    let protected_routes = build_protected_v1_routes(app_state.clone());

    // 3. Build Engine Routes
    let engine_public = build_engine_public_routes();
    let engine_protected = build_engine_protected_routes(app_state.clone());

    // 4. Combine all /v1 routes and attach API-specific fallback
    let v1_routes = protected_routes
        .merge(engine_public)
        .merge(engine_protected)
        .fallback(not_found_handler);

    // 5. Resolve static file serving path.
    let static_dir = std::env::var("STATIC_DIR").unwrap_or_else(|_| "dist".to_string());

    let mut app = Router::new()
        .nest("/v1", v1_routes)
        .with_state(app_state.clone())
        .layer(axum::middleware::from_fn(
            crate::middleware::auth_rate_limit::auth_brute_force_limiter,
        ))
        .layer(axum::middleware::from_fn(
            middleware::request_id::inject_request_id,
        ))
        .layer(axum::middleware::from_fn(
            middleware::rate_limit::inject_rate_limit_headers,
        ))
        .layer(axum::middleware::from_fn(
            middleware::deprecation::deprecation_middleware,
        ))
        .layer(tower_http::timeout::TimeoutLayer::new(std::time::Duration::from_secs(120)))
        .layer(tower_http::compression::CompressionLayer::new()
            .br(true)
            .gzip(true)
            .zstd(true))
        .layer(cors);


    // Static file serving with proper cache headers
    let static_path = std::path::Path::new(&static_dir);
    if static_path.exists() && static_path.is_dir() {
        tracing::info!("📦 Static file serving enabled from '{}'", static_dir);
        let serve_dir = ServeDir::new(&static_dir)
            .fallback(ServeFile::new(format!("{}/index.html", static_dir)));
        app = app.fallback_service(serve_dir);
    } else {
        tracing::info!(
            "📦 No '{}' directory found — static serving disabled (dev mode)",
            static_dir
        );
    }

    app
}

/// Routes requiring NEURAL_TOKEN authentication
fn build_protected_v1_routes(app_state: Arc<AppState>) -> Router<Arc<AppState>> {
    Router::new()
        .nest("/agents", build_agent_routes())
        .nest("/oversight", build_oversight_routes())
        .nest("/infra", build_infra_routes())
        .nest("/skills", build_skills_routes())
        .nest("/benchmarks", build_benchmark_routes())
        .nest("/continuity", build_continuity_routes())
        .nest("/docs", build_docs_routes())
        .route("/search/memory", build_search_memory_route())
        .route("/env-schema", get(routes::env_schema::get_env_schema))
        .route_layer(axum::middleware::from_fn_with_state(
            app_state,
            middleware::auth::validate_token,
        ))
}

fn build_agent_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/graph", get(routes::agent::get_swarm_graph_handler))
        .route("/", get(routes::agent::get_agents))
        .route("/", post(routes::agent::create_agent))
        .route("/{id}", put(routes::agent::update_agent))
        .route("/{id}/tasks", post(routes::agent::send_task))
        .route("/{id}/reset", post(routes::agent::reset_agent))
        .route("/{id}/pause", post(routes::agent::pause_agent))
        .route("/{id}/resume", post(routes::agent::resume_agent))
        .route("/{id}/mission", post(routes::agent::sync_mission))
        .route("/{agent_id}/memories", build_agent_memory_route())
        .route("/{agent_id}/memories/{row_id}", build_agent_memory_delete_route())
}

fn build_oversight_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/{id}/decide", post(routes::oversight::decide_oversight))
        .route("/pending", get(routes::oversight::get_pending))
        .route("/ledger", get(routes::oversight::get_ledger))
        .route("/settings", put(routes::oversight::update_settings))
        .route("/security/quotas", get(routes::oversight::get_security_quotas))
        .route("/security/quotas/{entity_id}", put(routes::oversight::update_agent_quota))
        .route("/security/missions/quotas", get(routes::oversight::get_mission_quotas))
        .route("/security/missions/{id}/quota", put(routes::oversight::update_mission_quota))
        .route("/security/audit-trail", get(routes::oversight::get_audit_trail))
        .route("/security/health", get(routes::oversight::get_agent_health))
        .route("/security/integrity", get(routes::oversight::get_integrity_status))
        .route("/security/policies", get(routes::oversight::get_policies))
        .route("/security/policies", put(routes::oversight::update_policy))
}

fn build_infra_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/providers", get(routes::model_manager::get_providers))
        .route("/providers/{id}", put(routes::model_manager::update_provider))
        .route("/providers/{id}", axum::routing::delete(routes::model_manager::delete_provider))
        .route("/providers/{id}/test", post(routes::model_manager::test_provider))
        .route("/models", get(routes::model_manager::get_models))
        .route("/models/{id}", put(routes::model_manager::update_model))
        .route("/models/{id}", axum::routing::delete(routes::model_manager::delete_model))
        .route("/model-store/catalog", get(routes::model_manager::get_model_catalog))
        .route("/model-store/pull", post(routes::model_manager::pull_model))
        .route("/nodes", get(routes::nodes::get_nodes))
        .route("/nodes/discover", post(routes::nodes::discover_nodes))
}

fn build_skills_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(routes::skills::list_all_skills))
        .route("/manifests", get(routes::skills::list_manifests))
        .route("/manifests/{name}", get(routes::skills::get_manifest))
        .route("/mcp-tools", get(routes::mcp::list_mcp_tools))
        .route("/import", post(routes::skills::import_capability))
        .route("/register", post(routes::skills::register_capability))
        .route("/proposals", get(routes::skills::list_capability_proposals))
        .route("/proposals/{id}/resolve", post(routes::skills::resolve_capability_proposal))
        .route("/mcp-tools/{name}/execute", post(routes::mcp::execute_mcp_tool))
        .route("/scripts/{name}", put(routes::skills::post_script))
        .route("/scripts/{name}", axum::routing::delete(routes::skills::delete_script))
        .route("/workflows/{name}", put(routes::skills::post_workflow))
        .route("/workflows/{name}", axum::routing::delete(routes::skills::delete_workflow))
        .route("/hooks/{name}", put(routes::skills::post_hook))
        .route("/hooks/{name}", axum::routing::delete(routes::skills::delete_hook))
}

fn build_benchmark_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(routes::benchmarks::get_benchmarks))
        .route("/", post(routes::benchmarks::create_benchmark))
        .route("/run/{test_id}", post(routes::benchmarks::trigger_benchmark))
        .route("/{test_id}", get(routes::benchmarks::get_benchmark_history))
}

fn build_continuity_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/jobs", get(routes::continuity::list_jobs_handler))
        .route("/jobs", post(routes::continuity::create_job_handler))
        .route("/jobs/{id}", get(routes::continuity::get_job_handler))
        .route("/jobs/{id}", put(routes::continuity::update_job_handler))
        .route("/jobs/{id}", axum::routing::delete(routes::continuity::delete_job_handler))
        .route("/jobs/{id}/runs", get(routes::continuity::list_job_runs_handler))
        .route("/workflows", get(routes::continuity::list_workflows_handler))
        .route("/workflows", post(routes::continuity::create_workflow_handler))
        .route("/workflows/{id}/steps", post(routes::continuity::add_workflow_step_handler))
        .route("/workflows/{id}", axum::routing::delete(routes::continuity::delete_workflow_handler))
        .route("/jobs/{id}/enable", post(routes::continuity::enable_job_handler))
        .route("/jobs/{id}/disable", post(routes::continuity::disable_job_handler))
}

fn build_docs_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/knowledge", get(routes::docs::list_knowledge_docs))
        .route("/knowledge/{category}/{name}", get(routes::docs::get_knowledge_doc))
        .route("/operations-manual", get(routes::docs::get_operations_manual))
}

fn build_engine_public_routes() -> Router<Arc<AppState>> {
    Router::new().route("/engine/health", get(routes::health::health_check))
}

fn build_engine_protected_routes(app_state: Arc<AppState>) -> Router<Arc<AppState>> {
    Router::new()
        .route("/engine/deploy", post(routes::deploy::trigger_deploy))
        .route("/engine/kill", post(routes::engine_control::kill_agents))
        .route("/engine/shutdown", post(routes::engine_control::shutdown_engine))
        .route("/engine/ws", get(routes::ws::ws_handler))
        .route("/engine/live-voice", get(routes::ws::live_voice_handler))
        .route("/engine/transcribe", post(routes::audio::transcribe_audio))
        .route("/engine/speak", post(routes::audio::text_to_speech))
        .route("/engine/templates/install", post(routes::templates::install_template))
        .route("/api/pull", post(routes::model_manager::ollama_proxy_pull))
        .route_layer(axum::middleware::from_fn_with_state(
            app_state,
            middleware::auth::validate_token,
        ))
}

// Memory feature gates
fn build_agent_memory_route() -> axum::routing::MethodRouter<Arc<AppState>> {
    #[cfg(feature = "vector-memory")]
    return get(routes::memory::get_agent_memory).post(routes::memory::save_agent_memory);
    #[cfg(not(feature = "vector-memory"))]
    return get(|| async { (axum::http::StatusCode::NOT_IMPLEMENTED, "Vector memory feature disabled") })
        .post(|| async { (axum::http::StatusCode::NOT_IMPLEMENTED, "Vector memory feature disabled") });
}

fn build_agent_memory_delete_route() -> axum::routing::MethodRouter<Arc<AppState>> {
    #[cfg(feature = "vector-memory")]
    return axum::routing::delete(routes::memory::delete_agent_memory);
    #[cfg(not(feature = "vector-memory"))]
    return axum::routing::delete(|| async { (axum::http::StatusCode::NOT_IMPLEMENTED, "Vector memory feature disabled") });
}

fn build_search_memory_route() -> axum::routing::MethodRouter<Arc<AppState>> {
    #[cfg(feature = "vector-memory")]
    return get(routes::memory::global_search);
    #[cfg(not(feature = "vector-memory"))]
    return get(|| async { (axum::http::StatusCode::NOT_IMPLEMENTED, "Vector memory feature disabled") });
}

async fn not_found_handler() -> impl axum::response::IntoResponse {
    let (status, body) = crate::routes::error::ProblemDetails::new(
        axum::http::StatusCode::NOT_FOUND,
        "Not Found",
        "The requested API endpoint does not exist or has been deprecated.".to_string(),
    );
    (status, body)
}

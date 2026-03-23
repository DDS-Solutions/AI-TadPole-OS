//! Defines the application's API surface and middleware pipeline.
//!
//! The router is responsible for mapping REST endpoints and WebSocket handlers
//! to their respective service implementations, while enforcing security through
//! authentication and rate-limiting layers.

use crate::state::AppState;
use crate::{middleware, routes};
use axum::{
    routing::{get, post, put},
    Router,
};
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use tower_http::services::{ServeDir, ServeFile};

/// Creates and configures the main Axum router, integrating all sub-routes, 
/// middlewares (Auth, CORS, State), and static asset servings.
pub fn create_router(app_state: Arc<AppState>) -> Router {
    // 1. Configure CORS
    let mut origins = vec![
        "http://localhost:5173".parse::<axum::http::HeaderValue>().unwrap(),
        "http://10.0.0.1:5173".parse::<axum::http::HeaderValue>().unwrap(),
        "http://localhost:8000".parse::<axum::http::HeaderValue>().unwrap(),
        "http://10.0.0.1:8000".parse::<axum::http::HeaderValue>().unwrap(),
    ];

    let mut cors = CorsLayer::new();

    // SEC-03: Dynamic CORS Origins (e.g. for Bunker/Remote deployments)
    let allow_credentials = if let Ok(allowed) = std::env::var("ALLOWED_ORIGINS") {
        if allowed == "*" {
            // RELAXED MODE: Allow all for troubleshooting legacy hardware
            tracing::warn!("⚠️ CORS RELAXED: Allowing all origins (*)");
            cors = cors.allow_origin(tower_http::cors::Any);
            false // Cannot use credentials with wildcard origin
        } else {
            for origin in allowed.split(',') {
                if let Ok(val) = origin.trim().parse::<axum::http::HeaderValue>() {
                    if !origins.contains(&val) {
                        origins.push(val);
                    }
                }
            }
            cors = cors.allow_origin(origins);
            true
        }
    } else {
        cors = cors.allow_origin(origins);
        true
    };

    let cors = cors
        .allow_methods([
            axum::http::Method::GET,
            axum::http::Method::POST,
            axum::http::Method::PUT,
            axum::http::Method::DELETE,
            axum::http::Method::OPTIONS,
        ])
        .allow_headers([
            axum::http::header::CONTENT_TYPE,
            axum::http::header::AUTHORIZATION,
            axum::http::HeaderName::from_static("x-request-id"),
            axum::http::HeaderName::from_static("traceparent"),
        ])
        .allow_credentials(allow_credentials);

    // 2. Build Protected API Routes
    // These routes require a valid Bearer token via the `validate_token` middleware.
    let protected_routes = Router::new()
        .route("/agents", get(routes::agent::get_agents))
        .route("/agents", post(routes::agent::create_agent))
        .route("/agents/:id/tasks", post(routes::agent::send_task))
        .route("/agents/:id", put(routes::agent::update_agent))
        .route("/agents/:id/reset", post(routes::agent::reset_agent))
        .route("/agents/:id/pause", post(routes::agent::pause_agent))
        .route("/agents/:id/resume", post(routes::agent::resume_agent))
        .route("/agents/:id/mission", post(routes::agent::sync_mission))
        // Agent Memories
        .route(
            "/agents/:agent_id/memories",
            get(routes::memory::get_agent_memory),
        )
        .route(
            "/agents/:agent_id/memories",
            post(routes::memory::save_agent_memory),
        )
        .route(
            "/agents/:agent_id/memories/:row_id",
            axum::routing::delete(routes::memory::delete_agent_memory),
        )
        .route(
            "/search/memory",
            get(routes::memory::global_search),
        )
        .route(
            "/oversight/:id/decide",
            post(routes::oversight::decide_oversight),
        )
        .route("/oversight/pending", get(routes::oversight::get_pending))
        .route("/oversight/ledger", get(routes::oversight::get_ledger))
        .route(
            "/oversight/settings",
            put(routes::oversight::update_settings),
        )
        // Security Dashboard
        .route(
            "/oversight/security/quotas",
            get(routes::oversight::get_security_quotas),
        )
        .route(
            "/oversight/security/quotas/:entity_id",
            put(routes::oversight::update_agent_quota),
        )
        .route(
            "/oversight/security/missions/quotas",
            get(routes::oversight::get_mission_quotas),
        )
        .route(
            "/oversight/security/missions/:id/quota",
            put(routes::oversight::update_mission_quota),
        )
        .route(
            "/oversight/security/audit-trail",
            get(routes::oversight::get_audit_trail),
        )
        .route(
            "/oversight/security/health",
            get(routes::oversight::get_agent_health),
        )
        .route(
            "/oversight/security/integrity",
            get(routes::oversight::get_integrity_status,
        ))
        .route(
            "/infra/providers",
            get(routes::model_manager::get_providers),
        )
        .route(
            "/infra/providers/:id",
            put(routes::model_manager::update_provider),
        )
        .route(
            "/infra/providers/:id",
            axum::routing::delete(routes::model_manager::delete_provider),
        )
        .route(
            "/infra/providers/:id/test",
            post(routes::model_manager::test_provider),
        )
        .route("/infra/models", get(routes::model_manager::get_models))
        .route(
            "/infra/models/:id",
            put(routes::model_manager::update_model),
        )
        .route(
            "/infra/models/:id",
            axum::routing::delete(routes::model_manager::delete_model),
        )
        .route(
            "/infra/model-store/catalog",
            get(routes::model_manager::get_model_catalog),
        )
        .route(
            "/infra/model-store/pull",
            post(routes::model_manager::pull_model),
        )
        .route("/infra/nodes", get(routes::nodes::get_nodes))
        .route("/infra/nodes/discover", post(routes::nodes::discover_nodes))
        // 🧠 Consolidated Skills API (Cognitive Abilities)
        .route("/skills", get(routes::skills::list_all_skills))
        .route("/skills/manifests", get(routes::skills::list_manifests))
        .route("/skills/manifests/:name", get(routes::skills::get_manifest))
        .route("/skills/mcp-tools", get(routes::mcp::list_mcp_tools))
        .route("/skills/import", post(routes::skills::import_capability))
        .route("/skills/register", post(routes::skills::register_capability))
        .route(
            "/skills/mcp-tools/:name/execute",
            post(routes::mcp::execute_mcp_tool),
        )
        .route(
            "/skills/scripts/:name",
            put(routes::skills::post_script),
        )
        .route(
            "/skills/scripts/:name",
            axum::routing::delete(routes::skills::delete_script),
        )
        .route(
            "/skills/workflows/:name",
            put(routes::skills::post_workflow),
        )
        .route(
            "/skills/workflows/:name",
            axum::routing::delete(routes::skills::delete_workflow),
        )
        .route(
            "/skills/hooks/:name",
            put(routes::skills::post_hook),
        )
        .route(
            "/skills/hooks/:name",
            axum::routing::delete(routes::skills::delete_hook),
        )
        .route("/benchmarks", get(routes::benchmarks::get_benchmarks))
        .route("/benchmarks", post(routes::benchmarks::create_benchmark))
        .route(
            "/benchmarks/run/:test_id",
            post(routes::benchmarks::trigger_benchmark),
        )
        .route(
            "/benchmarks/:test_id",
            get(routes::benchmarks::get_benchmark_history),
        )
        // Continuity Scheduler — /v1/continuity/jobs
        .route(
            "/continuity/jobs",
            get(routes::continuity::list_jobs_handler),
        )
        .route(
            "/continuity/jobs",
            post(routes::continuity::create_job_handler),
        )
        .route(
            "/continuity/jobs/:id",
            get(routes::continuity::get_job_handler),
        )
        .route(
            "/continuity/jobs/:id",
            put(routes::continuity::update_job_handler),
        )
        .route(
            "/continuity/jobs/:id",
            axum::routing::delete(routes::continuity::delete_job_handler),
        )
        .route(
            "/continuity/jobs/:id/runs",
            get(routes::continuity::list_job_runs_handler),
        )
        // Continuity Workflows
        .route(
            "/continuity/workflows",
            get(routes::continuity::list_workflows_handler),
        )
        .route(
            "/continuity/workflows",
            post(routes::continuity::create_workflow_handler),
        )
        .route(
            "/continuity/workflows/:id/steps",
            post(routes::continuity::add_workflow_step_handler),
        )
        .route(
            "/continuity/workflows/:id",
            axum::routing::delete(routes::continuity::delete_workflow_handler),
        )
        // Skills API
        .route(
            "/continuity/jobs/:id/enable",
            post(routes::continuity::enable_job_handler),
        )
        .route(
            "/continuity/jobs/:id/disable",
            post(routes::continuity::disable_job_handler),
        )
        // Documentation API
        .route("/docs/knowledge", get(routes::docs::list_knowledge_docs))
        .route(
            "/docs/knowledge/:category/:name",
            get(routes::docs::get_knowledge_doc),
        )
        .route(
            "/docs/operations-manual",
            get(routes::docs::get_operations_manual),
        )
        // Environment Schema API (SEC-02: returns metadata only, never secret values)
        .route("/env-schema", get(routes::env_schema::get_env_schema))
        .route_layer(axum::middleware::from_fn_with_state(
            app_state.clone(),
            middleware::auth::validate_token,
        ));

    // Engine routes — PUBLIC (no auth required)
    let engine_public = Router::new().route("/engine/health", get(routes::health::health_check));

    // Engine routes — PROTECTED (require NEURAL_TOKEN Bearer auth)
    let engine_protected = Router::new()
        .route("/engine/deploy", post(routes::deploy::trigger_deploy))
        .route("/engine/kill", post(routes::engine_control::kill_agents))
        .route(
            "/engine/shutdown",
            post(routes::engine_control::shutdown_engine),
        )
        .route("/engine/ws", get(routes::ws::ws_handler))
        .route("/engine/transcribe", post(routes::audio::transcribe_audio))
        .route("/engine/speak", post(routes::audio::text_to_speech))
        .route("/engine/templates/install", post(routes::templates::install_template))
        .route("/api/pull", post(routes::model_manager::ollama_proxy_pull))
        .route_layer(axum::middleware::from_fn_with_state(
            app_state.clone(),
            middleware::auth::validate_token,
        ));

    // Determine the static file serving path.
    let static_dir = std::env::var("STATIC_DIR").unwrap_or_else(|_| "dist".to_string());

    let mut app = Router::new()
        // Force all API traffic through versioned endpoints
        // Versioned API: /v1/agents, /v1/oversight, /v1/engine, etc.
        .nest(
            "/v1",
            protected_routes
                .clone()
                .merge(engine_public)
                .merge(engine_protected),
        )
        .fallback(not_found_handler)
        .with_state(app_state.clone())
        // Global middleware
        .layer(axum::middleware::from_fn(
            crate::middleware::auth_rate_limit::auth_brute_force_limiter,
        ))
        // X-Request-Id middleware for end-to-end tracing
        .layer(axum::middleware::from_fn(
            middleware::request_id::inject_request_id,
        ))
        // X-RateLimit-* headers for API consumer transparency
        .layer(axum::middleware::from_fn(
            middleware::rate_limit::inject_rate_limit_headers,
        ))
        // CORS must be the *outermost* layer so it runs first, before Auth
        .layer(cors);

    // Static file serving
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

async fn not_found_handler() -> impl axum::response::IntoResponse {
    let (status, body) = crate::routes::error::ProblemDetails::new(
        axum::http::StatusCode::NOT_FOUND,
        "Not Found",
        "The requested API endpoint does not exist or has been deprecated.".to_string(),
    );
    (status, body)
}

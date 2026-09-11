//! @docs ARCHITECTURE:Networking
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / router
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Type-safe state handling and bounded execution without unhandled panics.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: none
//! - **Telemetry Targets**: none declared
//! - **Witness Tests**: none declared

use crate::state::AppState;
use crate::{middleware, routes};
use axum::{
    routing::{get, post, put},
    Router,
};
use std::sync::Arc;
use tower_http::services::{ServeDir, ServeFile};

pub const DEFAULT_BODY_LIMIT_BYTES: usize = 128 * 1024;
pub const MISSION_SYNC_BODY_LIMIT_BYTES: usize = 64 * 1024;
pub const DEFAULT_TIMEOUT_SECS: u64 = 60;
pub const EXTENDED_TIMEOUT_SECS: u64 = 600;

/// Creates and configures the main Axum router.
///
/// ### 🔒 Gateway Orchestration
/// 1. **Middleware Stack**: Injects Request IDs, Rate Limits, and Auth Gates.
/// 2. **Static Assembly**: Routes `/v1` to the API and fallbacks to the React SPA
///    for client-side routing.
/// 3. **Hardware Adaptation**: Conditionally compiles vector search routes
///    based on if the `vector-memory` feature is active (ROUT-01).
pub fn create_router(app_state: Arc<AppState>) -> Router {
    // 1. Configure CORS
    let cors = middleware::cors::create_cors_layer();

    // 2. Build Protected API Routes
    let protected_routes = build_protected_v1_routes(app_state.clone());

    // 3. Build Engine Routes
    let engine_public = build_engine_public_routes();
    let engine_protected = build_engine_protected_routes(app_state.clone());

    // 4. Build public remote routes (no auth — needed for pre-pairing ping & pairing handshake)
    let remote_public = build_remote_public_routes();

    // 5. Combine all /v1 routes and attach API-specific fallback
    let v1_routes = protected_routes
        .merge(engine_public)
        .merge(engine_protected)
        .merge(remote_public)
        .fallback(not_found_handler)
        .route_layer(axum::middleware::from_fn_with_state(
            app_state.clone(),
            health_gate,
        ));

    // 6. Build Outward Gateway routes (/a2a/v1)
    let outward_state = routes::outward_routes::OutwardAppState::new("Tadpole SMB Solutions");
    let public_outward = routes::outward_routes::public_outward_router(outward_state.clone());
    let protected_outward = routes::outward_routes::protected_outward_router(outward_state)
        .route_layer(axum::middleware::from_fn_with_state(
            app_state.clone(),
            middleware::auth::validate_token,
        ));
    let outward_api = public_outward.merge(protected_outward);

    // 7. Resolve static file serving path.
    let static_dir = std::env::var("STATIC_DIR").unwrap_or_else(|_| "dist".to_string());

    // 8. Assemble root router (/v1/engine/metrics for authenticated UI/Prometheus monitoring)
    let mut app = Router::new()
        .nest("/v1", v1_routes)
        .with_state(app_state.clone())
        .merge(outward_api)
        // Inner security / headers
        .layer(axum::middleware::from_fn(
            middleware::security_headers::inject_security_headers,
        ))
        // Authentication brute-force protection
        .layer(axum::middleware::from_fn(
            crate::middleware::auth_rate_limit::auth_brute_force_limiter,
        ))
        // Rate limits
        .layer(axum::middleware::from_fn(
            middleware::rate_limit::inject_rate_limit_headers,
        ))
        // Deprecation headers
        .layer(axum::middleware::from_fn_with_state(
            app_state.clone(),
            middleware::deprecation::deprecation_middleware,
        ))
        // GAP-PERF-03: Default timeout for standard API routes.
        // LLM-bound routes (ws, agent tasks) apply their own extended timeout
        // via route-level layers in build_engine_protected_routes.
        .layer(tower_http::timeout::TimeoutLayer::with_status_code(
            axum::http::StatusCode::REQUEST_TIMEOUT,
            std::time::Duration::from_secs(DEFAULT_TIMEOUT_SECS),
        ))
        // Compression
        .layer(
            tower_http::compression::CompressionLayer::new()
                .br(true)
                .gzip(true)
                .zstd(true),
        )
        // CORS
        .layer(cors)
        // Request ID Injection (MUST wrap CORS to ensure even rejected preflights/origins have a request id)
        .layer(axum::middleware::from_fn(
            middleware::request_id::inject_request_id,
        ))
        // TRAC-03: TraceLayer should be at the base to ensure all subsequent
        // middleware have access to the request span.
        .layer(
            tower_http::trace::TraceLayer::new_for_http().make_span_with(
                |request: &axum::http::Request<axum::body::Body>| {
                    tracing::info_span!(
                        "http_request",
                        method = %request.method(),
                        uri = %request.uri(),
                        request_id = tracing::field::Empty,
                        trace_id = tracing::field::Empty,
                    )
                },
            ),
        );

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

/// Routes requiring `NEURAL_TOKEN` authentication.
///
/// ### 🛡️ Sector Sovereignty
/// Wraps all core system management endpoints (Agents, Infra, Skills) in a
/// mandatory Bearer token validation layer. 401/403 errors are handled by
/// the `ProblemDetails` RFC 9457 error pipeline.
fn build_protected_v1_routes(app_state: Arc<AppState>) -> Router<Arc<AppState>> {
    Router::new()
        .nest("/agents", build_agent_routes())
        .nest("/oversight", build_oversight_routes())
        .nest("/remote", build_remote_protected_routes())
        .nest("/infra", build_infra_routes())
        .nest("/skills", build_skills_routes())
        .nest("/benchmarks", build_benchmark_routes())
        .nest("/continuity", build_continuity_routes())
        .nest("/docs", build_docs_routes())
        .nest("/system", build_system_routes())
        .nest("/governance", build_governance_routes())
        .nest("/intelligence", build_intelligence_routes())
        .nest("/knowledge", build_knowledge_routes())
        .nest("/memory", build_memory_routes())
        .nest("/cas", build_cas_routes())
        .route("/search/memory", build_search_memory_route())
        .route("/env-schema", get(routes::env_schema::get_env_schema))
        .route_layer(axum::middleware::from_fn_with_state(
            app_state,
            middleware::auth::validate_token,
        ))
}

fn build_cas_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/history", get(routes::cas::get_cas_history_handler))
        .route("/restore", post(routes::cas::restore_cas_version_handler))
}

fn build_memory_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/search/bm25", get(routes::memory::bm25_search_handler))
        .route(
            "/graph",
            get(routes::memory::get_markdown_memory_graph_handler),
        )
}

/// Public remote routes — accessible without auth token.
/// These are limited to the connectivity check and one-time pairing handshake.
/// Pairing-token minting requires desktop bearer authentication, while every
/// post-pair companion route requires Ed25519 proof of possession (SEC-01).
fn build_remote_public_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/remote/ping", get(routes::remote::ping_remote_node))
        .route("/remote/pair", post(routes::remote::pair_device))
        .merge(build_remote_paired_routes())
        // The decision payload has its own signed canonical body because its
        // approval fields must be covered by the companion signature.
        .route(
            "/remote/oversight/decide",
            post(routes::remote::remote_decide_oversight),
        )
}

/// Remote routes requiring signed paired-device request headers.
fn build_remote_paired_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route(
            "/remote/agents/health",
            get(routes::remote::get_remote_agents_health),
        )
        .route(
            "/remote/oversight/pending",
            get(routes::remote::get_remote_pending_oversight),
        )
        .route(
            "/remote/agents/halt",
            post(routes::engine_control::kill_agents),
        )
        .route(
            "/remote/oversight/trigger-test-item",
            post(routes::remote::trigger_test_oversight_item),
        )
        .route_layer(axum::middleware::from_fn(middleware::paired_device_guard))
}

/// Protected remote routes — require valid auth token.
/// Desktop-only pairing-token minting and device administration.
fn build_remote_protected_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route(
            "/pairing-token",
            get(routes::remote::generate_pairing_token),
        )
        .route("/devices", get(routes::remote::get_paired_devices))
        .route("/devices/{id}", put(routes::remote::update_paired_device))
        .route("/revoke/{id}", post(routes::remote::revoke_device))
}

fn build_governance_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/blueprints", get(routes::governance::list_blueprints))
        .route("/blueprints", post(routes::governance::save_blueprint))
        .route(
            "/blueprints/{id}",
            axum::routing::delete(routes::governance::delete_blueprint),
        )
}

fn build_agent_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/graph", get(routes::agent::get_swarm_graph_handler))
        .route(
            "/chat/completions",
            post(routes::agent::create_chat_completion),
        )
        .route("/", get(routes::agent::get_agents))
        .route(
            "/",
            post(routes::agent::create_agent).layer(axum::extract::DefaultBodyLimit::max(
                DEFAULT_BODY_LIMIT_BYTES,
            )),
        )
        .route(
            "/{id}",
            get(routes::agent::get_agent)
                .put(routes::agent::update_agent)
                .layer(axum::extract::DefaultBodyLimit::max(
                    DEFAULT_BODY_LIMIT_BYTES,
                )),
        )
        .route(
            "/{id}/tasks",
            post(routes::agent::send_task).layer(axum::extract::DefaultBodyLimit::max(
                DEFAULT_BODY_LIMIT_BYTES,
            )),
        )
        .route("/{id}/reset", post(routes::agent::reset_agent))
        .route("/{id}/pause", post(routes::agent::pause_agent))
        .route("/{id}/resume", post(routes::agent::resume_agent))
        .route(
            "/{id}/mission",
            post(routes::agent::sync_mission).layer(axum::extract::DefaultBodyLimit::max(
                MISSION_SYNC_BODY_LIMIT_BYTES,
            )),
        )
        .route("/missions/{id}/clone", post(routes::agent::clone_mission))
        .route("/{agent_id}/memories", build_agent_memory_route())
        .route(
            "/{agent_id}/memories/{row_id}",
            build_agent_memory_delete_route(),
        )
}

fn build_oversight_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/{id}/decide", post(routes::oversight::decide_oversight))
        .route("/pending", get(routes::oversight::get_pending))
        .route("/ledger", get(routes::oversight::get_ledger))
        .route(
            "/settings",
            get(routes::oversight::get_settings).put(routes::oversight::update_settings),
        )
        .route(
            "/security/quotas",
            get(routes::oversight::get_security_quotas),
        )
        .route(
            "/security/quotas/{entity_id}",
            put(routes::oversight::update_agent_quota),
        )
        .route(
            "/security/missions/quotas",
            get(routes::oversight::get_mission_quotas),
        )
        .route(
            "/security/missions/{id}/quota",
            put(routes::oversight::update_mission_quota),
        )
        .route(
            "/security/audit-trail",
            get(routes::oversight::get_audit_trail),
        )
        .route("/security/health", get(routes::oversight::get_agent_health))
        .route(
            "/security/snapshot",
            get(routes::oversight::get_security_snapshot),
        )
        .route(
            "/security/integrity",
            get(routes::oversight::get_integrity_status),
        )
        .route("/security/policies", get(routes::oversight::get_policies))
        .route("/security/policies", put(routes::oversight::update_policy))
}

fn build_infra_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/providers", get(routes::model_manager::get_providers))
        .route(
            "/providers/{id}",
            put(routes::model_manager::update_provider),
        )
        .route(
            "/providers/{id}",
            axum::routing::delete(routes::model_manager::delete_provider),
        )
        .route(
            "/providers/{id}/test",
            post(routes::model_manager::test_provider),
        )
        .route(
            "/providers/{id}/sync",
            post(routes::model_manager::sync_provider_models),
        )
        .route("/models", get(routes::model_manager::get_models))
        .route("/models/{id}", put(routes::model_manager::update_model))
        .route(
            "/models/{id}",
            axum::routing::delete(routes::model_manager::delete_model),
        )
        .route(
            "/model-store/catalog",
            get(routes::model_manager::get_model_catalog),
        )
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
        .route(
            "/proposals/{id}/resolve",
            post(routes::skills::resolve_capability_proposal),
        )
        .route(
            "/mcp-tools/{name}/execute",
            post(routes::mcp::execute_mcp_tool),
        )
        .route("/scripts/{name}", put(routes::skills::post_script))
        .route(
            "/scripts/{name}",
            axum::routing::delete(routes::skills::delete_script),
        )
        .route("/workflows/{name}", put(routes::skills::post_workflow))
        .route(
            "/workflows/{name}",
            axum::routing::delete(routes::skills::delete_workflow),
        )
        .route("/hooks/{name}", put(routes::skills::post_hook))
        .route(
            "/hooks/{name}",
            axum::routing::delete(routes::skills::delete_hook),
        )
}

fn build_system_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/compute-profile", get(routes::system::get_compute_profile))
        .route(
            "/workspaces/status",
            get(routes::system::get_workspaces_status),
        )
        .route(
            "/workspaces/files",
            get(routes::system::get_workspace_files),
        )
        .route(
            "/environment",
            post(routes::system::update_environment_variables),
        )
}

fn build_intelligence_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/graph", get(routes::intelligence::get_code_graph))
        .route("/blast-radius", get(routes::intelligence::get_blast_radius))
}

fn build_benchmark_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(routes::benchmarks::get_benchmarks))
        .route("/", post(routes::benchmarks::create_benchmark))
        .route(
            "/run/{test_id}",
            post(routes::benchmarks::trigger_benchmark),
        )
        .route("/{test_id}", get(routes::benchmarks::get_benchmark_history))
}

fn build_continuity_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/jobs", get(routes::continuity::list_jobs_handler))
        .route("/jobs", post(routes::continuity::create_job_handler))
        .route("/jobs/{id}", get(routes::continuity::get_job_handler))
        .route("/jobs/{id}", put(routes::continuity::update_job_handler))
        .route(
            "/jobs/{id}",
            axum::routing::delete(routes::continuity::delete_job_handler),
        )
        .route(
            "/jobs/{id}/runs",
            get(routes::continuity::list_job_runs_handler),
        )
        .route(
            "/jobs/{id}/run",
            post(routes::continuity::run_job_now_handler),
        )
        .route(
            "/workflow-runs/{run_id}/steps",
            get(routes::continuity::get_workflow_run_steps_handler),
        )
        .route(
            "/workflows",
            get(routes::continuity::list_workflows_handler),
        )
        .route(
            "/workflows",
            post(routes::continuity::create_workflow_handler),
        )
        .route(
            "/workflows/{id}/steps",
            post(routes::continuity::add_workflow_step_handler),
        )
        .route(
            "/workflows/{id}/runs",
            get(routes::continuity::list_workflow_runs_handler),
        )
        .route(
            "/workflows/{id}/runs/{run_id}/cancel",
            post(routes::continuity::cancel_workflow_run_handler),
        )
        .route(
            "/workflows/{id}",
            axum::routing::delete(routes::continuity::delete_workflow_handler),
        )
        .route(
            "/jobs/{id}/enable",
            post(routes::continuity::enable_job_handler),
        )
        .route(
            "/jobs/{id}/disable",
            post(routes::continuity::disable_job_handler),
        )
}

fn build_docs_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/knowledge", get(routes::docs::list_knowledge_docs))
        .route(
            "/knowledge/{category}/{name}",
            get(routes::docs::get_knowledge_doc),
        )
        .route(
            "/operations-manual",
            get(routes::docs::get_operations_manual),
        )
}

fn build_engine_public_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/engine/health", get(routes::health::health_check))
        .route(
            "/engine/a2a/mailbox/send",
            post(routes::a2a::receive_envelope)
                .layer(axum::extract::DefaultBodyLimit::max(512 * 1024))
                .route_layer(axum::middleware::from_fn(
                    crate::middleware::auth_rate_limit::auth_brute_force_limiter,
                )),
        )
}

fn build_engine_protected_routes(app_state: Arc<AppState>) -> Router<Arc<AppState>> {
    // GAP-PERF-03: LLM-bound routes get an extended timeout to support
    // slower local model execution without hitting the global default.
    let llm_timeout_layer = tower_http::timeout::TimeoutLayer::with_status_code(
        axum::http::StatusCode::REQUEST_TIMEOUT,
        std::time::Duration::from_secs(EXTENDED_TIMEOUT_SECS),
    );

    Router::new()
        .route("/engine/deploy", post(routes::deploy::trigger_deploy))
        .route("/engine/kill", post(routes::engine_control::kill_agents))
        .route(
            "/engine/mirror/status",
            get(routes::engine_control::get_mirror_status),
        )
        .route(
            "/engine/shutdown",
            post(routes::engine_control::shutdown_engine),
        )
        .route(
            "/engine/ws",
            get(routes::ws::ws_handler).layer(llm_timeout_layer),
        )
        .route(
            "/engine/live-voice",
            get(routes::ws::live_voice_handler).layer(llm_timeout_layer),
        )
        .route(
            "/engine/transcribe",
            post(routes::audio::transcribe_audio).layer(llm_timeout_layer),
        )
        .route(
            "/engine/speak",
            post(routes::audio::text_to_speech).layer(llm_timeout_layer),
        )
        .route(
            "/engine/templates/install",
            post(routes::templates::install_template),
        )
        .route(
            "/engine/templates/import",
            post(routes::templates::import_template),
        )
        .route(
            "/engine/templates/catalog",
            get(routes::templates::get_templates_catalog),
        )
        .route(
            "/engine/templates/installed",
            get(routes::templates::get_installed_templates),
        )
        .route(
            "/engine/templates/uninstall",
            post(routes::templates::uninstall_template),
        )
        .route("/engine/metrics", get(routes::metrics::metrics_handler))
        .route(
            "/api/pull",
            post(routes::model_manager::ollama_proxy_pull).layer(llm_timeout_layer),
        )
        .route_layer(axum::middleware::from_fn_with_state(
            app_state,
            middleware::auth::validate_token,
        ))
}

// Memory feature gates
// These MethodRouters are conditionally compiled to prevent binary bloat
// and runtime panics on low-power nodes without vector-database libraries.
fn build_agent_memory_route() -> axum::routing::MethodRouter<Arc<AppState>> {
    #[cfg(feature = "vector-memory")]
    return get(routes::memory::get_agent_memory).post(routes::memory::save_agent_memory);
    #[cfg(not(feature = "vector-memory"))]
    return get(|| async {
        (
            axum::http::StatusCode::NOT_IMPLEMENTED,
            "Vector memory feature disabled",
        )
    })
    .post(|| async {
        (
            axum::http::StatusCode::NOT_IMPLEMENTED,
            "Vector memory feature disabled",
        )
    });
}

fn build_agent_memory_delete_route() -> axum::routing::MethodRouter<Arc<AppState>> {
    #[cfg(feature = "vector-memory")]
    return axum::routing::delete(routes::memory::delete_agent_memory);
    #[cfg(not(feature = "vector-memory"))]
    return axum::routing::delete(|| async {
        (
            axum::http::StatusCode::NOT_IMPLEMENTED,
            "Vector memory feature disabled",
        )
    });
}

fn build_search_memory_route() -> axum::routing::MethodRouter<Arc<AppState>> {
    #[cfg(feature = "vector-memory")]
    return get(routes::memory::global_search);
    #[cfg(not(feature = "vector-memory"))]
    return get(|| async {
        (
            axum::http::StatusCode::NOT_IMPLEMENTED,
            "Vector memory feature disabled",
        )
    });
}

fn build_knowledge_routes() -> Router<Arc<AppState>> {
    #[cfg(feature = "vector-memory")]
    return Router::new()
        .route(
            "/",
            post(routes::knowledge::write_knowledge).get(routes::knowledge::list_knowledge),
        )
        .route("/search", get(routes::knowledge::search_knowledge))
        .route("/{id}/confirm", post(routes::knowledge::confirm_knowledge))
        .route("/{id}/peers", get(routes::knowledge::get_knowledge_peers))
        .route(
            "/{id}",
            axum::routing::delete(routes::knowledge::delete_knowledge),
        );
    #[cfg(not(feature = "vector-memory"))]
    return Router::new().fallback(|| async {
        (
            axum::http::StatusCode::NOT_IMPLEMENTED,
            "Vector memory feature disabled",
        )
    });
}

async fn not_found_handler() -> impl axum::response::IntoResponse {
    crate::error::ProblemDetails::new(
        axum::http::StatusCode::NOT_FOUND,
        "Not Found",
        "The requested API endpoint does not exist or has been deprecated.".to_string(),
    )
}

async fn health_gate(
    axum::extract::State(state): axum::extract::State<Arc<AppState>>,
    req: axum::extract::Request,
    next: axum::middleware::Next,
) -> Result<axum::response::Response, crate::error::AppError> {
    let path = req.uri().path();
    if path != "/engine/health"
        && path != "/v1/engine/health"
        && path != "/metrics"
        && path != "/v1/engine/metrics"
        && state.health_state() == crate::types::SystemHealthState::Degraded
    {
        return Err(crate::error::AppError::DegradedState(
            "System is operating in a Degraded state due to a critical service failure."
                .to_string(),
        ));
    }
    Ok(next.run(req).await)
}

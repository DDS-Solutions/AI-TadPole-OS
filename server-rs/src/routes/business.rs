//! @docs ARCHITECTURE:Networking
//! ### AI Assist Note
//! Protected company-scoped business operations leave the public outward gateway unchanged.
//! ### 🔍 Debugging & Observability
//! [Business] evidence is company bound; errors follow AppError's client-safe response pipeline.

use crate::{
    agent::business::{types::*, BusinessService},
    error::AppError,
    state::AppState,
};
use axum::{
    extract::{DefaultBodyLimit, Path, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use serde::Serialize;
use std::{path::PathBuf, sync::Arc};

pub fn router(base_dir: PathBuf) -> Router<Arc<AppState>> {
    Router::new()
        .route("/companies", get(companies))
        .route("/companies/{company_key}", get(company))
        .route("/companies/{company_key}/runs", get(runs).post(create_run))
        .route("/companies/{company_key}/runs/{run_id}", get(run))
        .route(
            "/companies/{company_key}/runs/{run_id}/decision",
            post(decide_run),
        )
        .route("/companies/{company_key}/runs/{run_id}/report", get(report))
        .layer(DefaultBodyLimit::max(8192))
        .with_state(Arc::new(BusinessService::new(base_dir)))
}

#[derive(Serialize)]
pub struct Companies {
    companies: Vec<Company>,
}
#[derive(Serialize)]
pub struct Runs {
    runs: Vec<Run>,
}

/// GET /v1/business/companies
/// Lists the registered isolated practice companies. Bearer authentication is required.
/// @docs API_REFERENCE:GetBusinessCompanies
pub async fn companies(
    State(service): State<Arc<BusinessService>>,
) -> Result<Json<Companies>, AppError> {
    Ok(Json(Companies {
        companies: service.companies()?,
    }))
}

/// GET /v1/business/companies/{company_key}
/// Returns a registered company profile, offering, provenance and captured workflow configuration.
/// @docs API_REFERENCE:GetBusinessCompany
pub async fn company(
    State(service): State<Arc<BusinessService>>,
    Path(key): Path<String>,
) -> Result<Json<Company>, AppError> {
    Ok(Json(service.company(&key)?))
}

/// GET /v1/business/companies/{company_key}/runs
/// Lists up to 100 most recent runs in this company workspace.
/// @docs API_REFERENCE:GetBusinessRuns
pub async fn runs(
    State(service): State<Arc<BusinessService>>,
    Path(key): Path<String>,
) -> Result<Json<Runs>, AppError> {
    Ok(Json(Runs {
        runs: service.list_runs(&key).await?,
    }))
}

/// POST /v1/business/companies/{company_key}/runs
/// Creates a fact-sheet draft awaiting human approval. Payload-bound idempotency returns 201 for creation and 200 for replay; conflicting reuse returns 409. No model or external connector is called.
/// @docs API_REFERENCE:CreateBusinessRun
pub async fn create_run(
    State(service): State<Arc<BusinessService>>,
    Path(key): Path<String>,
    Json(request): Json<CreateRun>,
) -> Result<(StatusCode, Json<Run>), AppError> {
    let (run, created) = service.create_run(&key, request).await?;
    Ok((
        if created {
            StatusCode::CREATED
        } else {
            StatusCode::OK
        },
        Json(run),
    ))
}

/// GET /v1/business/companies/{company_key}/runs/{run_id}
/// Fetches a durable run only from its original company workspace.
/// @docs API_REFERENCE:GetBusinessRun
pub async fn run(
    State(service): State<Arc<BusinessService>>,
    Path((key, id)): Path<(String, String)>,
) -> Result<Json<Run>, AppError> {
    Ok(Json(service.run(&key, &id).await?))
}

/// POST /v1/business/companies/{company_key}/runs/{run_id}/decision
/// Approves or rejects the captured draft using expected_version. Approval rechecks source freshness; only approved fresh mock runs can atomically persist a ticket and receipt.
/// @docs API_REFERENCE:DecideBusinessRun
pub async fn decide_run(
    State(service): State<Arc<BusinessService>>,
    Path((key, id)): Path<(String, String)>,
    Json(request): Json<DecideRun>,
) -> Result<Json<Run>, AppError> {
    Ok(Json(service.decide_run(&key, &id, request).await?))
}

/// GET /v1/business/companies/{company_key}/runs/{run_id}/report
/// Independently verifies saved approval, ticket, receipt, company scope and source-backed response. Missing evidence never counts as verified completion.
/// @docs API_REFERENCE:GetBusinessRunReport
pub async fn report(
    State(service): State<Arc<BusinessService>>,
    Path((key, id)): Path<(String, String)>,
) -> Result<Json<Report>, AppError> {
    Ok(Json(service.report(&key, &id).await?))
}

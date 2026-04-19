//! Conversation Continuity — Mission-driven context API
//!
//! Provides endpoints for recurring mission scheduling, long-running
//! workflows, and job lifecycle management.
//!
//! @docs ARCHITECTURE:Networking
//!
//! ### AI Assist Note
//! **Conversation Continuity API**: Orchestrates the REST surface for
//! long-running mission context, **Job Scheduling**, and multi-step
//! **Workflow Management**. Features **Cron-Based Execution**:
//! recurring jobs rely on standard `cron` expressions for high-fidelity
//! timing. Implements **Workflow Sequencing**: coordinates the
//! execution of chained agent tasks. AI agents should verify `cron`
//! syntax and agent availability before creating/updating continuity
//! jobs to prevent runtime scheduling failures (CONT-01).
//!
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: 400 Bad Request on invalid cron expressions,
//!   404 on missing workflow/agent IDs, or job execution stalls due to
//!   budget exhaustion or runner suspension.
//! - **Trace Scope**: `server-rs::routes::continuity`

use crate::agent::continuity::{
    scheduler::{create_job, delete_job, get_job_by_id, list_jobs, list_runs_for_job, update_job},
    types::{CreateJobRequest, UpdateJobRequest},
    workflow::WorkflowEngine,
};
use crate::error::AppError;
use crate::state::AppState;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;

#[derive(Debug, Deserialize)]
pub struct CreateWorkflowRequest {
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AddStepRequest {
    pub agent_id: String,
    pub name: String,
    pub prompt_template: String,
    pub step_order: i32,
}

// ─────────────────────────────────────────────────────────
//  GET /v1/continuity/jobs
// ─────────────────────────────────────────────────────────

/// Lists all scheduled jobs.
#[tracing::instrument(skip(state), name = "continuity::list_jobs")]
pub async fn list_jobs_handler(
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, AppError> {
    let jobs = list_jobs(&state.resources.pool)
        .await
        .map_err(|e| AppError::InternalServerError(format!("Database error: {}", e)))?;
    Ok(Json(json!({ "jobs": jobs, "count": jobs.len() })))
}

// ─────────────────────────────────────────────────────────
//  POST /v1/continuity/jobs
// ─────────────────────────────────────────────────────────

/// Creates a new scheduled job.
#[tracing::instrument(skip(state, req), name = "continuity::create_job")]
pub async fn create_job_handler(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateJobRequest>,
) -> Result<impl IntoResponse, AppError> {
    // Validate agent exists (skip if it's a workflow job with an empty agent_id)
    if (!req.agent_id.is_empty() || req.workflow_id.is_none())
        && !state.registry.agents.contains_key(&req.agent_id)
    {
        return Err(AppError::NotFound(format!(
            "Agent '{}' not found",
            req.agent_id
        )));
    }

    let job = create_job(&state.resources.pool, req)
        .await
        .map_err(|e| AppError::BadRequest(e.to_string()))?;

    tracing::info!(
        "🕐 [Continuity] New job created: '{}' for agent '{}'",
        job.name,
        job.agent_id
    );
    state.emit_event(json!({
        "type": "engine:continuity_job_created",
        "data": job
    }));

    Ok((StatusCode::CREATED, Json(json!(job))))
}

// ─────────────────────────────────────────────────────────
//  GET /v1/continuity/jobs/:id
// ─────────────────────────────────────────────────────────

#[tracing::instrument(skip(state), name = "continuity::get_job")]
pub async fn get_job_handler(
    State(state): State<Arc<AppState>>,
    Path(job_id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let job = get_job_by_id(&state.resources.pool, &job_id)
        .await
        .map_err(|e| AppError::InternalServerError(format!("Database error: {}", e)))?;

    match job {
        Some(j) => Ok(Json(json!(j))),
        None => Err(AppError::NotFound("Job not found".to_string())),
    }
}

// ─────────────────────────────────────────────────────────
//  PUT /v1/continuity/jobs/:id
// ─────────────────────────────────────────────────────────

#[tracing::instrument(skip(state, req), name = "continuity::update_job")]
pub async fn update_job_handler(
    State(state): State<Arc<AppState>>,
    Path(job_id): Path<String>,
    Json(req): Json<UpdateJobRequest>,
) -> Result<impl IntoResponse, AppError> {
    let job = update_job(&state.resources.pool, &job_id, req)
        .await
        .map_err(|e| AppError::BadRequest(e.to_string()))?;
    Ok(Json(json!(job)))
}

// ─────────────────────────────────────────────────────────
//  DELETE /v1/continuity/jobs/:id
// ─────────────────────────────────────────────────────────

#[tracing::instrument(skip(state), name = "continuity::delete_job")]
pub async fn delete_job_handler(
    State(state): State<Arc<AppState>>,
    Path(job_id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    delete_job(&state.resources.pool, &job_id)
        .await
        .map_err(|e| AppError::InternalServerError(format!("Database error: {}", e)))?;
    tracing::info!("🗑 [Continuity] Job '{}' deleted.", job_id);
    Ok(StatusCode::NO_CONTENT)
}

// ─────────────────────────────────────────────────────────
//  GET /v1/continuity/jobs/:id/runs
// ─────────────────────────────────────────────────────────

#[tracing::instrument(skip(state), name = "continuity::list_job_runs")]
pub async fn list_job_runs_handler(
    State(state): State<Arc<AppState>>,
    Path(job_id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let runs = list_runs_for_job(&state.resources.pool, &job_id, 50)
        .await
        .map_err(|e| AppError::InternalServerError(format!("Database error: {}", e)))?;
    Ok(Json(json!({ "runs": runs, "count": runs.len() })))
}

// ─────────────────────────────────────────────────────────
//  POST /v1/continuity/jobs/:id/enable
//  POST /v1/continuity/jobs/:id/disable
// ─────────────────────────────────────────────────────────

#[tracing::instrument(skip(state), name = "continuity::enable_job")]
pub async fn enable_job_handler(
    State(state): State<Arc<AppState>>,
    Path(job_id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let req = UpdateJobRequest {
        name: None,
        prompt: None,
        workflow_id: None,
        cron_expr: None,
        budget_usd: None,
        enabled: Some(true),
        max_failures: None,
    };
    let job = update_job(&state.resources.pool, &job_id, req)
        .await
        .map_err(|e| AppError::BadRequest(e.to_string()))?;
    Ok(Json(json!(job)))
}

#[tracing::instrument(skip(state), name = "continuity::disable_job")]
pub async fn disable_job_handler(
    State(state): State<Arc<AppState>>,
    Path(job_id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let req = UpdateJobRequest {
        name: None,
        prompt: None,
        workflow_id: None,
        cron_expr: None,
        budget_usd: None,
        enabled: Some(false),
        max_failures: None,
    };
    let job = update_job(&state.resources.pool, &job_id, req)
        .await
        .map_err(|e| AppError::BadRequest(e.to_string()))?;
    Ok(Json(json!(job)))
}

// ─────────────────────────────────────────────────────────
//  GET /v1/continuity/workflows
// ─────────────────────────────────────────────────────────

#[tracing::instrument(skip(state), name = "continuity::list_workflows")]
pub async fn list_workflows_handler(
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, AppError> {
    let engine = WorkflowEngine::new(state);
    let workflows = engine
        .list_workflows()
        .await
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;
    Ok(Json(
        json!({ "workflows": workflows, "count": workflows.len() }),
    ))
}

// ─────────────────────────────────────────────────────────
//  POST /v1/continuity/workflows
// ─────────────────────────────────────────────────────────

#[tracing::instrument(skip(state, req), name = "continuity::create_workflow")]
pub async fn create_workflow_handler(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateWorkflowRequest>,
) -> Result<impl IntoResponse, AppError> {
    let engine = WorkflowEngine::new(state);
    let workflow = engine
        .create_workflow(req.name, req.description)
        .await
        .map_err(|e| AppError::BadRequest(e.to_string()))?;
    Ok((StatusCode::CREATED, Json(json!(workflow))))
}

// ─────────────────────────────────────────────────────────
//  POST /v1/continuity/workflows/:id/steps
// ─────────────────────────────────────────────────────────

#[tracing::instrument(skip(state, req), name = "continuity::add_workflow_step")]
pub async fn add_workflow_step_handler(
    State(state): State<Arc<AppState>>,
    Path(workflow_id): Path<String>,
    Json(req): Json<AddStepRequest>,
) -> Result<impl IntoResponse, AppError> {
    let engine = WorkflowEngine::new(state);
    let step = engine
        .add_step(
            &workflow_id,
            &req.agent_id,
            req.name,
            req.prompt_template,
            req.step_order,
        )
        .await
        .map_err(|e| AppError::BadRequest(e.to_string()))?;
    Ok((StatusCode::CREATED, Json(json!(step))))
}

// ─────────────────────────────────────────────────────────
//  DELETE /v1/continuity/workflows/:id
// ─────────────────────────────────────────────────────────

pub async fn delete_workflow_handler(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let engine = WorkflowEngine::new(state);
    engine
        .delete_workflow(&id)
        .await
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;
    Ok(StatusCode::NO_CONTENT)
}

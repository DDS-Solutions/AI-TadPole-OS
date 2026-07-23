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
//! recurring jobs rely on standard cron expressions for high-fidelity
//! timing. Implements **Workflow Sequencing**: coordinates the
//! execution of chained agent tasks. AI agents should verify cron
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
use chrono::Utc;
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;

/// Extracts tenant_id from the `X-Tenant-Id` header.
/// Falls back to "default" for backward compatibility.
fn extract_tenant_id(headers: &axum::http::HeaderMap) -> String {
    headers
        .get("X-Tenant-Id")
        .and_then(|v| v.to_str().ok())
        .filter(|s| !s.is_empty())
        .unwrap_or("default")
        .to_string()
}

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
    pub max_retries: Option<i32>,
    pub backoff_factor_secs: Option<i64>,
    pub depends_on: Option<Vec<String>>,
}

// ─────────────────────────────────────────────────────────
//  GET /v1/continuity/jobs
// ─────────────────────────────────────────────────────────

/// Lists all scheduled jobs.
#[tracing::instrument(skip(state), fields(trace_id = tracing::field::Empty, request_id = tracing::field::Empty, http_status = tracing::field::Empty), name = "continuity::list_jobs")]
pub async fn list_jobs_handler(
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, AppError> {
    let jobs = list_jobs(&state.resources.pool).await?;
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
    // Validate agent exists against the DB — the in-memory registry may be stale
    // if agents were created after server startup (CONT-02).
    // Skip validation for workflow jobs (agent_id is empty by design).
    if !req.agent_id.is_empty() {
        let agent_exists: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM agents WHERE id = ?1)",
        )
        .bind(&req.agent_id)
        .fetch_one(&state.resources.pool)
        .await
        .unwrap_or(false);

        if !agent_exists {
            return Err(AppError::NotFound(format!(
                "Agent '{}' not found",
                req.agent_id
            )));
        }
    } else if req.workflow_id.is_none() {
        // Neither agent nor workflow provided — reject
        return Err(AppError::BadRequest(
            "A scheduled job must have either an agent_id or a workflow_id".to_string(),
        ));
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

#[tracing::instrument(skip(state), fields(trace_id = tracing::field::Empty, request_id = tracing::field::Empty, http_status = tracing::field::Empty), name = "continuity::get_job")]
pub async fn get_job_handler(
    State(state): State<Arc<AppState>>,
    Path(job_id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let job = get_job_by_id(&state.resources.pool, &job_id).await?;

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

#[tracing::instrument(skip(state), fields(trace_id = tracing::field::Empty, request_id = tracing::field::Empty, http_status = tracing::field::Empty), name = "continuity::delete_job")]
pub async fn delete_job_handler(
    State(state): State<Arc<AppState>>,
    Path(job_id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    delete_job(&state.resources.pool, &job_id).await?;
    tracing::info!("🗑 [Continuity] Job '{}' deleted.", job_id);
    Ok(StatusCode::NO_CONTENT)
}

// ─────────────────────────────────────────────────────────
//  GET /v1/continuity/jobs/:id/runs
// ─────────────────────────────────────────────────────────

#[tracing::instrument(skip(state), fields(trace_id = tracing::field::Empty, request_id = tracing::field::Empty, http_status = tracing::field::Empty), name = "continuity::list_job_runs")]
pub async fn list_job_runs_handler(
    State(state): State<Arc<AppState>>,
    Path(job_id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let runs = list_runs_for_job(&state.resources.pool, &job_id, 50).await?;
    Ok(Json(json!({ "runs": runs, "count": runs.len() })))
}

// ─────────────────────────────────────────────────────────
//  POST /v1/continuity/jobs/:id/enable
//  POST /v1/continuity/jobs/:id/disable
// ─────────────────────────────────────────────────────────

#[tracing::instrument(skip(state), fields(trace_id = tracing::field::Empty, request_id = tracing::field::Empty, http_status = tracing::field::Empty), name = "continuity::enable_job")]
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

#[tracing::instrument(skip(state), fields(trace_id = tracing::field::Empty, request_id = tracing::field::Empty, http_status = tracing::field::Empty), name = "continuity::disable_job")]
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

#[tracing::instrument(skip(state, headers), name = "continuity::list_workflows")]
pub async fn list_workflows_handler(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
) -> Result<impl IntoResponse, AppError> {
    let tenant_id = extract_tenant_id(&headers);
    let engine = WorkflowEngine::new(state);
    let workflows = engine
        .list_workflows(&tenant_id)
        .await
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;
    Ok(Json(
        json!({ "workflows": workflows, "count": workflows.len() }),
    ))
}

// ─────────────────────────────────────────────────────────
//  POST /v1/continuity/workflows
// ─────────────────────────────────────────────────────────

#[tracing::instrument(skip(state, headers, req), name = "continuity::create_workflow")]
pub async fn create_workflow_handler(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Json(req): Json<CreateWorkflowRequest>,
) -> Result<impl IntoResponse, AppError> {
    let tenant_id = extract_tenant_id(&headers);
    let engine = WorkflowEngine::new(state);
    let workflow = engine
        .create_workflow(&tenant_id, req.name, req.description)
        .await
        .map_err(|e| AppError::BadRequest(e.to_string()))?;
    Ok((StatusCode::CREATED, Json(json!(workflow))))
}

// ─────────────────────────────────────────────────────────
//  POST /v1/continuity/workflows/:id/steps
// ─────────────────────────────────────────────────────────

#[tracing::instrument(skip(state, headers, req), name = "continuity::add_workflow_step")]
pub async fn add_workflow_step_handler(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Path(workflow_id): Path<String>,
    Json(req): Json<AddStepRequest>,
) -> Result<impl IntoResponse, AppError> {
    let tenant_id = extract_tenant_id(&headers);
    let engine = WorkflowEngine::new(state);
    let step = engine
        .add_step(
            &tenant_id,
            &workflow_id,
            &req.agent_id,
            req.name,
            req.prompt_template,
            req.step_order,
            req.max_retries,
            req.backoff_factor_secs,
            req.depends_on,
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
    headers: axum::http::HeaderMap,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let tenant_id = extract_tenant_id(&headers);
    let engine = WorkflowEngine::new(state);
    engine
        .delete_workflow(&tenant_id, &id)
        .await
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Debug, serde::Serialize)]
pub struct WorkflowStepRunResponse {
    pub id: String,
    pub run_id: String,
    pub step_id: String,
    pub started_at: String,
    pub completed_at: Option<String>,
    pub status: String,
    pub output_text: Option<String>,
    pub cost_usd: f64,
    pub metadata: Option<serde_json::Value>,
    pub step_name: String,
    pub agent_id: String,
    pub step_order: i32,
    pub step_config: Option<serde_json::Value>,
}

#[tracing::instrument(skip(state, headers), name = "continuity::get_workflow_run_steps")]
pub async fn get_workflow_run_steps_handler(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Path(run_id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let tenant_id = extract_tenant_id(&headers);
    let repo =
        crate::agent::continuity::repository::WorkflowRepository::new(state.resources.pool.clone());
    repo.verify_run_owner(&run_id, &tenant_id).await?;

    let rows = sqlx::query(
        "SELECT \
            wsr.id, \
            wsr.run_id, \
            wsr.step_id, \
            wsr.started_at, \
            wsr.completed_at, \
            wsr.status, \
            wsr.output_text, \
            wsr.cost_usd, \
            wsr.metadata, \
            ws.name as step_name, \
            ws.agent_id, \
            ws.step_order, \
            ws.config as step_config \
         FROM workflow_step_runs wsr \
         JOIN workflow_steps ws ON wsr.step_id = ws.id \
         WHERE wsr.run_id = ?1 \
         ORDER BY ws.step_order ASC, wsr.started_at ASC",
    )
    .bind(&run_id)
    .fetch_all(&state.resources.pool)
    .await?;

    let mut step_runs = Vec::new();
    for row in rows {
        use sqlx::Row;
        let metadata_str: Option<String> = row.get("metadata");
        let step_config_str: Option<String> = row.get("step_config");
        let completed_at_dt: Option<chrono::DateTime<Utc>> = row.get("completed_at");
        let started_at_dt: chrono::DateTime<Utc> = row.get("started_at");

        step_runs.push(WorkflowStepRunResponse {
            id: row.get("id"),
            run_id: row.get("run_id"),
            step_id: row.get("step_id"),
            started_at: started_at_dt.to_rfc3339(),
            completed_at: completed_at_dt.map(|dt| dt.to_rfc3339()),
            status: row.get("status"),
            output_text: row.get("output_text"),
            cost_usd: row.get("cost_usd"),
            metadata: metadata_str.and_then(|s| serde_json::from_str(&s).ok()),
            step_name: row.get("step_name"),
            agent_id: row.get("agent_id"),
            step_order: row.get("step_order"),
            step_config: step_config_str.and_then(|s| serde_json::from_str(&s).ok()),
        });
    }

    Ok(Json(
        json!({ "step_runs": step_runs, "count": step_runs.len() }),
    ))
}

#[tracing::instrument(skip(state, headers), name = "continuity::list_workflow_runs")]
pub async fn list_workflow_runs_handler(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Path(workflow_id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let tenant_id = extract_tenant_id(&headers);
    let engine = WorkflowEngine::new(state);
    let runs = engine
        .list_workflow_runs(&tenant_id, &workflow_id, 50)
        .await?;
    Ok(Json(json!({ "runs": runs, "count": runs.len() })))
}

#[tracing::instrument(skip(state, headers), name = "continuity::cancel_workflow_run")]
pub async fn cancel_workflow_run_handler(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Path((_workflow_id, run_id)): Path<(String, String)>,
) -> Result<impl IntoResponse, AppError> {
    let tenant_id = extract_tenant_id(&headers);
    let engine = WorkflowEngine::new(state);
    engine.cancel_run(&tenant_id, &run_id).await?;
    Ok(StatusCode::OK)
}

#[tracing::instrument(skip(state), fields(trace_id = tracing::field::Empty, request_id = tracing::field::Empty, http_status = tracing::field::Empty), name = "continuity::run_job_now")]
pub async fn run_job_now_handler(
    State(state): State<Arc<AppState>>,
    Path(job_id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let job = get_job_by_id(&state.resources.pool, &job_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Job not found".to_string()))?;

    let state_clone = Arc::clone(&state);
    tokio::spawn(async move {
        crate::agent::continuity::executor::execute_job(state_clone, job).await;
    });

    Ok(StatusCode::ACCEPTED)
}

// Metadata: [continuity]

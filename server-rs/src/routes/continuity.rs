//! @docs ARCHITECTURE:Networking
//!
//! ### AI Context Alignment
//! - **Subsystem**: Sovereign Engine / HTTP Routes / continuity
//!
//! ### ⚠️ Invariants & Non-Negotiables
//! - `[Structural]` Type-safe state handling and bounded execution without unhandled panics.
//!
//! ### 🔍 Debugging & Observability
//! - **Local Errors**: `AppError::BadRequest`, `AppError::NotFound`, `AppError::Sqlx`, `AppError::InternalServerError`
//! - **Telemetry Targets**: `[Continuity]`
//! - **Witness Tests**: `continuity::tests::test_tenant_sanitization`, `continuity::tests::test_create_job_db_propagation`

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
/// Sanitizes to alphanumeric/dash/underscore (max 64 chars) and falls back to "default".
fn extract_tenant_id(headers: &axum::http::HeaderMap) -> String {
    headers
        .get("X-Tenant-Id")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.trim())
        .filter(|s| {
            !s.is_empty()
                && s.len() <= 64
                && s.chars()
                    .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
        })
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
#[tracing::instrument(skip(state), name = "continuity::list_jobs")]
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
        let agent_exists: bool =
            sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM agents WHERE id = ?1)")
                .bind(&req.agent_id)
                .fetch_one(&state.resources.pool)
                .await
                .map_err(AppError::Sqlx)?;

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

    if let Some(ref wf_id) = req.workflow_id {
        let wf_exists: bool =
            sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM workflows WHERE id = ?1)")
                .bind(wf_id)
                .fetch_one(&state.resources.pool)
                .await
                .map_err(AppError::Sqlx)?;

        if !wf_exists {
            return Err(AppError::NotFound(format!(
                "Workflow '{}' not found",
                wf_id
            )));
        }
    }

    let job = create_job(&state.resources.pool, req).await?;

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
        enabled: Some(true),
        ..Default::default()
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
        enabled: Some(false),
        ..Default::default()
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
    if req.step_order < 0 {
        return Err(AppError::BadRequest(
            "step_order must be non-negative".into(),
        ));
    }
    if let Some(retries) = req.max_retries {
        if retries < 0 {
            return Err(AppError::BadRequest(
                "max_retries must be non-negative".into(),
            ));
        }
    }
    if let Some(backoff) = req.backoff_factor_secs {
        if backoff < 0 {
            return Err(AppError::BadRequest(
                "backoff_factor_secs must be non-negative".into(),
            ));
        }
    }

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

#[tracing::instrument(skip(state, headers), name = "continuity::delete_workflow")]
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

        let parsed_metadata = metadata_str.and_then(|s| match serde_json::from_str(&s) {
            Ok(v) => Some(v),
            Err(e) => {
                tracing::warn!(
                    "⚠️ [Continuity] Failed to deserialize step run metadata JSON: {}",
                    e
                );
                None
            }
        });

        let parsed_config = step_config_str.and_then(|s| match serde_json::from_str(&s) {
            Ok(v) => Some(v),
            Err(e) => {
                tracing::warn!(
                    "⚠️ [Continuity] Failed to deserialize step config JSON: {}",
                    e
                );
                None
            }
        });

        step_runs.push(WorkflowStepRunResponse {
            id: row.get("id"),
            run_id: row.get("run_id"),
            step_id: row.get("step_id"),
            started_at: started_at_dt.to_rfc3339(),
            completed_at: completed_at_dt.map(|dt| dt.to_rfc3339()),
            status: row.get("status"),
            output_text: row.get("output_text"),
            cost_usd: row.get("cost_usd"),
            metadata: parsed_metadata,
            step_name: row.get("step_name"),
            agent_id: row.get("agent_id"),
            step_order: row.get("step_order"),
            step_config: parsed_config,
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
    Path((workflow_id, run_id)): Path<(String, String)>,
) -> Result<impl IntoResponse, AppError> {
    let tenant_id = extract_tenant_id(&headers);

    let run_wf_id: Option<String> = sqlx::query_scalar(
        "SELECT workflow_id FROM workflow_runs WHERE id = ?1 AND tenant_id = ?2",
    )
    .bind(&run_id)
    .bind(&tenant_id)
    .fetch_optional(&state.resources.pool)
    .await
    .map_err(AppError::Sqlx)?;

    match run_wf_id {
        Some(wf_id) if wf_id == workflow_id => {}
        Some(_) => {
            return Err(AppError::BadRequest(
                "Run ID does not match specified workflow ID".into(),
            ))
        }
        None => return Err(AppError::NotFound("Workflow run not found".into())),
    }

    let engine = WorkflowEngine::new(state);
    engine.cancel_run(&tenant_id, &run_id).await?;
    Ok(StatusCode::OK)
}

#[tracing::instrument(skip(state), name = "continuity::run_job_now")]
pub async fn run_job_now_handler(
    State(state): State<Arc<AppState>>,
    Path(job_id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let job = get_job_by_id(&state.resources.pool, &job_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Job not found".to_string()))?;

    if !job.enabled {
        return Err(AppError::Conflict("Scheduled job is disabled".to_string()));
    }

    state.emit_event(json!({
        "type": "continuity:job_triggered",
        "job_id": job.id,
        "agent_id": job.agent_id,
        "timestamp": chrono::Utc::now().to_rfc3339()
    }));

    let state_clone = Arc::clone(&state);
    tokio::spawn(async move {
        crate::agent::continuity::executor::execute_job(state_clone, job).await;
    });

    Ok((
        StatusCode::ACCEPTED,
        Json(json!({
            "status": "accepted",
            "job_id": job_id,
            "message": "Job execution dispatched"
        })),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::HeaderMap;

    #[test]
    fn test_tenant_sanitization() {
        let mut headers = HeaderMap::new();
        assert_eq!(extract_tenant_id(&headers), "default");

        headers.insert("X-Tenant-Id", "tenant-123".parse().unwrap());
        assert_eq!(extract_tenant_id(&headers), "tenant-123");

        let mut bad_headers = HeaderMap::new();
        bad_headers.insert("X-Tenant-Id", "tenant/../evil".parse().unwrap());
        assert_eq!(extract_tenant_id(&bad_headers), "default");
    }
}

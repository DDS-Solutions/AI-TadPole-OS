//! @docs ARCHITECTURE:Persistence
//!
//! ### AI Assist Note
//! **Swarm Persistence**: Manages the lifecycle of inter-agent directives and 
//! peer reviews. Ensures **Atomic Coordination** (SWARM-01) by persisting 
//! mission-critical delegation and audit state to SQLite. Features **Cross-Agent 
//! Visibility** to enable the hierarchical and decentralized swarm loops.
//!
//! ### 🔍 Debugging & Observability
//! - **Failure Path**: SQL constraints violation during directive insertion, 
//!   orphaned review requests, or mission_id mismatch during cross-agent 
//!   retrieval.
//! - **Telemetry Link**: Search `[swarm_persistence]` in tracing logs.
//!

use sqlx::SqlitePool;
use uuid::Uuid;
use crate::agent::runner::RunContext;
use crate::error::AppError;

#[derive(sqlx::FromRow)]
#[allow(dead_code)]
pub(crate) struct AgentDirective {
    pub id: String,
    pub mission_id: String,
    pub source_agent_id: String,
    pub target_agent_id: String,
    pub instruction: String,
    pub status: String,
    pub result: Option<String>,
}

#[derive(sqlx::FromRow)]
#[allow(dead_code)]
pub(crate) struct PeerReviewRequest {
    pub id: String,
    pub mission_id: String,
    pub requester_id: String,
    pub reviewer_id: String,
    pub content_to_review: String,
    pub criteria: Option<String>,
    pub status: String,
}

/// Saves a new mission directive to the database.
pub async fn save_directive(
    pool: &SqlitePool,
    ctx: &RunContext,
    target_agent_id: &str,
    instruction: &str,
) -> Result<String, AppError> {
    let id = Uuid::new_v4().to_string();
    sqlx::query(
        "INSERT INTO agent_directives (id, mission_id, source_agent_id, target_agent_id, instruction, status) 
         VALUES (?, ?, ?, ?, ?, 'pending')"
    )
    .bind(&id)
    .bind(&ctx.mission_id)
    .bind(&ctx.agent_id)
    .bind(target_agent_id)
    .bind(instruction)
    .execute(pool)
    .await
    .map_err(AppError::Sqlx)?;
    
    Ok(id)
}

/// Retrieves all pending directives for a specific agent.
pub async fn get_pending_directives(
    pool: &SqlitePool,
    agent_id: &str,
) -> Result<Vec<AgentDirective>, AppError> {
    let rows = sqlx::query_as::<_, AgentDirective>(
        "SELECT id, mission_id, source_agent_id, target_agent_id, instruction, status, result 
         FROM agent_directives 
         WHERE target_agent_id = ? AND status = 'pending'
         ORDER BY created_at ASC",
    )
    .bind(agent_id)
    .fetch_all(pool)
    .await
    .map_err(AppError::Sqlx)?;
    
    Ok(rows)
}

/// Updates the status of a directive.
#[allow(dead_code)]
pub async fn update_directive_status(
    pool: &SqlitePool,
    id: &str,
    status: &str,
    result: Option<&str>,
) -> Result<(), AppError> {
    sqlx::query(
        "UPDATE agent_directives SET status = ?, result = ? WHERE id = ?"
    )
    .bind(status)
    .bind(result)
    .bind(id)
    .execute(pool)
    .await
    .map_err(AppError::Sqlx)?;
    
    Ok(())
}

/// Submits a peer review result.
pub async fn submit_review(
    pool: &SqlitePool,
    id: &str,
    feedback: &str,
    status: &str,
) -> Result<(), AppError> {
    sqlx::query(
        "UPDATE peer_reviews SET status = ?, feedback = ? WHERE id = ?"
    )
    .bind(status)
    .bind(feedback)
    .bind(id)
    .execute(pool)
    .await
    .map_err(AppError::Sqlx)?;
    
    Ok(())
}

/// Submits a peer review request.
pub async fn save_review_request(
    pool: &SqlitePool,
    ctx: &RunContext,
    reviewer_id: &str,
    content: &str,
    criteria: Option<&str>,
) -> Result<String, AppError> {
    let id = Uuid::new_v4().to_string();
    sqlx::query(
        "INSERT INTO peer_reviews (id, mission_id, requester_id, reviewer_id, content_to_review, criteria, status) 
         VALUES (?, ?, ?, ?, ?, ?, 'requested')"
    )
    .bind(&id)
    .bind(&ctx.mission_id)
    .bind(&ctx.agent_id)
    .bind(reviewer_id)
    .bind(content)
    .bind(criteria)
    .execute(pool)
    .await
    .map_err(AppError::Sqlx)?;
    
    Ok(id)
}

/// Retrieves all review requests for a specific reviewer.
pub async fn get_pending_reviews(
    pool: &SqlitePool,
    reviewer_id: &str,
) -> Result<Vec<PeerReviewRequest>, AppError> {
    let rows = sqlx::query_as::<_, PeerReviewRequest>(
        "SELECT id, mission_id, requester_id, reviewer_id, content_to_review, criteria, status 
         FROM peer_reviews 
         WHERE reviewer_id = ? AND status = 'requested'
         ORDER BY created_at ASC",
    )
    .bind(reviewer_id)
    .fetch_all(pool)
    .await
    .map_err(AppError::Sqlx)?;
    
    Ok(rows)
}

// Metadata: [swarm_persistence]
